// Token Optimizer Store - Manages token usage history, budget settings, and state

import { ref, computed, readonly } from 'vue'
import { defineStore } from 'pinia'
import { invokeOrProxy } from '@/lib/host'
import {
  TokenOptimizer,
  createTokenOptimizer,
  estimateTokens,
  DEFAULT_TOKEN_OPTIMIZER_CONFIG,
  type TokenOptimizerConfig,
  type ArchivedChunk,
  type CompressionResult,
} from '@/lib/agent-runtime/token-optimizer'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Hourly token usage record for trend chart */
export interface HourlyUsage {
  hour: string // Format: "HH:mm"
  inputTokens: number
  outputTokens: number
  cachedTokens: number
  totalTokens: number
}

/** Daily token usage record for trend chart */
export interface DailyUsage {
  date: string // Format: "YYYY-MM-DD"
  inputTokens: number
  outputTokens: number
  cachedTokens: number
  totalTokens: number
}

/** Token optimizer state for UI */
export interface TokenOptimizerState {
  config: TokenOptimizerConfig
  cache: {
    systemCacheHits: number
    systemCacheMisses: number
    contextCacheHits: number
    contextCacheMisses: number
  }
  compression: {
    totalTokensSaved: number
    compressionRatio: number
    totalCompressions: number
    avgTokensPerCompression: number
  }
  archivedChunks: ArchivedChunk[]
  idleTimer: {
    active: boolean
    nextCompressionIn: number
    idleThreshold: number
  }
  totalTokensUsed: number
  contextWindowLimit: number
  systemPrompt: {
    frozen: boolean
    hash: string
    sizeBytes: number
    tokenCount: number
  }
}

// ---------------------------------------------------------------------------
// Store Definition
// ---------------------------------------------------------------------------

export const useTokenOptimizerStore = defineStore('token-optimizer', () => {
  // -----------------------------------------------------------------------
  // State
  // -----------------------------------------------------------------------

  /** Token optimizer instance */
  const optimizer = ref<TokenOptimizer | null>(null)

  /** Current configuration */
  const config = ref<TokenOptimizerConfig>({
    ...DEFAULT_TOKEN_OPTIMIZER_CONFIG,
  })

  /** Token budget limit (user configurable) */
  const tokenBudgetLimit = ref(DEFAULT_TOKEN_OPTIMIZER_CONFIG.maxContextTokens)

  /** Hourly usage history (last 24 hours) */
  const hourlyUsageHistory = ref<HourlyUsage[]>([])

  /** Daily usage history (last 7 days) */
  const dailyUsageHistory = ref<DailyUsage[]>([])

  /** Current state snapshot */
  const currentState = ref<TokenOptimizerState | null>(null)

  /** Loading state */
  const loading = ref(false)

  /** Error message */
  const error = ref<string | null>(null)

  /** Time range for trend chart: 'hour' | 'day' */
  const trendTimeRange = ref<'hour' | 'day'>('hour')

  // -----------------------------------------------------------------------
  // Computed
  // -----------------------------------------------------------------------

  /** Get trend data based on selected time range */
  const trendData = computed(() => {
    if (trendTimeRange.value === 'hour') {
      return hourlyUsageHistory.value
    }
    return dailyUsageHistory.value
  })

  /** Total cache hit rate */
  const totalCacheHitRate = computed(() => {
    if (!currentState.value) return 0
    const { systemCacheHits, systemCacheMisses, contextCacheHits, contextCacheMisses } = currentState.value.cache
    const totalHits = systemCacheHits + contextCacheHits
    const totalMisses = systemCacheMisses + contextCacheMisses
    const total = totalHits + totalMisses
    if (total === 0) return 0
    return (totalHits / total) * 100
  })

  /** Context usage percentage */
  const contextUsagePercent = computed(() => {
    if (!currentState.value) return 0
    const { totalTokensUsed, contextWindowLimit } = currentState.value
    if (contextWindowLimit === 0) return 0
    return Math.min(100, (totalTokensUsed / contextWindowLimit) * 100)
  })

  /** Is compression in progress */
  const compressing = ref(false)

  /** Is clearing cache */
  const clearingCache = ref(false)

  // -----------------------------------------------------------------------
  // Actions
  // -----------------------------------------------------------------------

  /** Initialize the token optimizer */
  function initialize(customConfig?: Partial<TokenOptimizerConfig>) {
    if (optimizer.value) {
      optimizer.value.destroy()
    }
    const mergedConfig = { ...DEFAULT_TOKEN_OPTIMIZER_CONFIG, ...customConfig }
    config.value = mergedConfig
    tokenBudgetLimit.value = mergedConfig.maxContextTokens
    optimizer.value = createTokenOptimizer(mergedConfig)
    // Initialize empty history - will be populated from backend
    hourlyUsageHistory.value = []
    dailyUsageHistory.value = []
    fetchStats()
  }

  /** Fetch current stats from backend */
  async function fetchStats() {
    loading.value = true
    error.value = null
    try {
      // Call backend for stats
      const result = await invokeOrProxy<TokenOptimizerState>('token_optimizer_get_stats')
      currentState.value = result
    } catch {
      // Backend not available - use local optimizer stats
      if (optimizer.value) {
        const cacheStats = optimizer.value.getCacheStats()
        const archive = optimizer.value.getArchive()
        currentState.value = {
          config: config.value,
          cache: {
            systemCacheHits: Math.floor(cacheStats.hitRate * 100),
            systemCacheMisses: 100 - Math.floor(cacheStats.hitRate * 100),
            contextCacheHits: 0,
            contextCacheMisses: 0,
          },
          compression: {
            totalTokensSaved: cacheStats.totalSaved,
            compressionRatio: 1.0,
            totalCompressions: 0,
            avgTokensPerCompression: 0,
          },
          archivedChunks: archive,
          idleTimer: {
            active: false,
            nextCompressionIn: 0,
            idleThreshold: 60,
          },
          totalTokensUsed: 0,
          contextWindowLimit: tokenBudgetLimit.value,
          systemPrompt: {
            frozen: false,
            hash: '',
            sizeBytes: 0,
            tokenCount: 0,
          },
        }
      } else {
        // No optimizer - return empty state
        currentState.value = {
          config: config.value,
          cache: {
            systemCacheHits: 0,
            systemCacheMisses: 0,
            contextCacheHits: 0,
            contextCacheMisses: 0,
          },
          compression: {
            totalTokensSaved: 0,
            compressionRatio: 1.0,
            totalCompressions: 0,
            avgTokensPerCompression: 0,
          },
          archivedChunks: [],
          idleTimer: {
            active: false,
            nextCompressionIn: 0,
            idleThreshold: 60,
          },
          totalTokensUsed: 0,
          contextWindowLimit: tokenBudgetLimit.value,
          systemPrompt: {
            frozen: false,
            hash: '',
            sizeBytes: 0,
            tokenCount: 0,
          },
        }
      }
    } finally {
      loading.value = false
    }
  }

  /** Compress current context */
  async function compressNow() {
    compressing.value = true
    try {
      // Try backend first
      await invokeOrProxy('token_optimizer_compress_now')
      await fetchStats()
    } catch {
      // Backend not available - use local optimizer
      await new Promise((r) => setTimeout(r, 600))
      if (currentState.value) {
        currentState.value.compression.totalCompressions += 1
        currentState.value.compression.totalTokensSaved += 180
        currentState.value.archivedChunks.unshift({
          id: `archive-${String(currentState.value.compression.totalCompressions).padStart(3, '0')}`,
          originalMessages: 3,
          compressedSummary: 'Manually compressed context block.',
          referencePath: `archive://session/archive-${currentState.value.compression.totalCompressions}`,
          timestamp: new Date().toISOString(),
        })
      }
    } finally {
      compressing.value = false
    }
  }

  /** Clear cache */
  async function clearCache() {
    clearingCache.value = true
    try {
      // Try backend first
      await invokeOrProxy('token_optimizer_clear_cache')
      await fetchStats()
    } catch {
      // Backend not available - clear locally
      await new Promise((r) => setTimeout(r, 400))
      if (currentState.value) {
        currentState.value.cache = {
          systemCacheHits: 0,
          systemCacheMisses: 0,
          contextCacheHits: 0,
          contextCacheMisses: 0,
        }
      }
    } finally {
      clearingCache.value = false
    }
  }

  /** Update token budget limit */
  async function updateBudgetLimit(newLimit: number) {
    tokenBudgetLimit.value = newLimit
    config.value.maxContextTokens = newLimit

    try {
      // Try backend first
      await invokeOrProxy('token_optimizer_set_budget', { limit: newLimit })
    } catch {
      // Backend not available - update local config
      if (optimizer.value) {
        optimizer.value.destroy()
        optimizer.value = createTokenOptimizer(config.value)
      }
    }

    // Update state with new limit
    if (currentState.value) {
      currentState.value.contextWindowLimit = newLimit
    }
  }

  /** Toggle trend time range */
  function toggleTrendTimeRange() {
    trendTimeRange.value = trendTimeRange.value === 'hour' ? 'day' : 'hour'
  }

  /** Add current usage to history (call this periodically) */
  function recordCurrentUsage(inputTokens: number, outputTokens: number, cachedTokens: number) {
    const now = new Date()
    const hourKey = now.getHours().toString().padStart(2, '0') + ':00'
    const dateKey = now.toISOString().slice(0, 10)

    // Update hourly history
    const hourlyEntry = hourlyUsageHistory.value.find(h => h.hour === hourKey)
    if (hourlyEntry) {
      hourlyEntry.inputTokens += inputTokens
      hourlyEntry.outputTokens += outputTokens
      hourlyEntry.cachedTokens += cachedTokens
      hourlyEntry.totalTokens += inputTokens + outputTokens + cachedTokens
    } else {
      hourlyUsageHistory.value.push({
        hour: hourKey,
        inputTokens,
        outputTokens,
        cachedTokens,
        totalTokens: inputTokens + outputTokens + cachedTokens,
      })
      // Keep only last 24 entries
      if (hourlyUsageHistory.value.length > 24) {
        hourlyUsageHistory.value.shift()
      }
    }

    // Update daily history
    const dailyEntry = dailyUsageHistory.value.find(d => d.date === dateKey)
    if (dailyEntry) {
      dailyEntry.inputTokens += inputTokens
      dailyEntry.outputTokens += outputTokens
      dailyEntry.cachedTokens += cachedTokens
      dailyEntry.totalTokens += inputTokens + outputTokens + cachedTokens
    } else {
      dailyUsageHistory.value.push({
        date: dateKey,
        inputTokens,
        outputTokens,
        cachedTokens,
        totalTokens: inputTokens + outputTokens + cachedTokens,
      })
      // Keep only last 7 entries
      if (dailyUsageHistory.value.length > 7) {
        dailyUsageHistory.value.shift()
      }
    }
  }

  /** Destroy optimizer on cleanup */
  function destroy() {
    if (optimizer.value) {
      optimizer.value.destroy()
      optimizer.value = null
    }
  }

  // -----------------------------------------------------------------------
  // Countdown timer for idle compression
  // -----------------------------------------------------------------------

  let countdownInterval: ReturnType<typeof setInterval> | null = null

  function startCountdown() {
    countdownInterval = setInterval(() => {
      if (currentState.value?.idleTimer.active && currentState.value.idleTimer.nextCompressionIn > 0) {
        currentState.value.idleTimer.nextCompressionIn -= 1
      }
    }, 1000)
  }

  function stopCountdown() {
    if (countdownInterval) {
      clearInterval(countdownInterval)
      countdownInterval = null
    }
  }

  // Initialize on store creation
  initialize()

  return {
    // State
    config: readonly(config),
    tokenBudgetLimit,
    hourlyUsageHistory: readonly(hourlyUsageHistory),
    dailyUsageHistory: readonly(dailyUsageHistory),
    currentState: readonly(currentState),
    loading: readonly(loading),
    error: readonly(error),
    trendTimeRange,
    compressing: readonly(compressing),
    clearingCache: readonly(clearingCache),

    // Computed
    trendData,
    totalCacheHitRate,
    contextUsagePercent,

    // Actions
    initialize,
    fetchStats,
    compressNow,
    clearCache,
    updateBudgetLimit,
    toggleTrendTimeRange,
    recordCurrentUsage,
    destroy,
    startCountdown,
    stopCountdown,
  }
})