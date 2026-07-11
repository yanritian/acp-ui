# API 版本历史

> 最后更新: 2026-07-11
> 当前版本: v1.0.0-alpha

---

## 版本策略

Hermes Game Operator API 遵循 [语义化版本控制](https://semver.org/) 规范：

- **MAJOR** 版本：不兼容的 API 变更
- **MINOR** 版本：向后兼容的功能添加
- **PATCH** 版本：向后兼容的 bug 修复

---

## v1.0.0-alpha (2026-07-11)

### 🎉 初始版本

#### 任务管理 API

**端点**:

- `GET /api/tasks` - 列出所有任务
- `POST /api/tasks` - 创建新任务
- `GET /api/tasks/{task_id}` - 获取任务详情
- `POST /api/tasks/{task_id}/pause` - 暂停任务
- `POST /api/tasks/{task_id}/resume` - 恢复任务
- `POST /api/tasks/{task_id}/stop` - 停止任务
- `POST /api/tasks/{task_id}/redirect` - 重定向任务

**请求/响应示例**:

```bash
# 创建任务
POST /api/tasks
Content-Type: application/json
Authorization: Bearer {token}

{
  "domain": "game.godot",
  "project_path": "/path/to/project",
  "goal": "Add player movement",
  "mode": "propose_then_apply",
  "approval_policy": "safe_default"
}

# 响应
{
  "task_id": "task_001",
  "status": "planning",
  "event_stream": "operator://tasks/task_001/events"
}
```

---

#### 事件管理 API

**端点**:

- `GET /api/tasks/{task_id}/events` - 获取任务事件列表

**查询参数**:

- `limit` (可选): 返回事件数量限制，默认 100
- `type` (可选): 按事件类型过滤

**请求/响应示例**:

```bash
# 获取事件
GET /api/tasks/task_001/events?limit=50&type=task_started
Authorization: Bearer {token}

# 响应
{
  "events": [
    {
      "event_id": "evt_001",
      "task_id": "task_001",
      "timestamp": "2026-07-11T00:00:00Z",
      "type": "task_started",
      "level": "info",
      "title": "Task started",
      "message": "Task execution has started",
      "source": "operator"
    }
  ]
}
```

---

#### 审批管理 API

**端点**:

- `GET /api/approvals` - 获取待审批列表
- `GET /api/tasks/{task_id}/approvals` - 获取任务审批列表
- `POST /api/approvals/{approval_id}` - 审批请求

**请求/响应示例**:

```bash
# 获取待审批
GET /api/approvals
Authorization: Bearer {token}

# 响应
{
  "approvals": [
    {
      "approval_id": "a001",
      "task_id": "task_001",
      "level": "approve",
      "action": "file.patch",
      "title": "Approve file changes",
      "reason": "File modification required",
      "preview": {
        "files": ["scripts/Player.gd"],
        "diffs": [
          {
            "path": "scripts/Player.gd",
            "operation": "replace",
            "diff": "@@ -10,3 +10,5 @@ ..."
          }
        ]
      }
    }
  ]
}
```

```bash
# 审批请求
POST /api/approvals/a001
Content-Type: application/json
Authorization: Bearer {token}

{
  "decision": "approve"
}

# 响应
{
  "success": true
}
```

---

#### 认证 API

**端点**:

- `GET /health` - 健康检查
- `GET /auth/authorize` - 获取授权 URL
- `POST /auth/token` - 交换授权码
- `POST /auth/refresh` - 刷新令牌
- `GET /auth/userinfo` - 获取用户信息

**请求/响应示例**:

```bash
# 健康检查
GET /health

# 响应
{
  "status": "ok",
  "version": "1.0.0-alpha"
}
```

```bash
# 获取授权 URL
GET /auth/authorize?state=random-state-123

# 响应
{
  "authorization_url": "https://auth.example.com/authorize?..."
}
```

```bash
# 交换授权码
POST /auth/token
Content-Type: application/json

{
  "grant_type": "authorization_code",
  "code": "auth-code-123",
  "redirect_uri": "http://localhost:8080/callback"
}

# 响应
{
  "access_token": "access-token-123",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "refresh-token-123",
  "id_token": "id-token-123"
}
```

---

## 弃用通知

### 无弃用

当前版本无弃用的 API。

---

## 安全注意事项

### 认证

- 所有 API 端点（除了 `/health`）都需要 Bearer Token 认证
- Token 通过 `Authorization` 头传递
- Token 有效期为 1 小时

### 速率限制

- 默认限制：100 请求/秒
- 突发限制：200 请求
- 超过限制返回 `429 Too Many Requests`

### CORS

- 允许的源：`http://localhost:3000`, `https://acp-ui.github.io`
- 允许的方法：`GET`, `POST`, `PUT`, `DELETE`
- 允许的头：`Authorization`, `Content-Type`

---

## 错误处理

### 错误响应格式

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Error description",
    "details": "Additional details"
  }
}
```

### 常见错误码

| 错误码 | HTTP 状态码 | 说明 |
|--------|-----------|------|
| `INVALID_REQUEST` | 400 | 请求格式错误 |
| `UNAUTHORIZED` | 401 | 认证失败 |
| `FORBIDDEN` | 403 | 权限不足 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `CONFLICT` | 409 | 状态冲突 |
| `RATE_LIMIT_EXCEEDED` | 429 | 速率限制 |
| `INTERNAL_ERROR` | 500 | 内部错误 |

---

## SDK 支持

### 官方 SDK

- **TypeScript/JavaScript**: `@hermes-game-operator/sdk`
- **Python**: `hermes-game-operator`
- **Go**: `github.com/hermes-game-operator/go-sdk`

### 示例代码

```typescript
import { HermesClient } from '@hermes-game-operator/sdk';

const client = new HermesClient({
  serverUrl: 'http://localhost:8080',
  authToken: 'your-token'
});

// 列出任务
const tasks = await client.listTasks();

// 创建任务
const task = await client.createTask({
  goal: 'Add player movement',
  projectPath: '/path/to/project'
});

// 审批任务
await client.approve('a001', 'approve');
```

---

## 变更日志

### v1.0.0-alpha (2026-07-11)

- ✅ 初始版本发布
- ✅ 任务管理 API
- ✅ 事件管理 API
- ✅ 审批管理 API
- ✅ 认证 API
- ✅ 完整的错误处理
- ✅ 速率限制
- ✅ CORS 支持

---

## 未来计划

### v1.1.0-beta (计划中)

- 🔄 任务模板 API
- 🔄 任务依赖关系 API
- 🔄 多级审批 API
- 🔄 事件过滤和搜索
- 🔄 批量操作 API

### v1.2.0-stable (计划中)

- 📋 GraphQL API
- 📋 WebSocket 实时 API
- 📋 文件操作 API
- 📋 审计日志 API

---

## 反馈和建议

我们欢迎所有形式的反馈和建议！

- **GitHub Issues**: 报告 bug 或请求功能
- **Discord**: 加入社区讨论
- **Email**: api-feedback@example.com

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator API Team
**版本**: 1.0.0-alpha