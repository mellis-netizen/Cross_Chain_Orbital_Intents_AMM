# Cross Chain Orbital Intents AMM

A revolutionary DeFi protocol implementing orbital mathematics for cross-chain intent-based trading with concentrated liquidity and MEV protection.

## Overview

Cross Chain Orbital Intents AMM is a next-generation automated market maker that combines:
- **Orbital Mathematics**: Novel curve shapes for optimal capital efficiency
- **Cross-Chain Intents**: Seamless trading across multiple chains
- **MEV Protection**: Built-in mechanisms to protect users from sandwich attacks
- **Concentrated Liquidity**: Capital-efficient liquidity provision

## Key Features

### Orbital AMM Mathematics
- Toroidal (donut-shaped) liquidity curves for better capital efficiency
- Superellipse curves for customizable trading dynamics
- Multi-dimensional pool support (up to 10 tokens)
- Dynamic fee adjustment based on market conditions

### Cross-Chain Intent System
- Intent-based trading across Ethereum, Arbitrum, Optimism, and Base
- Solver network for optimal execution paths
- Cryptographic proof validation for cross-chain settlements
- Slippage protection and price impact limits

### MEV Protection
- Commit-reveal trading mechanism
- Time-weighted average price (TWAP) oracles
- Private mempool integration
- Fair ordering protocols

### Advanced Features
- Concentrated liquidity with custom tick ranges
- Dynamic fees based on volatility
- Multi-token pools (up to 10 tokens)
- Solver reputation system
- Real-time analytics dashboard

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (Next.js)                    │
├─────────────────────────────────────────────────────────────┤
│                    Backend API (Rust)                        │
├─────────────────────────────────────────────────────────────┤
│                   Smart Contracts (Solidity)                 │
├─────────────────────────────────────────────────────────────┤
│                     Core Components                          │
│  ┌─────────────┬──────────────┬──────────────┬───────────┐ │
│  │   Orbital   │   Intent     │    Solver    │   Bridge  │ │
│  │     AMM     │   Engine     │   Network    │  Protocol │ │
│  └─────────────┴──────────────┴──────────────┴───────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
.
├── contracts/          # Smart contracts
│   ├── intents/       # Intent system contracts
│   └── orbital-amm/   # AMM contracts
├── core/              # Core Rust implementations
│   ├── bridge/        # Cross-chain bridge
│   ├── engine/        # Intent execution engine
│   └── solver/        # Solver network
├── orbital-math/      # Mathematical implementations
├── frontend/          # Next.js frontend
├── backend/           # API and indexer
└── docs/             # Documentation
```

## Installation

### Prerequisites
- Rust 1.70+
- Node.js 18+
- Foundry (for smart contracts)
- Docker (optional)

### Quick Start

1. Clone the repository:
```bash
git clone https://github.com/yourusername/Cross_Chain_Orbital_Intents_AMM.git
cd Cross_Chain_Orbital_Intents_AMM
```

2. Install dependencies:
```bash
# Rust dependencies
cargo build

# Frontend dependencies
cd frontend
npm install
```

3. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. Run tests:
```bash
# Rust tests
cargo test

# Contract tests
forge test

# Frontend tests
cd frontend && npm test
```

## Deployment

### Local Development
```bash
# Start local blockchain
anvil

# Deploy contracts
./scripts/deploy_local.sh

# Start backend
cargo run --bin api

# Start frontend
cd frontend && npm run dev
```

### Testnet Deployment
```bash
# Deploy to Holesky
./scripts/deploy_holesky.sh

# Deploy frontend to Netlify
./deploy-netlify.sh
```

### Production Deployment
See [DEPLOYMENT_README.md](./DEPLOYMENT_README.md) for detailed production deployment instructions.

## Usage Examples

### Creating an Intent
```typescript
const intent = {
  source_chain: 1,      // Ethereum
  dest_chain: 10,       // Optimism
  token_in: "USDC",
  token_out: "ETH",
  amount_in: "1000000000", // 1000 USDC
  min_amount_out: "400000000000000000", // 0.4 ETH
  deadline: Math.floor(Date.now() / 1000) + 3600
};

await orbitalAMM.createIntent(intent);
```

### Providing Liquidity
```typescript
await orbitalAMM.addLiquidity({
  pool: "ETH-USDC",
  amounts: ["1000000000000000000", "2000000000"], // 1 ETH, 2000 USDC
  tick_lower: -887220,
  tick_upper: 887220
});
```

### Swapping Tokens
```typescript
await orbitalAMM.swap({
  token_in: "USDC",
  token_out: "ETH",
  amount_in: "1000000000", // 1000 USDC
  min_amount_out: "400000000000000000" // 0.4 ETH
});
```

## Testing

```bash
# Run all tests
./scripts/test_all.sh

# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Contract tests
forge test -vvv

# Frontend tests
cd frontend && npm test
```

## Documentation

- [Architecture Overview](./docs/architecture/00-index.md)
- [Quick Reference](./docs/QUICK_REFERENCE.md)
- [Security Audit](./docs/SECURITY_AUDIT_REPORT.md)
- [API Documentation](./docs/api/README.md)

## Security

- All smart contracts are audited
- Bug bounty program active
- Security contact: security@orbitalamm.xyz

See [SECURITY.md](./SECURITY.md) for security policies and procedures.

## Roadmap

### Phase 1 (Complete) ✅
- [x] Orbital mathematics implementation
- [x] Basic AMM functionality
- [x] Cross-chain intent system
- [x] Holesky testnet deployment

### Phase 2 (In Progress) 🚧
- [ ] Mainnet deployment
- [ ] Advanced MEV protection
- [ ] Solver network expansion
- [ ] Mobile app

### Phase 3 (Planned) 📋
- [ ] Additional chain support
- [ ] Advanced trading strategies
- [ ] Institutional features
- [ ] DAO governance

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.


---
