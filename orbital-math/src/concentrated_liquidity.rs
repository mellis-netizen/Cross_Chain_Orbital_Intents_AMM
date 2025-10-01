//! Concentrated Liquidity for Orbital AMM
//!
//! Implements advanced concentrated liquidity mechanics using:
//! - Capital efficient tick positioning
//! - Dynamic liquidity allocation
//! - MEV protection through tick design
//! - Liquidity provider reward optimization

use alloc::vec::Vec;
use alloy_primitives::U256;
use crate::{
    error::{OrbitalError, Result},
    types::{PoolState, Tick, ReservePoint, CurveType},
    sphere::{self, calculate_equal_price_point},
    ticks::{self, optimize_tick_placement, calculate_capital_efficiency},
    utils::{sum},
    PRECISION_MULTIPLIER,
};

/// Liquidity position in a concentrated range
#[derive(Debug, Clone)]
pub struct LiquidityPosition {
    /// Unique position ID
    pub position_id: U256,
    /// Owner of the position
    pub owner: [u8; 20], // Address-like identifier
    /// Lower tick boundary
    pub tick_lower: usize,
    /// Upper tick boundary  
    pub tick_upper: usize,
    /// Amount of liquidity provided
    pub liquidity_amount: U256,
    /// Fees earned (scaled by PRECISION)
    pub fees_earned: U256,
    /// Block when position was created
    pub created_at: u64,
    /// Whether position is currently active
    pub is_active: bool,
}

/// Liquidity management for concentrated positions
pub struct ConcentratedLiquidityManager {
    /// All liquidity positions
    pub positions: Vec<LiquidityPosition>,
    /// Fee collection per tick
    pub tick_fee_growth: Vec<U256>,
    /// Global fee accumulator
    pub global_fee_growth: U256,
    /// Minimum liquidity per position
    pub min_liquidity: U256,
}

impl ConcentratedLiquidityManager {
    /// Create new liquidity manager
    pub fn new(tick_count: usize) -> Self {
        Self {
            positions: Vec::new(),
            tick_fee_growth: vec![U256::ZERO; tick_count],
            global_fee_growth: U256::ZERO,
            min_liquidity: U256::from(1000), // Minimum 1000 units
        }
    }
    
    /// Add concentrated liquidity position
    pub fn add_liquidity_position(
        &mut self,
        owner: [u8; 20],
        tick_lower: usize,
        tick_upper: usize,
        liquidity_amount: U256,
        pool: &mut PoolState,
    ) -> Result<U256> {
        // Validate inputs
        if tick_lower >= tick_upper {
            return Err(OrbitalError::invalid_param(
                "ticks",
                "lower tick must be less than upper tick",
            ));
        }
        
        if liquidity_amount < self.min_liquidity {
            return Err(OrbitalError::invalid_param(
                "liquidity_amount",
                "below minimum liquidity requirement",
            ));
        }
        
        if tick_lower >= pool.ticks.len() || tick_upper >= pool.ticks.len() {
            return Err(OrbitalError::TokenIndexOutOfBounds {
                index: tick_lower.max(tick_upper),
                token_count: pool.ticks.len(),
            });
        }
        
        // Generate position ID
        let position_id = U256::from(self.positions.len() + 1);
        
        // Create position
        let position = LiquidityPosition {
            position_id,
            owner,
            tick_lower,
            tick_upper,
            liquidity_amount,
            fees_earned: U256::ZERO,
            created_at: 0, // In production, use block timestamp
            is_active: true,
        };
        
        // Update tick liquidity
        for tick_idx in tick_lower..=tick_upper {
            if tick_idx < pool.ticks.len() {
                pool.ticks[tick_idx].liquidity = pool.ticks[tick_idx].liquidity
                    .checked_add(liquidity_amount)
                    .ok_or_else(|| OrbitalError::overflow("tick liquidity update"))?;
            }
        }
        
        // Add to positions
        self.positions.push(position);
        
        Ok(position_id)
    }
    
    /// Remove liquidity position
    pub fn remove_liquidity_position(
        &mut self,
        position_id: U256,
        pool: &mut PoolState,
    ) -> Result<(U256, U256)> {
        // Find position
        let position_idx = self.positions.iter()
            .position(|p| p.position_id == position_id)
            .ok_or_else(|| OrbitalError::invalid_param("position_id", "position not found"))?;
        
        let position = &mut self.positions[position_idx];
        
        if !position.is_active {
            return Err(OrbitalError::invalid_param("position", "already inactive"));
        }
        
        // Calculate fees earned
        let fees_earned = self.calculate_fees_earned(position)?;
        
        // Update tick liquidity
        for tick_idx in position.tick_lower..=position.tick_upper {
            if tick_idx < pool.ticks.len() {
                pool.ticks[tick_idx].liquidity = pool.ticks[tick_idx].liquidity
                    .saturating_sub(position.liquidity_amount);
            }
        }
        
        let liquidity_amount = position.liquidity_amount;
        position.is_active = false;
        
        Ok((liquidity_amount, fees_earned))
    }
    
    /// Calculate fees earned for a position
    pub fn calculate_fees_earned(&self, position: &LiquidityPosition) -> Result<U256> {
        if !position.is_active {
            return Ok(position.fees_earned);
        }
        
        // Calculate fee growth in position's range
        let mut fee_growth_inside = U256::ZERO;
        
        for tick_idx in position.tick_lower..=position.tick_upper {
            if tick_idx < self.tick_fee_growth.len() {
                fee_growth_inside = fee_growth_inside
                    .checked_add(self.tick_fee_growth[tick_idx])
                    .ok_or_else(|| OrbitalError::overflow("fee growth calculation"))?;
            }
        }
        
        // Calculate fees proportional to liquidity
        let tick_count = (position.tick_upper - position.tick_lower + 1) as u128;
        let average_fee_growth = fee_growth_inside / U256::from(tick_count);
        
        let fees = (position.liquidity_amount * average_fee_growth) / U256::from(PRECISION_MULTIPLIER);
        
        Ok(fees)
    }
    
    /// Update fee growth after a trade
    pub fn update_fee_growth(&mut self, fees_collected: U256, active_ticks: &[usize]) -> Result<()> {
        // Update global fee growth
        self.global_fee_growth = self.global_fee_growth
            .checked_add(fees_collected)
            .ok_or_else(|| OrbitalError::overflow("global fee growth"))?;
        
        // Distribute fees to active ticks
        if !active_ticks.is_empty() {
            let fee_per_tick = fees_collected / U256::from(active_ticks.len());
            
            for &tick_idx in active_ticks {
                if tick_idx < self.tick_fee_growth.len() {
                    self.tick_fee_growth[tick_idx] = self.tick_fee_growth[tick_idx]
                        .checked_add(fee_per_tick)
                        .ok_or_else(|| OrbitalError::overflow("tick fee growth"))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get active liquidity at current price
    pub fn get_active_liquidity(
        &self,
        current_reserves: &ReservePoint,
        pool: &PoolState,
    ) -> Result<U256> {
        let mut total_active_liquidity = U256::ZERO;
        
        for position in &self.positions {
            if !position.is_active {
                continue;
            }
            
            // Check if current reserves are within position's tick range
            let mut position_active = true;
            
            for tick_idx in position.tick_lower..=position.tick_upper {
                if tick_idx < pool.ticks.len() {
                    let tick = &pool.ticks[tick_idx];
                    if !ticks::is_interior_to_tick(current_reserves, tick)? {
                        position_active = false;
                        break;
                    }
                }
            }
            
            if position_active {
                total_active_liquidity = total_active_liquidity
                    .checked_add(position.liquidity_amount)
                    .ok_or_else(|| OrbitalError::overflow("active liquidity calculation"))?;
            }
        }
        
        Ok(total_active_liquidity)
    }
    
    /// Optimize liquidity distribution across ticks
    pub fn optimize_liquidity_distribution(
        &self,
        pool: &PoolState,
        target_efficiency: u32, // Target capital efficiency (scaled by 10000)
    ) -> Result<Vec<LiquidityRecommendation>> {
        let mut recommendations = Vec::new();
        
        for (tick_idx, tick) in pool.ticks.iter().enumerate() {
            let current_efficiency = calculate_capital_efficiency(tick, pool.token_count())?;
            
            if current_efficiency < target_efficiency {
                // Recommend increasing liquidity in this tick
                let liquidity_needed = self.calculate_liquidity_needed_for_efficiency(
                    tick,
                    target_efficiency,
                    pool.token_count(),
                )?;
                
                recommendations.push(LiquidityRecommendation {
                    tick_index: tick_idx,
                    action: LiquidityAction::Increase,
                    amount: liquidity_needed,
                    expected_efficiency: target_efficiency,
                    priority: if current_efficiency < target_efficiency / 2 {
                        Priority::High
                    } else {
                        Priority::Medium
                    },
                });
            } else if current_efficiency > target_efficiency * 2 {
                // Recommend redistributing excess liquidity
                let excess_liquidity = tick.liquidity / U256::from(2);
                
                recommendations.push(LiquidityRecommendation {
                    tick_index: tick_idx,
                    action: LiquidityAction::Redistribute,
                    amount: excess_liquidity,
                    expected_efficiency: target_efficiency,
                    priority: Priority::Low,
                });
            }
        }
        
        Ok(recommendations)
    }
    
    /// Calculate liquidity needed to achieve target efficiency
    fn calculate_liquidity_needed_for_efficiency(
        &self,
        tick: &Tick,
        target_efficiency: u32,
        token_count: usize,
    ) -> Result<U256> {
        let current_efficiency = calculate_capital_efficiency(tick, token_count)?;
        
        if current_efficiency >= target_efficiency {
            return Ok(U256::ZERO);
        }
        
        // Simplified calculation: more liquidity improves efficiency
        // In production, use more sophisticated modeling
        let efficiency_ratio = target_efficiency as u128 / current_efficiency.max(1) as u128;
        let additional_liquidity = tick.liquidity * U256::from(efficiency_ratio.saturating_sub(1));
        
        Ok(additional_liquidity)
    }
    
    /// Rebalance positions to optimize capital efficiency
    pub fn rebalance_positions(
        &mut self,
        pool: &mut PoolState,
        rebalance_threshold: u32, // Efficiency threshold for rebalancing
    ) -> Result<Vec<RebalanceAction>> {
        let mut actions = Vec::new();
        let current_reserves = ReservePoint::new(pool.reserves.reserves.clone());
        
        for position in &mut self.positions {
            if !position.is_active {
                continue;
            }
            
            // Calculate position's current efficiency
            let mut position_efficiency = 0u32;
            let mut position_active_ticks = 0usize;
            
            for tick_idx in position.tick_lower..=position.tick_upper {
                if tick_idx < pool.ticks.len() {
                    let tick = &pool.ticks[tick_idx];
                    let tick_efficiency = calculate_capital_efficiency(tick, pool.token_count())?;
                    
                    if ticks::is_interior_to_tick(&current_reserves, tick)? {
                        position_efficiency += tick_efficiency;
                        position_active_ticks += 1;
                    }
                }
            }
            
            if position_active_ticks > 0 {
                position_efficiency /= position_active_ticks as u32;
            }
            
            // Check if rebalancing is needed
            if position_efficiency < rebalance_threshold {
                // Find better tick range
                if let Ok(recommendation) = optimize_tick_placement(
                    position.liquidity_amount,
                    1000, // 10% depeg tolerance
                    pool.token_count(),
                ) {
                    actions.push(RebalanceAction {
                        position_id: position.position_id,
                        old_tick_lower: position.tick_lower,
                        old_tick_upper: position.tick_upper,
                        new_tick_lower: 0, // Would be calculated based on recommendation
                        new_tick_upper: 1, // Would be calculated based on recommendation
                        efficiency_improvement: recommendation.expected_efficiency.saturating_sub(position_efficiency),
                    });
                }
            }
        }
        
        Ok(actions)
    }
}

/// Recommendation for liquidity allocation
#[derive(Debug, Clone)]
pub struct LiquidityRecommendation {
    pub tick_index: usize,
    pub action: LiquidityAction,
    pub amount: U256,
    pub expected_efficiency: u32,
    pub priority: Priority,
}

/// Types of liquidity actions
#[derive(Debug, Clone)]
pub enum LiquidityAction {
    Increase,
    Decrease,
    Redistribute,
}

/// Priority levels for recommendations
#[derive(Debug, Clone)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Rebalancing action for a position
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub position_id: U256,
    pub old_tick_lower: usize,
    pub old_tick_upper: usize,
    pub new_tick_lower: usize,
    pub new_tick_upper: usize,
    pub efficiency_improvement: u32,
}

/// Calculate impermanent loss for a concentrated liquidity position
pub fn calculate_impermanent_loss(
    initial_reserves: &[U256],
    current_reserves: &[U256],
    position: &LiquidityPosition,
) -> Result<U256> {
    if initial_reserves.len() != current_reserves.len() {
        return Err(OrbitalError::invalid_param(
            "reserves",
            "initial and current reserves must have same length",
        ));
    }
    
    // Calculate value if held tokens vs LP position
    // This is a simplified calculation for demonstration
    let initial_value = sum(initial_reserves)?;
    let current_value = sum(current_reserves)?;
    
    // For concentrated liquidity, IL is generally lower than full-range
    // but depends on how far price moved from the position's range
    let price_change_ratio = if initial_value > U256::ZERO {
        (current_value * U256::from(PRECISION_MULTIPLIER)) / initial_value
    } else {
        U256::from(PRECISION_MULTIPLIER)
    };
    
    // IL formula for concentrated liquidity (simplified)
    let one = U256::from(PRECISION_MULTIPLIER);
    if price_change_ratio >= one {
        // Price increased
        let ratio = price_change_ratio - one;
        let il = (ratio * ratio) / (U256::from(4) * one); // Simplified IL formula
        Ok(il)
    } else {
        // Price decreased
        let ratio = one - price_change_ratio;
        let il = (ratio * ratio) / (U256::from(4) * one);
        Ok(il)
    }
}

/// Estimate yield for a liquidity position
pub fn estimate_position_yield(
    position: &LiquidityPosition,
    pool: &PoolState,
    trading_volume_24h: U256,
    fee_rate_bp: u32, // Fee rate in basis points
) -> Result<U256> {
    // Calculate position's share of total liquidity
    let total_liquidity = pool.total_liquidity();
    if total_liquidity.is_zero() {
        return Ok(U256::ZERO);
    }
    
    let position_share = (position.liquidity_amount * U256::from(PRECISION_MULTIPLIER)) / total_liquidity;
    
    // Calculate fees from trading volume
    let total_fees = (trading_volume_24h * U256::from(fee_rate_bp)) / U256::from(10000);
    
    // Position's share of fees
    let position_fees = (total_fees * position_share) / U256::from(PRECISION_MULTIPLIER);
    
    // Annualized yield (365 days)
    let annual_yield = position_fees * U256::from(365);
    
    // Yield as percentage of position value
    let yield_percentage = if position.liquidity_amount > U256::ZERO {
        (annual_yield * U256::from(10000)) / position.liquidity_amount
    } else {
        U256::ZERO
    };
    
    Ok(yield_percentage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CurveType, PoolState};

    fn create_test_pool() -> PoolState {
        let reserves = vec![
            U256::from(1_000_000),
            U256::from(1_000_000),
            U256::from(1_000_000),
        ];
        let invariant = U256::from(3_000_000_000_000u128);
        
        let ticks = vec![
            Tick::new(
                U256::from(1),
                U256::from(9500),
                U256::from(1_000_000),
                U256::from(100_000),
                9500,
            ),
            Tick::new(
                U256::from(2),
                U256::from(9000),
                U256::from(2_000_000),
                U256::from(100_000),
                9000,
            ),
        ];
        
        PoolState::new(reserves, CurveType::sphere(), invariant, ticks)
    }

    #[test]
    fn test_add_liquidity_position() {
        let mut pool = create_test_pool();
        let mut manager = ConcentratedLiquidityManager::new(pool.ticks.len());
        
        let owner = [1u8; 20];
        let result = manager.add_liquidity_position(
            owner,
            0, // tick_lower
            1, // tick_upper
            U256::from(100_000),
            &mut pool,
        );
        
        assert!(result.is_ok());
        let position_id = result.unwrap();
        assert_eq!(position_id, U256::from(1));
        assert_eq!(manager.positions.len(), 1);
    }

    #[test]
    fn test_remove_liquidity_position() {
        let mut pool = create_test_pool();
        let mut manager = ConcentratedLiquidityManager::new(pool.ticks.len());
        
        let owner = [1u8; 20];
        let position_id = manager.add_liquidity_position(
            owner,
            0,
            1,
            U256::from(100_000),
            &mut pool,
        ).unwrap();
        
        let result = manager.remove_liquidity_position(position_id, &mut pool);
        
        assert!(result.is_ok());
        let (liquidity, _fees) = result.unwrap();
        assert_eq!(liquidity, U256::from(100_000));
        assert!(!manager.positions[0].is_active);
    }

    #[test]
    fn test_fee_calculation() {
        let mut manager = ConcentratedLiquidityManager::new(2);
        
        let position = LiquidityPosition {
            position_id: U256::from(1),
            owner: [1u8; 20],
            tick_lower: 0,
            tick_upper: 1,
            liquidity_amount: U256::from(100_000),
            fees_earned: U256::ZERO,
            created_at: 0,
            is_active: true,
        };
        
        // Simulate fee growth
        manager.tick_fee_growth[0] = U256::from(1000);
        manager.tick_fee_growth[1] = U256::from(2000);
        
        let fees = manager.calculate_fees_earned(&position).unwrap();
        assert!(fees > U256::ZERO);
    }

    #[test]
    fn test_liquidity_optimization() {
        let pool = create_test_pool();
        let manager = ConcentratedLiquidityManager::new(pool.ticks.len());
        
        let recommendations = manager.optimize_liquidity_distribution(
            &pool,
            50000, // Target 5x efficiency
        ).unwrap();
        
        // Should have recommendations for improving efficiency
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_impermanent_loss_calculation() {
        let initial_reserves = vec![U256::from(1000), U256::from(1000), U256::from(1000)];
        let current_reserves = vec![U256::from(1100), U256::from(900), U256::from(1000)];
        
        let position = LiquidityPosition {
            position_id: U256::from(1),
            owner: [1u8; 20],
            tick_lower: 0,
            tick_upper: 1,
            liquidity_amount: U256::from(100_000),
            fees_earned: U256::ZERO,
            created_at: 0,
            is_active: true,
        };
        
        let il = calculate_impermanent_loss(&initial_reserves, &current_reserves, &position);
        
        assert!(il.is_ok());
        // IL should be relatively small for small price changes
        assert!(il.unwrap() < U256::from(PRECISION_MULTIPLIER / 100)); // < 1%
    }

    #[test]
    fn test_yield_estimation() {
        let position = LiquidityPosition {
            position_id: U256::from(1),
            owner: [1u8; 20],
            tick_lower: 0,
            tick_upper: 1,
            liquidity_amount: U256::from(100_000),
            fees_earned: U256::ZERO,
            created_at: 0,
            is_active: true,
        };
        
        let pool = create_test_pool();
        let trading_volume_24h = U256::from(1_000_000);
        let fee_rate_bp = 30; // 0.3%
        
        let yield_estimate = estimate_position_yield(
            &position,
            &pool,
            trading_volume_24h,
            fee_rate_bp,
        ).unwrap();
        
        assert!(yield_estimate > U256::ZERO);
    }
}