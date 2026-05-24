# ACP-UI 项目当前状态分析

> **分析日期**: 2026-05-24
> **目的**: 准确识别已完成功能和待修复问题

---

## 一、已完成功能 ✅

### 1.1 SQLite 数据库表 (已有 14+ 表)

| 表名 | 用途 | 状态 |
|------|------|------|
| tasks | 任务历史 | ✅ 完成 |
| agent_executions | Agent执行记录 | ✅ 完成 |
| sessions | 多会话管理 | ✅ 完成 |
| memories | 记忆系统 (含scope字段) | ✅ 完成 |
| gateway_config | 远程控制配置 | ✅ 完成 |
| workflows | 工作流定义 | ✅ 完成 |
| execution_plans | 编排计划 | ✅ 完成 |
| runtime_events | 事件日志 | ✅ 完成 |
| errors | 错误记录 | ✅ 完成 |
| solutions | 解决方案 | ✅ 完成 |
| evolutions | 自进化记录 | ✅ 完成 |
| patterns | 模式库 | ✅ 完成 |

**结论**: 数据库基础设施已大部分完成，无需从零开始！

### 1.2 Hermes Rust Crates (已有 7 个)

| Crate | 文件数 | 主要模块 | 状态 |
|-------|--------|----------|------|
| hermes-agent | 32个 | agent_loop, memory_manager, sub_agent_orchestrator, skill_orchestrator, plugins, provider | ✅ 存在 |
| hermes-config | - | 配置加载 | ✅ 存在 |
| hermes-core | - | 核心类型 | ✅ 存在 |
| hermes-tools | - | 工具注册 | ✅ 存在 |
| hermes-environments | - | 环境管理 | ✅ 存在 |
| hermes-skills | - | Skills | ✅ 存在 |
| hermes-intelligence | - | 智能模块 | ✅ 存在 |

**结论**: Hermes crates 已实现，只是 Cargo.toml 没链接！

### 1.3 Rust 后端模块 (已有 17 个文件)

| 文件 | 行数 | 功能 | 状态 |
|------|------|------|------|
| lib.rs | 69KB | 主命令入口 | ✅ 存在 |
| database.rs | 32KB | 数据库管理 | ✅ 存在 |
| websocket.rs | 39KB | WebSocket服务 | ✅ 存在 |
| executive_agent.rs | 17KB | Executive Agent | ⚠️ 有编译错误 |
| agent.rs | 12KB | Agent进程管理 | ✅ 存在 |
| agent_config_parser.rs | 11KB | YAML解析 | ✅ 存在 |
| hooks_executor.rs | 20KB | Hooks执行 | ✅ 存在 |
| permission_checker.rs | 19KB | 权限检查 | ✅ 存在 |
| mcp_manager.rs | 8KB | MCP管理 | ✅ 存在 |
| log_stream.rs | 20KB | 日志流 | ✅ 存在 |
| session_manager.rs | 12KB | Session管理 | ✅ 存在 |
| tunnel.rs | 8KB | ngrok隧道 | ✅ 存在 |
| gateway_config.rs | 5KB | Gateway配置 | ✅ 存在 |
| event_router.rs | 10KB | 事件路由 | ✅ 存在 |
| config.rs | 15KB | 配置管理 | ✅ 存在 |
| feishu_rich_message.rs | 14KB | 飞书消息 | ✅ 存在 |

### 1.4 Vue 前端组件 (已有 35+ 个)

| 组件 | 功能 | 状态 |
|------|------|------|
| HermesDashboard.vue | Hermes仪表盘 | ✅ 存在 |
| EnhancedHermesDashboard.vue | 增强仪表盘 | ✅ 存在 |
| MemoryView.vue | 记忆系统视图 | ✅ 存在 |
| HistoryView.vue | 历史记录 | ✅ 存在 |
| TeamOrchestrationView.vue | Team编排 | ✅ 存在 |
| WorkflowView.vue | 工作流视图 | ✅ 存在 |
| TaskGraphView.vue | DAG可视化 | ✅ 存在 |
| LogStreamView.vue | 日志流 | ✅ 存在 |
| EvolutionView.vue | 自进化视图 | ✅ 存在 |
| PatternView.vue | 模式视图 | ✅ 存在 |
| BotSettings.vue | Bot配置 | ✅ 存在 |
| GatewaySettings.vue | Gateway配置 | ✅ 存在 |
| ExecutiveSessionView.vue | Executive会话 | ✅ 存在 |
| agent-pet/ | 宠物系统 | ✅ 存在 |
| agent-progress/ | 进度监控 | ✅ 存在 |
| collaboration/ | 协作网络 | ✅ 存在 |
| multi-session/ | 多会话 | ✅ 存在 |

### 1.5 TypeScript lib 模块 (已有 30+ 个)

| 模块 | 功能 | 状态 |
|------|------|------|
| orchestrator.ts | 编排器 | ✅ 存在 |
| task-parser.ts | 任务解析 | ✅ 存在 |
| agent-matcher.ts | Agent匹配 | ✅ 存在 |
| code-review-agent.ts | 代码审查Agent | ✅ 存在 |
| test-validator-agent.ts | 测试验证Agent | ✅ 存在 |
| browser-adapter.ts | 浏览器适配器 | ✅ 存在 |
| hbuilderx-adapter.ts | HBuilderX适配器 | ✅ 存在 |
| wechat-devtools-adapter.ts | 微信DevTools适配器 | ✅ 存在 |
| android-studio-adapter.ts | Android Studio适配器 | ✅ 存在 |
| skills-loader.ts | Skills加载器 | ✅ 存在 |
| offline-cache.ts | 离线缓存 | ✅ 存在 |
| command-queue.ts | 命令队列 | ✅ 存在 |
| error-matching.ts | 错误匹配 | ✅ 存在 |
| pattern-discovery.ts | 模式发现 | ✅ 存在 |
| memory-extraction.ts | 记忆提取 | ✅ 存在 |
| acp-bridge.ts | ACP桥接 | ✅ 存在 |
| feature-registry.ts | 功能注册 | ✅ 存在 |

---

## 二、待修复问题 ❌

### 2.1 构建错误 (P0 阻塞)

| 问题 | 原因 | 修复方案 | 优先级 |
|------|------|----------|--------|
| **Rust编译失败** | Cargo.toml 未链接 Hermes crates | 添加 hermes-agent, hermes-core 等依赖 | 🔴 立即 |
| **ThinkingChunkRecord缺失** | database.rs 类型未定义 | 在 database.rs 添加结构体定义 | 🔴 立即 |
| **ToolCallRecord缺失** | database.rs 类型未定义 | 在 database.rs 添加结构体定义 | 🔴 立即 |
| **vue-tsc找不到** | PATH 问题 | 检查 node_modules/.bin | 🟡 中等 |

### 2.2 Cargo.toml 缺失依赖

**当前 Cargo.toml 缺少**:
```toml
# 需要添加:
hermes-agent = { path = "hermes-crates/hermes-agent" }
hermes-core = { path = "hermes-crates/hermes-core" }
hermes-config = { path = "hermes-crates/hermes-config" }
hermes-tools = { path = "hermes-crates/hermes-tools" }
hermes-environments = { path = "hermes-crates/hermes-environments" }
hermes-skills = { path = "hermes-crates/hermes-skills" }
hermes-intelligence = { path = "hermes-crates/hermes-intelligence" }
```

### 2.3 功能"假响应"问题

根据 `docs/project-completion-plan.md` 文档：

| 问题组件 | 问题描述 | 修复方案 |
|----------|----------|----------|
| MultiAgentChat.vue | 假响应，非真实Agent输出 | 连接真实 AcpSessionRunner |
| WorkflowView.vue | 固定示例数据 | 使用真实 workflows store |
| orchestrator.ts | setTimeout 模拟执行 | 真实调用 Agent |
| TeamOrchestrationView.vue | 空面板 | 接入真实 Team 数据 |

---

## 三、架构优化新增需求

### 3.1 需新增的 SQLite 表 (基于架构设计)

| 表名 | 用途 | 状态 |
|------|------|------|
| agent_bases | Agent基础镜像 | ❌ 待创建 |
| agent_templates | Agent配置模板 | ❌ 待创建 |
| agent_instances | 运行实例 | ❌ 待创建 |
| teams | 团队定义 | ❌ 待创建 |
| team_executions | 团队执行记录 | ❌ 待创建 |
| route_decisions | 路由决策 | ❌ 待创建 |
| input_logs | 输入日志 | ❌ 待创建 |
| agent_flows | Agent执行流 | ❌ 待创建 |
| flow_steps | 时序步骤 | ❌ 待创建 |
| tool_calls | 工具调用明细 | ❌ 待创建 |
| anomalies | 异常记录 | ❌ 待创建 |
| circuit_breakers | 断路器状态 | ❌ 待创建 |
| healing_actions | 恢复动作 | ❌ 待创建 |
| skills | 生成Skill | ❌ 待创建 |
| knowledge_nodes | 知识库节点 | ❌ 待创建 |
| audit_log | 审计日志 | ❌ 待创建 |

### 3.2 需新增的 Hermes Crates

| Crate | 用途 | 状态 |
|-------|------|------|
| hermes-memory | SQLite + Chroma 混合存储 | ❌ 待创建 |
| hermes-orchestration | Agent Registry + Team Engine | ❌ 待创建 |
| hermes-self-improvement | 自愈 + 自进化 | ❌ 待创建 |

### 3.3 需新增的前端组件

| 组件 | 用途 | 状态 |
|------|------|------|
| AgentRegistryPanel.vue | Agent Registry管理 | ❌ 待创建 |
| BaseList.vue | Agent Base列表 | ❌ 待创建 |
| TemplateEditor.vue | Template编辑器 | ❌ 待创建 |
| InstanceMonitor.vue | Instance监控 | ❌ 待创建 |
| MemoryScopeView.vue | 作用域记忆视图 | ❌ 待创建 |
| RouteDecisionPanel.vue | 路由决策面板 | ❌ 待创建 |
| AnomalyPanel.vue | 异常检测面板 | ❌ 待创建 |

---

## 四、修正后的实施计划

### Phase 0: 构建修复 (精简版)

只需修复 2 个核心问题：

1. **修复 Cargo.toml** - 添加 Hermes crates 依赖链接
2. **修复 database.rs** - 添加缺失的类型定义
3. **验证构建** - cargo check + npm run build

**预计时间**: 1-2小时

### Phase 1: 基础设施补全

1. **新增 SQLite 表** - 16个新表 (基于架构设计)
2. **Chroma 集成** - 向量数据库
3. **WAL + FTS5** - 性能优化

### Phase 2-9: 按原计划执行

---

## 五、结论

**项目状态比预期好得多！**

| 类别 | 已完成 | 待新增 | 待修复 |
|------|--------|--------|--------|
| SQLite 表 | 14个 ✅ | 16个 | 2个类型定义 |
| Hermes crates | 7个 ✅ | 3个 | Cargo链接 |
| Rust 后端 | 17个文件 ✅ | 0 | 编译错误 |
| Vue 组件 | 35+ ✅ | 7个 | 假响应修复 |
| TypeScript lib | 30+ ✅ | 0 | 真实调用 |

**核心修复任务**:
1. Cargo.toml 添加 Hermes crates 依赖 (🔴 立即)
2. database.rs 添加 ThinkingChunkRecord, ToolCallRecord (🔴 立即)
3. 修复假响应问题 (🟡 Phase 1后)

---

**文档版本**: v1.0
**最后更新**: 2026-05-24