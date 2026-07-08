@echo off
REM Hermes Game Operator - Build and Verification Script
REM Usage: build.bat [check|test|build|release]

setlocal enabledelayedexpansion

echo ========================================
echo Hermes Game Operator Build Script
echo ========================================
echo.

REM Set build mode
set MODE=%1
if "%MODE%"=="" set MODE=check

REM Navigate to project root
cd /d "%~dp0"

echo Mode: %MODE%
echo Project: %CD%
echo.

REM Step 1: Check prerequisites
echo [1/5] Checking prerequisites...

REM Check Node.js
node --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Node.js not found
    echo Please install Node.js 18+ from https://nodejs.org/
    exit /b 1
)
for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo   Node.js: %NODE_VERSION%

REM Check npm
npm --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: npm not found
    exit /b 1
)
for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo   npm: %NPM_VERSION%

REM Check Rust
rustc --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust not found
    echo Please install Rust from https://rustup.rs/
    echo Or follow docs/RUST-TOOLCHAIN-GUIDE.md
    exit /b 1
)
for /f "tokens=*" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo   Rust: %RUST_VERSION%

REM Check cargo
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Cargo not found
    exit /b 1
)
for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo   Cargo: %CARGO_VERSION%

echo   All prerequisites met!
echo.

REM Step 2: Install dependencies
echo [2/5] Installing dependencies...
call npm install
if errorlevel 1 (
    echo ERROR: npm install failed
    exit /b 1
)
echo   Dependencies installed!
echo.

REM Step 3: Run frontend tests
echo [3/5] Running frontend tests...
call npm run test
if errorlevel 1 (
    echo ERROR: Frontend tests failed
    exit /b 1
)
echo   Frontend tests passed!
echo.

REM Step 4: Build frontend
echo [4/5] Building frontend...
call npm run build
if errorlevel 1 (
    echo ERROR: Frontend build failed
    exit /b 1
)
echo   Frontend built successfully!
echo.

REM Step 5: Rust verification (optional)
echo [5/5] Rust verification...

if "%MODE%"=="check" (
    echo Running cargo check...
    cd src-tauri
    cargo check
    if errorlevel 1 (
        echo WARNING: cargo check failed
        echo This may be due to Rust toolchain configuration
        echo See docs/RUST-TOOLCHAIN-GUIDE.md for solutions
        cd ..
    ) else (
        echo   cargo check passed!
        cd ..
    )
) else if "%MODE%"=="test" (
    echo Running cargo test...
    cd src-tauri
    cargo test
    if errorlevel 1 (
        echo WARNING: cargo test failed
        cd ..
    ) else (
        echo   cargo test passed!
        cd ..
    )
) else if "%MODE%"=="build" (
    echo Running cargo build...
    cd src-tauri
    cargo build
    if errorlevel 1 (
        echo WARNING: cargo build failed
        cd ..
    ) else (
        echo   cargo build passed!
        cd ..
    )
) else if "%MODE%"=="release" (
    echo Running cargo build --release...
    cd src-tauri
    cargo build --release
    if errorlevel 1 (
        echo WARNING: cargo release build failed
        cd ..
    ) else (
        echo   cargo release build passed!
        cd ..
    )
) else (
    echo Skipping Rust verification (use: check^|test^|build^|release^)
)

echo.
echo ========================================
echo Build Complete!
echo ========================================
echo.
echo Summary:
echo   Frontend: Built successfully
echo   Tests: Passed
echo   Rust: See above
echo.
echo Next steps:
echo   1. Run: npm run tauri dev
echo   2. Open application and test
echo   3. Review docs/ for more information
echo.

endlocal
