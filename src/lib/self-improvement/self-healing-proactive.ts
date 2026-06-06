// Self-Healing Proactive - Active code repair before failures
// Phase 2 enhancement: 主动修复代码，而不是只在错误发生后才修复

import { trackEvent, trackError } from './telemetry'
import { attemptHealing, getHealingStats, type HealingReport } from './self-healing'
import { invokeSkill } from '../skill-system/skill-invoker'

// ---------------------------------------------------------------------------
// Proactive Healing Types
// ---------------------------------------------------------------------------

export interface ProactiveCheckResult {
  category: 'syntax' | 'type' | 'security' | 'performance' | 'dependency' | 'config'
  issues: DetectedIssue[]
  autoFixed: number
  manualRequired: number
  timestamp: number
}

export interface DetectedIssue {
  id: string
  file: string
  line?: number
  severity: 'critical' | 'high' | 'medium' | 'low'
  message: string
  fixAvailable: boolean
  autoFixApplied?: boolean
  fixSuggestion?: string
}

export interface ProactiveHealingConfig {
  enabled: boolean
  checkIntervalMs: number
  autoFixEnabled: boolean
  categoriesToCheck: ProactiveCheckResult['category'][]
  maxAutoFixPerRun: number
  criticalRequiresApproval: boolean
}

export const DEFAULT_PROACTIVE_CONFIG: ProactiveHealingConfig = {
  enabled: true,
  checkIntervalMs: 60 * 60 * 1000, // 1 hour
  autoFixEnabled: true,
  categoriesToCheck: ['syntax', 'type', 'security', 'config'],
  maxAutoFixPerRun: 10,
  criticalRequiresApproval: true,
}

// ---------------------------------------------------------------------------
// Proactive Healing Engine
// ---------------------------------------------------------------------------

const PROACTIVE_KEY = 'acp-ui:proactive'
const MAX_RESULTS = 50

let proactiveConfig: ProactiveHealingConfig = DEFAULT_PROACTIVE_CONFIG
let checkResults: ProactiveCheckResult[] = []
let checkTimer: ReturnType<typeof setInterval> | null = null
let loaded = false

function ensureLoaded(): void {
  if (loaded) return
  loaded = true
  if (typeof localStorage === 'undefined') return
  const raw = localStorage.getItem(PROACTIVE_KEY)
  if (raw) {
    try { checkResults = JSON.parse(raw) } catch { checkResults = [] }
  }
}

function persist(): void {
  if (typeof localStorage === 'undefined') return
  const trimmed = checkResults.slice(-MAX_RESULTS)
  localStorage.setItem(PROACTIVE_KEY, JSON.stringify(trimmed))
}

/**
 * Configure proactive healing behavior
 */
export function configureProactiveHealing(config: Partial<ProactiveHealingConfig>): void {
  proactiveConfig = { ...proactiveConfig, ...config }
}

/**
 * Start proactive healing engine (periodic checks)
 */
export function startProactiveHealing(): void {
  if (!proactiveConfig.enabled || checkTimer) return

  // Initial check
  runProactiveCheck()

  // Periodic checks
  checkTimer = setInterval(runProactiveCheck, proactiveConfig.checkIntervalMs)

  trackEvent({
    type: 'behavior',
    name: 'proactive-healing-started',
    data: { intervalMs: proactiveConfig.checkIntervalMs },
  })
}

/**
 * Stop proactive healing engine
 */
export function stopProactiveHealing(): void {
  if (checkTimer) {
    clearInterval(checkTimer)
    checkTimer = null
  }
}

/**
 * Run a single proactive check
 */
export async function runProactiveCheck(): Promise<ProactiveCheckResult> {
  const startTime = Date.now()
  const issues: DetectedIssue[] = []

  // Run checks for each configured category
  for (const category of proactiveConfig.categoriesToCheck) {
    const categoryIssues = await checkCategory(category)
    issues.push(...categoryIssues)
  }

  // Attempt auto-fixes
  let autoFixed = 0
  let manualRequired = 0

  const fixableIssues = issues.filter(i =>
  i.fixAvailable && (i.severity !== 'critical' || !proactiveConfig.criticalRequiresApproval)
)
  const toFix = fixableIssues.slice(0, proactiveConfig.maxAutoFixPerRun)

  for (const issue of toFix) {
    const fixed = await attemptAutoFix(issue)
    if (fixed) {
      issue.autoFixApplied = true
      autoFixed++
    }
  }

  manualRequired = issues.filter(i => !i.autoFixApplied).length

  const result: ProactiveCheckResult = {
    category: 'syntax', // Overall result
    issues,
    autoFixed,
    manualRequired,
    timestamp: Date.now(),
  }

  checkResults.push(result)
  persist()

  trackEvent({
    type: 'behavior',
    name: 'proactive-check-complete',
    data: {
      totalIssues: issues.length,
      autoFixed,
      manualRequired,
      durationMs: Date.now() - startTime,
    },
  })

  return result
}

// ---------------------------------------------------------------------------
// Category Checkers
// ---------------------------------------------------------------------------

async function checkCategory(category: ProactiveCheckResult['category']): Promise<DetectedIssue[]> {
  switch (category) {
    case 'syntax':
      return checkSyntaxIssues()
    case 'type':
      return checkTypeIssues()
    case 'security':
      return checkSecurityIssues()
    case 'performance':
      return checkPerformanceIssues()
    case 'dependency':
      return checkDependencyIssues()
    case 'config':
      return checkConfigIssues()
    default:
      return []
  }
}

async function checkSyntaxIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('code-review', {
      action: 'syntax-check',
      scope: 'recent-changes',
    })

    if (!result.success) return []

    // Parse syntax issues from result
    return parseIssuesFromOutput(result.output, 'syntax')
  } catch {
    return []
  }
}

async function checkTypeIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('code-review', {
      action: 'type-check',
      scope: 'project',
    })

    if (!result.success) return []

    return parseIssuesFromOutput(result.output, 'type')
  } catch {
    return []
  }
}

async function checkSecurityIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('security-audit', {
      action: 'scan',
      scope: 'codebase',
    })

    if (!result.success) return []

    return parseIssuesFromOutput(result.output, 'security')
  } catch {
    return []
  }
}

async function checkPerformanceIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('code-review', {
      action: 'performance-analysis',
      scope: 'recent-changes',
    })

    if (!result.success) return []

    return parseIssuesFromOutput(result.output, 'performance')
  } catch {
    return []
  }
}

async function checkDependencyIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('file-operations', {
      action: 'check-dependencies',
    })

    if (!result.success) return []

    return parseIssuesFromOutput(result.output, 'dependency')
  } catch {
    return []
  }
}

async function checkConfigIssues(): Promise<DetectedIssue[]> {
  try {
    const result = await invokeSkill('file-operations', {
      action: 'validate-config',
    })

    if (!result.success) return []

    return parseIssuesFromOutput(result.output, 'config')
  } catch {
    return []
  }
}

// ---------------------------------------------------------------------------
// Auto-Fix Logic
// ---------------------------------------------------------------------------

async function attemptAutoFix(issue: DetectedIssue): Promise<boolean> {
  if (!proactiveConfig.autoFixEnabled) return false

  try {
    // Use file-operations skill to apply fix
    const result = await invokeSkill('file-operations', {
      action: 'apply-fix',
      issueId: issue.id,
      file: issue.file,
      line: issue.line,
      fix: issue.fixSuggestion,
    })

    if (result.success) {
      trackEvent({
        type: 'behavior',
        name: 'proactive-auto-fix',
        data: { issueId: issue.id, file: issue.file, severity: issue.severity },
      })
      return true
    }

    return false
  } catch {
    return false
  }
}

// ---------------------------------------------------------------------------
// Parsing Utilities
// ---------------------------------------------------------------------------

function parseIssuesFromOutput(output: string, category: ProactiveCheckResult['category']): DetectedIssue[] {
  const issues: DetectedIssue[] = []
  const lines = output.split('\n')

  for (const line of lines) {
    // Match common issue patterns
    const match = line.match(/(\w+):\s*([^:]+):(\d+)?(?:\s*:)?\s*(.+)/)
    if (match) {
      const severity = match[1].toLowerCase()
      const file = match[2]
      const lineNum = match[3] ? parseInt(match[3]) : undefined
      const message = match[4]

      issues.push({
        id: `${category}-${issues.length}`,
        file,
        line: lineNum,
        severity: severity as DetectedIssue['severity'],
        message,
        fixAvailable: line.includes('fix:') || line.includes('suggestion:'),
        fixSuggestion: line.match(/(?:fix|suggestion):\s*(.+)/)?.[1],
      })
    }
  }

  return issues
}

// ---------------------------------------------------------------------------
// Reports & Stats
// ---------------------------------------------------------------------------

export function getProactiveCheckResults(): ProactiveCheckResult[] {
  ensureLoaded()
  return [...checkResults].reverse()
}

export function getProactiveHealingStats(): {
  totalChecks: number
  totalIssuesDetected: number
  totalAutoFixed: number
  successRate: number
  lastCheckTimestamp: number
} {
  ensureLoaded()

  const totalChecks = checkResults.length
  const totalIssuesDetected = checkResults.reduce((sum, r) => sum + r.issues.length, 0)
  const totalAutoFixed = checkResults.reduce((sum, r) => sum + r.autoFixed, 0)

  const successRate = totalAutoFixed > 0 && totalIssuesDetected > 0
    ? totalAutoFixed / totalIssuesDetected
    : 0

  const lastCheckTimestamp = checkResults.length > 0
    ? checkResults[checkResults.length - 1].timestamp
    : 0

  return {
    totalChecks,
    totalIssuesDetected,
    totalAutoFixed,
    successRate,
    lastCheckTimestamp,
  }
}

export function clearProactiveHistory(): void {
  checkResults = []
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(PROACTIVE_KEY)
  }
}

// ---------------------------------------------------------------------------
// Integration with Self-Healing
// ---------------------------------------------------------------------------

/**
 * Enhanced healing that combines reactive and proactive approaches
 */
export async function enhancedHealing(
  errorName: string,
  context: Record<string, unknown>,
  retryFn?: () => Promise<unknown>,
): Promise<{
  reactiveSuccess: boolean
  proactiveIssues: DetectedIssue[]
  proactiveFixed: number
}> {
  // First, try reactive healing (traditional approach)
  const reactiveSuccess = await attemptHealing(errorName, context, retryFn)

  // If reactive failed, run proactive check on related files
  if (!reactiveSuccess && context.file) {
    const proactiveResult = await runProactiveCheck()
    const relatedIssues = proactiveResult.issues.filter(i => i.file === context.file)

    return {
      reactiveSuccess,
      proactiveIssues: relatedIssues,
      proactiveFixed: relatedIssues.filter(i => i.autoFixApplied).length,
    }
  }

  return {
    reactiveSuccess,
    proactiveIssues: [],
    proactiveFixed: 0,
  }
}