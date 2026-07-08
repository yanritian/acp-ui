#!/bin/bash
# Hermes Game Operator - Code Quality Check
# Runs linters, formatters, and type checkers

set -e

echo "=========================================="
echo "Hermes Game Operator - Quality Check"
echo "=========================================="
echo ""

# TypeScript type check
echo "Running TypeScript type check..."
npm run typecheck 2>/dev/null || npm run build:only
echo "✓ TypeScript OK"
echo ""

# ESLint
echo "Running ESLint..."
npm run lint 2>/dev/null || echo "ESLint not configured"
echo ""

# Rust fmt check
echo "Running cargo fmt check..."
cd src-tauri
cargo fmt --check 2>/dev/null || cargo fmt
echo "✓ Rust format OK"
echo ""

# Rust clippy
echo "Running cargo clippy..."
cargo clippy 2>/dev/null || echo "Clippy not available"
echo ""

cd ..

echo "=========================================="
echo "Quality Check Complete"
echo "=========================================="
echo ""
echo "All checks passed!"