#!/bin/bash
# Comprehensive test runner for Cross Chain Orbital Intents AMM

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_TIMEOUT=300
CARGO_FLAGS="--all-features"
COVERAGE_THRESHOLD=80

echo -e "${BLUE}🧪 Cross Chain Orbital Intents AMM - Comprehensive Test Suite${NC}"
echo "============================================================="

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust and Cargo.${NC}"
    exit 1
fi

# Function to run tests with timeout
run_test_suite() {
    local test_name="$1"
    local test_command="$2"
    local description="$3"
    
    echo -e "\n${YELLOW}📋 Running: $description${NC}"
    echo "Command: $test_command"
    echo "----------------------------------------"
    
    if timeout $TEST_TIMEOUT bash -c "$test_command"; then
        echo -e "${GREEN}✅ $test_name passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name failed${NC}"
        return 1
    fi
}

# Initialize test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 1. Unit Tests
echo -e "\n${BLUE}🔬 Unit Tests${NC}"
echo "=============="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "unit_tests" "cargo test --workspace --lib $CARGO_FLAGS" "Unit tests for all workspace members"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 2. Integration Tests
echo -e "\n${BLUE}🔗 Integration Tests${NC}"
echo "==================="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "integration_tests" "cargo test --workspace --test '*' $CARGO_FLAGS" "Integration tests across all modules"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 3. Documentation Tests
echo -e "\n${BLUE}📚 Documentation Tests${NC}"
echo "======================"

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "doc_tests" "cargo test --workspace --doc $CARGO_FLAGS" "Documentation examples and tests"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 4. Orbital Math Specific Tests
echo -e "\n${BLUE}🌌 Orbital Math Tests${NC}"
echo "====================="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "orbital_math_tests" "cd orbital-math && cargo test $CARGO_FLAGS" "Orbital mathematics library tests"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 5. Cross-Chain Bridge Tests
echo -e "\n${BLUE}🌉 Cross-Chain Bridge Tests${NC}"
echo "==========================="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "bridge_tests" "cd core/bridge && cargo test $CARGO_FLAGS" "Cross-chain bridge functionality"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 6. Solver Network Tests
echo -e "\n${BLUE}🤖 Solver Network Tests${NC}"
echo "======================="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "solver_tests" "cd core/solver && cargo test $CARGO_FLAGS" "Solver network and reputation system"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 7. Intent Engine Tests
echo -e "\n${BLUE}⚙️ Intent Engine Tests${NC}"
echo "======================"

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "engine_tests" "cd core/engine && cargo test $CARGO_FLAGS" "Intent creation and execution engine"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 8. Security Tests
echo -e "\n${BLUE}🔒 Security Tests${NC}"
echo "=================="

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if run_test_suite "security_tests" "cargo test --workspace $CARGO_FLAGS security" "Security vulnerability tests"; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# 9. Performance Benchmarks (if criterion is available)
echo -e "\n${BLUE}⚡ Performance Benchmarks${NC}"
echo "========================="

if command -v cargo &> /dev/null && cargo bench --version &> /dev/null; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if run_test_suite "benchmarks" "cargo bench --workspace $CARGO_FLAGS" "Performance benchmarks"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  Skipping benchmarks (criterion not available)${NC}"
fi

# 10. Test Coverage (if tarpaulin is available)
echo -e "\n${BLUE}📊 Test Coverage Analysis${NC}"
echo "========================="

if command -v cargo-tarpaulin &> /dev/null; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if run_test_suite "coverage" "cargo tarpaulin --workspace --out Html --output-dir target/coverage --timeout $TEST_TIMEOUT" "Test coverage analysis"; then
        echo -e "${GREEN}📈 Coverage report generated in target/coverage/tarpaulin-report.html${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
else
    echo -e "${YELLOW}⚠️  Test coverage analysis skipped (install with: cargo install cargo-tarpaulin)${NC}"
fi

# Final Results
echo -e "\n${BLUE}📋 Test Results Summary${NC}"
echo "======================="
echo -e "Total Test Suites: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed successfully!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ $FAILED_TESTS test suite(s) failed.${NC}"
    exit 1
fi