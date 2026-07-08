@echo off
REM Hermes Game Operator Quick Start Script for Windows

echo ==========================================
echo Hermes Game Operator - Environment Check
echo ==========================================
echo.

REM Check Node.js
echo Checking Node.js...
where node >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=*" %%i in ('node --version') do set NODE_VER=%%i
    echo [OK] Node.js: %NODE_VER%
) else (
    echo [ERROR] Node.js not found. Please install Node.js >= 18.x
    exit /b 1
)

REM Check npm
echo Checking npm...
where npm >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=*" %%i in ('npm --version') do set NPM_VER=%%i
    echo [OK] npm: %NPM_VER%
) else (
    echo [ERROR] npm not found
    exit /b 1
)

REM Check Rust
echo Checking Rust...
where rustc >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=*" %%i in ('rustc --version') do set RUST_VER=%%i
    echo [OK] Rust: %RUST_VER%
) else (
    echo [ERROR] Rust not found. Please install Rust >= 1.70
    exit /b 1
)

REM Check cargo
echo Checking cargo...
where cargo >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VER=%%i
    echo [OK] cargo: %CARGO_VER%
) else (
    echo [ERROR] cargo not found
    exit /b 1
)

REM Check link.exe (Windows SDK)
echo Checking Windows SDK (link.exe)...
where link.exe >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Windows SDK: link.exe found
) else (
    echo [WARN] Windows SDK not found. cargo check will fail.
    echo        Install via: Visual Studio Installer -^> Build Tools
)

REM Check Hermes CLI
echo Checking Hermes CLI...
where hermes >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Hermes CLI found
) else (
    echo [WARN] Hermes CLI not found. Agent execution will not work.
    echo        Download: https://github.com/hermes-ai/hermes-cli/releases
)

echo.
echo ==========================================
echo Running Tests
echo ==========================================

npm run test -- --run

echo.
echo ==========================================
echo Running Build
echo ==========================================

npm run build

echo.
echo ==========================================
echo Environment Check Complete
echo ==========================================
echo.
echo Next steps:
echo 1. If Windows SDK missing: Install VS Build Tools
echo 2. If Hermes CLI missing: Download from GitHub releases
echo 3. Run: npm run tauri dev
echo.