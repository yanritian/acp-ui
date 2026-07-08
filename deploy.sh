#!/bin/bash

# Hermes Game Operator Deployment Script
# Usage: ./deploy.sh [environment]
# Environments: staging, production

set -e

ENVIRONMENT=${1:-staging}
VERSION=$(grep '"version"' package.json | cut -d'"' -f4)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "🚀 Deploying Hermes Game Operator v${VERSION}"
echo "Environment: ${ENVIRONMENT}"
echo "Timestamp: ${TIMESTAMP}"

# ============================================================================
# Pre-deployment Checks
# ============================================================================

echo "📋 Running pre-deployment checks..."

# Check Node.js version
NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js 18+ required, found $(node --version)"
    exit 1
fi
echo "✅ Node.js version: $(node --version)"

# Check Rust toolchain
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust toolchain not found"
    exit 1
fi
echo "✅ Rust version: $(rustc --version)"

# Check Git
if ! command -v git &> /dev/null; then
    echo "❌ Git not found"
    exit 1
fi
echo "✅ Git version: $(git --version)"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: Uncommitted changes detected"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# ============================================================================
# Build
# ============================================================================

echo "🔨 Building application..."

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Run tests
echo "🧪 Running tests..."
npm run test

# Build frontend
echo "🎨 Building frontend..."
npm run build

# Build Tauri application
echo "🦀 Building Tauri application..."
npm run tauri build

# ============================================================================
# Package
# ============================================================================

echo "📦 Packaging application..."

# Create release directory
RELEASE_DIR="releases/v${VERSION}_${TIMESTAMP}"
mkdir -p "$RELEASE_DIR"

# Copy build artifacts
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    # Windows
    cp src-tauri/target/release/bundle/msi/*.msi "$RELEASE_DIR/" 2>/dev/null || true
    cp src-tauri/target/release/bundle/nsis/*.exe "$RELEASE_DIR/" 2>/dev/null || true
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    cp src-tauri/target/release/bundle/dmg/*.dmg "$RELEASE_DIR/" 2>/dev/null || true
else
    # Linux
    cp src-tauri/target/release/bundle/deb/*.deb "$RELEASE_DIR/" 2>/dev/null || true
    cp src-tauri/target/release/bundle/rpm/*.rpm "$RELEASE_DIR/" 2>/dev/null || true
    cp src-tauri/target/release/bundle/appimage/*.AppImage "$RELEASE_DIR/" 2>/dev/null || true
fi

# Copy web build
cp -r dist-web "$RELEASE_DIR/web"

# Generate checksums
echo "🔐 Generating checksums..."
cd "$RELEASE_DIR"
find . -type f -exec sha256sum {} \; > SHA256SUMS.txt
cd -

# ============================================================================
# Deploy
# ============================================================================

echo "🌐 Deploying to ${ENVIRONMENT}..."

if [ "$ENVIRONMENT" = "production" ]; then
    # Production deployment
    echo "🚀 Deploying to production..."

    # Upload to GitHub Releases
    if command -v gh &> /dev/null; then
        echo "📤 Uploading to GitHub Releases..."
        gh release create "v${VERSION}" "$RELEASE_DIR"/* \
            --title "v${VERSION}" \
            --notes "Release notes for v${VERSION}" \
            --target main
    else
        echo "⚠️  GitHub CLI not found, skipping release upload"
        echo "Please manually upload files from $RELEASE_DIR"
    fi

    # Deploy web version
    echo "🌍 Deploying web version..."
    if command -v npm &> /dev/null; then
        npm run deploy:web 2>/dev/null || echo "⚠️  Web deployment script not configured"
    fi

elif [ "$ENVIRONMENT" = "staging" ]; then
    # Staging deployment
    echo "🔧 Deploying to staging..."

    # Upload to staging server
    if [ -n "$STAGING_SERVER" ]; then
        echo "📤 Uploading to staging server..."
        rsync -avz "$RELEASE_DIR/" "$STAGING_SERVER:/var/www/hermes-staging/"
    else
        echo "⚠️  STAGING_SERVER not set"
        echo "Files prepared in: $RELEASE_DIR"
    fi
fi

# ============================================================================
# Post-deployment
# ============================================================================

echo "✅ Deployment complete!"
echo ""
echo "📦 Release artifacts:"
ls -lh "$RELEASE_DIR"
echo ""
echo "📊 Deployment summary:"
echo "  Version: ${VERSION}"
echo "  Environment: ${ENVIRONMENT}"
echo "  Timestamp: ${TIMESTAMP}"
echo "  Artifacts: $RELEASE_DIR"
echo ""

# ============================================================================
# Notifications
# ============================================================================

# Send notification
if [ -n "$SLACK_WEBHOOK" ]; then
    echo "📢 Sending Slack notification..."
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"🚀 Hermes Game Operator v${VERSION} deployed to ${ENVIRONMENT}\"}" \
        "$SLACK_WEBHOOK"
fi

# Update status page
if [ -n "$STATUS_PAGE_API" ]; then
    echo "📊 Updating status page..."
    curl -X POST "$STATUS_PAGE_API/incidents" \
        -H "Authorization: Bearer $STATUS_PAGE_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"title\":\"Release v${VERSION}\",\"status\":\"completed\"}"
fi

echo ""
echo "🎉 Deployment successful!"
echo ""
echo "Next steps:"
echo "1. Test the deployment in ${ENVIRONMENT}"
echo "2. Monitor error rates and performance"
echo "3. Collect user feedback"
echo "4. Update documentation if needed"
echo ""
