//! Tick geometry and concentrated liquidity
//!
//! Implements hyperplane boundaries and spherical cap calculations for
//! concentrated liquidity provision.
//!
//! Ticks in Orbital are nested spherical caps defined by hyperplane boundaries.
//! Unlike Uniswap V3 where ticks are disjoint, Orbital ticks overlap with
//! larger ticks containing smaller ones.

use alloc::vec::Vec;
use alloy_primitives::U256;
use crate::{
    error::{OrbitalError, Result},
    types::{Tick, ReservePoint, sqrt_approx},
    utils::{dot_product, sum},
};

/// Check if a reserve point is interior to a tick (not on boundary)
///
/// # Arguments
/// * `reserves` - Current reserve point
/// * `tick` - Tick to check against
///
/// # Returns
/// * `true` if reserves are interior (not on boundary)
/// * `false` if reserves are on or outside the boundary
///
/// # Algorithm
/// A point is interior if: r⃗ · 1⃗ < c * sqrt(N)
/// where c is the plane constant and N is number of tokens
pub fn is_interior_to_tick(reserves: &ReservePoint, tick: &Tick) -> Result<bool> {
    let n = U256::from(reserves.dimensions());
    
    // Calculate r⃗ · 1⃗ = Σr_i
    let dot_with_ones = sum(&reserves.reserves)?;
    
    // Calculate c * sqrt(N)
    let sqrt_n = sqrt_approx(n);
    let boundary = tick.plane_constant
        .checked_mul(sqrt_n)
        .ok_or_else(|| OrbitalError::overflow("boundary calculation"))?;
    
    // Interior if dot product < boundary
    Ok(dot_with_ones < boundary)
}

/// Check if reserves are exactly on the tick boundary
pub fn is_on_boundary(reserves: &ReservePoint, tick: &Tick) -> Result<bool> {
    let n = U256::from(reserves.dimensions());
    let dot_with_ones = sum(&reserves.reserves)?;
    let sqrt_n = sqrt_approx(n);
    let boundary = tick.plane_constant
        .checked_mul(sqrt_n)
        .ok_or_else(|| OrbitalError::overflow("boundary calculation"))?;
    
    // Check if within small tolerance (1 basis point)
    let tolerance = boundary / U256::from(10000);
    let diff = if dot_with_ones > boundary {
        dot_with_ones - boundary
    } else {
        boundary - dot_with_ones
    };
    
    Ok(diff <= tolerance)
}

/// Calculate normalized position on sphere for tick comparison
///
/// # Arguments
/// * `reserves` - Reserve point
/// * `radius` - Sphere radius
///
/// # Returns
/// * Normalized position value (distance from equal price point)
///
/// # Algorithm
/// Normalized position = (r⃗ · 1⃗) / (R * sqrt(N))
/// This allows comparing positions across ticks of different sizes
pub fn normalized_position(reserves: &ReservePoint, radius: U256) -> Result<U256> {
    let n = U256::from(reserves.dimensions());
    let sqrt_n = sqrt_approx(n);
    
    let dot_with_ones = sum(&reserves.reserves)?;
    
    let denominator = radius
        .checked_mul(sqrt_n)
        .ok_or_else(|| OrbitalError::overflow("normalized position"))?;
    
    if denominator.is_zero() {
        return Err(OrbitalError::division_by_zero("normalized position"));
    }
    
    // Scale by precision to maintain accuracy
    let scaled_dot = dot_with_ones
        .checked_mul(U256::from(crate::PRECISION_MULTIPLIER))
        .ok_or_else(|| OrbitalError::overflow("normalized position"))?;
    
    Ok(scaled_dot / denominator)
}

/// Find the next tick boundary that will be crossed during a trade
///
/// # Arguments
/// * `start` - Starting reserve point
/// * `end` - Ending reserve point
/// * `ticks` - All ticks in the pool
///
/// # Returns
/// * Index of the tick that will be crossed, or None if no crossing
pub fn find_next_crossing(
    start: &ReservePoint,
    end: &ReservePoint,
    ticks: &[Tick],
) -> Result<Option<usize>> {
    if ticks.is_empty() {
        return Ok(None);
    }
    
    let n = U256::from(start.dimensions());
    let sqrt_n = sqrt_approx(n);
    
    let start_dot = sum(&start.reserves)?;
    let end_dot = sum(&end.reserves)?;
    
    // Find tick boundaries between start and end
    for (idx, tick) in ticks.iter().enumerate() {
        let boundary = tick.plane_constant
            .checked_mul(sqrt_n)
            .ok_or_else(|| OrbitalError::overflow("boundary calculation"))?;
        
        // Check if we cross this boundary
        let crosses = (start_dot <= boundary && end_dot > boundary) ||
                     (start_dot >= boundary && end_dot < boundary);
        
        if crosses {
            return Ok(Some(idx));
        }
    }
    
    Ok(None)
}

/// Calculate at what fraction of the trade a tick crossing occurs
///
/// # Returns
/// * Value between 0.0 and 1.0 indicating where crossing happens
pub fn crossing_fraction(
    start: &ReservePoint,
    end: &ReservePoint,
    tick: &Tick,
) -> Result<U256> {
    let n = U256::from(start.dimensions());
    let sqrt_n = sqrt_approx(n);
    
    let start_dot = sum(&start.reserves)?;
    let end_dot = sum(&end.reserves)?;
    let boundary = tick.plane_constant
        .checked_mul(sqrt_n)
        .ok_or_else(|| OrbitalError::overflow("boundary calculation"))?;
    
    // Linear interpolation: boundary = start + t * (end - start)
    // Solve for t: t = (boundary - start) / (end - start)
    
    if end_dot == start_dot {
        return Ok(U256::ZERO); // No crossing
    }
    
    let numerator = if boundary > start_dot {
        boundary - start_dot
    } else {
        start_dot - boundary
    };
    
    let denominator = if end_dot > start_dot {
        end_dot - start_dot
    } else {
        start_dot - end_dot
    };
    
    if denominator.is_zero() {
        return Ok(U256::ZERO);
    }
    
    // Scale by precision for accuracy
    let fraction = numerator
        .checked_mul(U256::from(crate::PRECISION_MULTIPLIER))
        .ok_or_else(|| OrbitalError::overflow("crossing fraction"))?
        .checked_div(denominator)
        .ok_or_else(|| OrbitalError::division_by_zero("crossing fraction"))?;
    
    Ok(fraction)
}

/// Calculate capital efficiency for a tick configuration
///
/// # Returns
/// * Efficiency multiplier vs full-range (scaled by 10000)
pub fn calculate_capital_efficiency(
    tick: &Tick,
    token_count: usize,
) -> Result<u32> {
    let min_reserve = tick.min_reserve(token_count);
    let max_reserve = tick.max_reserve(token_count);
    
    if min_reserve.is_zero() {
        return Ok(10000); // 1x efficiency (no concentration)
    }
    
    if max_reserve.is_zero() {
        return Ok(10000);
    }
    
    // Efficiency = max / (max - min)
    let range = max_reserve.saturating_sub(min_reserve);
    
    if range.is_zero() {
        return Ok(10000);
    }
    
    let efficiency_scaled = max_reserve
        .checked_mul(U256::from(10000))
        .ok_or_else(|| OrbitalError::overflow("efficiency calculation"))?
        .checked_div(range)
        .ok_or_else(|| OrbitalError::division_by_zero("efficiency calculation"))?;
    
    // Cap at reasonable maximum (500x)
    let efficiency_u32: u32 = efficiency_scaled.try_into().unwrap_or(u32::MAX);
    Ok(efficiency_u32.min(5_000_000))
}

/// Optimize tick placement for a given liquidity amount and risk tolerance
///
/// # Arguments
/// * `total_liquidity` - Total liquidity to deploy
/// * `depeg_tolerance_bp` - Maximum acceptable depeg in basis points
/// * `token_count` - Number of tokens in pool
///
/// # Returns
/// * Recommended tick configuration
pub fn optimize_tick_placement(
    total_liquidity: U256,
    depeg_tolerance_bp: u32,
    token_count: usize,
) -> Result<TickRecommendation> {
    // Conservative: 99% limit for tight concentration
    // Moderate: 95% limit for balanced risk/reward
    // Aggressive: 90% limit for maximum efficiency
    
    let (depeg_limit, description) = if depeg_tolerance_bp <= 100 {
        (9900, "Ultra tight - High efficiency, high risk")
    } else if depeg_tolerance_bp <= 500 {
        (9500, "Tight - Good efficiency, moderate risk")
    } else if depeg_tolerance_bp <= 1000 {
        (9000, "Moderate - Balanced efficiency and risk")
    } else {
        (8500, "Wide - Lower efficiency, lower risk")
    };
    
    // Calculate expected efficiency
    let mock_tick = Tick::new(
        U256::ZERO,
        U256::from(depeg_limit),
        total_liquidity,
        U256::from(10000),
        depeg_limit,
    );
    
    let efficiency = calculate_capital_efficiency(&mock_tick, token_count)?;
    
    Ok(TickRecommendation {
        depeg_limit,
        expected_efficiency: efficiency,
        description: description.into(),
        recommended_liquidity: total_liquidity,
    })
}

/// Sort ticks by their boundary distance from equal price point
///
/// Returns ticks sorted from innermost (closest to equal price) to outermost
pub fn sort_ticks_by_boundary(ticks: &mut [Tick]) {
    ticks.sort_by(|a, b| a.plane_constant.cmp(&b.plane_constant));
}

/// Merge overlapping ticks with similar boundaries
///
/// Combines ticks that are within tolerance of each other
pub fn merge_similar_ticks(ticks: &[Tick], tolerance_bp: u32) -> Result<Vec<Tick>> {
    if ticks.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut sorted = ticks.to_vec();
    sort_ticks_by_boundary(&mut sorted);
    
    let mut merged = Vec::new();
    let mut current = sorted[0].clone();
    
    for tick in sorted.iter().skip(1) {
        let diff = if tick.plane_constant > current.plane_constant {
            tick.plane_constant - current.plane_constant
        } else {
            current.plane_constant - tick.plane_constant
        };
        
        let tolerance = (current.plane_constant * U256::from(tolerance_bp)) 
            / U256::from(10000);
        
        if diff <= tolerance {
            // Merge: combine liquidity
            current.liquidity = current.liquidity
                .checked_add(tick.liquidity)
                .ok_or_else(|| OrbitalError::overflow("merge liquidity"))?;
        } else {
            // Different tick, save current and start new
            merged.push(current);
            current = tick.clone();
        }
    }
    
    merged.push(current);
    Ok(merged)
}

/// Calculate the total liquidity active at a given reserve point
pub fn active_liquidity_at_point(
    reserves: &ReservePoint,
    ticks: &[Tick],
) -> Result<U256> {
    let mut total_liquidity = U256::ZERO;
    
    for tick in ticks {
        // A tick is active if the point is interior to it
        if is_interior_to_tick(reserves, tick)? || is_on_boundary(reserves, tick)? {
            total_liquidity = total_liquidity
                .checked_add(tick.liquidity)
                .ok_or_else(|| OrbitalError::overflow("active liquidity"))?;
        }
    }
    
    Ok(total_liquidity)
}

/// Recommendation for tick configuration
pub struct TickRecommendation {
    /// Depeg limit (basis points, e.g., 9500 = 95%)
    pub depeg_limit: u32,
    /// Expected capital efficiency multiplier (scaled by 10000)
    pub expected_efficiency: u32,
    /// Description of the configuration
    pub description: alloc::string::String,
    /// Recommended liquidity amount
    pub recommended_liquidity: U256,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_reserves(values: Vec<u64>) -> ReservePoint {
        ReservePoint::new(values.iter().map(|&v| U256::from(v)).collect())
    }

    #[test]
    fn test_is_interior_to_tick() {
        let reserves = create_test_reserves(vec![100, 100, 100]);
        let tick = Tick::new(
            U256::from(1),
            U256::from(200),  // c = 200
            U256::from(1000),
            U256::from(300),
            9500,
        );
        
        // Sum = 300, boundary = 200 * sqrt(3) ≈ 346
        // 300 < 346, so should be interior
        let result = is_interior_to_tick(&reserves, &tick).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_on_boundary() {
        let reserves = create_test_reserves(vec![200, 200]);
        let tick = Tick::new(
            U256::from(1),
            U256::from(283),  // c chosen so boundary ≈ 400
            U256::from(1000),
            U256::from(300),
            9500,
        );
        
        // Sum = 400, boundary ≈ 283 * sqrt(2) ≈ 400
        let result = is_on_boundary(&reserves, &tick).unwrap();
        assert!(result);
    }

    #[test]
    fn test_normalized_position() {
        let reserves = create_test_reserves(vec![100, 100]);
        let radius = U256::from(200);
        
        let pos = normalized_position(&reserves, radius).unwrap();
        
        // pos = (100+100)/(200*sqrt(2)) * PRECISION
        // Should be positive
        assert!(pos > U256::ZERO);
    }

    #[test]
    fn test_find_next_crossing() {
        let start = create_test_reserves(vec![100, 100, 100]);
        let end = create_test_reserves(vec![150, 150, 150]);
        
        let tick = Tick::new(
            U256::from(1),
            U256::from(80),  // Boundary between start and end
            U256::from(1000),
            U256::from(300),
            9500,
        );
        
        let ticks = vec![tick];
        let result = find_next_crossing(&start, &end, &ticks).unwrap();
        
        // Should find a crossing
        assert!(result.is_some());
    }

    #[test]
    fn test_calculate_capital_efficiency() {
        let tick = Tick::new(
            U256::from(1),
            U256::from(9500),
            U256::from(1000000),
            U256::from(10000),
            9500,
        );
        
        let efficiency = calculate_capital_efficiency(&tick, 5).unwrap();
        
        // Should be > 10000 (1x)
        assert!(efficiency > 10000);
    }

    #[test]
    fn test_optimize_tick_placement() {
        let recommendation = optimize_tick_placement(
            U256::from(1_000_000),
            100,  // 1% tolerance
            5,
        ).unwrap();
        
        assert_eq!(recommendation.depeg_limit, 9900);
        assert!(recommendation.expected_efficiency > 10000);
    }

    #[test]
    fn test_sort_ticks_by_boundary() {
        let mut ticks = vec![
            Tick::new(U256::from(1), U256::from(300), U256::from(100), U256::from(400), 9000),
            Tick::new(U256::from(2), U256::from(100), U256::from(100), U256::from(400), 9500),
            Tick::new(U256::from(3), U256::from(200), U256::from(100), U256::from(400), 9300),
        ];
        
        sort_ticks_by_boundary(&mut ticks);
        
        assert_eq!(ticks[0].plane_constant, U256::from(100));
        assert_eq!(ticks[1].plane_constant, U256::from(200));
        assert_eq!(ticks[2].plane_constant, U256::from(300));
    }

    #[test]
    fn test_merge_similar_ticks() {
        let ticks = vec![
            Tick::new(U256::from(1), U256::from(100), U256::from(1000), U256::from(400), 9500),
            Tick::new(U256::from(2), U256::from(101), U256::from(2000), U256::from(400), 9500),
            Tick::new(U256::from(3), U256::from(200), U256::from(3000), U256::from(400), 9000),
        ];
        
        let merged = merge_similar_ticks(&ticks, 200).unwrap();
        
        // First two should merge (within 2% tolerance)
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].liquidity, U256::from(3000));
    }

    #[test]
    fn test_active_liquidity_at_point() {
        let reserves = create_test_reserves(vec![100, 100, 100]);
        
        let ticks = vec![
            Tick::new(U256::from(1), U256::from(200), U256::from(1000), U256::from(400), 9500),
            Tick::new(U256::from(2), U256::from(50), U256::from(2000), U256::from(400), 9800),
        ];
        
        let active = active_liquidity_at_point(&reserves, &ticks).unwrap();
        
        // At least one tick should be active
        assert!(active > U256::ZERO);
    }

    #[test]
    fn test_crossing_fraction() {
        let start = create_test_reserves(vec![100, 100]);
        let end = create_test_reserves(vec![200, 200]);
        
        let tick = Tick::new(
            U256::from(1),
            U256::from(100),
            U256::from(1000),
            U256::from(300),
            9500,
        );
        
        let fraction = crossing_fraction(&start, &end, &tick).unwrap();
        
        // Should be between 0 and PRECISION
        assert!(fraction <= U256::from(crate::PRECISION_MULTIPLIER));
    }
}
