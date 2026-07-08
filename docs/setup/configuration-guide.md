# Hermes Game Operator 配置指南

> 版本: v0.1.0
> 更新日期: 2026-07-09

---

## 1. 环境要求

### 1.1 必需软件

| 软件 | 版本 | 用途 |
|------|------|------|
| Node.js | >= 18.x | 前端构建 |
| Rust | >= 1.70 | Tauri 后端 |
| Windows 10 SDK | 最新 | Rust 编译 |
| Hermes CLI | >= 1.0.0 | Agent 执行 |

### 1.2 可选软件

| 软件 | 用途 |
|------|------|
| Godot 4.x | 测试项目 |
| VSCode | 开发调试 |

---

## 2. 安装步骤

### 2.1 安装 Windows 10 SDK

1. 打开 **Visual Studio Installer**
2. 点击 **Modify**
3. 切换到 **Individual components** 标签
4. 搜索 **Windows 10 SDK**
5. 勾选最新版本
6. 点击 **Modify** 安装

### 2.2 安装 Hermes CLI

```bash
# 方式 1: 从源码编译
git clone https://github.com/hermes-ai/hermes-cli
cd hermes-cli
cargo install --path .

# 方式 2: 使用预编译包
# 下载: https://github.com/hermes-ai/hermes-cli/releases
# 解压到 PATH 目录
```

### 2.3 安装项目依赖

```bash
# 前端依赖
npm install

# 验证 Rust 环境 (需要 Windows SDK)
cd src-tauri
cargo check
```

---

## 3. 配置文件

### 3.1 环境变量

创建 `.env` 文件:

```env
# Hermes Agent API
HERMES_API_ENDPOINT=https://api.hermes.ai
HERMES_API_KEY=your-api-key-here

# Godot 配置
GODOT_PATH=/path/to/godot
GODOT_VERSION=4.2

# 日志级别
LOG_LEVEL=info
```

### 3.2 Tauri 配置

编辑 `src-tauri/tauri.conf.json`:

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "Hermes Game Operator",
    "version": "0.1.0"
  }
}
```

### 3.3 审批策略配置

```json
{
  "approval_policy": {
    "default": "safe_default",
    "file_read": "silent",
    "file_write": "approve",
    "command_execute": "approve",
    "dangerous_commands": "forbidden"
  }
}
```

---

## 4. 启动应用

### 4.1 开发模式

```bash
# 启动开发服务器
npm run tauri dev
```

### 4.2 生产构建

```bash
# 构建应用
npm run tauri build

# 输出位置
# src-tauri/target/release/
```

---

## 5. 测试

### 5.1 单元测试

```bash
npm run test
```

### 5.2 E2E 测试

```bash
# 启动开发服务器
npm run dev

# 运行 E2E 测试
npm run test:e2e
```

### 5.3 Rust 测试

```bash
cd src-tauri
cargo test
```

---

## 6. 故障排除

### 6.1 cargo check 失败

**错误**: `link.exe not found`

**解决**: 安装 Windows 10 SDK

### 6.2 Hermes CLI 未找到

**错误**: `Hermes CLI not found`

**解决**:
```bash
# 检查 Hermes 是否在 PATH
hermes --version

# 或设置完整路径
export HERMES_CLI_PATH=/path/to/hermes
```

### 6.3 API Key 无效

**错误**: `ANTHROPIC_API_KEY not set`

**解决**:
```bash
# 设置环境变量
export ANTHROPIC_API_KEY=sk-xxx

# 或在 .env 文件中配置
```

---

## 7. 示例项目

### 7.1 创建测试 Godot 项目

```bash
# 创建项目目录
mkdir -p D:/tmp/test-godot-project/scripts

# 创建 project.godot
cat > D:/tmp/test-godot-project/project.godot << 'EOF'
; Engine configuration file.
config_features=PackedStringArray("4.2")
config/name="Test Godot Project"
run/main_scene="res://main.tscn"
EOF

# 创建简单脚本
cat > D:/tmp/test-godot-project/scripts/Player.gd << 'EOF'
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0

func _physics_process(delta):
    var direction = Input.get_axis("ui_left", "ui_right")
    velocity.x = direction * SPEED
    move_and_slide()
EOF
```

### 7.2 测试完整流程

1. 启动应用: `npm run tauri dev`
2. 导航到 Game Operator
3. 选择项目路径: `D:/tmp/test-godot-project`
4. 输入目标: `给 Player 添加二段跳能力`
5. 点击 **Start Task**
6. 观察事件流
7. 审批文件修改
8. 查看任务总结

---

## 8. 常用命令

```bash
# 开发
npm run dev           # 启动前端开发服务器
npm run tauri dev     # 启动 Tauri 应用

# 构建
npm run build         # 构建前端
npm run tauri build   # 构建应用

# 测试
npm run test          # 运行单元测试
npm run test:e2e      # 运行 E2E 测试

# 代码检查
npm run lint          # ESLint 检查
npm run typecheck     # TypeScript 类型检查
```

---

## 9. 下一步

1. 安装 Windows 10 SDK
2. 验证 `cargo check` 通过
3. 安装 Hermes CLI
4. 使用真实 Godot 项目测试
5. 配置 API Key
6. 运行完整流程