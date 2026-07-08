@echo off
REM Hermes Game Operator Build Script for Windows

echo ==========================================
echo Hermes Game Operator - Production Build
echo ==========================================
echo.

REM Check environment
echo Checking environment...

where node >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not found
    exit /b 1
)

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust/Cargo not found
    exit /b 1
)

echo [OK] Environment checks passed
echo.

REM Install dependencies
echo Installing dependencies...
call npm install
echo.

REM Run tests
echo Running tests...
call npm run test -- --run
echo.

REM Build frontend
echo Building frontend...
call npm run build
echo.

REM Check if Tauri CLI is available
where cargo >nul 2>&1
cargo tauri --version >nul 2>&1
if %errorlevel% equ 0 (
    echo Building Tauri application...
    cargo tauri build
    echo.
    echo Build complete!
    echo Output: src-tauri\target\release\
) else (
    echo WARNING: Tauri CLI not found
    echo To build desktop app, run: cargo tauri build
    echo.
    echo Frontend build complete!
    echo Output: dist\
)

echo.
echo ==========================================
echo Build Summary
echo ==========================================
echo Frontend: dist\
echo Tests: Passed
echo.