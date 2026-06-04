@echo off
set ANDROID_SDK_ROOT=D:\Android\Sdk
set ANDROID_HOME=D:\Android\Sdk
set PATH=C:\Windows\System32;C:\Program Files\Git\cmd;D:\flutter_sdk\bin;%PATH%

echo ============================================
echo Flutter App Test - Agent Teams Platform
echo ============================================
echo.

cd /d D:\dingsun\acp-ui\acp_ui_flutter

echo Checking Flutter...
flutter --version
echo.

echo Checking devices...
flutter devices
echo.

echo Running Flutter app...
flutter run -d emulator-5554

pause