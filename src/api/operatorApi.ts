// Hermes Game Operator API
// Frontend API layer for communicating with the Operator backend

// Tauri internals type declaration
declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>
    }
  }
}

import type {
  StartTaskRequest,
  StartTaskResponse,
  ApproveRequest,
  RedirectRequest,
  OperatorTask,
  OperatorEvent,
  ApprovalRequest,
  TaskSummary,
} from '@/types/operator'

// ============================================================================
// Tauri Invoke Helper
// ============================================================================

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    if (window.__TAURI_INTERNALS__) {
      return await window.__TAURI_INTERNALS__.invoke(command, args) as T
    }
    throw new Error('Tauri internals not available')
  } catch (error) {
    console.error(`Operator API error [${command}]:`, error)
    throw error
  }
}

// ============================================================================
// Task Management
// ============================================================================

export const OperatorApi = {
  // Start a new operator task
  async startTask(request: StartTaskRequest): Promise<StartTaskResponse> {
    return invoke<StartTaskResponse>('operator_start_task', { request })
  },

  // Get task details
  async getTask(taskId: string): Promise<OperatorTask> {
    return invoke<OperatorTask>('operator_get_task', { taskId })
  },

  // List all tasks
  async listTasks(): Promise<OperatorTask[]> {
    return invoke<OperatorTask[]>('operator_list_tasks')
  },

  // Pause a running task
  async pauseTask(taskId: string, expectedRevision?: number): Promise<void> {
    return invoke<void>('operator_pause_task', { taskId, expectedRevision })
  },

  // Resume a paused task
  async resumeTask(taskId: string, expectedRevision?: number): Promise<void> {
    return invoke<void>('operator_resume_task', { taskId, expectedRevision })
  },

  // Stop a running task
  async stopTask(taskId: string, expectedRevision?: number): Promise<void> {
    return invoke<void>('operator_stop_task', { taskId, expectedRevision })
  },

  // Redirect task to new goal
  async redirectTask(request: RedirectRequest): Promise<void> {
    return invoke<void>('operator_redirect_task', { request })
  },

  // ============================================================================
  // Approval
  // ============================================================================

  // Approve or reject an action
  async approve(request: ApproveRequest): Promise<void> {
    return invoke<void>('operator_approve', { request })
  },

  // Get pending approvals for a task
  async getPendingApprovals(taskId: string): Promise<ApprovalRequest[]> {
    return invoke<ApprovalRequest[]>('operator_get_pending_approvals', { taskId })
  },

  // ============================================================================
  // Events
  // ============================================================================

  // List events for a task (sequence ascending, supports pagination)
  async listEvents(taskId: string, limit?: number, afterSequence?: number): Promise<OperatorEvent[]> {
    return invoke<OperatorEvent[]>('operator_list_events', { taskId, limit, afterSequence })
  },

  // ============================================================================
  // Summary
  // ============================================================================

  // Get task summary
  async getTaskSummary(taskId: string): Promise<TaskSummary> {
    return invoke<TaskSummary>('operator_get_task_summary', { taskId })
  },

  // ============================================================================
  // File Tools
  // ============================================================================

  // Read a file safely
  async fileRead(taskId: string, path: string): Promise<any> {
    return invoke<any>('operator_file_read', { taskId, path })
  },

  // Legacy compatibility entry point. The backend rejects direct writes;
  // project changes must use the structured patch approval flow.
  async filePatch(
    taskId: string,
    path: string,
    newContent: string,
    createBackup: boolean = true
  ): Promise<any> {
    return invoke<any>('operator_file_patch', { taskId, path, newContent, createBackup })
  },

  // Preview a patch without applying
  async filePatchPreview(
    taskId: string,
    path: string,
    newContent: string
  ): Promise<any> {
    return invoke<any>('operator_file_patch_preview', { taskId, path, newContent })
  },

  // List files in a directory
  async fileList(taskId: string, path: string): Promise<any> {
    return invoke<any>('operator_file_list', { taskId, path })
  },
}

// ============================================================================
// Godot-Specific API
// ============================================================================

export const GodotOperatorApi = {
  // Analyze a Godot project
  async analyzeProject(projectPath: string): Promise<any> {
    return invoke<any>('godot_analyze_project', { projectPath })
  },

  // Detect if a directory is a Godot project
  async detectProject(path: string): Promise<boolean> {
    return invoke<boolean>('godot_detect_project', { path })
  },
}

// ============================================================================
// Hermes CLI Connection API
// ============================================================================

export interface HermesConnectionStatus {
  available: boolean
  version?: string
  path?: string
  error?: string
}

export interface HermesAnalysisResult {
  projectName: string
  godotVersion: string
  scripts: string[]
  scenes: string[]
  playerControllers: string[]
  analysisTimeMs: number
}

export interface HermesPlan {
  taskId: string
  goal: string
  steps: HermesPlanStep[]
  totalEstimatedTimeSeconds: number
}

export interface HermesPlanStep {
  id: number
  description: string
  files: string[]
  estimatedTimeSeconds: number
  requiresApproval: boolean
}

export const HermesCliApi = {
  // Check if Hermes CLI is available
  async checkConnection(): Promise<HermesConnectionStatus> {
    try {
      return await invoke<HermesConnectionStatus>('hermes_check_connection')
    } catch (error) {
      return {
        available: false,
        error: String(error),
      }
    }
  },

  // Analyze project using Hermes
  async analyzeProject(taskId: string): Promise<HermesAnalysisResult> {
    return invoke<HermesAnalysisResult>('hermes_analyze_project', { taskId })
  },

  // Generate execution plan
  async generatePlan(taskId: string, goal: string): Promise<HermesPlan> {
    return invoke<HermesPlan>('hermes_generate_plan', { taskId, goal })
  },
}
