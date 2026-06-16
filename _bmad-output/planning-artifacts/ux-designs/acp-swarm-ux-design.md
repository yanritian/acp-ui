---
title: acp-swarm UX 设计文档
status: final
created: 2026-06-09
updated: 2026-06-09
project: acp-swarm
version: 1.0.0
---

# acp-swarm UX 设计文档

*基于 PRD v2.0.0（42 个 FR）和 Epic/Story 列表*

---

## 0. Design Philosophy

### 0.1 设计原则

1. **实时性优先**：所有状态变更通过 WebSocket 实时推送，Dashboard 500ms 内渲染
2. **信息密度适中**：Worker 卡片展示关键信息，点击展开详情
3. **错误可调试**：所有错误包含上下文，支持一键复制错误详情
4. **渐进式复杂度**：5 分钟跑通 demo → 高级配置 → 自定义 Hook

### 0.2 用户角色

| 角色 | 主要界面 | 核心任务 |
|------|---------|---------|
| **首次用户** | CLI + Dashboard | 5 分钟跑通 demo |
| **Worker 开发者** | SDK + API 文档 | 接入自定义 Worker |
| **蜂群编排者** | Dashboard + CLI | 配置 Goal、监控执行、调试失败 |

---

## 1. Dashboard UI 设计

### 1.1 整体布局

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  acp-swarm Dashboard                          [Settings] [Help]            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Swarm Status: READY   Queen: claude-code-1 (claude_code)           │  │
│  │  Workers: 3 healthy, 1 offline   Goals: 2 active, 5 converged       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Workers (3)                                                        │  │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐       │  │
│  │  │ codex-1         │ │ claude-code-1   │ │ gemini-1        │       │  │
│  │  │ [healthy] 👑    │ │ [busy]          │ │ [offline]       │       │  │
│  │  │                 │ │                 │ │                 │       │  │
│  │  │ Skills:         │ │ Skills:         │ │ Skills:         │       │  │
│  │  │ • code-gen (90%)│ │ • review (95%)  │ │ • multi-modal   │       │  │
│  │  │ • test-gen (80%)│ │ • planning (95%)│ │   (85%)         │       │  │
│  │  │                 │ │                 │ │                 │       │  │
│  │  │ Load: 1/3       │ │ Load: 2/2       │ │ Load: 0/4       │       │  │
│  │  │ Memory: 256MB   │ │ Memory: 512MB   │ │ Memory: N/A     │       │  │
│  │  │                 │ │                 │ │                 │       │  │
│  │  │ [Health] [Stop] │ │ [Health] [Stop] │ │ [Restart]       │       │  │
│  │  └─────────────────┘ └─────────────────┘ └─────────────────┘       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Active Goals (2)                                                    │  │
│  │  ┌────────────────────────────────────────────────────────────────┐ │  │
│  │  │ goal-fix-ts-errors                          [Converged] ✓     │ │  │
│  │  │ Worker: claude-code-1   Iteration: 2/5   Tokens: 45K/100K     │ │  │
│  │  │ ████████████████░░░░░░░░░ 45%                                 │ │  │
│  │  │                                                               │ │  │
│  │  │ Description: Fix all TypeScript compilation errors            │ │  │
│  │  │                                                               │ │  │
│  │  │ Files Modified (3):                                           │ │  │
│  │  │ + src/auth/handler.ts  (+12 -8)                              │ │  │
│  │  │ + src/utils/types.ts   (+5 -3)                               │ │  │
│  │  │ + src/middleware.ts    (+8 -5)                               │ │  │
│  │  │                                                               │ │  │
│  │  │ [View Diff] [Retry] [Cancel]                                 │ │  │
│  │  └────────────────────────────────────────────────────────────────┘ │  │
│  │  ┌────────────────────────────────────────────────────────────────┐ │  │
│  │  │ goal-generate-docs                          [Iterating] ⚡   │ │  │
│  │  │ Worker: codex-1   Iteration: 1/3   Tokens: 18K/50K            │ │  │
│  │  │ █████████░░░░░░░░░░░░░░ 36%                                   │ │  │
│  │  │                                                               │ │  │
│  │  │ Description: Generate API documentation for auth module       │ │  │
│  │  │                                                               │ │  │
│  │  │ Last Feedback: "Missing examples for /login endpoint"         │ │  │
│  │  │                                                               │ │  │
│  │  │ [View Output] [Cancel]                                       │ │  │
│  │  └────────────────────────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Execute Goal                                                        │  │
│  │  ┌───────────────────────────────────────────────────────────────┐  │  │
│  │  │ Goal Description                                              │  │  │
│  │  │ ┌───────────────────────────────────────────────────────────┐ │  │  │
│  │  │ │ Fix all TypeScript compilation errors in src/ directory   │ │  │  │
│  │  │ │                                                           │ │  │  │
│  │  │ └───────────────────────────────────────────────────────────┘ │  │  │
│  │  │                                                               │  │  │
│  │  │ [Upload .goal File]                                           │  │  │
│  │  └───────────────────────────────────────────────────────────────┘  │  │
│  │                                                                     │  │
│  │  Topology: [Star ▼]   Workers: [codex-1, claude-code-1] [Add]     │  │
│  │                                                                     │  │
│  │  [Execute Goal]                                                     │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Event Log                                              [Filter ▼]  │  │
│  │  ┌────────────────────────────────────────────────────────────────┐ │  │
│  │  │ 10:23:45  goal.converged      goal-fix-ts-errors              │ │  │
│  │  │ 10:23:40  evaluation.complete goal-fix-ts-errors converged    │ │  │
│  │  │ 10:23:35  goal.iterating      goal-fix-ts-errors iter 2       │ │  │
│  │  │ 10:23:30  evaluation.complete goal-fix-ts-errors 5 errors     │ │  │
│  │  │ 10:23:25  goal.active         goal-fix-ts-errors started      │ │  │
│  │  │ 10:23:20  queen.elected       claude-code-1 is Queen          │ │  │
│  │  │ ...                                                           │ │  │
│  │  └────────────────────────────────────────────────────────────────┘ │  │
│  │  [Export JSONL]                                                     │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Worker 卡片设计

**状态颜色**：
- `healthy`: 绿色边框 + 绿色文字
- `busy`: 蓝色边框 + 蓝色文字
- `starting`: 黄色边框 + 黄色文字
- `offline`: 灰色边框 + 灰色文字
- `unhealthy`: 红色边框 + 红色文字

**Queen 标识**：
- 黄色 👑 图标
- 黄色 "QUEEN" 徽章

**卡片内容**：
```
┌─────────────────────────────────────┐
│ codex-1                    [healthy]│  ← Worker ID + 状态
│                                     │
│ [codex] [QUEEN]                     │  ← Worker Type Badge + Queen Badge
│                                     │
│ Skills:                             │
│ • code-generation (90%)             │  ← 前 3 个 Skill + proficiency
│ • test-generation (80%)             │
│ • bug-fixing (85%)                  │
│                                     │
│ Load: 1/3      Memory: 256MB        │  ← 关键指标
│ CPU: 15%       Uptime: 2h 15m      │
│                                     │
│ Current: goal-fix-ts-errors         │  ← 当前执行的 Goal（如果有）
│                                     │
│ [Health Check] [Shutdown]           │  ← 操作按钮
└─────────────────────────────────────┘
```

**点击卡片**：展开侧边栏显示详细信息
- 完整 Skill 列表
- 执行历史（最近 10 个 Goal）
- Token 消耗曲线
- Iteration Log

### 1.3 Goal 进度展示

**状态图标**：
- `Pending`: ⏸️ 灰色
- `Active`: ⚡ 蓝色
- `Evaluating`: 🔍 紫色
- `Converged`: ✓ 绿色
- `Iterating`: 🔄 黄色
- `BudgetExhausted`: 💸 红色
- `MaxIterReached`: 🔢 红色
- `Failed`: ✗ 红色
- `Cancelled`: 🚫 灰色

**Token 进度条**：
```
Tokens: 45K/100K
████████████████░░░░░░░░░ 45%
```

颜色规则：
- < 50%: 绿色
- 50-80%: 黄色
- > 80%: 红色

**Diff 展示**（Goal Converged 时）：
```
Files Modified (3):

+ src/auth/handler.ts
  +12 lines, -8 lines
  [View Diff]

+ src/utils/types.ts
  +5 lines, -3 lines
  [View Diff]

+ src/middleware.ts
  +8 lines, -5 lines
  [View Diff]
```

点击 "View Diff" 弹出 diff 视图：
```diff
--- a/src/auth/handler.ts
+++ b/src/auth/handler.ts
@@ -42,7 +42,7 @@
- function oldCode() {
+ function newCode() {
   // ...
 }
```

### 1.4 Event Log 展示

**事件类型颜色**：
- `goal.converged`: 绿色
- `goal.failed`: 红色
- `goal.iterating`: 黄色
- `worker.registered`: 蓝色
- `worker.disconnected`: 红色
- `queen.elected`: 紫色

**日志格式**：
```
10:23:45  goal.converged      goal-fix-ts-errors converged after 2 iterations
10:23:40  evaluation.complete goal-fix-ts-errors converged=true
10:23:35  goal.iterating      goal-fix-ts-errors iteration 2
10:23:30  evaluation.complete goal-fix-ts-errors converged=false, 5 errors
10:23:25  goal.active         goal-fix-ts-errors started on claude-code-1
```

**过滤功能**：
```
[Filter ▼]
  • All Events
  • Goal Events Only
  • Worker Events Only
  • Errors Only
  • Last 10 Minutes
```

**导出功能**：
```
[Export JSONL]
```

导出格式：
```json
{"timestamp":"2026-06-09T10:23:45Z","type":"goal.converged","data":{...}}
{"timestamp":"2026-06-09T10:23:40Z","type":"evaluation.complete","data":{...}}
```

### 1.5 执行任务界面

**两种输入模式**：

1. **Goal 描述模式**（简单）：
```
Goal Description:
┌──────────────────────────────────────────────────────────┐
│ Fix all TypeScript compilation errors in src/ directory  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

2. **.goal 文件模式**（高级）：
```
[Upload .goal File]
┌──────────────────────────────────────────────────────────┐
│ 📄 fix-ts-errors.goal                                    │
│                                                          │
│ name: Fix TypeScript Errors                              │
│ workers:                                                 │
│   - id: claude                                           │
│     type: claude-code                                    │
│ goals:                                                   │
│   - id: fix-errors                                       │
│     executor: claude                                     │
│     condition:                                           │
│       run: "npx vue-tsc --noEmit"                        │
│       expect: "exit_code == 0"                           │
└──────────────────────────────────────────────────────────┘
```

**配置选项**：
```
Topology: [Star ▼]
  • Star (Queen + Workers) - Parallel execution with Queen coordination
  • Chain (Sequential) - Goals execute in dependency order

Workers: [codex-1, claude-code-1] [Add]
  • Auto-select (based on skills)
  • Manual selection
```

**执行按钮**：
```
[Execute Goal]
```

点击后：
1. 验证配置
2. 提交到编排引擎
3. 在 Active Goals 区域显示新 Goal
4. Event Log 显示 `goal.submitted` 事件

---

## 2. CLI 设计

### 2.1 命令结构

```
acp-swarm
├── run              # 执行 .goal 文件
├── worker           # Worker 管理
│   ├── list         # 列出所有 Worker
│   ├── register     # 注册 Worker
│   ├── status       # 查看 Worker 状态
│   └── shutdown     # 关闭 Worker
├── goal             # Goal 管理
│   ├── list         # 列出所有 Goal
│   ├── status       # 查看 Goal 状态
│   ├── retry        # 重试失败的 Goal
│   └── cancel       # 取消 Goal
├── watch            # 实时查看事件流
├── events           # 事件管理
│   ├── list         # 列出事件
│   └── export       # 导出事件
└── server           # 启动编排引擎
    ├── start        # 启动
    └── stop         # 停止
```

### 2.2 acp-swarm run

**用法**：
```bash
acp-swarm run --goal <path> [options]
```

**选项**：
```
--goal <path>              .goal 文件路径（必需）
--workers <ids>            Worker ID 列表（逗号分隔，可选）
--topology <star|chain>    拓扑类型（默认: star）
--timeout <ms>             超时时间（默认: 300000ms）
--verbose                  详细输出
--dry-run                  只解析 .goal 文件，不执行
```

**示例**：
```bash
# 执行 .goal 文件
acp-swarm run --goal examples/goals/fix-ts-errors.goal

# 指定 Worker
acp-swarm run --goal fix-ts-errors.goal --workers codex-1,claude-code-1

# 使用 Chain 拓扑
acp-swarm run --goal fix-ts-errors.goal --topology chain

# 详细输出
acp-swarm run --goal fix-ts-errors.goal --verbose

# 只解析不执行
acp-swarm run --goal fix-ts-errors.goal --dry-run
```

**输出**：
```
$ acp-swarm run --goal examples/goals/fix-ts-errors.goal

Loading goal file: examples/goals/fix-ts-errors.goal
  ✓ Parsed 1 goal, 2 workers

Submitting goal: fix-ts-errors
  ✓ Goal submitted (id: goal-abc123)

Executing on worker: claude-code-1
  [Iteration 0] Evaluating...
  [Iteration 0] Not converged (5 errors remaining)
  [Iteration 1] Evaluating...
  [Iteration 1] Converged ✓

Goal converged after 2 iterations
  Tokens used: 45,000 / 100,000
  Duration: 3m 15s
  Files modified: 3

$
```

**错误输出**：
```
$ acp-swarm run --goal invalid.goal

Loading goal file: invalid.goal
  ✗ Parse error at line 15, column 8
    Executor 'nonexistent-worker' not found in workers list

  Hint: Check that all executors are declared in the workers section
```

### 2.3 acp-swarm watch

**用法**：
```bash
acp-swarm watch [options]
```

**选项**：
```
--worker <id>              只看特定 Worker 的事件
--goal <id>                只看特定 Goal 的事件
--filter <pattern>         事件类型过滤（支持通配符）
--format <text|json>       输出格式（默认: text）
```

**示例**：
```bash
# 查看所有事件
acp-swarm watch

# 只看特定 Worker
acp-swarm watch --worker codex-1

# 只看 Goal 事件
acp-swarm watch --filter "goal.*"

# JSON 格式输出
acp-swarm watch --format json
```

**输出**：
```
$ acp-swarm watch

Watching events... (Ctrl+C to stop)

10:23:45 [goal.converged]      goal-fix-ts-errors converged after 2 iterations
10:23:40 [evaluation.complete] goal-fix-ts-errors converged=true
10:23:35 [goal.iterating]      goal-fix-ts-errors iteration 2
10:23:30 [evaluation.complete] goal-fix-ts-errors converged=false, 5 errors
10:23:25 [goal.active]         goal-fix-ts-errors started on claude-code-1
10:23:20 [queen.elected]       claude-code-1 is Queen
...
```

**实时刷新**：
- WebSocket 连接，实时推送
- 每行事件带时间戳和颜色
- Ctrl+C 停止

### 2.4 acp-swarm worker

**worker list**：
```bash
$ acp-swarm worker list

Workers (3):
  ID                Type          Status    Load    Memory
  codex-1           codex         healthy   1/3     256MB
  claude-code-1     claude_code   busy      2/2     512MB
  gemini-1          gemini        offline   0/4     N/A
```

**worker register**：
```bash
$ acp-swarm worker register --type codex --id codex-1

Registering worker...
  ✓ Worker registered (id: codex-1, type: codex)
  Heartbeat interval: 10s
```

**worker status**：
```bash
$ acp-swarm worker status codex-1

Worker: codex-1
  Type: codex
  Status: healthy
  Load: 1/3
  Memory: 256MB
  CPU: 15%
  Uptime: 2h 15m

  Skills:
    • code-generation (90%)
    • test-generation (80%)
    • bug-fixing (85%)

  Current Goal: goal-fix-ts-errors
    Iteration: 1/5
    Tokens: 18K/50K
```

### 2.5 acp-swarm goal

**goal list**：
```bash
$ acp-swarm goal list

Goals (7):
  ID                      Status        Worker          Iter   Tokens
  goal-fix-ts-errors      Converged ✓   claude-code-1   2/5   45K/100K
  goal-generate-docs      Iterating ⚡  codex-1         1/3   18K/50K
  goal-refactor-auth      Failed ✗      codex-1         3/5   67K/100K
  ...
```

**goal status**：
```bash
$ acp-swarm goal status goal-fix-ts-errors

Goal: goal-fix-ts-errors
  Status: Converged
  Worker: claude-code-1
  Iterations: 2/5
  Tokens: 45,000/100,000 (45%)
  Duration: 3m 15s

  Description: Fix all TypeScript compilation errors in src/ directory

  Files Modified (3):
    • src/auth/handler.ts (+12 -8)
    • src/utils/types.ts (+5 -3)
    • src/middleware.ts (+8 -5)

  Iteration Log:
    [0] Not converged (5 errors remaining)
    [1] Converged ✓
```

**goal retry**：
```bash
$ acp-swarm goal retry goal-refactor-auth

Retrying goal: goal-refactor-auth
  ✓ Goal resubmitted (new id: goal-refactor-auth-2)
```

### 2.6 错误信息和帮助文本

**错误格式**：
```
✗ Error: <错误描述>

  Context: <上下文信息>
  Hint: <解决建议>

  If this is a bug, please report it at:
  https://github.com/acp-swarm/acp-swarm/issues
```

**示例**：
```
✗ Error: Worker not found: codex-1

  Context: No worker with ID 'codex-1' is registered
  Hint: Run `acp-swarm worker register --type codex --id codex-1` to register the worker

  If this is a bug, please report it at:
  https://github.com/acp-swarm/acp-swarm/issues
```

**帮助文本**：
```bash
$ acp-swarm --help

acp-swarm - Heterogeneous Agent Swarm Orchestrator

USAGE:
    acp-swarm <SUBCOMMAND>

SUBCOMMANDS:
    run         Execute a .goal file
    worker      Worker management (list, register, status, shutdown)
    goal        Goal management (list, status, retry, cancel)
    watch       Watch real-time event stream
    events      Event management (list, export)
    server      Server management (start, stop)
    help        Print this message or the help of the given subcommand

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

EXAMPLES:
    # Run a .goal file
    acp-swarm run --goal examples/goals/fix-ts-errors.goal

    # Watch real-time events
    acp-swarm watch

    # List all workers
    acp-swarm worker list

For more information, see: https://docs.acp-swarm.dev
```

---

## 3. Worker SDK 用户体验

### 3.1 Rust SDK

**API 设计**：
```rust
use acp_worker_sdk::{Worker, WorkerConfig, Goal, GoalResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WorkerConfig {
        worker_id: "my-worker-1".into(),
        worker_type: "custom".into(),
        orchestrator_url: "http://localhost:9090".into(),
        skills: vec![
            WorkerSkill {
                name: "code-review".into(),
                proficiency: 0.9,
                ..Default::default()
            },
        ],
        max_concurrency: 2,
    };

    let mut worker = Worker::new(config);

    worker.on_goal(|goal| async move {
        println!("Received goal: {}", goal.description);

        // Execute goal logic
        let result = execute_my_agent(&goal.description).await?;

        Ok(GoalResult {
            success: true,
            output: result.output,
            tokens_used: result.tokens_used,
            files_modified: result.files_modified,
        })
    });

    println!("Worker started, waiting for goals...");
    worker.start().await?;

    Ok(())
}
```

**关键特性**：
- 自动注册到编排引擎
- 自动心跳（10 秒间隔）
- 自动接收 Goal 并调用回调
- 自动报告结果
- 网络错误自动重试

**错误处理**：
```rust
worker.on_goal(|goal| async move {
    match execute_my_agent(&goal.description).await {
        Ok(result) => Ok(GoalResult { success: true, .. }),
        Err(e) => {
            eprintln!("Goal execution failed: {}", e);
            Ok(GoalResult {
                success: false,
                output: format!("Error: {}", e),
                ..Default::default()
            })
        }
    }
});
```

### 3.2 TypeScript SDK

**API 设计**：
```typescript
import { Worker, WorkerConfig, Goal, GoalResult } from '@acp-swarm/worker-sdk';

const config: WorkerConfig = {
  workerId: 'my-worker-1',
  workerType: 'custom',
  orchestratorUrl: 'http://localhost:9090',
  skills: [
    { name: 'code-review', proficiency: 0.9 },
  ],
  maxConcurrency: 2,
};

const worker = new Worker(config);

worker.onGoal(async (goal: Goal): Promise<GoalResult> => {
  console.log(`Received goal: ${goal.description}`);

  // Execute goal logic
  const result = await executeMyAgent(goal.description);

  return {
    success: true,
    output: result.output,
    tokensUsed: result.tokensUsed,
    filesModified: result.filesModified,
  };
});

console.log('Worker started, waiting for goals...');
await worker.start();
```

**关键特性**：
- 自动注册到编排引擎
- 自动心跳（10 秒间隔）
- 自动接收 Goal 并调用回调
- 自动报告结果
- 网络错误自动重试
- TypeScript 类型定义完整

**错误处理**：
```typescript
worker.onGoal(async (goal: Goal): Promise<GoalResult> => {
  try {
    const result = await executeMyAgent(goal.description);
    return { success: true, output: result.output };
  } catch (e) {
    console.error('Goal execution failed:', e);
    return {
      success: false,
      output: `Error: ${e.message}`,
    };
  }
});
```

---

## 4. .goal 文件格式

### 4.1 YAML Schema 示例

**基础示例**：
```yaml
name: Fix TypeScript Errors
description: Fix all TypeScript compilation errors in the project
version: "1.0"

workers:
  - id: claude
    type: claude-code
    fallback: codex

goals:
  - id: fix-errors
    executor: claude
    description: Fix all TypeScript compilation errors reported by vue-tsc
    condition:
      type: command_success
      command: npx
      args: ["vue-tsc", "--noEmit"]
    evaluator: auto
    budget: 80000
    max_iter: 5
    timeout_ms: 300000
    checkpoint: on_converge

top:
  condition:
    type: command_success
    command: npx
    args: ["vue-tsc", "--noEmit"]
  evaluator: auto
```

**多 Goal 示例**：
```yaml
name: Build React Login Page
description: Build a React login page with form validation and unit tests

workers:
  - id: codex
    type: codex
  - id: claude
    type: claude-code

goals:
  - id: build-component
    executor: claude
    description: Build React login component with form validation
    condition:
      type: all
      conditions:
        - type: file_check
          path: src/components/Login.tsx
        - type: command_success
          command: npm
          args: ["run", "lint"]
    evaluator: auto
    budget: 50000
    max_iter: 3

  - id: write-tests
    executor: codex
    description: Write unit tests for login component
    condition:
      type: command_success
      command: npm
      args: ["test", "--", "--coverage=80"]
    evaluator: auto
    depends_on: [build-component]
    budget: 30000
    max_iter: 3

top:
  condition:
    type: all
    conditions:
      - type: command_success
        command: npm
        args: ["test"]
      - type: command_success
        command: npm
        args: ["run", "build"]
  evaluator: auto
```

### 4.2 错误信息格式

**解析错误**：
```
✗ Parse error at line 15, column 8

  14 | goals:
  15 |   - id: fix-errors
  16 |     executor: nonexistent-worker
                ^^^^^^^^^^^^^^^^^^^^

Error: Executor 'nonexistent-worker' not found in workers list

Hint: Check that all executors are declared in the workers section
```

**验证错误**：
```
✗ Validation error

Goal 'write-tests' depends on 'build-component', but 'build-component' is not defined

Hint: Add 'build-component' to the goals section, or remove the dependency
```

**循环依赖错误**：
```
✗ Circular dependency detected

  goal-a → goal-b → goal-c → goal-a

Hint: Remove the circular dependency by changing the depends_on fields
```

---

## 5. 交互流程

### 5.1 首次用户体验（UJ-1）

```
1. 用户 clone acp-swarm 仓库
   $ git clone https://github.com/acp-swarm/acp-swarm.git
   $ cd acp-swarm

2. 用户阅读 README，看到 5 分钟快速开始

3. 用户运行 quickstart 示例
   $ cargo run --example quickstart

   acp-swarm listening on http://localhost:9090
   Dashboard: http://localhost:1420

4. 用户打开 Dashboard，看到空状态
   "No workers registered yet. Register your first worker to get started."

5. 用户打开新终端，注册 Codex Worker
   $ acp-swarm worker register --type codex --id codex-1
   ✓ Worker registered (id: codex-1, type: codex)

6. 用户回到 Dashboard，看到 Worker 卡片出现
   Worker 卡片显示 "healthy" 状态

7. 用户在 Dashboard 点击 "Execute Goal"
   输入：Fix all TypeScript errors in src/ directory
   选择拓扑：Star
   点击 "Execute"

8. Dashboard 显示 Goal 进度
   goal-fix-ts-errors: [Iterating] iteration 0
   Tokens: 18K/100K

9. Goal 收敛，Dashboard 显示结果
   goal-fix-ts-errors: [Converged ✓] converged after 2 iterations
   Tokens: 45K/100K
   Files modified: 3

10. 用户截图发推特
    "Just used acp-swarm to fix 50 TS errors in 3 minutes! 🚀"
```

### 5.2 Worker 开发者体验（UJ-2）

```
1. 开发者阅读 Worker SDK 文档
   https://docs.acp-swarm.dev/guides/build-a-worker

2. 开发者安装 SDK
   $ pip install acp-worker-sdk

3. 开发者编写 Worker 代码
   from acp_worker import Worker, WorkerConfig

   config = WorkerConfig(
       worker_id="my-reviewer",
       worker_type="custom",
       orchestrator_url="http://localhost:9090",
   )

   worker = Worker(config)

   @worker.on_goal
   async def handle_goal(goal):
       result = await my_review_agent(goal.description)
       return {"success": True, "output": result}

   worker.start()

4. 开发者启动 Worker
   $ python my_worker.py
   Worker started, waiting for goals...

5. Dashboard 自动发现新 Worker
   Worker 卡片显示 "my-reviewer (custom)"

6. 开发者在 .goal 文件中声明 Worker 擅长 code-review
   workers:
     - id: my-reviewer
       type: custom

   goals:
     - id: review-code
       executor: my-reviewer
       ...

7. 开发者运行 .goal 文件
   $ acp-swarm run --goal review.goal

8. 编排引擎自动路由 code-review 任务给 my-reviewer
   Skill Router: code-review → my-reviewer (confidence: 0.9)

9. Worker 执行任务并返回结果
   goal-review-code: [Converged ✓]

10. 开发者提交 PR，分享 Worker 实现
    "Added custom code review Worker using acp-worker-sdk"
```

---

## 6. 视觉设计规范

### 6.1 颜色系统

**状态颜色**：
- Primary: `#3B82F6` (蓝色)
- Success: `#10B981` (绿色)
- Warning: `#F59E0B` (黄色)
- Error: `#EF4444` (红色)
- Neutral: `#6B7280` (灰色)

**背景颜色**：
- Background: `#F9FAFB` (浅灰)
- Surface: `#FFFFFF` (白色)
- Card: `#FFFFFF` (白色)

**文字颜色**：
- Primary: `#111827` (深灰)
- Secondary: `#6B7280` (中灰)
- Tertiary: `#9CA3AF` (浅灰)

### 6.2 字体

- 主字体: `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`
- 等宽字体: `ui-monospace, 'SF Mono', Menlo, monospace`

### 6.3 间距

- 基础单位: 4px
- 小间距: 8px
- 中间距: 16px
- 大间距: 24px
- 超大间距: 32px

### 6.4 圆角

- 小圆角: 4px (按钮、输入框)
- 中圆角: 8px (卡片)
- 大圆角: 12px (模态框)

---

*UX Design version: 1.0.0*
*Last updated: 2026-06-09*
