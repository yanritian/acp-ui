// Swarm API - TypeScript wrappers for Rust swarm commands
// Day 4 - Frontend integration layer

import { invokeOrProxy } from '@/lib/host'

// Types matching Rust definitions

export interface WorkerCapabilities {
  workerId: string
  workerType: string
  capabilities: string[]
  maxComplexity: number
  maxConcurrent: number
  supportsStreaming: boolean
  supportsCancel: boolean
  defaultTimeoutMs: number
}

export interface WorkerStatus {
  workerId: string
  workerType: string
  health: 'healthy' | 'busy' | 'starting' | 'stopping' | 'unhealthy' | 'offline'
  pid: number | null
  memoryBytes: number | null
  cpuPercent: number | null
  tasksCompleted: number
  tasksFailed: number
  currentTask: string | null
  startedAt: { timestampMs: number } | null
  lastHeartbeat: { timestampMs: number } | null
}

export interface TaskHandle {
  taskId: string
  workerId: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'timeout'
  startedAt: { timestampMs: number }
  output: string
  error: string | null
  pid: number | null
}

export interface TaskDescription {
  id: string
  prompt: string
  workingDir?: string
  context: Record<string, string>
  timeoutMs: number
  priority: number
  expectedFormat: 'structured' | 'markdown' | 'diff' | 'text' | 'code'
}

// TaskShard - A portion of a larger task for parallel execution
export interface TaskShard {
  id: string
  parentTask: string
  shardIndex: number
  totalShards: number
  payload: TaskPayload
  primaryWorker: string
  replicaWorker: string | null
  status: 'pending' | 'assigned_primary' | 'running_primary' | 'assigned_replica' | 'running_replica' | 'completed' | 'failed' | 'cancelled'
  timeoutMs: number
  maxRetries: number
  retryCount: number
  primaryResult: string | null
  replicaResult: string | null
  error: string | null
  createdAt: number
  startedAt: number | null
  completedAt: number | null
}

// TaskPayload - The execution payload for a shard
export interface TaskPayload {
  prompt: string
  context: Record<string, string>
  expectedOutput: string
}

// ShardGroup - Collection of shards for a task (M-1 fix: added shards field)
export interface ShardGroup {
  taskId: string
  originalPrompt: string
  shards: Record<string, TaskShard>  // HashMap<String, TaskShard> as object
  status: 'partitioning' | 'assigning' | 'executing' | 'aggregating' | 'completed' | 'failed' | 'cancelled'
  shardCount: number
  completedCount: number
  failedCount: number
  finalResult: string | null
  errors: string[]
  createdAt: number
  completedAt: number | null
}

export interface QueenLease {
  leaseId: string
  queenId: string
  grantedAt: { timestampMs: number }
  ttlSeconds: number
  expiresAt: { timestampMs: number }
  isValid: boolean
  renewalCount: number
  invalidReason: string | null
  currentComplexity: number | null
}

export interface TaskDecomposition {
  originalPrompt: string
  analysis: TaskAnalysisResult
  subtasks: SubTask[]
  workerAssignments: Record<string, string>
  useReplicas: boolean
  replicaCount: number
}

export interface TaskAnalysisResult {
  taskType: string
  complexityLevel: number
  isParallelizable: boolean
  recommendedShards: number
  estimatedTimePerShard: number
  requiredCapabilities: string[]
  dependencies: SubTaskDependency[]
}

export interface SubTask {
  id: string
  index: number
  prompt: string
  expectedOutput: string
  requiredCapabilities: string[]
  estimatedComplexity: number
  isCritical: boolean
  workingDir: string | null
  contextFiles: Record<string, string>
}

export interface SubTaskDependency {
  dependentId: string
  dependsOnId: string
  dependencyType: 'data_flow' | 'sync_point' | 'order_constraint'
}

// ============================================================
// Swarm Worker Commands
// ============================================================

export async function swarmRegisterWorker(
  workerType: 'codex' | 'claude_code',
  workerId: string
): Promise<WorkerCapabilities> {
  return invokeOrProxy('swarm_register_worker', { workerType, workerId })
}

export async function swarmListWorkers(): Promise<WorkerCapabilities[]> {
  return invokeOrProxy('swarm_list_workers')
}

export async function swarmSendTask(
  workerId: string,
  taskId: string,
  prompt: string,
  workingDir?: string,
  timeoutMs?: number
): Promise<TaskHandle> {
  return invokeOrProxy('swarm_send_task', {
    workerId,
    taskId,
    prompt,
    workingDir,
    timeoutMs
  })
}

export async function swarmGetWorkerStatus(workerId: string): Promise<WorkerStatus> {
  return invokeOrProxy('swarm_get_worker_status', { workerId })
}

export async function swarmHealthCheck(workerId: string): Promise<boolean> {
  return invokeOrProxy('swarm_health_check', { workerId })
}

export async function swarmCancelTask(workerId: string, taskId: string): Promise<void> {
  return invokeOrProxy('swarm_cancel_task', { workerId, taskId })
}

export async function swarmGetTaskOutput(workerId: string, taskId: string): Promise<string> {
  return invokeOrProxy('swarm_get_task_output', { workerId, taskId })
}

export async function swarmShutdownWorker(workerId: string): Promise<void> {
  return invokeOrProxy('swarm_shutdown_worker', { workerId })
}

// ============================================================
// Task Partitioner Commands (Day 2)
// ============================================================

export async function taskAnalyze(prompt: string): Promise<TaskAnalysisResult> {
  return invokeOrProxy('task_analyze', { prompt })
}

export async function taskDecompose(prompt: string): Promise<TaskDecomposition> {
  return invokeOrProxy('task_decompose', { prompt })
}

export async function taskCreateShards(
  decomposition: TaskDecomposition,
  parentTaskId: string
): Promise<ShardGroup> {
  return invokeOrProxy('task_create_shards', { decomposition, parentTaskId })
}

// ============================================================
// Queen Lease Commands (Day 3)
// ============================================================

export async function queenStartElection(): Promise<string> {
  return invokeOrProxy('queen_start_election')
}

export async function queenGetLease(): Promise<QueenLease | null> {
  return invokeOrProxy('queen_get_lease')
}

export async function queenGetCurrent(): Promise<string | null> {
  return invokeOrProxy('queen_get_current')
}

export async function queenCheckAndRenew(): Promise<string | null> {
  return invokeOrProxy('queen_check_and_renew')
}

export async function queenAdaptiveUpgrade(requiredComplexity: number): Promise<string | null> {
  return invokeOrProxy('queen_adaptive_upgrade', { requiredComplexity })
}

// ============================================================
// Helper Functions
// ============================================================

export function createTaskDescription(
  id: string,
  prompt: string,
  options?: Partial<TaskDescription>
): TaskDescription {
  return {
    id,
    prompt,
    workingDir: options?.workingDir,
    context: options?.context || {},
    timeoutMs: options?.timeoutMs || 60000,
    priority: options?.priority || 5,
    expectedFormat: options?.expectedFormat || 'text'
  }
}

export function formatWorkerType(type: string): string {
  const labels: Record<string, string> = {
    codex: 'Codex (Executor)',
    claude_code: 'Claude Code (Orchestrator)'
  }
  return labels[type] || type
}

export function formatHealthStatus(health: string): string {
  const labels: Record<string, string> = {
    healthy: 'Healthy',
    busy: 'Busy',
    starting: 'Starting',
    stopping: 'Stopping',
    unhealthy: 'Unhealthy',
    offline: 'Offline'
  }
  return labels[health] || health
}

export function formatShardStatus(status: string): string {
  const labels: Record<string, string> = {
    partitioning: 'Partitioning',
    assigning: 'Assigning Workers',
    executing: 'Executing',
    aggregating: 'Aggregating Results',
    completed: 'Completed',
    failed: 'Failed',
    cancelled: 'Cancelled'
  }
  return labels[status] || status
}

// ============================================================
// Swarm Orchestrator Class
// ============================================================

export class SwarmOrchestratorClient {
  private workers: Map<string, WorkerCapabilities> = new Map()
  private statuses: Map<string, WorkerStatus> = new Map()
  private tasks: Map<string, TaskHandle> = new Map()
  private queen: string | null = null

  async initialize(): Promise<void> {
    const registered = await swarmListWorkers()
    for (const w of registered) {
      this.workers.set(w.workerId, w)
    }
  }

  async registerWorker(type: 'codex' | 'claude_code', id: string): Promise<WorkerCapabilities> {
    const caps = await swarmRegisterWorker(type, id)
    this.workers.set(id, caps)
    return caps
  }

  async getHealthyWorkers(): Promise<string[]> {
    const healthy: string[] = []
    for (const [id] of Array.from(this.workers.entries())) {
      const status = await swarmHealthCheck(id)
      if (status) healthy.push(id)
    }
    return healthy
  }

  async executeTask(prompt: string, topology: 'star' | 'chain'): Promise<TaskHandle[]> {
    const handles: TaskHandle[] = []
    const healthyWorkers = await this.getHealthyWorkers()

    if (healthyWorkers.length === 0) {
      throw new Error('No healthy workers available')
    }

    // Use Queen (first Claude Code worker) or first available
    const queenCandidates = healthyWorkers.filter(id => {
      const w = this.workers.get(id)
      return w?.workerType === 'claude_code'
    })

    const queenId = queenCandidates[0] || healthyWorkers[0]
    const taskId = `task-${Date.now()}`

    // Send to Queen for decomposition
    const queenHandle = await swarmSendTask(
      queenId,
      `${taskId}-decompose`,
      `Analyze and decompose: ${prompt}`
    )
    handles.push(queenHandle)

    // Distribute to workers (simplified)
    if (topology === 'star' && healthyWorkers.length > 1) {
      const workers = healthyWorkers.filter(id => id !== queenId)
      for (let i = 0; i < Math.min(workers.length, 3); i++) {
        const subHandle = await swarmSendTask(
          workers[i],
          `${taskId}-sub-${i}`,
          `Execute subtask ${i + 1} of: ${prompt}`
        )
        handles.push(subHandle)
      }
    }

    return handles
  }

  async refreshStatuses(): Promise<void> {
    for (const [id] of Array.from(this.workers.entries())) {
      try {
        const status = await swarmGetWorkerStatus(id)
        this.statuses.set(id, status)
      } catch {
        // Worker might be offline
      }
    }
  }

  getWorkerCapabilities(id: string): WorkerCapabilities | undefined {
    return this.workers.get(id)
  }

  getWorkerStatus(id: string): WorkerStatus | undefined {
    return this.statuses.get(id)
  }

  getTask(taskId: string): TaskHandle | undefined {
    return this.tasks.get(taskId)
  }

  listWorkers(): WorkerCapabilities[] {
    return Array.from(this.workers.values())
  }
}