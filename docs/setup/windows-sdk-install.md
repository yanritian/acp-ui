# Windows SDK 安装指南

## 问题：cargo check 失败

错误信息：
```
error: linking with `link.exe` failed: exit code: 1
note: in the Visual Studio installer, ensure the "C++ build tools" workload is selected
```

## 解决方案

### 方法 1：Visual Studio Installer（推荐）

1. 打开 **Visual Studio Installer**
2. 点击 **Modify**
3. 切换到 **Workloads** 标签
4. 勾选 **Desktop development with C++**
5. 在右侧确认勾选：
   - MSVC v143 - VS 2022 C++ x64/x86 build tools
   - Windows 10 SDK 或 Windows 11 SDK
6. 点击 **Modify** 安装

### 方法 2：命令行安装（管理员权限）

```powershell
# 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 或使用 Chocolatey
choco install visualstudio2022-workload-vctools
```

### 方法 3：单独安装 Windows SDK

下载地址：
- Windows 10 SDK: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
- Windows 11 SDK: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

## 验证安装

```bash
# 验证 Rust 编译
cd D:/dingsun/acp-ui/src-tauri
cargo check

# 验证 link.exe 存在
where link.exe
```

## 常见问题

### Q: link.exe 找不到？
A: 需要安装 MSVC build tools，不仅仅是 Windows SDK

### Q: 安装后还是失败？
A: 重启终端或重启电脑让环境变量生效

### Q: 能用 MinGW/GCC 替代吗？
A: 可以，但需要切换 Rust target：
```bash
rustup target add x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu
```

## 时间估计

- Visual Studio Installer 方式：约 5-10 分钟
- 需要下载约 2-3 GB

---

**安装完成后继续执行 Hermes Game Operator 闭环验证。**