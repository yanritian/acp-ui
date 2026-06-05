// Hermes API Service - 连接真实 Hermes 状态数据
// 通过 Tauri WebSocket 和 events 获取实时数据

import { ref, computed } from 'vue'
import { isTauriHost } from './platform'

// Hermes Status Types
export interface HermesAgentStatus {
  agentType: string
  status: 'idle' | 'busy' | 'error'
  currentTask?: string
  thinkingCount: number
  toolCallCount: number
}

export interface HermesTaskStatus {
  taskId: string
  request: string
  workspace: string
  mode: string
  status: 'running' | 'completed' | 'failed'
  startedAt: number
  completedAt?: number
  filesGenerated: number
  summary?: string
}

export interface HermesMetrics {
  totalTasksCompleted: number
  totalTasksFailed: number
  averageExecutionTimeMs: number
  totalFilesGenerated: number
  totalThinkingChunks: number
  totalToolCalls: number
}

// Reactive state
const agentStatus = ref<HermesAgentStatus>({
  agentType: 'coder',
  status: 'idle',
  thinkingCount: 0,
  toolCallCount: 0,
})

const currentTask = ref<HermesTaskStatus | null>(null)
const metrics = ref<HermesMetrics>({
  totalTasksCompleted: 0,
  totalTasksFailed: 0,
  averageExecutionTimeMs: 0,
  totalFilesGenerated: 0,
  totalThinkingChunks: 0,
  totalToolCalls: 0,
})

const taskHistory = ref<HermesTaskStatus[]>([])
const isHermesConnected = ref(false)

// Event listeners (stored for cleanup)
type UnlistenFn = () => void
let unlisteners: UnlistenFn[] = []

// Tauri API references (loaded conditionally)
let tauriListen: ((event: string, handler: (event: unknown) => void) => Promise<UnlistenFn>) | null = null
let tauriInvoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null

/**
 * Initialize Hermes API connection
 */
export async function initHermesApi(): Promise<void> {
  // Only initialize in Tauri environment
  if (!isTauriHost()) {
    console.log('[HermesAPI] Not in Tauri environment, skipping initialization')
    return
  }

  try {
    // Dynamically import Tauri APIs
    const eventModule = await import('@tauri-apps/api/event')
    const coreModule = await import('@tauri-apps/api/core')
    tauriListen = eventModule.listen
    tauriInvoke = coreModule.invoke

    // Listen to Tauri events from Executive Agent
    const unlistenStarted = await tauriListen('task-started', (event) => {
      const payload = (event as any).payload as Record<string, unknown>
      currentTask.value = {
        taskId: payload.taskId as string,
        request: payload.request as string,
        workspace: payload.workspace as string,
        mode: payload.mode as string,
        status: 'running',
        startedAt: Date.now(),
        filesGenerated: 0,
      }
      agentStatus.value = {
        agentType: 'coder',
        status: 'busy',
        currentTask: payload.taskId as string,
        thinkingCount: 0,
        toolCallCount: 0,
      }
    })

    const unlistenCompleted = await tauriListen('task-completed', (event) => {
      const payload = (event as any).payload as Record<string, unknown>
      if (currentTask.value) {
        currentTask.value.status = 'completed'
        currentTask.value.completedAt = Date.now()
        currentTask.value.filesGenerated = (payload.files as string[])?.length || 0
        currentTask.value.summary = payload.summary as string

        // Update metrics
        metrics.value.totalTasksCompleted++
        metrics.value.totalFilesGenerated += currentTask.value.filesGenerated

        // Add to history
        taskHistory.value.unshift(currentTask.value)
        if (taskHistory.value.length > 50) {
          taskHistory.value.pop()
        }
      }
      agentStatus.value = {
        agentType: 'coder',
        status: 'idle',
        thinkingCount: agentStatus.value.thinkingCount,
        toolCallCount: agentStatus.value.toolCallCount,
      }
    })

    const unlistenFailed = await tauriListen('task-failed', (event) => {
      const payload = (event as any).payload as Record<string, unknown>
      if (currentTask.value) {
        currentTask.value.status = 'failed'
        currentTask.value.completedAt = Date.now()
      }
      metrics.value.totalTasksFailed++
      agentStatus.value.status = 'error'
    })

    const unlistenThinking = await tauriListen('thinking-chunk', (event) => {
      agentStatus.value.thinkingCount++
      metrics.value.totalThinkingChunks++
    })

    const unlistenToolCall = await tauriListen('tool-call', (event) => {
      agentStatus.value.toolCallCount++
      metrics.value.totalToolCalls++
    })

    const unlistenAgentStatus = await tauriListen('agent-status-update', (event) => {
      const payload = (event as any).payload as Record<string, unknown>
      agentStatus.value.agentType = payload.agentType as string
      agentStatus.value.status = payload.status as 'idle' | 'busy' | 'error'
    })

    unlisteners = [
      unlistenStarted,
      unlistenCompleted,
      unlistenFailed,
      unlistenThinking,
      unlistenToolCall,
      unlistenAgentStatus,
    ]

    isHermesConnected.value = true
    console.log('[HermesAPI] Initialized successfully')
  } catch (error) {
    console.error('Failed to initialize Hermes API:', error)
    isHermesConnected.value = false
  }
}

/**
 * Cleanup Hermes API listeners
 */
export function cleanupHermesApi(): void {
  unlisteners.forEach((unlisten) => unlisten())
  unlisteners = []
  isHermesConnected.value = false
}

/**
 * Get current Hermes status
 */
export function getHermesStatus() {
  return {
    agent: computed(() => agentStatus.value),
    task: computed(() => currentTask.value),
    metrics: computed(() => metrics.value),
    history: computed(() => taskHistory.value),
    connected: computed(() => isHermesConnected.value),
  }
}

/**
 * Fetch Hermes metrics from backend (if available)
 */
export async function fetchHermesMetrics(): Promise<HermesMetrics> {
  if (!tauriInvoke) {
    return metrics.value
  }
  try {
    // Try to invoke backend command for metrics
    const result = await tauriInvoke<HermesMetrics>('get_hermes_metrics')
    metrics.value = result
    return result
  } catch {
    // Backend command not available, return cached metrics
    return metrics.value
  }
}

/**
 * Get task history from database
 */
export async function fetchTaskHistory(limit: number = 20): Promise<HermesTaskStatus[]> {
  if (!tauriInvoke) {
    return taskHistory.value
  }
  try {
    const result = await tauriInvoke<HermesTaskStatus[]>('get_task_history', { limit })
    taskHistory.value = result
    return result
  } catch {
    return taskHistory.value
  }
}

/**
 * Composable for Hermes Dashboard
 */
export function useHermesApi() {
  const status = getHermesStatus()

  // Computed stats
  const systemLoad = computed(() => {
    if (status.agent.value.status === 'busy') return 80
    if (status.agent.value.status === 'error') return 0
    return 0
  })

  const progressPercent = computed(() => {
    if (!status.task.value) return 0
    if (status.task.value.status === 'completed') return 100
    if (status.task.value.status === 'failed') return 0
    // Estimate progress based on thinking/tool activity
    const activity = status.agent.value.thinkingCount + status.agent.value.toolCallCount
    return Math.min(90, activity * 5)
  })

  const runningTasksCount = computed(() => {
    return status.task.value?.status === 'running' ? 1 : 0
  })

  const completedTasksCount = computed(() => {
    return status.metrics.value.totalTasksCompleted
  })

  return {
    ...status,
    systemLoad,
    progressPercent,
    runningTasksCount,
    completedTasksCount,
    init: initHermesApi,
    cleanup: cleanupHermesApi,
    fetchMetrics: fetchHermesMetrics,
    fetchHistory: fetchTaskHistory,
  }
}