# 当前真实状态与验收边界

> 基线日期：2026-07-13
> 仓库：`D:/dingsun/acp-ui`

## 已验证通过的部分

本轮已经在 D 盘环境完成以下验证：

| 范围 | 结果 | 证据 |
|---|---:|---|
| Vue 类型检查 | 通过 | `node vue-tsc --noEmit`，退出码 0 |
| 前端测试 | 85 个文件、1283 个测试通过 | `node node_modules/vitest/vitest.mjs run --reporter=basic` |
| Vite 构建 | 547 个模块构建成功 | `node node_modules/vite/bin/vite.js build` |
| Game Operator 浏览器测试 | 13/13 通过 | `playwright.config.ts` + `tests/e2e/game-operator.spec.ts` |
| Rust 后端 | 315 个测试，313 通过，2 忽略，0 失败 | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1` |
| IDEA 插件 | `gradlew.bat build` 通过 | `clients/idea-game-operator/gradlew.bat` |
| VSCode 插件 | `npm run pretest` 通过 | `compile` + `lint` |
| 差异检查 | 通过 | `git diff --check`，只有换行格式警告 |

Rust 的 2 个 ignored 测试要求真实 Godot/Hermes 路径；它们不能被算作已完成的实机验证。

## 本轮已经修复的真实问题

- Rust 测试中的 `OperatorEvent` 初始化缺少结构字段，导致后端原本无法编译。
- 前端把后端线格式 `waiting_approval` 直接当成翻译 key，导致审批状态显示错误。
- 审批队列、执行计划、进度时间线复用了错误的页面标题 key。
- 项目路径输入错误地使用了任务目标占位符。
- 懒加载语言在应用首次挂载前没有真正装载，非默认语言可能先显示英文。
- `pt-BR` 和 `zh-TW` 的 Game Operator 区块仍是英文或简体中文。
- VSCode/IDEA 使用了错误的默认端口和旧 API 路径；审批请求缺少 `task_id`。
- IDEA 工具窗口原来只有假连接和 `TODO` 按钮；现在已经接入 API 客户端。
- IDEA 工程缺少 Wrapper，且旧 IntelliJ Gradle 插件与现有 Gradle 不兼容；现在固定到可用插件版本并生成 Wrapper。

## 尚未被真实证明的部分

### 1. 真实远程闭环

当前浏览器 E2E 主要验证页面导航、表单、占位符和按钮状态；前端单元测试中的 API 大量使用 mock。Rust 有 HTTP/状态相关测试，但还没有一条“启动真实 Tauri/远程服务 -> 创建真实 Godot 任务 -> Agent 执行 -> 人工审批 -> 应用补丁 -> Godot 验证 -> 回传结果”的完整证据链。

### 2. 真实 Godot/Hermes 执行

没有绑定到固定的 D 盘 Godot 测试项目、Hermes CLI 和可重复的执行环境之前，不能声称游戏代码被真实修改并运行成功。

### 3. 桌面运行时验收

IDEA 目前已经能编译插件，VSCode 已经能编译扩展，但还需要在真实 IDE 进程中安装/加载并点击完整操作；Tauri 桌面窗口也需要独立的 driver 验收。

### 4. 仓库卫生

当前 HEAD 中 `.artifacts/` 约有 2300 个已跟踪文件，内容包含浏览器、驱动、缓存、日志和构建产物。它们不应进入产品源码提交。清理前必须先做清单、确认没有需要保留的验收证据，再单独提交。

### 5. 历史文档过期

`docs/PROJECT-FANTASTIC-FINAL.md` 仍然写着 1278 个测试和 100% 完成，但当前复测数字已经是 1283，且上面列出的远程实机边界尚未完成。它不能作为当前验收依据。

## 工作区异常

以下项目状态不是本轮业务修复的一部分，执行 Agent 不得擅自处理：

- `.claude/scheduled_tasks.lock` 当前是删除状态。
- 根目录有异常命名的未跟踪文件 `Ddingsunacp-uidocsFAQ.md`。

每次提交前都要明确说明这两个项目是否被排除在提交之外。

## 当前验收结论

当前结论应写成：

> 核心前端、Rust 后端、VSCode/IDEA 构建和 Game Operator 页面测试已通过；真实 Godot 执行、Tauri/IDE 实机操作、远程安全闭环和仓库清理仍属于下一轮交付内容。

禁止写成：

> 所有平台、所有远程操作和游戏开发流程已经 100% 完成。
