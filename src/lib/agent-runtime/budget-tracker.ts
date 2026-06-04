// Budget Tracker - Loop budget control for Agent Harness
// Implements MVP Blueprint checklist: step, tool-call, time, token, and cost budgets

/**
 * Budget limits for an agent session
 */
export interface BudgetLimits {
  /** Maximum model turns/steps in the loop */
  maxSteps: number
  /** Maximum total tool calls allowed */
  maxToolCalls: number
  /** Maximum parallel tool calls per step */
  maxParallelToolCalls: number
  /** Maximum wall time in milliseconds */
  maxTimeMs: number
  /** Maximum input tokens */
  maxInputTokens: number
  /** Maximum output tokens */
  maxOutputTokens: number
  /** Maximum cost in USD */
  maxCostUsd: number
  /** Maximum tool result characters */
  maxToolResultChars: number
  /** Maximum retries per model call */
  maxRetriesPerModelCall: number
  /** Maximum retries per tool call */
  maxRetriesPerToolCall: number
}

/**
 * Current budget consumption state
 */
export interface BudgetConsumption {
  steps: number
  toolCalls: number
  parallelToolCalls: number
  inputTokens: number
  outputTokens: number
  costUsd: number
  timeMs: number
  toolResultChars: number
  retriesModel: number
  retriesTool: number
}

/**
 * Budget exceeded result
 */
export interface BudgetExceededResult {
  exceeded: true
  limitName: string
  current: number
  limit: number
  reason: string
  nextSafeAction: string
}

/**
 * Budget check result
 */
export type BudgetCheckResult =
  | { exceeded: false }
  | BudgetExceededResult

/**
 * Default budget limits (conservative for MVP)
 */
export const DEFAULT_BUDGET_LIMITS: BudgetLimits = {
  maxSteps: 50,
  maxToolCalls: 100,
  maxParallelToolCalls: 5,
  maxTimeMs: 300000, // 5 minutes
  maxInputTokens: 100000,
  maxOutputTokens: 50000,
  maxCostUsd: 5.00,
  maxToolResultChars: 50000,
  maxRetriesPerModelCall: 3,
  maxRetriesPerToolCall: 2,
}

/**
 * Budget tracker class
 */
export class BudgetTracker {
  private limits: BudgetLimits
  private consumption: BudgetConsumption
  private startTime: number
  private exceeded: BudgetExceededResult | null = null

  constructor(limits: Partial<BudgetLimits> = {}) {
    this.limits = { ...DEFAULT_BUDGET_LIMITS, ...limits }
    this.consumption = {
      steps: 0,
      toolCalls: 0,
      parallelToolCalls: 0,
      inputTokens: 0,
      outputTokens: 0,
      costUsd: 0,
      timeMs: 0,
      toolResultChars: 0,
      retriesModel: 0,
      retriesTool: 0,
    }
    this.startTime = Date.now()
  }

  /**
   * Check if budget is exceeded
   */
  check(): BudgetCheckResult {
    // Update time consumption
    this.consumption.timeMs = Date.now() - this.startTime

    // Check each limit in order
    const checks: Array<{ name: string; current: number; limit: number; reason: string }> = [
      {
        name: 'maxSteps',
        current: this.consumption.steps,
        limit: this.limits.maxSteps,
        reason: 'Step limit reached',
      },
      {
        name: 'maxToolCalls',
        current: this.consumption.toolCalls,
        limit: this.limits.maxToolCalls,
        reason: 'Tool call limit reached',
      },
      {
        name: 'maxTimeMs',
        current: this.consumption.timeMs,
        limit: this.limits.maxTimeMs,
        reason: 'Time limit reached',
      },
      {
        name: 'maxInputTokens',
        current: this.consumption.inputTokens,
        limit: this.limits.maxInputTokens,
        reason: 'Input token limit reached',
      },
      {
        name: 'maxOutputTokens',
        current: this.consumption.outputTokens,
        limit: this.limits.maxOutputTokens,
        reason: 'Output token limit reached',
      },
      {
        name: 'maxCostUsd',
        current: this.consumption.costUsd,
        limit: this.limits.maxCostUsd,
        reason: 'Cost limit reached',
      },
      {
        name: 'maxToolResultChars',
        current: this.consumption.toolResultChars,
        limit: this.limits.maxToolResultChars,
        reason: 'Tool result size limit reached',
      },
    ]

    for (const check of checks) {
      if (check.current > check.limit) {
        this.exceeded = {
          exceeded: true,
          limitName: check.name,
          current: check.current,
          limit: check.limit,
          reason: check.reason,
          nextSafeAction: 'Ask the user whether to continue with a larger budget.',
        }
        return this.exceeded
      }
    }

    return { exceeded: false }
  }

  /**
   * Record a step completion
   */
  recordStep(): void {
    this.consumption.steps++
  }

  /**
   * Record tool calls
   */
  recordToolCalls(count: number, parallelCount: number = 0): void {
    this.consumption.toolCalls += count
    this.consumption.parallelToolCalls = Math.max(
      this.consumption.parallelToolCalls,
      parallelCount
    )
  }

  /**
   * Record token usage
   */
  recordTokens(inputTokens: number, outputTokens: number): void {
    this.consumption.inputTokens += inputTokens
    this.consumption.outputTokens += outputTokens
  }

  /**
   * Record cost
   */
  recordCost(costUsd: number): void {
    this.consumption.costUsd += costUsd
  }

  /**
   * Record tool result size
   */
  recordToolResultSize(chars: number): void {
    this.consumption.toolResultChars += chars
  }

  /**
   * Record model retry
   */
  recordModelRetry(): void {
    this.consumption.retriesModel++
  }

  /**
   * Record tool retry
   */
  recordToolRetry(): void {
    this.consumption.retriesTool++
  }

  /**
   * Check if parallel tool calls limit is exceeded
   */
  checkParallelLimit(requestedCount: number): BudgetCheckResult {
    if (requestedCount > this.limits.maxParallelToolCalls) {
      return {
        exceeded: true,
        limitName: 'maxParallelToolCalls',
        current: requestedCount,
        limit: this.limits.maxParallelToolCalls,
        reason: `Requested ${requestedCount} parallel calls exceeds limit`,
        nextSafeAction: 'Reduce parallel calls or increase the limit.',
      }
    }
    return { exceeded: false }
  }

  /**
   * Check if retries are allowed
   */
  canRetryModel(): boolean {
    return this.consumption.retriesModel < this.limits.maxRetriesPerModelCall
  }

  /**
   * Check if tool retries are allowed
   */
  canRetryTool(): boolean {
    return this.consumption.retriesTool < this.limits.maxRetriesPerToolCall
  }

  /**
   * Get current consumption
   */
  getConsumption(): BudgetConsumption {
    return { ...this.consumption, timeMs: Date.now() - this.startTime }
  }

  /**
   * Get limits
   */
  getLimits(): BudgetLimits {
    return { ...this.limits }
  }

  /**
   * Get exceeded result if any
   */
  getExceededResult(): BudgetExceededResult | null {
    return this.exceeded
  }

  /**
   * Reset budget tracker for new task
   */
  reset(): void {
    this.consumption = {
      steps: 0,
      toolCalls: 0,
      parallelToolCalls: 0,
      inputTokens: 0,
      outputTokens: 0,
      costUsd: 0,
      timeMs: 0,
      toolResultChars: 0,
      retriesModel: 0,
      retriesTool: 0,
    }
    this.startTime = Date.now()
    this.exceeded = null
  }

  /**
   * Create a budget exceeded error
   */
  createBudgetError(result: BudgetExceededResult): Error {
    const error = new Error(
      `Budget exceeded: ${result.reason} (${result.current}/${result.limit})`
    )
    error.name = 'BudgetExceededError'
    return error
  }

  /**
   * Format consumption for logging
   */
  formatConsumption(): string {
    const c = this.getConsumption()
    const l = this.limits
    return [
      `Steps: ${c.steps}/${l.maxSteps}`,
      `ToolCalls: ${c.toolCalls}/${l.maxToolCalls}`,
      `Time: ${Math.round(c.timeMs/1000)}s/${Math.round(l.maxTimeMs/1000)}s`,
      `Tokens(in/out): ${c.inputTokens}/${l.maxInputTokens} / ${c.outputTokens}/${l.maxOutputTokens}`,
      `Cost: $${c.costUsd.toFixed(2)}/$${l.maxCostUsd.toFixed(2)}`,
    ].join(' | ')
  }
}

/**
 * Budget stop status
 */
export interface BudgetStopStatus {
  status: 'stopped'
  reason: string
  completed: boolean
  consumption: BudgetConsumption
  exceededLimit?: BudgetExceededResult
  nextSafeAction: string
}

/**
 * Create a stop status for budget exceeded
 */
export function createBudgetStopStatus(
  result: BudgetExceededResult,
  consumption: BudgetConsumption
): BudgetStopStatus {
  return {
    status: 'stopped',
    reason: result.reason,
    completed: false,
    consumption,
    exceededLimit: result,
    nextSafeAction: result.nextSafeAction,
  }
}

/**
 * Create a stop status for normal completion
 */
export function createCompletionStatus(
  reason: string,
  consumption: BudgetConsumption
): BudgetStopStatus {
  return {
    status: 'stopped',
    reason,
    completed: true,
    consumption,
    nextSafeAction: 'Task completed successfully.',
  }
}