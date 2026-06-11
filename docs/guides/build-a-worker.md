# Worker 接入指南

本指南说明如何将 Agent 接入 ACP-Swarm 蜂群编排系统。

## Worker Protocol v1

任何 Agent 只要实现以下 5 个接口，就能成为 ACP-Swarm 的 Worker。

### 1. 注册接口

```json
POST /acp/worker/register

{
  "worker_id": "my-agent-01",
  "worker_type": "custom-agent",
  "version": "1.0",
  "capabilities": {
    "skills": [
      { "name": "code-generation", "proficiency": 0.9, "avg_duration_ms": 30000 }
    ],
    "max_concurrency": 2,
    "max_context_tokens": 200000
  }
}

Response: { "status": "registered", "heartbeat_interval_ms": 10000 }
```

### 2. 心跳接口

```json
POST /acp/worker/heartbeat

{
  "worker_id": "my-agent-01",
  "status": "idle",
  "current_load": 0,
  "active_goals": []
}

Response: { "status": "ok" }
```

**重要**: Worker 必须每 `heartbeat_interval_ms` 毫秒发送心跳。连续 3 次未收到心跳，Worker 将被判定为失联。

### 3. 接收 Goal

```json
POST /acp/worker/goal/receive

{
  "goal_id": "goal-001",
  "description": "Fix TypeScript errors",
  "iteration": 0,
  "feedback_from_last_iteration": null,
  "workspace_path": "/workspace/project/",
  "token_budget": 50000,
  "timeout_ms": 300000
}

Response: { "status": "accepted", "execution_id": "exec-001" }
```

### 4. 返回结果

```json
POST /acp/worker/goal/result

{
  "execution_id": "exec-001",
  "goal_id": "goal-001",
  "success": true,
  "output": "Fixed 5 TypeScript errors",
  "tokens_used": 32000,
  "files_modified": ["src/utils.ts"]
}

Response: { "status": "received", "next_action": "evaluating" }
```

### 5. 错误报告

```json
POST /acp/worker/error

{
  "worker_id": "my-agent-01",
  "goal_id": "goal-001",
  "error_type": "timeout",
  "message": "Execution exceeded timeout limit"
}
```

---

## 实现示例

### Python Worker SDK

```python
from acp_swarm import WorkerClient

worker = WorkerClient(
    worker_id="python-agent-01",
    worker_type="python-llm"
)

worker.register()

while True:
    goal = worker.receive_goal()
    if goal:
        result = execute_goal(goal)
        worker.submit_result(goal.goal_id, result)
    
    worker.heartbeat()
    time.sleep(10)
```

### TypeScript Worker SDK

```typescript
import { SwarmWorker } from '@acp-swarm/sdk'

const worker = new SwarmWorker({
  workerId: 'ts-agent-01',
  workerType: 'typescript-agent'
})

worker.onGoal((goal) => {
  const result = executeGoal(goal)
  worker.submitResult(goal.id, result)
})

worker.start()
```

---

## Skill 声明

Worker 通过 Skill 声明告知编排器自己的能力：

```json
{
  "skills": [
    { "name": "code-generation", "proficiency": 0.9, "avg_duration_ms": 30000 },
    { "name": "test-writing", "proficiency": 0.7, "avg_duration_ms": 20000 },
    { "name": "documentation", "proficiency": 0.8, "avg_duration_ms": 15000 }
  ]
}
```

| 字段 | 说明 |
|------|------|
| name | Skill 名称 |
| proficiency | 熟练度（0.0-1.0） |
| avg_duration_ms | 平均执行时长 |

---

## 最佳实践

1. **心跳间隔**: 建议 10 秒发送一次心跳
2. **超时处理**: Goal 超时后应主动报告错误
3. **Token 管理**: 监控 Token 使用，避免预算耗尽
4. **错误恢复**: 实现重试机制，提高稳定性

---

## 参考

- [Worker Protocol 规范](../protocols/worker-protocol.md)
- [Goal Protocol 规范](../protocols/goal-protocol.md)
- [Event Protocol 规范](../protocols/event-protocol.md)