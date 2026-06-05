// Token Optimizer - OpenClacky-style dual-cache token optimization
// Implements: frozen prompt caching, insert-then-compress strategy, idle compression timer

import type { ChatMessage } from '../types'

// ---------------------------------------------------------------------------
// Interfaces & Types
// ---------------------------------------------------------------------------

/**
 * A simplified message representation used by the token optimizer.
 * Compatible with ChatMessage but focused on compression needs.
 */
export interface ContextMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
}

/**
 * Dual-cache system: system prompt is frozen (byte-identical) for ~100% cache hit.
 * Dynamic user context is injected per-session, separate from the frozen prompt.
 */
export interface CacheState {
  frozenPrompt: string | null
  frozenHash: string | null
  sessionContext: Map<string, SessionContextEntry>
  lastCompressionAt: number
  cacheHits: number
  cacheMisses: number
}

/**
 * A single entry in the session context map.
 */
export interface SessionContextEntry {
  key: string
  content: string
  tokens: number
  createdAt: number
  expiresAt: number
}

/**
 * Result of a context compression operation.
 */
export interface CompressionResult {
  originalTokens: number
  compressedTokens: number
  savedTokens: number
  compressionRatio: number
  archivedChunks: ArchivedChunk[]
  summary: string
}

/**
 * An archived chunk of compressed messages that can be referenced by the AI
 * for lookback without keeping the full messages in context.
 */
export interface ArchivedChunk {
  id: string
  originalMessages: number
  compressedSummary: string
  referencePath: string
  timestamp: string
}

/**
 * Configuration for the TokenOptimizer.
 */
export interface TokenOptimizerConfig {
  /** Maximum tokens before compression is triggered (default: 100000) */
  maxContextTokens: number
  /** Number of recent messages to keep uncompressed (default: 10) */
  slidingWindowSize: number
  /** Target token count for compressed cold data (default: 500) */
  coldDataTokenBudget: number
  /** Idle time in ms before proactive compression (default: 300000 = 5 min) */
  idleThresholdMs: number
  /** Maximum number of archived chunks to retain (default: 50) */
  maxArchiveSize: number
  /** TTL for session context entries in ms (default: 3600000 = 1 hour) */
  sessionContextTtlMs: number
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

export const DEFAULT_TOKEN_OPTIMIZER_CONFIG: TokenOptimizerConfig = {
  maxContextTokens: 100000,
  slidingWindowSize: 10,
  coldDataTokenBudget: 500,
  idleThresholdMs: 300_000,
  maxArchiveSize: 50,
  sessionContextTtlMs: 3_600_000,
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

/**
 * Estimate token count from a string using the ~4 chars per token heuristic.
 * Matches the estimation used in context-compactor.ts and context-builder.ts.
 */
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4)
}

/**
 * Simple deterministic hash for cache identification.
 * Same algorithm used in context-builder.ts for stable prefix hashing.
 */
function hashString(input: string): string {
  let hash = 0
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) - hash + input.charCodeAt(i)) | 0
  }
  return hash.toString(16)
}

/**
 * Generate a unique ID for archived chunks.
 */
function generateChunkId(index: number): string {
  const ts = Date.now().toString(36)
  return `archive-${ts}-${index}`
}

/**
 * Summarize a list of messages into a compact text summary.
 * Extracts key actions, decisions, and outcomes from the conversation history
 * to produce a dense representation suitable for cold storage.
 */
export function summarizeMessages(messages: ContextMessage[]): string {
  if (messages.length === 0) return ''

  const userMessages = messages.filter(m => m.role === 'user')
  const assistantMessages = messages.filter(m => m.role === 'assistant')

  const lines: string[] = []

  // Extract user intents (first sentence of each user message, truncated)
  if (userMessages.length > 0) {
    lines.push('User requests:')
    for (const msg of userMessages.slice(0, 5)) {
      const firstSentence = msg.content.split(/[.!?\n]/)[0] ?? ''
      const truncated = firstSentence.trim().slice(0, 120)
      if (truncated.length > 0) {
        lines.push(`- ${truncated}`)
      }
    }
  }

  // Extract assistant actions/decisions
  if (assistantMessages.length > 0) {
    lines.push('Assistant actions:')
    for (const msg of assistantMessages.slice(-5)) {
      // Look for action-oriented sentences
      const actionPatterns = [
        /(?:I(?:'ll| will)|decided to|going to|created|updated|deleted|fixed|added|removed|changed|ran|executed)\b/i,
      ]
      const sentences = msg.content.split(/[.!?\n]/)
      const actionSentences = sentences.filter(s =>
        actionPatterns.some(p => p.test(s))
      )
      const chosen = actionSentences.length > 0
        ? actionSentences.slice(0, 2)
        : [sentences[0] ?? '']
      for (const s of chosen) {
        const trimmed = s.trim().slice(0, 120)
        if (trimmed.length > 0) {
          lines.push(`- ${trimmed}`)
        }
      }
    }
  }

  // Summary statistics
  lines.push(`[${messages.length} messages compressed]`)

  return lines.join('\n')
}

// ---------------------------------------------------------------------------
// IdleCompressionTimer
// ---------------------------------------------------------------------------

/**
 * Timer that triggers proactive context compression after a period of user
 * inactivity. Resets on every user interaction and resumes after pauses.
 */
export class IdleCompressionTimer {
  private timer: ReturnType<typeof setTimeout> | null = null
  private idleThresholdMs: number
  private onIdleCompress: () => void
  private remainingMs: number
  private pausedAt: number = 0

  constructor(idleThresholdMs: number, onIdleCompress: () => void) {
    this.idleThresholdMs = idleThresholdMs
    this.remainingMs = idleThresholdMs
    this.onIdleCompress = onIdleCompress
  }

  /**
   * Reset the idle timer. Called on every user interaction.
   */
  reset(): void {
    this.clearTimer()
    this.remainingMs = this.idleThresholdMs
    this.startTimer()
  }

  /**
   * Pause the timer, preserving remaining time.
   */
  pause(): void {
    if (this.timer !== null) {
      this.pausedAt = Date.now()
      this.clearTimer()
    }
  }

  /**
   * Resume the timer with whatever time was remaining before pause.
   */
  resume(): void {
    if (this.pausedAt > 0) {
      this.pausedAt = 0
      this.startTimer()
    }
  }

  /**
   * Destroy the timer completely. Call during cleanup.
   */
  destroy(): void {
    this.clearTimer()
    this.pausedAt = 0
  }

  /**
   * Update the idle threshold (e.g. for configuration changes).
   */
  setThreshold(ms: number): void {
    this.idleThresholdMs = ms
    this.reset()
  }

  private startTimer(): void {
    this.timer = setTimeout(() => {
      this.timer = null
      this.onIdleCompress()
    }, this.remainingMs)
  }

  private clearTimer(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer)
      this.timer = null
    }
  }
}

// ---------------------------------------------------------------------------
// TokenOptimizer
// ---------------------------------------------------------------------------

/**
 * OpenClacky-style token optimization engine.
 *
 * Combines three strategies:
 * 1. **Dual-cache markers** - Freeze the system prompt for ~100% cache hit rate
 *    while injecting dynamic per-session context separately.
 * 2. **Insert-then-compress** - When the context window fills up, insert summary
 *    markers between old messages and compress them into compact cold storage
 *    while keeping recent messages in a sliding window.
 * 3. **Idle compression** - After a configurable idle period, proactively
 *    compress context before the provider cache expires.
 */
export class TokenOptimizer {
  private config: TokenOptimizerConfig
  private cacheState: CacheState
  private idleTimer: IdleCompressionTimer
  private archive: ArchivedChunk[] = []
  private chunkCounter: number = 0

  constructor(config: Partial<TokenOptimizerConfig> = {}) {
    this.config = { ...DEFAULT_TOKEN_OPTIMIZER_CONFIG, ...config }

    this.cacheState = {
      frozenPrompt: null,
      frozenHash: null,
      sessionContext: new Map(),
      lastCompressionAt: 0,
      cacheHits: 0,
      cacheMisses: 0,
    }

    this.idleTimer = new IdleCompressionTimer(
      this.config.idleThresholdMs,
      () => this.onIdleCompress()
    )
  }

  // -----------------------------------------------------------------------
  // Dual-Cache Marker System
  // -----------------------------------------------------------------------

  /**
   * Freeze a system prompt for maximum cache reuse across sessions.
   * Returns the hash of the frozen prompt for cache identification.
   *
   * The frozen prompt is kept byte-identical so that the LLM provider's
   * prefix cache can serve it at near-zero cost. Dynamic session data is
   * injected separately via `buildSessionContext`.
   */
  freezeSystemPrompt(prompt: string): string {
    const hash = hashString(prompt)

    // If the prompt hasn't changed, count as cache hit
    if (this.cacheState.frozenHash === hash) {
      this.cacheState.cacheHits++
      return hash
    }

    // New or changed prompt - freeze it
    this.cacheState.frozenPrompt = prompt
    this.cacheState.frozenHash = hash
    this.cacheState.cacheMisses++
    return hash
  }

  /**
   * Build session context with dynamic injection on top of the frozen prompt.
   *
   * Injections are key-value pairs (e.g. user name, session ID, time) that
   * change per session. Each injection is cached independently by key and
   * expires after the configured TTL.
   */
  buildSessionContext(
    baseContext: string,
    injections: Record<string, string>
  ): string {
    const now = Date.now()

    // Prune expired entries
    for (const [key, entry] of this.cacheState.sessionContext) {
      if (entry.expiresAt <= now) {
        this.cacheState.sessionContext.delete(key)
      }
    }

    // Upsert injections
    for (const [key, content] of Object.entries(injections)) {
      const existing = this.cacheState.sessionContext.get(key)
      if (existing && existing.content === content) {
        // Same content - extend TTL (cache hit)
        existing.expiresAt = now + this.config.sessionContextTtlMs
        this.cacheState.cacheHits++
      } else {
        // New or changed content
        this.cacheState.sessionContext.set(key, {
          key,
          content,
          tokens: estimateTokens(content),
          createdAt: now,
          expiresAt: now + this.config.sessionContextTtlMs,
        })
        this.cacheState.cacheMisses++
      }
    }

    // Assemble: frozen prompt + base context + dynamic injections
    const parts: string[] = []

    if (this.cacheState.frozenPrompt) {
      parts.push(this.cacheState.frozenPrompt)
    }

    parts.push(baseContext)

    if (this.cacheState.sessionContext.size > 0) {
      parts.push('--- Session Context ---')
      for (const [, entry] of this.cacheState.sessionContext) {
        parts.push(`[${entry.key}]: ${entry.content}`)
      }
    }

    return parts.join('\n')
  }

  // -----------------------------------------------------------------------
  // Insert-then-Compress Strategy
  // -----------------------------------------------------------------------

  /**
   * Check whether compression is needed given the current token count and
   * the configured maximum.
   */
  needsCompression(currentTokens: number, maxTokens?: number): boolean {
    const limit = maxTokens ?? this.config.maxContextTokens
    return currentTokens >= limit * 0.8
  }

  /**
   * Compress context using the insert-then-compress strategy.
   *
   * 1. Keep the last N messages uncompressed (sliding window).
   * 2. Summarize older messages into a compact representation.
   * 3. Archive compressed messages as reference-able chunks.
   */
  compressContext(
    messages: ContextMessage[],
    maxTokens?: number
  ): CompressionResult {
    const limit = maxTokens ?? this.config.maxContextTokens
    const originalTokens = this.totalTokens(messages)

    // If we're under budget, nothing to compress
    if (originalTokens <= limit * 0.8) {
      return {
        originalTokens,
        compressedTokens: originalTokens,
        savedTokens: 0,
        compressionRatio: 1,
        archivedChunks: [],
        summary: '',
      }
    }

    // Split into cold (to compress) and hot (sliding window) messages
    const hotCount = Math.min(this.config.slidingWindowSize, messages.length)
    const coldMessages = messages.slice(0, messages.length - hotCount)
    const hotMessages = messages.slice(messages.length - hotCount)

    // Summarize cold messages
    const summaryText = summarizeMessages(coldMessages)

    // Enforce cold data token budget - truncate summary if needed
    const budgetChars = this.config.coldDataTokenBudget * 4
    const boundedSummary = summaryText.length > budgetChars
      ? summaryText.slice(0, budgetChars) + '\n[...truncated]'
      : summaryText

    // Create archived chunk(s) from the cold messages
    const archivedChunks = this.createArchivedChunks(coldMessages, summaryText)

    // Calculate compressed token count
    const hotTokens = this.totalTokens(hotMessages)
    const compressedTokens = hotTokens + estimateTokens(boundedSummary)
    const savedTokens = originalTokens - compressedTokens

    // Build the summary marker that replaces old messages in context
    const marker = this.buildCompressionMarker(boundedSummary, archivedChunks)

    this.cacheState.lastCompressionAt = Date.now()

    return {
      originalTokens,
      compressedTokens,
      savedTokens,
      compressionRatio: originalTokens > 0
        ? compressedTokens / originalTokens
        : 1,
      archivedChunks,
      summary: marker,
    }
  }

  /**
   * Get archived chunks for AI lookback.
   * The AI can reference these chunks by ID to recall compressed context.
   */
  getArchive(): ArchivedChunk[] {
    return [...this.archive]
  }

  /**
   * Look up a specific archived chunk by its ID.
   */
  getArchivedChunk(id: string): ArchivedChunk | undefined {
    return this.archive.find(c => c.id === id)
  }

  /**
   * Get cache performance statistics.
   */
  getCacheStats(): { hitRate: number; totalSaved: number; archiveSize: number } {
    const total = this.cacheState.cacheHits + this.cacheState.cacheMisses
    const hitRate = total > 0 ? this.cacheState.cacheHits / total : 0

    let totalSaved = 0
    for (const chunk of this.archive) {
      totalSaved += estimateTokens(chunk.compressedSummary)
    }

    return {
      hitRate,
      totalSaved,
      archiveSize: this.archive.length,
    }
  }

  /**
   * Reset the idle timer. Should be called on every user interaction.
   */
  resetIdleTimer(): void {
    this.idleTimer.reset()
  }

  /**
   * Pause the idle timer (e.g. during long-running tool execution).
   */
  pauseIdleTimer(): void {
    this.idleTimer.pause()
  }

  /**
   * Resume the idle timer after a pause.
   */
  resumeIdleTimer(): void {
    this.idleTimer.resume()
  }

  /**
   * Destroy the optimizer and clean up all timers.
   */
  destroy(): void {
    this.idleTimer.destroy()
    this.archive = []
    this.cacheState.sessionContext.clear()
  }

  /**
   * Get the current cache state (for debugging / telemetry).
   */
  getCacheState(): Readonly<CacheState> {
    return { ...this.cacheState, sessionContext: new Map(this.cacheState.sessionContext) }
  }

  // -----------------------------------------------------------------------
  // Private helpers
  // -----------------------------------------------------------------------

  /**
   * Callback invoked when the idle timer fires.
   * This is a hook point - in production it would integrate with the
   * context builder / compactor to proactively compress before cache expiry.
   */
  private onIdleCompress(): void {
    // The idle compression callback is intentionally a no-op at the engine
    // level. The caller (e.g. acp-session-runner) should wire this up to
    // trigger compressContext() on the active message list.
    //
    // We still update the timestamp so callers can detect that idle
    // compression was recommended.
    this.cacheState.lastCompressionAt = Date.now()
  }

  /**
   * Calculate total tokens for a message list.
   */
  private totalTokens(messages: ContextMessage[]): number {
    let total = 0
    for (const m of messages) {
      total += estimateTokens(m.content)
    }
    return total
  }

  /**
   * Create archived chunks from a batch of cold messages.
   * Groups messages into chunks to stay within the max archive size.
   */
  private createArchivedChunks(
    messages: ContextMessage[],
    summary: string
  ): ArchivedChunk[] {
    // Create a single chunk for the entire compressed batch
    this.chunkCounter++
    const chunkId = generateChunkId(this.chunkCounter)
    const now = new Date()

    const chunk: ArchivedChunk = {
      id: chunkId,
      originalMessages: messages.length,
      compressedSummary: summary,
      referencePath: `archive://session/${chunkId}`,
      timestamp: now.toISOString(),
    }

    this.archive.push(chunk)

    // Enforce max archive size by evicting oldest chunks
    while (this.archive.length > this.config.maxArchiveSize) {
      this.archive.shift()
    }

    return [chunk]
  }

  /**
   * Build a compression marker string that replaces old messages in the
   * context. Contains the summary and references to archived chunks.
   */
  private buildCompressionMarker(
    summary: string,
    chunks: ArchivedChunk[]
  ): string {
    const lines: string[] = [
      '=== CONTEXT COMPRESSED ===',
      'The following is a summary of earlier conversation:',
      '',
      summary,
    ]

    if (chunks.length > 0) {
      lines.push('')
      lines.push('Archived references (use referencePath for lookback):')
      for (const chunk of chunks) {
        lines.push(`  [${chunk.id}] ${chunk.originalMessages} messages @ ${chunk.referencePath}`)
      }
    }

    lines.push('')
    lines.push('=== END COMPRESSED CONTEXT ===')

    return lines.join('\n')
  }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/**
 * Create a TokenOptimizer with optional configuration overrides.
 */
export function createTokenOptimizer(
  config?: Partial<TokenOptimizerConfig>
): TokenOptimizer {
  return new TokenOptimizer(config)
}

/**
 * Convert a ChatMessage array to ContextMessage array for use with
 * the TokenOptimizer. Strips fields not needed for compression.
 */
export function toContextMessages(messages: ChatMessage[]): ContextMessage[] {
  return messages.map(m => ({
    id: m.id,
    role: m.role,
    content: m.content,
    timestamp: m.timestamp,
  }))
}
