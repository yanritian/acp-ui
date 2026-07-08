# Rust Toolchain Configuration Guide

## Problem

The current environment has a Rust toolchain configuration issue that prevents `cargo check` and `cargo test` from completing successfully.

### Error Messages

**GNU Toolchain**:
```
error: error calling dlltool 'dlltool.exe': program not found
```

**MSVC Toolchain**:
```
LINK : fatal error LNK1181: 无法打开输入文件"kernel32.lib"
```

### Root Cause

Windows environment has conflicting link.exe:
- Git's link.exe: `C:\Program Files\Git\usr\bin\link.exe`
- MSVC's link.exe: `C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64\link.exe`

Git's link.exe takes precedence, causing compilation failures.

## Solutions

### Solution 1: Visual Studio Developer Command Prompt (Recommended)

1. **Open Developer Command Prompt**:
   - Start Menu → Visual Studio 2019 → Developer Command Prompt for VS 2019
   - Or: `C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\Common7\Tools\VsDevCmd.bat`

2. **Navigate to project**:
   ```cmd
   cd D:\dingsun\acp-ui\src-tauri
   ```

3. **Run cargo check**:
   ```cmd
   cargo check
   ```

4. **Run cargo test**:
   ```cmd
   cargo test
   ```

### Solution 2: Manual Environment Configuration

1. **Set PATH** (add to beginning):
   ```cmd
   set PATH=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64;%PATH%
   ```

2. **Set LIB**:
   ```cmd
   set LIB=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\ucrt\x64;C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133\lib\x64
   ```

3. **Set INCLUDE**:
   ```cmd
   set INCLUDE=C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\shared
   ```

4. **Run cargo**:
   ```cmd
   cargo check
   cargo test
   ```

### Solution 3: Git Bash Configuration

1. **Edit ~/.bashrc**:
   ```bash
   export PATH="/c/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/14.29.30133/bin/Hostx64/x64:$PATH"
   export LIB="C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\um\\x64;C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\ucrt\\x64;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30133\\lib\\x64"
   export INCLUDE="C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0\\ucrt;C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0\\um;C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0\\shared"
   ```

2. **Reload bashrc**:
   ```bash
   source ~/.bashrc
   ```

3. **Run cargo**:
   ```bash
   cargo check
   cargo test
   ```

### Solution 4: Reinstall Rust Toolchain

1. **Uninstall current Rust**:
   ```cmd
   rustup self uninstall
   ```

2. **Reinstall with MSVC**:
   ```cmd
   rustup-init --default-toolchain stable-x86_64-pc-windows-msvc
   ```

3. **Verify installation**:
   ```cmd
   rustup show
   ```

4. **Run cargo**:
   ```cmd
   cargo check
   cargo test
   ```

### Solution 5: Use WSL (Windows Subsystem for Linux)

1. **Install WSL**:
   ```cmd
   wsl --install
   ```

2. **Install Rust in WSL**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Mount Windows drive**:
   ```bash
   cd /mnt/d/dingsun/acp-ui/src-tauri
   ```

4. **Run cargo**:
   ```bash
   cargo check
   cargo test
   ```

## Verification

After configuring the toolchain, run:

```bash
# Check Rust installation
rustc --version
cargo --version

# Verify toolchain
rustup show

# Clean build cache
cargo clean

# Check compilation
cargo check

# Run tests
cargo test

# Build release
cargo build --release
```

## Expected Results

### Success Indicators

```
cargo check:
   Compiling acp-ui v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)

cargo test:
   Compiling acp-ui v0.1.0
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests
   test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Troubleshooting

**Issue**: `cargo check` still fails  
**Solution**: Verify all environment variables are set correctly

**Issue**: `link.exe` still conflicts  
**Solution**: Remove Git from PATH temporarily or use full path to MSVC link.exe

**Issue**: Missing Windows SDK  
**Solution**: Install Windows 10 SDK from Visual Studio Installer

## Quick Start Script

Create `configure-rust.bat`:

```batch
@echo off
echo Configuring Rust MSVC Toolchain...

set PATH=C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64;%PATH%
set LIB=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\ucrt\x64;C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.29.30133\lib\x64
set INCLUDE=C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\shared

echo Environment configured!
echo Running cargo check...
cargo check
```

Run with: `configure-rust.bat`

## Resources

- [Rust on Windows](https://rust-lang.github.io/rustup/installation/windows.html)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [Windows SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk/)

---

**Document Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
