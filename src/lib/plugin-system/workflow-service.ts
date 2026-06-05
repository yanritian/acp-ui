import { invokeOrProxy } from '@/lib/host'

export type WorkflowStatus = 'draft' | 'validating' | 'ready' | 'running' | 'paused' | 'completed' | 'failed' | 'cancelled'
export type StageStrategy = 'parallel' | 'sequential' | 'map_reduce' | 'competitive' | 'adversarial_review'
export type FailurePolicy = 'stop_all' | 'continue_others' | 'retry_with_backoff' | 'fallback_to_manual'

export interface WorkflowDefinition {
  id: string
  name: string
  description: string
  stages: WorkflowStage[]
  max_concurrent_agents: number
  timeout_ms: number
  on_failure: FailurePolicy
  created_at: string
  status: WorkflowStatus
  total_tokens_budget?: number
}

export interface WorkflowStage {
  id: string
  name: string
  description: string
  strategy: StageStrategy
  agents: StageAgent[]
  depends_on: string[]
  sync_points: StageSyncPoint[]
  status: WorkflowStatus
  results: StageResult[]
  started_at?: string
  completed_at?: string
}

export interface StageAgent {
  agent_id: string
  role: string
  prompt_template: string
  max_tokens?: number
}

export interface StageSyncPoint {
  wait_for: string[]
  timeout_ms: number
  on_timeout: string
}

export interface StageResult {
  agent_id: string
  success: boolean
  output: Record<string, unknown>
  tokens_used: number
  duration_ms: number
  review?: ReviewResult
}

export interface ReviewResult {
  reviewer_id: string
  approved: boolean
  issues: string[]
  score: number
}

export interface WorkflowProgress {
  workflow_id: string
  total_stages: number
  completed_stages: number
  running_stages: number
  failed_stages: number
  total_agents: number
  active_agents: number
  tokens_used: number
  tokens_budget?: number
  elapsed_ms: number
  estimated_remaining_ms?: number
}

export class WorkflowService {
  static async create(name: string, description: string, stages: WorkflowStage[], maxConcurrent?: number): Promise<WorkflowDefinition> {
    return invokeOrProxy('workflow_create', { name, description, stages, maxConcurrent })
  }

  static async validate(workflowId: string): Promise<boolean> {
    return invokeOrProxy('workflow_validate', { workflowId })
  }

  static async list(): Promise<WorkflowDefinition[]> {
    return invokeOrProxy('workflow_list')
  }

  static async get(id: string): Promise<WorkflowDefinition> {
    return invokeOrProxy('workflow_get', { id })
  }

  static async getProgress(id: string): Promise<WorkflowProgress> {
    return invokeOrProxy('workflow_get_progress', { id })
  }

  static async updateStatus(id: string, status: WorkflowStatus): Promise<void> {
    return invokeOrProxy('workflow_update_status', { id, status })
  }

  static async submitResult(workflowId: string, stageId: string, result: StageResult): Promise<void> {
    return invokeOrProxy('workflow_submit_result', { workflowId, stageId, result })
  }

  static async generateFromTask(taskDescription: string, availableAgents: string[]): Promise<WorkflowDefinition> {
    return invokeOrProxy('workflow_generate_from_task', { taskDescription, availableAgents })
  }

  static async cancel(id: string): Promise<void> {
    return invokeOrProxy('workflow_cancel', { id })
  }

  static async save(id: string): Promise<Record<string, unknown>> {
    return invokeOrProxy('workflow_save', { id })
  }
}
