@echo off
REM Hermes Game Operator Release Script for Windows

set VERSION=%1
if "%VERSION%"=="" set VERSION=0.1.0

echo ==========================================
echo Hermes Game Operator - Release v%VERSION%
echo ==========================================
echo.

REM Update version in package.json
echo Updating package.json version to %VERSION%...
echo Please manually update package.json version field to: %VERSION%
echo.

REM Update version in Cargo.toml
echo Updating Cargo.toml version to %VERSION%...
echo Please manually update src-tauri\Cargo.toml version field to: %VERSION%
echo.

REM Run tests
echo Running tests...
call npm run test -- --run
echo.

REM Build
echo Building...
call npm run build
echo.

REM Create git tag
echo Creating git tag v%VERSION%...
git tag -a "v%VERSION%" -m "Release v%VERSION%"
echo.

echo ==========================================
echo Release v%VERSION% Prepared
echo ==========================================
echo.
echo Next steps:
echo 1. Review and update package.json version to %VERSION%
echo 2. Review and update src-tauri\Cargo.toml version to %VERSION%
echo 3. Update CHANGELOG.md
echo 4. Commit: git commit -am "chore: release v%VERSION%"
echo 5. Push: git push origin main --tags
echo.