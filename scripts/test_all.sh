#!/bin/bash

# Cross Chain Orbital Intents AMM - Comprehensive Test Suite
# This script runs all tests across the entire project

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Test configuration
PARALLEL="${PARALLEL:-true}"
COVERAGE="${COVERAGE:-false}"
VERBOSE="${VERBOSE:-false}"
SKIP_INTEGRATION="${SKIP_INTEGRATION:-false}"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Progress tracking
declare -a test_results
declare -a test_names

add_test_result() {
    test_names+=("$1")
    test_results+=("$2")
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking test prerequisites..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found"
        exit 1
    fi
    
    # Check Foundry
    if ! command -v forge &> /dev/null; then
        log_error "Foundry not found"
        exit 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js not found"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Rust unit tests
run_rust_unit_tests() {
    log_info "Running Rust unit tests..."
    
    cd "$PROJECT_ROOT"
    
    local test_cmd="cargo test --lib"
    
    if [ "$VERBOSE" = "true" ]; then
        test_cmd="$test_cmd -- --nocapture"
    fi
    
    if [ "$PARALLEL" = "false" ]; then
        test_cmd="$test_cmd -- --test-threads=1"
    fi
    
    if eval "$test_cmd"; then
        log_success "Rust unit tests passed"
        add_test_result "rust_unit" "PASS"
        return 0
    else
        log_error "Rust unit tests failed"
        add_test_result "rust_unit" "FAIL"
        return 1
    fi
}

# Rust integration tests
run_rust_integration_tests() {
    if [ "$SKIP_INTEGRATION" = "true" ]; then
        log_info "Skipping integration tests"
        add_test_result "rust_integration" "SKIP"
        return 0
    fi
    
    log_info "Running Rust integration tests..."
    
    cd "$PROJECT_ROOT"
    
    local test_cmd="cargo test --test '*'"
    
    if [ "$VERBOSE" = "true" ]; then
        test_cmd="$test_cmd -- --nocapture"
    fi
    
    if [ "$PARALLEL" = "false" ]; then
        test_cmd="$test_cmd -- --test-threads=1"
    fi
    
    if eval "$test_cmd"; then
        log_success "Rust integration tests passed"
        add_test_result "rust_integration" "PASS"
        return 0
    else
        log_error "Rust integration tests failed"
        add_test_result "rust_integration" "FAIL"
        return 1
    fi
}

# Rust documentation tests
run_rust_doc_tests() {
    log_info "Running Rust documentation tests..."
    
    cd "$PROJECT_ROOT"
    
    if cargo test --doc; then
        log_success "Rust documentation tests passed"
        add_test_result "rust_doc" "PASS"
        return 0
    else
        log_error "Rust documentation tests failed"
        add_test_result "rust_doc" "FAIL"
        return 1
    fi
}

# Smart contract tests
run_contract_tests() {
    log_info "Running smart contract tests..."
    
    # Test intents contracts
    cd "$PROJECT_ROOT/contracts/intents"
    if [ -f "Cargo.toml" ]; then
        if cargo test; then
            log_success "Intent contracts tests passed"
        else
            log_error "Intent contracts tests failed"
            add_test_result "contract_intents" "FAIL"
            return 1
        fi
    fi
    
    # Test Orbital AMM contracts
    cd "$PROJECT_ROOT/contracts/orbital-amm"
    if [ -f "foundry.toml" ] || [ -f "forge.toml" ]; then
        local forge_cmd="forge test"
        
        if [ "$VERBOSE" = "true" ]; then
            forge_cmd="$forge_cmd -vvv"
        fi
        
        if eval "$forge_cmd"; then
            log_success "Orbital AMM contracts tests passed"
            add_test_result "contract_orbital" "PASS"
            return 0
        else
            log_error "Orbital AMM contracts tests failed"
            add_test_result "contract_orbital" "FAIL"
            return 1
        fi
    else
        log_warning "No Foundry configuration found for Orbital AMM contracts"
        add_test_result "contract_orbital" "SKIP"
        return 0
    fi
}

# Frontend tests
run_frontend_tests() {
    log_info "Running frontend tests..."
    
    cd "$PROJECT_ROOT/frontend"
    
    if [ ! -f "package.json" ]; then
        log_warning "No frontend package.json found"
        add_test_result "frontend" "SKIP"
        return 0
    fi
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing frontend dependencies..."
        npm ci
    fi
    
    # Run linting
    if npm run lint > /dev/null 2>&1; then
        log_success "Frontend linting passed"
    else
        log_warning "Frontend linting failed or not configured"
    fi
    
    # Run type checking
    if npm run type-check > /dev/null 2>&1; then
        log_success "Frontend type checking passed"
    else
        log_warning "Frontend type checking failed or not configured"
    fi
    
    # Run tests
    local test_cmd="npm test"
    
    if [ "$COVERAGE" = "true" ]; then
        test_cmd="$test_cmd -- --coverage"
    fi
    
    if eval "$test_cmd"; then
        log_success "Frontend tests passed"
        add_test_result "frontend" "PASS"
        return 0
    else
        log_error "Frontend tests failed"
        add_test_result "frontend" "FAIL"
        return 1
    fi
}

# Orbital math specific tests
run_orbital_math_tests() {
    log_info "Running orbital mathematics tests..."
    
    cd "$PROJECT_ROOT/orbital-math"
    
    local test_cmd="cargo test"
    
    if [ "$VERBOSE" = "true" ]; then
        test_cmd="$test_cmd -- --nocapture"
    fi
    
    if eval "$test_cmd"; then
        log_success "Orbital math tests passed"
        add_test_result "orbital_math" "PASS"
        return 0
    else
        log_error "Orbital math tests failed"
        add_test_result "orbital_math" "FAIL"
        return 1
    fi
}

# Cross-chain simulation tests
run_cross_chain_tests() {
    if [ "$SKIP_INTEGRATION" = "true" ]; then
        log_info "Skipping cross-chain tests"
        add_test_result "cross_chain" "SKIP"
        return 0
    fi
    
    log_info "Running cross-chain simulation tests..."
    
    cd "$PROJECT_ROOT"
    
    # Check if cross-chain test file exists
    if [ -f "tests/cross_chain_tests.rs" ]; then
        local test_cmd="cargo test --test cross_chain_tests"
        
        if [ "$VERBOSE" = "true" ]; then
            test_cmd="$test_cmd -- --nocapture"
        fi
        
        if eval "$test_cmd"; then
            log_success "Cross-chain tests passed"
            add_test_result "cross_chain" "PASS"
            return 0
        else
            log_error "Cross-chain tests failed"
            add_test_result "cross_chain" "FAIL"
            return 1
        fi
    else
        log_warning "Cross-chain test file not found"
        add_test_result "cross_chain" "SKIP"
        return 0
    fi
}

# Performance benchmarks
run_benchmarks() {
    log_info "Running performance benchmarks..."
    
    cd "$PROJECT_ROOT"
    
    # Check if benchmarks are configured
    if grep -q "bench" Cargo.toml; then
        if cargo bench; then
            log_success "Benchmarks completed"
            add_test_result "benchmarks" "PASS"
            return 0
        else
            log_warning "Benchmarks failed"
            add_test_result "benchmarks" "FAIL"
            return 1
        fi
    else
        log_info "No benchmarks configured"
        add_test_result "benchmarks" "SKIP"
        return 0
    fi
}

# Code coverage report
generate_coverage() {
    if [ "$COVERAGE" = "false" ]; then
        return 0
    fi
    
    log_info "Generating code coverage report..."
    
    cd "$PROJECT_ROOT"
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_info "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi
    
    if cargo tarpaulin --out Html --output-dir coverage; then
        log_success "Coverage report generated in coverage/tarpaulin-report.html"
        return 0
    else
        log_warning "Coverage generation failed"
        return 1
    fi
}

# Security audit
run_security_audit() {
    log_info "Running security audit..."
    
    cd "$PROJECT_ROOT"
    
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        log_info "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    if cargo audit; then
        log_success "Security audit passed"
        add_test_result "security_audit" "PASS"
        return 0
    else
        log_warning "Security audit found issues"
        add_test_result "security_audit" "WARN"
        return 1
    fi
}

# Generate test report
generate_test_report() {
    log_info "Generating test report..."
    
    local report_file="$PROJECT_ROOT/test-report.md"
    local timestamp=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
    
    cat > "$report_file" << EOF
# Test Report

**Generated:** $timestamp
**Environment:** $(uname -s) $(uname -r)
**Rust Version:** $(rustc --version)
**Node Version:** $(node --version 2>/dev/null || echo "N/A")

## Test Results

| Test Suite | Status |
|------------|--------|
EOF
    
    for i in "${!test_names[@]}"; do
        local name="${test_names[$i]}"
        local result="${test_results[$i]}"
        local icon
        
        case "$result" in
            "PASS") icon="✅" ;;
            "FAIL") icon="❌" ;;
            "SKIP") icon="⏭️" ;;
            "WARN") icon="⚠️" ;;
            *) icon="❓" ;;
        esac
        
        echo "| $name | $icon $result |" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Summary

EOF
    
    local total=${#test_results[@]}
    local passed=$(printf '%s\n' "${test_results[@]}" | grep -c "PASS" || true)
    local failed=$(printf '%s\n' "${test_results[@]}" | grep -c "FAIL" || true)
    local skipped=$(printf '%s\n' "${test_results[@]}" | grep -c "SKIP" || true)
    local warnings=$(printf '%s\n' "${test_results[@]}" | grep -c "WARN" || true)
    
    echo "- **Total:** $total" >> "$report_file"
    echo "- **Passed:** $passed" >> "$report_file"
    echo "- **Failed:** $failed" >> "$report_file"
    echo "- **Skipped:** $skipped" >> "$report_file"
    echo "- **Warnings:** $warnings" >> "$report_file"
    
    log_success "Test report generated: $report_file"
}

# Main test function
main() {
    echo "================================================"
    echo "Cross Chain Orbital Intents AMM - Test Suite"
    echo "================================================"
    echo ""
    
    log_info "Test configuration:"
    log_info "  Parallel: $PARALLEL"
    log_info "  Coverage: $COVERAGE"
    log_info "  Verbose: $VERBOSE"
    log_info "  Skip Integration: $SKIP_INTEGRATION"
    echo ""
    
    # Check prerequisites
    check_prerequisites
    
    # Initialize test tracking
    test_results=()
    test_names=()
    
    # Run all test suites
    local overall_success=true
    
    # Core Rust tests
    if ! run_rust_unit_tests; then
        overall_success=false
    fi
    
    if ! run_rust_doc_tests; then
        overall_success=false
    fi
    
    if ! run_rust_integration_tests; then
        overall_success=false
    fi
    
    # Orbital math tests
    if ! run_orbital_math_tests; then
        overall_success=false
    fi
    
    # Contract tests
    if ! run_contract_tests; then
        overall_success=false
    fi
    
    # Frontend tests
    if ! run_frontend_tests; then
        overall_success=false
    fi
    
    # Cross-chain tests
    if ! run_cross_chain_tests; then
        overall_success=false
    fi
    
    # Security audit
    run_security_audit || true  # Don't fail on warnings
    
    # Benchmarks (optional)
    run_benchmarks || true
    
    # Generate coverage report
    generate_coverage || true
    
    # Generate test report
    generate_test_report
    
    # Final summary
    echo ""
    echo "================================================"
    echo "Test Summary"
    echo "================================================"
    
    local total=${#test_results[@]}
    local passed=$(printf '%s\n' "${test_results[@]}" | grep -c "PASS" || true)
    local failed=$(printf '%s\n' "${test_results[@]}" | grep -c "FAIL" || true)
    
    log_info "Total test suites: $total"
    log_success "Passed: $passed"
    
    if [ "$failed" -gt 0 ]; then
        log_error "Failed: $failed"
    fi
    
    if [ "$overall_success" = true ]; then
        log_success "All critical tests passed!"
        exit 0
    else
        log_error "Some tests failed!"
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --parallel)
            PARALLEL="true"
            shift
            ;;
        --no-parallel)
            PARALLEL="false"
            shift
            ;;
        --coverage)
            COVERAGE="true"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --skip-integration)
            SKIP_INTEGRATION="true"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --parallel          Enable parallel test execution (default)"
            echo "  --no-parallel       Disable parallel test execution"
            echo "  --coverage          Generate code coverage report"
            echo "  --verbose           Enable verbose test output"
            echo "  --skip-integration  Skip integration tests"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main function
main