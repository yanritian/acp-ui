# Hermes Game Operator 安装指南

> 版本: 0.1.0-alpha
> 更新时间: 2026-07-09

---

## 系统要求

### 必需

- **Node.js**: 18.x 或更高版本
- **pnpm**: 8.x 或更高版本
- **Rust**: 1.70 或更高版本
- **Windows 10 SDK**: 用于 Rust 编译

### 可选

- **Hermes CLI**: 用于真实 Agent 执行

---

## 安装步骤

### 1. 安装 Node.js

```powershell
# 使用 winget 安装
winget install OpenJS.NodeJS.LTS

# 验证
node --version
npm --version
```

### 2. 安装 pnpm

```powershell
npm install -g pnpm

# 验证
pnpm --version
```

### 3. 安装 Rust

```powershell
# 下载并运行 rustup-init.exe
# https://rustup.rs/

# 验证
rustc --version
cargo --version
```

### 4. 安装 Windows 10 SDK (必需)

```powershell
# 使用 winget 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 或手动安装
# https://visualstudio.microsoft.com/downloads/
# 选择 "C++ build tools" 工作负载
```

### 5. 安装项目依赖

```powershell
cd D:\dingsun\acp-ui
pnpm install
```

---

## 验证安装

### 前端验证

```powershell
# 构建
npm run build

# 测试
npm run test

# 类型检查
npm run typecheck
```

### Rust 验证

```powershell
cd src-tauri

# 检查编译
cargo check

# 运行 Rust 测试 (需要 Windows SDK)
cargo test
```

---

## 安装 Hermes CLI (可选)

Hermes CLI 用于真实的 Agent 执行。

### 下载

```powershell
# 从 GitHub Releases 下载
# https://github.com/hermes-ai/hermes-cli/releases

# 添加到 PATH
$env:PATH += ";C:\path\to\hermes"
```

### 验证

```powershell
hermes --version
```

### 配置

```powershell
# 设置 API 密钥
$env:ANTHROPIC_API_KEY = "your-api-key"
```

---

## 运行应用

### 开发模式

```powershell
# 在项目根目录
npm run tauri dev
```

### 生产构建

```powershell
npm run tauri build
```

---

## 故障排除

### cargo check 失败

**错误**: `linking with link.exe failed`

**原因**: Windows SDK 未安装

**解决**:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

### 前端构建失败

**错误**: `Cannot find module`

**解决**:
```powershell
rm -rf node_modules
pnpm install
```

### 测试失败

**错误**: `Tauri internals not available`

**原因**: 测试在 Node.js 环境，没有 Tauri 运行时

**解决**: 这是正常的，测试使用 mock

---

## 测试项目

使用测试 Godot 项目验证功能：

```powershell
# 测试项目位置
D:\tmp\test-godot-project\

# 包含
# - project.godot
# - scripts/Player.gd
# - main.tscn
```

---

## 下一步

1. ✅ 安装所有依赖
2. ✅ 运行 `npm run build` 验证前端
3. ⏳ 安装 Windows SDK 后运行 `cargo check` 验证 Rust
4. ⏳ 安装 Hermes CLI 后测试真实 Agent 执行
5. ⏳ 使用测试项目验证完整闭环

---

**文档版本**: 1.0.0
**最后更新**: 2026-07-09