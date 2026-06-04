// Telemetry - Cost tracking, cache telemetry, and tracing
// Implements MVP Blueprint checklist: cost telemetry, cache hit rate, tracing

import type { BudgetConsumption } from './budget-tracker'
import type { CompactionSummary } from './context-compactor'

/**
 * Token usage record
 */
export interface TokenUsageRecord {
  id: string
  timestamp: number
  sessionId: string
  taskId: string
  modelProvider: string
  modelId: string
  inputTokens: number
  outputTokens: number
  cachedTokens: number
  costUsd: number
  latencyMs: number
  cacheHit: boolean
  contextHash?: string
}

/**
 * Cache telemetry record
 */
export interface CacheTelemetryRecord {
  timestamp: number
  contextHash: string
  cacheHit: boolean
  cachedTokens: number
  freshTokens: number
  provider: string
}

/**
 * Trace event types
 */
export type TraceEventType =
  | 'session_created'
  | 'session_loaded'
  | 'task_started'
  | 'tool_call_proposed'
  | 'tool_call_validated'
  | 'tool_call_executed'
  | 'tool_call_denied'
  | 'permission_checked'
  | 'approval_requested'
  | 'approval_resolved'
  | 'context_compacted'
  | 'budget_checked'
  | 'budget_exceeded'
  | 'model_call'
  | 'model_response'
  | 'error_occurred'
  | 'retry_attempt'
  | 'task_completed'
  | 'task_failed'

/**
 * Trace event record
 */
export interface TraceEvent {
  id: string
  timestamp: number
  sessionId: string
  taskId?: string
  eventType: TraceEventType
  data: Record<string, unknown>
  durationMs?: number
  error?: string
  parentEventId?: string
}

/**
 * Telemetry statistics
 */
export interface TelemetryStats {
  totalRuns: number
  successfulRuns: number
  failedRuns: number
  totalInputTokens: number
  totalOutputTokens: number
  totalCachedTokens: number
  totalCostUsd: number
  avgLatencyMs: number
  cacheHitRate: number
  avgTokensPerRun: number
  avgCostPerRun: number
  mostUsedTools: { tool: string; count: number }[]
  errorRate: number
  budgetExceededRate: number
}

/**
 * Telemetry store class
 */
export class TelemetryStore {
  private tokenRecords: TokenUsageRecord[] = []
  private cacheRecords: CacheTelemetryRecord[] = []
  private traceEvents: TraceEvent[] = []
  private maxRecords: number = 1000
  private sessionStats: Map<string, SessionTelemetry> = new Map()

  constructor(maxRecords?: number) {
    if (maxRecords) this.maxRecords = maxRecords
  }

  /**
   * Record token usage
   */
  recordTokenUsage(record: Omit<TokenUsageRecord, 'id' | 'timestamp'>): void {
    const fullRecord: TokenUsageRecord = {
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      ...record,
    }

    this.tokenRecords.push(fullRecord)

    // Update session stats
    this.updateSessionStats(record.sessionId, {
      inputTokens: record.inputTokens,
      outputTokens: record.outputTokens,
      cachedTokens: record.cachedTokens,
      costUsd: record.costUsd,
      latencyMs: record.latencyMs,
    })

    // Limit records
    if (this.tokenRecords.length > this.maxRecords) {
      this.tokenRecords.shift()
    }
  }

  /**
   * Record cache telemetry
   */
  recordCacheTelemetry(record: Omit<CacheTelemetryRecord, 'timestamp'>): void {
    const fullRecord: CacheTelemetryRecord = {
      timestamp: Date.now(),
      ...record,
    }

    this.cacheRecords.push(fullRecord)

    if (this.cacheRecords.length > this.maxRecords) {
      this.cacheRecords.shift()
    }
  }

  /**
   * Record trace event
   */
  recordTrace(event: Omit<TraceEvent, 'id' | 'timestamp'>): void {
    const fullEvent: TraceEvent = {
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      ...event,
    }

    this.traceEvents.push(fullEvent)

    if (this.traceEvents.length > this.maxRecords) {
      this.traceEvents.shift()
    }
  }

  /**
   * Update session stats
   */
  private updateSessionStats(sessionId: string, usage: {
    inputTokens: number
    outputTokens: number
    cachedTokens: number
    costUsd: number
    latencyMs: number
  }): void {
    let stats = this.sessionStats.get(sessionId)
    if (!stats) {
      stats = {
        sessionId,
        startTime: Date.now(),
        totalInputTokens: 0,
        totalOutputTokens: 0,
        totalCachedTokens: 0,
        totalCostUsd: 0,
        totalLatencyMs: 0,
        callCount: 0,
        toolCalls: new Map(),
        errors: [],
        budgetExceeded: false,
      }
      this.sessionStats.set(sessionId, stats)
    }

    stats.totalInputTokens += usage.inputTokens
    stats.totalOutputTokens += usage.outputTokens
    stats.totalCachedTokens += usage.cachedTokens
    stats.totalCostUsd += usage.costUsd
    stats.totalLatencyMs += usage.latencyMs
    stats.callCount++
  }

  /**
   * Record tool call
   */
  recordToolCall(sessionId: string, toolName: string, status: 'success' | 'error' | 'denied'): void {
    const stats = this.sessionStats.get(sessionId)
    if (stats) {
      const count = stats.toolCalls.get(toolName) ?? 0
      stats.toolCalls.set(toolName, count + 1)
    }

    this.recordTrace({
      sessionId,
      eventType: status === 'success' ? 'tool_call_executed' : status === 'denied' ? 'tool_call_denied' : 'error_occurred',
      data: { toolName, status },
    })
  }

  /**
   * Record error
   */
  recordError(sessionId: string, error: string, taskId?: string): void {
    const stats = this.sessionStats.get(sessionId)
    if (stats) {
      stats.errors.push({ error, timestamp: Date.now() })
    }

    this.recordTrace({
      sessionId,
      taskId,
      eventType: 'error_occurred',
      data: { error },
      error,
    })
  }

  /**
   * Record budget exceeded
   */
  recordBudgetExceeded(sessionId: string, limitName: string, current: number, limit: number): void {
    const stats = this.sessionStats.get(sessionId)
    if (stats) {
      stats.budgetExceeded = true
    }

    this.recordTrace({
      sessionId,
      eventType: 'budget_exceeded',
      data: { limitName, current, limit },
    })
  }

  /**
   * Record compaction
   */
  recordCompaction(sessionId: string, summary: CompactionSummary): void {
    this.recordTrace({
      sessionId,
      eventType: 'context_compacted',
      data: {
        originalMessages: summary.originalMessageCount,
        retainedMessages: summary.retainedMessageCount,
        objective: summary.currentObjective,
      },
    })
  }

  /**
   * Get session telemetry
   */
  getSessionTelemetry(sessionId: string): SessionTelemetry | undefined {
    return this.sessionStats.get(sessionId)
  }

  /**
   * Get overall statistics
   */
  getStats(): TelemetryStats {
    const sessions = Array.from(this.sessionStats.values())

    const totalRuns = sessions.length
    const successfulRuns = sessions.filter(s => !s.budgetExceeded && s.errors.length === 0).length
    const failedRuns = totalRuns - successfulRuns

    const totalInputTokens = sessions.reduce((sum, s) => sum + s.totalInputTokens, 0)
    const totalOutputTokens = sessions.reduce((sum, s) => sum + s.totalOutputTokens, 0)
    const totalCachedTokens = sessions.reduce((sum, s) => sum + s.totalCachedTokens, 0)
    const totalCostUsd = sessions.reduce((sum, s) => sum + s.totalCostUsd, 0)

    const avgLatencyMs = sessions.length > 0
      ? sessions.reduce((sum, s) => sum + s.totalLatencyMs, 0) / sessions.length
      : 0

    // Calculate cache hit rate
    const cacheHits = this.cacheRecords.filter(r => r.cacheHit).length
    const cacheHitRate = this.cacheRecords.length > 0
      ? cacheHits / this.cacheRecords.length * 100
      : 0

    // Aggregate tool usage
    const toolCounts: Map<string, number> = new Map()
    for (const session of sessions) {
      for (const [tool, count] of session.toolCalls) {
        toolCounts.set(tool, (toolCounts.get(tool) ?? 0) + count)
      }
    }
    const mostUsedTools = Array.from(toolCounts.entries())
      .map(([tool, count]) => ({ tool, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10)

    const avgTokensPerRun = totalRuns > 0
      ? (totalInputTokens + totalOutputTokens) / totalRuns
      : 0

    const avgCostPerRun = totalRuns > 0
      ? totalCostUsd / totalRuns
      : 0

    const errorRate = totalRuns > 0
      ? sessions.filter(s => s.errors.length > 0).length / totalRuns * 100
      : 0

    const budgetExceededRate = totalRuns > 0
      ? sessions.filter(s => s.budgetExceeded).length / totalRuns * 100
      : 0

    return {
      totalRuns,
      successfulRuns,
      failedRuns,
      totalInputTokens,
      totalOutputTokens,
      totalCachedTokens,
      totalCostUsd,
      avgLatencyMs,
      cacheHitRate,
      avgTokensPerRun,
      avgCostPerRun,
      mostUsedTools,
      errorRate,
      budgetExceededRate,
    }
  }

  /**
   * Get recent trace events
   */
  getRecentTraces(limit: number = 50): TraceEvent[] {
    return this.traceEvents.slice(-limit)
  }

  /**
   * Get trace events for session
   */
  getSessionTraces(sessionId: string): TraceEvent[] {
    return this.traceEvents.filter(e => e.sessionId === sessionId)
  }

  /**
   * Get token records for session
   */
  getSessionTokenRecords(sessionId: string): TokenUsageRecord[] {
    return this.tokenRecords.filter(r => r.sessionId === sessionId)
  }

  /**
   * Export telemetry data
   */
  export(format: 'json' | 'csv' = 'json'): string {
    if (format === 'json') {
      return JSON.stringify({
        stats: this.getStats(),
        recentTraces: this.getRecentTraces(100),
        tokenRecords: this.tokenRecords.slice(-100),
        cacheRecords: this.cacheRecords.slice(-100),
      }, null, 2)
    }

    // CSV format for token records
    const headers = ['timestamp', 'sessionId', 'taskId', 'modelProvider', 'inputTokens', 'outputTokens', 'costUsd', 'cacheHit']
    const rows = this.tokenRecords.slice(-100).map(r =>
      [r.timestamp, r.sessionId, r.taskId, r.modelProvider, r.inputTokens, r.outputTokens, r.costUsd, r.cacheHit].join(',')
    )
    return [headers.join(','), ...rows].join('\n')
  }

  /**
   * Clear all records
   */
  clear(): void {
    this.tokenRecords = []
    this.cacheRecords = []
    this.traceEvents = []
    this.sessionStats.clear()
  }

  /**
   * Clear session telemetry
   */
  clearSession(sessionId: string): void {
    this.sessionStats.delete(sessionId)
    this.tokenRecords = this.tokenRecords.filter(r => r.sessionId !== sessionId)
    this.traceEvents = this.traceEvents.filter(e => e.sessionId !== sessionId)
  }
}

/**
 * Session telemetry
 */
export interface SessionTelemetry {
  sessionId: string
  startTime: number
  totalInputTokens: number
  totalOutputTokens: number
  totalCachedTokens: number
  totalCostUsd: number
  totalLatencyMs: number
  callCount: number
  toolCalls: Map<string, number>
  errors: Array<{ error: string; timestamp: number }>
  budgetExceeded: boolean
}

/**
 * Model pricing configuration
 */
export const MODEL_PRICING: Record<string, { inputPer1k: number; outputPer1k: number; cachedPer1k?: number }> = {
  'claude-opus-4-7': { inputPer1k: 0.015, outputPer1k: 0.075, cachedPer1k: 0.0015 },
  'claude-sonnet-4-6': { inputPer1k: 0.003, outputPer1k: 0.015, cachedPer1k: 0.0003 },
  'claude-haiku-4-5': { inputPer1k: 0.001, outputPer1k: 0.005, cachedPer1k: 0.0001 },
  'gpt-4o': { inputPer1k: 0.005, outputPer1k: 0.015 },
  'gpt-4o-mini': { inputPer1k: 0.00015, outputPer1k: 0.0006 },
  'gpt-4-turbo': { inputPer1k: 0.01, outputPer1k: 0.03 },
  'gpt-3.5-turbo': { inputPer1k: 0.0005, outputPer1k: 0.0015 },
}

/**
 * Calculate cost from token usage
 */
export function calculateCost(
  modelId: string,
  inputTokens: number,
  outputTokens: number,
  cachedTokens: number = 0
): number {
  const pricing = MODEL_PRICING[modelId]
  if (!pricing) {
    // Default pricing if model not found
    return (inputTokens * 0.003 + outputTokens * 0.015) / 1000
  }

  const inputCost = (inputTokens - cachedTokens) * pricing.inputPer1k / 1000
  const outputCost = outputTokens * pricing.outputPer1k / 1000
  const cachedCost = cachedTokens * (pricing.cachedPer1k ?? 0) / 1000

  return inputCost + outputCost + cachedCost
}

// Singleton instance
let telemetryInstance: TelemetryStore | null = null

export function getTelemetryStore(): TelemetryStore {
  if (!telemetryInstance) {
    telemetryInstance = new TelemetryStore()
  }
  return telemetryInstance
}

export function createTelemetryStore(maxRecords?: number): TelemetryStore {
  return new TelemetryStore(maxRecords)
}