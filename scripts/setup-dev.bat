@echo off
REM Setup Development Environment for Hermes Game Operator

echo Setting up Hermes Game Operator development environment...

REM Install npm dependencies
echo Installing npm dependencies...
call npm install

REM Create .env file if not exists
if not exist .env (
    echo Creating .env file from .env.example...
    copy .env.example .env
    echo Please edit .env and add your API keys
)

REM Create test Godot project
set TEST_PROJECT=D:\tmp\test-godot-project
if not exist "%TEST_PROJECT%" (
    echo Creating test Godot project...
    mkdir "%TEST_PROJECT%\scripts"

    REM Create project.godot
    echo ; Engine configuration file. > "%TEST_PROJECT%\project.godot"
    echo config_features=PackedStringArray("4.2") >> "%TEST_PROJECT%\project.godot"
    echo config/name="Test Godot Project" >> "%TEST_PROJECT%\project.godot"
    echo run/main_scene="res://main.tscn" >> "%TEST_PROJECT%\project.godot"

    echo [OK] Test project created at %TEST_PROJECT%
) else (
    echo [OK] Test project already exists at %TEST_PROJECT%
)

REM Run tests
echo Running tests...
call npm run test -- --run

echo.
echo Setup complete!
echo.
echo To start development:
echo   npm run tauri dev
echo.
echo To run tests:
echo   npm run test
echo.