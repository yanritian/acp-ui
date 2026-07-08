# Hermes Game Operator Windows Deployment Script
# Usage: .\deploy.ps1 [-Environment staging|production]

param(
    [ValidateSet("staging", "production")]
    [string]$Environment = "staging"
)

$ErrorActionPreference = "Stop"

$Version = (Get-Content package.json | ConvertFrom-Json).version
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"

Write-Host "🚀 Deploying Hermes Game Operator v$Version" -ForegroundColor Green
Write-Host "Environment: $Environment" -ForegroundColor Cyan
Write-Host "Timestamp: $Timestamp" -ForegroundColor Cyan

# ============================================================================
# Pre-deployment Checks
# ============================================================================

Write-Host "`n📋 Running pre-deployment checks..." -ForegroundColor Yellow

# Check Node.js version
try {
    $NodeVersion = node --version
    $NodeMajor = [int]($NodeVersion -replace 'v(\d+)\..*', '$1')
    if ($NodeMajor -lt 18) {
        Write-Host "❌ Node.js 18+ required, found $NodeVersion" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Node.js version: $NodeVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Node.js not found" -ForegroundColor Red
    exit 1
}

# Check Rust toolchain
try {
    $RustVersion = rustc --version
    Write-Host "✅ Rust version: $RustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust toolchain not found" -ForegroundColor Red
    exit 1
}

# Check Git
try {
    $GitVersion = git --version
    Write-Host "✅ Git version: $GitVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Git not found" -ForegroundColor Red
    exit 1
}

# Check for uncommitted changes
$GitStatus = git status --porcelain
if ($GitStatus) {
    Write-Host "⚠️  Warning: Uncommitted changes detected" -ForegroundColor Yellow
    $Response = Read-Host "Continue anyway? (y/n)"
    if ($Response -ne 'y') {
        exit 1
    }
}

# ============================================================================
# Build
# ============================================================================

Write-Host "`n🔨 Building application..." -ForegroundColor Yellow

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Cyan
npm install

# Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Cyan
npm run test

# Build frontend
Write-Host "🎨 Building frontend..." -ForegroundColor Cyan
npm run build

# Build Tauri application
Write-Host "🦀 Building Tauri application..." -ForegroundColor Cyan
npm run tauri build

# ============================================================================
# Package
# ============================================================================

Write-Host "`n📦 Packaging application..." -ForegroundColor Yellow

# Create release directory
$ReleaseDir = "releases\v$Version`_$Timestamp"
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

# Copy build artifacts
$MsiPath = "src-tauri\target\release\bundle\msi"
$NsisPath = "src-tauri\target\release\bundle\nsis"

if (Test-Path $MsiPath) {
    Get-ChildItem -Path $MsiPath -Filter "*.msi" | Copy-Item -Destination $ReleaseDir
}

if (Test-Path $NsisPath) {
    Get-ChildItem -Path $NsisPath -Filter "*.exe" | Copy-Item -Destination $ReleaseDir
}

# Copy web build
Copy-Item -Path "dist-web" -Destination "$ReleaseDir\web" -Recurse

# Generate checksums
Write-Host "🔐 Generating checksums..." -ForegroundColor Cyan
Get-ChildItem -Path $ReleaseDir -Recurse -File | ForEach-Object {
    $Hash = Get-FileHash -Path $_.FullName -Algorithm SHA256
    "$($Hash.Hash)  $($_.FullName)"
} | Out-File -FilePath "$ReleaseDir\SHA256SUMS.txt"

# ============================================================================
# Deploy
# ============================================================================

Write-Host "`n🌐 Deploying to $Environment..." -ForegroundColor Yellow

if ($Environment -eq "production") {
    Write-Host "🚀 Deploying to production..." -ForegroundColor Cyan

    # Upload to GitHub Releases
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        Write-Host "📤 Uploading to GitHub Releases..." -ForegroundColor Cyan
        $Files = Get-ChildItem -Path $ReleaseDir -File | Select-Object -ExpandProperty FullName
        gh release create "v$Version" $Files `
            --title "v$Version" `
            --notes "Release notes for v$Version" `
            --target main
    } else {
        Write-Host "⚠️  GitHub CLI not found, skipping release upload" -ForegroundColor Yellow
        Write-Host "Please manually upload files from $ReleaseDir" -ForegroundColor Yellow
    }

    # Deploy web version
    Write-Host "🌍 Deploying web version..." -ForegroundColor Cyan
    try {
        npm run deploy:web 2>&1 | Out-Null
        Write-Host "✅ Web version deployed" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  Web deployment script not configured" -ForegroundColor Yellow
    }

} elseif ($Environment -eq "staging") {
    Write-Host "🔧 Deploying to staging..." -ForegroundColor Cyan

    if ($env:STAGING_SERVER) {
        Write-Host "📤 Uploading to staging server..." -ForegroundColor Cyan
        # Use robocopy for Windows
        robocopy $ReleaseDir "\\$env:STAGING_SERVER\var\www\hermes-staging" /MIR
    } else {
        Write-Host "⚠️  STAGING_SERVER environment variable not set" -ForegroundColor Yellow
        Write-Host "Files prepared in: $ReleaseDir" -ForegroundColor Yellow
    }
}

# ============================================================================
# Post-deployment
# ============================================================================

Write-Host "`n✅ Deployment complete!" -ForegroundColor Green
Write-Host "`n📦 Release artifacts:" -ForegroundColor Cyan
Get-ChildItem -Path $ReleaseDir | Format-Table Name, Length -AutoSize

Write-Host "`n📊 Deployment summary:" -ForegroundColor Cyan
Write-Host "  Version: $Version"
Write-Host "  Environment: $Environment"
Write-Host "  Timestamp: $Timestamp"
Write-Host "  Artifacts: $ReleaseDir"

# ============================================================================
# Notifications
# ============================================================================

# Send notification
if ($env:SLACK_WEBHOOK) {
    Write-Host "`n📢 Sending Slack notification..." -ForegroundColor Cyan
    $Body = @{
        text = "🚀 Hermes Game Operator v$Version deployed to $Environment"
    } | ConvertTo-Json

    Invoke-RestMethod -Uri $env:SLACK_WEBHOOK -Method Post -Body $Body -ContentType 'application/json'
}

# Update status page
if ($env:STATUS_PAGE_API) {
    Write-Host "📊 Updating status page..." -ForegroundColor Cyan
    $Body = @{
        title = "Release v$Version"
        status = "completed"
    } | ConvertTo-Json

    $Headers = @{
        Authorization = "Bearer $env:STATUS_PAGE_TOKEN"
    }

    Invoke-RestMethod -Uri "$env:STATUS_PAGE_API/incidents" -Method Post -Body $Body -Headers $Headers -ContentType 'application/json'
}

Write-Host "`n🎉 Deployment successful!" -ForegroundColor Green
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Test the deployment in $Environment"
Write-Host "2. Monitor error rates and performance"
Write-Host "3. Collect user feedback"
Write-Host "4. Update documentation if needed"
Write-Host ""
