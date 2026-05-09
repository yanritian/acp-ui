// Evolution Engine: Periodically analyzes telemetry data and generates improvement suggestions
// This is the "brain" of the self-improvement system.

import { getErrorPatterns, getPerformanceMetrics, getBehaviorPatterns, trackEvent } from './telemetry'
import { adaptStrategies } from './adaptive-strategy'
import { getHealingStats } from './self-healing'

export interface ImprovementSuggestion {
  id: string
  category: 'error-reduction' | 'performance' | 'ux' | 'stability' | 'pattern-discovery'
  priority: 'low' | 'medium' | 'high' | 'critical'
  title: string
  description: string
  evidence: Record<string, unknown>
  autoApplied: boolean
  timestamp: number
}

const SUGGESTIONS_KEY = 'acp-ui:suggestions'
const ANALYSIS_INTERVAL_MS = 5 * 60 * 1000 // 5 minutes

let suggestions: ImprovementSuggestion[] = []
let loaded = false
let analysisTimer: ReturnType<typeof setInterval> | null = null

function ensureLoaded(): void {
  if (loaded) return
  loaded = true
  if (typeof localStorage === 'undefined') return
  const raw = localStorage.getItem(SUGGESTIONS_KEY)
  if (raw) {
    try { suggestions = JSON.parse(raw) } catch { suggestions = [] }
  }
}

function persist(): void {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(SUGGESTIONS_KEY, JSON.stringify(suggestions))
}

/** Analyze current state and generate improvement suggestions */
function analyze(): ImprovementSuggestion[] {
  const newSuggestions: ImprovementSuggestion[] = []
  const errorPatterns = getErrorPatterns()
  const perfMetrics = getPerformanceMetrics()
  const behaviorPatterns = getBehaviorPatterns()
  const healingStats = getHealingStats()

  // 1. Critical: High-frequency errors
  const criticalErrors = errorPatterns.filter(p => p.count >= 20)
  for (const p of criticalErrors) {
    newSuggestions.push({
      id: `error-${p.name}`,
      category: 'stability',
      priority: 'critical',
      title: `高频错误: ${p.name}`,
      description: `错误 "${p.name}" 已发生 ${p.count} 次，建议检查相关模块或添加更完善的错误处理。`,
      evidence: { count: p.count, lastSeen: p.lastSeen, context: p.context },
      autoApplied: false,
      timestamp: Date.now(),
    })
  }

  // 2. High: Performance degradation
  const slowOps = perfMetrics.filter(m => m.p95 > 2000)
  for (const m of slowOps) {
    newSuggestions.push({
      id: `perf-${m.name}`,
      category: 'performance',
      priority: 'high',
      title: `性能瓶颈: ${m.name}`,
      description: `操作 "${m.name}" P95 耗时 ${m.p95}ms，建议优化或添加缓存。`,
      evidence: { p50: m.p50, p95: m.p95, p99: m.p99, count: m.count },
      autoApplied: false,
      timestamp: Date.now(),
    })
  }

  // 3. Medium: Low healing success rate
  if (healingStats.total > 5 && healingStats.rate < 0.3) {
    newSuggestions.push({
      id: 'healing-low-rate',
      category: 'stability',
      priority: 'medium',
      title: '自愈成功率过低',
      description: `自愈系统成功率为 ${(healingStats.rate * 100).toFixed(1)}%，建议调整重试策略或添加更多降级方案。`,
      evidence: { total: healingStats.total, resolved: healingStats.resolved, rate: healingStats.rate },
      autoApplied: false,
      timestamp: Date.now(),
    })
  }

  // 4. Low: Unused features (no behavior patterns)
  const allFeatures = ['chat', 'multi-agent', 'multi-session', 'workflow', 'orchestration', 'bot', 'gateway', 'memory', 'history', 'status', 'monitor', 'error', 'evolution', 'pattern']
  const usedFeatures = new Set(behaviorPatterns.map(p => p.name.replace('feature-', '')))
  for (const feature of allFeatures) {
    if (!usedFeatures.has(feature)) {
      newSuggestions.push({
        id: `unused-${feature}`,
        category: 'ux',
        priority: 'low',
        title: `功能未使用: ${feature}`,
        description: `功能 "${feature}" 从未被使用，考虑是否需要在 UI 中突出显示或添加引导。`,
        evidence: { feature },
        autoApplied: false,
        timestamp: Date.now(),
      })
    }
  }

  // 5. Pattern Discovery: Analyze recurring error patterns
  const recurringErrors = errorPatterns.filter(p => p.count >= 3)
  if (recurringErrors.length >= 2) {
    const patternNames = recurringErrors.map(p => p.name).join(', ')
    newSuggestions.push({
      id: 'recurring-pattern',
      category: 'pattern-discovery',
      priority: 'medium',
      title: '发现重复错误模式',
      description: `检测到 ${recurringErrors.length} 个重复出现的错误模式: ${patternNames}。建议将这些记录到模式库以便快速参考。`,
      evidence: { patterns: recurringErrors.map(p => ({ name: p.name, count: p.count })) },
      autoApplied: false,
      timestamp: Date.now(),
    })
  }

  // 6. Performance Pattern: Consistent slow operations
  const consistentlySlow = perfMetrics.filter(m => m.p50 > 500)
  if (consistentlySlow.length >= 2) {
    newSuggestions.push({
      id: 'perf-pattern',
      category: 'pattern-discovery',
      priority: 'low',
      title: '发现性能问题模式',
      description: `${consistentlySlow.length} 个操作持续缓慢 (P50 > 500ms)。建议分析是否为系统性问题。`,
      evidence: { operations: consistentlySlow.map(m => ({ name: m.name, p50: m.p50 })) },
      autoApplied: false,
      timestamp: Date.now(),
    })
  }

  return newSuggestions
}

/** Start the evolution engine (periodic analysis) */
export function startEvolutionEngine(): void {
  if (analysisTimer) return

  // Initial analysis
  runAnalysis()

  // Periodic analysis
  analysisTimer = setInterval(runAnalysis, ANALYSIS_INTERVAL_MS)
}

function runAnalysis(): void {
  const newSuggestions = analyze()

  // Merge with existing suggestions (update if same ID, add if new)
  for (const newS of newSuggestions) {
    const existingIdx = suggestions.findIndex(s => s.id === newS.id)
    if (existingIdx >= 0) {
      // Update existing
      suggestions[existingIdx] = { ...suggestions[existingIdx], ...newS }
    } else {
      // Add new
      suggestions.push(newS)
    }
  }

  // Keep only recent suggestions (last 50)
  suggestions = suggestions.slice(-50)
  persist()

  // Auto-adapt strategies
  adaptStrategies()

  trackEvent({
    type: 'behavior',
    name: 'evolution-analysis',
    data: { newSuggestions: newSuggestions.length, totalSuggestions: suggestions.length },
  })
}

/** Stop the evolution engine */
export function stopEvolutionEngine(): void {
  if (analysisTimer) {
    clearInterval(analysisTimer)
    analysisTimer = null
  }
}

/** Get all suggestions */
export function getSuggestions(): ImprovementSuggestion[] {
  ensureLoaded()
  return [...suggestions].reverse()
}

/** Get suggestions by priority */
export function getSuggestionsByPriority(priority: ImprovementSuggestion['priority']): ImprovementSuggestion[] {
  return getSuggestions().filter(s => s.priority === priority)
}

/** Mark a suggestion as applied */
export function markSuggestionApplied(id: string): void {
  ensureLoaded()
  const idx = suggestions.findIndex(s => s.id === id)
  if (idx >= 0) {
    suggestions[idx].autoApplied = true
    persist()
  }
}

/** Dismiss a suggestion */
export function dismissSuggestion(id: string): void {
  ensureLoaded()
  suggestions = suggestions.filter(s => s.id !== id)
  persist()
}

/** Clear all suggestions */
export function clearSuggestions(): void {
  suggestions = []
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(SUGGESTIONS_KEY)
  }
}

/** Get a summary of the system's current state */
export function getSystemHealth(): {
  errorCount: number
  perfMetrics: number
  healingRate: number
  suggestionCount: number
  lastAnalysis: number
} {
  ensureLoaded()
  const errorPatterns = getErrorPatterns()
  const perfMetrics = getPerformanceMetrics()
  const healingStats = getHealingStats()

  return {
    errorCount: errorPatterns.reduce((sum, p) => sum + p.count, 0),
    perfMetrics: perfMetrics.length,
    healingRate: healingStats.rate,
    suggestionCount: suggestions.length,
    lastAnalysis: suggestions.length > 0 ? Math.max(...suggestions.map(s => s.timestamp)) : 0,
  }
}
