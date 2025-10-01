//! Simple swap example
//!
//! Demonstrates basic usage of the orbital-math library for executing swaps
//! on a 2-token pool.

use orbital_math::{
    trades,
    types::{PoolState, CurveType},
    U256,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌀 Orbital AMM - Simple Swap Example\n");
    
    // Create a 2-token pool (e.g., USDC/USDT)
    println!("Creating pool with 1M USDC and 1M USDT...");
    let initial_reserves = vec![
        U256::from(1_000_000),  // 1M USDC
        U256::from(1_000_000),  // 1M USDT
    ];
    
    // For a 2D pool: R² = r₀² + r₁²
    let r_squared = U256::from(2_000_000_000_000u128);
    
    let mut pool = PoolState::new(
        initial_reserves,
        CurveType::sphere(),
        r_squared,
        vec![],  // No ticks for this simple example
    );
    
    println!("✓ Pool created\n");
    
    // Execute a swap: 10,000 USDC -> USDT
    println!("Executing swap: 10,000 USDC -> USDT");
    let amount_in = U256::from(10_000);
    let min_amount_out = U256::from(9_900); // 1% slippage tolerance
    
    let trade_info = trades::execute_swap_toroidal(
        &mut pool,
        0,  // USDC (token 0)
        1,  // USDT (token 1)
        amount_in,
        min_amount_out,
    )?;
    
    println!("\n📊 Trade Results:");
    println!("  Amount in:       {}", trade_info.amount_in);
    println!("  Amount out:      {}", trade_info.amount_out);
    println!("  Price before:    {}", trade_info.price_before);
    println!("  Price after:     {}", trade_info.price_after);
    println!("  Price impact:    {}bp ({}%)", 
        trade_info.price_impact_bp, 
        trade_info.price_impact_bp as f64 / 100.0
    );
    
    // Calculate effective exchange rate
    let rate = trade_info.exchange_rate();
    let rate_f64 = rate.as_limbs()[0] as f64 / 1e18;
    println!("  Exchange rate:   {:.6}", rate_f64);
    
    println!("\n💎 New Pool State:");
    println!("  USDC reserves:   {}", pool.reserves.reserves[0]);
    println!("  USDT reserves:   {}", pool.reserves.reserves[1]);
    
    let total_liquidity = pool.total_liquidity();
    println!("  Total liquidity: {}", total_liquidity);
    
    println!("\n✅ Swap completed successfully!");
    
    Ok(())
}