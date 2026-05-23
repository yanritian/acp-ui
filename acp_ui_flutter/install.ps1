# Set Environment Variables
$env:PATH = "D:\flutter_sdk\bin;D:\flutter_sdk\bin\cache\dart-sdk\bin;C:\Program Files\Git\cmd;C:\Program Files\Git\mingw64\bin;D:\Android\Sdk\platform-tools;C:\Windows\System32;C:\Windows;" + $env:PATH
$env:ANDROID_SDK_ROOT = "D:\Android\Sdk"
$env:ANDROID_HOME = "D:\Android\Sdk"
$env:GIT_EXECUTABLE = "C:\Program Files\Git\cmd\git.exe"

# Change Directory
Set-Location -Path "D:\dingsun\acp-ui\acp_ui_flutter"

Write-Host "========================================" -ForegroundColor Green
Write-Host " Starting Flutter Build and Install " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Clean
Write-Host "[1/3] Cleaning project..." -ForegroundColor Yellow
& flutter clean
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Clean failed. Check error above." -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit
}

# Build
Write-Host ""
Write-Host "[2/3] Building APK (This may take a few minutes)..." -ForegroundColor Yellow
& flutter build apk --debug
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed. Check error above." -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit
}

# Install
Write-Host ""
Write-Host "[3/3] Installing to device..." -ForegroundColor Yellow
$apkPath = "build\app\outputs\flutter-apk\app-debug.apk"
if (Test-Path $apkPath) {
    & adb install -r $apkPath
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Install failed." -ForegroundColor Red
    } else {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host " SUCCESS! App installed. " -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
    }
} else {
    Write-Host "ERROR: APK file not found." -ForegroundColor Red
}

Write-Host ""
Read-Host "Press Enter to close"