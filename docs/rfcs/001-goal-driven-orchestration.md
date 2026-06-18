# RFC-001: Goal-Driven Heterogeneous Swarm Orchestration

> **状态**: Draft → Accepted
> **版本**: 2.0.0（合并版）
> **最后更新**: 2026-06-11
> **来源**: 合并自 `RFC-001.md` + `战略聚焦.md` 第9节
> **参考**: Codex CLI `/goal` (v0.128.0), Claude Code `/goal` (v2.1.139), Kubernetes Reconcile Loop

---

## 摘要

本文档定义 acp-swarm 的核心编排模型：**Goal-Driven Execution**。

传统编排器给 Worker 分配"任务"（Task），任务有固定的执行步骤。Goal-Driven 模型给 Worker 分配"目标"（Goal），目标有明确的**完成条件**（Completion Condition）和独立的**评估器**（Evaluator）。Worker 不是一次性执行，而是进入 **Reconcile Loop**——执行、评估、反馈、再执行——直到完成条件满足（Converged）或预算耗尽。

---

## 一、核心数据结构

### 1.1 Goal（Rust 定义）

```rust
/// Goal —— 蜂群编排的核心抽象
pub struct Goal {
    // === 身份 ===
    pub id: String,
    pub description: String,
    pub parent_task_id: String,
    pub parent_goal_id: Option<String>,

    // === 完成条件 ===
    pub completion_condition: CompletionCondition,

    // === 角色分配 ===
    pub evaluator: Evaluator,
    pub executor: String,  // Worker ID

    // === 依赖关系 ===
    pub depends_on: Vec<String>,

    // === 预算与限制 ===
    pub token_budget: u64,
    pub tokens_used: u64,
    pub max_iterations: u32,
    pub current_iteration: u32,
    pub per_iteration_timeout_ms: u64,

    // === 状态 ===
    pub status: GoalStatus,
    pub iteration_log: Vec<IterationRecord>,

    // === 元数据 ===
    pub created_at: DateTime<Utc>,
    pub converged_at: Option<DateTime<Utc>>,
    pub output_files: Vec<String>,
}
```

### 1.2 CompletionCondition（完成条件）—— 8 种类型

| 类型 | 用途 | 示例 |
|------|------|------|
| **CommandSuccess** | 命令退出码为0 | `npm test` → 0 |
| **OutputContains** | 输出包含文本 | `npm run lint` → "0 errors" |
| **OutputMatches** | 输出匹配正则 | 覆盖率 > 80% |
| **FileCheck** | 文件存在/内容匹配 | `docs/api.md` 存在 |
| **HttpHealthCheck** | HTTP端点返回2xx | `/api/health` → 200 |
| **All** | 所有子条件满足（AND） | 编译通过 AND 测试通过 |
| **Any** | 任一子条件满足（OR） | 至少一个linter通过 |
| **QueenJudgment** | Queen主观判断 | 架构是否合理 |

```rust
pub enum CompletionCondition {
    CommandSuccess { command: String, args: Vec<String>, cwd: Option<String> },
    OutputContains { command: String, pattern: String, case_sensitive: bool },
    OutputMatches { command: String, regex: String },
    FileCheck { path: String, content_contains: Option<String>, max_size_bytes: Option<u64> },
    HttpHealthCheck { url: String, method: String, expected_status: Option<u16> },
    All { conditions: Vec<CompletionCondition> },
    Any { conditions: Vec<CompletionCondition> },
    QueenJudgment { criteria: String },
}
```

> **⚠️ 与战略聚焦.md的差异说明**: 战略聚焦.md第9节只列了7种类型，缺少 `HttpHealthCheck`。本RFC以8种为准。

### 1.3 Evaluator（评估器）—— 4 种类型

| 类型 | 说明 | 适用场景 |
|------|------|---------|
| **Auto** | 自动运行 CompletionCondition | 编译、测试、文件检查 |
| **Queen** | Queen Worker 主观判断 | 架构设计、代码质量 |
| **Adversarial** | 另一个 Worker 评估 | 代码审查、安全审计 |
| **Hybrid** | 先 Auto 再 Human | 先跑测试再人工审查 |

```rust
pub enum Evaluator {
    Auto,
    Queen,
    Adversarial { evaluator_worker: String },
    Hybrid { auto_condition: CompletionCondition, human_evaluator: Box<Evaluator> },
}
```

### 1.4 GoalStatus（状态机）

```
Pending ──→ Active ──→ Evaluating ──→ Converged ✓
               ↑            │
               └── Iterating ←┘ (未收敛)
               
任意状态 ──→ BudgetExhausted ✗
任意状态 ──→ MaxIterReached ✗
任意状态 ──→ Failed(reason) ✗
任意状态 ──→ Cancelled ✗
```

---

## 二、Reconcile Loop 算法

### 2.1 单 Goal 执行循环

```rust
pub async fn reconcile_goal(&self, goal: &mut Goal) -> GoalOutcome {
    loop {
        // 1. 预算检查
        if goal.tokens_used >= goal.token_budget {
            return GoalOutcome::BudgetExhausted { ... };
        }
        
        // 2. 最大迭代检查
        if goal.current_iteration >= goal.max_iterations {
            return GoalOutcome::MaxIterReached { ... };
        }

        // 3. Worker 执行
        goal.status = GoalStatus::Active;
        let execution = self.execute_on_worker(&goal.executor, &goal.description)?;

        // 4. 评估完成条件
        goal.status = GoalStatus::Evaluating;
        let evaluation = self.evaluate_goal(goal, &execution);

        // 5. 判断收敛
        if evaluation.converged {
            goal.status = GoalStatus::Converged;
            return GoalOutcome::Converged { ... };
        }

        // 6. 未收敛 → 追加反馈，进入下一轮
        goal.current_iteration += 1;
        goal.append_feedback(goal.current_iteration, &evaluation.feedback);
    }
}
```

### 2.2 Goal 执行结果

```rust
pub enum GoalOutcome {
    Converged { iterations: u32, tokens_used: u64, final_feedback: String },
    BudgetExhausted { iterations: u32, tokens_used: u64, last_feedback: String },
    MaxIterReached { iterations: u32, tokens_used: u64, last_feedback: String },
    Failed(String),
}
```

---

## 三、Goal 图编排（多 Goal 协调）

### 3.1 GoalGraph 数据结构

```rust
pub struct GoalGraph {
    goals: HashMap<String, Goal>,
    dependencies: HashMap<String, Vec<String>>,  // goal_id → [依赖的goal_id]
    reverse_deps: HashMap<String, Vec<String>>,   // goal_id → [依赖我的goal_id]
}

impl GoalGraph {
    /// 获取当前可执行的 Goal（无依赖或所有依赖已 Converged）
    pub fn ready_goals(&self) -> Vec<&Goal>;
    
    /// 检查是否有 Goal 失败导致下游阻塞
    pub fn has_blocked_goals(&self) -> Vec<(String, Vec<String>)>;
    
    /// 整个图是否全部 Converged
    pub fn all_converged(&self) -> bool;
}
```

### 3.2 Star 拓扑 = Goal 并行扇出 + 收敛检查

```
顶层 Goal: "React Login Module Complete"
  │
  ├── Sub-Goal A → Claude Code (无依赖，立即启动)
  │   condition: npm test -- --grep login
  │
  ├── Sub-Goal B → Codex (依赖 A)
  │   condition: npm test -- --grep api
  │
  └── Sub-Goal C → Gemini (依赖 A)
      condition: FileCheck { docs/api.md }
```

执行时序：
- t=0s: A 启动
- t=90s: A Converged → B 和 C 同时启动
- t=170s: B 和 C 都 Converged → 顶层 Goal Converged

### 3.3 Chain 拓扑 = Goal 依赖链 + 逐步收敛

```
Goal 1: Requirements → Goal 2: Architecture → Goal 3: Code → Goal 4: Test → Goal 5: Docs
```

每个 Goal 必须等前一个 Converged 后才能启动。

---

## 四、Queen 选举与 Goal 分解

### 4.1 Queen 的角色

1. **Goal 分解**: 收到顶层 Goal 后，Queen 将其拆解为 Sub-Goal 图
2. **Goal 评估**: 评估需要主观判断的条件（QueenJudgment 类型）
3. **Goal 协调**: 监控所有 Goal 状态，触发下游，处理失败

### 4.2 Queen Lease 机制

```rust
pub struct QueenLease {
    pub queen_id: WorkerId,
    pub granted_at: u64,      // UNIX 毫秒时间戳
    pub ttl_seconds: u64,     // 默认 30 秒
    pub renew_interval: u64,  // 每 10 秒续约
}

impl QueenLease {
    pub fn check_validity(&self) -> bool {
        let now_ms = SystemTime::now()...;
        now_ms < self.expires_at
    }
}
```

---

## 五、前端 TypeScript 类型

```typescript
// src/lib/swarm/goal-types.ts

export type GoalStatus = 'pending' | 'active' | 'evaluating' | 'converged' | 'iterating' 
  | 'budget_exhausted' | 'max_iter_reached' | 'failed' | 'cancelled';

export type CompletionCondition =
  | { type: 'command_success'; command: string; args?: string[] }
  | { type: 'output_contains'; command: string; pattern: string }
  | { type: 'output_matches'; command: string; regex: string }
  | { type: 'file_check'; path: string; content_contains?: string }
  | { type: 'http_health_check'; url: string; expected_status?: number }
  | { type: 'all'; conditions: CompletionCondition[] }
  | { type: 'any'; conditions: CompletionCondition[] }
  | { type: 'queen_judgment'; criteria: string };

export interface Goal {
  id: string;
  description: string;
  completion_condition: CompletionCondition;
  evaluator: Evaluator;
  executor: string;
  depends_on: string[];
  token_budget: number;
  max_iterations: number;
  current_iteration: number;
  status: GoalStatus;
}
```

---

## 六、具体使用示例

### 6.1 修复编译错误

```json
{
  "id": "goal-fix-compile",
  "description": "Fix all TypeScript compilation errors",
  "completion_condition": {
    "type": "command_success",
    "command": "npx",
    "args": ["vue-tsc", "--noEmit"]
  },
  "evaluator": "auto",
  "executor": "claude-code-worker",
  "token_budget": 80000,
  "max_iterations": 5
}
```

执行过程：
- Iteration 1: 修了3个，剩5个 → 反馈
- Iteration 2: 修了4个，剩1个 → 反馈
- Iteration 3: 修了最后1个 → Converged ✓

### 6.2 测试全过 + 覆盖率 > 80%

```json
{
  "completion_condition": {
    "type": "all",
    "conditions": [
      { "type": "command_success", "command": "npm", "args": ["test"] },
      { "type": "output_matches", "command": "npm", "args": ["run", "coverage"],
        "regex": "Statements\\s*:\\s*(8[0-9]|9[0-9]|100)\\." }
    ]
  }
}
```

---

## 七、错误场景与恢复策略

| 场景 | 处理 |
|------|------|
| Worker 崩溃 | Goal → Failed，如有Replica则切换 |
| 命令不存在 | 反馈 "Command not found"，Worker下轮可能安装 |
| Token 预算耗尽 | Goal → BudgetExhausted，UI提示用户追加预算 |
| Goal 依赖链失败 | 下游Goal标记为Blocked |
| Queen 崩溃 | QueenLease过期 → 自动选举新Queen |

---

## 参考

- Codex CLI `/goal`: https://openai.com/index/codex-goals/
- Kubernetes Reconcile Loop: https://book.kubebuilder.io/cronjob-tutorial/controller-implementation
- ZooKeeper Leader Election: 用于Queen租约选举