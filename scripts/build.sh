#!/bin/bash
# Hermes Game Operator Build Script
# Builds the application for production deployment

set -e

echo "=========================================="
echo "Hermes Game Operator - Production Build"
echo "=========================================="
echo ""

# Check environment
echo "Checking environment..."

if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js not found"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found"
    exit 1
fi

echo "✓ Node.js: $(node --version)"
echo "✓ npm: $(npm --version)"
echo "✓ cargo: $(cargo --version)"
echo ""

# Install dependencies
echo "Installing dependencies..."
npm install
echo ""

# Run tests
echo "Running tests..."
npm run test -- --run
echo ""

# Build frontend
echo "Building frontend..."
npm run build
echo ""

# Check if Tauri CLI is available
if command -v cargo tauri &> /dev/null; then
    echo "Building Tauri application..."
    cargo tauri build
    echo ""
    echo "Build complete!"
    echo "Output: src-tauri/target/release/"
else
    echo "WARNING: Tauri CLI not found"
    echo "To build desktop app, run: cargo tauri build"
    echo ""
    echo "Frontend build complete!"
    echo "Output: dist/"
fi

echo ""
echo "=========================================="
echo "Build Summary"
echo "=========================================="
echo "Frontend: dist/"
echo "Tests: Passed"
echo ""