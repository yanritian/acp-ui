# Godot 导出机制深度研究报告

**日期**: 2026-06-25  
**目标**: 彻底理解 Godot 的导出机制，为 ACP-UI 集成做准备

---

## 📚 1. Godot 项目结构

### 1.1 标准项目结构

```
my_godot_project/
├── project.godot          # 项目配置文件（必需）
├── export_presets.cfg     # 导出预设配置（可选）
├── .godot/                # 编辑器缓存（自动生成）
├── assets/                # 资源目录
│   ├── textures/          # 纹理
│   ├── models/            # 3D 模型
│   ├── audio/             # 音频
│   └── fonts/             # 字体
├── scenes/                # 场景文件
│   ├── main.tscn          # 主场景
│   └── levels/            # 关卡场景
├── scripts/               # GDScript 脚本
│   ├── player.gd
│   └── enemy.gd
└── export/                # 导出目录（我们自己创建）
```

### 1.2 project.godot 文件结构

```ini
; Engine configuration file.
; It's best edited using the editor UI and not directly,
; but it can also be manually edited if needed.

[application]

config/name="My Game"
config/description="A Godot game"
run/main_scene="res://scenes/main.tscn"
config/features=PackedStringArray("4.2", "GL Compatibility")
config/icon="res://icon.svg"

[display]

window/size/viewport_width=1920
window/size/viewport_height=1080
window/stretch/mode="canvas_items"

[input]

move_up={
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":87,"physical_keycode":0,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)
]
}

[rendering]

renderer/rendering_method="gl_compatibility"
renderer/rendering_method.mobile="gl_compatibility"
textures/default_filters/anisotropic_filtering_level=4
```

### 1.3 export_presets.cfg 文件结构

```ini
[preset.0]

name="Windows Desktop"
platform="Windows Desktop"
runnable=true
dedicated_server=false
custom_features=""
export_filter="all_resources"
include_filter=""
exclude_filter=""
export_path="export/game.exe"
encryption_include_filters=""
encryption_exclude_filters=""
encrypt_pck=false
encrypt_directory=false

[preset.0.options]

custom_template/debug=""
custom_template/release=""
debug/export_console_wrapper=1
binary_format/embed_pck=true
texture_format/s3tc_bptc=true
texture_format/etc2_astc=false
binary_format/architecture="x86_64"
codesign/enable=false
codesign/identity_type=0
codesign/identity=""
codesign/password=""
codesign/timestamp=true
codesign/timestamp_server_url=""
codesign/digest_algorithm=1
codesign/description=""
codesign/custom_options=PackedStringArray()
application/modify_resources=false
application/icon=""
application/console_wrapper_icon=""
application/icon_interpolation=4
application/file_version=""
application/product_version=""
application/company_name=""
application/product_name=""
application/file_description=""
application/copyright=""
application/trademarks=""
application/export_angle=0
ssh_remote_deploy/enabled=false
ssh_remote_deploy/host="user@host_ip"
ssh_remote_deploy/port="22"
ssh_remote_deploy/extra_args_ssh=""
ssh_remote_deploy/extra_args_scp=""
ssh_remote_deploy/run_script="Expand-Archive -LiteralPath '{temp_dir}\\{archive_name}' -DestinationPath '{temp_dir}'
$action = New-Object System.Net.WebClient
$url = '{godot_url}'
$action.DownloadFile('{godot_url}', '{temp_dir}\\godot.zip')
Expand-Archive -LiteralPath '{temp_dir}\\godot.zip' -DestinationPath '{temp_dir}'
& '{temp_dir}\\godot.exe' --path '{project_dir}' --headless --export-debug '{preset_name}' '{temp_dir}\\{archive_name}'"
```

---

## 🔧 2. Godot 导出命令

### 2.1 基本导出命令

```bash
# 导出项目（release 模式）
godot --headless --path /path/to/project --export-release "Windows Desktop" output.exe

# 导出项目（debug 模式）
godot --headless --path /path/to/project --export-debug "Windows Desktop" output.exe

# 导出所有预设
godot --headless --path /path/to/project --export-all
```

### 2.2 命令行参数详解

| 参数 | 说明 | 示例 |
|------|------|------|
| `--headless` | 无头模式（不显示 GUI） | 必需 |
| `--path` | 项目路径 | `/path/to/project` |
| `--export-release` | 导出 release 版本 | `"Windows Desktop"` |
| `--export-debug` | 导出 debug 版本 | `"Windows Desktop"` |
| `--export-all` | 导出所有预设 | 无 |
| `--verbose` | 详细输出 | 可选 |

### 2.3 导出流程

```
1. 加载 project.godot
   ↓
2. 解析 export_presets.cfg
   ↓
3. 验证导出配置
   - 检查导出模板
   - 检查资源完整性
   - 检查脚本语法
   ↓
4. 编译脚本
   - 编译 GDScript
   - 编译 C#（如果有）
   ↓
5. 打包资源
   - 打包纹理
   - 打包模型
   - 打包音频
   - 打包场景
   ↓
6. 生成可执行文件
   - 创建 .pck 文件
   - 嵌入到可执行文件
   - 或创建独立文件
   ↓
7. 输出结果
   - 生成 .exe（Windows）
   - 生成 .app（MacOS）
   - 生成 .x86_64（Linux）
   - 生成 .apk（Android）
   - 生成 .html5（Web）
```

---

## 🎯 3. 导出预设配置

### 3.1 支持的导出平台

| 平台 | 预设名称 | 输出文件 | 说明 |
|------|---------|---------|------|
| Windows | Windows Desktop | .exe | 需要导出模板 |
| MacOS | Mac OSX | .app, .dmg | 需要导出模板 |
| Linux | Linux/X11 | .x86_64 | 需要导出模板 |
| Android | Android | .apk | 需要 JDK + SDK |
| iOS | iOS | .ipa | 需要 Xcode |
| Web | Web | .html, .wasm | 需要导出模板 |

### 3.2 导出模板

**什么是导出模板？**
- Godot 引擎的预编译版本
- 每个平台需要对应的模板
- 模板决定了游戏运行的基础环境

**模板位置：**
- Windows: `%APPDATA%\Godot\export_templates\<version>\windows_release_x86_64.exe`
- MacOS: `~/Library/Application Support/Godot/export_templates/<version>/macos_release.app`
- Linux: `~/.local/share/godot/export_templates/<version>/linux_release.x86_64`

**模板下载：**
- 从 Godot 官网下载
- 或通过编辑器自动下载
- 版本必须与项目版本匹配

### 3.3 导出选项详解

**Windows 导出选项：**

```ini
[preset.0.options]

# 自定义模板（可选）
custom_template/debug=""
custom_template/release=""

# 控制台包装器
# 0 = 无控制台
# 1 = 有控制台（可看到输出）
debug/export_console_wrapper=1

# 嵌入 PCK 文件
# true = 将 .pck 嵌入 .exe
# false = 创建独立的 .pck 和 .exe
binary_format/embed_pck=true

# 纹理格式
texture_format/s3tc_bptc=true   # 高质量压缩
texture_format/etc2_astc=false  # 移动端格式

# 架构
binary_format/architecture="x86_64"  # 64 位

# 代码签名（MacOS/iOS）
codesign/enable=false
codesign/identity=""

# 应用程序信息
application/company_name="My Company"
application/product_name="My Game"
application/file_description="A game made with Godot"
application/copyright="Copyright 2026"
```

---

## ⚠️ 4. 常见错误与解决方案

### 4.1 导出模板缺失

**错误信息：**
```
ERROR: No export template found at path: 
C:\Users\...\export_templates\4.2\windows_release_x86_64.exe
```

**解决方案：**
```bash
# 方法 1：下载模板
# 访问 https://godotengine.org/download
# 下载对应版本的导出模板

# 方法 2：通过编辑器下载
# Godot 编辑器 -> 项目 -> 导出 -> 下载模板
```

### 4.2 脚本编译错误

**错误信息：**
```
SCRIPT ERROR: GDScript::load: 
res://scripts/player.gd:10 - Parse Error: Unexpected "EOF"
```

**解决方案：**
```bash
# 检查脚本语法
godot --headless --path /path/to/project --check-only

# 修复脚本错误
# 重新导出
```

### 4.3 资源缺失

**错误信息：**
```
ERROR: res://textures/missing.png: Resource not found
```

**解决方案：**
```bash
# 检查资源路径
# 确保所有资源都存在
# 更新 project.godot 中的引用
```

### 4.4 磁盘空间不足

**错误信息：**
```
ERROR: Failed to write file: export/game.exe
```

**解决方案：**
```bash
# 检查磁盘空间
# 清理临时文件
# 确保有足够的空间（至少 2 倍项目大小）
```

---

## 📊 5. 导出性能数据

### 5.1 导出时间参考

| 项目大小 | 场景数 | 资源数 | 导出时间 |
|---------|--------|--------|---------|
| 小型（< 10MB） | 1-5 | 10-50 | 5-10 秒 |
| 中型（10-100MB） | 5-20 | 50-200 | 15-30 秒 |
| 大型（100MB-1GB） | 20-100 | 200-1000 | 1-3 分钟 |
| 超大型（> 1GB） | 100+ | 1000+ | 5-10 分钟 |

### 5.2 内存使用参考

| 项目大小 | 导出时内存 | 运行时内存 |
|---------|-----------|-----------|
| 小型 | 200-500MB | 100-300MB |
| 中型 | 500MB-1GB | 300MB-800MB |
| 大型 | 1-2GB | 800MB-2GB |

---

## 🔬 6. 测试计划

### 6.1 测试用例

**测试 1：基本导出**
```
输入：标准 Godot 项目
期望：成功导出 .exe
验证：
- 文件存在
- 文件大小 > 0
- 可以运行
```

**测试 2：缺少导出模板**
```
输入：没有导出模板
期望：友好的错误提示
验证：
- 错误信息清晰
- 提供解决方案
```

**测试 3：脚本错误**
```
输入：有语法错误的项目
期望：编译错误提示
验证：
- 错误位置准确
- 错误信息清晰
```

**测试 4：大型项目**
```
输入：1GB 项目
期望：成功导出
验证：
- 导出时间 < 5 分钟
- 内存使用 < 2GB
- 输出文件完整
```

### 6.2 性能基准

| 指标 | 目标值 |
|------|--------|
| 检测时间 | < 500ms |
| 导出时间（小型） | < 15 秒 |
| 导出时间（中型） | < 45 秒 |
| 内存使用 | < 1GB |
| 成功率 | > 95% |

---

## 📝 7. 实现建议

### 7.1 检测 Godot 项目

```rust
pub fn detect_godot_project(cwd: &Path) -> bool {
    cwd.join("project.godot").exists()
}
```

### 7.2 获取 Godot 版本

```rust
pub fn get_godot_version(cwd: &Path) -> Result<String, Error> {
    let content = fs::read_to_string(cwd.join("project.godot"))?;
    
    // 查找 config/features=PackedStringArray("4.2", ...)
    let re = Regex::new(r#"PackedStringArray\("(\d+\.\d+)"#)?;
    if let Some(caps) = re.captures(&content) {
        Ok(caps[1].to_string())
    } else {
        Err(Error::VersionNotFound)
    }
}
```

### 7.3 查找 Godot 可执行文件

```rust
pub fn find_godot_executable() -> Result<String, Error> {
    // 1. 检查环境变量
    if let Ok(path) = std::env::var("GODOT_PATH") {
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    
    // 2. 检查 PATH
    if let Ok(output) = Command::new("godot").arg("--version").output() {
        if output.status.success() {
            return Ok("godot".to_string());
        }
    }
    
    // 3. 检查常见位置
    let common_paths = vec![
        "C:\\Program Files\\Godot\\godot.exe",
        "/usr/local/bin/godot",
        "/Applications/Godot.app/Contents/MacOS/Godot",
    ];
    
    for path in common_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    Err(Error::GodotNotFound)
}
```

### 7.4 执行导出

```rust
pub async fn export_godot(
    cwd: &Path,
    preset: &str,
    output: &Path,
    debug: bool,
) -> Result<(), Error> {
    let godot = find_godot_executable()?;
    
    let mode = if debug { "--export-debug" } else { "--export-release" };
    
    let output = Command::new(&godot)
        .arg("--headless")
        .arg("--path")
        .arg(cwd)
        .arg(mode)
        .arg(preset)
        .arg(output)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::ExportFailed(stderr.to_string()));
    }
    
    Ok(())
}
```

---

## 📚 8. 参考资料

- [Godot 官方文档 - 导出](https://docs.godotengine.org/en/stable/tutorials/export/index.html)
- [Godot 命令行参数](https://docs.godotengine.org/en/stable/tutorials/command_line.html)
- [Godot 导出模板](https://docs.godotengine.org/en/stable/tutorials/export/exporting_projects.html)
- [Godot 源码 - 导出模块](https://github.com/godotengine/godot/tree/master/editor/export)

---

**最后更新**: 2026-06-25  
**状态**: 🟡 调研中  
**下一步**: 创建测试项目，验证导出流程
