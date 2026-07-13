# 目标架构与运行时契约

## 1. 总体分层

```text
操作员客户端
  Web / Tauri / VSCode / IDEA / Remote API
             |
统一 Operator API + Event Stream + Error Codes
             |
Operator Control Plane
  Task Aggregate / State Machine / Approval / Patch / Audit
             |
Agent Runtime
  Planner / Executor / Memory / Checkpoint / Cancellation
             |
Capability Plane
  Skill Registry / MCP Registry / Hook Pipeline / Sandbox
             |
Game Domain Pack
  Godot Inspector / Godot Tools / Project Rules / Validators
             |
真实项目文件、Godot、Hermes CLI、测试运行器
```

控制面负责“任务是什么、现在处于什么状态、谁批准了什么”；执行面负责“Agent 如何完成动作”；能力面负责“Agent 可以调用什么”；领域包负责“游戏项目的知识和工具”。这些职责不能混在 Vue 组件、客户端按钮或单个超大 Rust 文件里。

## 2. 推荐目录边界

```text
src/
  api/                         # 前端 API 适配、DTO 和错误归一化
  features/game-operator/     # 游戏操作员页面和组件
  locales/                     # 13 个 locale，类型由 types.ts 约束
  stores/                      # UI 状态和服务端快照缓存
src-tauri/src/
  operator/                    # 任务、审批、事件、补丁、审计、恢复
  remote/                      # HTTP/SSE、认证、限流、远程配置
  runtime/                     # Agent 执行桥接、取消、检查点
  capabilities/               # Skill/MCP/Hook 运行时
  domains/godot/               # Godot 领域工具和验证器
clients/
  vscode-game-operator/       # VSCode 适配层，不复制业务状态机
  idea-game-operator/          # IDEA 适配层，不复制业务状态机
schemas/
  operator/                    # 版本化 DTO、错误码、事件 JSON Schema
tests/
  contract/                   # 跨客户端协议测试
  e2e/                         # 浏览器真实页面流程
  fixtures/godot/              # 固定 Godot 测试项目
docs/codex/second/             # 当前轮执行契约和证据模板
```

如果现有目录与这个结构不一致，执行 Agent 先做兼容迁移，再做删除；不能一次性大规模移动导致历史 import 和测试失效。

## 3. 任务聚合根

服务端的 `OperatorTask` 是唯一任务事实来源，最少包含：

```text
task_id
revision
domain
project_path
goal
constraints
mode
approval_policy
status
current_step_id
active_plan_revision
memory_snapshot_id
checkpoint_id
created_at / updated_at
created_by
```

所有写操作都带 `expected_revision`。版本不匹配时返回 `409 REVISION_CONFLICT`，客户端必须重新读取快照后让操作员确认，不允许静默覆盖。

## 4. 状态机

推荐状态：

```text
created -> inspecting -> planning -> waiting_approval
waiting_approval -> executing       [approve]
waiting_approval -> rejected        [reject]
executing -> paused                 [pause]
paused -> executing                 [resume]
executing -> waiting_approval       [new high-risk action]
executing -> validating
validating -> completed
validating -> failed
created/inspecting/planning/executing/paused -> cancelled [stop]
```

规则：

- 终态 `completed`、`failed`、`cancelled` 不接受普通控制命令。
- `pause` 是协作式取消：先发取消信号，再等待工具边界安全退出，最后写入 checkpoint。
- `stop` 不是 `pause` 的别名；它会关闭执行上下文，并且必须写入不可变审计事件。
- Agent 不能通过修改本地状态绕过服务端状态机。
- 每次状态变更产生带 `sequence`、`task_id`、`revision` 的事件。

## 5. 事件模型

事件必须包含：

```json
{
  "event_id": "evt_...",
  "task_id": "task_...",
  "sequence": 42,
  "task_revision": 8,
  "type": "task.status_changed",
  "level": "info",
  "title_key": "operatorEvents.statusChanged",
  "message_key": "operatorEvents.statusChangedMessage",
  "args": { "status": "paused" },
  "source": "operator-runtime",
  "created_at": "2026-07-13T00:00:00Z"
}
```

用户文案只能在客户端通过 `*_key + args` 翻译；服务端不能把某个语言的最终句子当作协议事实。事件必须支持按任务、序号和时间查询，并能从任意序号继续消费。

## 6. 统一 API

当前客户端已经使用的基础路径保持不变：

```text
GET  /api/health
GET  /api/operator/platforms
GET  /api/operator/tasks
POST /api/operator/tasks
GET  /api/operator/tasks/{task_id}
POST /api/operator/tasks/{task_id}/pause
POST /api/operator/tasks/{task_id}/resume
POST /api/operator/tasks/{task_id}/stop
GET  /api/operator/tasks/{task_id}/events
GET  /api/operator/tasks/{task_id}/approvals
POST /api/operator/approvals/decision
```

新增接口必须先更新 `schemas/operator` 和 Rust DTO，再更新 Web、VSCode、IDEA。审批请求至少包含 `task_id`、`approval_id`、`decision`、`expected_revision`。

远程事件第一版使用可恢复的 HTTP 事件流或带 `after_sequence` 的轮询；若采用 SSE，必须保留轮询 fallback。WebSocket 不是第一版的前置条件。

## 7. Agent 执行循环

```text
读取任务快照
  -> 读取项目规则和记忆
  -> 选择领域 Skill/MCP
  -> 生成结构化计划
  -> 记录计划事件
  -> 执行低风险只读工具
  -> 生成 diff / action proposal
  -> 根据风险等级等待审批
  -> 应用单个可回滚补丁
  -> 运行领域验证器
  -> 保存 checkpoint 和记忆摘要
  -> 汇报结果或进入下一步
```

每一个工具调用都要有 `call_id`、输入摘要、输出摘要、耗时、退出状态和关联任务。大输出存文件或对象存储，事件只存引用和摘要。

## 8. 记忆模型

分为四类：

1. **项目事实**：Godot 版本、目录结构、启动场景、脚本规则、测试命令。
2. **任务记忆**：当前目标、已经尝试过的方案、失败原因、用户决策。
3. **团队规则**：命名、代码风格、审批策略、禁止修改目录。
4. **临时上下文**：当前工具输出和短期推理摘要，任务结束后可压缩。

记忆必须带来源、时间、可信度、作用域和过期策略。模型输出不能直接写成永久项目事实，必须经过工具证据或操作员确认。

## 9. 检查点与恢复

checkpoint 至少保存：任务快照、状态机版本、计划版本、已应用补丁、未应用补丁、工具调用位置、记忆快照和最后事件序号。

恢复顺序：加载任务 -> 校验 revision -> 重放状态变更 -> 恢复未决审批 -> 恢复事件游标 -> 检查文件系统锁和补丁 hash -> 决定继续、暂停或要求人工确认。任何 hash 不一致都进入 `recovery_required`，不能自动覆盖文件。
