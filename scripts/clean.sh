#!/bin/bash
# Hermes Game Operator - Clean All Generated Files
# Removes all build artifacts and generated files

set -e

echo "=========================================="
echo "Hermes Game Operator - Clean All"
echo "=========================================="
echo ""

echo "Cleaning npm cache..."
rm -rf node_modules/.cache/
rm -rf node_modules/.vite/

echo "Cleaning dist directories..."
rm -rf dist/
rm -rf dist-web/

echo "Cleaning Rust build artifacts..."
rm -rf src-tauri/target/debug/
rm -rf src-tauri/target/release/
rm -rf src-tauri/target/wasm32-unknown-unknown/

echo "Cleaning test cache..."
rm -rf .vitest-cache/
rm -rf test-results/
rm -rf coverage/

echo "Cleaning temporary files..."
rm -rf tmp/
rm -f *.log
rm -f .env.local

echo ""
echo "Clean complete!"
echo ""
echo "To reinstall dependencies:"
echo "  npm install"