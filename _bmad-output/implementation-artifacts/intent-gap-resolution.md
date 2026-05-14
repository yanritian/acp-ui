# Intent Gap Resolution - 规格补充定义

## BMad Code Review发现的Intent Gap

审查发现以下规格意图不完整的问题，需要补充定义。

---

## #14: Prompt模式交互流程

### 问题
`Prompt` PermissionMode返回布尔值`ask_override`而非阻塞式用户确认。
没有UI交互或异步用户确认触发。

### 现状代码
```rust
// permission_checker.rs:133-139
PermissionMode::Prompt => {
    return PermissionResult {
        allowed: self.config.ask_override,
        reason: "Prompt mode: user decision required".to_string(),
        matched_rule: Some("mode-Prompt".to_string()),
    };
}
```

### 补充定义

Prompt模式应实现以下交互流程：

#### Desktop App交互
```
1. Agent请求执行Tool
2. PermissionChecker检测到Prompt模式
3. 返回PermissionResult { allowed: false, reason: "pending_user_approval", needs_prompt: true }
4. App弹出PermissionDialog:
   - Tool名称
   - 参数摘要
   - [允许] [拒绝] [始终允许此类型]
5. 用户选择后，PermissionChecker更新状态
6. Agent继续/中止执行
```

#### 远程渠道交互 (飞书/Telegram/Discord)
```
1. Agent请求执行Tool
2. PermissionChecker返回needs_prompt=true
3. IMGateway发送交互卡片:
   - 飞书: Interactive Card with buttons
   - Telegram: Inline Keyboard
   - Discord: Embed with buttons
4. 用户点击按钮
5. 回调更新PermissionChecker
6. Agent继续执行
```

#### API设计
```rust
pub struct PermissionResult {
    pub allowed: bool,
    pub reason: String,
    pub matched_rule: Option<String>,
    pub needs_prompt: bool,        // NEW
    pub prompt_message: Option<String>,  // NEW
    pub prompt_options: Vec<PromptOption>,  // NEW
}

pub struct PromptOption {
    pub option_id: String,
    pub label: String,
    pub action: PromptAction,  // Allow, Deny, AlwaysAllow
}
```

---

## #18: PRD功能范围定义

### 问题
PRD定义的MultiAgentBridge、TaskQueue、Telegram/Discord网关均未实现。
SQLite历史表（tasks、agent_executions、conversations、sync_records）也未创建。

### 当前版本交付范围

| 功能 | 状态 | 交付版本 |
|------|------|----------|
| **已实现** |||
| 单Agent连接 (ACP) | ✅ | v0.1.x |
| Agent配置管理 | ✅ | v0.1.x |
| WebSocket远程访问 | ✅ | v0.1.x |
| Session持久化 | ✅ | v0.1.x |
| Permission检查 (5级) | ✅ | v0.1.x |
| Hooks执行 | ✅ | v0.1.x |
| LogStream日志 | ✅ | v0.1.x |
| 自修复系统 | ✅ | v0.1.x |
| 自进化引擎 | ✅ | v0.1.x |
| 飞书Rich Message | ✅ | v0.1.x |
| Hermes Dashboard | ✅ | v0.1.x |
| **延后** |||
| MultiAgentBridge | ⏳ | v0.2.0 |
| TaskQueue优先级 | ⏳ | v0.2.0 |
| Telegram Gateway | ⏳ | v0.3.0 |
| Discord Gateway | ⏳ | v0.3.0 |
| PRD历史表完整schema | ⏳ | v0.2.0 |
| Agent互相监督 | ⏳ | v0.2.0 |
| DAG可视化编排 | ⏳ | v0.2.0 |

### v0.2.0补充计划

```
Phase 1: 多Agent核心
├── MultiAgentBridge类实现
│   ├── prompt(agentId, text)
│   ├── broadcast(text) → StitchedResponse
│   └── route(text, strategy) → 智能路由
├── TaskQueue优先级队列
│   ├── PriorityQueue<T>
│   ├── Task调度
│   └── 依赖检测
└── Agent互相监督
    ├── SupervisorAgent
    ├── CrossCheck机制
    └── QualityReport

Phase 2: PRD历史系统
├── SQLite表完整实现:
│   ├── tasks (id, name, status, created_at, completed_at, source, error_message)
│   ├── agent_executions (id, task_id, agent_name, role, status, start_time, end_time)
│   ├── conversations (id, task_id, timestamp, speaker, message)
│   └── sync_records (id, task_id, channel, status, synced_at)
├── HistoryStore:
│   ├── saveTask(record)
│   ├── queryTasks(filter)
│   ├── search(keyword)
│   └── export(format)
└── 全文搜索 (FTS5)

Phase 3: 远程Gateway完整
├── TelegramGateway:
│   ├── onMessage(msg)
│   ├── sendKeyboard(chatId, options)
│   └── /history命令
├── DiscordGateway:
│   ├── onMessage(message)
│   ├── Slash Commands
│   └── Embed消息
└── 跨渠道SessionSync
```

---

## 规格修正后的Acceptance Criteria

### Permission Prompt交互 (v0.1.5)

- [ ] Prompt模式返回needs_prompt=true
- [ ] App弹出PermissionDialog
- [ ] 飞书发送Interactive Card
- [ ] Telegram发送Inline Keyboard
- [ ] 用户选择更新Permission状态

### 多Agent协作 (v0.2.0)

- [ ] MultiAgentBridge可同时连接3+Agent
- [ ] broadcast()返回StitchedResponse
- [ ] route()根据任务类型选择最佳Agent
- [ ] TaskQueue支持优先级和依赖检测
- [ ] SupervisorAgent实现CrossCheck

### 历史系统完整 (v0.2.0)

- [ ] SQLite包含4张PRD定义表
- [ ] HistoryStore支持query/search/export
- [ ] FTS5全文搜索工作
- [ ] 跨渠道同步记录完整

---

**版本**: v1.0
**日期**: 2026-05-14
**状态**: 规格补充完成