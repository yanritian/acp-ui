// History Store - localStorage-based task history persistence (Web) / SQLite (Tauri)

import { loadKvStore, type KVStore } from '../host/storage'

// Minimal types extracted from the old multi-agent-types (the rest is dead code)
export interface TaskDefinition {
  id: string
  description: string
  assigneeAgentId: string
  input: string
  dependsOn?: string[]
  timeoutMs?: number
  priority?: number
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
  createdAt: number
  startedAt?: number
  completedAt?: number
}

export interface TaskResult {
  taskId: string
  agentId: string
  output: string
  status: 'success' | 'failed' | 'cancelled'
  toolCalls: Array<{ toolCallId: string; title: string; kind: string; status: string }>
  durationMs: number
  error?: string
}

export interface TaskRecord {
  id: string
  name: string
  status: 'pending' | 'running' | 'success' | 'failed' | 'cancelled'
  createdAt: number
  completedAt?: number
  source: 'app' | 'feishu' | 'telegram' | 'discord' | 'web'
  agents: AgentRecord[]
  conversations: ConversationRecord[]
  outputs: OutputRecord[]
  syncRecords: SyncRecord[]
  error?: ErrorRecord
}

export interface AgentRecord {
  agentId: string
  agentName: string
  role: string
  status: string
  startTime: number
  endTime?: number
  outputFiles: string[]
  tokenUsage?: number
}

export interface ConversationRecord {
  timestamp: number
  speaker: 'user' | string
  message: string
  toolCalls?: ToolCallRecord[]
}

export interface ToolCallRecord {
  toolCallId: string
  title: string
  kind: string
  status: string
}

export interface OutputRecord {
  type: 'file' | 'message'
  path?: string
  content?: string
  agentId: string
}

export interface SyncRecord {
  channel: string
  status: 'success' | 'failed'
  syncedAt: number
  messageId?: string
}

export interface ErrorRecord {
  message: string
  stack?: string
  timestamp: number
}

export interface HistoryFilter {
  status?: TaskRecord['status'][]
  source?: TaskRecord['source'][]
  from?: number
  to?: number
  keyword?: string
  limit?: number
  offset?: number
}

export interface TaskStatistics {
  totalTasks: number
  successRate: number
  averageDurationMs: number
  mostUsedAgents: { name: string; count: number }[]
  tasksBySource: { source: string; count: number }[]
}

const HISTORY_STORE_PATH = 'task-history.json'

export class HistoryStore {
  private records: TaskRecord[] = []
  private maxRecords: number = 1000
  private store: KVStore | null = null
  private initialized = false

  async init(): Promise<void> {
    if (this.initialized) return
    this.store = await loadKvStore(HISTORY_STORE_PATH)
    const saved = await this.store.get<TaskRecord[]>('records')
    if (saved) {
      this.records = saved
    }
    this.initialized = true
  }

  private async persist(): Promise<void> {
    if (this.store) {
      await this.store.set('records', this.records)
      await this.store.save()
    }
  }

  /**
   * Save task record
   */
  async saveTask(record: TaskRecord): Promise<void> {
    if (!this.initialized) await this.init()

    // Remove existing record with same ID if exists
    const existingIdx = this.records.findIndex(r => r.id === record.id)
    if (existingIdx >= 0) {
      this.records[existingIdx] = record
    } else {
      this.records.push(record)
    }

    // Keep records limited
    if (this.records.length > this.maxRecords) {
      this.records.shift()
    }

    await this.persist()
  }

  /**
   * Query tasks with filter
   */
  async queryTasks(filter: HistoryFilter): Promise<TaskRecord[]> {
    if (!this.initialized) await this.init()

    let results = [...this.records]

    // Filter by status
    if (filter.status && filter.status.length > 0) {
      results = results.filter(r => filter.status!.includes(r.status))
    }

    // Filter by source
    if (filter.source && filter.source.length > 0) {
      results = results.filter(r => filter.source!.includes(r.source))
    }

    // Filter by time range
    if (filter.from) {
      results = results.filter(r => r.createdAt >= filter.from!)
    }
    if (filter.to) {
      results = results.filter(r => r.createdAt <= filter.to!)
    }

    // Filter by keyword
    if (filter.keyword) {
      const keyword = filter.keyword.toLowerCase()
      results = results.filter(r =>
        r.name.toLowerCase().includes(keyword) ||
        r.conversations.some(c => c.message.toLowerCase().includes(keyword))
      )
    }

    // Sort by creation time (newest first)
    results.sort((a, b) => b.createdAt - a.createdAt)

    // Apply limit and offset
    if (filter.offset) {
      results = results.slice(filter.offset)
    }
    if (filter.limit) {
      results = results.slice(0, filter.limit)
    }

    return results
  }

  /**
   * Get single task detail
   */
  async getTaskDetail(taskId: string): Promise<TaskRecord | undefined> {
    if (!this.initialized) await this.init()
    return this.records.find(r => r.id === taskId)
  }

  /**
   * Search tasks by keyword
   */
  async search(keyword: string): Promise<TaskRecord[]> {
    return this.queryTasks({ keyword, limit: 20 })
  }

  /**
   * Get statistics
   */
  async getStatistics(): Promise<TaskStatistics> {
    if (!this.initialized) await this.init()

    const total = this.records.length
    const success = this.records.filter(r => r.status === 'success').length
    const successRate = total > 0 ? (success / total) * 100 : 0

    const durations = this.records
      .filter(r => r.completedAt)
      .map(r => r.completedAt! - r.createdAt)
    const averageDurationMs = durations.length > 0
      ? durations.reduce((a, b) => a + b, 0) / durations.length
      : 0

    // Count agent usage
    const agentCounts: Map<string, number> = new Map()
    for (const record of this.records) {
      for (const agent of record.agents) {
        const count = agentCounts.get(agent.agentName) ?? 0
        agentCounts.set(agent.agentName, count + 1)
      }
    }
    const mostUsedAgents = Array.from(agentCounts.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5)

    // Count by source
    const sourceCounts: Map<string, number> = new Map()
    for (const record of this.records) {
      const count = sourceCounts.get(record.source) ?? 0
      sourceCounts.set(record.source, count + 1)
    }
    const tasksBySource = Array.from(sourceCounts.entries())
      .map(([source, count]) => ({ source, count }))
      .sort((a, b) => b.count - a.count)

    return {
      totalTasks: total,
      successRate,
      averageDurationMs,
      mostUsedAgents,
      tasksBySource,
    }
  }

  /**
   * Export history
   */
  async export(format: 'json' | 'csv' | 'markdown'): Promise<string> {
    if (!this.initialized) await this.init()

    if (format === 'json') {
      return JSON.stringify(this.records, null, 2)
    }

    if (format === 'csv') {
      const headers = ['id', 'name', 'status', 'createdAt', 'completedAt', 'source']
      const rows = this.records.map(r =>
        [r.id, `"${r.name}"`, r.status, r.createdAt, r.completedAt ?? '', r.source].join(',')
      )
      return [headers.join(','), ...rows].join('\n')
    }

    if (format === 'markdown') {
      const lines: string[] = ['# Task History\n']
      for (const record of this.records) {
        lines.push(`## ${record.name}`)
        lines.push(`- **Status**: ${record.status}`)
        lines.push(`- **Source**: ${record.source}`)
        lines.push(`- **Created**: ${new Date(record.createdAt).toISOString()}`)
        if (record.completedAt) {
          lines.push(`- **Completed**: ${new Date(record.completedAt).toISOString()}`)
        }
        lines.push('')
      }
      return lines.join('\n')
    }

    return ''
  }

  /**
   * Delete task record
   */
  async deleteTask(taskId: string): Promise<void> {
    if (!this.initialized) await this.init()

    const index = this.records.findIndex(r => r.id === taskId)
    if (index !== -1) {
      this.records.splice(index, 1)
      await this.persist()
    }
  }

  /**
   * Clear all records
   */
  async clear(): Promise<void> {
    this.records = []
    await this.persist()
  }

  /**
   * Get record count
   */
  getRecordCount(): number {
    return this.records.length
  }

  /**
   * Create task record from task result
   */
  createRecordFromResult(
    task: TaskDefinition,
    result: TaskResult,
    source: TaskRecord['source']
  ): TaskRecord {
    return {
      id: task.id,
      name: task.description,
      status: result.status === 'success' ? 'success' : 'failed',
      createdAt: task.createdAt,
      completedAt: result.status !== 'cancelled' ? Date.now() : undefined,
      source,
      agents: [{
        agentId: result.agentId,
        agentName: '', // Would need to lookup
        role: task.assigneeAgentId,
        status: result.status,
        startTime: task.startedAt ?? task.createdAt,
        endTime: Date.now(),
        outputFiles: [],
      }],
      conversations: [],
      outputs: [{
        type: 'message',
        content: result.output,
        agentId: result.agentId,
      }],
      syncRecords: [],
      error: result.error ? {
        message: result.error,
        timestamp: Date.now(),
      } : undefined,
    }
  }
}

// Singleton instance
let historyStoreInstance: HistoryStore | null = null

export function getHistoryStore(): HistoryStore {
  if (!historyStoreInstance) {
    historyStoreInstance = new HistoryStore()
  }
  return historyStoreInstance
}

export function createHistoryStore(): HistoryStore {
  return new HistoryStore()
}