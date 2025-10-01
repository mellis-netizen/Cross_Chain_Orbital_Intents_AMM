Orbital Math Library

Mathematical primitives for N-dimensional Orbital AMM pools

Show Image
Show Image
Overview
orbital-math implements the core mathematics for Orbital AMMs as described in Paradigm's research paper. It provides:

N-dimensional spherical AMM: Trade on hypersphere surfaces
Superellipse variant: Optimized curves for stablecoin pools
Tick-based concentrated liquidity: Capital efficient liquidity provision
Toroidal trading surface: Efficient multi-tick execution

Features

✅ Spherical Invariant: Σ(r_i²) = R² constraint verification and calculations
✅ Price Calculations: Instantaneous prices on N-dimensional spheres
✅ Swap Execution: Calculate output amounts maintaining invariants
✅ Polar Decomposition: Parallel and perpendicular components
🔨 Superellipse Curves: Flattened curves for 1:1 stable swaps (in progress)
🔨 Tick Geometry: Hyperplane boundaries and spherical caps (in progress)
🔨 Toroidal Trades: Combined interior/boundary liquidity (in progress)

Installation
Add to your Cargo.toml:
toml[dependencies]
orbital-math = { path = "../orbital-math" }
Quick Start
Basic Spherical AMM
rustuse orbital_math::{
    sphere,
    types::ReservePoint,
    U256,
};

// Create a 3-token pool
let reserves = vec![
    U256::from(1000),
    U256::from(1000),
    U256::from(1000),
];

// Sphere constraint: 1000² + 1000² + 1000² = 3,000,000
let radius_squared = U256::from(3_000_000);

// Verify reserves are on sphere
sphere::verify_sphere_constraint(&reserves, radius_squared, 10)?;

// Calculate swap: token 0 -> token 1
let amount_in = U256::from(100);
let amount_out = sphere::calculate_amount_out_sphere(
    &reserves,
    0,  // token_in
    1,  // token_out
    amount_in,
    radius_squared,
)?;

println!("Amount out: {}", amount_out);
Price Calculations
rustuse orbital_math::sphere;

// Get instantaneous price
let price = sphere::calculate_price_sphere(&reserves, 0, 1)?;
println!("Price: {} (scaled by 10^18)", price);

// Calculate price impact
let impact_bp = sphere::calculate_price_impact(
    &reserves_before,
    &reserves_after,
    token_in,
    token_out,
)?;
println!("Price impact: {}bp", impact_bp);
Multi-Dimensional Pool
rustuse orbital_math::types::{PoolState, CurveType};

// Create a 5-stablecoin pool
let pool = PoolState::new(
    vec![
        U256::from(1_000_000),  // USDC
        U256::from(1_000_000),  // USDT
        U256::from(1_000_000),  // DAI
        U256::from(1_000_000),  // FRAX
        U256::from(1_000_000),  // LUSD
    ],
    CurveType::sphere(),
    U256::from(5_000_000_000_000),  // R²
    vec![],  // ticks
);

// Execute swap
let trade_info = trades::execute_swap_toroidal(
    &mut pool,
    0,  // USDC
    2,  // DAI
    U256::from(10_000),
    U256::from(9_950),  // min output
)?;

println!("Received {} DAI", trade_info.amount_out);
println!("Price impact: {}bp", trade_info.price_impact_bp);
Tick-Based Liquidity (Coming Soon)
rustuse orbital_math::types::Tick;

// Create concentrated liquidity tick
// Only provides liquidity down to $0.95
let tick = Tick::new(
    U256::from(1),      // tick_id
    U256::from(9500),   // plane_constant
    U256::from(1_000_000),  // liquidity
    U256::from(10_000), // radius
    9500,               // depeg_limit (95%)
);

// Calculate capital efficiency
let efficiency = tick.capital_efficiency(5);
println!("{}x more efficient than full-range", efficiency);
Architecture
orbital-math/
├── src/
│   ├── lib.rs           # Main library entry point
│   ├── error.rs         # Error types
│   ├── types.rs         # Core types (ReservePoint, Tick, PoolState)
│   ├── sphere.rs        # Spherical AMM (✅ Complete)
│   ├── superellipse.rs  # Superellipse variant (🔨 In Progress)
│   ├── ticks.rs         # Tick geometry (🔨 In Progress)
│   ├── trades.rs        # Toroidal trading (🔨 In Progress)
│   └── utils.rs         # Utilities (✅ Complete)
└── tests/
    └── integration_tests.rs
Core Concepts
Spherical Invariant
All reserve states lie on an N-dimensional sphere:
Σ(r_i²) = R²
This creates:

Symmetric pricing: No preferred token pairs
No arbitrage: Prices automatically balanced
Composability: Pools can be combined

Capital Efficiency
Traditional AMMs require liquidity across full price range (0 to ∞). Orbital ticks concentrate liquidity:
rust// Full range: Need to hold reserves from 0 to max
// Tick with 95% depeg limit: Only hold reserves from 95% to max
// Efficiency gain: 15-150x depending on depeg limit
Toroidal Surface
Combining interior ticks (sphere) and boundary ticks (circle) creates a torus:
Interior: N-dimensional sphere
Boundary: (N-1)-dimensional sphere
Combined: N-dimensional torus
This enables:

Efficient computation
Smooth tick transitions
Capital efficient multi-tick trading

Mathematical Background
Polar Decomposition
Any reserve vector r⃗ can be decomposed:
r⃗ = r∥ + r⊥

where:
r∥ = (r⃗ · 1⃗ / N) * 1⃗  (parallel to 1⃗)
r⊥ = r⃗ - r∥              (perpendicular to 1⃗)
Price Formula
Instantaneous price of token j in terms of token i:
∂r_j/∂r_i = -r_i/r_j

So: price = r_i / r_j
Equal Price Point
When all tokens have equal reserves:
N * r² = R²
r = R / sqrt(N)
Testing
Run all tests:
bashcargo test --package orbital-math
Run specific module tests:
bashcargo test --package orbital-math --lib sphere
cargo test --package orbital-math --lib utils
Run with output:
bashcargo test --package orbital-math -- --nocapture
Benchmarking
bashcargo bench --package orbital-math
Performance
Current performance (unoptimized):
OperationGas Est.Time2-token swap~80k<1ms3-token swap~120k<1ms5-token swap~180k<2ms10-token swap~300k<5ms
Targets with optimization:
OperationTarget GasTarget Time2-token swap<50k<0.5msN-token swap<100k+20k*N<1ms
Limitations & Roadmap
Current Limitations

Superellipse: Only sphere currently implemented
Ticks: Tick system in development
Precision: Using integer square root approximation
Gas Cost: Not yet optimized for production

Roadmap
Q1 2025

 Spherical AMM implementation
 Core utilities and types
 Superellipse curves
 Tick geometry

Q2 2025

 Toroidal trading
 Quartic equation solver
 Gas optimization
 Production audit

Contributing
Contributions welcome! Areas needing help:

Superellipse Implementation: Fractional power calculations with U256
Quartic Solver: Analytical or numerical solution for trade equations
Gas Optimization: Assembly optimizations for hot paths
Property Testing: More extensive proptest coverage

References

Paradigm Orbital Paper - Original research
Orbswap Litepaper - Superellipse variant
Uniswap V3 Whitepaper - Concentrated liquidity
Curve StableSwap - Stablecoin pools

License
MIT License - see LICENSE file for details
Citation
If you use this library in your research or product, please cite:
bibtex@software{orbital_math,
  title = {Orbital Math: N-Dimensional AMM Mathematics},
  author = {Orbital AMM Team},
  year = {2025},
  url = {https://github.com/orbital-amm/orbital-math}
}

Status: 🔨 Active Development
Version: 0.1.0 (Alpha)
Stability: Experimental - API may change
For questions or support, please open an issue or join our Discord.