# ACP-UI API 文档

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [Tauri 命令](#tauri-命令)
2. [REST API](#rest-api)
3. [WebSocket API](#websocket-api)
4. [类型定义](#类型定义)
5. [错误处理](#错误处理)

---

## Tauri 命令

### 游戏操作员 (Game Operator)

#### `game_operator_create`

创建新的游戏操作员实例。

**参数**:
```typescript
{
  project_path: string;      // Godot 项目路径
  agent_config?: AgentConfig; // Agent 配置
}
```

**返回**:
```typescript
{
  operator_id: string;
  status: 'idle' | 'planning' | 'running' | 'completed' | 'error';
  created_at: string;
}
```

**示例**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('game_operator_create', {
  project_path: '/path/to/godot/project',
  agent_config: {
    model: 'gpt-4',
    temperature: 0.7
  }
});
```

---

#### `game_operator_start`

启动游戏操作员。

**参数**:
```typescript
{
  operator_id: string;
  plan?: ExecutionPlan; // 执行计划（可选）
}
```

**返回**:
```typescript
{
  success: boolean;
  task_id: string;
  events: Event[];
}
```

---

#### `game_operator_stop`

停止游戏操作员。

**参数**:
```typescript
{
  operator_id: string;
  reason?: string;
}
```

**返回**:
```typescript
{
  success: boolean;
  stopped_at: string;
}
```

---

#### `game_operator_status`

获取游戏操作员状态。

**参数**:
```typescript
{
  operator_id: string;
}
```

**返回**:
```typescript
{
  operator_id: string;
  status: OperatorStatus;
  current_task?: Task;
  progress: number;
  events: Event[];
}
```

---

### 任务管理 (Task Management)

#### `task_create`

创建新任务。

**参数**:
```typescript
{
  operator_id: string;
  title: string;
  description: string;
  priority?: 'low' | 'medium' | 'high';
  metadata?: Record<string, any>;
}
```

**返回**:
```typescript
{
  task_id: string;
  created_at: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
}
```

---

#### `task_update`

更新任务。

**参数**:
```typescript
{
  task_id: string;
  updates: {
    title?: string;
    description?: string;
    status?: TaskStatus;
    metadata?: Record<string, any>;
  };
}
```

**返回**:
```typescript
{
  success: boolean;
  updated_at: string;
}
```

---

#### `task_delete`

删除任务。

**参数**:
```typescript
{
  task_id: string;
}
```

**返回**:
```typescript
{
  success: boolean;
  deleted_at: string;
}
```

---

#### `task_list`

列出所有任务。

**参数**:
```typescript
{
  operator_id: string;
  filter?: {
    status?: TaskStatus[];
    priority?: Priority[];
    limit?: number;
    offset?: number;
  };
}
```

**返回**:
```typescript
{
  tasks: Task[];
  total: number;
  has_more: boolean;
}
```

---

### 审批系统 (Approval System)

#### `approval_queue_list`

列出审批队列。

**参数**:
```typescript
{
  operator_id?: string;
  status?: 'pending' | 'approved' | 'rejected';
}
```

**返回**:
```typescript
{
  approvals: Approval[];
  total: number;
}
```

---

#### `approval_resolve`

处理审批请求。

**参数**:
```typescript
{
  approval_id: string;
  decision: 'approve' | 'reject';
  reason?: string;
  resolved_by: string;
}
```

**返回**:
```typescript
{
  success: boolean;
  resolved_at: string;
  decision: 'approve' | 'reject';
}
```

---

### 文件工具 (File Tools)

#### `file_read`

读取文件内容。

**参数**:
```typescript
{
  path: string;
  encoding?: 'utf-8' | 'base64';
}
```

**返回**:
```typescript
{
  content: string;
  size: number;
  modified_at: string;
}
```

---

#### `file_write`

写入文件。

**参数**:
```typescript
{
  path: string;
  content: string;
  create_dirs?: boolean;
}
```

**返回**:
```typescript
{
  success: boolean;
  bytes_written: number;
}
```

---

#### `file_patch`

应用补丁。

**参数**:
```typescript
{
  path: string;
  patch: string;
  dry_run?: boolean;
}
```

**返回**:
```typescript
{
  success: boolean;
  changes: FileChange[];
}
```

---

#### `file_patch_preview`

预览补丁。

**参数**:
```typescript
{
  path: string;
  patch: string;
}
```

**返回**:
```typescript
{
  diff: string;
  changes: FileChange[];
  risk: 'low' | 'medium' | 'high';
}
```

---

#### `file_list`

列出目录内容。

**参数**:
```typescript
{
  path: string;
  recursive?: boolean;
  pattern?: string;
}
```

**返回**:
```typescript
{
  files: FileInfo[];
  directories: DirectoryInfo[];
  total: number;
}
```

---

### 系统 (System)

#### `system_status`

获取系统状态。

**参数**: 无

**返回**:
```typescript
{
  version: string;
  uptime: number;
  operators: number;
  tasks: number;
  memory_usage: number;
  cpu_usage: number;
}
```

---

#### `system_health`

获取系统健康状态。

**参数**: 无

**返回**:
```typescript
{
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: HealthCheck[];
  last_check: string;
}
```

---

## REST API

### 基础信息

- **Base URL**: `http://localhost:3000/api`
- **Content-Type**: `application/json`
- **Authentication**: Bearer Token

---

### 端点

#### `GET /api/operators`

获取所有操作员列表。

**响应**:
```json
{
  "operators": [
    {
      "operator_id": "op_001",
      "status": "running",
      "created_at": "2026-07-12T10:00:00Z"
    }
  ],
  "total": 1
}
```

---

#### `POST /api/operators`

创建新操作员。

**请求**:
```json
{
  "project_path": "/path/to/project",
  "agent_config": {
    "model": "gpt-4",
    "temperature": 0.7
  }
}
```

**响应**:
```json
{
  "operator_id": "op_001",
  "status": "idle",
  "created_at": "2026-07-12T10:00:00Z"
}
```

---

#### `GET /api/operators/:id/status`

获取操作员状态。

**响应**:
```json
{
  "operator_id": "op_001",
  "status": "running",
  "current_task": {
    "task_id": "task_001",
    "title": "Implement player movement",
    "status": "running"
  },
  "progress": 0.75
}
```

---

#### `POST /api/operators/:id/start`

启动操作员。

**请求**:
```json
{
  "plan": {
    "steps": [
      {
        "action": "file.write",
        "path": "scripts/Player.gd",
        "content": "..."
      }
    ]
  }
}
```

**响应**:
```json
{
  "success": true,
  "task_id": "task_001"
}
```

---

#### `POST /api/operators/:id/stop`

停止操作员。

**请求**:
```json
{
  "reason": "User requested stop"
}
```

**响应**:
```json
{
  "success": true,
  "stopped_at": "2026-07-12T10:05:00Z"
}
```

---

#### `GET /api/approvals`

获取审批队列。

**查询参数**:
- `status`: pending | approved | rejected
- `operator_id`: 操作员ID

**响应**:
```json
{
  "approvals": [
    {
      "approval_id": "approval_001",
      "task_id": "task_001",
      "level": "approve",
      "action": "file.write",
      "title": "Create Player.gd",
      "status": "pending",
      "created_at": "2026-07-12T10:00:00Z"
    }
  ],
  "total": 1
}
```

---

#### `POST /api/approvals/:id/resolve`

处理审批。

**请求**:
```json
{
  "decision": "approve",
  "reason": "Looks good",
  "resolved_by": "user_001"
}
```

**响应**:
```json
{
  "success": true,
  "resolved_at": "2026-07-12T10:05:00Z"
}
```

---

## WebSocket API

### 连接

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');
```

### 事件类型

#### `operator.status`

操作员状态更新。

```json
{
  "type": "operator.status",
  "data": {
    "operator_id": "op_001",
    "status": "running",
    "progress": 0.75
  }
}
```

---

#### `task.progress`

任务进度更新。

```json
{
  "type": "task.progress",
  "data": {
    "task_id": "task_001",
    "progress": 0.5,
    "message": "Processing step 3/6"
  }
}
```

---

#### `approval.request`

审批请求。

```json
{
  "type": "approval.request",
  "data": {
    "approval_id": "approval_001",
    "action": "file.write",
    "title": "Create Player.gd",
    "level": "approve"
  }
}
```

---

#### `event.stream`

事件流。

```json
{
  "type": "event.stream",
  "data": {
    "operator_id": "op_001",
    "event": {
      "event_id": "evt_001",
      "type": "file.write",
      "timestamp": "2026-07-12T10:00:00Z",
      "data": {
        "path": "scripts/Player.gd",
        "size": 1024
      }
    }
  }
}
```

---

## 类型定义

### OperatorStatus

```typescript
type OperatorStatus = 
  | 'idle'
  | 'planning'
  | 'running'
  | 'completed'
  | 'error';
```

### TaskStatus

```typescript
type TaskStatus =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'cancelled';
```

### ApprovalLevel

```typescript
type ApprovalLevel =
  | 'silent'    // 自动批准
  | 'notify'    // 通知但不阻塞
  | 'approve'   // 需要批准
  | 'forbidden'; // 禁止
```

### Event

```typescript
interface Event {
  event_id: string;
  type: string;
  timestamp: string;
  data: any;
  metadata?: Record<string, any>;
}
```

### Task

```typescript
interface Task {
  task_id: string;
  operator_id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: 'low' | 'medium' | 'high';
  created_at: string;
  updated_at: string;
  metadata?: Record<string, any>;
}
```

### Approval

```typescript
interface Approval {
  approval_id: string;
  task_id: string;
  level: ApprovalLevel;
  action: string;
  title: string;
  reason?: string;
  status: 'pending' | 'approved' | 'rejected';
  created_at: string;
  resolved_at?: string;
  resolved_by?: string;
}
```

---

## 错误处理

### 错误格式

```typescript
interface ApiError {
  code: string;
  message: string;
  details?: any;
  i18n?: {
    key: string;
    params?: Record<string, any>;
  };
}
```

### 常见错误码

| 错误码 | 描述 | HTTP状态码 |
|--------|------|-----------|
| `OPERATOR_NOT_FOUND` | 操作员不存在 | 404 |
| `TASK_NOT_FOUND` | 任务不存在 | 404 |
| `APPROVAL_NOT_FOUND` | 审批不存在 | 404 |
| `INVALID_STATE` | 无效状态转换 | 400 |
| `PERMISSION_DENIED` | 权限被拒绝 | 403 |
| `PATH_VIOLATION` | 路径访问违规 | 403 |
| `COMMAND_BLOCKED` | 命令被阻止 | 403 |
| `INTERNAL_ERROR` | 内部错误 | 500 |

### 错误示例

```json
{
  "code": "OPERATOR_NOT_FOUND",
  "message": "Operator op_999 not found",
  "i18n": {
    "key": "errors.operator.notFound",
    "params": {
      "operator_id": "op_999"
    }
  }
}
```

---

## 示例代码

### TypeScript

```typescript
import { invoke } from '@tauri-apps/api/core';

// 创建操作员
const operator = await invoke('game_operator_create', {
  project_path: '/path/to/project'
});

// 启动操作员
await invoke('game_operator_start', {
  operator_id: operator.operator_id
});

// 获取状态
const status = await invoke('game_operator_status', {
  operator_id: operator.operator_id
});

// 列出审批
const approvals = await invoke('approval_queue_list', {
  operator_id: operator.operator_id
});

// 批准请求
await invoke('approval_resolve', {
  approval_id: approvals[0].approval_id,
  decision: 'approve',
  resolved_by: 'user_001'
});
```

### Python

```python
import requests

# 创建操作员
response = requests.post('http://localhost:3000/api/operators', json={
    'project_path': '/path/to/project'
})
operator = response.json()

# 启动操作员
requests.post(f'http://localhost:3000/api/operators/{operator["operator_id"]}/start')

# 获取状态
status = requests.get(f'http://localhost:3000/api/operators/{operator["operator_id"]}/status').json()

# 列出审批
approvals = requests.get('http://localhost:3000/api/approvals').json()

# 批准请求
requests.post(f'http://localhost:3000/api/approvals/{approvals[0]["approval_id"]}/resolve', json={
    'decision': 'approve',
    'resolved_by': 'user_001'
})
```

---

## 更多信息

- [GitHub 仓库](https://github.com/yanritian/acp-ui)
- [完整文档](https://github.com/yanritian/acp-ui/wiki)
- [问题反馈](https://github.com/yanritian/acp-ui/issues)
