# ACP-UI Pluggable Architecture Implementation Report

> 分支: `refactor/project-cleanup`
> 基于: 上一次重构提交 `cac9673`
> 日期: 2026-06-04

## 概述

本次实现基于对三个参考项目的深度研究（OpenClacky 省 Token、Ruflo Agent 蜂群、Claude Code Dynamic Workflows），结合对现有代码库 7 大能力域的审计，构建了 ACP-UI 的可插拔架构核心。

核心设计原则：**ACP 是控制与通讯协议，其他一切能力（Skill、MCP、Hook、CLI、Bot Adapter）均为可插拔插件。**

---

## 参考项目研究结论

### OpenClacky — 省 Token 七大策略

1. **双缓存标记**：滚动双缓冲实现接近 100% 缓存命中率
2. **冻结系统提示词**：系统提示词字节级冻结，动态信息走 session context 注入
3. **Insert-then-Compress**：压缩指令复用现有缓存，仅 ~500 token 冷数据
4. **16 个精简工具 + invoke_skill 元工具**：所有扩展能力走技能系统
5. **空闲自动压缩**：314 秒空闲后在缓存过期前主动压缩
6. **技能自进化**：执行后反思 + 复杂任务自动提取新技能
7. **Chunk 归档系统**：压缩前消息存为 .md 文件，AI 可按需回溯

### Ruflo — Agent 蜂群编排模式

- **双角色分离**：Claude Code = Orchestrator, Codex = Executor
- **MCP 作为统一通信总线**：300+ 工具通过 MCP 服务器暴露
- **共享记忆 + 隔离工作区**：Memory/AgentDB + Git worktree
- **自适应拓扑**：Hierarchical/Mesh/Ring/Star/Adaptive 五种
- **共识算法**：Raft/Byzantine/Gossip/CRDT/Quorum 五种
- **三层路由节省成本**：WASM 直接处理 → 小模型 → 大模型

### Claude Code Dynamic Workflows — Ultra Workflow

- **JavaScript 编排脚本**：编排逻辑代码化，可审查、可保存、可复跑
- **三层架构**：Claude 生成脚本 → Runtime 执行 → Subagents 工作
- **并行执行**：最多 16 个并发 agent，每次运行最多 1000 个
- **对抗性交叉验证**：多个 agent 互相审查结果
- **ultracode 模式**：xhigh 推理 + 自动工作流触发

---

## 现有代码基础审计

| 领域 | 后端成熟度 | 前端成熟度 | 可插拔就绪度 |
|------|-----------|-----------|-------------|
| ACP Protocol | N/A | **Production-Ready** | 高 |
| Skill System | Stub (mock) | Basic (UI) | 低 → **已升级** |
| MCP | Basic (进程管理) | 无 UI | 低 |
| Hooks | **Functional** | 无 UI | 中 |
| Self-Healing | **Functional** | **Functional** | 中 |
| Bot Adapters | Functional (3个) | 无 UI | 中 |
| Collaboration | Functional (DAG) | Basic (Store) | 中 |

---

## 新增实现

### 1. 插件注册表 (Plugin Registry)

**文件**: `src-tauri/src/plugin_registry.rs` (~520 行)

将所有能力建模为统一的 Plugin：
- 5 种 PluginKind: Skill / Mcp / Hook / Cli / Adapter
- PluginMeta 元数据：ID、版本、健康状态、能力列表、配置
- 9 个 Tauri 命令：list / register / unregister / get / set_enabled / update_config / search / get_stats / get_history
- `seed_defaults()` 自动注册 16 个核心技能 + 2 个 MCP 服务器
- 完整的执行审计和统计系统

### 2. Agent 蜂群编排器 (Swarm Orchestrator)

**文件**: `src-tauri/src/swarm_orchestrator.rs` (~680 行)

将 Codex 和 Claude Code 作为顶层对等 Agent 编排：
- 5 种拓扑: Hierarchical / Mesh / Pipeline / Star / Adaptive
- 5 种角色: Orchestrator / Executor / Specialist / Reviewer / Aggregator
- 5 种共识策略: FirstWins / Majority / BestScore / Merge / Adversarial
- Agent 预算管理：token_budget + tokens_used 追踪
- 自动淘汰机制：基于失败率驱逐低性能 Agent
- 7 个 Tauri 命令

### 3. Ultra Workflow 引擎

**文件**: `src-tauri/src/workflow_engine.rs` (~750 行)

复杂任务的多阶段编排：
- 5 种阶段策略: Parallel / Sequential / MapReduce / Competitive / AdversarialReview
- 4 种失败策略: StopAll / ContinueOthers / RetryWithBackoff / FallbackToManual
- DFS 环检测验证 DAG 合法性
- 模板化工作流生成：Research → Parallel+Review, Code → Competitive+Review, File → MapReduce
- 进度追踪和预估剩余时间
- 工作流保存/加载（可复跑）
- 10 个 Tauri 命令

### 4. 前端服务层

**目录**: `src/lib/plugin-system/` (4 个文件)

- `plugin-service.ts` — PluginService: 与 Rust 后端 plugin registry 通信
- `swarm-service.ts` — SwarmService: 与 swarm orchestrator 通信
- `workflow-service.ts` — WorkflowService: 与 workflow engine 通信
- `index.ts` — Barrel 导出

### 5. 前端状态管理

**文件**: `src/stores/plugin-registry.ts` + `src/stores/swarm.ts`

- Plugin Registry Store: 插件列表/过滤/搜索/健康统计/启停管理
- Swarm Store: Agent 列表/任务管理/健康监控

### 6. 架构设计文档

**文件**: `docs/pluggable-architecture-design.md`

完整的可插拔架构设计文档，包含：
- 架构图（6 层）
- Plugin Trait 设计
- Swarm Topology 设计
- Workflow Engine 设计
- Token Optimizer 设计（OpenClacky 模式）
- Self-Evolution 设计（Hook 自运行/检测/修复/思考）
- ACP 协议扩展点

---

## 文件变更汇总

### 新增文件 (10 个)

| 路径 | 行数 | 说明 |
|------|------|------|
| `src-tauri/src/plugin_registry.rs` | ~520 | 统一插件注册表 |
| `src-tauri/src/swarm_orchestrator.rs` | ~680 | Agent 蜂群编排器 |
| `src-tauri/src/workflow_engine.rs` | ~750 | Ultra Workflow 引擎 |
| `src/lib/plugin-system/plugin-service.ts` | ~100 | 插件前端服务 |
| `src/lib/plugin-system/swarm-service.ts` | ~100 | 蜂群前端服务 |
| `src/lib/plugin-system/workflow-service.ts` | ~120 | 工作流前端服务 |
| `src/lib/plugin-system/index.ts` | ~5 | Barrel 导出 |
| `src/stores/plugin-registry.ts` | ~100 | 插件 Pinia Store |
| `src/stores/swarm.ts` | ~60 | 蜂群 Pinia Store |
| `docs/pluggable-architecture-design.md` | ~180 | 架构设计文档 |

### 修改文件 (1 个)

| 路径 | 说明 |
|------|------|
| `src-tauri/src/lib.rs` | 新增 3 个 mod 声明 + AppState 3 个字段 + 26 个命令注册 + seed_defaults 调用 |

---

## 编译验证

| 检查项 | 结果 |
|--------|------|
| `cargo check` (Rust) | 0 错误, 31 警告 (30 既有 + 1 PluginMessage 未构造) |
| `npx vue-tsc --noEmit` (TypeScript strict) | 0 错误 |

---

## 后续集成路线

### 短期（下一阶段）

1. **Context Compression 实现**：基于 OpenClacky 的 Insert-then-Compress + 双缓存标记，增强现有的 `context-compactor.ts`
2. **Hook 集成 Agent Loop**：让 `hooks_executor.rs` 在 agent 执行 tool 时真正触发 Pre/Post hooks
3. **MCP JSON-RPC 客户端**：补全 `mcp_manager.rs` 的 JSON-RPC 通信层和工具发现
4. **Plugin Manager UI**：为 `plugin-registry.ts` store 创建 Vue 管理界面

### 中期

5. **Token Optimizer 前端实现**：Context Compression + Chunk Archiving + Idle Timer
6. **Self-Evolution 前后端统一**：将后端 EWMA 检测和前端 4 层闭环打通
7. **Feishu Webhook 接收端**：补全飞书适配器的消息接收能力
8. **Agent 间通信总线**：实现 agent-to-agent 消息路由

### 长期

9. **联邦通信**：跨机器 Agent 安全通信（mTLS + ed25519）
10. **工作流可视化编辑器**：Vue Flow 图形界面编辑 Workflow DAG
11. **对抗性审查集成**：在 Swarm 中启用 Adversarial 共识策略
12. **远程 Agent 管理 Dashboard**：类似 Qoder Mobile 的远程控制界面
