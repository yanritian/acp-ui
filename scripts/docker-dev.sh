#!/bin/bash
# Hermes Game Operator - Docker Development Environment
# Starts a development container for testing

set -e

echo "=========================================="
echo "Hermes Game Operator - Docker Dev"
echo "=========================================="
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "ERROR: Docker not found"
    echo "Please install Docker Desktop"
    exit 1
fi

echo "Building development container..."
docker build -t hermes-game-operator:dev -f Dockerfile.dev .

echo ""
echo "Starting development container..."
docker run -it --rm \
    -p 1420:1420 \
    -v "$(pwd)/src:/app/src" \
    -v "$(pwd)/src-tauri:/app/src-tauri" \
    -v "$(pwd)/package.json:/app/package.json" \
    hermes-game-operator:dev \
    /bin/bash -c "npm install && npm run dev"

echo ""
echo "Container stopped."