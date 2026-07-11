# API 使用指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 快速开始

### 安装 SDK

```bash
# Node.js/TypeScript
npm install @hermes-game-operator/sdk

# Python
pip install hermes-game-operator

# Go
go get github.com/hermes-game-operator/go-sdk
```

### 初始化客户端

```typescript
import { HermesClient } from '@hermes-game-operator/sdk';

const client = new HermesClient({
  serverUrl: 'https://your-domain.com',
  authToken: 'your-auth-token'
});
```

---

## 认证

### 获取访问令牌

```typescript
// 使用 OIDC
const token = await client.authenticate({
  grantType: 'authorization_code',
  code: 'authorization-code',
  redirectUri: 'https://your-app.com/callback'
});

// 使用客户端凭据
const token = await client.authenticate({
  grantType: 'client_credentials',
  clientId: 'your-client-id',
  clientSecret: 'your-client-secret'
});
```

### 刷新令牌

```typescript
const newToken = await client.refreshToken({
  refreshToken: 'your-refresh-token'
});
```

---

## 任务管理

### 列出任务

```typescript
// 基本列表
const tasks = await client.tasks.list();

// 带过滤
const tasks = await client.tasks.list({
  status: 'running',
  limit: 50,
  offset: 0
});

// 带排序
const tasks = await client.tasks.list({
  sortBy: 'created_at',
  sortOrder: 'desc'
});
```

### 获取任务详情

```typescript
const task = await client.tasks.get('task_001');

console.log(task.task_id);
console.log(task.status);
console.log(task.goal);
```

### 创建任务

```typescript
// 基本创建
const task = await client.tasks.create({
  domain: 'game.godot',
  projectPath: '/path/to/project',
  goal: 'Add player movement'
});

// 完整创建
const task = await client.tasks.create({
  domain: 'game.godot',
  projectPath: '/path/to/project',
  goal: 'Add player movement',
  mode: 'propose_then_apply',
  approvalPolicy: 'safe_default',
  metadata: {
    priority: 'high',
    tags: ['movement', 'player']
  }
});
```

### 更新任务

```typescript
const task = await client.tasks.update('task_001', {
  goal: 'Updated goal',
  status: 'paused'
});
```

### 删除任务

```typescript
await client.tasks.delete('task_001');
```

---

## 任务控制

### 暂停任务

```typescript
await client.tasks.pause('task_001');
```

### 恢复任务

```typescript
await client.tasks.resume('task_001');
```

### 停止任务

```typescript
await client.tasks.stop('task_001');
```

### 重定向任务

```typescript
await client.tasks.redirect('task_001', {
  newGoal: 'New goal direction',
  preserveCompletedWork: true
});
```

---

## 事件管理

### 列出事件

```typescript
// 基本列表
const events = await client.events.list('task_001');

// 带过滤
const events = await client.events.list('task_001', {
  type: 'task_started',
  limit: 100
});

// 带时间范围
const events = await client.events.list('task_001', {
  startTime: '2026-07-01T00:00:00Z',
  endTime: '2026-07-31T23:59:59Z'
});
```

### 获取事件详情

```typescript
const event = await client.events.get('task_001', 'evt_001');
```

### 订阅事件

```typescript
// WebSocket 订阅
const subscription = client.events.subscribe('task_001', {
  types: ['task_started', 'task_completed']
});

subscription.on('event', (event) => {
  console.log('New event:', event);
});

subscription.on('error', (error) => {
  console.error('Subscription error:', error);
});
```

---

## 审批管理

### 列出待审批

```typescript
// 所有待审批
const approvals = await client.approvals.list();

// 特定任务的待审批
const approvals = await client.approvals.list({
  taskId: 'task_001'
});
```

### 审批请求

```typescript
// 批准
await client.approvals.approve('approval_001', {
  reason: 'Approved by manager'
});

// 拒绝
await client.approvals.reject('approval_001', {
  reason: 'Not ready for production'
});

// 请求修改
await client.approvals.requestChanges('approval_001', {
  reason: 'Need to update documentation',
  changes: [
    { field: 'documentation', description: 'Update API docs' }
  ]
});
```

---

## 文件操作

### 读取文件

```typescript
const file = await client.files.read('task_001', 'scripts/Player.gd');

console.log(file.content);
console.log(file.size);
console.log(file.modified);
```

### 写入文件

```typescript
await client.files.write('task_001', 'scripts/Player.gd', {
  content: 'new file content',
  createBackup: true
});
```

### 列出文件

```typescript
// 基本列表
const files = await client.files.list('task_001', 'scripts');

// 递归列表
const files = await client.files.list('task_001', 'scripts', {
  recursive: true
});

// 带过滤
const files = await client.files.list('task_001', 'scripts', {
  pattern: '*.gd'
});
```

### 删除文件

```typescript
await client.files.delete('task_001', 'scripts/OldScript.gd', {
  createBackup: true
});
```

---

## 高级用法

### 批处理操作

```typescript
// 批量创建任务
const tasks = await client.tasks.batchCreate([
  { goal: 'Task 1', projectPath: '/project1' },
  { goal: 'Task 2', projectPath: '/project2' },
  { goal: 'Task 3', projectPath: '/project3' }
]);

// 批量更新任务
await client.tasks.batchUpdate([
  { taskId: 'task_1', status: 'paused' },
  { taskId: 'task_2', status: 'paused' }
]);
```

### 事务操作

```typescript
// 事务性更新
await client.transaction(async (tx) => {
  const task = await tx.tasks.create({
    goal: 'New task'
  });
  
  await tx.events.create(task.task_id, {
    type: 'task_created'
  });
  
  return task;
});
```

### 重试策略

```typescript
const client = new HermesClient({
  serverUrl: 'https://your-domain.com',
  retry: {
    maxAttempts: 3,
    backoff: 'exponential',
    initialDelay: 1000
  }
});
```

---

## 错误处理

### 错误类型

```typescript
import { 
  AuthenticationError,
  AuthorizationError,
  NotFoundError,
  ValidationError,
  RateLimitError
} from '@hermes-game-operator/sdk';

try {
  const task = await client.tasks.get('task_001');
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed');
  } else if (error instanceof NotFoundError) {
    console.error('Task not found');
  } else if (error instanceof RateLimitError) {
    console.error('Rate limit exceeded');
    console.error('Retry after:', error.retryAfter);
  }
}
```

### 错误恢复

```typescript
async function withRetry(fn, maxAttempts = 3) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === maxAttempts - 1) throw error;
      if (error instanceof RateLimitError) {
        await sleep(error.retryAfter * 1000);
      }
    }
  }
}

const task = await withRetry(() => client.tasks.get('task_001'));
```

---

## 最佳实践

### 1. 使用分页

```typescript
// ❌ 不推荐：获取所有数据
const allTasks = await client.tasks.list({ limit: 10000 });

// ✅ 推荐：使用分页
let offset = 0;
const limit = 50;
let tasks = [];

while (true) {
  const page = await client.tasks.list({ limit, offset });
  tasks = [...tasks, ...page];
  
  if (page.length < limit) break;
  offset += limit;
}
```

### 2. 缓存响应

```typescript
const cache = new Map();

async function getCachedTask(taskId) {
  if (cache.has(taskId)) {
    return cache.get(taskId);
  }
  
  const task = await client.tasks.get(taskId);
  cache.set(taskId, task);
  
  setTimeout(() => cache.delete(taskId), 60000); // 1 分钟缓存
  
  return task;
}
```

### 3. 使用 WebSocket 实时更新

```typescript
const subscription = client.events.subscribe('task_001');

subscription.on('event', (event) => {
  // 更新 UI
  updateTaskStatus(event.data);
});
```

### 4. 处理并发

```typescript
// ✅ 使用 Promise.all 并行请求
const [task1, task2, task3] = await Promise.all([
  client.tasks.get('task_1'),
  client.tasks.get('task_2'),
  client.tasks.get('task_3')
]);
```

---

## 调试

### 启用日志

```typescript
const client = new HermesClient({
  serverUrl: 'https://your-domain.com',
  debug: true,
  logger: {
    info: console.log,
    error: console.error,
    debug: console.debug
  }
});
```

### 检查请求

```typescript
client.on('request', (request) => {
  console.log('Request:', request.method, request.url);
});

client.on('response', (response) => {
  console.log('Response:', response.status);
});
```

---

## 示例应用

### 任务监控仪表板

```typescript
import { HermesClient } from '@hermes-game-operator/sdk';

const client = new HermesClient({
  serverUrl: 'https://your-domain.com',
  authToken: 'your-token'
});

// 订阅所有任务事件
const subscription = client.events.subscribeAll();

subscription.on('event', async (event) => {
  const task = await client.tasks.get(event.taskId);
  
  // 更新仪表板
  updateDashboard(task);
});
```

### 自动化工作流

```typescript
// 自动审批符合条件的事件
client.events.subscribe('task_001', {
  types: ['approval_requested']
}).on('event', async (event) => {
  if (event.data.level === 'info') {
    await client.approvals.approve(event.approvalId, {
      reason: 'Auto-approved: low risk'
    });
  }
});
```

---

## 参考

- [API 文档](API.md)
- [SDK 文档](SDK.md)
- [示例代码](EXAMPLES.md)
- [故障排除](TROUBLESHOOTING.md)

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator SDK Team