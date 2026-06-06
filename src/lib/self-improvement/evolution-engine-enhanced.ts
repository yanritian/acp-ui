// Evolution Engine Enhanced - Auto-apply improvement suggestions
// Phase 2 enhancement: 从"分析建议"到"自动执行改进"

import { getErrorPatterns, getPerformanceMetrics, getBehaviorPatterns, trackEvent } from './telemetry'
import { adaptStrategies } from './adaptive-strategy'
import { getHealingStats, attemptHealing } from './self-healing'
import type { ImprovementSuggestion } from './evolution-engine'
import { invokeSkill } from '../skill-system/skill-invoker'

// ---------------------------------------------------------------------------
// Auto-Apply Configuration
// ---------------------------------------------------------------------------

export interface AutoApplyConfig {
  enabled: boolean
  maxAutoApplyPerRun: number
  safeCategories: ImprovementSuggestion['category'][]  // Categories safe for auto-apply
  criticalRequiresApproval: boolean
  performanceAutoFix: boolean
  errorPatternAutoRecord: boolean
}

export const DEFAULT_AUTO_APPLY_CONFIG: AutoApplyConfig = {
  enabled: true,
  maxAutoApplyPerRun: 5,
  safeCategories: ['ux', 'pattern-discovery', 'performance'],
  criticalRequiresApproval: true,
  performanceAutoFix: true,
  errorPatternAutoRecord: true,
}

// ---------------------------------------------------------------------------
// Auto-Apply Actions
// ---------------------------------------------------------------------------

export interface AutoApplyAction {
  suggestionId: string
  actionType: 'cache-enable' | 'strategy-adjust' | 'pattern-record' | 'feature-highlight' | 'code-fix' | 'config-update'
  description: string
  executedAt: number
  success: boolean
  result?: string
  error?: string
}

export interface AutoApplyReport {
  timestamp: number
  totalSuggestions: number
  autoApplied: number
  manualRequired: number
  actions: AutoApplyAction[]
}

const AUTO_APPLY_KEY = 'acp-ui:auto-apply'
const MAX_REPORTS = 50

let autoApplyConfig: AutoApplyConfig = DEFAULT_AUTO_APPLY_CONFIG
let autoApplyReports: AutoApplyReport[] = []
let loaded = false

function ensureLoaded(): void {
  if (loaded) return
  loaded = true
  if (typeof localStorage === 'undefined') return
  const raw = localStorage.getItem(AUTO_APPLY_KEY)
  if (raw) {
    try { autoApplyReports = JSON.parse(raw) } catch { autoApplyReports = [] }
  }
}

function persist(): void {
  if (typeof localStorage === 'undefined') return
  const trimmed = autoApplyReports.slice(-MAX_REPORTS)
  localStorage.setItem(AUTO_APPLY_KEY, JSON.stringify(trimmed))
}

// ---------------------------------------------------------------------------
// Auto-Apply Engine
// ---------------------------------------------------------------------------

/**
 * Configure auto-apply behavior
 */
export function configureAutoApply(config: Partial<AutoApplyConfig>): void {
  autoApplyConfig = { ...autoApplyConfig, ...config }
}

/**
 * Enhanced analysis with auto-apply capability
 */
export async function analyzeAndAutoApply(
  suggestions: ImprovementSuggestion[]
): Promise<AutoApplyReport> {
  ensureLoaded()

  const actions: AutoApplyAction[] = []
  let autoAppliedCount = 0

  // Filter suggestions that can be auto-applied
  const autoApplicable = suggestions.filter(s => canAutoApply(s))

  // Limit to max per run
  const toApply = autoApplicable.slice(0, autoApplyConfig.maxAutoApplyPerRun)

  for (const suggestion of toApply) {
    try {
      const action = await executeAutoApply(suggestion)
      actions.push(action)

      if (action.success) {
        autoAppliedCount++
        trackEvent({
          type: 'behavior',
          name: 'evolution-auto-applied',
          data: { suggestionId: suggestion.id, actionType: action.actionType },
        })
      }
    } catch (error) {
      actions.push({
        suggestionId: suggestion.id,
        actionType: 'code-fix',
        description: `Failed to auto-apply: ${suggestion.title}`,
        executedAt: Date.now(),
        success: false,
        error: error instanceof Error ? error.message : String(error),
      })
    }
  }

  const report: AutoApplyReport = {
    timestamp: Date.now(),
    totalSuggestions: suggestions.length,
    autoApplied: autoAppliedCount,
    manualRequired: suggestions.length - autoAppliedCount,
    actions,
  }

  autoApplyReports.push(report)
  persist()

  return report
}

/**
 * Check if a suggestion can be auto-applied
 */
function canAutoApply(suggestion: ImprovementSuggestion): boolean {
  if (!autoApplyConfig.enabled) return false

  // Critical priority requires approval (unless disabled)
  if (suggestion.priority === 'critical' && autoApplyConfig.criticalRequiresApproval) {
    return false
  }

  // Check if category is safe for auto-apply
  if (!autoApplyConfig.safeCategories.includes(suggestion.category)) {
    return false
  }

  return true
}

/**
 * Execute auto-apply action for a suggestion
 */
async function executeAutoApply(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  const startTime = Date.now()

  switch (suggestion.category) {
    case 'performance':
      return handlePerformanceSuggestion(suggestion)

    case 'pattern-discovery':
      return handlePatternDiscoverySuggestion(suggestion)

    case 'ux':
      return handleUxSuggestion(suggestion)

    case 'error-reduction':
      return handleErrorReductionSuggestion(suggestion)

    case 'stability':
      return handleStabilitySuggestion(suggestion)

    default:
      return {
        suggestionId: suggestion.id,
        actionType: 'config-update',
        description: `Unknown category: ${suggestion.category}`,
        executedAt: startTime,
        success: false,
        error: 'Cannot auto-apply unknown category',
      }
  }
}

// ---------------------------------------------------------------------------
// Category-Specific Handlers
// ---------------------------------------------------------------------------

/**
 * Handle performance suggestions
 */
async function handlePerformanceSuggestion(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  if (!autoApplyConfig.performanceAutoFix) {
    return {
      suggestionId: suggestion.id,
      actionType: 'cache-enable',
      description: 'Performance auto-fix disabled',
      executedAt: Date.now(),
      success: false,
      error: 'Disabled by config',
    }
  }

  // Example: If operation is slow, try to enable caching via skill
  const operationName = suggestion.evidence.name as string

  try {
    // Invoke file-operations skill to check if caching can be added
    const result = await invokeSkill('file-operations', {
      action: 'analyze-cache-potential',
      operation: operationName,
    })

    if (result.success) {
      // Record the performance pattern for future reference
      return {
        suggestionId: suggestion.id,
        actionType: 'cache-enable',
        description: `Analyzed caching potential for ${operationName}`,
        executedAt: Date.now(),
        success: true,
        result: result.output,
      }
    }

    return {
      suggestionId: suggestion.id,
      actionType: 'cache-enable',
      description: `Could not analyze ${operationName}`,
      executedAt: Date.now(),
      success: false,
      error: result.error || 'Unknown error',
    }
  } catch (error) {
    return {
      suggestionId: suggestion.id,
      actionType: 'cache-enable',
      description: 'Performance analysis failed',
      executedAt: Date.now(),
      success: false,
      error: error instanceof Error ? error.message : String(error),
    }
  }
}

/**
 * Handle pattern discovery suggestions
 */
async function handlePatternDiscoverySuggestion(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  if (!autoApplyConfig.errorPatternAutoRecord) {
    return {
      suggestionId: suggestion.id,
      actionType: 'pattern-record',
      description: 'Pattern recording disabled',
      executedAt: Date.now(),
      success: false,
      error: 'Disabled by config',
    }
  }

  const patterns = suggestion.evidence.patterns as Array<{ name: string; count: number }> | undefined

  try {
    // Invoke memory-ops skill to record patterns
    const result = await invokeSkill('memory-ops', {
      action: 'record-pattern',
      patterns: patterns || [],
      category: 'error-pattern',
    })

    return {
      suggestionId: suggestion.id,
      actionType: 'pattern-record',
      description: `Recorded ${patterns?.length || 0} error patterns`,
      executedAt: Date.now(),
      success: result.success,
      result: result.output,
    }
  } catch (error) {
    return {
      suggestionId: suggestion.id,
      actionType: 'pattern-record',
      description: 'Pattern recording failed',
      executedAt: Date.now(),
      success: false,
      error: error instanceof Error ? error.message : String(error),
    }
  }
}

/**
 * Handle UX suggestions (unused features)
 */
async function handleUxSuggestion(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  const feature = suggestion.evidence.feature as string

  try {
    // Invoke communication skill to generate highlight suggestion
    const result = await invokeSkill('communication', {
      action: 'suggest-feature-highlight',
      feature,
    })

    return {
      suggestionId: suggestion.id,
      actionType: 'feature-highlight',
      description: `Generated highlight suggestion for ${feature}`,
      executedAt: Date.now(),
      success: result.success,
      result: result.output,
    }
  } catch (error) {
    return {
      suggestionId: suggestion.id,
      actionType: 'feature-highlight',
      description: 'UX suggestion failed',
      executedAt: Date.now(),
      success: false,
      error: error instanceof Error ? error.message : String(error),
    }
  }
}

/**
 * Handle error reduction suggestions
 */
async function handleErrorReductionSuggestion(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  // This is typically critical, but if auto-apply is forced:
  try {
    // Invoke security-audit skill to analyze error
    const result = await invokeSkill('security-audit', {
      action: 'analyze-error',
      errorName: suggestion.title.replace('高频错误: ', ''),
      context: suggestion.evidence,
    })

    return {
      suggestionId: suggestion.id,
      actionType: 'code-fix',
      description: 'Analyzed high-frequency error',
      executedAt: Date.now(),
      success: result.success,
      result: result.output,
    }
  } catch (error) {
    return {
      suggestionId: suggestion.id,
      actionType: 'code-fix',
      description: 'Error analysis failed',
      executedAt: Date.now(),
      success: false,
      error: error instanceof Error ? error.message : String(error),
    }
  }
}

/**
 * Handle stability suggestions (healing rate)
 */
async function handleStabilitySuggestion(suggestion: ImprovementSuggestion): Promise<AutoApplyAction> {
  if (suggestion.id === 'healing-low-rate') {
    // Adjust adaptive strategies
    adaptStrategies()

    return {
      suggestionId: suggestion.id,
      actionType: 'strategy-adjust',
      description: 'Adjusted healing strategies',
      executedAt: Date.now(),
      success: true,
      result: 'Strategies adapted based on low healing rate',
    }
  }

  // Try self-healing approach
  try {
    const result = await invokeSkill('delegation', {
      action: 'suggest-healing-improvement',
      healingStats: suggestion.evidence,
    })

    return {
      suggestionId: suggestion.id,
      actionType: 'strategy-adjust',
      description: 'Generated healing improvement suggestions',
      executedAt: Date.now(),
      success: result.success,
      result: result.output,
    }
  } catch (error) {
    return {
      suggestionId: suggestion.id,
      actionType: 'strategy-adjust',
      description: 'Stability analysis failed',
      executedAt: Date.now(),
      success: false,
      error: error instanceof Error ? error.message : String(error),
    }
  }
}

// ---------------------------------------------------------------------------
// Reports & Stats
// ---------------------------------------------------------------------------

/**
 * Get auto-apply history
 */
export function getAutoApplyReports(): AutoApplyReport[] {
  ensureLoaded()
  return [...autoApplyReports].reverse()
}

/**
 * Get auto-apply statistics
 */
export function getAutoApplyStats(): {
  totalRuns: number
  totalAutoApplied: number
  totalManualRequired: number
  successRate: number
  recentActions: AutoApplyAction[]
} {
  ensureLoaded()

  const totalRuns = autoApplyReports.length
  const totalAutoApplied = autoApplyReports.reduce((sum, r) => sum + r.autoApplied, 0)
  const totalManualRequired = autoApplyReports.reduce((sum, r) => sum + r.manualRequired, 0)

  const successfulActions = autoApplyReports.flatMap(r => r.actions).filter(a => a.success)
  const successRate = totalAutoApplied > 0 ? successfulActions.length / totalAutoApplied : 0

  const recentActions = autoApplyReports.slice(-5).flatMap(r => r.actions).reverse()

  return {
    totalRuns,
    totalAutoApplied,
    totalManualRequired,
    successRate,
    recentActions,
  }
}

/**
 * Clear auto-apply history
 */
export function clearAutoApplyHistory(): void {
  autoApplyReports = []
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(AUTO_APPLY_KEY)
  }
}

/**
 * Export evolution engine enhanced API
 */
export {
  // Re-export from evolution-engine.ts
  type ImprovementSuggestion,
} from './evolution-engine'