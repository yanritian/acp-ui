// Game Operator Client
// Handles communication with the Game Operator backend

import axios, { AxiosInstance } from 'axios'

export interface OperatorTask {
  task_id: string
  domain: string
  project_path: string
  goal: string
  status: string
  mode: string
  approval_policy: string
  created_at: string
  updated_at: string
}

export interface OperatorEvent {
  event_id: string
  task_id: string
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
      await this.client.get('/health')
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
    const response = await this.client.get('/api/tasks')
    return response.data
  }

  async getTask(taskId: string): Promise<OperatorTask> {
    if (!this.client) throw new Error('Not connected')
    const response = await this.client.get(`/api/tasks/${taskId}`)
    return response.data
  }

  async startTask(goal: string, projectPath?: string): Promise<OperatorTask> {
    if (!this.client) throw new Error('Not connected')
    const response = await this.client.post('/api/tasks', {
      domain: 'game.godot',
      project_path: projectPath || '',
      goal,
      mode: 'propose_then_apply',
      approval_policy: 'safe_default'
    })
    return response.data
  }

  async pauseTask(taskId: string): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    await this.client.post(`/api/tasks/${taskId}/pause`)
  }

  async resumeTask(taskId: string): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    await this.client.post(`/api/tasks/${taskId}/resume`)
  }

  async stopTask(taskId: string): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    await this.client.post(`/api/tasks/${taskId}/stop`)
  }

  async listEvents(taskId: string): Promise<OperatorEvent[]> {
    if (!this.client) throw new Error('Not connected')
    const response = await this.client.get(`/api/tasks/${taskId}/events`)
    return response.data
  }

  async getPendingApprovals(taskId?: string): Promise<ApprovalRequest[]> {
    if (!this.client) throw new Error('Not connected')
    const url = taskId ? `/api/tasks/${taskId}/approvals` : '/api/approvals'
    const response = await this.client.get(url)
    return response.data
  }

  async approve(approvalId: string, decision: 'approve' | 'reject' | 'request_changes'): Promise<void> {
    if (!this.client) throw new Error('Not connected')
    await this.client.post(`/api/approvals/${approvalId}`, { decision })
  }
}