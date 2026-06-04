import { invoke } from '@tauri-apps/api/core'
import type { HistoryFilter, TaskRecord, TaskStatistics } from './history-store'

export interface BackendTaskRecord {
  id: string
  name: string
  status: 'pending' | 'running' | 'success' | 'failed' | 'cancelled'
  source: string
  created_at: string
  completed_at: string | null
  error_message: string | null
  agents: BackendAgentExecution[]
}

export interface BackendAgentExecution {
  agent_id: string
  agent_name: string
  status: string
  started_at: string
  completed_at: string | null
  output: string | null
}

export interface BackendTaskStatistics {
  total_tasks: number
  successful_tasks: number
  failed_tasks: number
  running_tasks: number
  pending_tasks: number
  success_rate: number
  average_duration_ms: number
}

function parseRfc3339ToTimestamp(s: string | null): number {
  if (!s) return 0
  return new Date(s).getTime()
}

function toFrontendRecord(r: BackendTaskRecord): TaskRecord {
  return {
    id: r.id,
    name: r.name,
    status: r.status,
    createdAt: parseRfc3339ToTimestamp(r.created_at),
    completedAt: r.completed_at ? parseRfc3339ToTimestamp(r.completed_at) : undefined,
    source: r.source as TaskRecord['source'],
    agents: r.agents.map(a => ({
      agentId: a.agent_id,
      agentName: a.agent_name,
      role: a.agent_id,
      status: a.status,
      startTime: parseRfc3339ToTimestamp(a.started_at),
      endTime: a.completed_at ? parseRfc3339ToTimestamp(a.completed_at) : undefined,
      outputFiles: [],
    })),
    conversations: [],
    outputs: [],
    syncRecords: [],
    error: r.error_message ? {
      message: r.error_message,
      timestamp: parseRfc3339ToTimestamp(r.created_at),
    } : undefined,
  }
}

function toFrontendStatistics(s: BackendTaskStatistics): TaskStatistics {
  return {
    totalTasks: s.total_tasks,
    successRate: s.success_rate,
    averageDurationMs: s.average_duration_ms,
    mostUsedAgents: [],
    tasksBySource: [
      { source: 'app', count: s.total_tasks },
    ],
  }
}

export class TauriHistoryStore {
  async saveTask(record: TaskRecord): Promise<void> {
    // Convert frontend TaskRecord to backend format
    const backendRecord: BackendTaskRecord = {
      id: record.id,
      name: record.name,
      status: record.status,
      source: record.source,
      created_at: new Date(record.createdAt).toISOString(),
      completed_at: record.completedAt ? new Date(record.completedAt).toISOString() : null,
      error_message: record.error?.message ?? null,
      agents: record.agents.map(a => ({
        agent_id: a.agentId,
        agent_name: a.agentName,
        status: a.status,
        started_at: new Date(a.startTime).toISOString(),
        completed_at: a.endTime ? new Date(a.endTime).toISOString() : null,
        output: null,
      })),
    }
    await invoke('save_task_history', { task: backendRecord })
  }

  async queryTasks(filter: HistoryFilter): Promise<TaskRecord[]> {
    const records = await invoke<BackendTaskRecord[]>('get_task_history', {
      status: filter.status,
      source: filter.source?.[0],
      limit: filter.limit,
    })
    return records.map(toFrontendRecord)
  }

  async getTaskDetail(taskId: string): Promise<TaskRecord | undefined> {
    const record = await invoke<BackendTaskRecord | null>('get_task_detail', { taskId })
    return record ? toFrontendRecord(record) : undefined
  }

  async search(keyword: string): Promise<TaskRecord[]> {
    const records = await invoke<BackendTaskRecord[]>('search_tasks', {
      keyword,
      limit: 20,
    })
    return records.map(toFrontendRecord)
  }

  async getStatistics(): Promise<TaskStatistics> {
    const stats = await invoke<BackendTaskStatistics>('get_task_statistics')
    return toFrontendStatistics(stats)
  }

  async export(format: 'json' | 'csv' | 'markdown'): Promise<string> {
    const records = await invoke<BackendTaskRecord[]>('get_task_history', { limit: 1000 })
    const frontendRecords = records.map(toFrontendRecord)

    if (format === 'json') {
      return JSON.stringify(frontendRecords, null, 2)
    }

    if (format === 'csv') {
      const headers = ['id', 'name', 'status', 'createdAt', 'completedAt', 'source']
      const rows = frontendRecords.map(r =>
        [r.id, `"${r.name}"`, r.status, r.createdAt, r.completedAt ?? '', r.source].join(',')
      )
      return [headers.join(','), ...rows].join('\n')
    }

    if (format === 'markdown') {
      const lines: string[] = ['# Task History\n']
      for (const record of frontendRecords) {
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

  async deleteTask(taskId: string): Promise<void> {
    await invoke('delete_task_history', { taskId })
  }

  async clear(): Promise<void> {
    // Not supported via SQLite without additional commands
  }

  getRecordCount(): number {
    return 0
  }

  createRecordFromResult(): TaskRecord {
    throw new Error('Not implemented for TauriHistoryStore')
  }
}

let tauriHistoryStoreInstance: TauriHistoryStore | null = null

export function getTauriHistoryStore(): TauriHistoryStore {
  if (!tauriHistoryStoreInstance) {
    tauriHistoryStoreInstance = new TauriHistoryStore()
  }
  return tauriHistoryStoreInstance
}
