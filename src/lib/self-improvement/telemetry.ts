// Telemetry: Collect runtime errors, performance metrics, and user behavior patterns
// Data is stored locally (localStorage) and used by the evolution engine for adaptation.

const TELEMETRY_KEY = 'acp-ui:telemetry'
const MAX_EVENTS = 500

export interface TelemetryEvent {
  type: 'error' | 'performance' | 'behavior' | 'adaptation'
  name: string
  data: Record<string, unknown>
  timestamp: number
}

export interface ErrorPattern {
  name: string
  count: number
  lastSeen: number
  context: Record<string, unknown>
}

export interface PerformanceMetric {
  name: string
  p50: number
  p95: number
  p99: number
  count: number
}

export interface BehaviorPattern {
  name: string
  frequency: number
  lastUsed: number
  context: Record<string, unknown>
}

let cache: TelemetryEvent[] = []
let loaded = false

function ensureLoaded(): void {
  if (loaded) return
  loaded = true
  if (typeof localStorage === 'undefined') return
  const raw = localStorage.getItem(TELEMETRY_KEY)
  if (raw) {
    try { cache = JSON.parse(raw) } catch { cache = [] }
  }
}

function persist(): void {
  if (typeof localStorage === 'undefined') return
  // Keep only recent events to avoid localStorage overflow
  const trimmed = cache.slice(-MAX_EVENTS)
  localStorage.setItem(TELEMETRY_KEY, JSON.stringify(trimmed))
}

export function trackEvent(event: Omit<TelemetryEvent, 'timestamp'>): void {
  ensureLoaded()
  cache.push({ ...event, timestamp: Date.now() })
  persist()
}

export function trackError(error: Error, context: Record<string, unknown> = {}): void {
  trackEvent({
    type: 'error',
    name: error.name,
    data: { message: error.message, stack: error.stack, ...context },
  })
}

export function trackPerformance(name: string, durationMs: number, context: Record<string, unknown> = {}): void {
  trackEvent({
    type: 'performance',
    name,
    data: { durationMs, ...context },
  })
}

export function trackBehavior(name: string, context: Record<string, unknown> = {}): void {
  trackEvent({
    type: 'behavior',
    name,
    data: context,
  })
}

/** Get all events of a given type, optionally filtered by name */
export function getEvents(type?: TelemetryEvent['type'], name?: string): TelemetryEvent[] {
  ensureLoaded()
  return cache.filter(e => {
    if (type && e.type !== type) return false
    if (name && e.name !== name) return false
    return true
  })
}

/** Get error patterns: which errors occur most frequently */
export function getErrorPatterns(): ErrorPattern[] {
  ensureLoaded()
  const errors = cache.filter(e => e.type === 'error')
  const map = new Map<string, ErrorPattern>()

  for (const e of errors) {
    const existing = map.get(e.name)
    if (existing) {
      existing.count++
      existing.lastSeen = e.timestamp
    } else {
      map.set(e.name, { name: e.name, count: 1, lastSeen: e.timestamp, context: e.data as Record<string, unknown> })
    }
  }

  return Array.from(map.values()).sort((a, b) => b.count - a.count)
}

/** Get performance metrics with percentiles */
export function getPerformanceMetrics(): PerformanceMetric[] {
  ensureLoaded()
  const perf = cache.filter(e => e.type === 'performance')
  const byName = new Map<string, number[]>()

  for (const e of perf) {
    const arr = byName.get(e.name) ?? []
    const ms = (e.data as Record<string, unknown>).durationMs as number
    if (typeof ms === 'number') arr.push(ms)
    byName.set(e.name, arr)
  }

  function percentile(sorted: number[], p: number): number {
    if (sorted.length === 0) return 0
    const idx = Math.ceil((p / 100) * sorted.length) - 1
    return sorted[Math.max(0, idx)]
  }

  return Array.from(byName.entries()).map(([name, values]) => {
    const sorted = [...values].sort((a, b) => a - b)
    return {
      name,
      p50: percentile(sorted, 50),
      p95: percentile(sorted, 95),
      p99: percentile(sorted, 99),
      count: sorted.length,
    }
  })
}

/** Get behavior patterns: which actions are most common */
export function getBehaviorPatterns(): BehaviorPattern[] {
  ensureLoaded()
  const behaviors = cache.filter(e => e.type === 'behavior')
  const map = new Map<string, BehaviorPattern>()

  for (const e of behaviors) {
    const existing = map.get(e.name)
    if (existing) {
      existing.frequency++
      existing.lastUsed = e.timestamp
    } else {
      map.set(e.name, { name: e.name, frequency: 1, lastUsed: e.timestamp, context: e.data as Record<string, unknown> })
    }
  }

  return Array.from(map.values()).sort((a, b) => b.frequency - a.frequency)
}

/** Clear all telemetry data */
export function clearTelemetry(): void {
  cache = []
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(TELEMETRY_KEY)
  }
}
