# ACP-UI 架构优化实施计划 - 修订版

> **创建日期**: 2026-05-24
> **修订日期**: 2026-05-24
> **状态分析**: 基于 `docs/current-status-analysis.md`
> **基于文档**: `docs/architecture-optimization-integrated.md`

---

## 一、进度总览表

| Phase | 名称 | 任务数 | 已有基础 | 需新增 | 需修复 | 状态 |
|-------|------|--------|----------|--------|--------|------|
| Phase 0 | 构建修复 | 3 | 0 | 0 | 3 | ⏳ 待开始 |
| Phase 1 | 基础设施补全 | 6 | 14表已有 | 16表 | 2类型 | ⏳ 待开始 |
| Phase 2 | Agent Registry | 5 | 7 crates已有 | 3 crates | Cargo链接 | ⏳ 待开始 |
| Phase 3 | 记忆系统增强 | 4 | 已有基础 | Chroma+FTS5 | - | ⏳ 待开始 |
| Phase 4 | 智能路由 | 5 | orchestrator已有 | 路由模块 | - | ⏳ 待开始 |
| Phase 5 | 自愈系统增强 | 3 | errors表已有 | EWMA+熔断器 | - | ⏳ 待开始 |
| Phase 6 | Team编排增强 | 4 | TeamView已有 | DAG引擎 | 假响应修复 | ⏳ 待开始 |
| Phase 7 | Hermes集成 | 3 | 32文件已有 | 3新crates | - | ⏳ 待开始 |
| Phase 8 | 前端对接 | 5 | 35+组件已有 | 7新组件 | 假响应修复 | ⏳ 待开始 |
| Phase 9 | 测试验收 | 3 | E2E骨架已有 | 完整测试 | - | ⏳ 待开始 |

**总工作量大幅减少**: 大部分功能已存在，只需修复和补全！

---

## 二、已完成功能 ✅ (无需重做)

### 2.1 SQLite 表 (已有 14 个)

```
已存在表:
├── tasks               ✅ 任务历史
├── agent_executions    ✅ Agent执行记录
├── sessions            ✅ 多会话管理
├── memories            ✅ 记忆系统 (已有 scope, task_id 字段)
├── gateway_config      ✅ 远程控制配置
├── workflows           ✅ 工作流定义
├── execution_plans     ✅ 编排计划
├── runtime_events      ✅ 事件日志
├── errors              ✅ 错误记录
├── solutions           ✅ 解决方案
├── evolutions          ✅ 自进化记录
├── patterns            ✅ 模式库
└── ... (还有更多)
```

### 2.2 Hermes Crates (已有 7 个)

```
已存在 crates:
├── hermes-agent        ✅ 32个文件 (agent_loop, memory_manager, sub_agent_orchestrator, skill_orchestrator)
├── hermes-config       ✅ 配置加载
├── hermes-core         ✅ 核心类型
├── hermes-tools        ✅ 工具注册
├── hermes-environments ✅ 环境管理
├── hermes-skills       ✅ Skills
├── hermes-intelligence ✅ 智能模块
```

### 2.3 Rust 后端模块 (已有 17 个文件)

```
已存在文件:
├── lib.rs              ✅ 69KB 主命令
├── database.rs         ✅ 32KB 数据库 (需添加2个类型)
├── websocket.rs        ✅ 39KB WebSocket
├── executive_agent.rs  ⚠️ 17KB (有编译错误，依赖未链接)
├── agent.rs            ✅ Agent进程
├── hooks_executor.rs   ✅ Hooks执行
├── permission_checker.rs ✅ 权限检查
├── mcp_manager.rs      ✅ MCP管理
├── log_stream.rs       ✅ 日志流
├── session_manager.rs  ✅ Session管理
├── ... (还有更多)
```

### 2.4 Vue 组件 (已有 35+ 个)

```
已存在组件:
├── HermesDashboard.vue         ✅ Hermes仪表盘
├── EnhancedHermesDashboard.vue ✅ 增强仪表盘
├── MemoryView.vue              ✅ 记忆系统视图
├── HistoryView.vue             ✅ 历史记录
├── TeamOrchestrationView.vue   ⚠️ Team编排 (假响应)
├── WorkflowView.vue            ⚠️ 工作流 (固定示例)
├── TaskGraphView.vue           ✅ DAG可视化
├── LogStreamView.vue           ✅ 日志流
├── EvolutionView.vue           ✅ 自进化视图
├── PatternView.vue             ✅ 模式视图
├── agent-pet/                  ✅ 宠物系统
├── ... (还有更多)
```

### 2.5 TypeScript lib (已有 30+ 个)

```
已存在模块:
├── orchestrator.ts             ⚠️ 编排器 (setTimeout模拟)
├── task-parser.ts              ✅ 任务解析
├── agent-matcher.ts            ✅ Agent匹配
├── browser-adapter.ts          ✅ 浏览器适配器
├── hbuilderx-adapter.ts        ✅ HBuilderX适配器
├── wechat-devtools-adapter.ts  ✅ 微信DevTools适配器
├── android-studio-adapter.ts   ✅ Android Studio适配器
├── code-review-agent.ts        ✅ 代码审查Agent
├── test-validator-agent.ts     ✅ 测试验证Agent
├── skills-loader.ts            ✅ Skills加载器
├── offline-cache.ts            ✅ 离线缓存
├── ... (还有更多)
```

---

## 三、Phase 0: 构建修复 (精简版)

**目标**: 修复 Cargo.toml 链接 + database.rs 类型定义，让构建通过

**预计时间**: 1-2 小时

### 任务清单

#### P0-001: 修复 Cargo.toml (添加 Hermes crates 链接)

- [ ] **P0-001.1** 添加 Hermes crates 依赖
  ```toml
  # 在 src-tauri/Cargo.toml 添加:
  [dependencies]
  hermes-agent = { path = "hermes-crates/hermes-agent" }
  hermes-core = { path = "hermes-crates/hermes-core" }
  hermes-config = { path = "hermes-crates/hermes-config" }
  hermes-tools = { path = "hermes-crates/hermes-tools" }
  hermes-environments = { path = "hermes-crates/hermes-environments" }
  hermes-skills = { path = "hermes-crates/hermes-skills" }
  hermes-intelligence = { path = "hermes-crates/hermes-intelligence" }
  ```
  - 产出文件: `src-tauri/Cargo.toml`
  
- [ ] **P0-001.2** 验证 Rust 编译
  - 命令: `cargo check --manifest-path src-tauri/Cargo.toml`
  - 验收: 无 hermes_agent/hermes_core 未链接错误

#### P0-002: 修复 database.rs (添加缺失类型)

- [ ] **P0-002.1** 添加 ThinkingChunkRecord 结构体
  ```rust
  // 在 database.rs 添加:
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ThinkingChunkRecord {
      pub id: String,
      pub session_id: String,
      pub content: String,
      pub created_at: DateTime<Utc>,
  }
  ```
  
- [ ] **P0-002.2** 添加 ToolCallRecord 结构体
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ToolCallRecord {
      pub id: String,
      pub session_id: String,
      pub tool_name: String,
      pub arguments: String,
      pub result: Option<String>,
      pub created_at: DateTime<Utc>,
  }
  ```

#### P0-003: 验证构建通过

- [ ] **P0-003.1** 验证 Rust 构建
  - 命令: `cargo check --manifest-path src-tauri/Cargo.toml`
  - 验收: PASS
  
- [ ] **P0-003.2** 验证 Web 构建
  - 命令: `npm run build`
  - 验收: PASS (如有 vue-tsc 问题，修复 PATH)
  
- [ ] **P0-003.3** 提交修复
  - 命令: `git add . && git commit -m "fix: link Hermes crates and add missing types"`

---

## 四、Phase 1: 基础设施补全

**目标**: 新增 16 个 SQLite 表 + Chroma 集成 + WAL/FTS5

**已有基础**: 14 个表已存在

### 任务清单

#### P1-001: 新增 SQLite 表 (16 个)

- [ ] **P1-001.1** 在 database.rs init_tables() 添加新表
  ```sql
  -- Agent Registry 表
  CREATE TABLE agent_bases (id, transport, capabilities_json, default_skills_json)
  CREATE TABLE agent_templates (id, base_id, skills_strategy, hooks_strategy, permissions_json)
  CREATE TABLE agent_instances (id, template_id, status, pid, session_id)
  
  -- Team 编排表
  CREATE TABLE teams (id, name, members_json, sync_points_json)
  CREATE TABLE team_executions (id, team_id, plan_json, status)
  
  -- 智能路由表
  CREATE TABLE route_decisions (id, task_type, complexity, route_target, reason)
  CREATE TABLE input_logs (id, source, input_type, content_hash, route_to)
  
  -- 记忆增强表
  CREATE TABLE agent_flows (id, agent_id, flow_steps_json, state_snapshots_json)
  CREATE TABLE flow_steps (id, flow_id, step_index, step_type, tool_name)
  CREATE TABLE tool_calls (id, flow_id, tool_name, result_status, duration_ms)
  
  -- 自愈系统表
  CREATE TABLE anomalies (id, type, severity, target, health_score)
  CREATE TABLE circuit_breakers (id, target, state, failure_count, cool_down_until)
  CREATE TABLE healing_actions (id, anomaly_id, action_type, status)
  
  -- Skill + 知识库表
  CREATE TABLE skills (id, name, status, risk_level, definition_json)
  CREATE TABLE knowledge_nodes (id, node_type, content, relevance_score, expires_at)
  
  -- 审计日志
  CREATE TABLE audit_log (id, event_type, actor, target_type, details_json)
  ```

#### P1-002: WAL 模式 + FTS5

- [ ] **P1-002.1** 启用 WAL 模式 (已有基础，可能已启用)
  - 检查: `PRAGMA journal_mode`
  - 如果未启用: `PRAGMA journal_mode=WAL`
  
- [ ] **P1-002.2** 创建 FTS5 虚拟表
  ```sql
  CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
      content, tags, agent_id, session_id
  );
  ```

#### P1-003: Chroma 向量数据库

- [ ] **P1-003.1** 创建 hermes-memory crate (新建)
  - 目录: `src-tauri/hermes-crates/hermes-memory/`
  - 主要模块: chroma_provider.rs, embedding.rs, hybrid_search.rs
  
- [ ] **P1-003.2** 创建 Embedding 层
  - L1: 本地 ONNX (384维)
  - L2: 云端 API (1024维)

#### P1-004: 验证基础设施

- [ ] **P1-004.1** 验证新表创建
- [ ] **P1-004.2** 验证 FTS5 功能
- [ ] **P1-004.3** 提交 Phase 1

---

## 五、Phase 2: Agent Registry

**目标**: 实现 Docker-like Agent 编排

**已有基础**: Hermes crates 已存在，只需链接 + 补充 Registry 模块

### 任务清单

#### P2-001: 修复 Cargo.toml (已在 Phase 0 完成)

✅ Phase 0 已完成 Hermes crates 链接

#### P2-002: 创建 Agent Registry 模块

- [ ] **P2-002.1** 创建 agent_registry.rs
  ```rust
  // 在 src-tauri/src/ 新建:
  pub struct AgentBase { id, transport, capabilities }
  pub struct AgentTemplate { base_id, skills_strategy, hooks_strategy }
  pub struct AgentInstance { template_id, status, pid }
  pub struct ConfigStrategy { Append, Override, Exclude }
  ```
  
- [ ] **P2-002.2** 实现 Registry 加载
  - 函数: `load_base_from_yaml()`, `load_template_from_yaml()`
  
- [ ] **P2-002.3** 实现 Instance 管理
  - 函数: `spawn_instance()`, `stop_instance()`, `get_instance_status()`

#### P2-003: 创建配置示例

- [ ] **P2-003.1** 创建 agent-bases 目录
  - `config/agent-bases/claude-code-base.yaml`
  - `config/agent-bases/codex-base.yaml`
  
- [ ] **P2-003.2** 创建 agent-templates 目录
  - `config/agent-templates/frontend-dev.yaml`
  - `config/agent-templates/backend-dev.yaml`

#### P2-004: 创建 hermes-orchestration crate

- [ ] **P2-004.1** 创建 crate 结构
  - 目录: `src-tauri/hermes-crates/hermes-orchestration/`
  - 主要模块: agent_registry.rs, team_engine.rs

#### P2-005: 验证 Agent Registry

- [ ] 测试 Base 加载
- [ ] 测试 Template 派生
- [ ] 测试 Instance 启动

---

## 六、Phase 3: 记忆系统增强

**目标**: Chroma + FTS5 混合检索 + 记忆注入

**已有基础**: memories 表已存在，已有 scope 字段

### 任务清单

#### P3-001: 混合检索 (基于已存在的 memories 表)

- [ ] **P3-001.1** 实现 FTS5 全文搜索
  - 函数: `search_memories_fts(query)`
  
- [ ] **P3-001.2** 实现向量检索
  - 函数: `search_memories_vector(query, limit)`
  
- [ ] **P3-001.3** 实现混合检索
  - 函数: `search_memories_hybrid(query)` → FTS + Vector + 相关性评分

#### P3-002: 记忆注入系统

- [ ] **P3-002.1** 创建 memory_injection.rs
  - 时序点: Pre-Task, Pre-Tool, Pre-Turn, Pre-Compress
  
- [ ] **P3-002.2** 实现相关性检索
  - 函数: `retrieve_relevant_memories(task, limit)`

#### P3-003: 输入层记录

- [ ] **P3-003.1** 使用 input_logs 表 (Phase 1 新增)
  - 字段: source, input_type, content_hash, route_to

#### P3-004: Agent 流程记录

- [ ] **P3-004.1** 使用 agent_flows + flow_steps 表 (Phase 1 新增)
  - 记录每个工具调用步骤

---

## 七、Phase 4: 智能路由

**目标**: 三层复杂度评估 + 熔断器

**已有基础**: orchestrator.ts, task-parser.ts, agent-matcher.ts 已存在

### 任务清单

#### P4-001: 复用已有模块

✅ task-parser.ts 已存在 - 任务模板匹配
✅ agent-matcher.ts 已存在 - Jaccard 相似度匹配
✅ orchestrator.ts 已存在 - 需修复假响应

#### P4-002: 新增路由决策模块

- [ ] **P4-002.1** 创建 smart_router.rs
  ```rust
  // 三层渐进评估:
  // 1. 启发式 (零成本) - 单文件=简单, 多文件=中等
  // 2. 结构化 (低成本) - AST解析
  // 3. LLM评估 (高成本) - 5%边界请求
  ```
  
- [ ] **P4-002.2** 创建 circuit_breaker.rs
  - 三态: Closed → Open → HalfOpen

#### P4-003: 路由决策记录

- [ ] 使用 route_decisions 表 (Phase 1 新增)

#### P4-004: 修复 orchestrator.ts 假响应

- [ ] 替换 setTimeout 模拟为真实 Agent 调用

#### P4-005: 验证智能路由

- [ ] 测试简单任务路由
- [ ] 测试复杂任务路由

---

## 八、Phase 5: 自愈系统增强

**目标**: EWMA 动态基线 + Skill 验证

**已有基础**: errors, solutions, evolutions, patterns 表已存在

### 任务清单

#### P5-001: 复用已有表

✅ errors 表已存在 - 错误记录
✅ solutions 表已存在 - 解决方案
✅ patterns 表已存在 - 模式库

#### P5-002: 新增 EWMA 模块

- [ ] **P5-002.1** 创建 anomaly_detector.rs
  ```rust
  // EWMA 动态基线:
  // baseline = alpha * current + (1-alpha) * previous
  // 自适应阈值，无需手动调参
  ```
  
- [ ] **P5-002.2** 使用 anomalies 表 (Phase 1 新增)
  - 字段: type, severity, target, health_score

#### P5-003: Skill 验证流水线

- [ ] **P5-003.1** 创建 skill_validator.rs
  - 六步验证: LLM生成 → 语法检查 → 工具可用 → 权限检查 → DryRun → 安全扫描
  
- [ ] **P5-003.2** 使用 skills 表 (Phase 1 新增)

---

## 九、Phase 6: Team 编排增强

**目标**: DAG 执行引擎 + SyncPoint

**已有基础**: TeamOrchestrationView.vue 已存在 (假响应)

### 任务清单

#### P6-001: DAG 执行引擎

- [ ] **P6-001.1** 创建 dag_engine.rs
  - 拓扑排序验证
  - 并行/串行执行
  
- [ ] **P6-001.2** 创建 sync_point.rs
  - 同步等待/触发

#### P6-002: EventBus (复用已有)

✅ event_router.rs 已存在 (10KB)

- [ ] **P6-002.1** 扩展事件类型
  - TaskComplete, TaskFailed, SyncReached, TeamProgress

#### P6-003: Team 执行记录

- [ ] 使用 teams + team_executions 表 (Phase 1 新增)

#### P6-004: 修复 TeamOrchestrationView.vue 假响应

- [ ] 连接真实 Team 数据

---

## 十、Phase 7: Hermes 集成

**目标**: 将新模块集成到 Hermes AgentLoop

**已有基础**: hermes-agent 已有 32 个文件，包括 agent_loop.rs, memory_manager.rs, sub_agent_orchestrator.rs

### 任务清单

#### P7-001: 复用已有模块

✅ agent_loop.rs 已存在 (277KB) - 核心循环
✅ memory_manager.rs 已存在 (22KB) - 记忆管理
✅ sub_agent_orchestrator.rs 已存在 (25KB) - 子Agent编排
✅ skill_orchestrator.rs 已存在 (16KB) - Skill编排

#### P7-002: 集成新模块

- [ ] **P7-002.1** 集成 hermes-memory (Phase 1 新建)
  - 替换 memory_manager.rs 中的存储层
  
- [ ] **P7-002.2** 集成 hermes-orchestration (Phase 2 新建)
  - Agent Registry + Team Engine
  
- [ ] **P7-002.3** 集成 hermes-self-improvement (新建)
  - 自愈 + 自进化

#### P7-003: 验证 Hermes 集成

- [ ] 测试 AgentLoop
- [ ] 测试 SubAgentOrchestrator

---

## 十一、Phase 8: 前端对接

**目标**: 新增 7 个组件 + 修复假响应

**已有基础**: 35+ Vue 组件已存在

### 任务清单

#### P8-001: 新增 Agent Registry UI

- [ ] AgentRegistryPanel.vue
- [ ] BaseList.vue
- [ ] TemplateEditor.vue
- [ ] InstanceMonitor.vue

#### P8-002: 新增其他 UI

- [ ] MemoryScopeView.vue
- [ ] RouteDecisionPanel.vue
- [ ] AnomalyPanel.vue

#### P8-003: 修复假响应

- [ ] MultiAgentChat.vue → 真实 AcpSessionRunner
- [ ] WorkflowView.vue → 真实 workflows store
- [ ] TeamOrchestrationView.vue → 真实 Team 数据
- [ ] orchestrator.ts → 真实 Agent 调用

#### P8-004: Dashboard 集成

- [ ] 修改 HermesDashboard.vue 集成新面板

#### P8-005: 验证前端

- [ ] 所有新组件渲染正常
- [ ] 所有假响应已修复

---

## 十二、Phase 9: 测试验收

**目标**: E2E + 安全 + 性能测试

**已有基础**: tests/e2e/ 骨架已存在

### 任务清单

#### P9-001: 扩展 E2E 测试

- [ ] agent-registry.spec.ts
- [ ] team-orchestration.spec.ts
- [ ] memory-system.spec.ts

#### P9-002: 安全测试

- [ ] SQL 参数绑定检查
- [ ] Token 安全检查

#### P9-003: 性能测试

- [ ] 路由延迟 P99 < 50ms
- [ ] Agent 并发 > 10 Instance

---

## 十三、执行日志模板

每次执行时记录：

```markdown
| 任务ID | 开始时间 | 结束时间 | 状态 | 产出文件 | 备注 |
|--------|----------|----------|------|----------|------|
| P0-001.1 | 2026-05-24 10:00 | 2026-05-24 10:15 | ✅ | src-tauri/Cargo.toml | Hermes crates 链接成功 |
```

---

## 十四、修订总结

| 类别 | 原计划任务 | 修订后任务 | 减少 |
|------|------------|------------|------|
| Phase 0 | 5 个 | 3 个 | -2 |
| Phase 1 | 8 个 | 6 个 | -2 |
| Phase 2 | 6 个 | 5 个 | -1 |
| Phase 3 | 7 个 | 4 个 | -3 |
| Phase 4 | 6 个 | 5 个 | -1 |
| Phase 5 | 5 个 | 3 个 | -2 |
| Phase 6 | 7 个 | 4 个 | -3 |
| Phase 7 | 5 个 | 3 个 | -2 |
| Phase 8 | 6 个 | 5 个 | -1 |
| Phase 9 | 4 个 | 3 个 | -1 |
| **总计** | **53 个** | **38 个** | **-15** |

**原因**: 大量功能已存在，只需修复和补全！

---

**文档版本**: v1.1 (修订版)
**最后更新**: 2026-05-24