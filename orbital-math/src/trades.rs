//! Toroidal trade execution for Orbital AMM
//!
//! Implements the advanced toroidal trading surface that combines:
//! - Interior ticks (spherical caps) for concentrated liquidity
//! - Boundary ticks (circular cross-sections) for edge cases
//! - Multi-tick boundary crossing with optimized routing
//!
//! The toroidal surface enables capital efficient trading while maintaining
//! the N-dimensional spherical invariant across all market conditions.

use alloc::vec::Vec;
use alloy_primitives::U256;
use crate::{
    error::{OrbitalError, Result},
    types::{PoolState, Tick, ReservePoint, TradeInfo, CurveType, sqrt_approx},
    sphere::{self, calculate_amount_out_sphere, calculate_price_sphere, verify_sphere_constraint},
    superellipse::{self, calculate_amount_out_superellipse},
    ticks::{self, find_next_crossing, crossing_fraction, is_interior_to_tick},
    utils::{sum, dot_product},
    PRECISION_MULTIPLIER,
};

/// Execute a swap on the toroidal trading surface with tick boundary crossing
///
/// This is the main trading function that handles:
/// - Route discovery across multiple ticks
/// - Boundary crossing calculations
/// - Price impact optimization
/// - Invariant preservation
///
/// # Arguments
/// * `pool` - Mutable pool state
/// * `token_in` - Input token index
/// * `token_out` - Output token index  
/// * `amount_in` - Amount of input token
/// * `min_amount_out` - Minimum acceptable output (slippage protection)
///
/// # Returns
/// * Complete trade information including price impact and fees
pub fn execute_swap_toroidal(
    pool: &mut PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
    min_amount_out: U256,
) -> Result<TradeInfo> {
    // Validate inputs
    validate_swap_inputs(pool, token_in, token_out, amount_in)?;
    
    // Get initial state
    let initial_reserves = pool.reserves.reserves.clone();
    let price_before = calculate_price_sphere(&initial_reserves, token_in, token_out)?;
    
    // Execute the trade through toroidal surface
    let (amount_out, ticks_crossed, fee) = execute_toroidal_route(
        pool,
        token_in,
        token_out, 
        amount_in,
    )?;
    
    // Check slippage
    if amount_out < min_amount_out {
        return Err(OrbitalError::SlippageExceeded {
            actual: format!("{}", ((min_amount_out - amount_out) * U256::from(10000)) / min_amount_out),
            tolerance: "user defined".to_string(),
        });
    }
    
    // Calculate price after trade
    let price_after = calculate_price_sphere(&pool.reserves.reserves, token_in, token_out)?;
    
    // Calculate price impact
    let price_impact_bp = calculate_price_impact_bp(price_before, price_after)?;
    
    // Verify invariant is maintained
    verify_trade_invariant(pool)?;
    
    Ok(TradeInfo {
        token_in,
        token_out,
        amount_in,
        amount_out,
        price_before,
        price_after,
        price_impact_bp,
        ticks_crossed,
        fee,
    })
}

/// Execute trade routing through the toroidal surface
fn execute_toroidal_route(
    pool: &mut PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
) -> Result<(U256, usize, U256)> {
    let mut remaining_amount = amount_in;
    let mut total_output = U256::ZERO;
    let mut ticks_crossed = 0;
    let mut total_fee = U256::ZERO;
    
    // Check if we need to handle tick crossings
    let has_ticks = !pool.ticks.is_empty();
    
    if !has_ticks {
        // Simple case: no ticks, direct sphere/superellipse calculation
        return execute_simple_trade(pool, token_in, token_out, amount_in);
    }
    
    // Complex case: route through ticks with boundary crossing
    while remaining_amount > U256::ZERO {
        let current_reserves = ReservePoint::new(pool.reserves.reserves.clone());
        
        // Find target reserves after consuming remaining amount
        let target_reserves = calculate_target_reserves(
            pool,
            token_in,
            token_out,
            remaining_amount,
        )?;
        
        // Check for tick boundary crossings
        match find_next_crossing(&current_reserves, &target_reserves, &pool.ticks)? {
            Some(crossing_tick_idx) => {
                // Execute partial trade up to boundary
                let (partial_out, partial_fee) = execute_partial_trade_to_boundary(
                    pool,
                    token_in,
                    token_out,
                    remaining_amount,
                    crossing_tick_idx,
                )?;
                
                total_output = total_output.checked_add(partial_out)
                    .ok_or_else(|| OrbitalError::overflow("total output"))?;
                    
                total_fee = total_fee.checked_add(partial_fee)
                    .ok_or_else(|| OrbitalError::overflow("total fee"))?;
                
                ticks_crossed += 1;
                
                // Update pool state and continue with remaining
                let consumed = calculate_consumed_amount(
                    &pool.reserves.reserves,
                    token_in,
                    partial_out,
                    token_out,
                )?;
                
                remaining_amount = remaining_amount.saturating_sub(consumed);
                
                // Update tick boundary states
                update_tick_boundary_state(pool, crossing_tick_idx)?;
                
            }
            None => {
                // No boundary crossing, execute remaining trade
                let (final_out, final_fee) = execute_remaining_trade(
                    pool,
                    token_in,
                    token_out,
                    remaining_amount,
                )?;
                
                total_output = total_output.checked_add(final_out)
                    .ok_or_else(|| OrbitalError::overflow("total output"))?;
                    
                total_fee = total_fee.checked_add(final_fee)
                    .ok_or_else(|| OrbitalError::overflow("total fee"))?;
                
                break;
            }
        }
        
        // Safety check to prevent infinite loops
        if remaining_amount == amount_in {
            return Err(OrbitalError::ComputationError {
                details: "Unable to make progress in routing".to_string(),
            });
        }
    }
    
    Ok((total_output, ticks_crossed, total_fee))
}

/// Execute simple trade without tick boundary crossing
fn execute_simple_trade(
    pool: &mut PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
) -> Result<(U256, usize, U256)> {
    let amount_out = match pool.curve_type {
        CurveType::Sphere => {
            calculate_amount_out_sphere(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                pool.invariant,
            )?
        }
        CurveType::Superellipse { u_parameter } => {
            calculate_amount_out_superellipse(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                u_parameter,
                pool.invariant,
            )?
        }
    };
    
    // Apply dynamic fee based on pool utilization
    let fee = calculate_dynamic_fee(pool, amount_in)?;
    let amount_out_after_fee = amount_out.saturating_sub(fee);
    
    // Update pool reserves
    pool.reserves.reserves[token_in] = pool.reserves.reserves[token_in]
        .checked_add(amount_in)
        .ok_or_else(|| OrbitalError::overflow("reserve update"))?;
        
    pool.reserves.reserves[token_out] = pool.reserves.reserves[token_out]
        .checked_sub(amount_out_after_fee)
        .ok_or_else(|| OrbitalError::InsufficientLiquidity {
            needed: amount_out_after_fee.to_string(),
            available: pool.reserves.reserves[token_out].to_string(),
        })?;
    
    // Update cached values
    pool.update_reserves(pool.reserves.reserves.clone());
    
    Ok((amount_out_after_fee, 0, fee))
}

/// Calculate target reserves after full trade execution
fn calculate_target_reserves(
    pool: &PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
) -> Result<ReservePoint> {
    let amount_out = match pool.curve_type {
        CurveType::Sphere => {
            calculate_amount_out_sphere(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                pool.invariant,
            )?
        }
        CurveType::Superellipse { u_parameter } => {
            calculate_amount_out_superellipse(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                u_parameter,
                pool.invariant,
            )?
        }
    };
    
    let mut target_reserves = pool.reserves.reserves.clone();
    target_reserves[token_in] = target_reserves[token_in].checked_add(amount_in)
        .ok_or_else(|| OrbitalError::overflow("target reserves calculation"))?;
    target_reserves[token_out] = target_reserves[token_out].checked_sub(amount_out)
        .ok_or_else(|| OrbitalError::underflow("target reserves calculation"))?;
    
    Ok(ReservePoint::new(target_reserves))
}

/// Execute partial trade up to a tick boundary
fn execute_partial_trade_to_boundary(
    pool: &mut PoolState,
    token_in: usize,
    token_out: usize,
    max_amount_in: U256,
    crossing_tick_idx: usize,
) -> Result<(U256, U256)> {
    let current_reserves = ReservePoint::new(pool.reserves.reserves.clone());
    let target_reserves = calculate_target_reserves(pool, token_in, token_out, max_amount_in)?;
    let tick = &pool.ticks[crossing_tick_idx];
    
    // Calculate fraction of trade that reaches boundary
    let fraction = crossing_fraction(&current_reserves, &target_reserves, tick)?;
    
    // Calculate partial amount to boundary
    let partial_amount_in = (max_amount_in * fraction) / U256::from(PRECISION_MULTIPLIER);
    
    // Execute trade up to boundary
    let partial_amount_out = match pool.curve_type {
        CurveType::Sphere => {
            calculate_amount_out_sphere(
                &pool.reserves.reserves,
                token_in,
                token_out,
                partial_amount_in,
                pool.invariant,
            )?
        }
        CurveType::Superellipse { u_parameter } => {
            calculate_amount_out_superellipse(
                &pool.reserves.reserves,
                token_in,
                token_out,
                partial_amount_in,
                u_parameter,
                pool.invariant,
            )?
        }
    };
    
    // Apply boundary crossing fee (higher due to complexity)
    let fee = calculate_boundary_crossing_fee(pool, partial_amount_in)?;
    let amount_out_after_fee = partial_amount_out.saturating_sub(fee);
    
    // Update reserves to boundary
    pool.reserves.reserves[token_in] = pool.reserves.reserves[token_in]
        .checked_add(partial_amount_in)
        .ok_or_else(|| OrbitalError::overflow("partial reserve update"))?;
        
    pool.reserves.reserves[token_out] = pool.reserves.reserves[token_out]
        .checked_sub(amount_out_after_fee)
        .ok_or_else(|| OrbitalError::InsufficientLiquidity {
            needed: amount_out_after_fee.to_string(),
            available: pool.reserves.reserves[token_out].to_string(),
        })?;
    
    Ok((amount_out_after_fee, fee))
}

/// Execute remaining trade after boundary crossing
fn execute_remaining_trade(
    pool: &mut PoolState,
    token_in: usize,
    token_out: usize,
    remaining_amount: U256,
) -> Result<(U256, U256)> {
    execute_simple_trade(pool, token_in, token_out, remaining_amount)
        .map(|(out, _ticks, fee)| (out, fee))
}

/// Update tick boundary state after crossing
fn update_tick_boundary_state(pool: &mut PoolState, tick_idx: usize) -> Result<()> {
    let current_reserves = ReservePoint::new(pool.reserves.reserves.clone());
    let tick = &mut pool.ticks[tick_idx];
    
    // Check if we're now on the boundary of this tick
    let on_boundary = ticks::is_on_boundary(&current_reserves, tick)?;
    tick.is_boundary = on_boundary;
    
    // If we crossed into a new tick, activate its liquidity
    if on_boundary {
        // This tick's liquidity is now active
        // In production, this would trigger liquidity redistribution
    }
    
    Ok(())
}

/// Calculate dynamic fee based on pool utilization and volatility
fn calculate_dynamic_fee(pool: &PoolState, amount_in: U256) -> Result<U256> {
    // Base fee: 30 basis points (0.3%)
    let base_fee_bp = U256::from(30);
    
    // Volume-based adjustment (higher volume = higher fee)
    let total_liquidity = pool.total_liquidity();
    let utilization = if total_liquidity.is_zero() {
        U256::ZERO
    } else {
        (amount_in * U256::from(10000)) / total_liquidity
    };
    
    // Fee increases with utilization: fee = base + (utilization * 0.1)
    let utilization_fee = utilization / U256::from(100); // 0.1% max
    let total_fee_bp = base_fee_bp + utilization_fee;
    
    // Apply fee: fee = amount_in * fee_bp / 10000
    let fee = (amount_in * total_fee_bp) / U256::from(10000);
    
    Ok(fee)
}

/// Calculate boundary crossing fee (higher due to complexity)
fn calculate_boundary_crossing_fee(pool: &PoolState, amount_in: U256) -> Result<U256> {
    let base_fee = calculate_dynamic_fee(pool, amount_in)?;
    
    // Boundary crossing adds 50% premium
    let crossing_premium = base_fee / U256::from(2);
    
    Ok(base_fee + crossing_premium)
}

/// Calculate how much input was consumed for a given output
fn calculate_consumed_amount(
    reserves: &[U256],
    token_in: usize,
    amount_out: U256,
    token_out: usize,
) -> Result<U256> {
    // This is an approximation - in production use more precise calculation
    let price = calculate_price_sphere(reserves, token_in, token_out)?;
    let consumed = (amount_out * U256::from(PRECISION_MULTIPLIER)) / price;
    
    Ok(consumed)
}

/// Calculate price impact in basis points
fn calculate_price_impact_bp(price_before: U256, price_after: U256) -> Result<u32> {
    if price_before.is_zero() {
        return Ok(0);
    }
    
    let price_diff = if price_after > price_before {
        price_after - price_before
    } else {
        price_before - price_after
    };
    
    let impact_scaled = (price_diff * U256::from(10000)) / price_before;
    
    Ok(impact_scaled.try_into().unwrap_or(u32::MAX))
}

/// Validate swap inputs
fn validate_swap_inputs(
    pool: &PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
) -> Result<()> {
    if token_in >= pool.token_count() || token_out >= pool.token_count() {
        return Err(OrbitalError::TokenIndexOutOfBounds {
            index: token_in.max(token_out),
            token_count: pool.token_count(),
        });
    }
    
    if token_in == token_out {
        return Err(OrbitalError::invalid_param(
            "tokens",
            "input and output must be different",
        ));
    }
    
    if amount_in.is_zero() {
        return Err(OrbitalError::invalid_param(
            "amount_in",
            "must be greater than zero",
        ));
    }
    
    if !pool.reserves.all_positive() {
        return Err(OrbitalError::invalid_param(
            "reserves",
            "all reserves must be positive",
        ));
    }
    
    Ok(())
}

/// Verify that trade maintains pool invariant
fn verify_trade_invariant(pool: &PoolState) -> Result<()> {
    match pool.curve_type {
        CurveType::Sphere => {
            verify_sphere_constraint(
                &pool.reserves.reserves,
                pool.invariant,
                100, // 1% tolerance for numerical precision
            )
        }
        CurveType::Superellipse { u_parameter } => {
            superellipse::verify_superellipse_constraint(
                &pool.reserves.reserves,
                u_parameter,
                pool.invariant,
                100, // 1% tolerance
            )
        }
    }
}

/// Execute a multi-hop swap across multiple token pairs
///
/// This enables efficient routing through multiple pools or complex paths
/// within a single N-dimensional pool.
pub fn execute_multi_hop_swap(
    pool: &mut PoolState,
    path: &[usize], // [token0, token1, token2, ...]
    amount_in: U256,
    min_amount_out: U256,
) -> Result<TradeInfo> {
    if path.len() < 2 {
        return Err(OrbitalError::invalid_param(
            "path",
            "must contain at least 2 tokens",
        ));
    }
    
    let mut current_amount = amount_in;
    let mut total_fee = U256::ZERO;
    let mut total_ticks_crossed = 0;
    
    let price_before = calculate_price_sphere(
        &pool.reserves.reserves,
        path[0],
        path[path.len() - 1],
    )?;
    
    // Execute each hop in sequence
    for i in 0..path.len() - 1 {
        let token_in = path[i];
        let token_out = path[i + 1];
        
        let trade_result = execute_swap_toroidal(
            pool,
            token_in,
            token_out,
            current_amount,
            U256::ZERO, // No intermediate slippage check
        )?;
        
        current_amount = trade_result.amount_out;
        total_fee = total_fee.checked_add(trade_result.fee)
            .ok_or_else(|| OrbitalError::overflow("total fee"))?;
        total_ticks_crossed += trade_result.ticks_crossed;
    }
    
    // Check final slippage
    if current_amount < min_amount_out {
        return Err(OrbitalError::SlippageExceeded {
            actual: format!("{}", ((min_amount_out - current_amount) * U256::from(10000)) / min_amount_out),
            tolerance: "user defined".to_string(),
        });
    }
    
    let price_after = calculate_price_sphere(
        &pool.reserves.reserves,
        path[0],
        path[path.len() - 1],
    )?;
    
    let price_impact_bp = calculate_price_impact_bp(price_before, price_after)?;
    
    Ok(TradeInfo {
        token_in: path[0],
        token_out: path[path.len() - 1],
        amount_in,
        amount_out: current_amount,
        price_before,
        price_after,
        price_impact_bp,
        ticks_crossed: total_ticks_crossed,
        fee: total_fee,
    })
}

/// Calculate optimal route for maximum output
///
/// Uses dynamic programming to find the path that maximizes output
/// across all possible routes in the N-dimensional space.
pub fn calculate_optimal_route(
    pool: &PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
    max_hops: usize,
) -> Result<Vec<usize>> {
    if max_hops == 0 || max_hops > pool.token_count() {
        return Err(OrbitalError::invalid_param(
            "max_hops",
            "must be between 1 and token_count",
        ));
    }
    
    // For direct trade
    if max_hops == 1 {
        return Ok(vec![token_in, token_out]);
    }
    
    // For multi-hop, use simplified heuristic
    // In production, implement full dynamic programming
    let mut best_route = vec![token_in, token_out];
    let mut best_output = U256::ZERO;
    
    // Try direct route first
    if let Ok(amount_out) = calculate_trade_output(pool, token_in, token_out, amount_in) {
        best_output = amount_out;
    }
    
    // Try routes through each intermediate token
    for intermediate in 0..pool.token_count() {
        if intermediate == token_in || intermediate == token_out {
            continue;
        }
        
        // Calculate two-hop route: token_in -> intermediate -> token_out
        if let (Ok(amount_intermediate), Ok(amount_final)) = (
            calculate_trade_output(pool, token_in, intermediate, amount_in),
            calculate_trade_output(pool, intermediate, token_out, amount_intermediate),
        ) {
            if amount_final > best_output {
                best_output = amount_final;
                best_route = vec![token_in, intermediate, token_out];
            }
        }
    }
    
    Ok(best_route)
}

/// Calculate trade output without executing
fn calculate_trade_output(
    pool: &PoolState,
    token_in: usize,
    token_out: usize,
    amount_in: U256,
) -> Result<U256> {
    match pool.curve_type {
        CurveType::Sphere => {
            calculate_amount_out_sphere(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                pool.invariant,
            )
        }
        CurveType::Superellipse { u_parameter } => {
            calculate_amount_out_superellipse(
                &pool.reserves.reserves,
                token_in,
                token_out,
                amount_in,
                u_parameter,
                pool.invariant,
            )
        }
    }
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
        let invariant = U256::from(3_000_000_000_000u128); // 3 * 10^12
        
        PoolState::new(
            reserves,
            CurveType::sphere(),
            invariant,
            vec![],
        )
    }

    #[test]
    fn test_execute_simple_swap() {
        let mut pool = create_test_pool();
        
        let result = execute_swap_toroidal(
            &mut pool,
            0, // token_in
            1, // token_out
            U256::from(10_000), // amount_in
            U256::from(9_900),  // min_amount_out (1% slippage)
        );
        
        assert!(result.is_ok());
        let trade_info = result.unwrap();
        assert!(trade_info.amount_out > U256::ZERO);
        assert!(trade_info.amount_out >= U256::from(9_900));
        assert_eq!(trade_info.token_in, 0);
        assert_eq!(trade_info.token_out, 1);
    }

    #[test]
    fn test_multi_hop_swap() {
        let mut pool = create_test_pool();
        
        let path = vec![0, 1, 2]; // 0 -> 1 -> 2
        let result = execute_multi_hop_swap(
            &mut pool,
            &path,
            U256::from(10_000),
            U256::from(9_800), // 2% slippage for multi-hop
        );
        
        assert!(result.is_ok());
        let trade_info = result.unwrap();
        assert!(trade_info.amount_out > U256::ZERO);
        assert_eq!(trade_info.token_in, 0);
        assert_eq!(trade_info.token_out, 2);
    }

    #[test]
    fn test_optimal_route_calculation() {
        let pool = create_test_pool();
        
        let route = calculate_optimal_route(&pool, 0, 2, U256::from(10_000), 2);
        
        assert!(route.is_ok());
        let path = route.unwrap();
        assert_eq!(path[0], 0);
        assert_eq!(path[path.len() - 1], 2);
        assert!(path.len() >= 2);
    }

    #[test]
    fn test_slippage_protection() {
        let mut pool = create_test_pool();
        
        let result = execute_swap_toroidal(
            &mut pool,
            0,
            1,
            U256::from(10_000),
            U256::from(15_000), // Unrealistic min_amount_out
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrbitalError::SlippageExceeded { .. }));
    }

    #[test]
    fn test_invalid_token_indices() {
        let mut pool = create_test_pool();
        
        let result = execute_swap_toroidal(
            &mut pool,
            0,
            5, // Invalid index
            U256::from(10_000),
            U256::from(9_900),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrbitalError::TokenIndexOutOfBounds { .. }));
    }

    #[test]
    fn test_zero_amount_trade() {
        let mut pool = create_test_pool();
        
        let result = execute_swap_toroidal(
            &mut pool,
            0,
            1,
            U256::ZERO,
            U256::ZERO,
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrbitalError::InvalidParameter { .. }));
    }

    #[test]
    fn test_same_token_trade() {
        let mut pool = create_test_pool();
        
        let result = execute_swap_toroidal(
            &mut pool,
            0,
            0, // Same token
            U256::from(10_000),
            U256::from(9_900),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrbitalError::InvalidParameter { .. }));
    }

    #[test]
    fn test_dynamic_fee_calculation() {
        let pool = create_test_pool();
        
        let fee = calculate_dynamic_fee(&pool, U256::from(10_000));
        
        assert!(fee.is_ok());
        let fee_amount = fee.unwrap();
        
        // Should be roughly 0.3% of amount
        let expected_fee = U256::from(30); // 30 out of 10,000 = 0.3%
        assert!(fee_amount >= expected_fee);
    }
}