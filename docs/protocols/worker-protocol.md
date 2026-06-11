# Worker Protocol v1 规范

Worker Protocol 定义 Agent 如何接入 ACP-Swarm 蜂群编排系统。

## 接口定义

### 1. 注册

**请求**

```
POST /acp/worker/register
Content-Type: application/json
```

```json
{
  "worker_id": "string",
  "worker_type": "string",
  "version": "string",
  "capabilities": {
    "skills": [
      {
        "name": "string",
        "proficiency": "number (0.0-1.0)",
        "avg_duration_ms": "number"
      }
    ],
    "max_concurrency": "number",
    "max_context_tokens": "number"
  }
}
```

**响应**

```json
{
  "status": "registered",
  "heartbeat_interval_ms": 10000
}
```

### 2. 心跳

**请求**

```
POST /acp/worker/heartbeat
Content-Type: application/json
```

```json
{
  "worker_id": "string",
  "status": "idle | busy | disconnected | error",
  "current_load": "number",
  "active_goals": ["string"]
}
```

**响应**

```json
{
  "status": "ok"
}
```

### 3. 接收 Goal

**请求**

```
POST /acp/worker/goal/receive
Content-Type: application/json
```

```json
{
  "goal_id": "string",
  "description": "string",
  "iteration": "number",
  "feedback_from_last_iteration": "string | null",
  "workspace_path": "string",
  "token_budget": "number",
  "timeout_ms": "number"
}
```

**响应**

```json
{
  "status": "accepted",
  "execution_id": "string"
}
```

### 4. 返回结果

**请求**

```
POST /acp/worker/goal/result
Content-Type: application/json
```

```json
{
  "execution_id": "string",
  "goal_id": "string",
  "success": "boolean",
  "output": "string",
  "tokens_used": "number",
  "files_modified": ["string"]
}
```

**响应**

```json
{
  "status": "received",
  "next_action": "evaluating"
}
```

---

## 状态机

```
        ┌──────────┐
        │  Offline │
        └──────────┘
              │ register()
              ▼
        ┌──────────┐
        │   Idle   │◄────────────┐
        └──────────┘             │
              │ receive_goal()   │ submit_result()
              ▼                  │
        ┌──────────┐             │
        │   Busy   │─────────────┘
        └──────────┘
              │ error / timeout
              ▼
        ┌──────────┐
        │  Error   │
        └──────────┘
              │ heartbeat_failure (3x)
              ▼
        ┌──────────┐
        │Disconnected│
        └──────────┘
```

---

## 错误处理

| 错误类型 | HTTP状态码 | 说明 |
|---------|-----------|------|
| worker_not_found | 404 | Worker ID 不存在 |
| goal_not_found | 404 | Goal ID 不存在 |
| invalid_request | 400 | 请求格式错误 |
| timeout | 408 | 执行超时 |
| internal_error | 500 | 内部错误 |