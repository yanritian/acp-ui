# Hermes Game Operator - Development Setup Script
# Run this script to set up the development environment

Write-Host "=== Hermes Game Operator Setup ===" -ForegroundColor Cyan

# Check Node.js
Write-Host "`nChecking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "  Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: Node.js not found. Please install Node.js >= 18.x" -ForegroundColor Red
    exit 1
}

# Check npm
Write-Host "`nChecking npm..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version
    Write-Host "  npm: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: npm not found" -ForegroundColor Red
    exit 1
}

# Check Rust
Write-Host "`nChecking Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "  Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  WARNING: Rust not found. Install from https://rustup.rs" -ForegroundColor Yellow
}

# Check Windows SDK
Write-Host "`nChecking Windows SDK..." -ForegroundColor Yellow
$sdkPath = "C:\Program Files (x86)\Windows Kits\10"
if (Test-Path $sdkPath) {
    Write-Host "  Windows SDK found at: $sdkPath" -ForegroundColor Green
} else {
    Write-Host "  WARNING: Windows SDK not found. Install via Visual Studio Installer" -ForegroundColor Yellow
}

# Install dependencies
Write-Host "`nInstalling npm dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Dependencies installed successfully" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Failed to install dependencies" -ForegroundColor Red
    exit 1
}

# Verify build
Write-Host "`nVerifying build..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Build successful" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Build failed" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Setup Complete ===" -ForegroundColor Cyan
Write-Host "Run 'npm run tauri dev' to start development server" -ForegroundColor White
