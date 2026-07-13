// Game Operator Client
// Handles communication with the Game Operator backend

import axios, { AxiosInstance } from 'axios'

export interface OperatorTask {
  task_id: string
  /** Optimistic concurrency control revision */
  revision: number
  domain: string
  project_path: string
  goal: string
  status: string
  mode: string
  approval_policy: string
  created_at: string
  updated_at: string
  checkpoint_id?: string
  memory_snapshot_id?: string
}

export interface OperatorEvent {
  event_id: string
  task_id: string
  sequence: number
  task_revision: number
  timestamp: string
  type: string
  level: string
  title: string
  message?: string
  source: string
}

export interface ApprovalRequest {
  approval_id: string
  task_id: string
  task_revision: number
  level: string
  action: string
  title: string
  reason: string
  risk?: string
}

export class GameOperatorClient {
  private client: AxiosInstance | undefined
  private connected = false

  async connect(serverUrl: string, authToken?: string): Promise<void> {
    this.client = axios.create({
      baseURL: serverUrl,
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {}
    })

    // Test connection
    try {
      await this.client.get('/api/health')
      this.connected = true
    } catch (error: any) {
      this.connected = false
      throw new Error(`Cannot connect to server: ${error.message}`)
    }
  }

  async disconnect(): Promise<void> {
    this.client = undefined
    this.connected = false
  }

  isConnected(): boolean {
    return this.connected
  }

  async listTasks(): Promise<OperatorTask[]> {
    if (!this.client) throw new Error('Not connected')
    try {
      const response = await this.client.get('/api/operator/tasks')
      return response.data.tasks || response.data || []
    } catch (error: any) {
      if (error.response?.status === 401) {
        throw new Error('Authentication failed. Please check your token.')
      }
      if (error.response?.status === 403) {
        throw new Error('Access denied. You do not have permission to list tasks.')
      }
      throw new Error(`Failed to list tasks: ${error.message}`)
    }
  }

  async getTask(taskId: string): Promise<OperatorTask> {
    if (!this.client) throw new Error('Not connected')
    try {
      const response = await this.client.get(`/api/operator/tasks/${encodeURIComponent(taskId)}`)
      return response.data
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Task not found: ${taskId}`)
      }
      throw new Error(`Failed to get task: ${error.message}`)
    }
  }

  async startTask(goal: string, projectPath?: string): Promise<OperatorTask> {
    if (!this.client) throw new Error('Not connected')
    try {
      const response = await this.client.post('/api/operator/tasks', {
        domain: 'game.godot',
        project_path: projectPath || '',
        goal,
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      })
      return await this.getTask(response.data.task_id)
    } catch (error: any) {
      if (error.response?.status === 400) {
        throw new Error(`Invalid task request: ${error.response.data.message || 'Bad request'}`)
      }
      if (error.response?.status === 429) {
        throw new Error('Rate limit exceeded. Please try again later.')
      }
      throw new Error(`Failed to start task: ${error.message}`)
    }
  }

  async pauseTask(taskId: string, expectedRevision?: number): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    try {
      await this.client.post(`/api/operator/tasks/${encodeURIComponent(taskId)}/pause`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision })
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Task not found: ${taskId}`)
      }
      if (error.response?.status === 409) {
        throw new Error('Task cannot be paused in its current state')
      }
      throw new Error(`Failed to pause task: ${error.message}`)
    }
  }

  async resumeTask(taskId: string, expectedRevision?: number): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    try {
      await this.client.post(`/api/operator/tasks/${encodeURIComponent(taskId)}/resume`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision })
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Task not found: ${taskId}`)
      }
      if (error.response?.status === 409) {
        throw new Error('Task cannot be resumed in its current state')
      }
      throw new Error(`Failed to resume task: ${error.message}`)
    }
  }

  async stopTask(taskId: string, expectedRevision?: number): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    try {
      await this.client.post(`/api/operator/tasks/${encodeURIComponent(taskId)}/stop`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision })
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Task not found: ${taskId}`)
      }
      if (error.response?.status === 409) {
        throw new Error('Task cannot be stopped in its current state')
      }
      throw new Error(`Failed to stop task: ${error.message}`)
    }
  }

  async listEvents(taskId: string, afterSequence?: number): Promise<OperatorEvent[]> {
    if (!this.client) throw new Error('Not connected')
    try {
      const query = afterSequence === undefined ? '' : `?after_sequence=${encodeURIComponent(String(afterSequence))}`
      const response = await this.client.get(`/api/operator/tasks/${encodeURIComponent(taskId)}/events${query}`)
      return response.data.events || response.data || []
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Task not found: ${taskId}`)
      }
      throw new Error(`Failed to list events: ${error.message}`)
    }
  }

  async getPendingApprovals(taskId?: string): Promise<ApprovalRequest[]> {
    if (!this.client) throw new Error('Not connected')
    try {
      if (taskId) {
        const response = await this.client.get(`/api/operator/tasks/${encodeURIComponent(taskId)}/approvals`)
        return response.data.approvals || response.data || []
      }

      const tasks = await this.listTasks()
      const approvals = await Promise.all(
        tasks.map(async task => {
          const response = await this.client!.get(`/api/operator/tasks/${encodeURIComponent(task.task_id)}/approvals`)
          return response.data.approvals || response.data || []
        })
      )
      return approvals.flat()
    } catch (error: any) {
      if (error.response?.status === 404) {
        return []
      }
      throw new Error(`Failed to get approvals: ${error.message}`)
    }
  }

  async approve(
    taskId: string,
    approvalId: string,
    decision: 'approve' | 'reject' | 'request_changes',
    expectedRevision?: number
  ): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    try {
      await this.client.post('/api/operator/approvals/decision', {
        task_id: taskId,
        approval_id: approvalId,
        decision,
        ...(expectedRevision === undefined ? {} : { expected_revision: expectedRevision }),
      })
    } catch (error: any) {
      if (error.response?.status === 404) {
        throw new Error(`Approval not found: ${approvalId}`)
      }
      if (error.response?.status === 409) {
        throw new Error('Approval already processed')
      }
      throw new Error(`Failed to approve: ${error.message}`)
    }
  }
}
