# ACP-UI 产品需求文档 (PRD)
# 版本: 2.0 - 自进化多 Agent 编排平台
# 日期: 2026-06-06
# 作者: John (Product Manager)

---

## 1. 产品定位

### 1.1 核心愿景

**ACP-UI = 自进化多 Agent 编排平台**

一个让 Agent 能够自主完成完整开发流程的平台：
```
需求分析 → 计划撰写 → 写代码 → 代码调整 → 功能测试 → 反馈给用户
```

### 1.2 产品类比

| 产品 | 功能 | ACP-UI 对应 |
|------|------|-------------|
| Codex Mobile | 移动端控制 Agent | ✅ 本地多 Agent |
| Claude Code Remote | 远程执行 | ⏳ 后期 Cloud Agent |
| Qoder | AI 编程助手 | ✅ Skill 系统 |
| Dingding | 企业协作/通知 | ✅ Bot 集成 |

### 1.3 核心用户

**开发者** - 需要高效管理多个本地 Code Agent，自动化开发流程

---

## 2. 核心功能需求

### 2.1 首页改造（Phase 1）

**问题：**
- 当前首页默认 `/chat`，但未配置 Agent 时显示空 WelcomeScreen
- Agent Config 在导航列表第 3 位，不够突出
- 25 个功能页面平铺，信息过载

**解决方案：首页重构**

```
┌─────────────────────────────────────────────────┐
│  🤖 ACP UI                                       │
├─────────────────────────────────────────────────┤
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │  本地 AGENT 状态                         │   │
│   │  ────────────────────────────────────   │   │
│   │  🟢 Claude Code        运行中           │   │
│   │  🔴 Copilot            已停止           │   │
│   │  🟢 MCP Filesystem     运行中           │   │
│   │                              [+ 添加]   │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │  快速任务                                │   │
│   │  ────────────────────────────────────   │   │
│   │  选择 Agent: [Claude Code ▼]            │   │
│   │  输入任务: [________________]  [执行]   │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │  最近执行                                │   │
│   │  ────────────────────────────────────   │   │
│   │  Claude Code: 生成 README      ✓ 完成   │   │
│   │  Claude Code: 分析代码结构    ⏳ 进行中 │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

**首次体验流程：**
```
App 启动 → 检测本机 Agent
  ├── 发现 Claude Code → 自动添加 → 显示状态卡片
  ├── 未发现 → 显示引导卡片 "点击添加你的第一个 Agent"
  │
→ Agent 添加成功 → 显示测试按钮
  → "发送测试任务：列出当前目录" → 确认连接正常
  │
→ 页显示完整：Agent 状态 + 快速任务 + 最近任务
```

**功能层级调整：**
```
第一层（首页直接展示）:
├── 🤖 Agent 管理 → 本地 Agent 配置（核心）
├── 📝 任务执行 → 发送任务给 Agent
├── 📊 监控 → Agent 状态 + 执行日志

第二层（侧边栏）:
├── 💬 Chat
├── 🤖 Multi-Agent
├── ⚡ Workflow
├── 🧩 Skills
├── 🔌 Plugins

第三层（折叠）:
├── Gateway (降级)
├── Bot Config (降级)
├── Memory, Evolution, Pattern...
├── Hermes, Token Optimizer...
```

---

### 2.2 自进化闭环（Phase 2）

**目标：Agent 自主完成完整开发流程**

```
┌─────────────────────────────────────────────────┐
│                自进化闭环                         │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │需求分析 │ → │计划撰写 │ → │写代码   │      │
│  └────┬────┘    └────┬────┘    └────┬────┘      │
│       │              │              │           │
│       ▼              ▼              ▼           │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │Skill:   │    │Skill:   │    │Skill:   │      │
│  │planning │    │code-    │    │code-    │      │
│  │         │    │execution│    │review   │      │
│  └─────────┘    └─────────┘    └─────────┘      │
│                                                 │
│       ↓                                          │
│                                                 │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │代码调整 │ → │功能测试 │ → │反馈用户 │      │
│  └────┬────┘    └────┬────┘    └────┬────┘      │
│       │              │              │           │
│       ▼              ▼              ▼           │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │Skill:   │    │Skill:   │    │MCP+Skill│      │
│  │file-    │    │testing  │    │通知推送 │      │
│  │operations│    │         │    │         │      │
│  └─────────┘    └─────────┘    └─────────┘      │
│                                                 │
│       ↓                                          │
│       │                                          │
│       ▼                                          │
│  ┌─────────────────────────────────────────┐    │
│  │         Evolution Engine                 │    │
│  │  分析执行结果 → 生成改进建议 → 自动优化  │    │
│  └─────────────────────────────────────────┘    │
│                                                 │
└─────────────────────────────────────────────────┘
```

**关键组件增强：**

| 组件 | 当前状态 | 需要增强 |
|------|----------|----------|
| Evolution Engine | ✅ 分析建议 | → 自动执行改进 |
| Self-Healing | ✅ Retry/Fallback | → 主动修复代码 |
| Skill System | ✅ 16 Core Skills | → 开发流程 Skill 链 |
| Hermes | ✅ DAG+QA | → 完整开发流程编排 |

**新增 Skill 链：**
```typescript
const DEVELOPMENT_FLOW_SKILLS = [
  'planning',        // 需求分析 + 计划撰写
  'code-execution',  // 写代码
  'code-review',     // 代码审查
  'file-operations', // 代码调整
  'testing',         // 功能测试
  'documentation',   // 文档生成
  'communication',   // 反馈用户
];
```

---

### 2.3 多端同步（Phase 3）

**目标：App / Tauri / Web 数据实时同步**

```
┌─────────────────────────────────────────────────┐
│              多端同步架构                         │
├─────────────────────────────────────────────────┤
│                                                 │
│   ┌─────────┐    ┌─────────┐    ┌─────────┐     │
│   │手机 App │    │Tauri桌面│    │Web浏览器│     │
│   │ Flutter │    │  Rust   │    │  Vue 3  │     │
│   └────┬────┘    └────┬────┘    └────┬────┘     │
│        │              │              │          │
│        └──────────────┼──────────────┘          │
│                       │                         │
│                       ▼                         │
│              ┌─────────────────┐                │
│              │   Sync Engine   │                │
│              │   SQLite + WS   │                │
│              └────────┬────────┘                │
│                       │                         │
│                       ▼                         │
│   ┌─────────────────────────────────────────┐   │
│   │              同步数据                     │   │
│   │  • Agent 配置                            │   │
│   │  • 任务历史                              │   │
│   │  • Skill 版本                            │   │
│   │  • Evolution 建议                        │   │
│   │  • 自愈记录                              │   │
│   │  • 用户偏好                              │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

**同步策略：**
```
本地优先 (Local-First):
├── SQLite 本地存储（主）
├── WebSocket 实时推送（辅）
├── Offline Cache 离线缓存
└── Conflict Resolution 最后写入胜出
```

---

### 2.4 Hermes Rust 增强（Phase 4）

**目标：MCP + Skill 深度集成**

```
┌─────────────────────────────────────────────────┐
│           Hermes Rust 增强架构                    │
├─────────────────────────────────────────────────┤
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │           Hermes Core (Rust)             │   │
│   │  ──────────────────────────────────────  │   │
│   │  • Task Parser → DAG 构建               │   │
│   │  • Agent Matcher → 最佳匹配             │   │
│   │  • Orchestrator → 执行编排              │   │
│   │  • QA Validator → 结果验证              │   │
│   └────────────────┬────────────────────────┘   │
│                    │                            │
│                    ▼                            │
│   ┌─────────────────────────────────────────┐   │
│   │           MCP Integration                │   │
│   │  ──────────────────────────────────────  │   │
│   │  • MCP Filesystem → 文件操作            │   │
│   │  • MCP GitHub → PR/Issue 操作           │   │
│   │  • MCP PostgreSQL → 数据库查询          │   │
│   │  • MCP Memory → 记忆存储                │   │
│   └────────────────┬────────────────────────┘   │
│                    │                            │
│                    ▼                            │
│   ┌─────────────────────────────────────────┐   │
│   │           Skill System                   │   │
│   │  ──────────────────────────────────────  │   │
│   │  • 16 Core Skills → 不可变              │   │
│   │  • User Skills → 可进化                 │   │
│   │  • Skill Chains → 开发流程组合          │   │
│   │  • Version History → 回滚               │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Hermes Rust 新增 Trait：**
```rust
// 自进化 Trait
trait SelfEvolving {
    fn analyze_execution_result(&self, result: &ExecutionResult) -> EvolutionSuggestion;
    fn apply_evolution(&mut self, suggestion: &EvolutionSuggestion) -> bool;
    fn get_evolution_history(&self) -> Vec<EvolutionRecord>;
}

// 自修复 Trait  
trait SelfHealing {
    fn detect_failure_pattern(&self, error: &Error) -> FailurePattern;
    fn attempt_healing(&mut self, pattern: &FailurePattern) -> HealingResult;
    fn get_healing_stats(&self) -> HealingStats;
}

// MCP 集成 Trait
trait McpIntegrated {
    fn call_mcp_tool(&self, server: &str, tool: &str, params: Value) -> Result<Value>;
    fn list_mcp_tools(&self, server: &str) -> Result<Vec<ToolInfo>>;
    fn validate_mcp_access(&self, agent: &str, tool: &str) -> bool;
}

// Skill 调用 Trait
trait SkillCapable {
    fn invoke_skill(&self, skill: &str, params: Value) -> Result<SkillResult>;
    fn evolve_skill(&mut self, skill: &str, feedback: &SkillFeedback) -> Result<SkillVersion>;
    fn get_skill_history(&self, skill: &str) -> Result<Vec<SkillVersion>>;
}
```

---

## 3. 实施路线

### Phase 1: 首页改造（1-2 周）

**目标：提升首次体验，突出 Agent 配置**

| 任务 | 文件 | 优先级 |
|------|------|--------|
| 创建 Dashboard 首页 | `src/views/DashboardView.vue` | P0 |
| Agent 状态卡片组件 | `src/features/agents/AgentStatusCard.vue` | P0 |
| 快速任务输入组件 | `src/features/tasks/QuickTaskInput.vue` | P0 |
| 最近执行列表组件 | `src/features/history/RecentTasks.vue` | P1 |
| 首次引导流程 | `src/features/onboarding/OnboardingFlow.vue` | P1 |
| 功能层级调整 | `src/lib/feature-registry.ts` | P2 |
| 路由调整 | `src/router.ts` | P2 |

### Phase 2: 自进化闭环（2-3 周）

**目标：完整开发流程自动化**

| 任务 | 文件 | 优先级 |
|------|------|--------|
| 开发流程 Skill 链 | `src/lib/skill-system/dev-flow-skills.ts` | P0 |
| Evolution Engine 增强 | `src/lib/self-improvement/evolution-engine.ts` | P0 |
| Hermes 流程编排 | `src-tauri/src/hermes_flow.rs` | P0 |
| 主动修复逻辑 | `src/lib/self-improvement/self-healing.ts` | P1 |
| QA Agent 增强 | `src/lib/code-review-agent.ts` | P1 |

### Phase 3: 多端同步（2-3 周）

**目标：App/Tauri/Web 数据同步**

| 任务 | 文件 | 优先级 |
|------|------|--------|
| Sync Engine 设计 | `src-tauri/src/sync_engine.rs` | P0 |
| WebSocket Sync 协议 | `src/lib/sync-protocol.ts` | P0 |
| Flutter Sync Client | `acp_ui_flutter/lib/sync/` | P1 |
| Conflict Resolution | `src-tauri/src/conflict_resolver.rs` | P1 |
| Offline Sync Queue | `src/lib/sync-queue.ts` | P2 |

### Phase 4: Hermes Rust 增强（3-4 周）

**目标：MCP + Skill 深度集成**

| 任务 | 文件 | 优先级 |
|------|------|--------|
| SelfEvolving Trait | `src-tauri/src/traits/self_evolving.rs` | P0 |
| SelfHealing Trait | `src-tauri/src/traits/self_healing.rs` | P0 |
| McpIntegrated Trait | `src-tauri/src/traits/mcp_integrated.rs` | P0 |
| SkillCapable Trait | `src-tauri/src/traits/skill_capable.rs` | P0 |
| Hermes Trait 组合 | `src-tauri/src/hermes/enhanced_hermes.rs` | P1 |

---

## 4. 成功指标

### 4.1 首页改造

| 指标 | 当前 | 目标 |
|------|------|------|
| 首次配置 Agent 时间 | ~5 分钟 | < 1 分钟 |
| 首页功能可见性 | 7/25 | 100% 核心 |
| 用户首次任务成功率 | 未知 | > 90% |

### 4.2 自进化闭环

| 指标 | 当前 | 目标 |
|------|------|------|
| 开发流程自动化程度 | 部分 | 完整 |
| Evolution 建议→执行 | 手动 | 自动 |
| 自愈成功率 | ~30% | > 70% |

### 4.3 多端同步

| 指标 | 当前 | 目标 |
|------|------|------|
| 数据一致性 | 无 | 100% |
| 同步延迟 | 无 | < 1 秒 |
| 离线恢复能力 | 部分 | 完整 |

---

## 5. 附录

### 5.1 现有系统组件索引

```
Self-Improvement:
├── src/lib/self-improvement/evolution-engine.ts
├── src/lib/self-improvement/self-healing.ts
├── src/lib/self-improvement/telemetry.ts
├── src/lib/self-improvement/adaptive-strategy.ts

Skill System:
├── src/lib/skill-system/types.ts
├── src/features/plugins/skills/SkillManager.vue
├── src/features/plugins/skills/SkillExecutor.vue
├── src/features/plugins/skills/SkillVersionHistory.vue

Hermes:
├── src/lib/hermes-api.ts
├── src/lib/task-parser.ts
├── src/lib/agent-matcher.ts
├── src/lib/orchestrator.ts
├── src/features/hermes/HermesDashboard.vue

Agent Config:
├── src/views/AgentConfigView.vue
├── src/lib/agent-templates.ts
├── src/stores/config.ts
```

### 5.2 关键决策记录

| 决策 | 原因 | 日期 |
|------|------|------|
| 远程 Agent 暂不实现 | 本地沙箱完整，Cloud Agent 后期规划 | 2026-06-06 |
| 首页优先于功能优化 | 首次体验决定用户留存 | 2026-06-06 |
| Skill 链式调用 | 开发流程需要多个 Skill 协作 | 2026-06-06 |
| Rust Trait 扩展 | 保持 Hermes 核心稳定，增强可扩展 | 2026-06-06 |

---

**文档状态：草稿 → 待评审**
**下一步：用户确认方向 → 开始 Phase 1 实施**