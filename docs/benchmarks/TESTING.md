# Cross Chain Orbital Intents AMM - Testing Guide

This document provides comprehensive information about testing the Cross Chain Orbital Intents AMM system.

## Table of Contents

- [Overview](#overview)
- [Test Architecture](#test-architecture)
- [Running Tests](#running-tests)
- [Test Types](#test-types)
- [Coverage Requirements](#coverage-requirements)
- [CI/CD Pipeline](#cicd-pipeline)
- [Contributing](#contributing)

## Overview

Our testing strategy follows a comprehensive multi-layer approach:

1. **Unit Tests** - Test individual functions and modules
2. **Integration Tests** - Test component interactions
3. **End-to-End Tests** - Test complete user workflows
4. **Property-Based Tests** - Test mathematical invariants
5. **Security Tests** - Test for vulnerabilities
6. **Performance Tests** - Test system performance
7. **Smart Contract Tests** - Test blockchain contracts

## Test Architecture

```
tests/
├── unit/                   # Unit tests for individual components
├── integration/            # Integration tests for component interactions
├── e2e/                   # End-to-end tests for complete workflows
├── property/              # Property-based tests
├── security/              # Security vulnerability tests
├── performance/           # Performance benchmarks
└── contracts/             # Smart contract tests (Foundry)

frontend/
├── src/
│   └── components/
│       └── __tests__/     # React component tests
├── tests/
│   ├── integration/       # Frontend integration tests
│   └── e2e/              # Playwright E2E tests
└── jest.config.js         # Jest configuration
```

## Running Tests

### Prerequisites

1. **Rust** (latest stable)
2. **Node.js** (v18 or later)
3. **Foundry** (for smart contract tests)
4. **Docker** (for integration tests)

### Quick Start

```bash
# Run all tests
./scripts/run_tests.sh

# Run specific test suites
cargo test --workspace            # Rust tests
npm test --prefix frontend       # Frontend tests
forge test                       # Smart contract tests
```

### Detailed Test Commands

#### Rust Tests

```bash
# Unit tests
cargo test --workspace --lib --all-features

# Integration tests
cargo test --workspace --test '*' --all-features

# Documentation tests
cargo test --workspace --doc --all-features

# Specific module tests
cargo test --package intents-engine
cargo test --package intents-solver
cargo test --package orbital-math

# Property-based tests
cargo test property_tests --all-features

# Security tests
cargo test security_tests --all-features

# Performance benchmarks
cargo bench --workspace --all-features
```

#### Frontend Tests

```bash
cd frontend

# Unit and component tests
npm test

# Watch mode
npm run test:watch

# Coverage report
npm run test:coverage

# Component tests only
npm run test:component

# Integration tests
npm run test:integration

# End-to-end tests
npm run test:e2e

# E2E tests with UI
npm run test:e2e:ui
```

#### Smart Contract Tests

```bash
# Install dependencies
forge install

# Build contracts
forge build

# Run all tests
forge test

# Verbose output
forge test -vvv

# Gas reporting
forge test --gas-report

# Coverage analysis
forge coverage

# Fuzz testing
forge test --fuzz-runs 10000
```

## Test Types

### 1. Unit Tests

Test individual functions and modules in isolation.

**Location**: `tests/`, `core/*/tests/`, `orbital-math/src/`

**Example**:
```rust
#[test]
fn test_swap_calculation() {
    let pool = create_test_pool();
    let output = pool.calculate_swap_output(1000, true);
    assert_eq!(output, 990); // Expected output
}
```

### 2. Integration Tests

Test interactions between multiple components.

**Location**: `tests/integration_tests.rs`, `frontend/tests/integration/`

**Coverage**:
- Intent lifecycle (creation → matching → execution → settlement)
- Cross-chain message passing
- Solver network interactions
- Frontend-backend integration

### 3. End-to-End Tests

Test complete user workflows using Playwright.

**Location**: `frontend/tests/e2e/`

**Scenarios**:
- Wallet connection
- Token swapping
- Liquidity provision
- Cross-chain transactions
- Error handling

### 4. Property-Based Tests

Test mathematical properties and invariants using Proptest.

**Location**: `tests/property_tests.rs`

**Properties Tested**:
- K invariant (x * y ≥ constant)
- Price monotonicity
- Fee accumulation
- Balance conservation
- Overflow protection

**Example**:
```rust
proptest! {
    #[test]
    fn prop_k_invariant_holds(
        amount_in in 1u64..=10_000u64,
        token_in_is_a in any::<bool>()
    ) {
        let pool = create_test_pool();
        let k_before = pool.reserve_a * pool.reserve_b;
        
        pool.swap(amount_in, token_in_is_a);
        
        let k_after = pool.reserve_a * pool.reserve_b;
        prop_assert!(k_after >= k_before);
    }
}
```

### 5. Security Tests

Test for common vulnerabilities and attack vectors.

**Location**: `tests/security_tests.rs`

**Tests Cover**:
- Reentrancy protection
- Overflow/underflow protection
- Authorization checks
- Replay attack prevention
- MEV protection
- Front-running protection

### 6. Performance Tests

Benchmark critical performance metrics.

**Location**: `benches/performance_benchmarks.rs`

**Benchmarks**:
- Swap calculations
- Intent matching
- Cross-chain processing
- Memory allocation patterns
- Concurrent operations

**Example**:
```rust
fn benchmark_swap_calculation(c: &mut Criterion) {
    let pool = BenchmarkPool::new();
    
    c.bench_function("swap_calculation", |b| {
        b.iter(|| {
            pool.calculate_swap_output(black_box(1000), black_box(true))
        });
    });
}
```

### 7. Smart Contract Tests

Test blockchain smart contracts using Foundry.

**Location**: `test/`

**Coverage**:
- Pool creation and initialization
- Swap execution
- Liquidity management
- Fee calculations
- Access control
- Upgrade mechanisms

## Coverage Requirements

### Minimum Coverage Targets

- **Rust Core**: 85%
- **Frontend**: 80%
- **Smart Contracts**: 85%
- **Integration Tests**: 75%

### Coverage Reporting

```bash
# Rust coverage
cargo tarpaulin --out Html --output-dir target/coverage

# Frontend coverage
npm run test:coverage

# Smart contract coverage
forge coverage --report lcov
```

### Viewing Coverage Reports

- **Rust**: `target/coverage/tarpaulin-report.html`
- **Frontend**: `frontend/coverage/lcov-report/index.html`
- **Contracts**: Coverage data in `lcov.info`

## CI/CD Pipeline

Our GitHub Actions pipeline runs comprehensive tests on every push and PR:

### Test Jobs

1. **rust-tests**: Unit and integration tests for all Rust code
2. **frontend-tests**: React component and integration tests
3. **e2e-tests**: Playwright end-to-end tests
4. **smart-contract-tests**: Foundry smart contract tests
5. **security-tests**: Security vulnerability scans
6. **performance-tests**: Performance benchmarks
7. **property-tests**: Property-based testing
8. **integration-tests**: Cross-chain integration tests

### Quality Gates

Tests must pass the following criteria to merge:

- ✅ All test suites pass
- ✅ Code coverage meets minimum thresholds
- ✅ Security scans pass
- ✅ Performance benchmarks within acceptable ranges
- ✅ No critical or high-severity vulnerabilities

## Test Data and Fixtures

### Mock Data

- **Tokens**: Mock ERC20 tokens for testing
- **Pools**: Pre-configured pools with different parameters
- **Users**: Test accounts with various balances
- **Solvers**: Mock solver network participants

### Test Networks

- **Local**: Hardhat/Anvil local blockchain
- **Testnets**: Holesky, Mumbai for integration testing
- **Staging**: Dedicated staging environment

## Performance Benchmarks

### Target Metrics

- **Swap Calculation**: < 1ms
- **Intent Matching**: < 100ms for 1000 solvers
- **Cross-chain Message**: < 5s end-to-end
- **Frontend Rendering**: < 100ms for component updates
- **Memory Usage**: < 512MB for full system

### Running Benchmarks

```bash
# Rust benchmarks
cargo bench --all-features

# Frontend performance
npm run test:performance

# Load testing
npm run test:load
```

## Debugging Tests

### Common Issues

1. **Flaky Tests**: Use `serial_test` for tests that can't run in parallel
2. **Timing Issues**: Use `tokio::time::sleep` for async timing
3. **State Pollution**: Ensure proper cleanup between tests
4. **Mock Setup**: Verify mocks are properly configured

### Debugging Tools

```bash
# Rust debugging
RUST_LOG=debug cargo test test_name -- --nocapture

# Frontend debugging
npm test -- --verbose --no-coverage

# E2E debugging
npm run test:e2e -- --debug
```

## Contributing

### Before Submitting PR

1. Run full test suite: `./scripts/run_tests.sh`
2. Check coverage: Tests must not decrease overall coverage
3. Add tests for new features
4. Update documentation for test changes

### Test Writing Guidelines

1. **Test Naming**: Use descriptive names (e.g., `test_swap_with_insufficient_balance`)
2. **Test Structure**: Follow Arrange-Act-Assert pattern
3. **Mock Usage**: Mock external dependencies, test real logic
4. **Error Cases**: Test both success and failure scenarios
5. **Edge Cases**: Test boundary conditions and edge cases

### Adding New Tests

1. **Unit Tests**: Add to appropriate module's `tests/` directory
2. **Integration Tests**: Add to `tests/integration_tests.rs`
3. **E2E Tests**: Add to `frontend/tests/e2e/`
4. **Property Tests**: Add to `tests/property_tests.rs`

## Troubleshooting

### Common Test Failures

#### Rust Tests

```bash
# Dependency issues
cargo clean && cargo build

# Test isolation
cargo test -- --test-threads=1

# Memory issues
export RUST_MIN_STACK=8388608
```

#### Frontend Tests

```bash
# Node modules
rm -rf node_modules && npm install

# Jest cache
npm test -- --clearCache

# Playwright issues
npx playwright install
```

#### Smart Contract Tests

```bash
# Foundry cache
forge clean

# Dependencies
forge install

# Gas estimation
forge test --gas-limit 30000000
```

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Playwright Documentation](https://playwright.dev/)
- [Foundry Book](https://book.getfoundry.sh/)
- [Proptest Guide](https://proptest-rs.github.io/proptest/)

## Support

For testing-related questions:

1. Check this documentation
2. Search existing GitHub issues
3. Create a new issue with the `testing` label
4. Join our Discord for real-time help

---

*This testing guide is maintained by the QA Engineering team. Last updated: 2024-10-02*