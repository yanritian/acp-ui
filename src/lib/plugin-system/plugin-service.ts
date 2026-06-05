import { invokeOrProxy } from '@/lib/host'

// Re-export types from backend
export interface PluginMeta {
  id: string
  kind: PluginKind
  name: string
  version: string
  description: string
  author?: string
  capabilities: Capability[]
  enabled: boolean
  registered_at: string
  last_health_check?: string
  health_status: HealthStatus
  config: Record<string, unknown>
}

export type PluginKind = 'skill' | 'mcp' | 'hook' | 'cli' | 'adapter'
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown'

export interface Capability {
  name: string
  description: string
  input_schema: Record<string, unknown>
  output_schema: Record<string, unknown>
  requires_permission: boolean
}

export interface PluginStats {
  total_invocations: number
  success_count: number
  failure_count: number
  average_duration_ms: number
  total_input_tokens: number
  total_output_tokens: number
  success_rate: number
}

export interface PluginExecutionRecord {
  id: string
  plugin_id: string
  capability: string
  input: Record<string, unknown>
  response: PluginResponse
  timestamp: string
}

export interface PluginResponse {
  plugin_id: string
  success: boolean
  result?: Record<string, unknown>
  error?: string
  duration_ms: number
  token_usage?: { input_tokens: number; output_tokens: number; cached_tokens: number }
}

export class PluginService {
  static async list(kind?: PluginKind): Promise<PluginMeta[]> {
    return invokeOrProxy('plugin_list', { kind })
  }

  static async register(meta: PluginMeta): Promise<void> {
    return invokeOrProxy('plugin_register', { meta })
  }

  static async unregister(id: string): Promise<PluginMeta> {
    return invokeOrProxy('plugin_unregister', { id })
  }

  static async get(id: string): Promise<PluginMeta> {
    return invokeOrProxy('plugin_get', { id })
  }

  static async setEnabled(id: string, enabled: boolean): Promise<void> {
    return invokeOrProxy('plugin_set_enabled', { id, enabled })
  }

  static async updateConfig(id: string, config: Record<string, unknown>): Promise<void> {
    return invokeOrProxy('plugin_update_config', { id, config })
  }

  static async search(query: string): Promise<PluginMeta[]> {
    return invokeOrProxy('plugin_search', { query })
  }

  static async getStats(id: string): Promise<PluginStats> {
    return invokeOrProxy('plugin_get_stats', { id })
  }

  static async getHistory(id: string, limit?: number): Promise<PluginExecutionRecord[]> {
    return invokeOrProxy('plugin_get_history', { id, limit: limit ?? 50 })
  }
}
