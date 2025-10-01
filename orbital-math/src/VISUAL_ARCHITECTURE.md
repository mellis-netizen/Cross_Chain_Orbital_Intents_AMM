Orbital AMM Visual Architecture Guide
System Overview
┌─────────────────────────────────────────────────────────────────┐
│                     Cross-Chain Orbital AMM                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐  │
│  │   Frontend   │◄────►│   Backend    │◄────►│  Blockchain  │  │
│  │              │      │   Services   │      │   Contracts  │  │
│  │  Next.js +   │      │              │      │              │  │
│  │  React       │      │  Intent      │      │  Orbital AMM │  │
│  │              │      │  Engine      │      │  (Stylus)    │  │
│  │  Wallet      │      │              │      │              │  │
│  │  Integration │      │  Solver      │      │  Intents     │  │
│  │              │      │  Network     │      │  System      │  │
│  └──────────────┘      └──────────────┘      └──────────────┘  │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
Core Components
1. Orbital Math Library (orbital-math)
┌──────────────────────────────────────────────────────────┐
│              orbital-math Core Library                    │
├──────────────────────────────────────────────────────────┤
│                                                            │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   │
│  │   sphere    │   │ superellipse│   │    ticks    │   │
│  │             │   │             │   │             │   │
│  │ • Verify    │   │ • Flattened │   │ • Boundary  │   │
│  │   Σ(r²)=R²  │   │   curves    │   │   checks    │   │
│  │ • Swaps     │   │ • Stable    │   │ • Crossing  │   │
│  │ • Prices    │   │   optimized │   │   detection │   │
│  │ • Impact    │   │             │   │ • Efficiency│   │
│  └─────────────┘   └─────────────┘   └─────────────┘   │
│                                                            │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   │
│  │   trades    │   │    utils    │   │    types    │   │
│  │             │   │             │   │             │   │
│  │ • Toroidal  │   │ • Power     │   │ • Reserve   │   │
│  │   invariant │   │ • Sqrt      │   │   Point     │   │
│  │ • Quartic   │   │ • Dot prod  │   │ • Tick      │   │
│  │   solver    │   │ • Lerp      │   │ • PoolState │   │
│  │ • Segment   │   │             │   │ • TradeInfo │   │
│  └─────────────┘   └─────────────┘   └─────────────┘   │
│                                                            │
└──────────────────────────────────────────────────────────┘
2. N-Dimensional Pool Geometry
2D (Circle)
      r₁
      ↑
      |     ●
      |   ●   ●
      |  ●     ●  ← Reserves on circle: r₀² + r₁² = R²
      | ●       ●
      |●─────────●──→ r₀
3D (Sphere)
       r₂
       ↑
      ╱│╲
     ╱ │ ╲
    ╱  │  ╲     ← Reserves on sphere: r₀² + r₁² + r₂² = R²
   ╱   ●   ╲
  ╱  ╱   ╲  ╲
 ╱  ╱     ╲  ╲
╱──╱───────╲──╲→ r₁
  ╱         ╲
 ↙           ↘
r₀
N-D (Hypersphere)
Σ(rᵢ²) = R²  for i = 0 to N-1

All reserve states constrained to N-dimensional sphere surface
3. Tick System
┌────────────────────────────────────────────────────────┐
│            Nested Tick Structure                        │
│                                                          │
│         Equal Price Point                               │
│               ●                                         │
│           ┌───┴───┐                                     │
│          ╱         ╲                                    │
│         ╱  Tick 1   ╲     ← Tightest: 99% depeg limit  │
│        ╱   (Inner)   ╲                                  │
│       ╱               ╲                                 │
│      ╱   ┌─────────┐   ╲                               │
│     ╱   ╱  Tick 2   ╲   ╲  ← Medium: 95% depeg limit   │
│    ╱   ╱  (Middle)   ╲   ╲                             │
│   ╱   ╱               ╲   ╲                            │
│  ╱   ╱   ┌─────────┐   ╲   ╲                           │
│ ╱   ╱   ╱  Tick 3   ╲   ╲   ╲ ← Widest: 90% depeg      │
│╱   ╱   ╱  (Outer)    ╲   ╲   ╲                         │
│   ╱   ╱               ╲   ╲   ╲                        │
│      ╱                 ╲                                │
│                                                          │
│ Properties:                                             │
│ • Ticks are NESTED (not disjoint like Uni V3)          │
│ • Smaller ticks = higher capital efficiency            │
│ • LPs choose tick based on risk tolerance              │
└────────────────────────────────────────────────────────┘
4. Toroidal Trading Surface
┌────────────────────────────────────────────────────────┐
│           Torus (Donut) in 3D                           │
│                                                          │
│              ╭─────────╮                                │
│            ╱           ╲                                │
│          ╱   ●────●────● ╲   ← Boundary ticks (circle)│
│         │   ╱           ╲ │                             │
│         │  ╱             ╲│                             │
│         │ │   Interior    ││   ← Interior ticks (sphere)│
│         │ │   ticks       ││                            │
│         │  ╲             ╱│                             │
│         │   ╲           ╱ │                             │
│          ╲   ●────●────●  ╱                             │
│            ╲           ╱                                │
│              ╰─────────╯                                │
│                                                          │
│ Combined Invariant:                                     │
│ Interior: Σ(rᵢ²) = R²         (N-sphere)               │
│ Boundary: Σ(rᵢ² for i≠j) = C  (N-1 sphere)             │
│ Result: Toroidal surface                                │
└────────────────────────────────────────────────────────┘
5. Swap Execution Flow
┌─────────────────────────────────────────────────────────┐
│                  Swap Execution                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  1. User Request                                         │
│     │                                                    │
│     ├─► Amount In: 1000 USDC                            │
│     ├─► Token Out: DAI                                  │
│     └─► Min Out: 995 DAI                                │
│                                                           │
│  2. Calculate Output                                     │
│     │                                                    │
│     ├─► Check current reserves                          │
│     ├─► Consolidate interior ticks → sphere             │
│     ├─► Consolidate boundary ticks → circle             │
│     ├─► Compute toroidal invariant                      │
│     └─► Solve quartic equation                          │
│                                                           │
│  3. Check Tick Crossings                                │
│     │                                                    │
│     ├─► Compute new position                            │
│     ├─► Detect boundary crossings                       │
│     └─► Segment trade if needed                         │
│                                                           │
│  4. Execute Trade                                        │
│     │                                                    │
│     ├─► Update reserves                                 │
│     ├─► Update tick states                              │
│     ├─► Verify invariant                                │
│     ├─► Check slippage                                  │
│     └─► Emit events                                     │
│                                                           │
│  5. Return Results                                       │
│     │                                                    │
│     ├─► Amount Out: 998 DAI                             │
│     ├─► Price Impact: 0.2%                              │
│     ├─► Fee: 3 USDC                                     │
│     └─► Ticks Crossed: 1                                │
│                                                           │
└─────────────────────────────────────────────────────────┘
6. Capital Efficiency Comparison
┌──────────────────────────────────────────────────────────┐
│         Capital Efficiency vs Depeg Limit                 │
│                                                            │
│  Efficiency                                               │
│     ↑                                                     │
│ 150x┤                     ●                              │
│     │                    ╱                               │
│ 100x┤                  ●                                 │
│     │                 ╱                                  │
│  50x┤               ●                                    │
│     │              ╱                                     │
│  25x┤            ●                                       │
│     │          ╱                                         │
│  15x┤       ●                                            │
│     │      ╱                                             │
│   1x┤●────────────────────────────────────→             │
│     └────────────────────────────────────               │
│     $0.80  $0.85  $0.90  $0.95  $0.99 Depeg Limit      │
│                                                            │
│  Interpretation:                                          │
│  • 99% limit = 150x efficiency (great for stable pairs) │
│  • 95% limit = 15x efficiency (balanced risk/reward)    │
│  • 80% limit = 1x efficiency (no advantage)             │
└──────────────────────────────────────────────────────────┘
7. Cross-Chain Intent Flow
┌──────────────────────────────────────────────────────────────┐
│              Cross-Chain Swap Intent                          │
├──────────────────────────────────────────────────────────────┤
│                                                                │
│  User (Ethereum)          Solver Network          Arbitrum   │
│       │                        │                      │       │
│       │  1. Create Intent      │                      │       │
│       ├───────────────────────►│                      │       │
│       │   "Swap 1000 USDC      │                      │       │
│       │    for DAI on Arbitrum"│                      │       │
│       │                        │                      │       │
│       │  2. Intent Broadcast   │                      │       │
│       │                        │                      │       │
│       │       ┌────────────────┴─────────────┐       │       │
│       │       │ Solver A: 995 DAI, 500ms     │       │       │
│       │       │ Solver B: 997 DAI, 300ms     │◄──Best│       │
│       │       │ Solver C: 994 DAI, 200ms     │       │       │
│       │       └────────────────┬─────────────┘       │       │
│       │                        │                      │       │
│       │  3. Lock Funds         │                      │       │
│       ├───────────────────────►│                      │       │
│       │   Lock 1000 USDC       │                      │       │
│       │                        │                      │       │
│       │  4. Execute on Dest    │                      │       │
│       │                        ├─────────────────────►│       │
│       │                        │  Swap via Orbital    │       │
│       │                        │  Pool                │       │
│       │                        │                      │       │
│       │  5. Proof of Execution │                      │       │
│       │                        │◄─────────────────────┤       │
│       │                        │  Merkle Proof        │       │
│       │                        │                      │       │
│       │  6. Verify & Release   │                      │       │
│       │◄───────────────────────┤                      │       │
│       │   997 DAI delivered    │                      │       │
│       │                        │                      │       │
└──────────────────────────────────────────────────────────────┘
8. Data Flow in orbital-math
┌────────────────────────────────────────────────────────┐
│            Data Flow: User Swap Request                 │
├────────────────────────────────────────────────────────┤
│                                                          │
│  Input: SwapRequest                                     │
│    │                                                    │
│    ├─► pool_id: 123                                    │
│    ├─► token_in: 0 (USDC)                              │
│    ├─► token_out: 2 (DAI)                              │
│    ├─► amount_in: 1000                                 │
│    └─► min_amount_out: 995                             │
│              │                                          │
│              ↓                                          │
│         Load PoolState                                  │
│              │                                          │
│    ┌─────────┴─────────┐                              │
│    │ reserves: [1M,1M,1M,1M,1M]                        │
│    │ invariant: R² = 5M²                               │
│    │ curve_type: Sphere                                │
│    │ ticks: [Tick1, Tick2, Tick3]                      │
│    └─────────┬─────────┘                              │
│              ↓                                          │
│      sphere::calculate_amount_out_sphere()             │
│              │                                          │
│    ┌─────────┴─────────┐                              │
│    │ 1. Verify inputs                                  │
│    │ 2. Calculate new reserves                         │
│    │ 3. Solve: (r₀+1000)² + r₁² + (r₂-x)² + ...= R²  │
│    │ 4. x = r₂ - sqrt(R² - sum_others)                │
│    └─────────┬─────────┘                              │
│              ↓                                          │
│        amount_out = 998                                │
│              │                                          │
│              ↓                                          │
│      Calculate price_impact                            │
│              │                                          │
│    ┌─────────┴─────────┐                              │
│    │ price_before = r₀/r₂                             │
│    │ price_after = (r₀+1000)/(r₂-998)                 │
│    │ impact = |before-after|/before                    │
│    └─────────┬─────────┘                              │
│              ↓                                          │
│        price_impact = 0.2%                             │
│              │                                          │
│              ↓                                          │
│     Check slippage (998 >= 995) ✓                     │
│              │                                          │
│              ↓                                          │
│      Update PoolState                                  │
│              │                                          │
│    ┌─────────┴─────────┐                              │
│    │ reserves: [1M+1k, 1M, 1M-998, 1M, 1M]            │
│    │ verify_sphere_constraint() ✓                      │
│    └─────────┬─────────┘                              │
│              ↓                                          │
│  Output: TradeInfo                                     │
│    │                                                    │
│    ├─► amount_out: 998                                 │
│    ├─► price_before: 1.0                               │
│    ├─► price_after: 1.002                              │
│    ├─► price_impact_bp: 20                             │
│    ├─► ticks_crossed: 0                                │
│    └─► fee: 3                                          │
│                                                          │
└────────────────────────────────────────────────────────┘
Mathematical Formulas
Sphere Constraint
┌─────────────────────────────────────────┐
│  Σ(rᵢ²) = R²                            │
│  i=0 to N-1                             │
│                                          │
│  All reserve states must lie on the     │
│  surface of an N-dimensional sphere     │
└─────────────────────────────────────────┘
Swap Calculation
┌─────────────────────────────────────────┐
│  Given: Add Δᵢₙ to token i              │
│  Find: Δₒᵤₜ to remove from token j      │
│                                          │
│  (rᵢ + Δᵢₙ)² + Σ(rₖ² for k≠i,j) +     │
│               (rⱼ - Δₒᵤₜ)² = R²        │
│                                          │
│  Solve for Δₒᵤₜ:                        │
│  Δₒᵤₜ = rⱼ - √(R² - Σ(rₖ² for k≠j))   │
└─────────────────────────────────────────┘
Price Formula
┌─────────────────────────────────────────┐
│  Instantaneous price:                    │
│                                          │
│  ∂rⱼ/∂rᵢ = -rᵢ/rⱼ                       │
│                                          │
│  Price of token j in terms of i:        │
│  P = rᵢ / rⱼ                            │
└─────────────────────────────────────────┘
Tick Boundary
┌─────────────────────────────────────────┐
│  Hyperplane boundary:                    │
│                                          │
│  r⃗ · 1⃗ = c√N                           │
│                                          │
│  where 1⃗ = (1,1,...,1)                 │
│       c = plane constant                │
│       N = number of tokens              │
└─────────────────────────────────────────┘
Capital Efficiency
┌─────────────────────────────────────────┐
│  For tick with depeg limit p:           │
│                                          │
│  Efficiency = (c + √(R²-c²/(N-1))) /    │
│               √(R²-c²/(N-1))            │
│                                          │
│  Example (N=5, p=0.95):                 │
│  Efficiency ≈ 15x                       │
└─────────────────────────────────────────┘
Implementation Status
✅ Completed (Green)

Spherical AMM mathematics
Swap calculations (2D to N-D)
Price impact calculations
Utility functions
Error handling
Test coverage > 95%

🔨 In Progress (Yellow)

Superellipse curves
Tick geometry
Toroidal invariant
Quartic solver

⏳ Planned (Gray)

Multi-tick optimization
Gas optimization
Formal verification
Production audit

Next Steps

Complete superellipse module

Implement fractional powers
Optimize for stablecoins


Build tick system

Boundary checking
Crossing detection
Capital efficiency


Implement toroidal trades

Consolidate ticks
Solve quartic
Handle crossings


Deploy testnet

Integration testing
Gas benchmarks
Security review




This is a living document - Updates as implementation progresses
Last updated: January 2025