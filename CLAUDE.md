# ACP-UI Project Rules

## 绝对规则：禁止 C 盘操作

**无论任何操作、无论什么情况，都不允许在 C 盘进行文件的更新、修改、创建、删除。**

- 所有文件读写必须限定在当前项目目录 (`D:\dingsun\acp-ui`) 及其子目录
- 不允许在 `C:\Users\`、`C:\Windows\`、`C:\Program Files\` 等 C 盘路径下创建或修改任何文件
- 不允许使用 C 盘作为临时文件目录
- 如果操作需要临时文件，使用 `D:\tmp` 或项目内的 `tmp/` 目录
- 如果工具默认行为会写入 C 盘（如缓存、配置），必须重定向到 D 盘

### C 盘缓存重定向配置

**必须配置以下环境变量，确保工具使用 D 盘缓存：**

```powershell
# 在 PowerShell 中执行（一次性配置）
[Environment]::SetEnvironmentVariable('npm_config_cache', 'D:\dingsun\acp-ui\.npm-cache', 'User')
[Environment]::SetEnvironmentVariable('TEMP', 'D:\dingsun\acp-ui\.tmp\temp', 'User')
[Environment]::SetEnvironmentVariable('TMP', 'D:\dingsun\acp-ui\.tmp\tmp', 'User')
[Environment]::SetEnvironmentVariable('CARGO_HOME', 'D:\Rust\.cargo', 'User')
[Environment]::SetEnvironmentVariable('RUSTUP_HOME', 'D:\Rust\.rustup', 'User')
[Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', 'D:\dingsun\acp-ui\src-tauri\target', 'User')
```

**在 Bash/PowerShell 会话中临时设置：**

```bash
export npm_config_cache="D:/dingsun/acp-ui/.npm-cache"
export TEMP="D:/dingsun/acp-ui/.tmp/temp"
export TMP="D:/dingsun/acp-ui/.tmp/tmp"
export CARGO_HOME="D:/Rust/.cargo"
export RUSTUP_HOME="D:/Rust/.rustup"
export CARGO_TARGET_DIR="D:/dingsun/acp-ui/src-tauri/target"
```

**违反此规则 = 立即停止并回退。**

## 开发工作区路径

| 用途 | 路径 |
|------|------|
| 项目根目录 | `D:\dingsun\acp-ui` |
| Flutter 项目 | `D:\dingsun\acp-ui\acp_ui_flutter` |
| Tauri 后端 | `D:\dingsun\acp-ui\src-tauri` |
| Vue 前端 | `D:\dingsun\acp-ui\src` |
| 临时文件 | `D:\tmp` |
| 项目文档 | `D:\dingsun\acp-ui\docs\` |

## 项目文档

完整的项目结构、文档索引、测试计划、实施计划见：
- `docs/project-completion-plan.md` — **总计划文档**
- `docs/system-architecture.md` — 系统架构
- `docs/implementation-plan-phased.md` — 分阶段实施计划
- `docs/superpowers/plans/2026-05-04-agent-teams-platform-completion.md` — Agent Teams 完成计划
