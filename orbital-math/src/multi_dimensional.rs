//! Multi-dimensional pool example
//!
//! Demonstrates a 5-stablecoin pool using N-dimensional spherical AMM

use orbital_math::{
    trades, sphere,
    types::{PoolState, CurveType},
    U256,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Orbital AMM - 5D Stablecoin Pool Example\n");
    
    // Create a 5-stablecoin pool
    let tokens = ["USDC", "USDT", "DAI", "FRAX", "LUSD"];
    println!("Creating 5-dimensional pool:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {} - 1,000,000 units", i, token);
    }
    
    let initial_reserves = vec![
        U256::from(1_000_000),  // USDC
        U256::from(1_000_000),  // USDT
        U256::from(1_000_000),  // DAI
        U256::from(1_000_000),  // FRAX
        U256::from(1_000_000),  // LUSD
    ];
    
    // For 5D: R² = Σ(r_i²) = 5 * 1M² = 5 * 10^12
    let r_squared = U256::from(5_000_000_000_000u128);
    
    let mut pool = PoolState::new(
        initial_reserves,
        CurveType::sphere(),
        r_squared,
        vec![],
    );
    
    println!("✓ Pool created with {} tokens\n", pool.token_count());
    
    // Show all prices at equal point
    println!("📈 Initial Prices (all at 1:1):");
    for i in 0..tokens.len() {
        for j in (i+1)..tokens.len() {
            let price = sphere::calculate_price_sphere(&pool.reserves.reserves, i, j)?;
            let price_f64 = price.as_limbs()[0] as f64 / 1e18;
            println!("  {}/{}: {:.6}", tokens[i], tokens[j], price_f64);
        }
    }
    
    // Execute swap: USDC -> DAI
    println!("\n🔄 Executing swap: 50,000 USDC -> DAI");
    let trade_info = trades::execute_swap_toroidal(
        &mut pool,
        0,  // USDC
        2,  // DAI
        U256::from(50_000),
        U256::from(49_500),  // 1% slippage
    )?;
    
    println!("\n📊 Trade Results:");
    println!("  Amount in (USDC): {}", trade_info.amount_in);
    println!("  Amount out (DAI): {}", trade_info.amount_out);
    println!("  Price impact:     {}bp", trade_info.price_impact_bp);
    
    // Show updated reserves
    println!("\n💎 Updated Reserves:");
    for (i, token) in tokens.iter().enumerate() {
        let reserve = pool.reserves.reserves[i];
        let change = if i == 0 {
            "+50,000"
        } else if i == 2 {
            format!("-{}", 50_000 - (reserve.as_limbs()[0] as i64 - 1_000_000))
        } else {
            "no change".to_string()
        };
        println!("  {}: {} ({})", token, reserve, change);
    }
    
    // Verify invariant maintained
    println!("\n🔍 Verifying invariant...");
    match sphere::verify_sphere_constraint(&pool.reserves.reserves, r_squared, 10) {
        Ok(_) => println!("  ✅ Invariant maintained: Σ(r_i²) = R²"),
        Err(e) => println!("  ❌ Invariant violated: {}", e),
    }
    
    // Show new prices
    println!("\n📈 Updated Prices:");
    let price_usdc_dai = sphere::calculate_price_sphere(&pool.reserves.reserves, 0, 2)?;
    let price_f64 = price_usdc_dai.as_limbs()[0] as f64 / 1e18;
    println!("  USDC/DAI: {:.6} (changed from 1.0)", price_f64);
    
    // Show that other pairs are slightly affected due to N-dimensional geometry
    let price_usdt_frax = sphere::calculate_price_sphere(&pool.reserves.reserves, 1, 3)?;
    let price_f64 = price_usdt_frax.as_limbs()[0] as f64 / 1e18;
    println!("  USDT/FRAX: {:.6} (slightly changed)", price_f64);
    
    println!("\n✨ Advantage: In N-dimensional pools, swaps between any pair");
    println!("   affect all other pairs, enabling better capital efficiency!");
    
    Ok(())
}