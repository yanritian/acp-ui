---
name: realtime-agent-orchestration
description: 实时可视化、可干预的多Agent编排系统架构
type: project
---

## 核心需求

1. **实时可视化** - 不是黑盒，每个Agent的状态实时展现
2. **可干预** - 需求变化时可暂停/修改/取消
3. **动画效果** - 流程图式的动态展示
4. **会话流** - 每个Agent的消息实时滚动
5. **Plan展示** - Team编排计划可视化

## 系统架构

### 后端组件

1. **AgentEventSystem** (新增)
   - Agent状态枚举: Idle → Starting → Running → Paused → Completed → Error
   - 事件类型: StatusChanged, MessageReceived, ProgressUpdated, PlanUpdated
   - 实时推送: 通过Tauri emit到前端

2. **InterventionAPI** (新增)
   - pause_agent(agent_id) - 暂停某个Agent
   - resume_agent(agent_id) - 继续执行
   - cancel_agent(agent_id) - 取消Agent
   - modify_task(task_id, new_prompt) - 修改任务指令
   - inject_message(agent_id, message) - 插入消息到Agent会话

3. **TeamExecutionTracker** (新增)
   - 记录Team的执行进度
   - Plan节点状态跟踪
   - Agent间依赖关系可视化

### 前端组件

1. **TeamOrchestrationView** (新组件)
   - 流程图可视化 (DAG图)
   - 每个节点显示Agent状态动画
   - 连线显示数据流向动画

2. **AgentSessionPanel** (新组件)
   - 单个Agent的会话流
   - 实时消息滚动
   - 状态指示器 (颜色/动画)

3. **PlanVisualization** (新组件)
   - Team的编排计划展示
   - 节点状态实时更新
   - 进度条/动画效果

4. **InterventionControls** (新组件)
   - 悬浮控制面板
   - 暂停/继续/取消按钮
   - 修改指令输入框

## 数据结构

```rust
// Agent状态
enum AgentStatus {
    Idle,
    Starting,
    Running,
    Paused,
    Completed,
    Error,
}

// 事件类型
enum AgentEvent {
    StatusChanged { agent_id: String, status: AgentStatus },
    MessageReceived { agent_id: String, role: String, content: String },
    ProgressUpdated { agent_id: String, progress: f32 },
    PlanUpdated { team_id: String, plan: ExecutionPlan },
}

// 执行计划节点
struct PlanNode {
    id: String,
    agent_name: String,
    status: AgentStatus,
    dependencies: Vec<String>,
    started_at: Option<DateTime>,
    completed_at: Option<DateTime>,
}
```

## UI设计

### 主视图布局
```
┌────────────────────────────────────────────────────┐
│  Team Orchestration Dashboard                       │
├────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────┐    ┌─────────┐         │
│  │ Agent A │───▶│ Agent B │───▶│ Agent C │         │
│  │ ●运行中 │    │ ○等待   │    │ ○空闲   │         │
│  └─────────┘    └─────────┘    └─────────┘         │
│      ↓              ↓                              │
│  ┌──────────────────────────────────────────┐      │
│  │ Agent A Session                          │      │
│  │ ├ User: 请帮我分析代码                    │      │
│  │ ├ Claude: 我正在分析... ████████░ 80%    │      │
│  │ └ 最新消息实时滚动...                     │      │
│  └──────────────────────────────────────────┘      │
├────────────────────────────────────────────────────┤
│  [暂停] [继续] [取消] [修改指令] [插入消息]          │
└────────────────────────────────────────────────────┘
```

### 状态动画
- Idle: 灰色静态圆点
- Starting: 黄色闪烁
- Running: 蓝色脉冲动画 + 进度条
- Paused: 橙色闪烁
- Completed: 绿色静态 + 完成标记
- Error: 红色闪烁 + 错误标记

## 实现步骤

1. ✅ 分析现有代码结构
2. 🔄 创建AgentEventSystem
3. 🔄 添加InterventionAPI命令
4. 🔄 创建TeamOrchestrationView组件
5. 🔄 创建AgentSessionPanel组件
6. 🔄 实现动画效果
7. 🔄 WebSocket远程同步

## Why: 用户需要实时控制和观察Agent Teams执行过程，而不是等待最终结果
## How to apply: 在开发新功能时优先考虑实时性和可干预性