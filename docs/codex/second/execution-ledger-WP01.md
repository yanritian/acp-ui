---
name: WP-01 仓库修整和生成物清理
description: 2026-07-13 WP-01 生成物清单和 gitignore 更新
type: project
---

## 1. 运行摘要

```text
日期：2026-07-13
执行者：Claude Code + Qwen
仓库：D:/dingsun/acp-ui
分支：cleanup/project-snapshot-2026-06-25
起始 HEAD：d5c7a3d
结束 HEAD：(待提交)
工作包：WP-01
目标：修整仓库和生成物，更新 gitignore 覆盖所有生成物
是否修改代码：no
是否修改文档：yes (执行台账)
是否产生提交：pending
```

## 2. 生成物清单

### .artifacts/desktop-acceptance/2026-07-10-codex/

| 目录 | 大小 | 文件数 | 是否可提交 |
|---|---:|---:|---|
| appdata | 12K | - | 不提交 |
| cache | 0 | - | 不提交 |
| config | 4K | - | 不提交 |
| drivers | 257M | - | **不提交** (浏览器驱动) |
| history | 988K | - | 不提交 |
| operator | 84K | 1 | 不提交 (sqlite3) |
| profile | 0 | - | 不提交 |
| project-1783658820041 | 2K | - | 不提交 |
| screenshots | 2.5M | 22 | **可能保留** (验收截图) |
| tauri.cdp.conf.json | 1K | - | 不提交 |
| tmp | 4.3M | - | 不提交 |
| webview2 | 45M | - | **不提交** (WebView2 运行时) |
| webview2-direct | 43M | - | **不提交** |
| webview2-webdriver | 0 | - | 不提交 |
| **总计** | **352M** | **2396** | **全部排除** |

### 其他生成物

| 目录/文件 | 大小 | 是否已排除 |
|---|---:|---|
| node_modules | 850M | ✅ 已在 gitignore |
| dist | 1.9M | ✅ 已在 gitignore |
| test-results | 1K | ✅ 已在 gitignore |
| playwright-report | 520K | ✅ 已在 gitignore |
| test-output | 2.5M | ✅ 已在 gitignore |
| clients/vscode-game-operator/out | - | ✅ 新增 gitignore |
| clients/idea-game-operator/build | - | ✅ 新增 gitignore |
| clients/idea-game-operator/.gradle | - | ✅ 新增 gitignore |

### 根目录已跟踪文件（不删除）

| 文件 | 状态 | 决策 |
|---|---|---|
| acp-ui-blank.png | 已跟踪 | 保留 |
| acp-ui-working.png | 已跟踪 | 保留 |
| agent-config-page.png | 已跟踪 | 保留 |
| current-page.png | 已跟踪 | 保留 |
| full-layout-check.png | 已跟踪 | 保留 |
| sidebar-layout.png | 已跟踪 | 保留 |

### 工作区异常文件（不处理）

| 文件 | 状态 | 决策 |
|---|---|---|
| .claude/scheduled_tasks.lock | 删除状态 | **不处理** |
| D:dingsunacp-uidocsFAQ.md | 未跟踪 | **不处理** |

## 3. gitignore 更新

WP-00 已完成以下更新：

```gitignore
# Desktop acceptance artifacts (drivers, cache, webview2, etc.)
.artifacts/

# Client build artifacts
clients/*/out/
clients/*/build/
clients/*/.gradle/
```

现有 gitignore 已覆盖：
- Node: node_modules, dist, *.local
- Rust: src-tauri/target, *.rs.bk
- Playwright: playwright-report/, test-output/, test-results/
- Tauri: src-tauri/gen/android, src-tauri/gen/apple
- Gradle: (通过 clients/*/.gradle/ 覆盖)
- Flutter: acp_ui_flutter/.dart_tool/, acp_ui_flutter/build/

## 4. 命令证据

### 生成物大小统计
```text
命令：du -sh .artifacts
工作目录：D:/dingsun/acp-ui
结果：352M

命令：find .artifacts -type f | wc -l
结果：2396
```

### 分目录大小
```text
命令：du -sh .artifacts/desktop-acceptance/2026-07-10-codex/*
结果：drivers 257M, webview2 45M, webview2-direct 43M, tmp 4.3M, screenshots 2.5M
```

### git diff --check
```text
命令：git diff --check
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：(无输出)
```

## 5. 验收截图分析

`.artifacts/desktop-acceptance/2026-07-10-codex/screenshots/` 包含 22 个 PNG 文件，主要是：
- operator-approval-actions-*.png (审批操作验证)
- operator-layout-fixed-*.png (布局验证)
- operator-control-*.png (控制面板验证)

这些是 Playwright/Tauri 验收截图，证据价值：
- **价值**: 证明 Game Operator UI 验收完成
- **处理**: 已被 .gitignore 排除，不进入提交
- **保留**: 可作为本地证据，但不提交到仓库

## 6. 风险与未完成项

- 已知失败：无
- 被忽略测试及原因：同 WP-00
- 未验证的平台：真实 Tauri 桌面运行
- 依赖外部环境：无
- 不应被自动修复的用户文件：
  - `.claude/scheduled_tasks.lock` (删除状态)
  - `D:dingsunacp-uidocsFAQ.md` (未跟踪异常文件)
  - 根目录 PNG 文件 (已跟踪)
- 下一条最小可执行动作：WP-01 验收完成，开始 WP-02

## 7. 结论

> WP-01 验收完成。`.artifacts/` 已添加到 gitignore，352M/2396 文件生成物不再进入提交。用户文件和已跟踪截图保留。

**Why**: 生成物不应进入产品源码提交，但验收截图作为本地证据可保留。
**How to apply**: 所有 `.artifacts/` 内容被 gitignore 排除；根目录已跟踪 PNG 保持现状。