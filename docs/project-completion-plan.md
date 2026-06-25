# ACP-UI 项目完整恢复与实现计划

> **创建日期**: 2026-05-23
> **当前版本**: v0.1.14
> **当前分支**: my-agent-teams-platform
> **状态**: 待执行

---

## 一、项目定位

**ACP-UI** 是一个面向开发者的本地多 Agent 协作平台。

```
核心价值 =
  Claude Code 的沙箱能力 +
  Hermes 的智能调度 +
  Trae 的本地化版本 +
  多客户端开发工具支持
```

### 六大差异化优势

| 优势 | 说明 |
|------|------|
| 完全本地 | 隐私保护，无云端依赖 |
| 低延迟 | 局域网毫秒级响应 |
| 完全控制 | 自定义 MCP/Skills/Hooks |
| 多端协同 | 桌面 + 手机 + CLI 实时同步 |
| 多客户端 | HBuilderX / 微信小程序 / Android Studio / 浏览器 |
| 多 Agent 协作 | Claude Code + Codex + Gemini 同时工作 |

---

## 二、技术栈

### 2.1 三端架构

| 端 | 技术栈 | 状态 |
|----|--------|------|
| **Web/Tauri 桌面端** | Vue 3.5 + TypeScript 5.6 + Vite 6 + Tauri 2 + Rust | 主要开发 |
| **Flutter 移动端** | Flutter 3.22+ + Dart 3.8+ + Riverpod + go_router | 骨架阶段，大量问题 |
| **Rust 后端** | Tauri backend + SQLite + WebSocket + 7 Hermes crates | 部分实现 |

### 2.2 核心依赖

**前端** (package.json):
- `@agentclientprotocol/sdk: ^0.13.1`
- `vue: ^3.5.13`, `pinia: ^3.0.4`, `vue-i18n: ^10.0.7`
- `@vue-flow/core: ^1.48.2` (DAG 可视化)
- `@tauri-apps/*: ^2.x` (桌面端插件)
- 测试: vitest, playwright

**Rust 后端** (Cargo.toml):
- `tauri: ^2`, `rusqlite: ^0.32`, `tokio: ^1`, `tokio-tungstenite: ^0.26`
- `reqwest: ^0.12`, `serde_yaml: ^0.9`
- 7 个 Hermes 本地 crates (hermes-agent, hermes-config, hermes-core, hermes-tools, hermes-environments, hermes-skills, hermes-intelligence)

**Flutter 移动端** (pubspec.yaml):
- `flutter_riverpod: ^2.4.0`, `go_router: ^13.0.0`
- `web_socket_channel: ^2.4.0`, `sqflite: ^2.3.0`
- `flutter_markdown: ^0.6.0` (过旧)

---

## 三、项目结构

```
D:\dingsun\acp-ui\
├── src/                          # Vue 前端
│   ├── App.vue                   # 主组件 (1362行，过大)
│   ├── main.ts
│   ├── components/               # 50+ Vue 组件
│   │   ├── ChatView.vue
│   │   ├── MultiAgentChat.vue    # ⚠️ 假响应
│   │   ├── MultiSessionChat.vue
│   │   ├── WorkflowView.vue      # ⚠️ 固定示例数据
│   │   ├── TeamOrchestrationView.vue  # ⚠️ 空面板
│   │   ├── GatewaySettings.vue   # ⚠️ 未持久化
│   │   ├── BotSettings.vue       # ⚠️ 只监听事件
│   │   ├── HistoryView.vue
│   │   ├── MemoryView.vue
│   │   ├── SettingsView.vue
│   │   ├── HermesDashboard.vue
│   │   ├── PlanVisualization.vue
│   │   ├── agent-pet/            # Agent 宠物系统
│   │   ├── agent-progress/       # 实时进度监控
│   │   └── collaboration/        # 协作网络
│   ├── stores/                   # Pinia 状态管理
│   │   ├── session.ts            # ⚠️ 需迁移到 AcpSessionRunner
│   │   ├── multi-session.ts      # ⚠️ cwd 默认 "." 错误
│   │   ├── agent-pool.ts
│   │   └── ...
│   ├── lib/                      # 业务逻辑
│   │   ├── orchestrator.ts       # ⚠️ setTimeout 模拟执行
│   │   ├── multi-agent/bridge.ts # ⚠️ output 空字符串
│   │   ├── workflow/skill-engine.ts  # ⚠️ 模拟成功
│   │   ├── core/agent-teams-service.ts  # ⚠️ 返回空对象
│   │   ├── self-improvement/     # 自我修复
│   │   ├── agent-runtime/        # 统一运行时 (新增)
│   │   └── team-service/         # Team 服务 (新增)
│   └── ...
│
├── src-tauri/                    # Tauri + Rust 后端
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # ⚠️ 2072行，过大
│   │   ├── websocket.rs          # ⚠️ 1353行，过大
│   │   ├── database.rs           # ⚠️ 1220行，过大
│   │   ├── tunnel.rs             # ngrok 隧道
│   │   ├── permission_checker.rs
│   │   ├── agent_config_parser.rs
│   │   ├── mcp_manager.rs
│   │   ├── hooks_executor.rs
│   │   ├── log_stream.rs
│   │   └── hermes-crates/        # 7 个 Hermes crates (未纳入 git)
│   └── ...
│
├── acp_ui_flutter/               # Flutter 移动端
│   ├── pubspec.yaml
│   ├── lib/
│   │   ├── main.dart
│   │   ├── app.dart              # ⚠️ 首页直接跳 AgentTeams
│   │   ├── core/                 # 核心业务逻辑
│   │   │   ├── agent/            # Agent 管理
│   │   │   │   ├── agent_bridge.dart
│   │   │   │   ├── agent_pool.dart
│   │   │   │   └── process_stub.dart  # ⚠️ Web stub，Android 不可用
│   │   │   ├── session/session_manager.dart
│   │   │   ├── permission/permission_checker.dart
│   │   │   ├── transport/        # 传输层
│   │   │   │   ├── acp_transport.dart
│   │   │   │   └── websocket_transport.dart
│   │   │   ├── self_improvement/self_healing.dart
│   │   │   ├── gateway/im_gateway.dart
│   │   │   └── orchestrator/orchestrator.dart
│   │   ├── data/                 # 数据层
│   │   │   ├── models/           # 数据模型
│   │   │   ├── services/         # 服务
│   │   │   └── stores/           # Riverpod stores
│   │   ├── features/             # 功能页面
│   │   │   ├── chat/chat_view.dart
│   │   │   ├── multi_agent/multi_agent_view.dart
│   │   │   ├── history/history_view.dart
│   │   │   ├── settings/settings_view.dart
│   │   │   ├── evolution/evolution_dashboard.dart
│   │   │   ├── hermes/hermes_dashboard.dart
│   │   │   ├── collaboration/collaboration_network_view.dart
│   │   │   ├── agent_teams/agent_teams_dashboard.dart  # ⚠️ 1668行
│   │   │   ├── agent_pet/
│   │   │   └── agent_progress/
│   │   └── shared/               # 共享组件
│   │       ├── theme/app_theme.dart
│   │       └── widgets/sidebar.dart
│   └── android/                  # ⚠️ Java 8→21 升级，可能不兼容
│
├── docs/                         # 项目文档
│   ├── system-architecture.md    # 完整架构设计
│   ├── implementation-plan-phased.md  # 分阶段实施计划
│   ├── multi-agent-architecture-full.md
│   ├── flutter-migration-plan.md  # Flutter 迁移计划
│   ├── COLLABORATION-GUIDE.md    # 协作功能说明
│   ├── AGENT-TEAMS-USER-GUIDE.md # 用户使用手册
│   ├── TEST-PLAN.md
│   ├── TEST-REPORT.md
│   ├── QUICK-START.md
│   └── superpowers/plans/2026-05-04-agent-teams-platform-completion.md
│
├── _bmad-output/
│   └── planning-artifacts/agent-teams-platform-prd.md  # Agent Teams PRD
│
├── tests/                        # 测试
│   ├── e2e/                      # Playwright E2E
│   │   ├── agent-teams.spec.ts
│   │   └── collaboration.spec.ts
│   └── ...
│
└── .claude/                      # Claude Code 配置
    ├── plans/                    # 实施计划
    ├── skills/                   # Skills
    └── settings.local.json
```

---

## 四、文档索引

| 文档 | 路径 | 用途 | 状态 |
|------|------|------|------|
| 系统架构 | `docs/system-architecture.md` | 完整架构设计 + 详细组件说明 | ✅ 完成 |
| 分阶段计划 | `docs/implementation-plan-phased.md` | Phase 0-5 实施计划 + QA 验证 | ✅ 完成 |
| 多Agent架构 | `docs/multi-agent-architecture-full.md` | Agent 沙箱 + Hermes 调度 + QA Agents | ✅ 完成 |
| Flutter 迁移 | `docs/flutter-migration-plan.md` | Tauri → Flutter 迁移 5 阶段计划 | ✅ 完成 |
| Agent Teams PRD | `_bmad-output/.../agent-teams-platform-prd.md` | Agent Teams 产品设计 + 实现计划 | ✅ 完成 |
| Agent Teams 完成计划 | `docs/superpowers/plans/2026-05-04-agent-teams-platform-completion.md` | 12 个 Task 的实施计划 | ✅ 完成 |
| 协作指南 | `docs/COLLABORATION-GUIDE.md` | 协作网络可视化使用手册 | ✅ 完成 |
| 用户手册 | `docs/AGENT-TEAMS-USER-GUIDE.md` | 用户使用手册 | ✅ 完成 |
| **Agent Platform PRD** | `docs/AGENT-PLATFORM-PRD.md` | **统一 Agent 调度平台产品规划** | ✅ **新增** |
| **Agent Adapter 计划** | `docs/AGENT-ADAPTER-PLAN.md` | **Agent Adapter 实现计划** | ✅ **新增** |
| **Agent Platform 思考** | `docs/AGENT-PLATFORM-THINKING.md` | **Orca 学习 + API 调研反思** | ✅ **新增** |
| Loop Engine 架构 | `docs/loop-engine-architecture.md` | 4 层 Loop 系统架构设计 | ✅ 完成 |
| 测试计划 | `docs/TEST-PLAN.md` | 测试覆盖计划 | ⚠️ 需更新 |
| 测试报告 | `docs/TEST-REPORT.md` | 测试结果 | ⚠️ 需更新 |
| 快速开始 | `docs/QUICK-START.md` | 项目启动指南 | ⚠️ 需更新 |

---

## 五、当前问题清单

### 5.1 严重问题 (P0)

| # | 问题 | 位置 | 说明 |
|---|------|------|------|
| 1 | **功能全是"壳"** | 多处 | MultiAgentChat 假响应、WorkflowView 固定示例、Orchestrator setTimeout 模拟 |
| 2 | **Gateway Bot 未实现** | `lib.rs:1490-1531` | 飞书/Telegram/Discord 只打印日志 |
| 3 | **历史记录存不完整** | `history-store-tauri.ts:79` | `saveTask` 空函数 |
| 4 | **SQL 字符串拼接** | `lib.rs` | 内存查询存在 SQL 注入风险 |
| 5 | **超大文件** | `lib.rs:2072行`, `websocket.rs:1353行`, `database.rs:1220行` | 违反单一职责 |
| 6 | **Flutter Windows 被删** | `acp_ui_flutter/windows/` | 整个目录被删除 |
| 7 | **仓库被截图污染** | 根目录 50+ .png | 应加入 .gitignore |
| 8 | **53MB 二进制文件** | `cloudflared.exe` | 不应在仓库中 |

### 5.2 Flutter 端问题 (P0 - 重点)

| # | 问题 | 位置 | 说明 |
|---|------|------|------|
| F1 | **process_stub.dart 不可用** | `lib/core/agent/process_stub.dart` | Web 平台 stub，在 Android 上应该用 `dart:io` 的真实 Process，但当前代码在 Android 上也用了 stub |
| F2 | **首页直接跳 AgentTeams** | `lib/app.dart:25` | `/` 直接跳到 AgentTeamsDashboard，跳过了核心功能 |
| F3 | **依赖版本过旧** | `pubspec.yaml` | flutter_markdown 0.6.x, go_router 13.x |
| F4 | **Android Gradle Java 8→21** | `android/app/build.gradle` | 可能与其他依赖不兼容 |
| F5 | **缺少核心业务逻辑** | `lib/core/` | Agent 桥接、Session 管理等核心功能只有接口没有实现 |
| F6 | **WebSocket 无认证** | `websocket_transport.dart` | 没有 token 认证机制 |
| F7 | **l10n 生成文件在 git 中** | `lib/l10n/*.dart` | build_runner 生成文件不应被跟踪 |
| F8 | **Android Studio 构建错误** | `android/` | 依赖解析、Gradle 版本、Flutter SDK 版本可能不匹配 |
| F9 | **缺少 Flutter 测试** | `test/` | 没有单元测试和 widget 测试 |
| F10 | **空目录遗留** | `acp_ui_flutter_new/` | 开发过程残留 |

### 5.3 中等问题 (P1)

| # | 问题 | 说明 |
|---|------|------|
| 1 | TypeScript strict: false | 类型安全未启用 |
| 2 | console.log/println 大量存在 | 不应在生产代码中 |
| 3 | 硬编码路径 | `D:/dingsun/acp-ui/erp_system` |
| 4 | CSP 过于宽松 | unsafe-inline |
| 5 | 测试目录碎片化 | test_demo, test_erp 等 7 个 |
| 6 | Flutter 端与 Web 端功能不同步 | 部分功能只有 Web 端有 |

---

## 六、实施计划

### 总体时间线

```
2026-05-24 → 2026-05-25   Phase 0: 清理与基线
2026-05-26 → 2026-05-28   Phase 1: Flutter 端修复
2026-05-29 → 2026-06-02   Phase 2: 统一运行时 + 核心链路打通
2026-06-03 → 2026-06-05   Phase 3: Rust 后端修复 + 持久化
2026-06-06 → 2026-06-08   Phase 4: Gateway + Bot 实现
2026-06-09 → 2026-06-10   Phase 5: 测试覆盖 + 验收
```

---

### Phase 0: 清理与基线 (5 月 24 日 - 5 月 25 日)

**目标**: 清理混乱的工作区，建立干净的开发基线

#### Task 0.1: 清理工作区
- [ ] 更新 `.gitignore`，排除 `.png`, `.bat`, `.py`, `test_*` 目录
- [ ] 移动根目录截图到 `_screenshots/` 目录
- [ ] 移除 `cloudflared.exe` (53MB)
- [ ] 删除 `acp_ui_flutter_new/` 空目录
- [ ] 删除 `test_demo/`, `test_erp/`, `test_final/` 等临时目录

#### Task 0.2: 建立新分支
- [ ] 基于当前 `my-agent-teams-platform` 创建 `feat/platform-completion` 分支
- [ ] 记录当前所有未提交的改动清单

#### Task 0.3: 验证构建
- [ ] `npm run build` 必须通过
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml` 必须通过
- [ ] `cd acp_ui_flutter && flutter analyze` 记录当前错误

#### Task 0.4: 更新文档索引
- [ ] 确认所有 `docs/` 文档的最新状态
- [ ] 在每份文档顶部添加"最后更新"和"当前状态"标记

---

### Phase 1: Flutter 端修复 (5 月 26 日 - 5 月 28 日)

**目标**: 让 Flutter 端能在 Android Studio 中正常构建和运行

#### Task 1.1: 修复 Android 构建

**问题**: process_stub.dart 在 Android 上不可用，Gradle 版本不兼容

- [ ] **1.1.1** 修复 `process_stub.dart`
  - 创建条件编译：`dart:io` 在 mobile/desktop 使用真实 Process
  - Web 平台保留 stub
  - 使用 `universal_io` 或条件 import 实现跨平台

- [ ] **1.1.2** 修复 Gradle 兼容性
  - 检查当前 Flutter SDK 版本推荐的 Gradle 版本
  - 确认 Java 21 与所有依赖的兼容性
  - 如不兼容，回退到 Java 17 + Gradle 8.x

- [ ] **1.1.3** 恢复 Windows 平台
  - 从 `flutter create .` 重新生成 `windows/` 目录
  - 或者从 git 历史恢复被删除的文件

#### Task 1.2: 修复依赖版本

- [ ] 升级 `flutter_markdown` 到 `^0.7.x`
- [ ] 升级 `go_router` 到 `^14.x`
- [ ] 升级 `web_socket_channel` 到 `^3.x`
- [ ] 升级 `intl` 到最新版本
- [ ] 运行 `flutter pub deps` 确认无冲突

#### Task 1.3: 修复路由和首页

- [ ] 修改 `app.dart` 路由
  - `/` 改为显示功能选择页或 Dashboard 总览
  - 不再直接跳 AgentTeamsDashboard
  - 添加缺失的 `/chat`, `/status`, `/monitor` 路由

#### Task 1.4: 补充 Flutter 核心业务逻辑

- [ ] **1.4.1** 实现 `agent_bridge.dart`
  - 真实的 ACP 客户端连接
  - 支持 stdio (本地 Agent) 和 WebSocket (远程 Agent)
  - session 创建、prompt、cancel 完整生命周期

- [ ] **1.4.2** 实现 `session_manager.dart`
  - 多 Session 管理
  - cwd 必须为绝对路径
  - 断线重连

- [ ] **1.4.3** 实现 `orchestrator.dart`
  - 替代当前的空实现
  - 任务分配、依赖检查、并行执行
  - 事件总线

- [ ] **1.4.4** 实现 `im_gateway.dart`
  - WebSocket 连接到桌面端 Gateway
  - token 认证
  - 命令发送和响应接收

#### Task 1.5: WebSocket 认证

- [ ] 修改 `websocket_transport.dart`
  - 连接时发送认证 token
  - 支持 JSON-RPC 请求-响应模式
  - 超时处理和重连

#### Task 1.6: Flutter 测试

- [ ] **1.6.1** 创建测试基础设施
  - `test/` 目录结构
  - mock 数据工厂

- [ ] **1.6.2** 编写单元测试
  - `websocket_transport_test.dart`
  - `permission_checker_test.dart`
  - `agent_bridge_test.dart` (用 fake transport)

- [ ] **1.6.3** 编写 widget 测试
  - `chat_view_test.dart`
  - `sidebar_test.dart`
  - `agent_teams_dashboard_test.dart` (空状态测试)

- [ ] **1.6.4** 验证
  - `flutter test` 必须通过
  - `flutter analyze` 无 error

#### Task 1.7: 清理 Flutter git 追踪

- [ ] 从 `.gitignore` 排除 `lib/l10n/*.dart` (build_runner 生成)
- [ ] 从 `.gitignore` 排除 `android/.gradle/`, `build/`, `.dart_tool/`
- [ ] 从 git 中移除已追踪的生成文件

**验收标准**:
- `flutter analyze` 无 error
- `flutter test` 通过
- Android Studio 能正常构建和运行到模拟器
- 应用能连接桌面端 WebSocket 并显示 Agent 状态

---

### Phase 2: 统一运行时 + 核心链路打通 (5 月 29 日 - 6 月 2 日)

**目标**: 消灭所有"假数据"和"模拟响应"，建立真实的执行链路

> 按 `docs/superpowers/plans/2026-05-04-agent-teams-platform-completion.md` 的 Task 1-6 执行

#### Task 2.1: 定义统一运行时契约 (对应原计划 Task 1)

- [ ] 创建 `src/lib/agent-runtime/types.ts`
- [ ] 创建 `src/lib/agent-runtime/runtime-errors.ts`
- [ ] 创建 `src/lib/agent-runtime/output-buffer.ts`
- [ ] 编写 `output-buffer.test.ts`
- [ ] 编写 `acp-session-runner.test.ts`

#### Task 2.2: 封装真实 ACP Session Runner (对应原计划 Task 2)

- [ ] 实现 `src/lib/agent-runtime/acp-session-runner.ts`
- [ ] 迁移 `src/stores/session.ts` 到使用 Runner
- [ ] 验证单会话行为

#### Task 2.3: 修复总会话 (对应原计划 Task 3)

- [ ] 创建 `src/components/multi-session/NewSessionDialog.vue`
- [ ] 修改 `SessionTabs.vue` (不再默认 `.`)
- [ ] 修改 `multi-session.ts` (使用 AcpSessionRunner)
- [ ] 编写 `multi-session.test.ts`

#### Task 2.4: 建立真实 Team Runtime (对应原计划 Task 4)

- [ ] 创建 `src/lib/team-service/types.ts`
- [ ] 创建 `src/lib/team-service/agent-teams-service.ts`
- [ ] 创建 `src/stores/team-runtime.ts`
- [ ] 修改 `MultiAgentChat.vue` (删除假响应)
- [ ] 修改 `AgentStatusPanel.vue`

#### Task 2.5: 工作流改为真实定义 (对应原计划 Task 5)

- [ ] 创建 `src/stores/workflows.ts`
- [ ] 创建 `src/components/workflows/WorkflowEditor.vue`
- [ ] 创建 `src/components/workflows/WorkflowRunPanel.vue`
- [ ] 修改 `WorkflowView.vue` (删除固定示例)
- [ ] 修改 `skill-engine.ts` (真实调用)

#### Task 2.6: 编排监控接入真实计划 (对应原计划 Task 6)

- [ ] 修改 `src/lib/orchestration/types.ts`
- [ ] 修改 `src/lib/orchestration/orchestrator.ts` (修复并行逻辑)
- [ ] 修改 `TeamOrchestrationView.vue`
- [ ] 修改 `PlanVisualization.vue` (改为受控组件)

---

### Phase 3: Rust 后端修复 + 持久化 (6 月 3 日 - 6 月 5 日)

**目标**: 修复 SQL 安全、拆分大文件、完成持久化

> 对应原计划 Task 7

#### Task 3.1: SQL 安全修复

- [ ] 修复 `lib.rs` 中所有 SQL 字符串拼接
- [ ] 全部改为 `rusqlite::params!` 参数绑定
- [ ] 运行 `Select-String` 验证无拼接残留

#### Task 3.2: 扩展 SQLite 表

- [ ] 增加 `workflows` 表
- [ ] 增加 `execution_plans` 表
- [ ] 增加 `runtime_events` 表
- [ ] 增加 `gateway_config` 表
- [ ] 增加 `memories.scope` 字段

#### Task 3.3: 实现 gateway_config 模块

- [ ] 创建 `src-tauri/src/gateway_config.rs`
- [ ] 创建 `src-tauri/src/security.rs`
- [ ] 配置持久化 + 敏感字段遮蔽
- [ ] 接入 `lib.rs` commands

#### Task 3.4: 拆分大文件

- [ ] `lib.rs` (2072行) → 拆分为 `commands/` 目录
- [ ] `websocket.rs` (1353行) → 拆分为认证/命令路由/连接管理
- [ ] `database.rs` (1220行) → 拆分为各表的 CRUD 模块

#### Task 3.5: Hermes crates 纳入版本控制

- [ ] 确认 `hermes-crates/` 的来源
- [ ] 如果是项目核心依赖，纳入 git
- [ ] 如果是外部依赖，改为 crate.io 或 git submodule

---

### Phase 4: Gateway + Bot 实现 (6 月 6 日 - 6 月 8 日)

**目标**: 远程控制和 Bot 配置做成真实服务

> 对应原计划 Task 8-9

#### Task 4.1: 远程控制 Gateway

- [ ] 创建 `src-tauri/src/gateway_server.rs`
- [ ] 定义远程协议 (command/request/response)
- [ ] 实现 token 认证
- [ ] 支持命令: get_status, list_agents, pause_agent, resume_agent, cancel_agent
- [ ] 每个命令必须有响应回执

#### Task 4.2: Bot 适配器

- [ ] 创建 `src-tauri/src/bot_adapters/mod.rs`
- [ ] 创建 `src-tauri/src/bot_adapters/app_ws.rs` (App WebSocket)
- [ ] 创建 `src-tauri/src/bot_adapters/telegram.rs` (Telegram long polling)
- [ ] 修改 `bot.rs` 命令接入 TeamOrchestrator
- [ ] Bot 命令返回真实状态，不发假事件

#### Task 4.3: 飞书/Telegram/Discord 真实接入

- [ ] 实现飞书 Bot (`feishu.rs`)
- [ ] 实现 Telegram Bot (已在 4.2)
- [ ] 实现 Discord Bot (`discord.rs`)
- [ ] 消息卡片生成
- [ ] 会话跨渠道同步

#### Task 4.4: BotSettings 和 GatewaySettings 改造

- [ ] 修改 `BotSettings.vue` (显示真实状态 + 测试指令)
- [ ] 修改 `GatewaySettings.vue` (使用 gateway store)

---

### Phase 5: 记忆系统 + UI 收敛 + 验收 (6 月 9 日 - 6 月 10 日)

**目标**: 完成最后的功能闭环，全面验收

#### Task 5.1: 记忆系统 (对应原计划 Task 10)

- [ ] 扩展 memories 表 (scope, task_id)
- [ ] 修改 memory store 支持 scope 筛选
- [ ] 任务发送前注入相关记忆
- [ ] MemoryView 增强

#### Task 5.2: UI 收敛 (对应原计划 Task 11)

- [ ] 创建 `src/lib/feature-registry.ts`
- [ ] 修改 `App.vue` 使用注册表
- [ ] 统一视觉密度
- [ ] 消除"廉价感"

#### Task 5.3: 端到端验收 (对应原计划 Task 12)

- [ ] 扩展 Playwright 测试
- [ ] 运行完整验证命令
- [ ] 编写验证文档
- [ ] 截图留存

---

## 七、测试计划

### 7.1 测试矩阵

| 层级 | 框架 | 目标覆盖率 | 状态 |
|------|------|-----------|------|
| Vue 单元测试 | Vitest | 80%+ | ⚠️ 不足 |
| Rust 单元测试 | cargo test | 60%+ | ❌ 没有 |
| Vue E2E | Playwright | 关键路径 | ⚠️ 骨架 |
| Flutter 单元测试 | flutter test | 70%+ | ❌ 没有 |
| Flutter Widget 测试 | flutter_test | 关键组件 | ❌ 没有 |
| Flutter 集成测试 | integration_test | 核心流程 | ❌ 没有 |

### 7.2 关键测试用例

**Vue 前端**:
1. OutputBuffer: assistant chunk, thought chunk, tool call 汇总
2. AcpSessionRunner: new session, load session, prompt, cancel
3. Multi-session cwd validation: 绝对路径检查
4. Team Runtime: 广播、路由、负载均衡模式

**Rust 后端**:
1. Gateway config: 保存、加载、敏感字段遮蔽
2. SQL 参数绑定: 无拼接残留
3. Token 认证: 生成、hash 比较

**Flutter 移动端**:
1. WebSocket Transport: 连接、断开、重连、RPC
2. Permission Checker: 允许/拒绝规则
3. Agent Bridge: session 创建、prompt
4. ChatView: 空状态、消息列表、发送

### 7.3 验证命令

```bash
# 前端
npm run build
npm run test -- --run
npx playwright test

# Rust
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# Flutter
cd acp_ui_flutter
flutter pub get
flutter analyze
flutter test
```

---

## 八、验收标准

### 8.1 功能验收

| 功能 | 标准 | 状态 |
|------|------|------|
| 多 Agent | 真实调用多个 Agent，展示各自的输出/错误/耗时 | ❌ |
| 工作流 | 可新建、保存、运行、取消，无固定示例 | ❌ |
| 总会话 | 显式选择 Agent 和绝对 cwd，不默认 "." | ❌ |
| 编排监控 | 显示真实计划和节点状态 | ❌ |
| Bot 配置 | 持久化，测试指令返回真实状态 | ❌ |
| 远程控制 | WebSocket 带 token，远程命令有回执 | ❌ |
| 记忆 | 支持 scope，任务发送前注入 | ❌ |
| Flutter 端 | Android Studio 能构建运行，核心链路打通 | ❌ |

### 8.2 工程验收

| 检查项 | 标准 | 状态 |
|--------|------|------|
| npm run build | PASS | ❌ |
| npm run test | PASS | ❌ |
| cargo check | PASS | ❌ |
| cargo test | PASS | ❌ |
| flutter analyze | 无 error | ❌ |
| flutter test | PASS | ❌ |
| playwright test | PASS | ❌ |

### 8.3 安全验收

| 检查项 | 标准 | 状态 |
|--------|------|------|
| SQL 参数绑定 | 无拼接 | ❌ |
| Token 不明文打印 | 只存 hash | ❌ |
| 无硬编码密钥 | grep 验证 | ❌ |
| CSP 收紧 | 移除 unsafe-inline | ❌ |

---

## 九、回滚方案

如果实施过程中出现主会话不可用：

1. 保留 `src/lib/agent-runtime`，但把 `src/stores/session.ts` 回退
2. 高级功能入口保留，但禁用运行按钮并显示真实错误
3. 验证 `npm run build` 和 `cargo check` 通过
4. 提交回滚

---

## 十、风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| Flutter 端修复时间超预期 | 延期 | 先保证 Web 端核心链路，Flutter 端降级为纯展示 |
| Rust 大文件拆分引入 bug | 功能回归 | 拆分前先跑测试，每次拆分后验证 |
| Hermes crates 来源不明 | 构建失败 | 优先确认 crates 状态，必要时替换为 crate.io 版本 |
| 测试覆盖率达不到 80% | 质量不达标 | 先写关键路径测试，逐步提升 |

---

## 十一、交付文件

计划完成后应交付：

1. `docs/implementation-artifacts/agent-teams-verification.md` — 验证文档
2. `docs/test-report-updated.md` — 更新后的测试报告
3. `docs/CHANGELOG.md` — 变更日志
4. `docs/flutter-status.md` — Flutter 端状态说明
5. 所有测试通过截图
6. 构建通过截图

---

## 十二、实施进度跟踪

> **更新日期**: 2026-05-24
> **执行状态**: ✅ Phase 0-9 已完成

### 阶段完成状态

| Phase | 名称 | 状态 | 提交 | 说明 |
|-------|------|------|------|------|
| Phase 0 | Hermes Crates 链接 | ✅ 完成 | feb8407 | 8个 Hermes crates 已链接到 Cargo.toml |
| Phase 1 | SQLite + hermes-memory | ✅ 完成 | 9ad3e2e | 16个新表 + FTS5 + Chroma hybrid |
| Phase 2 | Agent Registry | ✅ 完成 | a76a6ac | Docker-like Base/Template/Instance |
| Phase 3 | Smart Router | ✅ 完成 | 9502f81 | 三层渐进复杂度评估 |
| Phase 4 | Circuit Breaker | ✅ 完成 | 9502f81 | 三态熔断器 Closed→Open→HalfOpen |
| Phase 5 | Self-Healing | ✅ 完成 | 9502f81 | EWMA 动态基线 + 异常检测 |
| Phase 6 | Team DAG | ✅ 完成 | 62fca06 | DAG 执行引擎 + SyncPoints |
| Phase 7 | Hermes Memory Crate | ✅ 完成 | b869e8f | SQLite + Chroma hybrid search |
| Phase 8 | Frontend Build | ✅ 完成 | - | Vue 前端构建成功 |
| Phase 9 | Unit Tests | ✅ 完成 | 0f65f30 | 14 个单元测试全部通过 |
| Phase 10 | Tauri Commands | ✅ 完成 | 5972258 | 9个新命令暴露给前端 |
| Phase 11 | Playwright Config | ✅ 完成 | b3d8a9c | Windows 环境适配 |

### 核心模块文件

| 模块 | 文件路径 | 状态 |
|------|---------|------|
| Agent Registry | `src-tauri/src/agent_registry.rs` | ✅ |
| Smart Router | `src-tauri/src/smart_router.rs` | ✅ |
| Circuit Breaker | `src-tauri/src/circuit_breaker.rs` | ✅ |
| Self-Healing | `src-tauri/src/self_healing.rs` | ✅ |
| Team DAG | `src-tauri/src/team_dag.rs` | ✅ |
| Hermes Memory | `src-tauri/hermes-crates/hermes-memory/` | ✅ |
| Database Schema | `src-tauri/src/database.rs` | ✅ 16表已添加 |
| Executive Agent | `src-tauri/src/executive_agent.rs` | ✅ Hermes Native |

### 测试结果

```
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- config::tests::defaults_keep_all_eleven_stdio_agents ✅
- event_router::tests::test_event_routing ✅
- event_router::tests::test_shared_router ✅
- event_router::tests::test_queue_overflow ✅
- permission_checker::tests::test_dangerous_command_detection ✅
- permission_checker::tests::test_permission_modes ✅
- permission_checker::tests::test_path_restriction ✅
- permission_checker::tests::test_allow_deny_override ✅
- session_manager::tests::test_branch_lock_collision ✅
- session_manager::tests::test_session_compaction ✅

### 下一步

架构优化核心模块已完成。后续可选：
1. 添加集成测试验证 Rust + Vue 联动
2. 完善 E2E 测试（Playwright）- 需手动启动 dev server
3. Flutter 移动端修复（独立任务）

### 新增 Tauri 命令

| 命令 | 功能 |
|------|------|
| `analyze_task_complexity` | Smart Router 任务复杂度分析 |
| `get_circuit_breaker_status` | 熔断器状态查询 |
| `is_circuit_breaker_allowed` | 检查请求是否允许 |
| `reset_circuit_breaker` | 重置熔断器 |
| `get_all_circuit_breakers` | 所有熔断器状态 |
| `create_dag_plan` | DAG 执行计划创建 |
| `get_dag_plan_progress` | 计划进度跟踪 |
| `check_anomaly` | 异常检测检查 |
| `update_anomaly_baseline` | 基线更新 |

### 前端集成状态

- `src/lib/team-service/agent-teams-service.ts` ✅ 完整实现
- `src/lib/team-service/types.ts` ✅ 类型定义
- `src/views/AgentTeamsDashboard.vue` ✅ 界面组件
- `src/components/HermesDashboard.vue` ✅ 监控面板
- `src/components/EnhancedHermesDashboard.vue` ✅ 增强面板
