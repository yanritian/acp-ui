# Qwen3.7 Plus + Claude Code 执行交接说明

> 日期：2026-07-08  
> 作者：Codex  
> 状态：待执行  
> 目标读者：Qwen3.7 Plus、Claude Code、项目负责人  

## 1. 执行总目标

先做 Hermes Game Operator 的 Godot MVP。

不要扩展运营、企业、传统开发、视频、漫画，也不要同时做 Unity/Ren'Py/Unreal。那些是后续 Domain Pack。

当前唯一目标：

```text
Godot 项目 -> 用户需求 -> Agent 分析 -> 计划 -> 用户审批 -> 修改文件 -> 展示进度/diff/记忆 -> 可暂停/继续/停止 -> 总结
```

## 2. 分工建议

### Qwen3.7 Plus 负责

- 产品拆解。
- 交互流程设计。
- Godot Skill 文档。
- Hook 规则设计。
- 测试用例和验收清单。
- 每阶段复盘和偏航检查。

### Claude Code 负责

- 修复仓库构建和测试基线。
- 修改 Vue/Tauri/Rust 代码。
- 接入 Hermes/Agent 执行链。
- 实现 Operator 事件流。
- 实现暂停、继续、停止。
- 实现 VSCode 插件连接。
- 编写和运行测试。

### 项目负责人负责

- 确认产品方向。
- 提供真实 Godot 测试项目。
- 决定是否批准危险操作。
- 判断 MVP 演示是否可信。

## 3. 当前仓库已知状态

### 3.1 清理方向是对的

当前分支已删除旧的假 AI 游戏设计/开发链路，包括：

- `GameDesigner.vue`
- `GameDeveloper.vue`
- `game_designer_agent.rs`
- `game_developer_agent.rs`
- 对应路由和命令注册

这条方向必须保留。

### 3.2 仍需修复的问题

必须先解决：

- 前端 build 失败。
- 旧 `test/game.test.js` 还引用 `/games/designer`。
- Vitest 与 Tauri/WebdriverIO 测试混在一起。
- `executive_agent.rs` 使用空 `ToolRegistry`。
- `Cargo.lock` 被 `.gitignore` 忽略。
- 同时存在 `package-lock.json` 和未跟踪 `pnpm-lock.yaml`。
- demo/mock 页面太多，容易误导主线。

## 4. 执行顺序

### Step 0：禁止继续扩功能

在基线修复前，不允许做：

- Unity 支持。
- Ren'Py 支持。
- IDEA 插件。
- 移动端遥控器。
- 多 Agent 编排。
- 资产生成。
- 企业/运营/视频/漫画功能。

### Step 1：修构建基线

目标：

```text
npm run build 通过
```

已知失败点：

```text
src/features/workflow/collaboration/CollaborationNetworkFlow.vue
Type instantiation is excessively deep and possibly infinite
```

建议修法：

- 避免 Vue Flow 复杂泛型直接参与模板推导。
- 用更窄的本地类型封装 `flowNodes` 和 `flowEdges`。
- 对 `nodeTypes`、`edgeTypes`、`defaultEdgeOptions` 做显式类型或局部 `as Record<string, unknown>` 隔离。
- 不要大改 Collaboration 功能，只做让 build 通过的最小修复。

验收：

```bash
npm.cmd run build
```

### Step 2：修测试分层

目标：

```text
Vitest 只跑单元测试
WebdriverIO/Tauri 测试单独跑
Playwright 测试单独跑
```

必须处理：

- 删除或迁移 `test/game.test.js`，它测试已经删除的 `/games/designer`。
- 不要让 WebdriverIO 测试被 Vitest 收集。
- 对直接使用 `@tauri-apps/api/core` 的 API 测试补 mock，或改用 `invokeOrProxy`。

建议：

```text
test/**/*.test.js -> WebdriverIO 专用，不进入 Vitest
src/**/*.test.ts -> Vitest
tests/e2e/**/*.spec.ts -> Playwright
```

验收：

```bash
npm.cmd run test -- --run --root D:/dingsun/acp-ui
```

不要求第一天所有业务测试全绿，但必须做到失败原因清晰，不再混入错误测试层。

### Step 3：锁文件和仓库卫生

必须决定：

- 如果继续用 npm：保留 `package-lock.json`，删除/忽略 `pnpm-lock.yaml`。
- 如果改用 pnpm：提交 `pnpm-lock.yaml`，移除 `package-lock.json`。

当前建议：

```text
先用 npm，保留 package-lock.json。
```

必须修：

- `.gitignore` 不应忽略 `Cargo.lock`。
- Tauri 应用应提交 `src-tauri/Cargo.lock`。
- `bin/` 中的 `.exe`、`.vsix` 不应混在源码主线，后续改为 release artifact。

### Step 4：提交清理基线

把“删除旧假 AI 游戏功能”的改动单独提交。

提交范围：

- 删除旧前端页面。
- 删除旧 Rust agent/command。
- 删除旧路由/feature registry。
- 归档旧报告。
- 删除或迁移旧测试。

不要把新的 Godot Operator 功能混进这个提交。

### Step 5：Godot Operator 后端最小闭环

新增或改造 Tauri 命令：

```text
hermes_game_start_task
hermes_game_pause_task
hermes_game_resume_task
hermes_game_stop_task
hermes_game_get_task
hermes_game_approve_action
hermes_game_reject_action
```

第一版可以复用 `ExecutiveAgentManager`，但需要补足：

- 任务状态。
- 任务事件。
- 暂停/继续/停止控制。
- 审批请求。
- 文件改动记录。

关键点：

- `execute_with_hermes_native` 不能继续使用空 `ToolRegistry`。
- 必须注册必要工具，至少 file/search/terminal/skill。
- 如果短期接不通完整 Hermes tools，就实现最小工具桥，但必须标明真实能力范围。

### Step 6：Godot 项目分析器

实现最小 Godot 项目分析：

输入：

```text
projectPath
```

输出：

```json
{
  "engine": "godot",
  "projectFile": "project.godot",
  "projectName": "...",
  "godotVersion": "...",
  "scripts": [],
  "scenes": [],
  "assets": [],
  "entryScene": "..."
}
```

分析范围：

- `project.godot`
- `.tscn`
- `.gd`
- `assets/`

不要第一版做完整语义索引。

### Step 7：Operator 前端面板

新增或改造 Game Operator 页面。

建议组件：

```text
src/features/games/operator/GameOperatorView.vue
src/features/games/operator/OperatorControlBar.vue
src/features/games/operator/ProgressTimeline.vue
src/features/games/operator/PlanPanel.vue
src/features/games/operator/MemoryPanel.vue
src/features/games/operator/ChangePanel.vue
src/features/games/operator/ApprovalPanel.vue
```

第一版可简化，但必须有：

- 当前状态。
- 当前计划。
- 进度事件。
- 暂停/继续/停止。
- 审批卡片。
- 文件变更摘要。
- 最终总结。

### Step 8：Godot Skill Pack

新增 Skill 文档，不要硬编码在 Rust 字符串里。

建议路径：

```text
skills/godot/godot-analyze/SKILL.md
skills/godot/godot-codegen/SKILL.md
skills/godot/godot-debug/SKILL.md
skills/godot/godot-improve/SKILL.md
```

如果项目最终有固定 Skill 根目录，以实际 Skill loader 为准。

每个 Skill 必须包含：

- name
- description
- when to use
- inputs
- available tools
- output format
- guardrails
- examples
- acceptance criteria

### Step 9：Hook 初版

实现最小 Hook：

```text
before_write_file
after_write_file
before_command
after_command
before_finish
```

规则：

- 写文件前必须记录原内容或 diff。
- 删除文件必须审批。
- 覆盖文件必须审批。
- 运行构建/测试命令需要可见进度。
- 任务结束前必须列出所有文件改动。

### Step 10：真实演示

准备一个真实 Godot 项目。

演示任务：

```text
给 Player 增加二段跳能力，不要修改输入映射。先展示计划，等我批准后再改文件。
```

必须录制或保存：

- 初始项目状态。
- Agent 计划。
- 审批事件。
- 文件 diff。
- 最终结果。
- 运行/验证说明。

## 5. 关键技术注意事项

### 5.1 不要让 VSCode 插件变成孤岛

VSCode 插件应该连接 Operator Core，而不是自己实现一套 Agent。

插件职责：

- 展示 UI。
- 发命令。
- 接收事件。
- 展示 diff。
- 传当前文件上下文。

核心能力必须在本地服务/桌面后端里。

### 5.2 IDEA 插件只做协议预留

MVP 不做 IDEA 插件实现。

只需设计：

- 本地 WebSocket 协议。
- task event 格式。
- approval event 格式。
- file diff event 格式。

### 5.3 暂停/继续/停止必须是真控制

不要只做 UI 按钮。

后端必须有任务状态机：

```text
Running -> Paused -> Running
Running -> Cancelling -> Cancelled
Running -> WaitingForApproval -> Running
Running -> Failed
Running -> Completed
```

如果底层 AgentLoop 暂时不能硬中断，也要实现：

- cooperative cancellation flag
- tool call 前检查取消状态
- 每阶段开始前检查暂停状态
- 长命令运行时记录“不可立即中断”的限制

### 5.4 记忆必须可解释

MemoryPanel 不显示一堆原始日志。

只显示：

- 被本次任务使用的记忆。
- 新增的候选记忆。
- 用户固定的项目记忆。

新增记忆最好先进入候选区，用户可确认。

### 5.5 安全边界必须先设计

最低要求：

- 默认 workspace 限定为当前游戏项目。
- 写 workspace 外文件必须审批。
- 删除文件必须审批。
- 运行 shell 命令必须显示命令内容。
- Git 操作必须显示影响。
- 不允许静默执行危险命令。

## 6. 验收清单

### 构建验收

- [ ] `npm.cmd run build` 通过
- [ ] `npm.cmd run test -- --run --root D:/dingsun/acp-ui` 分层清晰
- [ ] Rust 工具链恢复后 `cargo check` 通过或有明确错误清单

### 清理验收

- [ ] 旧假 AI 游戏页面删除
- [ ] 旧假 AI Rust agent 删除
- [ ] 旧命令注册删除
- [ ] 旧 `/games/designer` 测试删除或迁移
- [ ] 旧报告归档

### Godot MVP 验收

- [ ] 能选择 Godot 项目
- [ ] 能识别项目
- [ ] 能启动 Operator 任务
- [ ] 能生成计划
- [ ] 能显示进度
- [ ] 能暂停
- [ ] 能继续
- [ ] 能停止
- [ ] 能审批写文件
- [ ] 能展示 diff
- [ ] 能输出总结

### 产品验收

- [ ] 用户能看懂 Agent 当前在做什么
- [ ] 用户能知道 Agent 为什么这么做
- [ ] 用户能随时接管方向
- [ ] 用户能确认哪些文件被改了
- [ ] 用户不会误以为 mock/demo 是真实功能

## 7. 禁止事项

执行过程中禁止：

- 禁止新增假 AI 模板。
- 禁止跳过构建修复直接加功能。
- 禁止同时扩 Unity/Ren'Py/Unreal。
- 禁止把 VSCode/IDEA 插件写成各自独立 Agent。
- 禁止把全路径写权限暴露给远程控制而无审批。
- 禁止在主导航展示未完成 demo。
- 禁止只有文档没有可运行产物。

## 8. 推荐提交拆分

### Commit 1：清理旧假 AI 游戏功能

内容：

- 删除旧页面、Rust agent、命令、路由。
- 删除旧测试或移出 Vitest。
- 归档旧报告。

### Commit 2：修复构建和测试基线

内容：

- 修 Vue Flow 类型问题。
- 修测试分层。
- 修 lock 文件策略。
- 提交 Cargo.lock。

### Commit 3：新增 Godot Operator 基础 UI

内容：

- Operator 页面。
- 状态栏。
- 控制按钮。
- 进度列表。

### Commit 4：新增后端任务状态机和事件流

内容：

- start/pause/resume/stop 命令。
- task event。
- approval event。

### Commit 5：接入 Godot 项目分析和最小 Agent 执行

内容：

- Godot project analyzer。
- Hermes execution bridge。
- 文件 diff。

### Commit 6：新增 Godot Skill + Hook 初版

内容：

- Skill 文档。
- Hook 规则。
- 审批策略。

## 9. 给执行模型的第一条提示词

建议直接发给 Qwen3.7 Plus：

```text
请基于 docs/codex/2026-07-08-hermes-game-operator-master-plan.md 和 docs/codex/2026-07-08-qwen-claude-execution-brief.md，先输出 Godot MVP 的详细任务拆分。

要求：
1. 不扩 Unity/Ren'Py/Unreal。
2. 不做运营/企业/视频/漫画。
3. 先修 build/test 基线。
4. 每个任务必须有输入、输出、涉及文件、验收命令。
5. 明确哪些任务给 Claude Code 执行，哪些任务由你继续做产品/测试拆解。
```

建议直接发给 Claude Code：

```text
请先执行 docs/codex/2026-07-08-qwen-claude-execution-brief.md 的 Step 1 到 Step 4。

目标：
1. 修复 npm build。
2. 清理旧 /games/designer 测试。
3. 修测试分层。
4. 固定 lock 文件策略。
5. 提交“删除旧假 AI 游戏功能”的干净基线。

不要开始做 Unity、Ren'Py、IDEA 插件或视频/漫画/企业功能。
```

## 10. 成功标志

第一阶段成功不是“页面很多”，而是：

一个真实 Godot 项目中，用户能放心地让 Agent 改一处玩法代码，并且全程知道它在做什么、为什么做、改了哪里、怎么停、怎么改方向。

这个闭环成立，Hermes Game Operator 才成立。

