#!/bin/bash
# Hermes Game Operator Quick Start Script
# Run this script to verify your development environment

set -e

echo "=========================================="
echo "Hermes Game Operator - Environment Check"
echo "=========================================="
echo ""

# Check Node.js
echo "Checking Node.js..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✓ Node.js: $NODE_VERSION"
else
    echo "✗ Node.js not found. Please install Node.js >= 18.x"
    exit 1
fi

# Check npm
echo "Checking npm..."
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    echo "✓ npm: $NPM_VERSION"
else
    echo "✗ npm not found"
    exit 1
fi

# Check Rust
echo "Checking Rust..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✓ Rust: $RUST_VERSION"
else
    echo "✗ Rust not found. Please install Rust >= 1.70"
    exit 1
fi

# Check cargo
echo "Checking cargo..."
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo "✓ cargo: $CARGO_VERSION"
else
    echo "✗ cargo not found"
    exit 1
fi

# Check link.exe (Windows SDK)
echo "Checking Windows SDK (link.exe)..."
if command -v link.exe &> /dev/null; then
    echo "✓ Windows SDK: link.exe found"
else
    echo "⚠ Windows SDK not found. cargo check will fail."
    echo "  Install via: Visual Studio Installer -> Build Tools"
fi

# Check Hermes CLI
echo "Checking Hermes CLI..."
if command -v hermes &> /dev/null; then
    HERMES_VERSION=$(hermes --version 2>/dev/null || echo "unknown")
    echo "✓ Hermes CLI: $HERMES_VERSION"
else
    echo "⚠ Hermes CLI not found. Agent execution will not work."
    echo "  Download: https://github.com/hermes-ai/hermes-cli/releases"
fi

echo ""
echo "=========================================="
echo "Running Tests"
echo "=========================================="

# Run npm tests
echo "Running npm test..."
npm run test -- --run 2>&1 | tail -5

echo ""
echo "=========================================="
echo "Running Build"
echo "=========================================="

# Run build
echo "Running npm build..."
npm run build 2>&1 | tail -5

echo ""
echo "=========================================="
echo "Environment Check Complete"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. If Windows SDK missing: Install VS Build Tools"
echo "2. If Hermes CLI missing: Download from GitHub releases"
echo "3. Run: npm run tauri dev"
echo ""