# 游戏开发 Agent 闭环验收与后续路线

更新日期：2026-07-10

适用仓库：`D:\dingsun\acp-ui`

Hermes Game 源码：`D:\dev-tools\hermes-game`

> 交接更新：真实 Tauri 窗口已经完成启动、Remote 任务发现和 SQLite 重启恢复实测，但 1024x720 下审批决策按钮可达性仍未通过。最终产品不再按 P0/P1 阶段定义，完整目标以 `2026-07-10-hermes-game-operator-complete-product-blueprint.md` 为准；当前状态与证据见 `2026-07-10-game-agent-handoff-and-execution-plan.md`；一体化 Claude Code 提示词见 `2026-07-10-qwen-claude-execution-prompts.md`。本文件第十节旧提示词只作历史记录，不应原样重复执行。

## 一、结论

方向是对的，但产品仍不能宣称“100% 完成”。当前已经完成的是 Godot 游戏开发 Agent 的第一条可信执行闭环：

```text
操作员创建任务
  -> ACP 分析 Godot 项目并生成计划
  -> 第一次审批：是否允许 Agent 生成候选补丁
  -> Hermes Game 0.4.0 在 no-tools 模式读取受限项目上下文
  -> Hermes 只返回严格 JSON，不直接写项目
  -> ACP 严格解析、检查路径、计算真实哈希并生成逐文件 diff
  -> 第二次审批：操作员查看每个文件的 operation/path/diff
  -> ACP 再次检查文件是否过期
  -> 集中备份并批量应用
  -> 失败时逆序回滚，成功时记录事件、文件变化和备份位置
```

这条链路解决了 Agent 产品最重要的信任问题：模型负责提出候选改动，确定性代码负责校验和执行，操作员始终拥有停止、修改方向和最终落盘权。

当前已经真实验证了 Qwen 在线生成结构化 Godot 补丁，工具注册数为 0，提案阶段项目没有被修改；随后同一份真实产物通过 Rust 严格解析器并应用到隔离项目副本。它不是只靠 fake CLI 跑通的演示。

Godot 引擎级自动验证和 Operator SQLite 持久化恢复现已完成。持久层能够在进程重启后恢复任务、事件、审批、文件变更、pending patch 与 pending validation，且不会在启动时擅自恢复 Hermes 或 Godot 子进程。当前尚未完成的核心部分包括：真实桌面窗口验收、VSCode/IDEA/Web 独立操作端、企业级身份与审计、技能/MCP/Hook 的游戏领域编排。因此正确做法是继续沿当前架构逐层补齐，而不是推倒重来或同时扩展到运营、视频、漫画等领域。

## 二、为什么当前架构正确

### 2.1 控制面与执行器分离

ACP Operator 是控制面，负责：

- 任务状态机。
- 计划与补丁两级审批。
- 暂停、恢复、停止和改目标。
- 远程 API、安全边界和审计。
- 项目路径与补丁规则校验。
- 备份、应用、回滚和事件记录。

Hermes Game 是执行器，负责：

- 理解目标和受限项目上下文。
- 使用游戏开发系统提示。
- 生成结构化候选改动。

Hermes 在提案阶段不持有终端、文件写入或技能工具。这样即使模型被项目文件中的提示注入误导，也无法绕过 ACP 直接改文件。

### 2.2 两次审批不能合并

第一次审批确认的是“允许 Agent 根据这个计划生成候选改动”。第二次审批确认的是“允许 ACP 应用眼前这些具体文件差异”。

两次审批的风险对象不同，不能用一次“同意计划”代替最终落盘授权。当前分别使用：

- `operator.plan.generate_patch`
- `operator.patch.apply`

任务只有在第二次审批成功、补丁实际应用、固定模板的 Godot 验证通过或被明确标记为 skipped，并记录完整事件后才进入 `completed`。验证失败时任务进入 `failed`，已批准文件和备份仍保留给操作员处理。

### 2.3 模型输出只被当作不可信数据

结构化补丁 v1 只接受：

- `version: 1`
- 非空 `summary`
- `changes` 数组
- `operation: create | replace`
- 项目内的 `/` 相对路径
- 完整 UTF-8 目标文件内容
- 可选 `expected_sha256`
- 只供人查看、不自动执行的 `validation`

以下内容会拒绝整个补丁集：

- Markdown 代码块或 JSON 外的解释文字。
- 未知字段、未知版本、空 changes。
- `delete`、移动、重命名、命令或二进制写入。
- 绝对路径、盘符路径、反斜杠、`.`、`..`。
- 重复路径或大小写冲突路径。
- 符号链接路径。
- `.git`、`.godot`、`.operator`、`node_modules`、`target`、`build`、`dist`。
- 不在文本扩展名白名单内的文件。
- 超出单文件、总内容、文件数或路径长度限制的产物。

### 2.4 预览与应用之间有并发保护

ACP 在生成预览时读取真实原文件并计算 SHA-256。第二次审批后、写入前会再次读取并比较：

- 用户在等待审批时修改了文件：拒绝应用，不覆盖用户内容。
- `create` 目标在等待期间已出现：拒绝应用。
- 模型提供的 `expected_sha256` 不匹配：提案阶段直接拒绝。

这使“操作员随时手动修改项目”与 Agent 并行工作成为可信语义，而不是最后写入者覆盖一切。

## 三、已经完成的实现

### 3.1 Hermes Game 0.4.0

`D:\dev-tools\hermes-game` 已增加：

- CLI 标准版本输出，当前为 `hermes-game 0.4.0`。
- `codegen --no-tools`。
- `codegen --prompt-file <UTF-8-file>`。
- `create_agent_with_tools(engine, false)`，创建空工具注册表。
- 无工具模式单元测试，确认工具定义和工具列表均为空。

ACP 实际调用合同：

```text
hermes-game --engine godot codegen controlled-proposal-request \
  --output <proposal_uuid.json> \
  --no-tools \
  --prompt-file <proposal_uuid.request.txt>
```

长提示不再放进 Windows 命令行，避免命令行长度、转义和敏感上下文泄漏问题。ACP 会先检查 `--version`，再检查 `codegen --help` 同时包含 `--no-tools` 和 `--prompt-file`；旧 0.3 CLI 会被拒绝。

已验证部署文件：

```text
D:\dingsun\acp-ui\bin\hermes-game.exe
version: hermes-game 0.4.0
size: 53,744,128 bytes
sha256: 3368A727FFB50CCA9F156ADB97852126411BB4A521A06F61791E69AD42F72E5A
```

该文件由 D 盘 Debug 构建剥离调试符号得到，已经验证版本和能力探测。当前机器上的全优化 Release PE 在链接后首次执行会被系统移除，尚未取得可审计的 Defender 事件；这属于发布产物问题，不应伪装成已解决。正式分发前仍需在干净 Windows 构建机生成、签名并验证 Release 二进制。

### 3.2 受限 Godot 上下文

`HermesGameBridge` 只采集 Godot 文本上下文：

- `project.godot`
- `.gd`
- `.tscn`
- `.tres`
- shader 文本

边界：

- 最多 40 个文件。
- 每个纳入提示的文件最多 48 KiB。
- 总上下文最多 192 KiB。
- 最多检查 5000 个目录项。
- 源文件大于 512 KiB 时跳过。
- 跳过符号链接和受保护目录。
- 不读取 JSON、CFG、INI、`.env` 等可能含凭据的文件。
- 在提示中明确把项目内容标为不可信数据，禁止遵循源码内嵌指令。

### 3.3 结构化补丁引擎

`src-tauri/src/operator/structured_patch.rs` 已实现：

- 严格 serde schema 和未知字段拒绝。
- 路径、扩展名、数量和大小限制。
- Windows 保留设备名拒绝。
- 大小写不敏感的重复路径拒绝。
- `PathGuard` 与 canonical project root 校验。
- 符号链接组件拒绝。
- 原文件和新内容 SHA-256。
- 逐文件 review diff。
- 应用前 stale-check。
- 全量 preflight。
- 集中备份。
- 批量写入与写后哈希验证。
- 中途失败的逆序回滚。

备份位置：

```text
D:\dingsun\acp-ui\.operator\hermes-runs\<task_id>\backups\<approval_id>\<project-relative-path>
```

`create` 没有原文件，因此不会伪造空备份；`replace` 必须保留原文件副本。

### 3.4 Operator 状态与事件

已完成：

- 首轮 plan approval ID 使用 UUID，不会因同毫秒重规划而重复。
- `request_changes` 会生成新的可操作审批，旧审批不可重用。
- Hermes 合法产物进入 `pending_patches`。
- 任务从 `running` 返回 `waiting_approval`。
- 产生 `FilePatchProposed` 事件。
- 第二次审批预览包含 files 和逐文件 diffs。
- 应用后产生 `FilePatchApplied`，任务保持 running，等待 Godot 验收门给出结果。
- 验收发出 `ValidationStarted` 以及 `ValidationPassed`、`ValidationFailed`、`ValidationSkipped` 之一；passed/skipped 后才 `TaskCompleted`，failed 后 `TaskFailed`。
- pause/stop/redirect 可取消正在运行的 Godot 子进程；pause 保留 pending validation，resume 重新执行。
- stale patch 使任务失败但保留用户修改。
- stop、redirect、reject、request_changes 会丢弃旧 pending patch。
- Planning、WaitingApproval、Running、Paused、Redirecting 均可停止。
- pending patch 意外丢失时不会先消费审批记录，任务仍停在 `waiting_approval`，避免卡死在 `running`。
- 通用 Hermes 直接写入路径对 `game.*` 被阻断，真实项目写入只能经过结构化补丁审批。

### 3.5 前端审批体验

`ApprovalDrawer.vue` 已显示：

- 文件 operation。
- 项目相对路径。
- 可展开逐文件 diff。
- approve、reject、request_changes。

共享 TypeScript 类型增加 `FileDiffPreview`、`preview.diffs` 和四类验证事件。`ProgressTimeline` 可显示验证开始、通过、失败和跳过；界面没有隐藏的直接写入按钮。

### 3.6 Remote Operator

远程 HTTP 与桌面端共享同一个 `OperatorState`。核心路由包括任务创建、查询、事件、审批、暂停、恢复、停止、改目标、摘要、平台能力和审计。

结构化补丁二次审批也可经远程 HTTP 完成，HTTP 级测试验证了：

- 远程读取 pending patch approval 和精确 diff。
- 远程提交 approve。
- 项目文件被应用。
- 原文件备份存在。
- 桌面端与远程端看到同一任务状态。
- 远程事件接口可读取明确的验证事件；测试环境无显式 Godot 时返回 skipped，不伪造 passed。

当前安全边界：

- 默认绑定 `127.0.0.1:1422`。
- 非回环监听必须配置高熵 Token 和允许项目根目录。
- Origin、Client、Domain、Project Root allowlist。
- canonical 路径校验。
- Token 只保存固定长度摘要并恒定时间比较。
- Token 不进入 URL、日志或审计正文。
- CORS 不是任意 Origin。
- 写操作进入有界内存审计。

这适合本机与受控局域网。公网仍必须增加 TLS、账户/RBAC、速率限制、密钥轮换和持久化审计。

## 四、真实在线闭环证据

烟测根目录：

```text
D:\dingsun\acp-ui\.tmp-tests\hermes-real-smoke
```

执行器：

```text
D:\dingsun\acp-ui\bin\hermes-game.exe
```

模型：

```text
alibaba-coding-plan:qwen3.6-plus
```

日志关键事实：

- Hermes Game 版本为 0.4.0。
- 调用携带 `--no-tools` 和 `--prompt-file`。
- 工具注册数为 0。
- 在线调用正常完成，stderr 为空。
- 输出为单个 JSON 对象，没有 Markdown fence。
- JSON 大小 2294 bytes。
- JSON SHA-256 为 `55ABC81FCF99FB0BAEEFA37D284BF6C881409BD742A891EC10EB4EF8A6282BC0`。
- `version = 1`。
- 一项 change：`create scripts/health.gd`。
- 提案结束后原项目没有出现 `health.gd`。

随后执行 opt-in Rust 闭环测试：

```text
operator::structured_patch::tests::validates_and_applies_external_real_hermes_artifact
```

结果：

- 严格解析通过。
- 补丁应用到 `apply-project` 隔离副本。
- `scripts/health.gd` 已生成，大小 1232 bytes。
- 原始 smoke 项目仍未生成该文件。
- 备份根目录已创建。

这证明了“真实模型提案 + 真实严格解析 + 真实落盘”。随后又完成了真实 Godot 验收，因此当前证据链已经延伸到“真实引擎扫描与脚本解析”。

### 4.1 真实 Godot 验收证据

Godot 仅安装在 D 盘：

```text
D:\dev-tools\godot\4.7-stable\Godot_v4.7-stable_win64.exe
D:\dev-tools\godot\godot.exe
```

版本与完整性：

```text
Godot 4.7.stable.official.5b4e0cb0f
godot.exe SHA-256:
B2CA888D5115A6CEDEE564764A2EE494A625F2EC2EDBABD010FE33C9A88A6BF8

官方 zip SHA-512，下载值与 SHA512-SUMS-4.7-stable.txt 一致：
41645A908EB3181D6F2D1201ED7B6D6F095F6A23AAED8903D5D255277CC8D142814F3E6817F865B3CAC142C39B8AFF99280091D3BBDAA301517730B3BA0522B9
```

ACP 固定执行模板：

```text
godot.exe --headless --editor --path <canonical-D-project> --quit-after 1
```

模型返回的 `validation` 数组只作为操作员参考，不进入命令参数。runner 使用参数数组，不经过 shell；stdout/stderr 由独立线程持续排空并分别限制为 64 KiB；默认超时 120 秒，可取消；超时或取消后执行 kill、wait 和进程回收。

真实 smoke 项目：

```text
D:\dingsun\acp-ui\.tmp-tests\hermes-real-smoke\apply-project
```

真实运行完成了 Godot 的首次文件系统扫描、全局类名加载、GDExtension 校验、自动加载脚本与插件初始化，没有 GDScript parse/script diagnostics。stderr 只有 `Scan thread aborted...` 的退出阶段 warning，不被误判为脚本失败。

测试环境不会读取宿主机的自动发现结果：`OperatorState` 在 `cfg(test)` 下使用 `GodotValidationDiscovery::Disabled`，生产构建使用 `Auto`；审批或恢复时只发现一次 Godot，并把确定的可执行路径固化进 `ApprovedTaskRun`。真实引擎测试通过显式 D 盘环境变量 opt-in，确保普通离线测试在安装 Godot 前后结果一致。

## 五、验证结果

所有缓存、临时目录、工具链和产物均配置在 D 盘。

### 5.1 ACP Rust

```text
cargo test --lib
312 passed; 0 failed; 2 ignored
```

两个 ignored 用例分别是：真实 Hermes artifact/project/backup 应用测试，以及真实 Godot executable/project 验收测试。它们都要求显式 D 盘环境变量。本轮已经分别单独运行：

```text
real Hermes artifact: 1 passed; 0 failed
real Godot 4.7:       1 passed; 0 failed
```

因此默认离线套件、真实模型产物套件和真实引擎套件均通过。

关键覆盖：

- 结构化 schema、路径、大小和保护目录。
- preview 不写项目。
- create/replace、备份和应用。
- stale user edit。
- 批量写入失败回滚。
- Hermes 旧 CLI 能力拒绝。
- plan request_changes 新审批 ID。
- pending patch 丢失时审批不被消费。
- stop/redirect/pause/resume。
- 远程二次审批、落盘和备份。
- Godot 成功、非零退出、零退出脚本诊断、超时、取消、输出截断与子进程回收。
- ValidationStarted/Passed/Failed/Skipped 事件和任务终态。
- 测试禁用宿主 Godot 自动发现，生产仍自动发现 D 盘 Godot。
- Remote 安全策略、CORS、scope 和脱敏审计。
- SQLite schema v1、事务快照和 append-only 事件。
- waiting approval、planning、running、pending patch 与 pending validation 重启恢复。
- stale patch 恢复收口、失败快照回滚、未来 schema 和非 D 路径拒绝。
- 事件重排/断档拒绝，以及 Remote HTTP 读取同一份恢复状态。

### 5.2 Hermes Game Rust

```text
cargo test
7 passed; 0 failed
```

包含无工具注册表为 0 的测试。

### 5.3 前端

```text
vue-tsc --noEmit
passed; 0 errors

Vitest
79 files passed
1209 tests passed

Game Operator + Remote 相关套件
5 files passed
104 tests passed
```

全量前端套件通过。相关套件覆盖 GameOperatorView、ApprovalDrawer、OperatorControlBar、ProgressTimeline 和 Remote SDK，其中时间线包含四类验证事件。

### 5.4 已知测试噪声

ACP Rust 编译仍有约 276 条历史 warning，主要来自旧 adapter、demo 和未使用代码。它们没有导致本轮测试失败，但说明项目确实需要渐进式整理。不要使用一次性 `cargo fix` 或大范围删除来制造“干净”假象。

## 六、成熟度边界

| 能力 | 当前状态 | 是否可宣称完成 |
|---|---|---|
| Godot 任务计划与首轮审批 | 已实现并测试 | MVP 完成 |
| Hermes 无工具结构化提案 | 已真实在线验证 | MVP 完成 |
| diff 二次审批 | 桌面类型/UI/HTTP 已接通 | MVP 完成 |
| 路径、哈希、备份、应用、回滚 | 已实现并测试 | MVP 完成 |
| 暂停、恢复、停止、改目标 | 子进程语义已测试 | MVP 完成 |
| 远程局域网控制 | HTTP/SDK/第一阶段安全层 | 受控环境可用 |
| Godot 引擎自动验证 | 固定模板、超时/取消/截断/事件已实现，真实 4.7 smoke 通过 | MVP 完成 |
| 重启后任务/审批恢复 | SQLite schema v1、显式 resume、stale 收口和 Remote 同源读取已测试 | MVP 完成 |
| 真实 Tauri 窗口交互验收 | 单测通过，尚缺视觉/交互实测 | 未完成 |
| VSCode 独立操作面板 | 有旧插件资产和 HTTP SDK 基座 | 未完成 |
| IDEA 工具窗口 | 只有平台协议声明 | 未完成 |
| Web/移动独立操作端 | 有 HTTP SDK 基座 | 未完成 |
| 企业账户、RBAC、TLS、持久审计 | 未实现 | 未完成 |
| Skills/MCP/Hooks 游戏领域编排 | 基础设施分散存在，尚未形成 Game Domain Pack | 未完成 |
| Unity/Ren'Py/Unreal 同等级闭环 | 只有旧 adapter/提示/声明 | 未完成 |

## 七、下一阶段优先级

### 已完成 P0-1：Godot 验证门

下列要求均已实现并测试：

1. 只发现 D 盘 Godot 可执行文件，或接受显式 `GODOT_BIN`/`GODOT4_BIN` D 盘路径。
2. 使用参数数组启动，不拼接 shell 字符串。
3. 固定执行 headless editor import/parser 检查。
4. 支持超时、取消、输出大小上限和子进程回收。
5. 发出 `ValidationStarted`、`ValidationPassed`、`ValidationFailed`、`ValidationSkipped`。
6. 验证失败时保留已应用文件和备份，任务明确进入 `failed`。
7. 验证命令来自 ACP 固定模板，不执行模型返回的 `validation` 文本。
8. 没有 Godot 时明确 skipped 和原因，不误报 passed。
9. 生产自动发现与测试禁用发现分离，测试不依赖宿主安装状态。
10. D 盘真实 Godot 4.7 smoke 已通过。

### 已完成 P0-2：Operator 持久化与恢复

下列要求均已实现并测试，详细设计见 `docs/codex/2026-07-10-operator-persistence-and-recovery-spec.md`：

1. SQLite `PRAGMA user_version = 1`，默认库位于 `D:\dingsun\acp-ui\.operator\operator-state.sqlite3`。
2. tasks、append-only events、approvals、file changes、pending patches、pending validations 和 recovery origins 在同一事务快照中保存。
3. 生产启动恢复失败时明确拒绝静默回退到内存模式。
4. 遗留 planning/running/redirecting/idle 转为 paused，不在启动阶段 spawn Hermes 或 Godot。
5. 操作员显式 resume 后按原阶段恢复；planning 只重新生成计划审批，不越过第一道审批。
6. pending patch 恢复时重新解析并校验真实文件；stale 时关闭审批、任务失败且不覆盖用户修改。
7. 事件 ID、任务、sequence、payload、timestamp 均做一致性校验，并拒绝重排和断档。
8. Tauri 与 Remote HTTP 继续共享同一份恢复后的 `OperatorState`。
9. 事务失败不覆盖最后一次成功快照；未来 schema、非 D 路径和不一致数据明确拒绝。
10. 重启、二次重启、审批延续、patch apply、validation pending 和真实 HTTP 路由均有回归测试。

### P0-3：真实桌面窗口验收

需要启动 Tauri dev server 和应用窗口，使用真实任务检查：

- 两次审批是否清楚区分。
- 长 diff 是否可读、滚动和折叠。
- polling 时按钮是否稳定。
- stop/redirect 后旧审批是否消失。
- pause/resume 是否有明确反馈。
- 远程状态条是否真实 online。
- 错误、stale、validation failure 是否不会被绿色完成状态覆盖。

### P1-1：VSCode 操作端

先做最小可用面板，不在扩展内复制 Agent runtime：

- workspace folder 自动映射 project path。
- Remote Operator 地址、Token、Client ID 设置。
- Start/Pause/Resume/Stop/Redirect。
- 计划审批和逐文件 diff 审批。
- Timeline 与任务摘要。
- 打开本地 artifact/backup 文件。
- 断线重连与 scope 错误提示。

扩展只是 HTTP 客户端，真正任务状态仍由 ACP Operator 管理。

### P1-2：IDEA 工具窗口

第一版使用 Kotlin 工具窗口：

- `Project.basePath` 作为项目路径。
- 调用同一 HTTP API。
- 展示任务、审批和 timeline。
- 使用 IDE diff viewer 显示 patch preview。
- 不急于接 PSI 写入，文件落盘仍由 ACP 完成。

### P1-3：Web 远程控制台

复用 `operatorRemoteApi.ts`，增加：

- 远程连接配置。
- 任务列表和详情。
- HTTP event polling。
- 审批与控制操作。
- 审计查看。

浏览器 WebSocket 不能把 Token 放 query string。正式支持前需要设计首帧认证或短期一次性票据。

### P1-4：Game Domain Pack

在闭环稳定后收敛 Skills/MCP/Hooks：

- Skills：Godot gameplay、scene、UI、save system、debug、performance、export。
- MCP：Godot editor bridge、asset catalog、issue tracker、版本控制只读信息。
- Hooks：提案前上下文过滤、补丁后格式检查、验证前后事件、成本与超时策略。
- Memory：项目约定、已批准架构决策、失败验证和操作员偏好。

所有能力必须经过 Operator policy。Skill/MCP/Hook 不能绕过二次审批直接写项目。

## 八、项目应该如何修整

项目确实混乱，但不应该大爆炸式重构。建议按以下顺序处理：

### 8.1 建立可信主线

明确一条产品主线：

```text
src-tauri/src/operator
src-tauri/src/domains/games/godot
src/features/game-operator
src/api/operatorRemoteApi.ts
D:/dev-tools/hermes-game
docs/codex
```

其他 adapter、ERP、视频、办公、营销模块暂时不是 Game Agent P0，不要让它们进入当前验收标准。

### 8.2 固定验证入口

建立 D 盘脚本统一执行：

- Rust operator tests。
- Remote HTTP tests。
- Hermes Game tests。
- vue-tsc。
- Game Operator Vitest。
- 真实 Hermes 可选 smoke。
- 后续 Godot headless validation。

脚本必须先检查 TEMP/TMP/CARGO_HOME/RUSTUP_HOME/Node 路径均在 D 盘，发现 C 盘立即失败。

### 8.3 隔离历史代码

先做清单，不直接删除：

- 哪些 examples 仍被使用。
- 哪些 adapter 只是样板。
- 哪些旧 Operator 模块被新控制面替代。
- 哪些 mock 报告曾把“存在文件”误当成“功能完成”。

确认无引用并有替代测试后，再按小批次归档或删除。每批必须独立提交和回归。

### 8.4 修复编码与 warning

优先修复会影响产品的编码问题：CLI 中文日志乱码、旧文档乱码、按钮字符损坏。warning 按模块逐步清理，不要在功能提交中混入全仓自动修复。

### 8.5 建立完成定义

以后任何人说“完成”必须同时给出：

- 对应 acceptance criteria。
- 真实实现文件。
- 可重复命令。
- 测试数量和结果。
- 是否使用 fake/mock。
- 真实进程或真实模型证据。
- 未覆盖边界。
- 产物和日志路径。

没有这些证据，只能说“已实现待验收”。

## 九、Game Agent 第一阶段完成定义

满足以下全部条件后，才能把“Godot Game Agent 第一阶段”标为完成：

1. 真实 Godot 项目可创建任务并生成计划。
2. 操作员能审批、拒绝、请求修改计划。
3. Hermes 无工具生成结构化候选补丁。
4. 操作员能看到逐文件 diff。
5. 补丁应用有边界、哈希、备份和回滚。
6. Godot headless 验证结果进入事件流。
7. 任务、事件、审批和 pending patch 可在重启后恢复。
8. stop/pause/resume/redirect 对真实长任务有效。
9. 桌面窗口完成真实交互验收。
10. VSCode 至少有一个可用操作面板。
11. Remote Operator 在受控局域网完成真实客户端验收。
12. 安全文档明确本机、局域网和公网部署边界。

当前已完成第 1 至 7 的后端闭环和第 8 的子进程测试；第 9、10、11 仍需继续。第 12 已完成受控局域网安全边界文档，公网部署边界仍需随企业身份层继续加固。

## 十、给 Qwen 3.7 Plus + Claude Code 的下一轮提示词（当前）

```text
你们继续接手 D:\dingsun\acp-ui。不要宣称项目 100% 完成，也不要扩展运营、企业、视频、漫画、Unity 或 IDEA。当前只推进 Godot Game Agent 的下一项 P0：真实 Tauri 桌面窗口交互验收，并修复验收发现的问题。

开始前必须阅读：
1. D:\dingsun\acp-ui\docs\codex\2026-07-10-game-agent-closure-and-roadmap.md
2. D:\dingsun\acp-ui\docs\codex\2026-07-10-operator-persistence-and-recovery-spec.md
3. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-hermes-structured-patch-approval.md
4. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-secure-remote-operator-access.md

硬约束：
1. 所有源码、Node/Rust 工具链、缓存、TEMP/TMP、SQLite、WebView2 user-data、截图、日志和构建产物只能在 D 盘。启动前显式设置 D 盘 TEMP/TMP、CARGO_HOME、RUSTUP_HOME、XDG_CACHE_HOME、ACP_OPERATOR_STATE_DB 和 WEBVIEW2_USER_DATA_FOLDER，并核对实际路径。不要让本轮产物写入 C 盘。
2. 必须启动真实 Tauri 桌面进程并操作真实应用窗口。普通浏览器、组件单测、mock invoke 或静态截图只能作为补充，不能替代桌面验收。
3. 不得破坏现有两级审批 operator.plan.generate_patch -> operator.patch.apply，不得让 Hermes 在提案阶段获得工具或写权限。
4. 不得新增生产环境“跳过审批”“注入补丁”或测试后门。若需要确定性夹具，只能放在 test/debug 编译边界，且必须证明 release 构建不包含入口。
5. Operator 继续使用 D 盘 SQLite schema v1；真实 Tauri 与 Remote HTTP 必须共享同一个恢复状态。不要复制第二份前端状态或 mock 数据源。
6. 不执行模型返回的 validation 文本；Godot 只运行 ACP 固定参数模板。保留路径、hash、backup、stale-check、CORS、token 和 scope 安全边界。
7. 不做全仓重构，不运行全仓 cargo fix，不删除或回退用户已有修改。

当前可信基线：
- ACP Rust default suite: 312 passed, 0 failed, 2 opt-in tests ignored；真实 Hermes artifact 与真实 Godot 用例分别单独 1 passed。
- Hermes Game Rust: 7 passed, 0 failed。
- Frontend Vitest: 79 files, 1209 tests passed；Vue typecheck 通过。
- Operator SQLite schema v1、append-only event、重启恢复、显式 resume、stale patch 收口、pending validation 和 Remote 同源读取均已实现，不要重复实现。
- D:\dingsun\acp-ui\bin\hermes-game.exe 为 0.4.0；D:\dev-tools\godot\godot.exe 为 4.7.stable.official.5b4e0cb0f。

本轮目标：
建立可重复的 Windows Tauri 桌面验收路径，实际打开 Game Operator，完成至少一条从创建任务到计划审批的真实 UI 流程，并用确定性场景覆盖补丁审批、长 diff、错误、恢复和控制按钮。发现交互、布局或状态同步问题要直接修复并增加回归测试，不能只写报告。

实施顺序：
1. 先盘点 Tauri 启动、路由、窗口配置和现有 E2E 依赖。项目已安装 @wdio/tauri-service 但没有可确认的配置；优先建立最小 `wdio.tauri.conf.ts`/desktop spec 或同等级真实窗口驱动。若当前 Windows 驱动确实不可用，记录精确阻塞证据，再使用可审计的真实窗口操作与截图方案，不能退化成浏览器 mock 后宣称通过。
2. 在 D 盘创建隔离 Godot fixture、隔离 SQLite 和 WebView2 user-data。不得使用或污染用户真实 `.operator` 库；不得复用会被测试改写的基准项目。
3. 启动真实 Tauri 应用，确认进入实际 Game Operator 主界面，而不是营销页、旧 demo 或 mock 页面。记录进程、窗口标题、应用版本、后端数据库和项目路径。
4. 验收创建任务：选择真实 D 盘 Godot 项目、填写目标、创建后看到 planning -> waiting_approval、计划步骤和第一道审批；按钮在请求期间不得重复提交或造成布局抖动。
5. 验收审批语义：approve/reject/request_changes 文案和风险对象明确；request_changes 后旧审批消失且新 approval ID 出现；计划审批绝不能被展示成文件落盘授权。
6. 用真实后端状态验收补丁审批：显示 summary、operation、relative path、逐文件 diff 和风险；至少覆盖长文件、多文件、create/replace。长 diff 必须可滚动、可读、不溢出窗口，审批按钮始终可达。确定性 fixture 与真实 Qwen/Hermes opt-in 结果要分开报告，不能混称。
7. 验收控制条：running 时 pause/stop/redirect 可用；paused 时 resume 可用；terminal 状态不得显示误导性操作。操作后旧轮询结果不能把新状态覆盖回去。
8. 验收异常：停机期间外部修改导致 stale 时，UI 明确失败且不显示 completed；Godot validation failed/skipped、持久库恢复失败、Hermes 不可用均显示真实原因，不泄露 token 或完整敏感路径正文。
9. 做一次真实桌面进程重启：在 waiting_approval 或 paused 时关闭并重启应用，确认任务、事件、审批、diff/recovery 状态从同一 SQLite 恢复，且启动时没有自动 spawn Hermes/Godot。
10. 至少在 1440x900、1280x800、1024x720 三种窗口尺寸截图检查。不得有文字/按钮重叠、内容裁切、卡片套卡片、不可达审批按钮或靠颜色单独表达状态。截图和验收日志放在 D:\dingsun\acp-ui\.artifacts\desktop-acceptance\<timestamp>，不得提交临时数据库或密钥。
11. 对发现的问题做小范围修复，并补充 Vitest 与真实 desktop E2E。不要为了截图重做整个视觉系统；优先修复状态真实性、操作可达性、长 diff、错误和恢复反馈。
12. 最后更新本路线图，列出每个场景是 real、deterministic fixture 还是 mock；只有真实窗口和真实后端场景才能计入 P0-3 完成。

最低验收证据：
- 真实 Tauri 进程启动命令、PID/退出码和窗口截图。
- 三种窗口尺寸的关键状态截图。
- 创建任务、计划审批、补丁审批、pause/resume/stop/redirect、stale、validation 和重启恢复的逐项结果。
- desktop E2E 的精确测试数、Vitest 1209+、vue-tsc、ACP Rust 312+ 和 Hermes Game 7 项回归结果。
- 所有夹具/数据库/日志/截图的 D 盘绝对路径，以及确认 C 盘未写本轮产物的方法。
- 未覆盖边界与下一步，不得只回复“100% 完成”。
```

## 附录 A、已完成的持久化实施提示词（仅供审计，不要重复执行）

```text
你们继续接手 D:\dingsun\acp-ui。不要宣称项目 100% 完成，也不要同时扩展运营、企业、视频、漫画或 Unity。当前只推进 Godot Game Agent 的下一项 P0：Operator 状态持久化与重启恢复。

开始前必须阅读：
1. D:\dingsun\acp-ui\docs\codex\2026-07-10-game-agent-closure-and-roadmap.md
2. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-hermes-structured-patch-approval.md
3. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-secure-remote-operator-access.md

硬约束：
1. 所有源码、工具链、缓存、TEMP/TMP、测试夹具和构建产物只允许在 D 盘。发现任何命令将项目产物写到 C 盘时立即停止该命令并改为 D 盘。
2. 不得破坏现有两级审批：operator.plan.generate_patch -> operator.patch.apply。
3. Hermes 提案必须继续固定使用 --no-tools 和 --prompt-file；不能恢复终端、文件或 skill 工具。
4. 模型输出始终是不可信数据。不能执行模型 validation 文本，不能放宽路径、哈希、备份或 stale-check。
5. Remote Operator 必须继续共享同一个 OperatorState；不能改成 mock。不能恢复 Any CORS，不能把 Token 放 URL、日志或审计正文。
6. 不做全仓重构，不运行全仓 cargo fix，不删除用户已有修改。

当前可信基线：
- ACP Rust: 296 passed, 0 failed, 2 opt-in tests ignored by default；真实 Hermes artifact 与真实 Godot 用例均已单独 1 passed。
- Hermes Game Rust: 7 passed, 0 failed。
- Vue typecheck: passed。
- Frontend full Vitest: 79 files, 1209 tests passed；Game Operator + Remote 相关 5 files, 104 tests passed。
- D:\dingsun\acp-ui\bin\hermes-game.exe: version 0.4.0，--no-tools/--prompt-file 已验证。
- 真实 Qwen 在线提案已通过严格解析并应用到 D 盘隔离副本。
- D:\dev-tools\godot\godot.exe: Godot 4.7.stable.official.5b4e0cb0f，真实 headless smoke 已通过。
- Godot validation runner、四类事件、超时/取消/截断/回收和生产/测试发现隔离已经完成，不要重复实现。

本轮目标：
把当前内存中的 Operator 控制面状态持久化到 D 盘版本化 SQLite，并在桌面进程重启后安全恢复任务、事件、审批、文件变更和 pending patch。恢复只能重建可观察状态，绝不能在启动时自动继续模型或 Godot 子进程；遗留执行态必须转为 paused/interrupted，等待操作员显式 resume。

实现顺序：
1. 先写规格，定义 schema version、恢复状态映射、事务边界、幂等规则和损坏数据库处理；不要边写表边猜语义。
2. 复用项目已有 rusqlite，数据库默认放在 D:\dingsun\acp-ui\.operator\operator-state.sqlite3；测试必须注入 D 盘临时路径，不能写用户真实库。
3. 至少持久化 tasks、append-only events、approvals、file_changes、pending_patches 以及 patch artifact/hash snapshot、pending validation 元数据、审批结果、backup_root 和 apply 结果。
4. 使用 schema_meta 或 PRAGMA user_version 做版本化迁移；初始化和迁移必须在事务中，重复启动幂等。
5. 提供 OperatorStateStore trait/边界，使业务状态机测试可使用内存或临时 SQLite；不要把 SQL 散落在每个 Tauri command 中。
6. 每个影响可恢复语义的状态变更必须在明确的持久化检查点落库。写失败不能假装成功，应返回错误并发出可审计失败事件；不要持久化 bearer token、API key 或模型密钥。
7. 启动恢复时校验数据库路径和项目路径均在 D 盘；终态原样恢复；planning/running/validating 等遗留执行态转为 paused/interrupted 并追加 recovery event；绝不自动 spawn Hermes/Godot。
8. 未决 plan/patch 审批必须可继续读取和决策。pending patch 恢复后必须重新执行路径、保护目录和 stale hash 校验，不能只相信数据库里的旧对象。
9. resume 必须沿现有共享 OperatorState 后台路径继续；Remote HTTP 与 Tauri 看到同一个恢复后的状态，不能建立第二份 mock 或缓存副本。
10. 写故障注入测试：首次建库、重复启动、迁移、重启恢复、running 转 paused、终态保持、审批恢复、pending patch stale、事务回滚、损坏/不兼容 schema、非 D 数据库拒绝、远程读取恢复状态。
11. 先完成最小纵向闭环再扩表：创建任务 -> 等待计划审批 -> 进程重启模拟 -> 状态/事件/审批恢复 -> 操作员继续审批。随后覆盖 patch approval 和 validation pending。
12. 完成后更新本文件和独立实现规格，列出准确 schema、恢复规则、命令、测试结果和仍未覆盖的 crash window。

每次修改后至少运行：
$env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
$env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Rust\.cargo'
$env:RUSTUP_HOME='D:\Rust\.rustup'
$env:RUSTFLAGS='-C force-unwind-tables'
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib operator::commands::tests -- --test-threads=1
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib operator::structured_patch::tests -- --test-threads=1
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib http_server::tests -- --test-threads=1
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib -- --test-threads=1

前端必须使用 D 盘 Node 直接运行 vue-tsc 与 Vitest。若新增 recovery UI/event，则至少覆盖 GameOperatorView、ProgressTimeline 和 operatorRemoteApi，最后跑全量 1209+ tests。

交付时必须报告：改了什么、schema 和迁移版本、恢复映射、真实命令、精确测试结果、是否使用临时 SQLite、模拟了哪些重启/crash window、仍有哪些边界。不要只回复“100% 完成”。
```

## 十一、下一位执行者先看这些文件

- `src-tauri/src/operator/structured_patch.rs`
- `src-tauri/src/operator/hermes_game_bridge.rs`
- `src-tauri/src/operator/hermes_process.rs`
- `src-tauri/src/operator/godot_validation.rs`
- `src-tauri/src/operator/persistence.rs`
- `src-tauri/src/operator/commands.rs`
- `src-tauri/src/operator/state_machine.rs`
- `src-tauri/src/operator/types.rs`
- `src-tauri/src/http_server.rs`
- `src/features/game-operator/components/ApprovalDrawer.vue`
- `src/features/game-operator/views/GameOperatorView.vue`
- `src/api/operatorRemoteApi.ts`
- `D:\dev-tools\hermes-game\src\main.rs`
- `D:\dev-tools\hermes-game\src\engine.rs`

最后提醒：当前最宝贵的不是功能数量，而是已经建立的“模型无写权、操作员可介入、差异可审、应用可验证、失败可追溯”的骨架。后续所有平台和领域都应该复用这条骨架。
