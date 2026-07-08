# Hermes Game Operator API 参考

> 版本: v0.1.0
> 更新日期: 2026-07-08

---

## 1. 概述

Hermes Game Operator API 提供了完整的任务生命周期管理接口。

### 1.1 基础 URL

```
tauri://operator
```

### 1.2 认证

当前版本使用 Tauri 内部调用，无需额外认证。

---

## 2. 任务管理 API

### 2.1 创建任务

```typescript
OperatorApi.startTask(request: StartTaskRequest): Promise<StartTaskResponse>
```

**请求参数**:

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| domain | string | ✅ | 领域标识，如 `'game.godot'` |
| project_path | string | ✅ | 项目绝对路径 |
| goal | string | ✅ | 任务目标描述 |
| mode | string | ❌ | 执行模式，默认 `'propose_then_apply'` |
| approval_policy | string | ❌ | 审批策略，默认 `'safe_default'` |

**示例**:

```typescript
const response = await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: 'D:/projects/my-game',
  goal: '给 Player 添加二段跳能力',
  mode: 'propose_then_apply',
  approval_policy: 'safe_default'
})

// 返回
{
  task_id: 'task_1234567890',
  status: 'planning',
  event_stream: 'operator://tasks/task_1234567890/events'
}
```

---

### 2.2 获取任务

```typescript
OperatorApi.getTask(taskId: string): Promise<OperatorTask>
```

**返回字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| task_id | string | 任务唯一标识 |
| domain | string | 领域标识 |
| project_path | string | 项目路径 |
| goal | string | 任务目标 |
| status | OperatorTaskStatus | 当前状态 |
| mode | string | 执行模式 |
| approval_policy | string | 审批策略 |
| created_at | string | 创建时间 (ISO 8601) |
| updated_at | string | 更新时间 (ISO 8601) |
| started_at | string? | 开始时间 |
| completed_at | string? | 完成时间 |
| summary | string? | 任务总结 |
| error | string? | 错误信息 |

---

### 2.3 列出任务

```typescript
OperatorApi.listTasks(): Promise<OperatorTask[]>
```

---

### 2.4 暂停任务

```typescript
OperatorApi.pauseTask(taskId: string): Promise<void>
```

**前置条件**: 任务状态为 `running`

---

### 2.5 继续任务

```typescript
OperatorApi.resumeTask(taskId: string): Promise<void>
```

**前置条件**: 任务状态为 `paused`

---

### 2.6 停止任务

```typescript
OperatorApi.stopTask(taskId: string): Promise<void>
```

**前置条件**: 任务状态为 `running` 或 `paused`

---

### 2.7 重定向任务

```typescript
OperatorApi.redirectTask(request: RedirectRequest): Promise<void>
```

**请求参数**:

| 字段 | 类型 | 说明 |
|------|------|------|
| task_id | string | 任务 ID |
| new_goal | string | 新目标 |

---

## 3. 审批 API

### 3.1 审批操作

```typescript
OperatorApi.approve(request: ApproveRequest): Promise<void>
```

**请求参数**:

| 字段 | 类型 | 说明 |
|------|------|------|
| task_id | string | 任务 ID |
| approval_id | string | 审批请求 ID |
| decision | 'approve' \| 'reject' \| 'request_changes' | 决定 |
| comment | string? | 评论 |

---

### 3.2 获取待审批列表

```typescript
OperatorApi.getPendingApprovals(taskId: string): Promise<ApprovalRequest[]>
```

**返回字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| approval_id | string | 审批 ID |
| task_id | string | 任务 ID |
| level | ApprovalLevel | 审批级别 |
| action | string | 操作类型 |
| title | string | 标题 |
| reason | string | 原因 |
| risk | string? | 风险说明 |
| preview | object? | 预览数据 |

---

## 4. 事件 API

### 4.1 列出事件

```typescript
OperatorApi.listEvents(taskId: string, limit?: number): Promise<OperatorEvent[]>
```

**返回字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| event_id | string | 事件 ID |
| task_id | string | 任务 ID |
| timestamp | string | 时间戳 (ISO 8601) |
| type | OperatorEventType | 事件类型 |
| level | EventLevel | 事件级别 |
| title | string | 标题 |
| message | string? | 详细消息 |
| source | string | 来源 |
| payload | object? | 附加数据 |

---

## 5. 文件 API

### 5.1 读取文件

```typescript
OperatorApi.fileRead(taskId: string, path: string): Promise<FileReadResult>
```

### 5.2 修改文件

```typescript
OperatorApi.filePatch(
  taskId: string,
  path: string,
  newContent: string,
  createBackup?: boolean
): Promise<FilePatchResult>
```

### 5.3 预览修改

```typescript
OperatorApi.filePatchPreview(
  taskId: string,
  path: string,
  newContent: string
): Promise<FilePatch>
```

### 5.4 列出文件

```typescript
OperatorApi.fileList(taskId: string, path: string): Promise<FileListResult>
```

---

## 6. Godot API

### 6.1 检测项目

```typescript
GodotOperatorApi.detectProject(path: string): Promise<boolean>
```

### 6.2 分析项目

```typescript
GodotOperatorApi.analyzeProject(projectPath: string): Promise<GodotProjectInfo>
```

**返回字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| project_name | string | 项目名称 |
| godot_version | string? | Godot 版本 |
| scripts | string[] | 脚本文件列表 |
| scenes | string[] | 场景文件列表 |
| assets | string[] | 资源文件列表 |
| main_scene | string? | 主场景 |

---

## 7. Hermes CLI API

### 7.1 检查连接

```typescript
HermesCliApi.checkConnection(): Promise<HermesConnectionStatus>
```

### 7.2 执行任务

```typescript
HermesCliApi.executeTask(
  taskId: string,
  goal: string,
  projectPath: string
): Promise<void>
```

### 7.3 分析项目

```typescript
HermesCliApi.analyzeProject(taskId: string): Promise<HermesAnalysisResult>
```

### 7.4 生成计划

```typescript
HermesCliApi.generatePlan(
  taskId: string,
  goal: string
): Promise<HermesPlan>
```

### 7.5 执行步骤

```typescript
HermesCliApi.executeStep(
  taskId: string,
  stepId: number,
  approve: boolean
): Promise<void>
```

---

## 8. 类型定义

### 8.1 OperatorTaskStatus

```typescript
type OperatorTaskStatus =
  | 'idle'
  | 'planning'
  | 'waiting_approval'
  | 'running'
  | 'paused'
  | 'redirecting'
  | 'cancelling'
  | 'cancelled'
  | 'failed'
  | 'completed'
```

### 8.2 OperatorEventType

```typescript
type OperatorEventType =
  | 'task_created'
  | 'task_started'
  | 'project_analyzed'
  | 'plan_started'
  | 'plan_ready'
  | 'approval_requested'
  | 'approval_granted'
  | 'approval_rejected'
  | 'tool_call_started'
  | 'tool_call_succeeded'
  | 'tool_call_failed'
  | 'file_read'
  | 'file_patch_proposed'
  | 'file_patch_applied'
  | 'task_paused'
  | 'task_resumed'
  | 'task_redirected'
  | 'task_cancelling'
  | 'task_cancelled'
  | 'task_failed'
  | 'task_completed'
  | 'summary_ready'
```

### 8.3 ApprovalLevel

```typescript
type ApprovalLevel = 'silent' | 'notify' | 'approve' | 'forbidden'
```

### 8.4 EventLevel

```typescript
type EventLevel = 'info' | 'warning' | 'error' | 'debug'
```

---

## 9. 错误处理

### 9.1 错误码

| 错误码 | 说明 |
|--------|------|
| TASK_NOT_FOUND | 任务不存在 |
| INVALID_STATE_TRANSITION | 无效状态转换 |
| PATH_OUTSIDE_BOUNDARY | 路径越权 |
| COMMAND_FORBIDDEN | 命令禁止 |
| APPROVAL_REQUIRED | 需要审批 |
| HERMES_CLI_NOT_FOUND | Hermes CLI 未找到 |

### 9.2 错误示例

```typescript
try {
  await OperatorApi.pauseTask('invalid_task_id')
} catch (error) {
  // Error: Task not found: invalid_task_id
}
```

---

## 10. 最佳实践

1. **先检测项目**: 使用 `detectProject` 验证路径
2. **轮询事件**: 每 2 秒轮询 `listEvents` 获取进度
3. **处理审批**: 检查 `getPendingApprovals` 并及时响应
4. **优雅停止**: 使用 `stopTask` 而非强制终止
5. **错误处理**: 捕获所有 API 错误并展示给用户