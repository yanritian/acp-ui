# ACP-UI 架构优化设计 - 完整整合版

> **整合者**: Winston (BMad 系统架构师团队)
> **日期**: 2026-05-24
> **状态**: 最终评审

---

## 一、四大模块优化总览

| 模块 | 核心优化 | 关键决策 |
|------|----------|----------|
| **Agent编排系统** | Docker-like三层架构 + Team编排 | Base/Template/Instance + DAG动态调整 |
| **多维记忆系统** | SQLite+Chroma混合 + 混合检索 | FTS5 + 向量 + 相关性评分 |
| **智能路由系统** | 三层渐进评估 + 熔断器 | 启发式→结构化→LLM评估 |
| **自愈与自进化** | EWMA动态基线 + 六步验证流水线 | 断路器 + Skill人工审核 |

---

## 二、架构图整合

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ACP-UI 完整架构 v2                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                     CLIENT LAYER (客户端层)                           │  │
│  │  Tauri桌面 + Flutter移动 + Web H5 + IM远程(飞书/TG/Discord)           │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                   ORCHESTRATION LAYER (编排层)                        │  │
│  │                                                                       │  │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐ │  │
│  │  │ AgentRegistry   │ │ TeamEngine      │ │ IntelligenceCoordinator │ │  │
│  │  │ (Docker-like)   │ │ (DAG编排)       │ │ (智能路由)               │ │  │
│  │  │                 │ │                 │ │                         │ │  │
│  │  │ Base/Template   │ │ SyncPoints      │ │ TaskAnalyzer            │ │  │
│  │  │ Instance Pool   │ │ EventBus        │ │ SmartRouter             │ │  │
│  │  └─────────────────┘ └─────────────────┘ │ LoadBalancer            │ │  │
│  │                                           └─────────────────────────┘ │  │
│  │                                                                       │  │
│  │  ┌─────────────────┐ ┌─────────────────────────────────────────────┐ │  │
│  │  │ AnomalyDetector │ │ SelfHealing + SelfEvolution                  │ │  │
│  │  │ (异常检测)      │ │ (自愈与自进化)                                │ │  │
│  │  │                 │ │                                              │ │  │
│  │  │ Heartbeat       │ │ CircuitBreaker + PatternLearner              │ │  │
│  │  │ ResourceWatch   │ │ FallbackChain + SkillGenerator               │ │  │
│  │  │ EWMA Baseline   │ │ ContextCompressor + ValidationPipeline       │ │  │
│  │  └─────────────────┘ └─────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                     AGENT LAYER (Agent层)                             │  │
│  │                                                                       │  │
│  │  ┌─────────────────────────────────────────────────────────────────┐ │  │
│  │  │ AgentBase: claude-code-base | codex-base | hermes-base          │ │  │
│  │  │ AgentTemplate: frontend-dev | backend-dev | qa-reviewer        │ │  │
│  │  │ AgentInstance: 多实例并发 + 状态隔离                             │ │  │
│  │  └─────────────────────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                   HERMES ENGINE LAYER (引擎层)                        │  │
│  │  AgentLoop + SubAgentOrchestrator + FallbackChain + MemoryManager     │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                  INFRASTRUCTURE LAYER (基础设施层)                    │  │
│  │                                                                       │  │
│  │  ┌─────────────────────┐ ┌─────────────────────────────────────────┐ │  │
│  │  │ SQLite (结构化)      │ │ Chroma (向量)                           │ │  │
│  │  │                     │ │                                         │ │  │
│  │  │ - agent_bases       │ │ Collections:                            │ │  │
│  │  │ - agent_templates   │ │ - acp_memory_global_v1                  │ │  │
│  │  │ - agent_instances   │ │ - acp_memory_agent_v1                  │ │  │
│  │  │ - teams             │ │ - acp_memory_session_v1                 │ │  │
│  │  │ - route_decisions   │ │ - acp_memory_task_v1                   │ │  │
│  │  │ - input_logs        │ │                                         │ │  │
│  │  │ - agent_flows       │ │ Embedding:                              │ │  │
│  │  │ - flow_steps        │ │ - L1: 本地ONNX (384维)                  │ │  │
│  │  │ - tool_calls        │ │ - L2: 云端API (1024维)                  │ │  │
│  │  │ - memories (FTS5)   │ │                                         │ │  │
│  │  │ - anomalies         │ │                                         │ │  │
│  │  │ - circuit_breakers  │ │                                         │ │  │
│  │  │ - skills            │ │                                         │ │  │
│  │  │ - knowledge_nodes   │ │                                         │ │  │
│  │  │ - audit_log         │ │                                         │ │  │
│  │  └─────────────────────┘ └─────────────────────────────────────────┘ │  │
│  │                                                                       │  │
│  │  ┌─────────────────────┐ ┌─────────────────────────────────────────┐ │  │
│  │  │ Tool Adapters       │ │ Core Services                           │ │  │
│  │  │ Browser/HBuilderX   │ │ MCP Manager + Hooks + PermChecker       │ │  │
│  │  │ WeChat/Android      │ │ LogStream + EventBus                    │ │  │
│  │  └─────────────────────┘ └─────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、核心决策汇总 (ADR)

### ADR-Agent-001: Docker-like三层架构
- **决策**: Base → Template → Instance
- **理由**: 用户心智模型清晰，配置可复用，实例隔离

### ADR-Agent-002: Template派生策略
- **决策**: ConfigStrategy (Append/Override/Exclude)
- **理由**: 权限deny不可覆盖（安全优先）

### ADR-Agent-003: 多层派生深度
- **决策**: 限制最多3层
- **理由**: 防止配置爆炸和调试困难

### ADR-Memory-001: 输入层分表
- **决策**: 不分表，单表 + source/input_type字段
- **理由**: 跨来源查询频繁，SQLite不擅长UNION

### ADR-Memory-002: Collection分割
- **决策**: 按 scope 分割 (global/agent/session/task)
- **理由**: 平衡隔离性和跨session检索需求

### ADR-Memory-003: Embedding分层
- **决策**: L1本地ONNX + L2云端API
- **理由**: 低成本日常 + 高精度重要记忆

### ADR-Router-001: 复杂度评估
- **决策**: 三层渐进 (启发式→结构化→LLM)
- **理由**: 80%请求零成本拦截，5%边界请求LLM辅助

### ADR-Router-002: 熔断器
- **决策**: 三段式断路器 (Closed→Open→HalfOpen)
- **理由**: 可恢复、可观测、状态持久化

### ADR-Router-003: 路由缓存
- **决策**: SimHash相似匹配 (>0.85)
- **理由**: 命中率30-50%，可控风险

### ADR-Healing-001: 异常阈值
- **决策**: EWMA动态基线
- **理由**: 自适应，减少误报，无需手动调参

### ADR-Healing-002: Skill生成
- **决策**: 六步验证流水线 + 人工审核高风险
- **理由**: 防止有害Skill，异步不阻塞

### ADR-Healing-003: 数据安全
- **决策**: 三层脱敏 + 用户代码hash存储
- **理由**: 纵深防御，隐私保护

---

## 四、新增SQLite表汇总

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| agent_bases | Agent基础镜像 | id, transport, capabilities |
| agent_templates | Agent配置模板 | base_id, skills_strategy, hooks_strategy |
| agent_instances | 运行实例 | template_id, status, pid |
| teams | 团队定义 | members_json, sync_points_json |
| team_executions | 团队执行记录 | plan_json, strategy, status |
| route_decisions | 路由决策 | task_type, complexity, route_target |
| input_logs | 输入日志 | source, input_type, content_hash |
| agent_flows | Agent执行流 | flow_steps_json, state_snapshots_json |
| flow_steps | 时序步骤(影子表) | step_index, step_type, tool_name |
| tool_calls | 工具调用明细 | tool_name, result_status, duration_ms |
| anomalies | 异常记录 | severity, target, health_score |
| circuit_breakers | 断路器状态 | state, failure_count, cool_down_until |
| healing_actions | 恢复动作 | anomaly_id, action_type, status |
| skills | 生成Skill | name, status, risk_level |
| knowledge_nodes | 知识库节点 | node_type, relevance_score, expires_at |
| audit_log | 审计日志 | event_type, actor, target_type |

---

## 五、新增Hermes Crates

| Crate | 职责 | 关键模块 |
|-------|------|----------|
| hermes-memory | SQLite + Chroma混合 | sqlite_provider, chroma_provider, embedding |
| hermes-orchestration | Agent编排 + Team | agent_registry, team_engine, execution_plan |
| hermes-self-improvement | 自愈 + 自进化 | self_healing, self_evolution, circuit_breaker |

---

## 六、实施路线图整合

| Phase | 周期 | 核心交付 |
|-------|------|----------|
| Phase 1: 基础设施 | 2-3周 | 新增SQLite表 + WAL优化 + FTS5 |
| Phase 2: Agent Registry | 2-3周 | Base/Template/Instance + Registry |
| Phase 3: 记忆系统 | 2-3周 | Chroma集成 + 混合检索 + 缓存 |
| Phase 4: 智能路由 | 2-3周 | 三层评估 + 熔断器 + 学习引擎 |
| Phase 5: 自愈系统 | 2周 | EWMA基线 + 断路器 + Skill生成 |
| Phase 6: Team编排 | 3-4周 | DAG执行 + SyncPoint + EventBus |
| Phase 7: Hermes集成 | 2-3周 | SubAgent复用 + Memory共享 |
| Phase 8: 前端对接 | 2-3周 | Dashboard + 配置面板 |
| Phase 9: 测试验收 | 1-2周 | E2E + 安全测试 + 性能测试 |

**总计**: 约16-20周

---

## 七、风险矩阵

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| SQLite写锁竞争 | 高 | 中 | WAL模式 + 批量写入 |
| 派生链过深 | 中 | 低 | 限制3层 + 缓存 |
| Instance崩溃数据丢失 | 高 | 中 | 状态快照 + 优雅关闭 |
| DAG循环依赖 | 高 | 低 | 创建时拓扑排序验证 |
| 动态基线冷启动 | 中 | 高 | 固定阈值兜底 + 初期仅记录 |
| Skill生成质量差 | 中 | 中 | 六步验证 + 人工审核 |
| 学习数据膨胀 | 中 | 高 | 定期清理 (默认90天) |
| 断路器误触发 | 中 | 中 | HalfOpen快速恢复 + 用户重置 |

---

## 八、监控指标汇总

| 指标 | 告警阈值 | 频率 |
|------|----------|------|
| 健康分 | < 50 | 实时 |
| 断路器Open数量 | > 2 | 每分钟 |
| 自愈成功率 | < 30% | 每5分钟 |
| 路由延迟P99 | > 50ms | 每分钟 |
| Chroma查询延迟 | > 200ms | 每5分钟 |
| 记忆注入token占比 | > 15% | 每轮次 |
| Instance并发数 | > 10 | 实时 |

---

## 九、WebSocket事件汇总

| 事件 | 触发时机 | 数据 |
|------|----------|------|
| anomaly-detected | 异常检测 | anomaly_type, severity, target |
| healing-action | 恢复执行 | action_type, target, result |
| circuit-breaker-state | 状态变更 | target, state, failure_count |
| pattern-learned | 模式学习 | pattern_id, confidence_score |
| skill-generated | Skill生成 | skill_id, status, risk_level |
| route-decision | 路由决策 | agent_id, complexity, reason |
| health-score-change | 健康分变化 | score, trend |
| team-execution-progress | Team执行进度 | plan_id, completed_nodes |

---

**文档版本**: v2.0
**完成日期**: 2026-05-24