/**
 * ACP Worker SDK - TypeScript
 *
 * 使用此 SDK 实现 ACP-UI 的 Worker：
 * - 注册 Worker 并声明能力
 * - 接收任务并执行
 * - 发送心跳和状态更新
 * - 返回执行结果
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export interface WorkerCapabilities {
  workerId: string;
  workerType: string;
  capabilities: string[];
  maxComplexity: number;
  maxConcurrent: number;
  supportsStreaming: boolean;
  supportsCancel: boolean;
  defaultTimeoutMs: number;
}

export interface TaskDescription {
  taskId: string;
  prompt: string;
  workingDir?: string;
  context: Record<string, string>;
  timeoutMs: number;
  priority: number;
  expectedFormat: 'structured' | 'markdown' | 'diff' | 'text' | 'code';
}

export interface TaskResult {
  taskId: string;
  output: string;
  success: boolean;
  error?: string;
  tokensUsed?: number;
}

export interface WorkerStatus {
  workerId: string;
  health: 'healthy' | 'busy' | 'offline';
  currentTask?: string;
  tasksCompleted: number;
  tasksFailed: number;
}

// ============================================================================
// Worker SDK Class
// ============================================================================

export class AcpWorker {
  private workerId: string;
  private workerType: string;
  private capabilities: string[];
  private onTask?: (task: TaskDescription) => Promise<TaskResult>;
  private heartbeatInterval?: ReturnType<typeof setInterval>;

  constructor(options: {
    workerId: string;
    workerType: 'codex' | 'claude_code' | 'custom';
    capabilities: string[];
  }) {
    this.workerId = options.workerId;
    this.workerType = options.workerType;
    this.capabilities = options.capabilities;
  }

  /**
   * 注册 Worker 到 ACP-UI
   */
  async register(): Promise<WorkerCapabilities> {
    const caps = await invoke<WorkerCapabilities>('swarm_register_worker', {
      workerType: this.workerType,
      workerId: this.workerId,
    });

    // 启动心跳
    this.startHeartbeat();

    return caps;
  }

  /**
   * 设置任务处理回调
   */
  setTaskHandler(handler: (task: TaskDescription) => Promise<TaskResult>): void {
    this.onTask = handler;
  }

  /**
   * 执行任务（主动调用）
   */
  async executeTask(task: TaskDescription): Promise<TaskResult> {
    if (!this.onTask) {
      throw new Error('Task handler not set');
    }

    const result = await this.onTask(task);

    // 上报结果
    await invoke('swarm_report_task_result', {
      workerId: this.workerId,
      taskId: task.taskId,
      success: result.success,
      output: result.output,
      error: result.error,
    });

    return result;
  }

  /**
   * 发送心跳
   */
  async sendHeartbeat(): Promise<void> {
    await invoke('swarm_worker_heartbeat', {
      workerId: this.workerId,
    });
  }

  /**
   * 更新状态
   */
  async updateStatus(status: Partial<WorkerStatus>): Promise<void> {
    await invoke('swarm_update_worker_status', {
      workerId: this.workerId,
      status,
    });
  }

  /**
   * 关闭 Worker
   */
  async shutdown(): Promise<void> {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }

    await invoke('swarm_shutdown_worker', {
      workerId: this.workerId,
    });
  }

  /**
   * 启动心跳定时器
   */
  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      this.sendHeartbeat().catch(console.error);
    }, 10000); // 10秒心跳
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * 创建并注册 Worker
 */
export async function createWorker(options: {
  workerId?: string;
  workerType: 'codex' | 'claude_code' | 'custom';
  capabilities: string[];
  taskHandler: (task: TaskDescription) => Promise<TaskResult>;
}): Promise<AcpWorker> {
  const workerId = options.workerId || `worker-${Date.now()}`;

  const worker = new AcpWorker({
    workerId,
    workerType: options.workerType,
    capabilities: options.capabilities,
  });

  worker.setTaskHandler(options.taskHandler);
  await worker.register();

  return worker;
}

/**
 * 检查是否在 ACP-UI 环境中运行
 */
export function isAcpEnvironment(): boolean {
  try {
    // 检查 Tauri API 是否可用
    return typeof invoke === 'function';
  } catch {
    return false;
  }
}

// ============================================================================
// Export
// ============================================================================

export default {
  AcpWorker,
  createWorker,
  isAcpEnvironment,
};