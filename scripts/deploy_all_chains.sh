#!/bin/bash

# Cross Chain Orbital Intents AMM - Multi-Chain Deployment Script
# This script deploys the Orbital AMM contracts to all supported chains

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
CONTRACTS_DIR="$PROJECT_ROOT/contracts/orbital-amm"

# Default values
ENVIRONMENT="${ENVIRONMENT:-testnet}"
DRY_RUN="${DRY_RUN:-false}"
VERIFY_CONTRACTS="${VERIFY_CONTRACTS:-true}"

# Chain configurations
declare -A CHAINS
declare -A RPC_URLS
declare -A EXPLORERS
declare -A API_KEYS

# Initialize chain configurations
init_chain_config() {
    if [ "$ENVIRONMENT" = "mainnet" ]; then
        CHAINS[ethereum]="1"
        CHAINS[arbitrum]="42161"
        CHAINS[optimism]="10"
        CHAINS[base]="8453"
        
        RPC_URLS[ethereum]="$ETH_RPC_URL"
        RPC_URLS[arbitrum]="$ARB_RPC_URL"
        RPC_URLS[optimism]="$OP_RPC_URL"
        RPC_URLS[base]="$BASE_RPC_URL"
        
        EXPLORERS[ethereum]="etherscan"
        EXPLORERS[arbitrum]="arbiscan"
        EXPLORERS[optimism]="optimism"
        EXPLORERS[base]="basescan"
        
        API_KEYS[ethereum]="$ETHERSCAN_API_KEY"
        API_KEYS[arbitrum]="$ARBISCAN_API_KEY"
        API_KEYS[optimism]="$OPSCAN_API_KEY"
        API_KEYS[base]="$BASESCAN_API_KEY"
    else
        # Testnet configuration
        CHAINS[holesky]="17000"
        CHAINS[arbitrum_sepolia]="421614"
        CHAINS[optimism_sepolia]="11155420"
        CHAINS[base_sepolia]="84532"
        
        RPC_URLS[holesky]="$HOLESKY_RPC_URL"
        RPC_URLS[arbitrum_sepolia]="$ARB_SEPOLIA_RPC_URL"
        RPC_URLS[optimism_sepolia]="$OP_SEPOLIA_RPC_URL"
        RPC_URLS[base_sepolia]="$BASE_SEPOLIA_RPC_URL"
        
        EXPLORERS[holesky]="etherscan"
        EXPLORERS[arbitrum_sepolia]="arbiscan"
        EXPLORERS[optimism_sepolia]="optimism"
        EXPLORERS[base_sepolia]="basescan"
        
        API_KEYS[holesky]="$ETHERSCAN_API_KEY"
        API_KEYS[arbitrum_sepolia]="$ARBISCAN_API_KEY"
        API_KEYS[optimism_sepolia]="$OPSCAN_API_KEY"
        API_KEYS[base_sepolia]="$BASESCAN_API_KEY"
    fi
}

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if forge is installed
    if ! command -v forge &> /dev/null; then
        log_error "Foundry forge not found. Please install Foundry first."
        exit 1
    fi
    
    # Check if environment variables are set
    if [ -z "$DEPLOYER_PRIVATE_KEY" ]; then
        log_error "DEPLOYER_PRIVATE_KEY not set"
        exit 1
    fi
    
    # Check if contracts directory exists
    if [ ! -d "$CONTRACTS_DIR" ]; then
        log_error "Contracts directory not found: $CONTRACTS_DIR"
        exit 1
    fi
    
    # Validate private key format
    if [[ ! "$DEPLOYER_PRIVATE_KEY" =~ ^0x[0-9a-fA-F]{64}$ ]]; then
        log_error "Invalid private key format"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Deploy to a single chain
deploy_to_chain() {
    local chain_name="$1"
    local chain_id="${CHAINS[$chain_name]}"
    local rpc_url="${RPC_URLS[$chain_name]}"
    local explorer="${EXPLORERS[$chain_name]}"
    local api_key="${API_KEYS[$chain_name]}"
    
    log_info "Deploying to $chain_name (Chain ID: $chain_id)..."
    
    # Check if RPC URL is set
    if [ -z "$rpc_url" ]; then
        log_error "RPC URL not set for $chain_name"
        return 1
    fi
    
    # Build deployment command
    local deploy_cmd="forge script script/Deploy.s.sol:Deploy"
    deploy_cmd="$deploy_cmd --rpc-url $rpc_url"
    deploy_cmd="$deploy_cmd --private-key $DEPLOYER_PRIVATE_KEY"
    
    if [ "$DRY_RUN" = "false" ]; then
        deploy_cmd="$deploy_cmd --broadcast"
    fi
    
    # Add verification if enabled and API key is available
    if [ "$VERIFY_CONTRACTS" = "true" ] && [ -n "$api_key" ] && [ "$DRY_RUN" = "false" ]; then
        deploy_cmd="$deploy_cmd --verify --etherscan-api-key $api_key"
    fi
    
    # Execute deployment
    cd "$CONTRACTS_DIR"
    
    if eval "$deploy_cmd"; then
        log_success "Deployment to $chain_name completed successfully"
        
        # Save deployment info
        local deployment_file="deployments/${ENVIRONMENT}_${chain_name}.json"
        mkdir -p "deployments"
        
        # Extract contract addresses from broadcast logs
        if [ -f "broadcast/Deploy.s.sol/$chain_id/run-latest.json" ]; then
            cp "broadcast/Deploy.s.sol/$chain_id/run-latest.json" "$deployment_file"
            log_info "Deployment info saved to $deployment_file"
        fi
        
        return 0
    else
        log_error "Deployment to $chain_name failed"
        return 1
    fi
}

# Verify deployment
verify_deployment() {
    local chain_name="$1"
    local deployment_file="deployments/${ENVIRONMENT}_${chain_name}.json"
    
    if [ ! -f "$deployment_file" ]; then
        log_warning "Deployment file not found for $chain_name"
        return 1
    fi
    
    log_info "Verifying deployment on $chain_name..."
    
    # Extract contract addresses and verify they're deployed
    local orbital_amm_address=$(jq -r '.transactions[] | select(.contractName == "OrbitalAMM") | .contractAddress' "$deployment_file" 2>/dev/null)
    local intent_engine_address=$(jq -r '.transactions[] | select(.contractName == "IntentEngine") | .contractAddress' "$deployment_file" 2>/dev/null)
    
    if [ "$orbital_amm_address" != "null" ] && [ "$orbital_amm_address" != "" ]; then
        log_success "OrbitalAMM deployed at: $orbital_amm_address"
    else
        log_error "OrbitalAMM address not found in deployment file"
        return 1
    fi
    
    if [ "$intent_engine_address" != "null" ] && [ "$intent_engine_address" != "" ]; then
        log_success "IntentEngine deployed at: $intent_engine_address"
    else
        log_error "IntentEngine address not found in deployment file"
        return 1
    fi
    
    return 0
}

# Generate deployment summary
generate_summary() {
    log_info "Generating deployment summary..."
    
    local summary_file="deployments/${ENVIRONMENT}_summary.json"
    local summary="{"
    summary="$summary\"environment\": \"$ENVIRONMENT\","
    summary="$summary\"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    summary="$summary\"chains\": {"
    
    local first=true
    for chain in "${!CHAINS[@]}"; do
        local deployment_file="deployments/${ENVIRONMENT}_${chain}.json"
        
        if [ -f "$deployment_file" ]; then
            if [ "$first" = false ]; then
                summary="$summary,"
            fi
            
            summary="$summary\"$chain\": {"
            summary="$summary\"chainId\": \"${CHAINS[$chain]}\","
            
            # Extract contract addresses
            local orbital_amm_address=$(jq -r '.transactions[] | select(.contractName == "OrbitalAMM") | .contractAddress' "$deployment_file" 2>/dev/null || echo "null")
            local intent_engine_address=$(jq -r '.transactions[] | select(.contractName == "IntentEngine") | .contractAddress' "$deployment_file" 2>/dev/null || echo "null")
            
            summary="$summary\"contracts\": {"
            summary="$summary\"OrbitalAMM\": \"$orbital_amm_address\","
            summary="$summary\"IntentEngine\": \"$intent_engine_address\""
            summary="$summary}"
            summary="$summary}"
            
            first=false
        fi
    done
    
    summary="$summary}"
    summary="$summary}"
    
    echo "$summary" | jq '.' > "$summary_file"
    log_success "Deployment summary saved to $summary_file"
}

# Update environment files with contract addresses
update_env_files() {
    log_info "Updating environment files with contract addresses..."
    
    local env_file="$PROJECT_ROOT/.env.$ENVIRONMENT"
    
    if [ ! -f "$env_file" ]; then
        log_warning "Environment file not found: $env_file"
        return
    fi
    
    # Backup original file
    cp "$env_file" "$env_file.backup"
    
    for chain in "${!CHAINS[@]}"; do
        local deployment_file="deployments/${ENVIRONMENT}_${chain}.json"
        
        if [ -f "$deployment_file" ]; then
            local orbital_amm_address=$(jq -r '.transactions[] | select(.contractName == "OrbitalAMM") | .contractAddress' "$deployment_file" 2>/dev/null)
            local intent_engine_address=$(jq -r '.transactions[] | select(.contractName == "IntentEngine") | .contractAddress' "$deployment_file" 2>/dev/null)
            
            # Update environment file
            local chain_upper=$(echo "$chain" | tr '[:lower:]' '[:upper:]')
            
            if [ "$orbital_amm_address" != "null" ] && [ "$orbital_amm_address" != "" ]; then
                sed -i.bak "s/^${chain_upper}_ORBITAL_AMM_ADDRESS=.*/${chain_upper}_ORBITAL_AMM_ADDRESS=$orbital_amm_address/" "$env_file"
            fi
            
            if [ "$intent_engine_address" != "null" ] && [ "$intent_engine_address" != "" ]; then
                sed -i.bak "s/^${chain_upper}_INTENT_ENGINE_ADDRESS=.*/${chain_upper}_INTENT_ENGINE_ADDRESS=$intent_engine_address/" "$env_file"
            fi
        fi
    done
    
    # Remove backup files
    rm -f "$env_file.bak"
    rm -f "$env_file.backup"
    
    log_success "Environment file updated: $env_file"
}

# Main deployment function
main() {
    echo "=============================================="
    echo "Cross Chain Orbital Intents AMM Deployment"
    echo "=============================================="
    echo ""
    
    log_info "Environment: $ENVIRONMENT"
    log_info "Dry run: $DRY_RUN"
    log_info "Verify contracts: $VERIFY_CONTRACTS"
    echo ""
    
    # Initialize configuration
    init_chain_config
    
    # Check prerequisites
    check_prerequisites
    
    # Create deployments directory
    mkdir -p "$CONTRACTS_DIR/deployments"
    
    # Deploy to all chains
    local failed_chains=()
    local successful_chains=()
    
    for chain in "${!CHAINS[@]}"; do
        if deploy_to_chain "$chain"; then
            successful_chains+=("$chain")
            
            # Verify deployment
            if verify_deployment "$chain"; then
                log_success "Deployment verification passed for $chain"
            else
                log_warning "Deployment verification failed for $chain"
            fi
        else
            failed_chains+=("$chain")
        fi
        echo ""
    done
    
    # Generate summary
    if [ ${#successful_chains[@]} -gt 0 ]; then
        generate_summary
        
        if [ "$DRY_RUN" = "false" ]; then
            update_env_files
        fi
    fi
    
    # Final report
    echo "=============================================="
    echo "Deployment Summary"
    echo "=============================================="
    
    if [ ${#successful_chains[@]} -gt 0 ]; then
        log_success "Successfully deployed to: ${successful_chains[*]}"
    fi
    
    if [ ${#failed_chains[@]} -gt 0 ]; then
        log_error "Failed deployments: ${failed_chains[*]}"
        exit 1
    else
        log_success "All deployments completed successfully!"
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --no-verify)
            VERIFY_CONTRACTS="false"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --environment ENV    Set deployment environment (testnet|mainnet)"
            echo "  --dry-run           Perform dry run without broadcasting"
            echo "  --no-verify         Skip contract verification"
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