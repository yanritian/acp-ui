# ACP Worker SDK

三语言 Worker SDK，用于实现 ACP-UI 的自定义 Worker。

## TypeScript SDK

```typescript
import { createWorker, AcpWorker } from 'acp-worker-sdk';

// 快速创建 Worker
const worker = await createWorker({
  workerType: 'custom',
  capabilities: ['code-generation', 'file-operations'],
  taskHandler: async (task) => {
    // 执行任务
    const result = await executeTask(task.prompt);
    
    return {
      taskId: task.taskId,
      output: result,
      success: true,
    };
  },
});

// 手动管理 Worker
const worker = new AcpWorker({
  workerId: 'my-worker',
  workerType: 'custom',
  capabilities: ['analysis'],
});

await worker.register();
worker.setTaskHandler(handler);
await worker.shutdown();
```

## Python SDK

```python
from acp_worker_sdk import AcpWorker

worker = AcpWorker(
    worker_id='py-worker',
    worker_type='custom',
    capabilities=['data-processing']
)

worker.register()
worker.set_task_handler(my_handler)
worker.shutdown()
```

## Rust SDK

```rust
use acp_worker_sdk::AcpWorker;

let worker = AcpWorker::new("rust-worker", "custom", vec!["compute"]);
worker.register()?;
worker.set_task_handler(my_handler);
worker.shutdown()?;
```

## API

| 方法 | 说明 |
|------|------|
| `register()` | 注册 Worker 到 ACP-UI |
| `set_task_handler()` | 设置任务处理回调 |
| `execute_task()` | 执行任务并上报结果 |
| `send_heartbeat()` | 发送心跳（自动 10 秒） |
| `update_status()` | 更新 Worker 状态 |
| `shutdown()` | 关闭 Worker |

## Worker Types

- `codex` - Codex 执行器
- `claude_code` - Claude Code 执行器
- `custom` - 自定义 Worker

## Capabilities

推荐能力标签：
- `code-generation` - 代码生成
- `file-operations` - 文件操作
- `analysis` - 分析任务
- `testing` - 测试执行
- `documentation` - 文档生成