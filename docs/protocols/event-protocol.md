# Event Protocol v1 规范

Event Protocol 定义状态变更事件的推送格式。

## 传输方式

| 传输方式 | 适用场景 |
|---------|---------|
| WebSocket | Web Dashboard |
| Server-Sent Events | 轻量级客户端 |
| Tauri Events | 桌面应用内部 |

---

## 事件类型

### Goal 事件

```json
{ "type": "goal.submitted", "goal_id": "goal-001", "timestamp": 1234567890 }
{ "type": "goal.active", "goal_id": "goal-001", "iteration": 0 }
{ "type": "goal.evaluating", "goal_id": "goal-001", "evaluator": "auto" }
{ "type": "goal.iterating", "goal_id": "goal-001", "iteration": 1, "feedback": "..." }
{ "type": "goal.converged", "goal_id": "goal-001", "iterations": 3 }
{ "type": "goal.failed", "goal_id": "goal-001", "reason": "budget_exhausted" }
{ "type": "goal.cancelled", "goal_id": "goal-001" }
```

### Worker 事件

```json
{ "type": "worker.registered", "worker_id": "codex-01", "worker_type": "codex" }
{ "type": "worker.connected", "worker_id": "codex-01" }
{ "type": "worker.disconnected", "worker_id": "codex-01" }
{ "type": "worker.error", "worker_id": "codex-01", "error": "..." }
```

### Queen 事件

```json
{ "type": "queen.elected", "queen_id": "claude-01", "ttl_seconds": 30 }
{ "type": "queen.lease_renewed", "queen_id": "claude-01" }
{ "type": "queen.lease_expired", "queen_id": "claude-01" }
```

---

## 事件订阅

### WebSocket

```javascript
const ws = new WebSocket('ws://localhost:1421/events')

ws.onmessage = (event) => {
  const data = JSON.parse(event.data)
  console.log(`Event: ${data.type}`, data)
}
```

### Tauri Events

```typescript
import { listen } from '@tauri-apps/api/event'

listen('goal.converged', (event) => {
  console.log('Goal converged:', event.payload)
})
```

---

## 事件过滤

客户端可以订阅特定类型的事件：

```json
{
  "subscribe": ["goal.converged", "goal.failed", "worker.disconnected"]
}
```