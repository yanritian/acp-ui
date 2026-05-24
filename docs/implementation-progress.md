# ACP-UI 架构优化实施进度 - 修订版

> **修订日期**: 2026-05-24
> **状态分析**: 基于 `docs/current-status-analysis.md`
> **用途**: 详细追踪每个任务的完成状态和产出

---

## 一、进度总览

```
总进度: 0/38 (0%) - 修订后减少15个任务
├── Phase 0: 0/3 (0%) - 构建修复 (Cargo链接 + 类型定义)
├── Phase 1: 0/6 (0%) - 基础设施补全 (16新表 + Chroma)
├── Phase 2: 0/5 (0%) - Agent Registry (复用7 crates)
├── Phase 3: 0/4 (0%) - 记忆系统增强 (复用memories表)
├── Phase 4: 0/5 (0%) - 智能路由 (复用orchestrator)
├── Phase 5: 0/3 (0%) - 自愈系统增强 (复用errors表)
├── Phase 6: 0/4 (0%) - Team编排增强 (复用TeamView)
├── Phase 7: 0/3 (0%) - Hermes集成 (复用32文件)
├── Phase 8: 0/5 (0%) - 前端对接 (复用35+组件)
└── Phase 9: 0/3 (0%) - 测试验收 (复用E2E骨架)
```

---

## 二、已完成基础 ✅ (无需重做)

### 2.1 SQLite 表 (已有 14 个)

| 表名 | 用途 | 状态 | 备注 |
|------|------|------|------|
| tasks | 任务历史 | ✅ 已有 | 无需重做 |
| agent_executions | Agent执行记录 | ✅ 已有 | 无需重做 |
| sessions | 多会话管理 | ✅ 已有 | 无需重做 |
| memories | 记忆系统 | ✅ 已有 | 已有scope字段 |
| gateway_config | 远程控制 | ✅ 已有 | 无需重做 |
| workflows | 工作流定义 | ✅ 已有 | 无需重做 |
| execution_plans | 编排计划 | ✅ 已有 | 无需重做 |
| runtime_events | 事件日志 | ✅ 已有 | 无需重做 |
| errors | 错误记录 | ✅ 已有 | 无需重做 |
| solutions | 解决方案 | ✅ 已有 | 无需重做 |
| evolutions | 自进化记录 | ✅ 已有 | 无需重做 |
| patterns | 模式库 | ✅ 已有 | 无需重做 |

### 2.2 Hermes Crates (已有 7 个)

| Crate | 文件数 | 状态 | 备注 |
|-------|--------|------|------|
| hermes-agent | 32个 | ✅ 已有 | agent_loop, memory_manager, sub_agent_orchestrator |
| hermes-config | - | ✅ 已有 | 配置加载 |
| hermes-core | - | ✅ 已有 | 核心类型 |
| hermes-tools | - | ✅ 已有 | 工具注册 |
| hermes-environments | - | ✅ 已有 | 环境管理 |
| hermes-skills | - | ✅ 已有 | Skills |
| hermes-intelligence | - | ✅ 已有 | 智能模块 |

### 2.3 Rust 后端 (已有 17 个文件)

| 文件 | 行数 | 状态 | 备注 |
|------|------|------|------|
| lib.rs | 69KB | ✅ 已有 | 主命令 |
| database.rs | 32KB | ⚠️ 需修复 | 缺ThinkingChunkRecord/ToolCallRecord |
| websocket.rs | 39KB | ✅ 已有 | WebSocket |
| executive_agent.rs | 17KB | ⚠️ 需修复 | Cargo链接问题 |
| agent.rs | 12KB | ✅ 已有 | Agent进程 |
| hooks_executor.rs | 20KB | ✅ 已有 | Hooks执行 |
| permission_checker.rs | 19KB | ✅ 已有 | 权限检查 |
| mcp_manager.rs | 8KB | ✅ 已有 | MCP管理 |
| log_stream.rs | 20KB | ✅ 已有 | 日志流 |
| session_manager.rs | 12KB | ✅ 已有 | Session管理 |
| tunnel.rs | 8KB | ✅ 已有 | 隧道 |
| event_router.rs | 10KB | ✅ 已有 | 事件路由 |

### 2.4 Vue 组件 (已有 35+ 个)

| 组件 | 状态 | 备注 |
|------|------|------|
| HermesDashboard.vue | ✅ 已有 | 无需重做 |
| EnhancedHermesDashboard.vue | ✅ 已有 | 无需重做 |
| MemoryView.vue | ✅ 已有 | 无需重做 |
| HistoryView.vue | ✅ 已有 | 无需重做 |
| TeamOrchestrationView.vue | ⚠️ 假响应 | 需修复 |
| WorkflowView.vue | ⚠️ 固定示例 | 需修复 |
| TaskGraphView.vue | ✅ 已有 | 无需重做 |
| LogStreamView.vue | ✅ 已有 | 无需重做 |
| EvolutionView.vue | ✅ 已有 | 无需重做 |
| PatternView.vue | ✅ 已有 | 无需重做 |
| agent-pet/* | ✅ 已有 | 无需重做 |
| agent-progress/* | ✅ 已有 | 无需重做 |
| collaboration/* | ✅ 已有 | 无需重做 |
| multi-session/* | ✅ 已有 | 无需重做 |

### 2.5 TypeScript lib (已有 30+ 个)

| 模块 | 状态 | 备注 |
|------|------|------|
| orchestrator.ts | ⚠️ setTimeout模拟 | 需修复 |
| task-parser.ts | ✅ 已有 | 无需重做 |
| agent-matcher.ts | ✅ 已有 | 无需重做 |
| browser-adapter.ts | ✅ 已有 | 无需重做 |
| hbuilderx-adapter.ts | ✅ 已有 | 无需重做 |
| wechat-devtools-adapter.ts | ✅ 已有 | 无需重做 |
| android-studio-adapter.ts | ✅ 已有 | 无需重做 |
| code-review-agent.ts | ✅ 已有 | 无需重做 |
| test-validator-agent.ts | ✅ 已有 | 无需重做 |
| skills-loader.ts | ✅ 已有 | 无需重做 |
| offline-cache.ts | ✅ 已有 | 无需重做 |
| command-queue.ts | ✅ 已有 | 无需重做 |

---

## 三、Phase 0 进度: 构建修复

**状态**: ⏳ 待开始
**核心任务**: 修复 Cargo.toml + database.rs 类型定义

### P0-001: 修复 Cargo.toml

| 子任务 | 状态 | 产出 | 备注 |
|--------|------|------|------|
| P0-001.1 添加 Hermes crates 依赖 | ⏳ | src-tauri/Cargo.toml | 7个 crate 链接 |
| P0-001.2 验证 Rust 编译 | ⏳ | - | cargo check |

### P0-002: 修复 database.rs

| 子任务 | 状态 | 产出 | 备注 |
|--------|------|------|------|
| P0-002.1 添加 ThinkingChunkRecord | ⏳ | database.rs | 结构体定义 |
| P0-002.2 添加 ToolCallRecord | ⏳ | database.rs | 结构体定义 |

### P0-003: 验证构建

| 子任务 | 状态 | 产出 | 备注 |
|--------|------|------|------|
| P0-003.1 cargo check | ⏳ | - | 验收: PASS |
| P0-003.2 npm run build | ⏳ | - | 验收: PASS |
| P0-003.3 git commit | ⏳ | - | 提交修复 |

---

## 四、Phase 1 进度: 基础设施补全

**状态**: ⏳ 待开始
**已有基础**: 14 个表已存在

### P1-001: 新增 16 个 SQLite 表

| 表名 | 状态 | 备注 |
|------|------|------|
| agent_bases | ⏳ | Agent基础镜像 |
| agent_templates | ⏳ | Agent配置模板 |
| agent_instances | ⏳ | 运行实例 |
| teams | ⏳ | 团队定义 |
| team_executions | ⏳ | 团队执行记录 |
| route_decisions | ⏳ | 路由决策 |
| input_logs | ⏳ | 输入日志 |
| agent_flows | ⏳ | Agent执行流 |
| flow_steps | ⏳ | 时序步骤 |
| tool_calls | ⏳ | 工具调用 |
| anomalies | ⏳ | 异常记录 |
| circuit_breakers | ⏳ | 断路器 |
| healing_actions | ⏳ | 恢复动作 |
| skills | ⏳ | Skill |
| knowledge_nodes | ⏳ | 知识库 |
| audit_log | ⏳ | 审计日志 |

### P1-002: WAL + FTS5

| 子任务 | 状态 | 备注 |
|--------|------|------|
| P1-002.1 WAL 模式 | ⏳ | 可能已启用 |
| P1-002.2 FTS5 虚拟表 | ⏳ | memories_fts |

### P1-003: Chroma 向量数据库

| 子任务 | 状态 | 产出 | 备注 |
|--------|------|------|------|
| P1-003.1 hermes-memory crate | ⏳ | hermes-memory/ | 新建 |
| P1-003.2 Embedding 层 | ⏳ | embedding.rs | L1+L2 |

### P1-004: 验证

| 子任务 | 状态 | 备注 |
|--------|------|------|
| P1-004.1 新表验证 | ⏳ | sqlite3 .tables |
| P1-004.2 FTS5 验证 | ⏳ | 全文搜索 |
| P1-004.3 git commit | ⏳ | 提交 |

---

## 五、Phase 2-9 进度概览

### Phase 2: Agent Registry (复用7 crates)

| 任务 | 状态 | 备注 |
|------|------|------|
| P2-001 Cargo链接 | ⏳ | Phase 0 已完成 |
| P2-002 agent_registry.rs | ⏳ | 新建 |
| P2-003 配置示例 | ⏳ | YAML文件 |
| P2-004 hermes-orchestration | ⏳ | 新 crate |
| P2-005 验证 | ⏳ | - |

### Phase 3: 记忆系统增强 (复用memories表)

| 任务 | 状态 | 备注 |
|------|------|------|
| P3-001 混合检索 | ⏳ | FTS+Vector |
| P3-002 记忆注入 | ⏳ | 新模块 |
| P3-003 输入层记录 | ⏳ | input_logs表 |
| P3-004 Agent流程记录 | ⏳ | agent_flows表 |

### Phase 4: 智能路由 (复用orchestrator)

| 任务 | 状态 | 备注 |
|------|------|------|
| P4-001 复用已有 | ✅ | task-parser, agent-matcher已有 |
| P4-002 smart_router.rs | ⏳ | 新建 |
| P4-003 route_decisions表 | ⏳ | Phase 1 新增 |
| P4-004 修复假响应 | ⏳ | orchestrator.ts |
| P4-005 验证 | ⏳ | - |

### Phase 5: 自愈系统增强 (复用errors表)

| 任务 | 状态 | 备注 |
|------|------|------|
| P5-001 复用已有 | ✅ | errors, solutions, patterns已有 |
| P5-002 EWMA模块 | ⏳ | anomaly_detector.rs |
| P5-003 Skill验证 | ⏳ | skill_validator.rs |

### Phase 6: Team编排增强 (复用TeamView)

| 任务 | 状态 | 备注 |
|------|------|------|
| P6-001 DAG引擎 | ⏳ | dag_engine.rs |
| P6-002 EventBus扩展 | ⏳ | event_router已有 |
| P6-003 Team记录 | ⏳ | teams表 |
| P6-004 修复假响应 | ⏳ | TeamOrchestrationView |

### Phase 7: Hermes集成 (复用32文件)

| 任务 | 状态 | 备注 |
|------|------|------|
| P7-001 复用已有 | ✅ | agent_loop, memory_manager已有 |
| P7-002 集成新模块 | ⏳ | 3个新crate |
| P7-003 验证 | ⏳ | - |

### Phase 8: 前端对接 (复用35+组件)

| 任务 | 状态 | 备注 |
|------|------|------|
| P8-001 Agent Registry UI | ⏳ | 4个新组件 |
| P8-002 其他UI | ⏳ | 3个新组件 |
| P8-003 修复假响应 | ⏳ | 4个组件需修复 |
| P8-004 Dashboard集成 | ⏳ | HermesDashboard |
| P8-005 验证 | ⏳ | - |

### Phase 9: 测试验收 (复用E2E骨架)

| 任务 | 状态 | 备注 |
|------|------|------|
| P9-001 E2E扩展 | ⏳ | 3个新测试 |
| P9-002 安全测试 | ⏳ | - |
| P9-003 性能测试 | ⏳ | - |

---

## 六、产出文件清单

### 已产出文件 (项目已有)

| 文件 | 状态 | 用途 |
|------|------|------|
| src-tauri/src/database.rs | ✅ 已有 | 数据库管理 |
| src-tauri/src/lib.rs | ✅ 已有 | 主命令 |
| src-tauri/hermes-crates/* | ✅ 已有 | 7个 Hermes crates |
| src/components/* | ✅ 已有 | 35+ Vue 组件 |
| src/lib/* | ✅ 已有 | 30+ TypeScript 模块 |

### 待产出文件

| 文件 | Phase | 状态 | 用途 |
|------|-------|------|------|
| src-tauri/Cargo.toml (修改) | P0 | ⏳ | Hermes crates 链接 |
| src-tauri/src/models.rs (新建) | P1 | ⏳ | Rust 结构体 |
| src-tauri/hermes-crates/hermes-memory/* | P1 | ⏳ | Chroma集成 |
| src-tauri/src/agent_registry.rs | P2 | ⏳ | Agent Registry |
| src-tauri/src/router/smart_router.rs | P4 | ⏳ | 智能路由 |
| src-tauri/src/self_improvement/* | P5 | ⏳ | 自愈模块 |
| src-tauri/src/team/dag_engine.rs | P6 | ⏳ | DAG引擎 |
| src/components/agent-registry/*.vue | P8 | ⏳ | Agent Registry UI |
| tests/e2e/*.spec.ts | P9 | ⏳ | E2E测试 |

---

## 七、修复优先级

| 问题 | 优先级 | Phase | 状态 |
|------|--------|-------|------|
| Cargo.toml 未链接 Hermes crates | 🔴 立即 | P0 | ⏳ |
| ThinkingChunkRecord 类型缺失 | 🔴 立即 | P0 | ⏳ |
| ToolCallRecord 类型缺失 | 🔴 立即 | P0 | ⏳ |
| orchestrator.ts 假响应 | 🟡 中等 | P4 | ⏳ |
| TeamOrchestrationView 假响应 | 🟡 中等 | P6 | ⏳ |
| WorkflowView 固定示例 | 🟡 中等 | P8 | ⏳ |

---

**文档版本**: v1.1 (修订版)
**最后更新**: 2026-05-24