// Context Compactor - Auto-compaction for Agent Harness
// Implements MVP Blueprint checklist: context overflow detection, summary generation, rehydration

import type { ChatMessage, ToolCallInfo } from '../types'
import type { RuntimeSession, RuntimeOutput } from './types'

/**
 * Compaction trigger thresholds
 */
export interface CompactionThresholds {
  /** Context percentage to trigger compaction (e.g., 80% of limit) */
  contextPercentThreshold: number
  /** Minimum messages before compaction */
  minMessagesBeforeCompaction: number
  /** Maximum messages to keep after compaction */
  maxMessagesToKeep: number
  /** Maximum tool calls to keep in summary */
  maxToolCallsInSummary: number
  /** Maximum age of messages to keep (ms) */
  maxAgeMsToKeep: number
}

/**
 * Default compaction thresholds
 */
export const DEFAULT_COMPACTION_THRESHOLDS: CompactionThresholds = {
  contextPercentThreshold: 80, // Trigger at 80% of context limit
  minMessagesBeforeCompaction: 20,
  maxMessagesToKeep: 10,
  maxToolCallsInSummary: 15,
  maxAgeMsToKeep: 600000, // 10 minutes
}

/**
 * Compaction summary structure
 */
export interface CompactionSummary {
  /** Current objective being pursued */
  currentObjective: string
  /** User constraints and preferences */
  userConstraints: string[]
  /** Authoritative instructions loaded */
  authoritativeInstructions: string[]
  /** Active plan or goal */
  activePlan: string | null
  /** Tools and connectors used */
  toolsUsed: string[]
  /** Source data inspected */
  sourceDataInspected: string[]
  /** Actions already taken */
  actionsTaken: string[]
  /** Decisions made */
  decisionsMade: string[]
  /** Errors and blockers encountered */
  errorsAndBlockers: string[]
  /** Approval state */
  approvalState: string | null
  /** Pending tasks */
  pendingTasks: string[]
  /** Next recommended step */
  nextRecommendedStep: string
  /** Things not to redo */
  doNotRedo: string[]
  /** Compaction timestamp */
  compactionTimestamp: number
  /** Original message count */
  originalMessageCount: number
  /** Retained message count */
  retainedMessageCount: number
}

/**
 * Rehydration artifacts - what to restore after compaction
 */
export interface RehydrationArtifacts {
  /** Active plan if any */
  activePlan: string | null
  /** Goal state if any */
  goalState: string | null
  /** Latest todo list */
  todoList: string[]
  /** Approval state */
  approvalState: string | null
  /** Recent important tool results */
  recentToolResults: string[]
  /** Loaded scoped instructions */
  loadedInstructions: string[]
  /** Selected skills */
  selectedSkills: string[]
  /** Connector availability */
  connectorAvailability: string[]
  /** Source-of-truth references */
  sourceOfTruthRefs: string[]
}

/**
 * Compaction result
 */
export interface CompactionResult {
  /** Summary of compacted content */
  summary: CompactionSummary
  /** Messages retained after compaction */
  retainedMessages: ChatMessage[]
  /** Tool calls retained */
  retainedToolCalls: ToolCallInfo[]
  /** Rehydration artifacts */
  rehydrationArtifacts: RehydrationArtifacts
  /** Whether compaction was triggered */
  triggered: boolean
  /** Reason for compaction if triggered */
  triggerReason?: string
  /** Space freed (estimated tokens) */
  spaceFreed?: number
}

/**
 * Context compactor class
 */
export class ContextCompactor {
  private thresholds: CompactionThresholds
  private estimatedContextLimit: number
  private lastCompactionTimestamp: number = 0

  constructor(
    thresholds: Partial<CompactionThresholds> = {},
    estimatedContextLimit: number = 100000 // Default 100k tokens
  ) {
    this.thresholds = { ...DEFAULT_COMPACTION_THRESHOLDS, ...thresholds }
    this.estimatedContextLimit = estimatedContextLimit
  }

  /**
   * Check if compaction is needed
   */
  needsCompaction(
    messages: ChatMessage[],
    toolCalls: ToolCallInfo[],
    currentTokenEstimate: number
  ): { needed: boolean; reason?: string } {
    // Check context percentage threshold
    const contextPercent = (currentTokenEstimate / this.estimatedContextLimit) * 100
    if (contextPercent >= this.thresholds.contextPercentThreshold) {
      return {
        needed: true,
        reason: `Context at ${Math.round(contextPercent)}% of limit (${currentTokenEstimate}/${this.estimatedContextLimit} tokens)`,
      }
    }

    // Check minimum messages threshold
    if (messages.length >= this.thresholds.minMessagesBeforeCompaction) {
      // Check if we have too much old content
      const now = Date.now()
      const oldMessages = messages.filter(
        m => now - m.timestamp > this.thresholds.maxAgeMsToKeep
      )
      if (oldMessages.length > messages.length * 0.5) {
        return {
          needed: true,
          reason: `Over 50% of messages older than ${Math.round(this.thresholds.maxAgeMsToKeep/60000)} minutes`,
        }
      }
    }

    return { needed: false }
  }

  /**
   * Perform compaction
   */
  compact(
    messages: ChatMessage[],
    toolCalls: ToolCallInfo[],
    objective: string = 'Task in progress',
    plan: string | null = null
  ): CompactionResult {
    const now = Date.now()
    const originalCount = messages.length

    // Don't compact if not enough messages
    if (messages.length < this.thresholds.minMessagesBeforeCompaction) {
      return this.createNoCompactionResult(messages, toolCalls)
    }

    // Generate summary from all messages
    const summary = this.generateSummary(messages, toolCalls, objective, plan)

    // Retain recent messages
    const retainedMessages = this.selectRetainedMessages(messages)

    // Retain important tool calls
    const retainedToolCalls = this.selectRetainedToolCalls(toolCalls)

    // Create rehydration artifacts
    const rehydrationArtifacts = this.createRehydrationArtifacts(
      retainedMessages,
      retainedToolCalls,
      plan
    )

    // Calculate space freed
    const originalTokens = this.estimateTokens(messages)
    const retainedTokens = this.estimateTokens(retainedMessages)
    const spaceFreed = originalTokens - retainedTokens

    this.lastCompactionTimestamp = now

    return {
      summary,
      retainedMessages,
      retainedToolCalls,
      rehydrationArtifacts,
      triggered: true,
      triggerReason: `Compacted ${originalCount} messages to ${retainedMessages.length}`,
      spaceFreed,
    }
  }

  /**
   * Generate compaction summary
   */
  private generateSummary(
    messages: ChatMessage[],
    toolCalls: ToolCallInfo[],
    objective: string,
    plan: string | null
  ): CompactionSummary {
    const now = Date.now()

    // Extract user constraints from user messages
    const userConstraints = this.extractUserConstraints(messages)

    // Extract tools used
    const toolsUsed = this.extractToolsUsed(toolCalls)

    // Extract actions taken
    const actionsTaken = this.extractActionsTaken(messages, toolCalls)

    // Extract decisions made
    const decisionsMade = this.extractDecisionsMade(messages)

    // Extract errors
    const errorsAndBlockers = this.extractErrors(messages)

    // Extract pending tasks
    const pendingTasks = this.extractPendingTasks(messages)

    // Extract do-not-redo items
    const doNotRedo = this.extractDoNotRedo(messages, toolCalls)

    // Determine next step from last assistant message
    const lastAssistantMsg = [...messages].reverse().find(m => m.role === 'assistant')
    const nextRecommendedStep = lastAssistantMsg?.content.slice(0, 200) || 'Continue with task'

    return {
      currentObjective: objective,
      userConstraints,
      authoritativeInstructions: [], // Would be populated from actual instruction tracking
      activePlan: plan,
      toolsUsed,
      sourceDataInspected: [], // Would be populated from file read tracking
      actionsTaken,
      decisionsMade,
      errorsAndBlockers,
      approvalState: null, // Would be populated from permission tracking
      pendingTasks,
      nextRecommendedStep,
      doNotRedo,
      compactionTimestamp: now,
      originalMessageCount: messages.length,
      retainedMessageCount: this.thresholds.maxMessagesToKeep,
    }
  }

  /**
   * Select messages to retain after compaction
   */
  private selectRetainedMessages(messages: ChatMessage[]): ChatMessage[] {
    const now = Date.now()
    const maxKeep = this.thresholds.maxMessagesToKeep

    // Always keep the first user message (task definition)
    const firstUserMsg = messages.find(m => m.role === 'user')

    // Keep recent messages within time threshold
    const recentMessages = messages.filter(
      m => now - m.timestamp <= this.thresholds.maxAgeMsToKeep
    )

    // Keep last assistant message (current state)
    const lastAssistantMsg = [...messages].reverse().find(m => m.role === 'assistant')

    // Combine and deduplicate
    const toKeep: ChatMessage[] = []
    const keptIds = new Set<string>()

    if (firstUserMsg && !keptIds.has(firstUserMsg.id)) {
      toKeep.push(firstUserMsg)
      keptIds.add(firstUserMsg.id)
    }

    if (lastAssistantMsg && !keptIds.has(lastAssistantMsg.id)) {
      toKeep.push(lastAssistantMsg)
      keptIds.add(lastAssistantMsg.id)
    }

    for (const m of recentMessages) {
      if (!keptIds.has(m.id) && toKeep.length < maxKeep) {
        toKeep.push(m)
        keptIds.add(m.id)
      }
    }

    // Sort by timestamp
    toKeep.sort((a, b) => a.timestamp - b.timestamp)

    return toKeep
  }

  /**
   * Select tool calls to retain
   */
  private selectRetainedToolCalls(toolCalls: ToolCallInfo[]): ToolCallInfo[] {
    // Keep recent tool calls, prioritizing completed ones
    const recent = toolCalls
      .filter(tc => tc.status === 'completed' || tc.status === 'in_progress')
      .slice(-this.thresholds.maxToolCallsInSummary)

    return recent
  }

  /**
   * Create rehydration artifacts
   */
  private createRehydrationArtifacts(
    messages: ChatMessage[],
    toolCalls: ToolCallInfo[],
    plan: string | null
  ): RehydrationArtifacts {
    // Extract todo items from messages
    const todoList = this.extractPendingTasks(messages)

    // Get recent tool result summaries
    const recentToolResults = toolCalls
      .filter(tc => tc.status === 'completed')
      .slice(-5)
      .map(tc => `${tc.kind}: ${tc.title}`)

    return {
      activePlan: plan,
      goalState: null, // Would be populated from goal tracking
      todoList,
      approvalState: null,
      recentToolResults,
      loadedInstructions: [],
      selectedSkills: [],
      connectorAvailability: [],
      sourceOfTruthRefs: [],
    }
  }

  /**
   * Create a result when no compaction is needed
   */
  private createNoCompactionResult(
    messages: ChatMessage[],
    toolCalls: ToolCallInfo[]
  ): CompactionResult {
    return {
      summary: this.generateEmptySummary(),
      retainedMessages: messages,
      retainedToolCalls: toolCalls,
      rehydrationArtifacts: this.createEmptyRehydrationArtifacts(),
      triggered: false,
    }
  }

  /**
   * Generate empty summary for non-compaction cases
   */
  private generateEmptySummary(): CompactionSummary {
    return {
      currentObjective: '',
      userConstraints: [],
      authoritativeInstructions: [],
      activePlan: null,
      toolsUsed: [],
      sourceDataInspected: [],
      actionsTaken: [],
      decisionsMade: [],
      errorsAndBlockers: [],
      approvalState: null,
      pendingTasks: [],
      nextRecommendedStep: '',
      doNotRedo: [],
      compactionTimestamp: Date.now(),
      originalMessageCount: 0,
      retainedMessageCount: 0,
    }
  }

  /**
   * Create empty rehydration artifacts
   */
  private createEmptyRehydrationArtifacts(): RehydrationArtifacts {
    return {
      activePlan: null,
      goalState: null,
      todoList: [],
      approvalState: null,
      recentToolResults: [],
      loadedInstructions: [],
      selectedSkills: [],
      connectorAvailability: [],
      sourceOfTruthRefs: [],
    }
  }

  /**
   * Estimate token count from messages
   */
  estimateTokens(messages: ChatMessage[]): number {
    // Rough estimation: ~4 chars per token
    let totalChars = 0
    for (const m of messages) {
      totalChars += m.content.length
      if (m.thought) totalChars += m.thought.length
      if (m.toolCalls) {
        for (const tc of m.toolCalls) {
          totalChars += tc.title.length + tc.kind.length
        }
      }
    }
    return Math.ceil(totalChars / 4)
  }

  /**
   * Extract user constraints from messages
   */
  private extractUserConstraints(messages: ChatMessage[]): string[] {
    const constraints: string[] = []
    const constraintPatterns = [
      /do not|don't|never|avoid/i,
      /must|should|need|required/i,
      /only|just|limit|max/i,
      /prefer|want|would like/i,
    ]

    for (const m of messages) {
      if (m.role === 'user') {
        for (const pattern of constraintPatterns) {
          if (pattern.test(m.content)) {
            // Extract the sentence containing the constraint
            const sentences = m.content.split(/[.!?]/)
            for (const s of sentences) {
              if (pattern.test(s) && s.trim().length > 10) {
                constraints.push(s.trim())
              }
            }
          }
        }
      }
    }

    return constraints.slice(0, 10) // Limit to 10 constraints
  }

  /**
   * Extract tools used from tool calls
   */
  private extractToolsUsed(toolCalls: ToolCallInfo[]): string[] {
    const tools = new Set<string>()
    for (const tc of toolCalls) {
      tools.add(tc.kind)
    }
    return Array.from(tools)
  }

  /**
   * Extract actions taken from messages and tool calls
   */
  private extractActionsTaken(messages: ChatMessage[], toolCalls: ToolCallInfo[]): string[] {
    const actions: string[] = []

    for (const tc of toolCalls) {
      if (tc.status === 'completed') {
        actions.push(`${tc.kind}: ${tc.title}`)
      }
    }

    return actions.slice(-20) // Keep last 20 actions
  }

  /**
   * Extract decisions made from assistant messages
   */
  private extractDecisionsMade(messages: ChatMessage[]): string[] {
    const decisions: string[] = []
    const decisionPatterns = [
      /decided to|i will|i'm going to|chose|selected/i,
      /because|since|the reason/i,
    ]

    for (const m of messages) {
      if (m.role === 'assistant') {
        for (const pattern of decisionPatterns) {
          const matches = m.content.match(pattern)
          if (matches) {
            // Extract surrounding context
            const idx = m.content.indexOf(matches[0])
            const snippet = m.content.slice(Math.max(0, idx - 20), idx + 100)
            decisions.push(snippet.trim())
          }
        }
      }
    }

    return decisions.slice(-10)
  }

  /**
   * Extract errors from messages
   */
  private extractErrors(messages: ChatMessage[]): string[] {
    const errors: string[] = []
    const errorPatterns = [
      /error|failed|exception|timeout/i,
      /cannot|unable to|couldn't/i,
      /blocked|denied|not allowed/i,
    ]

    for (const m of messages) {
      for (const pattern of errorPatterns) {
        if (pattern.test(m.content)) {
          errors.push(m.content.slice(0, 150))
        }
      }
    }

    return errors.slice(-10)
  }

  /**
   * Extract pending tasks from messages
   */
  private extractPendingTasks(messages: ChatMessage[]): string[] {
    const tasks: string[] = []
    const taskPatterns = [
      /todo|pending|still need|next step|remaining/i,
      /haven't|not yet|to do/i,
    ]

    for (const m of messages) {
      if (m.role === 'assistant') {
        for (const pattern of taskPatterns) {
          if (pattern.test(m.content)) {
            const sentences = m.content.split(/[.!?]/)
            for (const s of sentences) {
              if (pattern.test(s) && s.trim().length > 10) {
                tasks.push(s.trim())
              }
            }
          }
        }
      }
    }

    return tasks.slice(-10)
  }

  /**
   * Extract do-not-redo items
   */
  private extractDoNotRedo(messages: ChatMessage[], toolCalls: ToolCallInfo[]): string[] {
    const doNotRedo: string[] = []

    // Add completed tool calls to do-not-redo
    for (const tc of toolCalls) {
      if (tc.status === 'completed') {
        doNotRedo.push(`Tool ${tc.kind} already executed: ${tc.title}`)
      }
    }

    // Add explicitly stated do-not items
    for (const m of messages) {
      if (m.role === 'user') {
        if (/do not|don't|never/i.test(m.content)) {
          doNotRedo.push(m.content.slice(0, 100))
        }
      }
    }

    return doNotRedo.slice(-15)
  }

  /**
   * Format summary for context injection
   */
  formatSummaryForContext(summary: CompactionSummary): string {
    const lines: string[] = ['## Session Summary (Auto-Compacted)']

    if (summary.currentObjective) {
      lines.push(`**Objective**: ${summary.currentObjective}`)
    }

    if (summary.userConstraints.length > 0) {
      lines.push('\n**User Constraints**:')
      for (const c of summary.userConstraints) {
        lines.push(`- ${c}`)
      }
    }

    if (summary.activePlan) {
      lines.push(`\n**Active Plan**: ${summary.activePlan}`)
    }

    if (summary.toolsUsed.length > 0) {
      lines.push(`\n**Tools Used**: ${summary.toolsUsed.join(', ')}`)
    }

    if (summary.actionsTaken.length > 0) {
      lines.push('\n**Actions Already Taken**:')
      for (const a of summary.actionsTaken.slice(-10)) {
        lines.push(`- ${a}`)
      }
    }

    if (summary.decisionsMade.length > 0) {
      lines.push('\n**Key Decisions**:')
      for (const d of summary.decisionsMade.slice(-5)) {
        lines.push(`- ${d}`)
      }
    }

    if (summary.errorsAndBlockers.length > 0) {
      lines.push('\n**Errors/Blockers**:')
      for (const e of summary.errorsAndBlockers) {
        lines.push(`- ${e}`)
      }
    }

    if (summary.pendingTasks.length > 0) {
      lines.push('\n**Pending Tasks**:')
      for (const t of summary.pendingTasks) {
        lines.push(`- ${t}`)
      }
    }

    if (summary.doNotRedo.length > 0) {
      lines.push('\n**Do Not Redo**:')
      for (const d of summary.doNotRedo.slice(-5)) {
        lines.push(`- ${d}`)
      }
    }

    lines.push(`\n**Next Step**: ${summary.nextRecommendedStep}`)

    return lines.join('\n')
  }

  /**
   * Get last compaction timestamp
   */
  getLastCompactionTimestamp(): number {
    return this.lastCompactionTimestamp
  }
}

/**
 * Create a context compactor with default settings
 */
export function createContextCompactor(
  thresholds?: Partial<CompactionThresholds>,
  contextLimit?: number
): ContextCompactor {
  return new ContextCompactor(thresholds, contextLimit)
}