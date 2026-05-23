@echo off
echo ========================================
echo Flutter Build Script
echo ========================================
echo.
pause

REM Set environment variables
set ANDROID_SDK_ROOT=D:\Android\Sdk
set ANDROID_HOME=D:\Android\Sdk
set PATH=D:\flutter_sdk\bin;D:\flutter_sdk\bin\cache\dart-sdk\bin;C:\Program Files\Git\cmd;C:\Program Files\Git\mingw64\bin;D:\Android\Sdk\platform-tools;C:\Windows\System32;C:\Windows;%PATH%

echo.
echo [1/4] Checking Flutter environment...
where flutter >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Flutter not found!
    echo Please check if D:\flutter_sdk exists
    pause
    exit /b 1
)
flutter --version

echo.
echo [2/4] Checking device connection...
adb devices
pause

echo.
echo [3/4] Cleaning previous build...
flutter clean
if %errorlevel% neq 0 (
    echo ERROR: Clean failed!
    pause
    exit /b 1
)

echo.
echo [4/4] Building and Installing...
flutter build apk --debug
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

adb install -r build/app/outputs/flutter-apk/app-debug.apk
if %errorlevel% neq 0 (
    echo ERROR: Install failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo SUCCESS! Build and Install Complete!
echo ========================================
echo Please check your emulator for the new UI.
echo.
pause