# 游戏开发 Agent 完整执行计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task-by-task. Every item uses checkbox tracking and must leave evidence.

**Goal:** 将当前已经能通过基础测试的 ACP-UI 修整为一个可暂停、可修改、可审批、可恢复、可远程操作并能真实驱动 Godot 项目的游戏开发 Agent。

**Architecture:** 以 Rust Operator Control Plane 作为任务事实源，以版本化 Operator API 连接 Web、Tauri、VSCode、IDEA 和远程客户端，以 Agent Runtime 执行 Skill/MCP/Hook，以 Godot Domain Pack 提供游戏领域能力。所有高风险动作经过 proposal、revision、approval、patch、validation 和 audit。

**Tech Stack:** Vue 3 + TypeScript + Vite + Vitest + Playwright；Tauri/Rust；Kotlin + IntelliJ Gradle Plugin；VSCode Extension API；Godot headless；HTTP + 可恢复事件流；JSON Schema；D 盘本地工具链。

---

## 使用方式

这不是按“阶段完成后宣布项目完成”的路线图，而是一份有依赖关系的完整执行契约。执行者可以并行准备独立测试和文档，但每个工作包必须在依赖满足后独立验收；不能因为某个工作包完成就宣称产品完成。

每个工作包都遵循：读取 -> 写失败测试 -> 实现 -> 最窄验证 -> 回归验证 -> 记录证据 -> 小提交。

## WP-00：建立执行基线

**Files:** `docs/codex/second/*`、`.gitignore`、`10-execution-ledger-template.md`

- [ ] 记录 `git status --short`、分支、HEAD、D 盘 Node/Java/Rust/Gradle/Godot/Hermes 路径。
- [ ] 读取 `01-current-state-and-acceptance.md`，把当前异常文件列入排除清单。
- [ ] 运行 Vue 类型检查、前端全量测试、Vite 构建、Rust lib 测试、VSCode pretest、IDEA wrapper build。
- [ ] 将每条命令的退出码和摘要写入执行台账。
- [ ] 若基线失败，先修复基线，不得直接开始跨模块重构。

## WP-01：修整仓库和生成物

**Files:** `.gitignore`、`.artifacts/`、根目录测试产物、`docs/codex/second/04-restructure-and-repository-governance.md`

- [ ] 生成 `.artifacts` 跟踪清单和总大小报告。
- [ ] 区分可保留的验收截图/报告与不可提交的缓存、驱动、target、appdata、日志。
- [ ] 不删除异常 FAQ 或用户修改；先将它们列入清单并在提交说明中排除。
- [ ] 更新 `.gitignore`，覆盖 Node、Rust、Gradle、Playwright、Tauri 和临时测试产物。
- [ ] 只删除确认的生成物，运行 `git diff --check` 和全量测试。
- [ ] 使用独立提交 `chore: remove generated acceptance artifacts`。

## WP-02：建立 Operator 协议唯一来源

**Files:** 新建 `schemas/operator/`、`src-tauri/src/operator/`、`src/api/`、两个客户端 API 文件

- [ ] 定义 Task、Event、Approval、PatchProposal、Checkpoint、ErrorResponse 的 JSON Schema。
- [ ] 给每个写请求增加 `revision` 或 `expected_revision` 语义，并为冲突写测试。
- [ ] 为状态、事件类型、风险等级和错误码建立枚举，不允许客户端依赖自由字符串。
- [ ] 生成或手工维护 TypeScript/Kotlin DTO，并用契约测试验证字段一致。
- [ ] 验证所有客户端都使用 `/api/operator/...` 和统一默认端口 `1422`。
- [ ] 新增接口必须先更新 schema 和 Rust DTO，再更新 Web、VSCode、IDEA。

## WP-03：完成任务控制面

**Files:** `src-tauri/src/operator/commands.rs`、`state_machine.rs`、`models.rs`、`events.rs`、`audit.rs`

- [ ] 为 `created`、`inspecting`、`planning`、`waiting_approval`、`executing`、`paused`、`validating`、`completed`、`failed`、`cancelled` 建立显式状态转移表。
- [ ] 写失败测试覆盖非法转移、终态控制、重复控制、并发控制和 revision 冲突。
- [ ] 实现暂停的协作式取消，确保工具在安全边界退出并保存 checkpoint。
- [ ] 实现停止的进程回收、状态终止和不可变审计。
- [ ] 让每个状态变化产生序号连续的事件。
- [ ] 为重启恢复写测试：加载快照、重放事件、恢复待审批和恢复事件游标。

## WP-04：实现可审阅补丁和审批闭环

**Files:** `src-tauri/src/operator/patches.rs`、`approvals.rs`、`backup.rs`、`ApprovalDrawer.vue`、相关客户端命令

- [ ] 定义 patch proposal：文件、old hash、new hash、diff、风险、验证命令、创建者和过期时间。
- [ ] 写失败测试证明没有审批时 high/critical patch 不能 apply。
- [ ] 实现 apply 前 hash 检查；外部修改返回 `PATCH_HASH_MISMATCH`。
- [ ] 实现 backup/checkpoint，apply 失败可恢复到 apply 前状态。
- [ ] 实现 approve、reject、request_changes 的状态和审计事件。
- [ ] 前端显示 diff、风险、文件、验证命令、审批人和结果，不只显示一个按钮。
- [ ] 在 Web、VSCode、IDEA 端验证审批 body 都包含 `task_id`、`approval_id` 和 decision。

## WP-05：实现 Skill、MCP、Hook 能力面

**Files:** 新建 `src-tauri/src/capabilities/`、`skills/`、`mcp/`、`hooks/`、协议 schema 和测试

- [ ] 定义能力 manifest、版本、输入输出 schema、风险、超时、可取消性和领域限制。
- [ ] 实现 Skill registry 的加载、校验、版本选择和禁用。
- [ ] 实现 MCP adapter 的连接、工具发现、调用超时、输出上限和错误归一化。
- [ ] 实现 Hook lifecycle、阻断/警告失败策略、超时和审计。
- [ ] 所有能力调用生成 `call_id` 和输入输出摘要。
- [ ] 写安全测试证明未知能力、越权领域、超时和超大输出会被拒绝。

## WP-06：完成 Sandbox、认证和远程安全

**Files:** `src-tauri/src/remote/`、`src-tauri/src/security/`、`src/api/`、远程配置文档

- [ ] 实现 canonical path boundary，覆盖 Windows、UNC、符号链接、大小写和短路径。
- [ ] 实现命令 allowlist、环境变量过滤、网络开关、资源限制和进程树回收。
- [ ] 实现短期 token、scope、撤销、角色和项目级授权。
- [ ] 写失败测试覆盖未授权、越权、重放审批、重复 apply、token 泄露和跨任务读取。
- [ ] 将 request id、actor id、来源、审计 id 和错误码写入服务端日志。
- [ ] 默认远程服务只监听回环地址；显式远程访问必须经过 TLS 或可信代理配置。

## WP-07：实现 Godot Domain Pack

**Files:** 新建 `src-tauri/src/domains/godot/`、`tests/fixtures/godot/`、Godot skills/MCP、领域测试

- [ ] 准备固定 D 盘 Godot fixture，包含 `project.godot`、主场景、玩家脚本和基线验证。
- [ ] 实现项目扫描、版本识别、规则读取、目录摘要和 Git 状态检查。
- [ ] 实现 GDScript/场景/资源的只读读取和结构化编辑能力。
- [ ] 实现二段跳等最小目标的计划生成、diff、审批、apply 和回滚。
- [ ] 实现 Godot headless 验证器，记录退出码、日志路径、耗时和摘要。
- [ ] 固定 Godot/Hermes 版本和 D 盘路径，缺失依赖时测试失败并说明缺什么。
- [ ] 完成一次真实领域闭环后，保存命令、diff、日志、截图和任务事件序列。

## WP-08：完成 Agent Runtime、记忆和恢复

**Files:** `src-tauri/src/runtime/`、memory/checkpoint 模块、前端 Memory/进度组件

- [ ] 将 planner、executor、tool runner、cancellation、checkpoint 拆成明确服务。
- [ ] 将项目事实、任务记忆、团队规则、临时上下文分开存储。
- [ ] 每条永久记忆绑定来源、可信度、作用域和过期策略。
- [ ] 实现事件序列恢复、任务 revision 校验和文件 hash 校验。
- [ ] 进程重启后恢复 waiting approval、paused 和 recovery_required 三种关键状态。
- [ ] 前端显示当前记忆摘要、来源和更新时间，禁止显示无法追溯的“模型认为”。

## WP-09：完成 Web/Tauri/VSCode/IDEA 客户端一致性

**Files:** `src/features/game-operator/`、`src-tauri/`、两个客户端目录

- [ ] Web 使用服务端 snapshot + event stream 驱动按钮和状态。
- [ ] Tauri 使用同一 Web API 和本地桥接，不复制任务状态机。
- [ ] VSCode 从 workspace 传递 Godot 项目路径，支持任务树、事件、审批和控制命令。
- [ ] IDEA 工具窗口和 Tools action 调用同一个 API client，支持刷新、审批和错误反馈。
- [ ] 每个客户端写契约测试，至少验证创建、暂停、恢复、停止和审批。
- [ ] 在真实 Extension Host、IDEA sandbox 和 Tauri driver 中各完成一条命令验收。

## WP-10：完成 13 种语言和 UI 质量

**Files:** `src/locales/types.ts`、13 个 locale、Game Operator 组件、locale tests

- [ ] 新 key 先加入类型，再补齐 13 个 locale。
- [ ] 加入 key 集合一致性测试、缺失 key 测试和硬编码用户文案扫描。
- [ ] 覆盖标题、状态、按钮、占位符、tooltip、aria-label、错误和空状态。
- [ ] 对中文、英文、葡萄牙文运行浏览器截图检查，确保长文本不溢出。
- [ ] 客户端错误分支只使用错误码，不使用英文文案匹配。

## WP-11：完成真实验收和发布纪律

**Files:** `tests/contract/`、`tests/e2e/`、`tests/fixtures/godot/`、执行台账

- [ ] 启动真实 Operator server，跑完整任务生命周期。
- [ ] 记录暂停、修改目标、恢复、审批拒绝、审批通过、验证失败、回滚和重启恢复。
- [ ] 跑 Web Playwright、Rust 集成、Godot fixture、VSCode Extension Host、IDEA sandbox、Tauri driver。
- [ ] 每个失败保留最小复现和日志，不删除失败证据来让目录变干净。
- [ ] 运行秘密扫描、生成物扫描、依赖审计和 `git diff --check`。
- [ ] 更新真实状态文档，不再使用未证实的 100% 文案。
- [ ] 每个独立工作包单独提交；最后提交只更新文档和证据索引。

## 交付判定

所有 WP 都有证据并不等于无限扩张的“平台完成”。第一版交付只判定 Godot 游戏开发闭环、可控 Agent Runtime、远程安全和四类客户端是否达到契约；运营、企业、视频和漫画继续作为未来领域包，不能混进本轮完成声明。
