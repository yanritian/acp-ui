// Self-Healing: Automatic recovery from failures
// Detects failure patterns and applies appropriate recovery actions.

import { trackEvent, trackError } from './telemetry'
import { shouldRetry, getRetryDelay, getFallbackStrategy } from './adaptive-strategy'

export interface HealingAction {
  type: 'retry' | 'fallback' | 'reset' | 'degrade'
  description: string
  success: boolean
  durationMs: number
}

export interface HealingReport {
  timestamp: number
  errorName: string
  context: Record<string, unknown>
  actions: HealingAction[]
  resolved: boolean
}

const HEALING_KEY = 'acp-ui:healing'
const MAX_REPORTS = 100

let reports: HealingReport[] = []
let loaded = false

function ensureLoaded(): void {
  if (loaded) return
  loaded = true
  if (typeof localStorage === 'undefined') return
  const raw = localStorage.getItem(HEALING_KEY)
  if (raw) {
    try { reports = JSON.parse(raw) } catch { reports = [] }
  }
}

function persist(): void {
  if (typeof localStorage === 'undefined') return
  const trimmed = reports.slice(-MAX_REPORTS)
  localStorage.setItem(HEALING_KEY, JSON.stringify(trimmed))
}

/**
 * Attempt to heal a failed operation.
 * Returns true if the issue was resolved, false otherwise.
 */
export async function attemptHealing(
  errorName: string,
  context: Record<string, unknown>,
  retryFn?: () => Promise<unknown>,
): Promise<boolean> {
  const startTime = Date.now()
  const actions: HealingAction[] = []

  // Strategy 1: Retry with exponential backoff
  if (shouldRetry(errorName) && retryFn) {
    const maxRetries = 3
    let resolved = false

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      const delay = getRetryDelay(attempt)
      actions.push({ type: 'retry', description: `Retry attempt ${attempt}/${maxRetries} (delay: ${delay}ms)`, success: false, durationMs: delay })

      await new Promise(r => setTimeout(r, delay))

      try {
        await retryFn()
        actions[actions.length - 1].success = true
        resolved = true
        break
      } catch (e) {
        trackError(e instanceof Error ? e : new Error(String(e)), { ...context, attempt })
      }
    }

    if (resolved) {
      const report: HealingReport = {
        timestamp: Date.now(),
        errorName,
        context,
        actions,
        resolved: true,
      }
      ensureLoaded()
      reports.push(report)
      persist()

      trackEvent({
        type: 'behavior',
        name: 'self-heal-success',
        data: { errorName, attempts: actions.length, durationMs: Date.now() - startTime },
      })
      return true
    }
  }

  // Strategy 2: Fallback action
  const fallback = getFallbackStrategy()
  if (fallback.enabled && fallback.triggerErrors.includes(errorName)) {
    actions.push({ type: 'fallback', description: `Fallback: ${fallback.fallbackAction}`, success: true, durationMs: Date.now() - startTime })

    // Emit fallback event for UI to handle
    trackEvent({
      type: 'behavior',
      name: 'self-heal-fallback',
      data: { errorName, fallbackAction: fallback.fallbackAction },
    })

    // Record successful fallback
    const report: HealingReport = {
      timestamp: Date.now(),
      errorName,
      context,
      actions,
      resolved: true,
    }
    ensureLoaded()
    reports.push(report)
    persist()
    return true
  }

  // Record the healing attempt (failed or partial)
  const report: HealingReport = {
    timestamp: Date.now(),
    errorName,
    context,
    actions,
    resolved: false,
  }
  ensureLoaded()
  reports.push(report)
  persist()

  trackEvent({
    type: 'behavior',
    name: 'self-heal-failed',
    data: { errorName, actions: actions.length },
  })
  return false
}

/** Get recent healing reports */
export function getHealingReports(): HealingReport[] {
  ensureLoaded()
  return [...reports].reverse()
}

/** Get healing success rate */
export function getHealingStats(): { total: number; resolved: number; rate: number } {
  ensureLoaded()
  const total = reports.length
  const resolved = reports.filter(r => r.resolved).length
  return { total, resolved, rate: total > 0 ? resolved / total : 0 }
}

/** Clear healing history */
export function clearHealingHistory(): void {
  reports = []
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(HEALING_KEY)
  }
}
