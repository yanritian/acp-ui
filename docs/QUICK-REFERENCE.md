# Hermes Game Operator 开发者快速参考

## 常用命令

```bash
# 开发
npm run dev              # 启动前端开发服务器
npm run tauri dev        # 启动 Tauri 应用

# 构建
npm run build            # 构建前端
npm run tauri build      # 构建应用

# 测试
npm run test             # 运行单元测试
npm run test:e2e         # 运行 E2E 测试

# 代码检查
npm run lint             # ESLint 检查
npm run typecheck        # TypeScript 类型检查
```

## API 快速参考

### 任务管理

```typescript
// 启动任务
await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: 'D:/projects/my-game',
  goal: '添加二段跳功能'
})

// 获取任务
const task = await OperatorApi.getTask('task_123')

// 暂停任务
await OperatorApi.pauseTask('task_123')

// 继续任务
await OperatorApi.resumeTask('task_123')

// 停止任务
await OperatorApi.stopTask('task_123')
```

### 审批流程

```typescript
// 获取待审批列表
const approvals = await OperatorApi.getPendingApprovals('task_123')

// 审批操作
await OperatorApi.approve({
  task_id: 'task_123',
  approval_id: 'approval_1',
  decision: 'approve'  // 或 'reject'
})
```

### 文件操作

```typescript
// 读取文件
const content = await OperatorApi.fileRead('task_123', 'scripts/Player.gd')

// 修改文件
await OperatorApi.filePatch('task_123', 'scripts/Player.gd', 'new content')

// 预览修改
const diff = await OperatorApi.filePatchPreview('task_123', 'scripts/Player.gd', 'new')
```

## 状态机

```
Idle → Planning → WaitingApproval → Running → Completed
                       ↓                   ↓
                    Cancelled           Paused
```

## 审批等级

| 等级 | 行为 |
|------|------|
| silent | 自动执行 |
| notify | 仅通知 |
| approve | 需审批 |
| forbidden | 禁止 |

## 安全边界

### 允许的路径
- 项目目录内的文件
- 相对于项目根目录的路径

### 禁止的路径
- `../` 父目录遍历
- `/etc/` 系统目录
- `C:/Windows/` Windows 系统目录

### 禁止的命令字符
- `;` 命令分隔符
- `|` 管道
- `&` AND 操作符
- `$()` 命令替换
- `` ` `` 反引号

## 错误码

| 错误码 | 说明 |
|--------|------|
| TASK_NOT_FOUND | 任务不存在 |
| INVALID_STATE_TRANSITION | 无效状态转换 |
| PATH_OUTSIDE_BOUNDARY | 路径越权 |
| COMMAND_FORBIDDEN | 命令禁止 |
| APPROVAL_REQUIRED | 需要审批 |

## 文件结构

```
src/features/game-operator/
├── views/
│   └── GameOperatorView.vue
└── components/
    ├── OperatorControlBar.vue
    ├── ProgressTimeline.vue
    ├── PlanPanel.vue
    └── ApprovalDrawer.vue

src-tauri/src/operator/
├── types.rs
├── state_machine.rs
├── commands.rs
├── security.rs
├── file_tools.rs
├── approval_queue.rs
└── hermes_cli_bridge.rs
```

## 环境变量

```env
HERMES_API_ENDPOINT=https://api.hermes.ai
HERMES_API_KEY=your-api-key
GODOT_PATH=/path/to/godot
LOG_LEVEL=info
```

## 快速启动

```bash
# 1. 安装依赖
npm install

# 2. 运行测试
npm run test

# 3. 启动开发
npm run tauri dev

# 4. 导航到 Game Operator
# http://localhost:1420/#/games
```