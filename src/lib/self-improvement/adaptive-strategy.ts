// Adaptive Strategy: Adjust runtime behavior based on historical patterns
// The system learns from past failures and successes to optimize future operations.

import { getErrorPatterns, getPerformanceMetrics, getBehaviorPatterns, trackEvent } from './telemetry'

const STRATEGY_KEY = 'acp-ui:strategies'

export interface RetryStrategy {
  maxRetries: number
  baseDelayMs: number
  backoffMultiplier: number
  applicableErrors: string[]
}

export interface CacheStrategy {
  enabled: boolean
  ttlMs: number
  keys: string[]
}

export interface FallbackStrategy {
  enabled: boolean
  fallbackAction: string
  triggerErrors: string[]
}

export interface AgentPreference {
  agentName: string
  successRate: number
  avgResponseTimeMs: number
  usageCount: number
}

export interface AdaptiveConfig {
  retry: RetryStrategy
  cache: CacheStrategy
  fallback: FallbackStrategy
  preferredAgents: AgentPreference[]
  lastUpdated: number
}

function defaultConfig(): AdaptiveConfig {
  return {
    retry: {
      maxRetries: 3,
      baseDelayMs: 1000,
      backoffMultiplier: 2,
      applicableErrors: ['transport-closed', 'task-failed', 'session-not-found'],
    },
    cache: {
      enabled: true,
      ttlMs: 5 * 60 * 1000, // 5 minutes
      keys: ['agent-config', 'available-models', 'available-modes'],
    },
    fallback: {
      enabled: true,
      fallbackAction: 'switch-to-next-agent',
      triggerErrors: ['agent-not-found', 'transport-closed'],
    },
    preferredAgents: [],
    lastUpdated: Date.now(),
  }
}

let config: AdaptiveConfig | null = null

function loadConfig(): AdaptiveConfig {
  if (config) return config
  if (typeof localStorage === 'undefined') {
    config = defaultConfig()
    return config
  }
  const raw = localStorage.getItem(STRATEGY_KEY)
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as AdaptiveConfig
      if (parsed && parsed.retry && parsed.cache && parsed.fallback) {
        config = parsed
        return config
      }
    } catch {
      // Corrupted data, reset
    }
  }
  config = defaultConfig()
  return config
}

function saveConfig(): void {
  if (!config) return
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(STRATEGY_KEY, JSON.stringify(config))
}

/** Analyze historical data and adapt strategies */
export function adaptStrategies(): void {
  const cfg = loadConfig()
  const errorPatterns = getErrorPatterns()
  const perfMetrics = getPerformanceMetrics()
  const behaviorPatterns = getBehaviorPatterns()

  // 1. Adapt retry strategy based on error frequency
  const frequentErrors = errorPatterns.filter(p => p.count >= 3)
  if (frequentErrors.length > 0) {
    const newApplicable = new Set(cfg.retry.applicableErrors)
    for (const p of frequentErrors) {
      newApplicable.add(p.name)
    }
    cfg.retry.applicableErrors = Array.from(newApplicable)

    // If errors are very frequent, increase retry count
    if (errorPatterns.some(p => p.count >= 10)) {
      cfg.retry.maxRetries = Math.min(cfg.retry.maxRetries + 1, 5)
    }
  }

  // 2. Adapt cache TTL based on performance
  const configFetchPerf = perfMetrics.find(m => m.name === 'config-load')
  if (configFetchPerf && configFetchPerf.p95 > 500) {
    // Config loading is slow, increase cache TTL
    cfg.cache.ttlMs = Math.min(cfg.cache.ttlMs * 2, 30 * 60 * 1000) // max 30 min
  }

  // 3. Update agent preferences based on behavior patterns
  const agentUsage = behaviorPatterns.filter(p => p.name.startsWith('agent-'))
  cfg.preferredAgents = agentUsage.map(p => {
    // Try to extract success rate from context, fallback to default
    const contextSuccessRate = p.context?.successRate as number | undefined
    const contextAvgTime = p.context?.avgResponseTimeMs as number | undefined

    return {
      agentName: p.name.replace('agent-', ''),
      successRate: contextSuccessRate ?? 0.85, // Default 85% success rate for known agents
      avgResponseTimeMs: contextAvgTime ?? 5000, // Default 5s avg response time
      usageCount: p.frequency,
    }
  }).sort((a, b) => b.usageCount - a.usageCount)

  cfg.lastUpdated = Date.now()
  config = cfg
  saveConfig()

  trackEvent({
    type: 'adaptation',
    name: 'strategy-updated',
    data: {
      errorPatterns: errorPatterns.length,
      perfMetrics: perfMetrics.length,
      behaviorPatterns: behaviorPatterns.length,
    },
  })
}

export function getRetryStrategy(): RetryStrategy {
  return loadConfig().retry
}

export function getCacheStrategy(): CacheStrategy {
  return loadConfig().cache
}

export function getFallbackStrategy(): FallbackStrategy {
  return loadConfig().fallback
}

export function getPreferredAgents(): AgentPreference[] {
  return loadConfig().preferredAgents
}

/** Check if a given error should trigger a retry */
export function shouldRetry(errorName: string): boolean {
  const strategy = getRetryStrategy()
  return strategy.applicableErrors.includes(errorName)
}

/** Calculate delay for a given retry attempt (exponential backoff) */
export function getRetryDelay(attempt: number): number {
  const strategy = getRetryStrategy()
  return strategy.baseDelayMs * Math.pow(strategy.backoffMultiplier, attempt - 1)
}

/** Check if a value should be cached */
export function shouldCache(key: string): boolean {
  const strategy = getCacheStrategy()
  return strategy.enabled && strategy.keys.includes(key)
}

export function getCacheTtl(): number {
  return getCacheStrategy().ttlMs
}

/** Reset all strategies to defaults */
export function resetStrategies(): void {
  config = defaultConfig()
  saveConfig()
}
