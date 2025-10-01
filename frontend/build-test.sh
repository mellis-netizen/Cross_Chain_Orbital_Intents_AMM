#!/bin/bash

echo "🔧 Testing Orbital AMM build process..."

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Run this from the frontend directory"
    exit 1
fi

# Clean any previous builds
echo "🧹 Cleaning previous builds..."
rm -rf .next out

# Build the application
echo "🏗️ Building application..."
./node_modules/next/dist/bin/next build

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Check output directory
    if [ -d "out" ]; then
        echo "📁 Static files generated in 'out' directory"
        echo "📊 Build stats:"
        echo "   - Total files: $(find out -type f | wc -l)"
        echo "   - HTML files: $(find out -name "*.html" | wc -l)"
        echo "   - JS files: $(find out -name "*.js" | wc -l)"
        echo "   - CSS files: $(find out -name "*.css" | wc -l)"
        echo "   - Total size: $(du -sh out | cut -f1)"
        
        echo ""
        echo "🎉 Orbital AMM is ready for deployment!"
        echo "📤 You can now deploy the 'out' directory to Netlify"
    else
        echo "❌ Output directory not found"
        exit 1
    fi
else
    echo "❌ Build failed"
    exit 1
fi