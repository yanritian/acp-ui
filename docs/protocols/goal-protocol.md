# Goal Protocol v1 规范

Goal Protocol 定义目标的提交、评估和收敛规则。

## Goal 数据结构

```json
{
  "id": "string",
  "description": "string",
  "completion_condition": "CompletionCondition",
  "evaluator": "Evaluator",
  "assigned_worker": "string | null",
  "status": "GoalStatus",
  "iterations": ["IterationRecord"],
  "max_iterations": "number",
  "token_budget": "number | null",
  "token_used": "number",
  "created_at": "number",
  "started_at": "number | null",
  "converged_at": "number | null",
  "dependencies": ["string"]
}
```

---

## CompletionCondition 类型

### command_success

```json
{
  "type": "command_success",
  "command": "npm test",
  "expected_exit_code": 0
}
```

### output_contains

```json
{
  "type": "output_contains",
  "text": "0 errors",
  "case_sensitive": false
}
```

### output_matches

```json
{
  "type": "output_matches",
  "pattern": "Statements:\\s*:\\s*(8[0-9]|9[0-9]|100)\\."
}
```

### file_check

```json
{
  "type": "file_check",
  "path": "docs/api.md",
  "must_exist": true,
  "content_contains": null
}
```

### http_health_check

```json
{
  "type": "http_health_check",
  "url": "/api/health",
  "expected_status": 200
}
```

### all (AND)

```json
{
  "type": "all",
  "conditions": [
    { "type": "command_success", "command": "npm test" },
    { "type": "output_matches", "pattern": "Coverage > 80%" }
  ]
}
```

### any (OR)

```json
{
  "type": "any",
  "conditions": [
    { "type": "command_success", "command": "eslint" },
    { "type": "command_success", "command": "prettier --check" }
  ]
}
```

### queen_judgment

```json
{
  "type": "queen_judgment",
  "criteria": "Architecture is well-designed"
}
```

---

## GoalStatus 状态机

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

## Reconcile Loop 算法

```
while not converged:
    1. 检查预算
    2. 检查最大迭代
    3. Worker执行
    4. 评估完成条件
    5. 如果收敛 → 返回成功
    6. 否则 → 追加反馈，继续迭代
```

---

## GoalGraph 编排

### 依赖关系

```json
{
  "id": "goal-002",
  "dependencies": ["goal-001"]
}
```

### 执行顺序

- Goal 必须等待依赖 Goal 收敛后才能启动
- 依赖失败的 Goal 将被标记为 Blocked