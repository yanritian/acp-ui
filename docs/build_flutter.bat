@echo off
set PATH=D:\flutter_sdk\bin;C:\Program Files\Git\cmd;C:\Program Files\Git\bin;C:\Windows\System32;C:\Windows;%PATH%
cd D:\dingsun\acp-ui\acp_ui_flutter
echo Building Flutter app...
flutter build apk --debug
echo Done!