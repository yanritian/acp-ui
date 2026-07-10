# 2026-07-09 游戏开发 Operator 运行闭环检查

> 状态更新（2026-07-10）：本文保留 7 月 9 日的阶段性检查记录。Hermes 0.4.0 无工具结构化提案、逐文件 diff 二次审批、哈希复核、备份、应用、回滚和真实在线模型烟测已经完成。最新事实、测试证据与后续执行提示词以 `docs/codex/2026-07-10-game-agent-closure-and-roadmap.md` 为准；本文中关于 `proposal.md`、Hermes 0.3.0、`269 passed` 和“尚未接入真实写入”的描述均已过期。

目标：把 Hermes Game Operator 从“状态演示”推进到“可创建任务、可审批、可后台执行、可暂停、可恢复、可停止、可改目标、可远程控制、可测试验证”的第一阶段真实闭环。

硬约束：

- 项目工作目录：`D:\dingsun\acp-ui`
- 工具链、缓存、临时目录、测试输出必须放在 D 盘。
- 不要把项目操作、临时文件、Node/npm 缓存、Cargo 缓存写到 C 盘。
- 如果 Node/npm 仍然解析到 C 盘，不要运行前端 npm build/test，先建立 D 盘 Node 工具链。

## 当前结论

Godot MVP 后端主链路已经比此前更接近真实 Operator：

```text
start task
  -> analyze Godot project
  -> generate plan
  -> wait approval
  -> approve returns immediately
  -> background Hermes Game runner executes
  -> operator can stop / pause / resume / redirect
  -> event stream records every operator intervention
```

远程操作入口也已经接入同一个 `OperatorState`。桌面 App 启动时默认只在回环地址启动 Remote Operator HTTP server：

```text
http://127.0.0.1:1422
```

本机模式不强制 Token，但会限制浏览器 Origin。远程监听必须显式配置 bind、Token 和项目根目录，否则服务拒绝启动：

```powershell
$env:ACP_OPERATOR_HTTP_BIND='0.0.0.0'
$env:ACP_OPERATOR_HTTP_PORT='1422'
$env:ACP_OPERATOR_HTTP_TOKEN='<high-entropy-token>'
$env:ACP_OPERATOR_ALLOWED_PROJECT_ROOTS='D:\games\project-a;D:\games\project-b'
$env:ACP_OPERATOR_ALLOWED_DOMAINS='game.godot'
$env:ACP_OPERATOR_ALLOWED_CLIENTS='vscode-main,idea-main'
$env:ACP_OPERATOR_ALLOWED_ORIGINS='http://192.168.1.20:5173'
```

仍可用 `ACP_OPERATOR_HTTP_DISABLED=1` 完全关闭服务。当前安全层适合本机和受控局域网；公开到公网前仍需 TLS/反向代理、持久化身份/RBAC、持久化审计和速率限制。

当前 Hermes Game 生成的是 reviewable proposal artifact，不会直接覆盖项目源码。默认 artifact 路径：

```text
D:\dingsun\acp-ui\.operator\hermes-runs\<task_id>\proposal.md
```

## 本轮完成的关键能力

### 1. D 盘 Hermes Game 执行器发现

后端只从 D 盘可信来源发现 Hermes 执行器：

- `HERMES_GAME_CLI_PATH`
- `HERMES_CLI_PATH`
- `D:\dingsun\acp-ui\bin\hermes-game.exe`
- `D:\dev-tools\hermes-game\target\release\hermes-game.exe`
- `D:\tmp\hermes-official\hermes.exe`

这避免误用 C 盘 PATH 上的工具，也符合当前 D-only 约束。

已经在 D 盘构建真实 Hermes Game Rust CLI：

```powershell
cd D:\dev-tools\hermes-game
$env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
$env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Rust\.cargo'
$env:RUSTUP_HOME='D:\Rust\.rustup'
& 'D:\Rust\.cargo\bin\cargo.exe' build --release
```

确认结果：

```text
D:\dev-tools\hermes-game\target\release\hermes-game.exe
hermes-game 0.3.0
```

真实 CLI 参数来自 `D:\dev-tools\hermes-game\src\main.rs`：

```text
hermes-game --engine godot codegen "<description>" --output <artifact>
```

当前 `HermesGameBridge` 的调用顺序已经与真实 CLI 对齐。

真实 CLI smoke 结果：

- `hermes-game.exe --version` 通过，输出 `hermes-game 0.3.0`。
- `hermes-game.exe --help` 和 `codegen --help` 通过。
- `codegen hello_player --output D:\dingsun\acp-ui\.tmp-tests\hermes-real-smoke\proposal.md` 能启动、加载配置、创建 LLM provider、注册工具并进入 `AgentLoop`。
- 该 smoke 在 30 秒内未完成，已主动 kill 子进程。后续必须把真实 codegen 做成流式事件和明确超时策略，否则 UI 会只看到“执行中”。

### 2. 审批后改为后台执行

`operator_approve` 不再持有全局 state mutex 等待 Hermes Game 跑完。它现在只完成审批状态更新，然后启动后台 runner 并立即返回。

这对前端面板很重要：审批按钮不会卡住，进度应该通过 `operator_list_events` 和 `operator_get_task` 轮询或订阅读取。

### 3. Stop 是真实取消

后台 Hermes Game bridge 使用 child process loop，每 100ms 检查任务状态：

- `Cancelling`
- `Cancelled`
- `Paused`
- goal changed

一旦满足条件，就 kill 当前 Hermes Game 子进程并等待退出。`operator_stop_task` 会把任务稳定落到 `Cancelled`，并写入 `TaskCancelling` / `TaskCancelled` 事件。

Hermes Game 子进程现在有默认 15 分钟运行超时，可用 `HERMES_GAME_TIMEOUT_SECONDS` 覆盖。超时后后端会 kill 子进程并返回 timeout，避免真实模型调用无限挂住。

### 4. Pause / Resume 的产品语义

当前没有实现 OS 级“冻结子进程”。本轮采用更可靠的语义：

- Pause：停止当前 Hermes Game 子进程，任务保留为 `Paused`。
- Resume：按当前 goal 重新启动后台执行。

这比假装能冻结执行更诚实，也更容易在之后接入真实 Hermes Rust runtime。

### 5. Redirect 可以运行中改方向

`operator_redirect_task` 现在会：

- 允许从 `Planning`、`WaitingApproval`、`Running`、`Paused` 等活跃态发起。
- 标记旧 pending approval 为 `request_changes`。
- 更新 task goal。
- 写入 `TaskRedirected` 和新的 `PlanStarted` 事件。
- 对 Godot 任务立即重新分析并生成新 approval。
- 让旧后台 Hermes Game 子进程因 goal changed 被取消，旧结果不会再把任务误标为 completed。

这就是“操作员随时做出改动”的最小可信闭环。

### 6. Remote Operator HTTP API

新增 HTTP 端点都共享桌面端同一个 `OperatorState`，不是 mock：

```text
GET  /api/operator/platforms
GET  /api/operator/tasks
POST /api/operator/tasks
GET  /api/operator/tasks/{task_id}
GET  /api/operator/tasks/{task_id}/events?limit=100
GET  /api/operator/tasks/{task_id}/approvals
GET  /api/operator/tasks/{task_id}/summary
GET  /api/operator/audit?limit=100
POST /api/operator/tasks/{task_id}/pause
POST /api/operator/tasks/{task_id}/resume
POST /api/operator/tasks/{task_id}/stop
POST /api/operator/tasks/{task_id}/redirect
POST /api/operator/approvals/decision
```

`/api/operator/platforms` 当前声明的平台能力：

- `tauri-desktop`
- `web-remote`
- `vscode-extension`
- `idea-plugin`
- `mobile-operator`
- `godot-domain-pack`

这不代表所有端 UI 都已经完成，而是协议已经给这些端留下同一条控制链路。

### 7. 前端远程 SDK

新增 `D:\dingsun\acp-ui\src\api\operatorRemoteApi.ts`，用于 Web、VSCode、IDEA、移动端或其他远程面板调用同一套 HTTP 协议。

默认连接地址：

```ts
http://127.0.0.1:1422
```

核心导出：

```ts
import {
  OperatorRemoteApi,
  createOperatorRemoteApi,
  buildOperatorEventsWebSocketUrl,
} from '@/api'
```

默认客户端：

```ts
await OperatorRemoteApi.getPlatforms()
await OperatorRemoteApi.startTask({
  domain: 'game.godot',
  project_path: 'D:\\your\\godot-project',
  goal: '给玩家角色增加二段跳',
})
```

自定义远程地址：

```ts
const remote = createOperatorRemoteApi({
  baseUrl: 'http://192.168.1.20:1422',
  token: '<token>',
  clientId: 'vscode-main',
})
```

SDK 覆盖：

- `getPlatforms()`
- `startTask(request)`
- `listTasks()`
- `getTask(taskId)`
- `pauseTask(taskId)`
- `resumeTask(taskId)`
- `stopTask(taskId)`
- `redirectTask(taskId, request)`
- `approve(request)`
- `getPendingApprovals(taskId)`
- `listEvents(taskId, limit)`
- `getTaskSummary(taskId)`
- `getAudit(limit)`
- `eventsWebSocketUrl`

共享类型新增在 `D:\dingsun\acp-ui\src\types\operator.ts`：

- `RemotePlatformCapability`
- `RemoteRedirectRequest`
- `RemoteCommandResponse`
- `RemoteAuditRecord`

`eventsWebSocketUrl` 永远不会拼接 Token。当前浏览器远程端应使用 HTTP event polling；能够设置握手 Header 的 IDE/原生客户端可在 WebSocket 握手中携带 Bearer Token。后续若要支持浏览器原生 WebSocket，需要单独设计首帧认证协议，不能退回 query-string Token。

### 8. Game Operator 面板 P0 闭环

`D:\dingsun\acp-ui\src\features\game-operator\views\GameOperatorView.vue` 已补上第一阶段操作员闭环：

- start task 后立即刷新 task / events / approvals 快照。
- approval 后不等待后台执行完成，继续轮询 task / events / approvals。
- stop 后不提前停止轮询，继续等待后端落到 `cancelled` 或其他终态。
- resume 后重新开启轮询。
- 新增 Redirect Goal 控件，调用 `operator_redirect_task`，并清空旧 pending approval UI。
- 新增 Remote Operator 状态条，读取 `OperatorRemoteApi.getPlatforms()`，展示远程 server 是否可达和平台能力数量。

`D:\dingsun\acp-ui\src\features\game-operator\components\OperatorControlBar.vue` 已修正：

- Stop 在 `planning`、`waiting_approval`、`running`、`paused`、`redirecting` 都可见。
- 清理旧编码损坏的按钮字符。

`D:\dingsun\acp-ui\src\features\game-operator\components\ApprovalDrawer.vue` 已修正：

- 按后端 `approval.options` 渲染审批动作。
- 支持 `request_changes`。
- 兼容旧 mock 数据缺失 `options` 时默认显示 approve / reject。

`D:\dingsun\acp-ui\src\features\game-operator\components\PlanPanel.vue` 已修正：

- 清理旧编码损坏的步骤字符。
- 用稳定文本标签展示 Done / Now / Next / Failed。
- 补充 paused / cancelled 状态展示。

### 9. Remote Operator 第一阶段安全层

远程安全策略位于 HTTP 边界，不复制 Operator core：

- `ACP_OPERATOR_HTTP_BIND` 默认 `127.0.0.1`；只接受明确 IP 地址。
- 非回环 bind 强制要求 `ACP_OPERATOR_HTTP_TOKEN` 和 `ACP_OPERATOR_ALLOWED_PROJECT_ROOTS`。
- 非回环 Token 至少 32 字节；进程内只保存 SHA-256 固定长度摘要，并用 `subtle` 恒定时间比较，Debug 只显示 `[REDACTED]`。
- Origin、Client、Domain 和 canonical project root 均可配置 allowlist。
- Client ID 只接受 ASCII 字母数字与 `-_.:`，其余值在审计中统一降级为 `unidentified`。
- 新任务在进入 `OperatorState` 前会把项目路径改存 canonical 路径，避免后续继续使用原始 junction/symlink 别名。
- 列表、详情、events、approvals、summary、pause/resume/stop/redirect/approve 都会重新校验既有任务 scope；越界任务不会出现在列表，也不能被远程控制。
- CORS 不再允许任意 Origin，只放行 GET/POST/OPTIONS 和必要 Header。
- 合法 `OPTIONS` 预检不要求 Bearer，但仍受 Origin allowlist 约束。
- 未授权返回统一 `401` 和 `WWW-Authenticate`；范围越界返回统一 `403`，都不会进入 `OperatorState`。合法 Origin 下的拒绝响应包含可读 CORS Header。
- start/pause/resume/stop/redirect/approve 以及其他 HTTP 写请求写入有界审计队列。
- 审计只保存 request id、客户端、method、path、action、outcome、status 和拒绝代码，不保存 Header、Token、Goal、comment 或请求正文。
- 审计接口本身受相同认证保护，队列最多保留 1000 条。

当前审计是进程内有界队列，重启后会丢失。它用于第一阶段可观察和脱敏验证，不等于企业级持久化审计。

## 已验证测试

所有 Rust 测试都使用 D 盘工具链和 D 盘临时目录。

环境变量：

```powershell
$env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
$env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Rust\.cargo'
$env:RUSTUP_HOME='D:\Rust\.rustup'
```

运行 Operator 命令测试：

```powershell
& 'D:\Rust\.cargo\bin\cargo.exe' test operator::commands::tests --lib
```

结果：

```text
running 9 tests
9 passed; 0 failed
```

运行 Remote HTTP server 测试：

```powershell
& 'D:\Rust\.cargo\bin\cargo.exe' test http_server::tests --lib
```

结果：

```text
running 16 tests
16 passed; 0 failed
```

新增 HTTP 级回环测试覆盖：

```text
GET  /api/operator/platforms
POST /api/operator/tasks
GET  /api/operator/tasks/{task_id}
401  missing/invalid Bearer Token
403  Origin/client/domain/project root denied
GET  /api/operator/audit
CORS preflight allow/deny
audit action classification and secret redaction
```

运行 Remote Operator 安全策略测试：

```powershell
& 'D:\Rust\.cargo\bin\cargo.exe' test operator::security::tests --lib
```

结果：`6 passed; 0 failed`。

运行完整 Rust lib 测试：

```powershell
& 'D:\Rust\.cargo\bin\cargo.exe' test --lib
```

最新结果：

```text
269 passed; 0 failed
```

覆盖点：

- Godot task start -> planning -> approval
- local fallback approval execution
- Hermes CLI fake execution
- Hermes Game fake execution
- background Hermes Game cancellation
- background Hermes Game redirect/replan
- background Hermes Game pause/resume
- Hermes connection explicit path
- frontend event type serialization
- Remote Operator HTTP router creation
- Remote platform capability registry
- Remote HTTP state sharing the same OperatorState
- Remote HTTP task roundtrip
- Remote bind configuration fail-closed
- Bearer authentication and allowlists
- Existing out-of-scope task filtering and control denial
- Canonical project path storage
- Fixed-size token digest and 32-byte remote-token minimum
- Browser-readable CORS headers on allowed-origin 401/403
- Audit client-id sanitization
- Restricted CORS preflight
- Bounded, body-free remote audit records

已经定位 D 盘 Node 24，并直接执行 D 盘项目依赖，没有调用 C 盘 Node/npm：

```text
vue-tsc --noEmit: passed, 0 errors
Game Operator + Remote SDK Vitest: 5 files passed, 99 tests passed
```

## 现在仍然不是“100% 产品完成”

不要再宣称项目 100% 完成。当前更准确的状态是：

- 后端 Operator runtime 的核心控制闭环已验证。
- Remote Operator HTTP、第一阶段安全边界和 TypeScript SDK 已打通。
- 远程审计目前只在内存中，尚未持久化，也没有账户/RBAC。
- Hermes Game 真实改文件能力还没有接入，只生成 proposal artifact。
- 前端 Game Operator 面板是否完整呈现 pause/resume/redirect/cancel 事件，还需要 UI 验证。
- VSCode 扩展、IDEA 插件、移动端目前有协议入口和 SDK 基座，但独立 UI 端仍要继续实现。
- 还没有端到端跑真实 Godot 工程 + 真实 Hermes Rust runtime + proposal/diff/apply 的完整用户路径。
- 旧文档存在编码损坏，应该逐步用新的 UTF-8 文档替换，不建议相信旧报告里的“100% 完成”。

## 下一步执行优先级

### P0：前端 Game Operator 面板验收

1. 已完成：使用 D 盘 Node 24 跑通 `vue-tsc --noEmit`。
2. 已完成：Game Operator + Remote SDK 共 99 个 Vitest 通过。
3. 用真实 Tauri 窗口验证 approval 后按钮不阻塞，timeline 能持续刷新。
4. 验证 pause/resume/stop/redirect 对应事件在 timeline 中出现。
5. 验证 redirect 后旧 approval 不再误导用户，只展示新 pending approval。
6. 验证 Remote Operator 状态条能在桌面端 server 启动后显示 online。

### P0：真实 Hermes Game Rust runtime 接入

1. 定位 D 盘 Hermes Rust 源码。
2. 明确它的 CLI 参数或 crate API。
3. 优先保留子进程边界，先接 CLI，等协议稳定后再考虑 crate 级集成。
4. 输出必须逐步变成结构化事件：`task_started`、`tool_call`、`file_patch_proposed`、`diff`、`summary`。
5. 长任务必须能流式给出进度，不能只等最终 stdout。

### P1：proposal -> patch preview -> apply

1. 让 Hermes Game 输出 machine-readable patch/diff，而不是只写 markdown proposal。
2. 后端转成 `FilePatchProposed` 事件和 approval preview。
3. 用户 approve patch 后才写真实文件。
4. 所有文件写入必须走 `PathGuard` 和 backup。

### P1：远程控制安全层

第一阶段已完成：Token、默认回环绑定、Origin/Client/Domain/Project allowlist、脱敏内存审计。

剩余：

1. 对 destructive action 做二次 approval。
2. 将审计持久化并提供保留/导出策略。
3. 公网部署增加 TLS、账户/RBAC、速率限制和密钥轮换。

### P1：VSCode / IDEA / Web 平台接入

协议已经统一。后续各端只需要围绕同一套 SDK 做 UI：

```text
platform UI
  -> operatorRemoteApi
  -> Remote Operator HTTP server
  -> shared OperatorState
  -> Hermes Game runner
```

VSCode 端最小面板：

- 项目路径自动取 workspace folder。
- 输入 goal。
- Start / Pause / Resume / Stop / Redirect。
- Approval 卡片。
- Timeline。
- Proposal artifact 链接。

IDEA 端最小面板：

- 项目路径取 Project basePath。
- 同样调用 HTTP API。
- 先做工具窗口，不急着做深 IDE PSI 集成。

Web 端最小面板：

- 远程地址输入。
- 平台能力显示。
- 任务列表。
- 任务详情和事件流。
- Approval 操作。

### P1：整理项目

项目确实需要修整，但不要一口气重构全仓库。顺序应该是：

1. 固定 D-only 工具链和测试入口。
2. 收敛三条主线：`operator runtime`、`game domain`、`frontend game-operator`。
3. 把旧 mock/demo/重复测试逐步归档。
4. 保留当前通过的 Rust operator tests 作为护栏。
5. 每次只整理一个边界，整理后立刻跑对应测试。

## 给 Qwen 3.7 Plus + Claude Code 的继续执行提示词

```text
你们继续接手 D:\dingsun\acp-ui，必须遵守：

1. 所有文件读写、临时目录、缓存和工具链配置只允许在 D 盘；不要把项目操作写到 C 盘。
2. 不要再宣称项目 100% 完成。先阅读 docs/codex/2026-07-09-runtime-closure-check.md。
3. 当前后端 Operator runtime 已通过 Rust 测试：
   D:\Rust\.cargo\bin\cargo.exe test --lib
   最新结果为 269 passed。
4. 不要破坏这些能力：
   - approve 立即返回并后台执行
   - stop 能取消 Hermes Game 子进程
   - pause 停止当前子进程并保留 Paused
   - resume 重新启动当前目标
   - redirect 改 goal、作废旧 approval、重新生成 Godot plan、取消旧后台结果
   - 事件流必须记录 operator 介入
   - Remote Operator HTTP API 使用同一个 OperatorState，不要改回 mock
   - Remote Operator 默认只绑定 127.0.0.1；非回环必须 Token + project roots
   - 不允许恢复 Any CORS，不允许把 Token 放进 URL、日志或审计正文
5. 远程协议已经有：
   - Rust HTTP endpoints in src-tauri/src/http_server.rs
   - TypeScript SDK in src/api/operatorRemoteApi.ts
   - Shared types in src/types/operator.ts
6. 前端 Game Operator 面板已经补了基本闭环：
   - approval 后用 task/event/approval polling 展示后台进度
   - stop 后继续轮询直到后端终态
   - redirect 后清空旧 approval，并请求新计划
   - 展示 Remote Operator server 状态和平台能力数量
7. D 盘 Node 24 已找到并验证：
   - vue-tsc --noEmit 已通过
   - Game Operator + Remote SDK Vitest 共 99 passed
   - 继续使用 D 盘 node.exe 直接执行 D 盘 node_modules，不要调用 C 盘 npm
8. 下一步定位 D 盘 Hermes Rust 版本，优先以 CLI 子进程接入，不要先做 crate 深度耦合。
9. Remote Operator 第一阶段安全层已完成；下一层做 destructive action 二次审批和持久化审计，不要把内存审计误称为企业级审计。
10. 每次修改后至少跑：
   $env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
   $env:TMP=$env:TEMP
   $env:CARGO_HOME='D:\Rust\.cargo'
   $env:RUSTUP_HOME='D:\Rust\.rustup'
   & 'D:\Rust\.cargo\bin\cargo.exe' test operator::commands::tests --lib
   & 'D:\Rust\.cargo\bin\cargo.exe' test operator::security::tests --lib
   & 'D:\Rust\.cargo\bin\cargo.exe' test http_server::tests --lib

任务目标：
把 Godot Game Operator 做成真实可用的第一阶段：
用户选择 Godot 项目 -> 生成计划 -> 审批 -> Hermes Game 后台执行 -> 用户可暂停/恢复/停止/改方向 -> 事件面板实时显示 -> 产出 proposal/diff -> 用户审批后应用文件。

不要大范围重构。每次只改一个闭环，并用测试证明它真的成立。
```
