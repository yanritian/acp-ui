import type { AgentConfig, ChatMessage, PermissionRequest, ToolCallInfo } from '../types'
import type { BudgetLimits, BudgetConsumption } from './budget-tracker'
import type { CompactionSummary, RehydrationArtifacts } from './context-compactor'

export type RuntimeTransportKind = 'stdio' | 'websocket' | 'http'
export type RuntimeConnectionStatus = 'idle' | 'connecting' | 'connected' | 'busy' | 'paused' | 'error' | 'disconnected'
export type RuntimeTaskStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'stopped_budget_exceeded'

export interface RuntimeAgentConfig {
  name: string
  config: AgentConfig
  cwd: string
}

export interface RuntimeSession {
  id: string
  agentName: string
  acpSessionId: string
  cwd: string
  title: string
  supportsLoadSession: boolean
  status: RuntimeConnectionStatus
  createdAt: number
  lastUpdated: number
}

export interface RuntimeTask {
  id: string
  title: string
  prompt: string
  source: 'single-session' | 'multi-session' | 'multi-agent' | 'workflow' | 'bot' | 'remote'
  status: RuntimeTaskStatus
  targetSessionIds: string[]
  createdAt: number
  startedAt?: number
  completedAt?: number
  error?: string
}

export interface RuntimeOutput {
  taskId: string
  sessionId: string
  agentName: string
  content: string
  thought: string
  messages: ChatMessage[]
  toolCalls: ToolCallInfo[]
  status: RuntimeTaskStatus
  error?: string
}

export interface RuntimeEvent {
  id: string
  taskId?: string
  sessionId?: string
  type:
    | 'session-created'
    | 'session-loaded'
    | 'session-closed'
    | 'task-started'
    | 'task-output'
    | 'task-completed'
    | 'task-failed'
    | 'permission-requested'
    | 'transport-closed'
  message: string
  timestamp: number
  payload?: unknown
}

export interface RuntimePromptOptions {
  taskId: string
  prompt: string
  source: RuntimeTask['source']
  memories?: string[]
  budgetLimits?: Partial<BudgetLimits>
  objective?: string // For compaction summary
}

export interface RuntimeBudgetState {
  limits: BudgetLimits
  consumption: BudgetConsumption
  exceeded: boolean
  exceededReason?: string
}

export interface RuntimeCompactionState {
  lastCompactionTimestamp: number
  summary: CompactionSummary | null
  rehydrationArtifacts: RehydrationArtifacts | null
  pendingCompaction: boolean
}

export interface RuntimePermissionBridge {
  getPendingPermission(): PermissionRequest | null
  resolvePermission(optionId: string): void
  cancelPermission(): void
}
