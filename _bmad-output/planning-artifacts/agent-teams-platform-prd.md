# Agent Teams Platform 产品设计文档

> 本地部署的多代理协作平台，融合 Warp + BMad，支持 App/飞书/Telegram/Discord 远程控制

---

## 目录

1. [产品概述](#1-产品概述)
2. [系统架构](#2-系统架构)
3. [核心模块](#3-核心模块)
4. [智能编排流程](#4-智能编排流程)
5. [展示系统](#5-展示系统)
6. [远程控制](#6-远程控制)
7. [历史记录](#7-历史记录)
8. [技术选型](#8-技术选型)
9. [实现计划](#9-实现计划)
10. [文件结构](#10-文件结构)

---

## 1. 产品概述

### 产品定位

与 Warp Oz 云平台不同，Agent Teams Platform 采用**本地部署**模式：

| 维度 | Warp Oz | Agent Teams Platform |
|------|---------|---------------------|
| 部署模式 | 云平台 | 本地机器 |
| 数据隐私 | 云端存储 | 本地 SQLite |
| 远程控制 | Web/App | 飞书/TG/Discord |
| 许可证 | AGPL v3 | 自研无限制 |
| 定制性 | 受限 | 完全自主 |

### 核心目标

1. **多代理缝合会话** - Claude Code + Codex 同时工作，统一展示
2. **智能编排** - Codex 分析需求 → 用户确认 → 多Agent协作执行
3. **工作流引擎** - skill.md 定义任务流程，自动执行
4. **Agent Teams** - 角色分工，互相监督，协作完成复杂任务
5. **远程控制** - 飞书/Telegram/Discord 多渠道统一接入
6. **历史追溯** - 所有任务可搜索、查看、导出

---

## 2. 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                     智能编排流程                                 │
│                                                                 │
│  Phase 1: 需求接收        Phase 2: 计划确认                     │
│  ┌───────────────┐       ┌───────────────┐                     │
│  │ 远程需求      │       │ 执行计划展示   │                     │
│  │ PRD/图片/文本 │  ──►  │ DAG + Agent分配│  ──► 用户确认       │
│  │               │       │               │                     │
│  │ Codex 分析    │       │ App + 远程卡片 │                     │
│  └───────────────┘       └───────────────┘                     │
│                                                                 │
│  Phase 3: 多Agent执行     Phase 4: 结果整合                     │
│  ┌───────────────┐       ┌───────────────┐                     │
│  │ Warp终端启动  │       │ 输出聚合      │                     │
│  │ Orchestrator  │  ──►  │ 质量检查      │  ──► 反馈用户       │
│  │ 动态编排      │       │ 历史保存      │                     │
│  │ Agent互相监督 │       │ 同步远程      │                     │
│  └───────────────┘       └───────────────┘                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    本地核心服务                                  │
│  ┌─────────────────────────────────────────────────────┐       │
│  │            AgentTeamsService (核心)                  │       │
│  │  ├── MultiAgentBridge (多代理连接)                   │       │
│  │  ├── WorkflowEngine (skill.md 执行)                  │       │
│  │  ├── SessionManager (会话缝合)                       │       │
│  │  ├── TaskQueue (任务队列/优先级)                      │       │
│  │  ├── Orchestrator (动态编排)                         │       │
│  │  ├── SupervisorAgent (互相监督)                      │       │
│  │  └── RealtimeSyncService (状态广播)                  │       │
│  └─────────────────────────────────────────────────────┘       │
│                           │                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐       │
│  │Claude    │ │Codex     │ │Gemini    │ │ 本地 LLM      │       │
│  │Code      │ │CLI       │ │CLI       │ │ (可选)        │       │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
        ┌─────┴─────┐ ┌─────┴─────┐ ┌─────┴─────┐
        │ Tauri App │ │ IM Gateway│ │ Web UI    │
        │ (本地GUI) │ │ (远程控制)│ │ (可选)    │
        └───────────┘ └───────────┘ └───────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
        ┌─────┴─────┐ ┌─────┴─────┐ ┌─────┴─────┐
        │ 飞书 Bot  │ │ TG Bot    │ │ Discord   │
        │           │ │           │ │ Bot       │
        └───────────┘ └───────────┘ └───────────┘
```

### 数据流

```
用户在飞书发送："帮我实现登录功能，PRD见附件"
    │
    ▼
飞书 Gateway 接收 → IMGateway 统一处理
    │
    ▼
Phase 1: Codex Analyzer 分析
├── 解析 PRD 内容
├── 拆解子任务
├── 分配 Agent 角色
└── 生成执行计划 (DAG)
    │
    ▼
Phase 2: 执行计划展示
├── 飞书卡片展示计划
├── App 同步展示
└── 用户点击 [批准执行]
    │
    ▼
Phase 3: Warp终端启动多Agent协作
├── Orchestrator 按DAG调度
├── Claude Code 执行 UI 任务
├── Codex 执行 Auth 任务
├── SupervisorAgent 互相监督
└── RealtimeSyncService 广播进度
    │
    ▼
Phase 4: 结果整合
├── 输出聚合
├── 质量检查
├── 生成文件清单
└── 反馈用户
    │
    ▼
回复发送：
├── 飞书原频道 (完成卡片)
├── App 完整报告
├── 同步到 Telegram/Discord
└── 保存到历史记录
```

---

## 3. 核心模块

### 3.1 MultiAgentBridge (多代理桥接)

将单代理连接扩展为多代理：

```typescript
// src/lib/multi-agent/bridge.ts
class MultiAgentBridge {
  connections: Map<string, AcpClientBridge>  // 多个代理实例
  
  // 向指定代理发送消息
  async prompt(agentId: string, text: string): Promise<Response>
  
  // 广播给所有代理（缝合会话核心）
  async broadcast(text: string): Promise<StitchedResponse>
  
  // 智能路由（根据任务类型选择最佳代理）
  async route(text: string, strategy: RoutingStrategy): Promise<Response>
}
```

**缝合会话策略**：

```typescript
async handleUserMessage(text: string) {
  const taskType = classifyTask(text)  // 'ui' | 'algorithm' | 'general' | 'complex'
  
  switch (taskType) {
    case 'ui':
      return bridge.prompt('claude-code', text)
    case 'algorithm':
      return bridge.prompt('codex', text)
    case 'complex':
      return bridge.broadcast(text)  // 多代理同时工作
    default:
      return bridge.route(text, 'auto')
  }
}
```

### 3.2 AgentTeamsService (本地核心服务)

```typescript
// src/lib/core/agent-teams-service.ts
class AgentTeamsService {
  private bridge: MultiAgentBridge
  private workflow: WorkflowEngine
  private orchestrator: Orchestrator
  private queue: TaskQueue
  
  // 启动本地服务
  async start(config: ServiceConfig): Promise<void>
  
  // 处理来自任意渠道的消息
  async handleMessage(source: MessageSource, text: string): Promise<Response>
  
  // 执行工作流
  async runWorkflow(skillId: string, trigger: TriggerSource): Promise<Result>
  
  // 获取状态（供各渠道展示）
  getStatus(): ServiceStatus
}
```

### 3.3 SkillEngine (工作流引擎)

```typescript
// src/lib/orchestration/skill-engine.ts
interface SkillDefinition {
  name: string
  trigger: 'manual' | 'cron' | 'webhook'
  steps: SkillStep[]
}

class SkillEngine {
  async load(path: string): Promise<SkillDefinition>
  async execute(skill: SkillDefinition): Promise<Result>
  async schedule(skillId: string, cron: string): Promise<void>
}
```

### 3.4 IMGateway (即时通讯网关)

```typescript
// src/lib/gateway/im-gateway.ts
interface MessageSource {
  channel: 'app' | 'feishu' | 'telegram' | 'discord' | 'web'
  userId: string
  sessionId?: string
}

class IMGateway {
  private service: AgentTeamsService
  
  registerFeishu(config: FeishuConfig): void
  registerTelegram(config: TGConfig): void
  registerDiscord(config: DiscordConfig): void
  
  async handleIncoming(source: MessageSource, message: string): Promise<void>
  async sendReply(source: MessageSource, response: string): Promise<void>
}
```

### 3.5 SessionSync (跨渠道会话同步)

```typescript
// src/lib/core/session-sync.ts
class SessionSync {
  // 同一用户在不同渠道共享会话上下文
  // App 发起任务 → 飞书继续对话 → Discord 查看结果
  
  private sessions: Map<string, UnifiedSession>
  
  getSession(userId: string): UnifiedSession
  syncSession(session: UnifiedSession): Promise<void>
}
```

---

## 4. 智能编排流程

### 4.1 Phase 1: 需求接收与分析

```typescript
// src/lib/orchestration/input-analyzer.ts
interface IncomingRequest {
  source: 'feishu' | 'telegram' | 'discord' | 'app'
  userId: string
  text?: string
  attachments?: Attachment[]  // PRD、图片、代码等
  projectContext?: ProjectContext
}

class InputAnalyzer {
  async analyze(input: IncomingRequest): Promise<AnalysisResult> {
    const requirements = await this.extractRequirements(input)
    const attachmentAnalysis = await this.analyzeAttachments(input.attachments)
    const taskType = this.classifyTaskType(requirements, attachmentAnalysis)
    
    return { requirements, attachmentAnalysis, taskType }
  }
  
  // PRD分析：提取功能列表、技术要求
  private async analyzePRD(content: string): Promise<PRDAnalysis>
  
  // 图片分析：识别设计元素、组件结构
  private async analyzeImage(content: string): Promise<ImageAnalysis>
}
```

**Codex 编排Agent**：

```typescript
// src/lib/orchestration/planner-agent.ts
class PlannerAgent {
  private codexClient: AcpClientBridge
  
  async planTask(analysis: AnalysisResult): Promise<ExecutionPlan> {
    const prompt = this.buildPlanningPrompt(analysis)
    const response = await this.codexClient.prompt({ sessionId, prompt })
    return this.parsePlanResponse(response)
  }
}
```

**执行计划结构**：

```typescript
interface ExecutionPlan {
  id: string
  title: string
  tasks: PlannedTask[]
  agentAssignments: AgentAssignment[]
  dependencyGraph: DependencyGraph  // DAG
  estimation: {
    totalTime: string
    riskLevel: 'low' | 'medium' | 'high'
  }
}

interface PlannedTask {
  id: string
  name: string
  assignee: AgentRole  // 'claude-code' | 'codex' | 'gemini'
  dependsOn: string[]
  estimatedTime: string
}
```

### 4.2 Phase 2: 计划反馈与确认

**飞书执行计划卡片**：

```typescript
function buildPlanCard(plan: ExecutionPlan): FeishuCard {
  return {
    header: { title: `执行计划: ${plan.title}`, template: 'blue' },
    elements: [
      // 任务列表
      { tag: 'div', text: { content: plan.tasks.map(t => 
        `• ${t.id}: ${t.name} → @${t.assignee}`
      ).join('\n') } },
      
      // 确认按钮
      { tag: 'action', actions: [
        { tag: 'button', text: '批准执行', type: 'primary' },
        { tag: 'button', text: '调整计划', type: 'default' },
        { tag: 'button', text: '取消', type: 'default' }
      ]}
    ]
  }
}
```

**用户交互处理**：

```typescript
// src/lib/orchestration/plan-handler.ts
class PlanHandler {
  async handleConfirmation(
    planId: string,
    action: 'approve' | 'modify' | 'cancel' | 'question'
  ): Promise<void> {
    switch (action) {
      case 'approve':
        await this.startExecution(plan)
      case 'modify':
        await this.applyModifications(plan, modifications)
      case 'cancel':
        await this.planStore.delete(planId)
      case 'question':
        await this.plannerAgent.answerQuestion(plan, question)
    }
  }
}
```

### 4.3 Phase 3: 多Agent协作执行

**Warp终端集成**：

```typescript
// src/lib/execution/warp-executor.ts
class WarpExecutor {
  async execute(plan: ExecutionPlan): Promise<void> {
    await this.initWarpEnvironment()
    await this.startOrchestrator(plan)
    await this.launchAgents(plan.agentAssignments)
    await this.executeDAG(plan.dependencyGraph, plan.tasks)
  }
}
```

**动态编排控制器**：

```typescript
// src/lib/orchestration/orchestrator.ts
class Orchestrator {
  async executeDAG(graph: DependencyGraph, tasks: PlannedTask[]): Promise<void> {
    const sortedTasks = this.topologicalSort(graph)
    
    for (const taskId of sortedTasks) {
      if (!this.checkDependencies(task)) {
        await this.waitForDependencies(task)
      }
      
      const parallelTasks = this.findParallelTasks(taskId, sortedTasks)
      if (parallelTasks.length > 0) {
        await this.executeParallel([task, ...parallelTasks])
      } else {
        await this.executeTask(task)
      }
      
      await this.verifyCheckpoint(task)
      await this.broadcastProgress()
    }
  }
  
  // 动态调整：优先级变更、添加/移除Agent、暂停/跳过任务
  async adjustExecution(adjustment: ExecutionAdjustment): Promise<void>
  
  // 异常处理：重试、降级、中止、询问用户
  async handleError(error: ExecutionError): Promise<void>
}
```

### 4.4 Agent互相监督机制

```typescript
// src/lib/orchestration/supervisor.ts
class SupervisorAgent {
  private monitors: Map<string, AgentMonitor>
  
  async startSupervision(agents: string[]): Promise<void>
  
  // 交叉检查：让其他Agent检查某Agent的输出
  async crossCheck(taskId: string, outputs: AgentOutput[]): Promise<CrossCheckResult>
  
  // 质量检查：代码/文档质量评审
  async qualityCheck(output: AgentOutput): Promise<QualityReport>
  
  // 进度同步：广播给所有Agent和用户界面
  async syncProgress(): Promise<void>
  
  // 问题预警：超时、卡住、输出异常
  async detectIssues(): Promise<Issue[]>
}
```

**Agent Monitor**：

```typescript
// src/lib/orchestration/agent-monitor.ts
class AgentMonitor {
  recordActivity(activity: AgentActivity): void
  
  isTimeout(thresholdMs = 300000): boolean   // 5分钟无活动
  isStuck(thresholdMs = 60000): boolean      // 1分钟进度无变化
  hasAbnormalOutput(): boolean               // 输出包含错误或太短
  
  getProgress(): ProgressInfo
}
```

### 4.5 Phase 4: 结果整合与反馈

```typescript
// src/lib/orchestration/result-integrator.ts
class ResultIntegrator {
  async integrate(plan: ExecutionPlan, results: Map<string, TaskResult>): Promise<IntegratedResult> {
    const orderedOutputs = this.orderByPlan(plan, results)
    const qualityReport = await this.qualityCheck(orderedOutputs)
    const fileList = this.extractFiles(orderedOutputs)
    const summary = await this.generateSummary(orderedOutputs)
    
    return { planId, outputs, qualityReport, fileList, summary }
  }
  
  async feedbackToUser(result: IntegratedResult, source: string): Promise<void> {
    await this.sendAppReport(result)       // App完整报告
    await this.sendRemoteSummary(source, result)  // 远程摘要
    await this.saveToHistory(result)       // 历史保存
  }
}
```

---

## 5. 展示系统

### 5.1 GUI App 主界面

```
┌─────────────────────────────────────────────────────────────┐
│  Agent Teams Console                        [设置] [历史]   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐  ┌─────────────────────────────┐  │
│  │ 左侧面板            │  │ 主聊天区域                   │  │
│  │                     │  │                             │  │
│  │ ┌─ 代理状态 ──────┐ │  │  用户: 实现登录功能         │  │
│  │ │ 🟢 Claude Code  │ │  │                             │  │
│  │ │    状态: 工作中  │ │  │  ┌─ Agent Team 响应 ──────┐│  │
│  │ │    任务: UI设计  │ │  │  │ Claude: ████████░░ 80% ││  │
│  │ │ 🟢 Codex        │ │  │  │ Codex: ██████░░░░ 60% ││  │
│  │ │    任务: Auth   │ │  │  └────────────────────────┘│  │
│  │ │ ⚪ Gemini 空闲   │ │  │                             │  │
│  │ └─────────────────┘ │  │  ┌─ 实时监控面板 ─────────┐│  │
│  │                     │  │  │ 开始: 10:00             ││  │
│  │ ┌─ 工作流 ─────────┐│  │  │ 耗时: 03:45            ││  │
│  │ │ Step 2/5  40%    ││  │  │ 🔗 同步: 飞书✅ TG✅   ││  │
│  │ └─────────────────┘│  │  └────────────────────────┘│  │
│  │                     │  │                             │  │
│  │ ┌─ 任务队列 ──────┐│  │  [发送消息]                  │  │
│  │ │ 1.登录 ⚡ 2.用户⏳││  │                             │  │
│  │ └─────────────────┘│  └─────────────────────────────┘  │
│  └─────────────────────┘                                   │
│                                                             │
│  输入框: _______________________________________ [发送]     │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 实时同步系统

```typescript
// src/lib/sync/realtime-sync.ts
class RealtimeSyncService {
  async broadcastStatus(status: TaskStatus): Promise<void> {
    await this.updateApp(status)                    // Tauri IPC
    await this.updateFeishuCard(status.cardId, status)  // 飞书卡片
    await this.updateTelegramMessage(status.msgId, status)  // TG消息
    await this.updateDiscordEmbed(status.msgId, status)  // Discord Embed
    await this.historyStore.appendLog(status)       // 历史记录
  }
}
```

**同步时机**：

| 事件 | 同步频率 |
|------|----------|
| TASK_START | 立即 |
| AGENT_PROGRESS | 每5秒最多一次 |
| AGENT_COMPLETE | 立即 |
| TASK_COMPLETE | 立即 |
| ERROR | 立即 |
| USER_INPUT_REQUIRED | 立即 |

---

## 6. 远程控制

### 6.1 飞书 Bot

```typescript
// src/lib/gateway/feishu.ts
class FeishuGateway {
  async onMessage(event: FeishuEvent): Promise<void> {
    const source = { channel: 'feishu', userId: event.user_id }
    const response = await this.service.handleMessage(source, event.message)
    await this.bot.sendMessage(event.chat_id, response.text)
  }
  
  // 实时状态卡片
  async sendCard(chatId: string, status: TaskStatus): Promise<void>
  
  // /历史 命令
  async handleHistoryCommand(userId: string): Promise<void>
}
```

### 6.2 Telegram Bot

```typescript
// src/lib/gateway/telegram.ts
class TelegramGateway {
  async onMessage(msg: TelegramMessage): Promise<void> {
    const source = { channel: 'telegram', userId: msg.from.id.toString() }
    const response = await this.service.handleMessage(source, msg.text)
    await this.bot.sendMessage(msg.chat.id, response.text)
  }
  
  // Inline Keyboard 快捷操作
  async sendKeyboard(chatId: number, options: ActionOptions): Promise<void>
  
  // /history 命令
  async handleHistoryCommand(userId: string): Promise<void>
}
```

### 6.3 Discord Bot

```typescript
// src/lib/gateway/discord.ts
class DiscordGateway {
  async onMessage(message: DiscordMessage): Promise<void> {
    const source = { channel: 'discord', userId: message.author.id }
    const response = await this.service.handleMessage(source, message.content)
    await message.reply(response.text)
  }
  
  // Slash Commands: /history, /status, /run
  async registerCommands(): Promise<void>
}
```

---

## 7. 历史记录

### 7.1 SQLite 表结构

```sql
-- 任务表
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  source TEXT NOT NULL,
  error_message TEXT
);

-- 代理执行表
CREATE TABLE agent_executions (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  agent_name TEXT NOT NULL,
  role TEXT,
  status TEXT NOT NULL,
  start_time INTEGER NOT NULL,
  end_time INTEGER,
  output_files TEXT,
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- 对话记录表
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  speaker TEXT NOT NULL,
  message TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- 同步记录表
CREATE TABLE sync_records (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  channel TEXT NOT NULL,
  status TEXT NOT NULL,
  synced_at INTEGER NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- 全文搜索索引
CREATE INDEX idx_tasks_search ON tasks USING FTS5(name, error_message);
```

### 7.2 HistoryStore

```typescript
// src/lib/storage/history-store.ts
class HistoryStore {
  async saveTask(record: TaskRecord): Promise<void>
  async queryTasks(filter: HistoryFilter): Promise<TaskRecord[]>
  async getTaskDetail(taskId: string): Promise<TaskRecord>
  async search(keyword: string): Promise<TaskRecord[]>
  async getStatistics(): Promise<TaskStatistics>
  async export(format: 'json' | 'csv' | 'markdown'): Promise<string>
}
```

### 7.3 历史界面

```
┌─────────────────────────────────────────────────────────────┐
│  历史记录                              [搜索] [筛选] [导出]  │
├─────────────────────────────────────────────────────────────┤
│  筛选: [全部] [成功] [失败]  时间: [今天] [本周]            │
│                                                             │
│  2026-05-03 10:00  ✅ 登录功能实现                          │
│  Claude Code + Codex | 耗时: 5分钟 | 文件: 2                │
│  [查看详情] [重新执行]                                      │
│                                                             │
│  2026-05-03 09:30  ✅ 用户管理模块                          │
│  Agent Team (4代理) | 耗时: 15分钟 | 文件: 8                │
│  [查看详情] [重新执行]                                      │
│                                                             │
│  2026-05-02 14:00  ❌ API对接                               │
│  Claude Code | 耗时: 2分钟 | 错误: 认证失败                 │
│  [查看详情] [重试]                                          │
└─────────────────────────────────────────────────────────────┘
```

---

## 8. 技术选型

| 用途 | 选择 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 跨平台容器 | Tauri 2 (Rust) |
| 状态管理 | Pinia |
| 本地数据库 | SQLite (better-sqlite3) |
| 飞书 SDK | `@larksuiteoapi/node-sdk` |
| Telegram SDK | `node-telegram-bot-api` |
| Discord SDK | `discord.js` |
| 工作流校验 | zod |
| DAG可视化 | dagre |
| 调度 | cron-parser |
| 全文搜索 | SQLite FTS5 |

---

## 9. 实现计划

### Phase 1: 本地核心服务 (1周)

```
目标: 多代理基础架构
├── MultiAgentBridge 类
├── AgentTeamsService
├── AgentPoolStore (状态管理)
├── TaskQueue (任务队列)
└── Tauri 集成启动/停止
```

### Phase 2: IM Gateway (1周)

```
目标: 飞书 + Telegram 接入
├── IMGateway 抽象层
├── FeishuGateway 实现
├── TelegramGateway 实现
└── 跨渠道会话同步 (SessionSync)
```

### Phase 3: 智能编排 (2周)

```
目标: Codex编排 → 确认 → 执行
├── InputAnalyzer (需求分析)
├── PlannerAgent (Codex编排)
├── ExecutionPlanView (App UI)
├── 飞书/TG计划卡片
├── PlanHandler (用户交互)
└── WarpExecutor (终端集成)
```

### Phase 4: 多Agent协作执行 (2周)

```
目标: 动态编排 + 互相监督
├── Orchestrator (DAG执行)
├── SupervisorAgent (监督机制)
├── AgentMonitor (状态监控)
├── ResultIntegrator (结果整合)
├── RealtimeSyncService (实时同步)
└── 异常处理/重试机制
```

### Phase 5: Discord + 历史系统 (1周)

```
目标: Discord接入 + 历史追溯
├── DiscordGateway 实现
├── HistoryStore (SQLite)
├── HistoryView (App UI)
├── 远程 /历史 命令
└── 导出功能
```

### Phase 6: 工作流引擎 (2周)

```
目标: skill.md 执行
├── SkillEngine
├── BMad skills 集成
├── WorkflowView 组件
├── cron/webhook 触发
└── 工作流进度展示
```

---

## 10. 文件结构

```
src/
├── lib/
│   ├── core/
│   │   ├── agent-teams-service.ts   # 本地核心服务
│   │   ├── multi-agent-bridge.ts    # 多代理连接
│   │   ├── session-sync.ts          # 跨渠道同步
│   │   └── task-queue.ts            # 任务队列
│   │
│   ├── orchestration/
│   │   ├── input-analyzer.ts        # 需求分析
│   │   ├── planner-agent.ts         # Codex编排
│   │   ├── plan-handler.ts          # 计划处理
│   │   ├── orchestrator.ts          # 动态编排
│   │   ├── warp-executor.ts         # Warp终端
│   │   ├── supervisor.ts            # 监督Agent
│   │   ├── agent-monitor.ts         # Agent监控
│   │   ├── result-integrator.ts     # 结果整合
│   │   ├── skill-engine.ts          # 工作流引擎
│   │   └── types/
│   │       ├── plan.ts
│   │       ├── task.ts
│   │       └── result.ts
│   │
│   ├── gateway/
│   │   ├── im-gateway.ts            # IM网关抽象
│   │   ├── feishu.ts                # 飞书实现
│   │   ├── telegram.ts              # TG实现
│   │   ├── discord.ts               # Discord实现
│   │   └── history-command.ts       # 远程历史命令
│   │
│   ├── sync/
│   │   ├── realtime-sync.ts         # 实时同步
│   │   ├── status-manager.ts        # 状态管理
│   │   └── broadcast.ts             # 广播通道
│   │
│   └── storage/
│   │   ├── history-store.ts         # 历史存储
│   │   ├── sqlite.ts                # SQLite封装
│   │   └── export.ts                # 导出功能
│   │
│   └── multi-agent/
│   │   ├── bridge.ts                # 多代理桥接
│   │   └── routing.ts               # 路由策略
│
├── stores/
│   ├── agent-pool.ts                # 代理池状态
│   ├── service.ts                   # 服务状态
│   ├── workflow.ts                  # 工作流状态
│   ├── history.ts                   # 历史状态
│   ├── realtime.ts                  # 实时状态
│   └── sync.ts                      # 同步状态
│
├── components/
│   ├── AppLayout.vue                # 主布局
│   ├── MultiAgentChat.vue           # 缝合会话视图
│   ├── AgentStatusPanel.vue         # 代理状态面板
│   ├── RealtimeMonitor.vue          # 实时监控
│   ├── WorkflowProgress.vue         # 工作流进度
│   ├── TaskQueuePanel.vue           # 任务队列
│   ├── ExecutionPlanView.vue        # 执行计划确认
│   ├── HistoryView.vue              # 历史记录
│   ├── TaskDetailModal.vue          # 任务详情弹窗
│   ├── SyncStatusIndicator.vue      # 同步状态
│   └── WorkflowView.vue             # 工作流管理
│
├── database/
│   ├── schema.sql                   # SQLite表结构
│   └── migrations/                  # 迁移脚本
│
└── web/                             # 可选Web UI
    └── index.html
    └── app.ts
```

---

## 11. BMad 集成

项目已安装 BMad-Method (v6.6.0)，提供以下集成点：

| BMad 资源 | 集成方式 |
|-----------|----------|
| `bmad-quick-dev` | 快速开发工作流模板 |
| `bmad-create-story` | 用户故事创建流程 |
| `bmad-dev-story` | 故事开发执行 |
| `bmad-code-review` | 代码审查流程 |
| Agent 角色 (pm/architect/dev/qa) | Agent Teams 成员模板 |

将 BMad skill 的 `SKILL.md` 解析为 `SkillDefinition`，直接作为工作流定义使用。

---

## 12. 完整流程示例

```
用户在飞书发送:
"帮我实现用户管理模块，PRD见附件"

↓ Phase 1: 分析

Codex Analyzer 分析:
• PRD: 10页，包含CRUD需求
• 任务类型: 全栈开发
• 复杂度: 中等

生成执行计划 (DAG):
Task 1: API设计 → Codex (依赖: 无)
Task 2: 数据库模型 → Codex (依赖: Task 1)
Task 3: 后端实现 → Claude Code (依赖: Task 2)
Task 4: 前端实现 → Claude Code (依赖: Task 3)
Task 5: 测试 → Gemini (依赖: Task 3, 4)

↓ Phase 2: 确认

飞书卡片展示计划
App 同步展示 DAG 图
用户点击 [批准执行]

↓ Phase 3: 执行

Warp 终端启动
├── Codex 开始 Task 1, 2 (并行)
├── Task 2 完成 → 自动启动 Claude Code Task 3
├── SupervisorAgent 监控所有Agent
├── 交叉检查 Task 3 输出质量
├── Task 3 完成 → 自动启动 Task 4
├── Gemini 同时执行 Task 5
├── RealtimeSyncService 广播进度到飞书/App
└── 全部完成

↓ Phase 4: 整合

结果汇总:
• 生成文件: 12个
• 质量评分: 95/100
• 耗时: 15分钟

反馈:
• App: 完整报告 + 文件预览
• 飞书: 完成卡片 + 文件列表
• Telegram/Discord: 同步通知
• 历史记录: 自动保存，可搜索/导出
```

---

## 附录

### 配置示例

```json
// service-config.json
{
  "agents": ["claude-code", "codex", "gemini"],
  "gateway": {
    "feishu": { "app_id": "...", "app_secret": "..." },
    "telegram": { "bot_token": "..." },
    "discord": { "bot_token": "...", "guild_id": "..." }
  },
  "storage": {
    "history": "./data/history.db",
    "retentionDays": 30
  },
  "port": 3000
}
```

### 快速开始

```bash
# 1. 启动本地服务
npm run dev

# 2. 配置飞书 Bot
# 创建飞书应用 → 获取 app_id/app_secret
# 配置 service-config.json

# 3. 配置 Telegram Bot
# @BotFather 创建 → 获取 bot_token

# 4. 测试
# 飞书发送消息 → Codex分析 → 确认 → 多Agent执行 → 收到结果
```

---

**版本**: v1.0
**日期**: 2026-05-03
**状态**: 设计完成，待开发