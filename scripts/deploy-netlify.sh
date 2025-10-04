#!/bin/bash

# Orbital AMM Netlify Deployment Script
# This script builds and deploys the complete N-dimensional Orbital AMM to Netlify

set -e

echo "🌌 Deploying Orbital AMM to Netlify..."

# Check if we're in the correct directory
if [ ! -f "frontend/package.json" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if Netlify CLI is installed
if ! command -v netlify &> /dev/null; then
    echo "📦 Installing Netlify CLI..."
    npm install -g netlify-cli
fi

# Navigate to frontend directory
cd frontend

echo "🔧 Installing dependencies..."
npm install

echo "🏗️ Building the application..."
npm run build

echo "🚀 Deploying to Netlify..."

# Check if site is already linked
if [ ! -f ".netlify/state.json" ]; then
    echo "🔗 Creating new Netlify site..."
    netlify deploy --prod --dir=out --open
else
    echo "📤 Deploying to existing site..."
    netlify deploy --prod --dir=out
fi

echo ""
echo "🎉 Orbital AMM deployed successfully!"
echo ""
echo "🌟 Features now live:"
echo "   ✅ N-dimensional pool creation (3-1000 tokens)"
echo "   ✅ Toroidal trading with spherical constraints"
echo "   ✅ Concentrated liquidity management"
echo "   ✅ 10-token pool demonstration"
echo "   ✅ Real-time analytics dashboard"
echo "   ✅ MEV protection interface"
echo "   ✅ Mobile-responsive design"
echo ""
echo "🔗 Your Orbital AMM is now accessible worldwide!"
echo "📱 Share the link and experience the future of DeFi"

# Go back to project root
cd ..