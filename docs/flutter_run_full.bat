@echo off
set ANDROID_SDK_ROOT=D:\Android\Sdk
set ANDROID_HOME=D:\Android\Sdk
set PATH=C:\Windows\System32;C:\Windows\System32\WindowsPowerShell\v1.0;C:\Program Files\Git\cmd;D:\flutter_sdk\bin;%PATH%

echo ============================================
echo Flutter Pub Get - Restore Dependencies
echo ============================================
echo.

cd /d D:\dingsun\acp-ui\acp_ui_flutter

echo Running flutter pub get...
flutter pub get
echo.

echo Done! Now running flutter app...
flutter run -d emulator-5554

pause