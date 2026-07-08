#!/bin/bash
# Run all tests for Hermes Game Operator

echo "=== Running Tests ==="

echo ""
echo "1. Unit Tests..."
npm run test -- --run

echo ""
echo "2. Build Test..."
npm run build

echo ""
echo "3. E2E Tests (requires dev server)..."
echo "   Run 'npm run dev' first, then 'npm run test:e2e'"

echo ""
echo "=== Tests Complete ==="
