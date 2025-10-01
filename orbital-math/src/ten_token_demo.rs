//! 10-Token Orbital AMM Demonstration
//!
//! This example showcases the full power of the N-dimensional Orbital AMM
//! with a realistic 10-token pool including:
//! - Multiple stablecoins and volatile tokens
//! - Capital efficient tick configuration
//! - Complex multi-hop routing
//! - Real-world trading scenarios

use orbital_math::{
    trades::{self, execute_swap_toroidal, execute_multi_hop_swap, calculate_optimal_route},
    sphere::{self, calculate_price_sphere, verify_sphere_constraint},
    ticks::{self, optimize_tick_placement},
    types::{PoolState, CurveType, Tick},
    superellipse,
    U256,
};

#[derive(Debug)]
pub struct TokenInfo {
    pub symbol: &'static str,
    pub name: &'static str,
    pub initial_reserves: u64,
    pub token_type: TokenType,
}

#[derive(Debug)]
pub enum TokenType {
    Stable,
    Volatile,
    Synthetic,
}

/// Comprehensive 10-token pool demonstration
pub fn run_ten_token_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Orbital AMM - 10-Token Pool Demonstration");
    println!("============================================\n");
    
    // Define our 10-token pool
    let tokens = create_token_list();
    
    println!("📋 Pool Composition:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {} ({}) - {:?} - {} initial units", 
            i, token.symbol, token.name, token.token_type, token.initial_reserves);
    }
    
    // Create the pool with different reserves for realistic simulation
    let initial_reserves: Vec<U256> = tokens.iter()
        .map(|t| U256::from(t.initial_reserves))
        .collect();
    
    // Calculate sphere constant: R² = Σ(r_i²)
    let r_squared = calculate_sphere_constant(&initial_reserves)?;
    
    // Create pool with concentrated liquidity ticks
    let ticks = create_concentrated_liquidity_ticks(&tokens)?;
    
    let mut pool = PoolState::new(
        initial_reserves.clone(),
        CurveType::sphere(),
        r_squared,
        ticks,
    );
    
    println!("\n✅ Pool created with {} tokens", pool.token_count());
    println!("   Sphere constant R² = {}", r_squared);
    println!("   Active ticks: {}", pool.ticks.len());
    
    // Demonstrate various trading scenarios
    demonstrate_trading_scenarios(&mut pool, &tokens)?;
    
    // Show capital efficiency gains
    demonstrate_capital_efficiency(&pool)?;
    
    // Multi-hop routing examples
    demonstrate_multi_hop_routing(&mut pool, &tokens)?;
    
    // Advanced features
    demonstrate_advanced_features(&mut pool, &tokens)?;
    
    println!("\n🎉 Demonstration completed successfully!");
    println!("   This showcases the power of N-dimensional Orbital AMMs");
    println!("   for efficient trading across multiple token types.");
    
    Ok(())
}

fn create_token_list() -> Vec<TokenInfo> {
    vec![
        TokenInfo {
            symbol: "USDC",
            name: "USD Coin",
            initial_reserves: 10_000_000, // 10M
            token_type: TokenType::Stable,
        },
        TokenInfo {
            symbol: "USDT", 
            name: "Tether USD",
            initial_reserves: 8_000_000, // 8M
            token_type: TokenType::Stable,
        },
        TokenInfo {
            symbol: "DAI",
            name: "MakerDAO DAI",
            initial_reserves: 12_000_000, // 12M
            token_type: TokenType::Stable,
        },
        TokenInfo {
            symbol: "FRAX",
            name: "Frax",
            initial_reserves: 5_000_000, // 5M
            token_type: TokenType::Stable,
        },
        TokenInfo {
            symbol: "WETH",
            name: "Wrapped Ethereum",
            initial_reserves: 2_500, // 2,500 ETH
            token_type: TokenType::Volatile,
        },
        TokenInfo {
            symbol: "WBTC",
            name: "Wrapped Bitcoin", 
            initial_reserves: 150, // 150 BTC
            token_type: TokenType::Volatile,
        },
        TokenInfo {
            symbol: "LINK",
            name: "Chainlink",
            initial_reserves: 500_000, // 500k LINK
            token_type: TokenType::Volatile,
        },
        TokenInfo {
            symbol: "UNI",
            name: "Uniswap",
            initial_reserves: 800_000, // 800k UNI
            token_type: TokenType::Volatile,
        },
        TokenInfo {
            symbol: "stETH",
            name: "Lido Staked ETH",
            initial_reserves: 2_400, // 2,400 stETH
            token_type: TokenType::Synthetic,
        },
        TokenInfo {
            symbol: "rETH",
            name: "Rocket Pool ETH",
            initial_reserves: 1_800, // 1,800 rETH
            token_type: TokenType::Synthetic,
        },
    ]
}

fn calculate_sphere_constant(reserves: &[U256]) -> Result<U256, Box<dyn std::error::Error>> {
    let sum_of_squares = reserves
        .iter()
        .try_fold(U256::ZERO, |acc, &r| {
            let r_squared = r.checked_mul(r)
                .ok_or("Overflow in reserve squared")?;
            acc.checked_add(r_squared)
                .ok_or("Overflow in sum of squares")
        })?;
    
    Ok(sum_of_squares)
}

fn create_concentrated_liquidity_ticks(tokens: &[TokenInfo]) -> Result<Vec<Tick>, Box<dyn std::error::Error>> {
    let mut ticks = Vec::new();
    
    // Create different tick configurations for different token types
    
    // Tight ticks for stablecoin pairs (98-99% range)
    let stable_tick = Tick::new(
        U256::from(1),
        U256::from(9800), // 98% depeg limit
        U256::from(50_000_000), // High liquidity for stables
        U256::from(100_000),
        9800,
    );
    ticks.push(stable_tick);
    
    // Medium ticks for ETH derivatives (95-98% range)  
    let eth_tick = Tick::new(
        U256::from(2),
        U256::from(9500), // 95% depeg limit
        U256::from(20_000_000), // Medium liquidity
        U256::from(100_000),
        9500,
    );
    ticks.push(eth_tick);
    
    // Wide ticks for volatile tokens (80-95% range)
    let volatile_tick = Tick::new(
        U256::from(3),
        U256::from(8000), // 80% depeg limit
        U256::from(10_000_000), // Lower liquidity
        U256::from(100_000),
        8000,
    );
    ticks.push(volatile_tick);
    
    // Ultra-tight tick for stable-stable pairs
    let ultra_tight_tick = Tick::new(
        U256::from(4),
        U256::from(9950), // 99.5% depeg limit
        U256::from(100_000_000), // Maximum liquidity
        U256::from(100_000),
        9950,
    );
    ticks.push(ultra_tight_tick);
    
    Ok(ticks)
}

fn demonstrate_trading_scenarios(
    pool: &mut PoolState,
    tokens: &[TokenInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Trading Scenarios");
    println!("===================");
    
    // Scenario 1: Stablecoin arbitrage (USDC -> USDT)
    println!("\n1. Stablecoin Arbitrage: 100k USDC -> USDT");
    let trade1 = execute_swap_toroidal(
        pool,
        0, // USDC
        1, // USDT
        U256::from(100_000),
        U256::from(99_500), // 0.5% slippage tolerance
    )?;
    
    println!("   Amount out: {} USDT", trade1.amount_out);
    println!("   Price impact: {}bp", trade1.price_impact_bp);
    println!("   Fee: {} USDC", trade1.fee);
    println!("   Ticks crossed: {}", trade1.ticks_crossed);
    
    // Scenario 2: Volatile token swap (WETH -> WBTC)
    println!("\n2. Volatile Swap: 10 WETH -> WBTC");
    let trade2 = execute_swap_toroidal(
        pool,
        4, // WETH
        5, // WBTC
        U256::from(10),
        U256::from(0), // Market order
    )?;
    
    println!("   Amount out: {} WBTC", trade2.amount_out);
    println!("   Price impact: {}bp", trade2.price_impact_bp);
    println!("   Fee: {} WETH", trade2.fee);
    
    // Scenario 3: Cross-category swap (DAI -> LINK)
    println!("\n3. Cross-Category: 50k DAI -> LINK");
    let trade3 = execute_swap_toroidal(
        pool,
        2, // DAI
        6, // LINK
        U256::from(50_000),
        U256::from(0), // Market order
    )?;
    
    println!("   Amount out: {} LINK", trade3.amount_out);
    println!("   Price impact: {}bp", trade3.price_impact_bp);
    println!("   Fee: {} DAI", trade3.fee);
    
    // Verify pool invariant after trades
    verify_sphere_constraint(&pool.reserves.reserves, pool.invariant, 100)?;
    println!("\n✅ Pool invariant maintained after all trades");
    
    Ok(())
}

fn demonstrate_capital_efficiency(pool: &PoolState) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💎 Capital Efficiency Analysis");
    println!("=============================");
    
    for (i, tick) in pool.ticks.iter().enumerate() {
        let efficiency = ticks::calculate_capital_efficiency(tick, pool.token_count())?;
        let efficiency_multiplier = efficiency as f64 / 10000.0;
        
        println!("Tick {}: {}x capital efficiency vs full range", 
            i + 1, efficiency_multiplier);
        
        // Calculate liquidity concentration
        let min_reserve = tick.min_reserve(pool.token_count());
        let max_reserve = tick.max_reserve(pool.token_count());
        
        println!("  Reserve range: {} - {} (depeg limit: {}%)", 
            min_reserve, max_reserve, tick.depeg_limit as f64 / 100.0);
    }
    
    let total_efficiency = pool.ticks.iter()
        .map(|t| ticks::calculate_capital_efficiency(t, pool.token_count()).unwrap_or(10000))
        .sum::<u32>() as f64 / (pool.ticks.len() * 10000) as f64;
    
    println!("\nOverall pool efficiency: {:.2}x vs traditional AMM", total_efficiency);
    
    Ok(())
}

fn demonstrate_multi_hop_routing(
    pool: &mut PoolState,
    tokens: &[TokenInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛣️  Multi-Hop Routing");
    println!("===================");
    
    // Complex route: FRAX -> DAI -> WETH -> UNI
    println!("\n1. Complex Route: 25k FRAX -> DAI -> WETH -> UNI");
    let path = vec![3, 2, 4, 7]; // FRAX -> DAI -> WETH -> UNI
    
    let multi_hop_trade = execute_multi_hop_swap(
        pool,
        &path,
        U256::from(25_000),
        U256::from(0), // Market order
    )?;
    
    println!("   Final amount: {} UNI", multi_hop_trade.amount_out);
    println!("   Total price impact: {}bp", multi_hop_trade.price_impact_bp);
    println!("   Total fees: {} FRAX equivalent", multi_hop_trade.fee);
    println!("   Total ticks crossed: {}", multi_hop_trade.ticks_crossed);
    
    // Find optimal route
    println!("\n2. Optimal Route Discovery: USDC -> rETH");
    let optimal_path = calculate_optimal_route(
        pool,
        0, // USDC
        9, // rETH  
        U256::from(100_000),
        3, // Max 3 hops
    )?;
    
    print!("   Optimal path: ");
    for (i, &token_idx) in optimal_path.iter().enumerate() {
        if i > 0 { print!(" -> "); }
        print!("{}", tokens[token_idx].symbol);
    }
    println!();
    
    // Execute optimal route
    let optimal_trade = execute_multi_hop_swap(
        pool,
        &optimal_path,
        U256::from(100_000),
        U256::from(0),
    )?;
    
    println!("   Optimal output: {} rETH", optimal_trade.amount_out);
    println!("   Price impact: {}bp", optimal_trade.price_impact_bp);
    
    Ok(())
}

fn demonstrate_advanced_features(
    pool: &mut PoolState,
    tokens: &[TokenInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Advanced Features");
    println!("==================");
    
    // Price impact analysis across all pairs
    println!("\n1. Price Impact Matrix (for 10k trade):");
    println!("   From\\To    USDC    USDT     DAI    FRAX    WETH");
    
    for i in 0..5 {
        print!("   {:8}", tokens[i].symbol);
        for j in 0..5 {
            if i == j {
                print!("     -  ");
            } else {
                // Calculate price impact for 10k trade
                let amount = U256::from(10_000);
                if let Ok(trade_output) = trades::calculate_trade_output(pool, i, j, amount) {
                    let price_before = calculate_price_sphere(&pool.reserves.reserves, i, j)?;
                    
                    // Simulate the trade impact
                    let mut test_reserves = pool.reserves.reserves.clone();
                    test_reserves[i] = test_reserves[i].checked_add(amount).unwrap();
                    test_reserves[j] = test_reserves[j].checked_sub(trade_output).unwrap();
                    
                    if let Ok(price_after) = calculate_price_sphere(&test_reserves, i, j) {
                        let impact = if price_before > U256::ZERO {
                            let diff = if price_after > price_before {
                                price_after - price_before
                            } else {
                                price_before - price_after
                            };
                            ((diff * U256::from(10000)) / price_before).as_limbs()[0] as u32
                        } else {
                            0
                        };
                        print!("  {:3}bp ", impact);
                    } else {
                        print!("   err ");
                    }
                } else {
                    print!("   err ");
                }
            }
        }
        println!();
    }
    
    // Superellipse comparison for stablecoins
    println!("\n2. Superellipse vs Sphere Comparison:");
    
    // Create superellipse variant for stablecoin subset
    let stable_reserves = vec![
        pool.reserves.reserves[0], // USDC
        pool.reserves.reserves[1], // USDT  
        pool.reserves.reserves[2], // DAI
        pool.reserves.reserves[3], // FRAX
    ];
    
    // Calculate equivalent superellipse constant
    let u_param = 25000; // u = 2.5
    let superellipse_k = superellipse::superellipse_to_sphere_approximation(
        &stable_reserves,
        u_param,
        pool.invariant,
    )?;
    
    // Compare price impact for stablecoin swaps
    let sphere_output = sphere::calculate_amount_out_sphere(
        &stable_reserves,
        0, 1, // USDC -> USDT
        U256::from(100_000),
        pool.invariant,
    )?;
    
    let superellipse_output = superellipse::calculate_amount_out_superellipse(
        &stable_reserves,
        0, 1, // USDC -> USDT
        U256::from(100_000),
        u_param,
        superellipse_k,
    )?;
    
    println!("   100k USDC -> USDT:");
    println!("   Sphere output:      {} USDT", sphere_output);
    println!("   Superellipse output: {} USDT", superellipse_output);
    
    let improvement = if superellipse_output > sphere_output {
        ((superellipse_output - sphere_output) * U256::from(10000)) / sphere_output
    } else {
        U256::ZERO
    };
    
    println!("   Superellipse improvement: {}bp", improvement);
    
    // Tick utilization analysis
    println!("\n3. Tick Utilization:");
    
    for (i, tick) in pool.ticks.iter().enumerate() {
        let current_reserves = orbital_math::types::ReservePoint::new(pool.reserves.reserves.clone());
        let is_active = ticks::is_interior_to_tick(&current_reserves, tick)
            .unwrap_or(false);
        let utilization = (tick.liquidity * U256::from(100)) / pool.total_liquidity();
        
        println!("   Tick {}: {}% utilization, {} active", 
            i + 1, utilization, if is_active { "✓" } else { "✗" });
    }
    
    Ok(())
}

// Helper function to access calculate_trade_output
mod trades {
    use super::*;
    
    pub fn calculate_trade_output(
        pool: &PoolState,
        token_in: usize,
        token_out: usize,
        amount_in: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        match pool.curve_type {
            CurveType::Sphere => {
                Ok(sphere::calculate_amount_out_sphere(
                    &pool.reserves.reserves,
                    token_in,
                    token_out,
                    amount_in,
                    pool.invariant,
                )?)
            }
            CurveType::Superellipse { u_parameter } => {
                Ok(superellipse::calculate_amount_out_superellipse(
                    &pool.reserves.reserves,
                    token_in,
                    token_out,
                    amount_in,
                    u_parameter,
                    pool.invariant,
                )?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ten_token_pool_creation() {
        let tokens = create_token_list();
        assert_eq!(tokens.len(), 10);
        
        let initial_reserves: Vec<U256> = tokens.iter()
            .map(|t| U256::from(t.initial_reserves))
            .collect();
        
        let r_squared = calculate_sphere_constant(&initial_reserves).unwrap();
        assert!(r_squared > U256::ZERO);
        
        let pool = PoolState::new(
            initial_reserves,
            CurveType::sphere(),
            r_squared,
            vec![],
        );
        
        assert_eq!(pool.token_count(), 10);
    }

    #[test]
    fn test_concentrated_liquidity_ticks() {
        let tokens = create_token_list();
        let ticks = create_concentrated_liquidity_ticks(&tokens).unwrap();
        
        assert!(!ticks.is_empty());
        
        // Verify tick ordering and configuration
        for tick in &ticks {
            assert!(tick.depeg_limit <= 10000);
            assert!(tick.liquidity > U256::ZERO);
        }
    }

    #[test]
    fn test_trading_scenarios() {
        let tokens = create_token_list();
        let initial_reserves: Vec<U256> = tokens.iter()
            .map(|t| U256::from(t.initial_reserves))
            .collect();
        
        let r_squared = calculate_sphere_constant(&initial_reserves).unwrap();
        let ticks = create_concentrated_liquidity_ticks(&tokens).unwrap();
        
        let mut pool = PoolState::new(
            initial_reserves,
            CurveType::sphere(),
            r_squared,
            ticks,
        );
        
        // Test basic swap
        let result = execute_swap_toroidal(
            &mut pool,
            0, // USDC
            1, // USDT
            U256::from(10_000),
            U256::from(9_900),
        );
        
        assert!(result.is_ok());
        let trade = result.unwrap();
        assert!(trade.amount_out > U256::ZERO);
        assert_eq!(trade.token_in, 0);
        assert_eq!(trade.token_out, 1);
    }

    #[test]
    fn test_multi_hop_routing() {
        let tokens = create_token_list();
        let initial_reserves: Vec<U256> = tokens.iter()
            .map(|t| U256::from(t.initial_reserves))
            .collect();
        
        let r_squared = calculate_sphere_constant(&initial_reserves).unwrap();
        
        let mut pool = PoolState::new(
            initial_reserves,
            CurveType::sphere(),
            r_squared,
            vec![],
        );
        
        let path = vec![0, 2, 4]; // USDC -> DAI -> WETH
        let result = execute_multi_hop_swap(
            &mut pool,
            &path,
            U256::from(10_000),
            U256::from(0),
        );
        
        assert!(result.is_ok());
        let trade = result.unwrap();
        assert!(trade.amount_out > U256::ZERO);
        assert_eq!(trade.token_in, 0);
        assert_eq!(trade.token_out, 4);
    }

    #[test]
    fn test_optimal_route_calculation() {
        let tokens = create_token_list();
        let initial_reserves: Vec<U256> = tokens.iter()
            .map(|t| U256::from(t.initial_reserves))
            .collect();
        
        let r_squared = calculate_sphere_constant(&initial_reserves).unwrap();
        
        let pool = PoolState::new(
            initial_reserves,
            CurveType::sphere(),
            r_squared,
            vec![],
        );
        
        let route = calculate_optimal_route(
            &pool,
            0, // USDC
            9, // rETH
            U256::from(10_000),
            3, // Max hops
        );
        
        assert!(route.is_ok());
        let path = route.unwrap();
        assert_eq!(path[0], 0);
        assert_eq!(path[path.len() - 1], 9);
        assert!(path.len() >= 2);
        assert!(path.len() <= 4); // Max 3 hops + 1
    }
}