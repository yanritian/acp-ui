---
name: WP-00 执行基线建立
description: 2026-07-13 WP-00 基线测试结果
type: project
---

## 1. 运行摘要

```text
日期：2026-07-13
执行者：Claude Code + Qwen
仓库：D:/dingsun/acp-ui
分支：cleanup/project-snapshot-2026-06-25
起始 HEAD：d30df2a
结束 HEAD：(待提交)
工作包：WP-00
目标：建立执行基线，验证所有构建和测试通过
是否修改代码：yes (.gitignore)
是否修改文档：yes (执行台账)
是否产生提交：pending
```

## 2. 环境证据

| 工具 | 路径 | 版本 | 结果 |
|---|---|---|---|
| Node | D:/dingsun/acp-ui/node_modules/.bin/ | 18.x | 通过 |
| Vue 类型检查 | vue-tsc | --noEmit | 退出码 0，无错误 |
| Vitest | vitest v3.2.6 | --reporter=basic | 85 文件/1283 测试通过 |
| Vite 构建 | vite v6.4.3 | build | 547 模块构建成功 |
| Rust/Cargo | cargo | --lib --test-threads=1 | 313 passed; 2 ignored |
| VSCode pretest | clients/vscode-game-operator | npm run pretest | 退出码 0 |
| IDEA build | clients/idea-game-operator | gradlew.bat build | BUILD SUCCESSFUL |

## 3. 修改清单

| 文件 | 行为变化 | 风险 | 回滚方式 |
|---|---|---:|---|
| .gitignore | 添加 `.artifacts/` 到排除清单 | low | git checkout |
| clients/vscode-game-operator/node_modules/ | 安装依赖 (npm install) | low | rm -rf node_modules |

## 4. 命令证据

### Vue 类型检查
```text
命令：node node_modules/vue-tsc/bin/vue-tsc.js --noEmit
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：(无输出，表示无错误)
```

### 前端测试
```text
命令：node node_modules/vitest/vitest.mjs run --reporter=basic
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：Test Files 85 passed (85), Tests 1283 passed (1283)
```

### Vite 构建
```text
命令：node node_modules/vite/bin/vite.js build
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：547 modules transformed, ✓ built in 10.25s
```

### Rust lib 测试
```text
命令：cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：test result: ok. 313 passed; 0 failed; 2 ignored
```

### VSCode pretest
```text
命令：npm run pretest
工作目录：D:/dingsun/acp-ui/clients/vscode-game-operator
退出码：0
关键输出：compile + lint 通过
```

### IDEA build
```text
命令：JAVA_HOME="D:/Android/Android Studio/jbr" ./gradlew.bat build --no-daemon
工作目录：D:/dingsun/acp-ui/clients/idea-game-operator
退出码：0
关键输出：BUILD SUCCESSFUL in 1m 4s, 16 actionable tasks: 16 executed
```

### 差异检查
```text
命令：git diff --check
工作目录：D:/dingsun/acp-ui
退出码：0
关键输出：(无输出，表示无问题)
```

## 5. 风险与未完成项

- 已知失败：无
- 被忽略测试及原因：
  - `operator::godot_validation::tests::validates_external_real_godot_project` — requires ACP_REAL_GODOT_BIN and ACP_REAL_GODOT_PROJECT on D:
  - `operator::structured_patch::tests::validates_and_applies_external_real_hermes_artifact` — requires ACP_REAL_HERMES_PATCH_ARTIFACT, ACP_REAL_HERMES_PROJECT, and ACP_REAL_HERMES_BACKUP_ROOT
- 未验证的平台：真实 Tauri 桌面运行、真实 Godot 执行、真实 Hermes 执行
- 依赖外部环境：Godot headless、Hermes CLI 需要固定 D 盘路径
- 不应被自动修复的用户文件：
  - `.claude/scheduled_tasks.lock` (删除状态)
  - `D:dingsunacp-uidocsFAQ.md` (未跟踪异常文件)
- 下一条最小可执行动作：创建 WP-00 提交，然后开始 WP-01

## 6. 提交记录

(待创建)

```text
commit：(pending)
message：chore(wp-00): establish execution baseline and add .artifacts to gitignore
included files：.gitignore, docs/codex/second/execution-ledger-WP00.md
excluded user files：.claude/scheduled_tasks.lock, D:dingsunacp-uidocsFAQ.md
verification after commit：git status --short, git diff --check
```

## 7. 基线结论

> **核心前端、Rust 后端、VSCode/IDEA 构建和所有测试已通过；真实 Godot 执行、Tauri/IDE 实机操作、远程安全闭环和仓库清理仍属于下一轮交付内容。**