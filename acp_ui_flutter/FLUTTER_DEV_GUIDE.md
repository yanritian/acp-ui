# Flutter 项目开发文档

## 项目信息
- **项目名称**: ACP-UI Flutter
- **项目路径**: `D:\dingsun\acp-ui\acp_ui_flutter\`
- **Flutter SDK**: `D:\flutter_sdk`
- **Android SDK**: `D:\Android\Sdk`
- **Gradle Home**: `D:\Android\.gradle` (已迁移)
- **模拟器数据**: `D:\Android\.android\avd\` (已迁移)

## 开发环境配置

### 1. 环境变量
```
FLUTTER_ROOT=D:\flutter_sdk
ANDROID_HOME=D:\Android\Sdk
ANDROID_SDK_ROOT=D:\Android\Sdk
GRADLE_USER_HOME=D:\Android\.gradle
ANDROID_AVD_HOME=D:\Android\.android\avd
```

### 2. PATH 配置
```
D:\flutter_sdk\bin
D:\flutter_sdk\bin\cache\dart-sdk\bin
D:\Android\Sdk\platform-tools
D:\Android\Sdk\cmdline-tools\latest\bin
```

### 3. 关键文件修复
**Flutter.bat 修复**: `D:\flutter_sdk\bin\flutter.bat`
- 问题: Windows 环境下 `WHERE` 命令和 PowerShell 路径检测失败
- 解决: 使用 Dart 快照直接运行，跳过环境检查
- 当前内容:
```batch
@ECHO off
REM Wrapper to call flutter via dart snapshot directly to avoid environment issues
D:\flutter_sdk\bin\cache\dart-sdk\bin\dart.exe D:\flutter_sdk\bin\cache\flutter_tools.snapshot %*
```

## 构建和运行流程

### 方式一: 使用 PowerShell 脚本 (推荐)
**脚本位置**: `D:\tmp\build_and_run.ps1`

**功能**:
- 自动清理构建缓存
- 编译 APK
- 安装到模拟器
- 启动应用

**使用方法**:
```powershell
powershell -ExecutionPolicy Bypass -File D:\tmp\build_and_run.ps1
```

### 方式二: Android Studio 运行
**前提条件**:
1. Flutter 插件已安装
2. Git 路径配置正确 (`C:\Program Files\Git\cmd\git.exe`)
3. Android SDK 路径正确 (`D:\Android\Sdk`)

**操作步骤**:
1. 打开项目: `D:\dingsun\acp-ui\acp_ui_flutter\`
2. 启动模拟器: Tools → Device Manager → FlutterTest → 启动
3. 点击运行按钮 (绿色 ▶)

**常见问题**:
- `No Connected Devices Found`: 重启 ADB (`adb kill-server && adb start-server`)
- `WHERE` 命令找不到: 已修复 flutter.bat
- 设备未识别: 检查模拟器是否完全启动

### 方式三: 命令行构建
```cmd
cd D:\dingsun\acp-ui\acp_ui_flutter
flutter clean
flutter build apk --debug
adb install -r build\app\outputs\flutter-apk\app-debug.apk
adb shell am start -n acp.acp_ui_flutter/.MainActivity
```

## UI 设计说明

### 移动端适配
**文件**: `lib\features\agent_teams\agent_teams_dashboard.dart`

**设计原则**:
- Material Design 3 风格
- 紫色主色调 (`kPrimaryColor = Color(0xFF6750A4)`)
- 底部导航栏 (首页、进度、协作、宠物)
- 浮动操作按钮 (FAB)

**视图模式**:
1. **首页 (Home)**: 统计概览 + 快捷操作 + Agent 列表
2. **进度 (Progress)**: 执行状态 + 日志终端
3. **协作 (Collaboration)**: Agent 网络可视化
4. **宠物 (Pet)**: 宠物互动面板

### 主布局修复
**文件**: `lib\app.dart`

**问题**: 移动端显示桌面端侧边栏布局
**解决**: 添加屏幕宽度检测，移动端隐藏侧边栏
```dart
final screenWidth = MediaQuery.of(context).size.width;
final isMobile = screenWidth < 600;

if (isMobile) {
  return child; // 全屏显示
}
```

### 颜色警告修复
**问题**: `Colors.xxx[N]` 在较新 Flutter 版本中产生警告
**解决**: 使用 Material Design 标准颜色常量
```dart
const Color kGrey100 = Color(0xFFF5F5F5);
const Color kGrey400 = Color(0xFFBDBDBD);
// ... 等
```

## 常见问题和解决方案

### 1. Flutter 命令找不到
**原因**: PATH 未包含 Flutter SDK
**解决**:
```powershell
$env:PATH = "D:\flutter_sdk\bin;" + $env:PATH
```

### 2. Git 路径问题
**错误**: `Error: Unable to find git in your PATH`
**解决**: 配置 Git 路径到环境变量
```
C:\Program Files\Git\cmd
```

### 3. 模拟器未连接
**解决**:
```cmd
adb kill-server
adb start-server
adb devices
```

### 4. 构建失败 (Kotlin 编译错误)
**现象**: Gradle 构建时报 Kotlin daemon 错误
**原因**: 增量编译缓存损坏
**解决**: `flutter clean` 后重新构建

### 5. Symlink 权限错误
**错误**: `ERROR_ACCESS_DENIED file system exception thrown while trying to create a symlink`
**解决**:
1. 开启开发者模式 (设置 → 更新和安全 → 开发者选项)
2. 或者使用管理员权限运行

### 6. Android Studio 设备不识别
**解决**:
1. Tools → Device Manager → 刷新设备
2. 或者重启 Android Studio
3. 或者使用命令行 `adb devices` 确认连接

## C 盘空间优化

### 已完成的迁移
| 项目 | 原路径 | 新路径 | 释放空间 |
|------|--------|--------|----------|
| Gradle 缓存 | `C:\Users\...\gradle` | `D:\Android\.gradle` | ~4GB |
| 模拟器数据 | `C:\Users\...\avd` | `D:\Android\.android\avd` | ~11GB |
| pip 缓存 | `C:\Users\...\pip` | 已删除 | ~45MB |

### 环境变量配置
```powershell
# 设置用户环境变量
[System.Environment]::SetEnvironmentVariable('GRADLE_USER_HOME', 'D:\Android\.gradle', 'User')
[System.Environment]::SetEnvironmentVariable('ANDROID_AVD_HOME', 'D:\Android\.android', 'User')
```

### 清理命令
```powershell
# 清理 Gradle 缓存
Remove-Item -Recurse -Force "C:\Users\Administrator\.gradle\caches\"

# 清理 Python 缓存
Remove-Item -Recurse -Force "C:\Users\Administrator\AppData\Local\pip\"
Remove-Item -Recurse -Force "C:\Users\Administrator\.cache\pip\"

# 清理项目缓存
Remove-Item -Recurse -Force "D:\dingsun\acp-ui\acp_ui_flutter\build\"
Remove-Item -Recurse -Force "D:\dingsun\acp-ui\acp_ui_flutter\.dart_tool\"
```

## 模拟器管理

### 启动模拟器
```cmd
# 方法一: Android Studio
Tools → Device Manager → FlutterTest → 启动

# 方法二: 命令行
D:\Android\Sdk\emulator\emulator.exe -avd FlutterTest
```

### 模拟器配置
- **名称**: FlutterTest
- **系统**: Android 34 (Google APIs)
- **架构**: x86_64
- **RAM**: 4096M
- **分辨率**: 1080x2052
- **存储**: D:\Android\.android\avd\

### 连接测试
```cmd
adb devices
# 应显示: emulator-5554    device
```

## 快速操作清单

### 日常开发
1. 启动模拟器
2. 修改代码
3. 运行构建脚本: `powershell -ExecutionPolicy Bypass -File D:\tmp\build_and_run.ps1`
4. 查看模拟器效果

### 问题排查
1. 检查设备连接: `adb devices`
2. 清理缓存: `flutter clean`
3. 重新构建: `flutter build apk --debug`
4. 查看日志: `adb logcat | Select-String "flutter"`

### 维护任务
- 每周清理 Gradle 缓存
- 每月清理模拟器缓存
- 定期更新 Flutter SDK: `flutter upgrade`

## 注意事项

1. **不要直接移动 Git/Python**: 需要重新安装到目标盘
2. **模拟器文件较大**: 建议始终存储在 D 盘
3. **构建缓存会增长**: 定期执行 `flutter clean`
4. **环境变量优先级**: 用户变量 > 系统变量
5. **Android Studio 缓存**: 可定期 Invalidate Caches

## 联系方式

如遇问题，请参考本文档或联系开发团队。

---

*文档最后更新: 2026-05-23*
*维护者: AI Assistant*
