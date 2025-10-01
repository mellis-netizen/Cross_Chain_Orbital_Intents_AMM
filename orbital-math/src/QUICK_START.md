Quick Start Guide - Cross-Chain Orbital AMM
🚀 Get Started in 5 Minutes
This guide will get you up and running with the Orbital AMM development environment.
Prerequisites

Git
Rust 1.70+ (install from rustup.rs)
Node.js 18+ (for frontend)
Basic understanding of Rust and blockchain concepts

Setup
1. Clone & Navigate
bashgit clone https://github.com/mellis-netizen/Cross_Chain_Orbital_Intents_AMM.git
cd Cross_Chain_Orbital_Intents_AMM
2. Build the Math Library
bashcd core/orbital-math
cargo build
3. Run Tests
bashcargo test
Expected output:
running 24 tests
test tests::test_precision_constants ... ok
test sphere::tests::test_verify_sphere_constraint_valid ... ok
test sphere::tests::test_calculate_equal_price_point ... ok
test utils::tests::test_pow ... ok
...
test result: ok. 24 passed; 0 failed
4. Try the Examples
bash# View sphere constraint example
cat examples/simple_swap.rs

# Run it (when implemented)
cargo run --example simple_swap
📖 Understanding the Project
Project Structure
Cross_Chain_Orbital_Intents_AMM/
│
├── 📋 Documentation
│   ├── DEVELOPMENT_PLAN.md          ← 12-week roadmap
│   ├── IMPLEMENTATION_SUMMARY.md    ← Current status
│   ├── VISUAL_ARCHITECTURE.md       ← Diagrams & flows
│   └── PROJECT_DELIVERY_SUMMARY.md  ← What's been delivered
│
├── 💻 Core Math Library (START HERE)
│   └── core/orbital-math/
│       ├── README.md                ← Library documentation
│       └── src/
│           ├── sphere.rs            ✅ COMPLETE - Spherical AMM
│           ├── utils.rs             ✅ COMPLETE - Utilities
│           ├── types.rs             ✅ COMPLETE - Core types
│           ├── error.rs             ✅ COMPLETE - Error handling
│           ├── superellipse.rs      🔨 TODO - Superellipse curves
│           ├── ticks.rs             🔨 TODO - Tick geometry
│           └── trades.rs            🔨 TODO - Toroidal trades
│
├── 🎨 Frontend
│   └── frontend/                    (Existing Next.js app)
│
├── 🔧 Smart Contracts
│   └── contracts/
│       ├── orbital-amm/             (Existing 2D AMM)
│       └── intents/                 (Existing intents system)
│
└── 🧪 Backend Services
    └── core/
        ├── engine/                  (Intent engine)
        ├── solver/                  (Solver network)
        └── bridge/                  (Bridge abstraction)
🎯 What to Work On
✅ Already Complete

Spherical AMM mathematics
Utility functions
Error handling
Core type definitions

🔨 In Progress (High Priority)

Superellipse Curves (superellipse.rs)
Tick Geometry (ticks.rs)
Toroidal Trading (trades.rs)

⏳ Upcoming (Medium Priority)

Integration tests
Benchmarking
Multi-dimensional pool contract

🔬 Code Examples
Example 1: Simple 2D Swap
rustuse orbital_math::{sphere, types::ReservePoint, U256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Two-token pool: USDC and USDT
    let reserves = vec![
        U256::from(1_000_000),  // 1M USDC
        U256::from(1_000_000),  // 1M USDT
    ];
    
    // Circle constraint: r0² + r1² = R²
    let radius_squared = U256::from(2_000_000_000_000);
    
    // Swap 1000 USDC for USDT
    let amount_out = sphere::calculate_amount_out_sphere(
        &reserves,
        0,  // USDC
        1,  // USDT
        U256::from(1_000),
        radius_squared,
    )?;
    
    println!("Received {} USDT", amount_out);
    Ok(())
}
Example 2: 5-Token Stablecoin Pool
rustuse orbital_math::{sphere, types::PoolState, CurveType, U256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        U256::from(5_000_000_000_000),
        vec![],
    );
    
    println!("Pool has {} tokens", pool.token_count());
    println!("Total liquidity: {}", pool.total_liquidity());
    
    Ok(())
}
Example 3: Price Impact Calculation
rustuse orbital_math::sphere;

// Calculate price impact of a trade
let impact_bp = sphere::calculate_price_impact(
    &reserves_before,
    &reserves_after,
    token_in,
    token_out,
)?;

println!("Price impact: {}bp ({}%)", impact_bp, impact_bp as f64 / 100.0);
🧪 Running Tests
Run All Tests
bashcd core/orbital-math
cargo test
Run Specific Module
bashcargo test --lib sphere
cargo test --lib utils
cargo test --lib types
Run with Output
bashcargo test -- --nocapture
Run with Detailed Logs
bashRUST_LOG=debug cargo test -- --nocapture
📊 Understanding the Math
Key Concept 1: Spherical Constraint
What it means: All reserve states lie on an N-dimensional sphere.
Why it matters: This ensures balanced pricing and no arbitrage.
Formula: Σ(r_i²) = R²
Example:

2D: r₀² + r₁² = R² (circle)
3D: r₀² + r₁² + r₂² = R² (sphere)
ND: Sum of all squares equals constant

Key Concept 2: Swap Calculation
What happens: Adding to one reserve requires removing from another.
Formula:
Input: Add Δ_in to token i
Output: Remove Δ_out from token j
Constraint: (r_i + Δ_in)² + (r_j - Δ_out)² + Σ(other r²) = R²
Code:
rustlet amount_out = sphere::calculate_amount_out_sphere(
    &reserves,
    token_in,
    token_out,
    amount_in,
    radius_squared,
)?;
Key Concept 3: Tick-Based Liquidity (Coming Soon)
What it is: Concentrated liquidity in specific price ranges.
Why it matters: LPs can get 15-150x capital efficiency.
How it works: Ticks define boundaries where liquidity is active.
🐛 Troubleshooting
Build Fails
bash# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build
Tests Fail
bash# Check Rust version
rustc --version  # Should be 1.70+

# Run specific failing test
cargo test test_name -- --nocapture
Can't Find Module
bash# Ensure you're in the right directory
cd core/orbital-math

# Check Cargo.toml exists
ls Cargo.toml
🎓 Learning Path
Week 1: Understanding the Basics

✅ Read IMPLEMENTATION_SUMMARY.md
✅ Review VISUAL_ARCHITECTURE.md diagrams
✅ Study core/orbital-math/src/sphere.rs
✅ Run tests and understand what they verify

Week 2: Contributing

Pick an unimplemented function in superellipse.rs, ticks.rs, or trades.rs
Write tests first (TDD approach)
Implement the function
Ensure all tests pass
Submit PR

Week 3: Advanced Topics

Understand toroidal invariant mathematics
Study quartic equation solving
Implement tick crossing detection
Optimize for gas efficiency

🤝 Contributing
Areas Needing Help

Superellipse Implementation (High Priority)

Fractional power calculations
Curve optimization for stablecoins
File: src/superellipse.rs


Tick Geometry (High Priority)

Boundary checking
Crossing detection
File: src/ticks.rs


Toroidal Trading (Medium Priority)

Invariant calculation
Quartic solver
File: src/trades.rs


Testing (Always Needed)

Property-based tests
Edge case coverage
Integration tests



How to Contribute

Fork the repository
Create a feature branch
Write tests first
Implement the feature
Document your code
Submit a pull request

Code Style
rust// ✅ Good: Documented, error handling, tests
/// Calculate the sum of reserves
pub fn sum_reserves(reserves: &[U256]) -> Result<U256> {
    reserves
        .iter()
        .try_fold(U256::ZERO, |acc, &r| {
            acc.checked_add(r)
                .ok_or_else(|| OrbitalError::overflow("sum"))
        })
}

// ❌ Bad: No docs, unwrap, no error handling
pub fn sum_reserves(reserves: &[U256]) -> U256 {
    reserves.iter().fold(U256::ZERO, |acc, &r| acc + r)
}
📚 Resources
Essential Reading

Paradigm Orbital Paper - Original research
Orbswap Litepaper - Superellipse variant
Project Development Plan - Full roadmap

Code References

core/orbital-math/src/sphere.rs - Working spherical AMM
core/orbital-math/src/utils.rs - Utility functions
tests/ - Test examples

Getting Help

Read the existing documentation
Check the code comments
Look at test cases for examples
Open an issue on GitHub

🎯 Quick Wins
Want to make an immediate contribution? Try these:
Easy (1-2 hours)

 Add more test cases to sphere.rs
 Improve documentation in types.rs
 Create example programs
 Fix any TODOs in comments

Medium (1-2 days)

 Implement nth_root with better precision
 Add benchmarking for key functions
 Implement basic tick boundary checking
 Add property-based tests with proptest

Hard (1 week)

 Implement superellipse swap calculation
 Build quartic equation solver
 Optimize gas costs
 Add formal verification

🚦 Next Steps
Right now:

Build the project: cargo build
Run tests: cargo test
Read the code in sphere.rs

This week:

Understand the math
Pick a TODO to work on
Write tests
Implement and submit

This month:

Complete math library
Build pool contract
Deploy to testnet
Start frontend integration

💬 Community

GitHub: Repository Issues
Documentation: This repository
Code: core/orbital-math/


Welcome to the team! Let's build the future of AMMs together. 🚀
Last updated: January 2025