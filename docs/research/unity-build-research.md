# Unity 构建机制深度研究报告

**日期**: 2026-06-25  
**目标**: 彻底理解 Unity 的构建机制，为 ACP-UI 集成做准备

---

## 📚 1. Unity 项目结构

### 1.1 标准项目结构

```
my_unity_project/
├── Assets/                    # 资源目录（必需）
│   ├── Scenes/                # 场景文件
│   │   ├── Main.unity         # 主场景
│   │   └── Levels/            # 关卡场景
│   ├── Scripts/               # C# 脚本
│   │   ├── Player.cs
│   │   └── Enemy.cs
│   ├── Materials/             # 材质
│   ├── Textures/              # 纹理
│   ├── Models/                # 3D 模型
│   ├── Audio/                 # 音频
│   ├── Prefabs/               # 预制体
│   ├── Plugins/               # 插件
│   ├── Resources/             # 运行时加载的资源
│   ├── StreamingAssets/       # 流式资源
│   └── Editor/                # 编辑器扩展
├── ProjectSettings/           # 项目设置（必需）
│   ├── AudioManager.asset
│   ├── ClusterInputManager.asset
│   ├── DynamicsManager.asset
│   ├── EditorBuildSettings.asset  # 构建设置（场景列表）
│   ├── EditorSettings.asset
│   ├── GraphicsSettings.asset
│   ├── InputManager.asset
│   ├── NavMeshAreas.asset
│   ├── PackageManagerSettings.asset
│   ├── Physics2DSettings.asset
│   ├── PresetManager.asset
│   ├── ProjectSettings.asset
│   ├── QualitySettings.asset
│   ├── TagManager.asset
│   ├── TimeManager.asset
│   └── UnityConnectSettings.asset
├── Packages/                  # Package Manager
│   ├── manifest.json          # 依赖清单
│   └── packages-lock.json     # 锁定版本
├── Library/                   # 编译缓存（自动生成）
│   ├── ScriptAssemblies/      # 编译后的 DLL
│   └── PlayerDataCache/       # 玩家数据缓存
├── Logs/                      # 日志文件
├── Temp/                      # 临时文件
├── obj/                       # 编译对象
└── UserSettings/              # 用户设置
```

### 1.2 关键文件说明

**EditorBuildSettings.asset（场景列表）：**
```yaml
%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!1045 &1
EditorBuildSettings:
  m_ObjectHideFlags: 0
  serializedVersion: 2
  m_Scenes:
  - enabled: 1
    path: Assets/Scenes/Main.unity
    guid: abc123...
  - enabled: 1
    path: Assets/Scenes/Level1.unity
    guid: def456...
```

**manifest.json（包依赖）：**
```json
{
  "dependencies": {
    "com.unity.2d.sprite": "1.0.0",
    "com.unity.2d.tilemap": "1.0.0",
    "com.unity.collab-proxy": "1.3.9",
    "com.unity.ide.rider": "2.0.7",
    "com.unity.ide.visualstudio": "2.0.7",
    "com.unity.ide.vscode": "1.2.3",
    "com.unity.test-framework": "1.1.24",
    "com.unity.textmeshpro": "3.0.4",
    "com.unity.timeline": "1.4.7",
    "com.unity.ugui": "1.0.0",
    "com.unity.modules.ai": "1.0.0",
    "com.unity.modules.androidjni": "1.0.0"
  }
}
```

---

## 🔧 2. Unity 构建命令

### 2.1 基本构建命令

```bash
# 构建 Windows 可执行文件
Unity -quit -batchmode -projectPath /path/to/project -buildTarget Win64 -buildApp /path/to/output/game.exe

# 构建 MacOS 应用
Unity -quit -batchmode -projectPath /path/to/project -buildTarget OSX -buildApp /path/to/output/game.app

# 构建 Linux 可执行文件
Unity -quit -batchmode -projectPath /path/to/project -buildTarget Linux64 -buildApp /path/to/output/game.x86_64

# 构建 Android APK
Unity -quit -batchmode -projectPath /path/to/project -buildTarget Android -buildApp /path/to/output/game.apk

# 构建 iOS Xcode 项目
Unity -quit -batchmode -projectPath /path/to/project -buildTarget iOS -buildApp /path/to/output/ios_project
```

### 2.2 命令行参数详解

| 参数 | 说明 | 必需 |
|------|------|------|
| `-quit` | 构建完成后退出 Unity | 是 |
| `-batchmode` | 批处理模式（无 GUI） | 是 |
| `-projectPath` | Unity 项目路径 | 是 |
| `-buildTarget` | 构建目标平台 | 是 |
| `-buildApp` | 输出文件路径 | 是 |
| `-logFile` | 日志文件路径 | 否 |
| `-executeMethod` | 执行自定义构建方法 | 否 |
| `-customBuildOptions` | 自定义构建选项 | 否 |

### 2.3 构建目标平台

| 平台 | -buildTarget 值 | 输出文件 | 说明 |
|------|----------------|---------|------|
| Windows 32 | Win | .exe | 32 位 Windows |
| Windows 64 | Win64 | .exe | 64 位 Windows |
| MacOS | OSX | .app | MacOS 应用 |
| Linux | Linux64 | .x86_64 | Linux 可执行 |
| Android | Android | .apk | Android 安装包 |
| iOS | iOS | .xcodeproj | Xcode 项目 |
| WebGL | WebGL | index.html | Web 游戏 |

### 2.4 构建流程

```
1. 启动 Unity（批处理模式）
   ↓
2. 加载项目
   - 解析 Assets/
   - 解析 ProjectSettings/
   - 导入 Packages/
   ↓
3. 编译脚本
   - 编译 C# 脚本
   - 生成 DLL
   - 检查编译错误
   ↓
4. 构建场景
   - 加载场景列表
   - 序列化场景数据
   - 打包资源
   ↓
5. 生成可执行文件
   - 创建播放器
   - 嵌入资源
   - 应用设置
   ↓
6. 输出结果
   - 生成 .exe（Windows）
   - 生成 .app（MacOS）
   - 生成 .x86_64（Linux）
   - 生成 .apk（Android）
   - 生成 WebGL 文件
   ↓
7. 退出 Unity
```

---

## 🎯 3. 自定义构建方法

### 3.1 创建构建脚本

**Assets/Editor/BuildScript.cs：**
```csharp
using UnityEditor;
using UnityEngine;
using System.Linq;

public static class BuildScript
{
    [MenuItem("Build/Build Windows")]
    public static void BuildWindows()
    {
        // 获取场景列表
        var scenes = EditorBuildSettings.scenes
            .Where(s => s.enabled)
            .Select(s => s.path)
            .ToArray();
        
        // 构建选项
        var options = new BuildPlayerOptions
        {
            scenes = scenes,
            locationPathName = "Builds/Windows/game.exe",
            target = BuildTarget.StandaloneWindows64,
            options = BuildOptions.None
        };
        
        // 执行构建
        var report = BuildPipeline.BuildPlayer(options);
        
        if (report.summary.result == UnityEditor.BuildResult.Succeeded)
        {
            Debug.Log("Build succeeded: " + report.summary.outputPath);
        }
        else
        {
            Debug.LogError("Build failed");
        }
    }
    
    [MenuItem("Build/Build Android")]
    public static void BuildAndroid()
    {
        var scenes = EditorBuildSettings.scenes
            .Where(s => s.enabled)
            .Select(s => s.path)
            .ToArray();
        
        var options = new BuildPlayerOptions
        {
            scenes = scenes,
            locationPathName = "Builds/Android/game.apk",
            target = BuildTarget.Android,
            options = BuildOptions.None
        };
        
        var report = BuildPipeline.BuildPlayer(options);
        
        if (report.summary.result == UnityEditor.BuildResult.Succeeded)
        {
            Debug.Log("Build succeeded: " + report.summary.outputPath);
        }
        else
        {
            Debug.LogError("Build failed");
        }
    }
}
```

### 3.2 通过命令行调用自定义方法

```bash
# 调用自定义构建方法
Unity -quit -batchmode -projectPath /path/to/project -executeMethod BuildScript.BuildWindows

# 带参数调用
Unity -quit -batchmode -projectPath /path/to/project -executeMethod BuildScript.Build -customBuildOptions development
```

---

## ⚠️ 4. 常见错误与解决方案

### 4.1 编译错误

**错误信息：**
```
Assets/Scripts/Player.cs(10,5): error CS1002: ; expected
```

**解决方案：**
```bash
# 查看详细的编译日志
Unity -quit -batchmode -projectPath /path/to/project -logFile build.log

# 检查 build.log 中的错误
cat build.log | grep "error"
```

### 4.2 缺少场景

**错误信息：**
```
Build completed with a result of 'Failed'
Build completed with a result of 'Failed' in 0 seconds
No enabled scenes were found in the build settings
```

**解决方案：**
```bash
# 检查 EditorBuildSettings.asset
cat ProjectSettings/EditorBuildSettings.asset

# 确保至少有一个场景被启用
# 或在构建脚本中指定场景
```

### 4.3 许可证问题

**错误信息：**
```
License Error: No valid Unity license found
```

**解决方案：**
```bash
# 激活许可证
Unity -quit -batchmode -serial XXXX-XXXX-XXXX-XXXX-XXXX -username user@example.com -password password

# 或使用 Unity Plus/Pro
# Personal 版本有构建限制
```

### 4.4 内存不足

**错误信息：**
```
OutOfMemoryException: Out of memory
```

**解决方案：**
```bash
# 增加 Unity 内存限制
# 编辑 ProjectSettings/ProjectSettings.asset
# 增加内存限制

# 或优化项目资源
# 压缩纹理
# 减少多边形数量
```

### 4.5 构建目标不支持

**错误信息：**
```
Build target 'XXX' is not supported in this Unity version
```

**解决方案：**
```bash
# 检查 Unity 版本支持的平台
# Personal 版本不支持某些平台
# 升级到 Plus 或 Pro 版本
```

---

## 📊 5. 构建性能数据

### 5.1 构建时间参考

| 项目大小 | 脚本数 | 场景数 | 构建时间 |
|---------|--------|--------|---------|
| 小型（< 50MB） | 10-50 | 1-5 | 30-60 秒 |
| 中型（50-500MB） | 50-200 | 5-20 | 2-5 分钟 |
| 大型（500MB-2GB） | 200-500 | 20-50 | 5-15 分钟 |
| 超大型（> 2GB） | 500+ | 50+ | 15-30 分钟 |

### 5.2 内存使用参考

| 项目大小 | 构建时内存 | 运行时内存 |
|---------|-----------|-----------|
| 小型 | 1-2GB | 200-500MB |
| 中型 | 2-4GB | 500MB-1GB |
| 大型 | 4-8GB | 1-2GB |

---

## 🔬 6. 测试计划

### 6.1 测试用例

**测试 1：基本构建**
```
输入：标准 Unity 项目
期望：成功构建 .exe
验证：
- 文件存在
- 文件大小 > 0
- 可以运行
```

**测试 2：编译错误**
```
输入：有编译错误的项目
期望：构建失败，错误提示
验证：
- 错误信息准确
- 提供修复建议
```

**测试 3：缺少场景**
```
输入：没有启用场景的项目
期望：构建失败，友好提示
验证：
- 提示添加场景
- 提供解决方案
```

**测试 4：大型项目**
```
输入：2GB 项目
期望：成功构建
验证：
- 构建时间 < 30 分钟
- 内存使用 < 8GB
- 输出文件完整
```

### 6.2 性能基准

| 指标 | 目标值 |
|------|--------|
| 检测时间 | < 1 秒 |
| 构建时间（小型） | < 1 分钟 |
| 构建时间（中型） | < 5 分钟 |
| 内存使用 | < 4GB |
| 成功率 | > 90% |

---

## 📝 7. 实现建议

### 7.1 检测 Unity 项目

```rust
pub fn detect_unity_project(cwd: &Path) -> bool {
    cwd.join("Assets").exists() && cwd.join("ProjectSettings").exists()
}
```

### 7.2 获取 Unity 版本

```rust
pub fn get_unity_version(cwd: &Path) -> Result<String, Error> {
    let path = cwd.join("ProjectSettings/ProjectVersion.txt");
    let content = fs::read_to_string(path)?;
    
    // 解析 m_EditorVersion: 2021.3.0f1
    let re = Regex::new(r"m_EditorVersion:\s*(\S+)")?;
    if let Some(caps) = re.captures(&content) {
        Ok(caps[1].to_string())
    } else {
        Err(Error::VersionNotFound)
    }
}
```

### 7.3 查找 Unity 可执行文件

```rust
pub fn find_unity_executable() -> Result<String, Error> {
    // 1. 检查环境变量
    if let Ok(path) = std::env::var("UNITY_PATH") {
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    
    // 2. 检查常见位置
    let common_paths = vec![
        // Windows
        "C:\\Program Files\\Unity\\Hub\\Editor\\2021.3.0f1\\Editor\\Unity.exe",
        "C:\\Program Files\\Unity\\Hub\\Editor\\2022.3.0f1\\Editor\\Unity.exe",
        // MacOS
        "/Applications/Unity/Hub/Editor/2021.3.0f1/Unity.app/Contents/MacOS/Unity",
        "/Applications/Unity/Hub/Editor/2022.3.0f1/Unity.app/Contents/MacOS/Unity",
        // Linux
        "/opt/Unity/Hub/Editor/2021.3.0f1/Editor/Unity",
    ];
    
    for path in common_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    // 3. 检查 Unity Hub
    // Unity Hub 可以管理多个 Unity 版本
    
    Err(Error::UnityNotFound)
}
```

### 7.4 执行构建

```rust
pub async fn build_unity(
    cwd: &Path,
    target: &str,
    output: &Path,
) -> Result<(), Error> {
    let unity = find_unity_executable()?;
    
    let output_cmd = Command::new(&unity)
        .arg("-quit")
        .arg("-batchmode")
        .arg("-projectPath")
        .arg(cwd)
        .arg("-buildTarget")
        .arg(target)
        .arg("-buildApp")
        .arg(output)
        .output()
        .await?;
    
    if !output_cmd.status.success() {
        let stderr = String::from_utf8_lossy(&output_cmd.stderr);
        return Err(Error::BuildFailed(stderr.to_string()));
    }
    
    Ok(())
}
```

---

## 📚 8. Unity vs Godot 对比

| 特性 | Unity | Godot |
|------|-------|-------|
| 脚本语言 | C# | GDScript, C# |
| 构建命令 | -batchmode | --headless |
| 构建目标 | -buildTarget | --export-release |
| 输出路径 | -buildApp | 最后一个参数 |
| 许可证 | 需要（Personal/Plus/Pro） | 免费 |
| 构建模板 | 内置 | 需要下载 |
| 构建时间 | 较慢 | 较快 |
| 内存使用 | 较高 | 较低 |
| 跨平台 | 需要许可证 | 免费 |

---

## 📚 9. 参考资料

- [Unity 官方文档 - 命令行参数](https://docs.unity3d.com/Manual/CommandLineArguments.html)
- [Unity 官方文档 - 构建](https://docs.unity3d.com/Manual/BuildSettings.html)
- [Unity 官方文档 - 批处理模式](https://docs.unity3d.com/Manual/CommandLineArguments.html#batchmode)
- [Unity 源码 - 构建模块](https://github.com/Unity-Technologies/UnityCsReference)

---

**最后更新**: 2026-06-25  
**状态**: 🟢 完成  
**下一步**: Phase 1 - 引擎集成实现
