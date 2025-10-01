#!/bin/bash

# Orbital AMM Deployment Script for Holesky Testnet
# This script deploys the production-grade N-dimensional Orbital AMM

set -e

echo "🚀 Deploying Orbital AMM to Holesky Testnet..."

# Check if Rust and Cargo are installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo not found. Please install Rust first."
    exit 1
fi

# Check if stylus CLI is installed
if ! command -v cargo-stylus &> /dev/null; then
    echo "📦 Installing Arbitrum Stylus CLI..."
    cargo install cargo-stylus
fi

# Set environment variables for Holesky
export RPC_URL="https://ethereum-holesky-rpc.publicnode.com"
export PRIVATE_KEY=${HOLESKY_PRIVATE_KEY:-""}

if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ Error: Please set HOLESKY_PRIVATE_KEY environment variable"
    echo "   Example: export HOLESKY_PRIVATE_KEY=0x..."
    exit 1
fi

echo "🔧 Building Orbital AMM contract..."

# Build the contract
cargo build --release --target wasm32-unknown-unknown

echo "✅ Contract built successfully!"

echo "🌐 Deploying to Holesky testnet..."

# Deploy using stylus
cargo stylus deploy \
    --private-key=$PRIVATE_KEY \
    --endpoint=$RPC_URL \
    --estimate-gas

echo "🎉 Orbital AMM deployed successfully to Holesky!"

# Export ABI for frontend integration
echo "📄 Exporting contract ABI..."
cargo stylus export-abi --json > orbital_amm_abi.json

echo "✅ ABI exported to orbital_amm_abi.json"

echo ""
echo "🔗 Contract deployed! Next steps:"
echo "1. Note the contract address from the deployment output"
echo "2. Fund the contract with test ETH for gas"
echo "3. Use the ABI file for frontend integration"
echo "4. Create test pools with the create_orbital_pool function"
echo ""
echo "🌌 The Orbital AMM is now live on Holesky testnet!"