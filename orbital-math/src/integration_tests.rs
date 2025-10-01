//! Integration tests for orbital-math library
//!
//! These tests verify that all modules work together correctly
//! for end-to-end scenarios.

use orbital_math::{
    sphere, superellipse, ticks, trades,
    types::{PoolState, CurveType, Tick, ReservePoint},
    error::Result,
    U256,
};

/// Helper to create test reserves
fn reserves(values: &[u64]) -> Vec<U256> {
    values.iter().map(|&v| U256::from(v)).collect()
}

#[test]
fn test_end_to_end_2d_swap() -> Result<()> {
    // Create a 2-token pool (USDC/USDT)
    let initial_reserves = reserves(&[1_000_000, 1_000_000]);
    let r_squared = U256::from(2_000_000_000_000u128);
    
    let mut pool = PoolState::new(
        initial_reserves.clone(),
        CurveType::sphere(),
        r_squared,
        vec![],
    );
    
    // Execute swap: 10k USDC -> USDT
    let trade_info = trades::execute_swap_toroidal(
        &mut pool,
        0,  // USDC
        1,  // USDT
        U256::from(10_000),
        U256::from(9_900),  // 1% slippage tolerance
    )?;
    
    // Verify trade executed
    assert!(trade_info.amount_out >= U256::from(9_900));
    assert!(trade_info.amount_out < U256::from(10_000));
    
    // Verify price impact is reasonable
    assert!(trade_info.price_impact_bp < 200); // < 2%
    
    // Verify reserves updated
    assert_eq!(pool.reserves.reserves[0], U256::from(1_010_000));
    assert!(pool.reserves.reserves[1] < U256::from(1_000_000));
    
    // Verify invariant maintained
    sphere::verify_sphere_constraint(&pool.reserves.reserves, r_squared, 10)?;
    
    Ok(())
}

#[test]
fn test_end_to_end_5d_swap() -> Result<()> {
    // Create a 5-stablecoin pool
    let initial_reserves = reserves(&[1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000]);
    let r_squared = U256::from(5_000_000_000_000u128);
    
    let mut pool = PoolState::new(
        initial_reserves,
        CurveType::sphere(),
        r_squared,
        vec![],
    );
    
    // Execute swap: token 0 -> token 2
    let trade_info = trades::execute_swap_toroidal(
        &mut pool,
        0,
        2,
        U256::from(50_000),
        U256::from(49_000),
    )?;
    
    // Verify reasonable output
    assert!(trade_info.amount_out >= U256::from(49_000));
    
    // Verify invariant
    sphere::verify_sphere_constraint(&pool.reserves.reserves, r_squared, 10)?;
    
    Ok(())
}

#[test]
fn test_concentrated_liquidity_efficiency() -> Result<()> {
    // Create tick with 95% depeg limit
    let tick = Tick::new(
        U256::from(1),
        U256::from(9500),
        U256::from(1_000_000),
        U256::from(10_000),
        9500,
    );
    
    let efficiency = ticks::calculate_capital_efficiency(&tick, 5)?;
    
    // Should be significantly higher than 1x (10000)
    assert!(efficiency > 50_000); // At least 5x efficiency
    
    Ok(())
}

#[test]
fn test_tick_boundary_detection() -> Result<()> {
    let start = ReservePoint::new(reserves(&[100, 100, 100]));
    let end = ReservePoint::new(reserves(&[150, 150, 150]));
    
    let tick = Tick::new(
        U256::from(1),
        U256::from(80),
        U256::from(1_000),
        U256::from(200),
        9500,
    );
    
    let ticks_vec = vec![tick];
    let crossing = ticks::find_next_crossing(&start, &end, &ticks_vec)?;
    
    // Should detect crossing
    assert!(crossing.is_some());
    
    Ok(())
}

#[test]
fn test_superellipse_vs_sphere() -> Result<()> {
    let reserves_vec = reserves(&[1000, 1000]);
    let k = U256::from(2_000_000);
    
    // Test sphere (u=2.0)
    sphere::verify_sphere_constraint(&reserves_vec, k, 10)?;
    superellipse::verify_superellipse_constraint(&reserves_vec, 20000, k, 10)?;
    
    // Test superellipse (u=2.5)
    let k_25 = U256::from(3_162_277); // Approx 1000^2.5 * 2
    superellipse::verify_superellipse_constraint(&reserves_vec, 25000, k_25, 100)?;
    
    Ok(())
}

#[test]
fn test_multiple_sequential_swaps() -> Result<()> {
    let mut pool = PoolState::new(
        reserves(&[1_000_000, 1_000_000, 1_000_000]),
        CurveType::sphere(),
        U256::from(3_000_000_000_000u128),
        vec![],
    );
    
    // Execute 5 sequential swaps
    for _ in 0..5 {
        let trade_info = trades::execute_swap_toroidal(
            &mut pool,
            0,
            1,
            U256::from(10_000),
            U256::from(9_800),
        )?;
        
        assert!(trade_info.amount_out > U256::ZERO);
    }
    
    // Verify pool still maintains invariant
    sphere::verify_sphere_constraint(
        &pool.reserves.reserves,
        pool.invariant,
        10,
    )?;
    
    Ok(())
}

#[test]
fn test_large_trade_impact() -> Result<()> {
    let mut pool = PoolState::new(
        reserves(&[1_000_000, 1_000_000]),
        CurveType::sphere(),
        U256::from(2_000_000_000_000u128),
        vec![],
    );
    
    // Large trade (10% of pool)
    let trade_info = trades::execute_swap_toroidal(
        &mut pool,
        0,
        1,
        U256::from(100_000),
        U256::ZERO, // No minimum for testing
    )?;
    
    // Price impact should be noticeable but not extreme
    assert!(trade_info.price_impact_bp > 100);  // > 1%
    assert!(trade_info.price_impact_bp < 2000); // < 20%
    
    Ok(())
}

#[test]
fn test_tick_optimization_recommendations() -> Result<()> {
    // Conservative: low tolerance
    let rec_conservative = ticks::optimize_tick_placement(
        U256::from(1_000_000),
        100,  // 1% tolerance
        5,
    )?;
    
    assert_eq!(rec_conservative.depeg_limit, 9900);
    assert!(rec_conservative.expected_efficiency > 100_000); // > 10x
    
    // Aggressive: high tolerance
    let rec_aggressive = ticks::optimize_tick_placement(
        U256::from(1_000_000),
        1500,  // 15% tolerance
        5,
    )?;
    
    assert_eq!(rec_aggressive.depeg_limit, 8500);
    assert!(rec_aggressive.expected_efficiency < rec_conservative.expected_efficiency);
    
    Ok(())
}

#[test]
fn test_price_calculation_consistency() -> Result<()> {
    let reserves_vec = reserves(&[1_000_000, 2_000_000, 1_500_000]);
    
    // Calculate price for different pairs
    let price_01 = sphere::calculate_price_sphere(&reserves_vec, 0, 1)?;
    let price_10 = sphere::calculate_price_sphere(&reserves_vec, 1, 0)?;
    
    // Prices should be reciprocals (approximately)
    let product = price_01
        .checked_mul(price_10)
        .unwrap()
        / U256::from(orbital_math::PRECISION_MULTIPLIER);
    
    // Product should be close to PRECISION_MULTIPLIER (1.0)
    let expected = U256::from(orbital_math::PRECISION_MULTIPLIER);
    let diff = if product > expected {
        product - expected
    } else {
        expected - product
    };
    
    // Allow 1% deviation
    let tolerance = expected / U256::from(100);
    assert!(diff < tolerance, "Price reciprocals should multiply to ~1.0");
    
    Ok(())
}

#[test]
fn test_zero_amount_swap() -> Result<()> {
    let mut pool = PoolState::new(
        reserves(&[1_000_000, 1_000_000]),
        CurveType::sphere(),
        U256::from(2_000_000_000_000u128),
        vec![],
    );
    
    // Zero amount should fail gracefully
    let result = trades::execute_swap_toroidal(
        &mut pool,
        0,
        1,
        U256::ZERO,
        U256::ZERO,
    );
    
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_insufficient_liquidity() -> Result<()> {
    let mut pool = PoolState::new(
        reserves(&[1_000, 1_000]),
        CurveType::sphere(),
        U256::from(2_000_000u128),
        vec![],
    );
    
    // Try to swap more than available
    let result = trades::execute_swap_toroidal(
        &mut pool,
        0,
        1,
        U256::from(10_000),
        U256::ZERO,
    );
    
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_equal_price_point() -> Result<()> {
    let r_squared = U256::from(5_000_000_000_000u128);
    let token_count = 5;
    
    let equal_reserve = sphere::calculate_equal_price_point(r_squared, token_count)?;
    
    // Create pool at equal price point
    let reserves_vec = vec![equal_reserve; token_count];
    
    // Verify it satisfies constraint
    sphere::verify_sphere_constraint(&reserves_vec, r_squared, 10)?;
    
    // All prices should be 1.0
    for i in 0..token_count {
        for j in 0..token_count {
            if i != j {
                let price = sphere::calculate_price_sphere(&reserves_vec, i, j)?;
                let expected = U256::from(orbital_math::PRECISION_MULTIPLIER);
                
                // Should be very close to 1.0
                let diff = if price > expected {
                    price - expected
                } else {
                    expected - price
                };
                
                let tolerance = expected / U256::from(100);
                assert!(diff < tolerance, "Prices at equal point should be 1.0");
            }
        }
    }
    
    Ok(())
}

#[test]
fn test_trade_segmentation() -> Result<()> {
    let pool = PoolState::new(
        reserves(&[1_000_000, 1_000_000, 1_000_000]),
        CurveType::sphere(),
        U256::from(3_000_000_000_000u128),
        vec![],
    );
    
    let segments = trades::segment_trade(&pool, 0, 1, U256::from(100_000))?;
    
    // Should produce at least one segment
    assert!(!segments.is_empty());
    
    // Sum of segment inputs should equal total input
    let total_in: U256 = segments.iter().map(|s| s.amount_in).sum();
    assert_eq!(total_in, U256::from(100_000));
    
    // All segments should have positive amounts
    for segment in &segments {
        assert!(segment.amount_in > U256::ZERO);
        assert!(segment.amount_out > U256::ZERO);
    }
    
    Ok(())
}

#[test]
fn test_concentration_ratio_calculation() -> Result<()> {
    // u=2.0 (sphere) should have 1x concentration
    let ratio_20 = superellipse::concentration_ratio(20000);
    assert_eq!(ratio_20, 10000);
    
    // u=2.5 should have 1.5x concentration
    let ratio_25 = superellipse::concentration_ratio(25000);
    assert_eq!(ratio_25, 15000);
    
    // u=3.0 should have 2x concentration
    let ratio_30 = superellipse::concentration_ratio(30000);
    assert_eq!(ratio_30, 20000);
    
    Ok(())
}

#[test]
fn test_optimal_u_selection() -> Result<()> {
    // High volatility -> lower u (more sphere-like)
    let u_high_vol = superellipse::optimal_u_for_volatility(200);
    assert!(u_high_vol <= 22000);
    
    // Low volatility -> higher u (more concentrated)
    let u_low_vol = superellipse::optimal_u_for_volatility(25);
    assert!(u_low_vol >= 25000);
    
    // Higher volatility should use lower u
    assert!(u_high_vol < u_low_vol);
    
    Ok(())
}