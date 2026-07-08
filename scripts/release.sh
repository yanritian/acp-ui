#!/bin/bash
# Hermes Game Operator Release Script
# Prepares a release version

set -e

VERSION=${1:-"0.1.0"}

echo "=========================================="
echo "Hermes Game Operator - Release v$VERSION"
echo "=========================================="
echo ""

# Update version in package.json
echo "Updating package.json version to $VERSION..."
if command -v jq &> /dev/null; then
    jq ".version = \"$VERSION\"" package.json > package.json.tmp
    mv package.json.tmp package.json
else
    echo "WARNING: jq not found, please update package.json manually"
fi

# Update version in Cargo.toml
echo "Updating Cargo.toml version to $VERSION..."
if command -v sed &> /dev/null; then
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
else
    echo "WARNING: sed not found, please update Cargo.toml manually"
fi

# Update CHANGELOG
echo "Updating CHANGELOG.md..."
cat > CHANGELOG_ENTRY.md << EOF
## [$VERSION] - $(date +%Y-%m-%d)

### Added
- Release version $VERSION

### Changed
- Updated dependencies

### Fixed
- Bug fixes and improvements

EOF

# Run tests
echo "Running tests..."
npm run test -- --run
echo ""

# Build
echo "Building..."
npm run build
echo ""

# Create git tag
echo "Creating git tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION"
echo ""

echo "=========================================="
echo "Release v$VERSION Prepared"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Review CHANGELOG.md"
echo "2. Review package.json and Cargo.toml versions"
echo "3. Commit: git commit -am 'chore: release v$VERSION'"
echo "4. Push: git push origin main --tags"
echo ""