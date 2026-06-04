import type { InitializeResponse } from '@agentclientprotocol/sdk'
import { AcpClientBridge, createAcpClient } from '../acp-bridge'
import type { AgentConfig, SavedSession } from '../types'
import { OutputBuffer } from './output-buffer'
import { RuntimeError, assertAbsoluteCwd, toRuntimeError } from './runtime-errors'
import { BudgetTracker, DEFAULT_BUDGET_LIMITS, createBudgetStopStatus } from './budget-tracker'
import { ContextCompactor, createContextCompactor } from './context-compactor'
import type { RuntimeOutput, RuntimePromptOptions, RuntimeSession, RuntimeConnectionStatus, RuntimeBudgetState, RuntimeCompactionState } from './types'

export interface AcpSessionRunnerOptions {
  agentName: string
  agentConfig: AgentConfig
  cwd: string
  appVersion: string
  onOutput?: (output: RuntimeOutput) => void
  onTransportClose?: (reason?: string) => void
}

export class AcpSessionRunner {
  private client: AcpClientBridge | null = null
  private runtimeSession: RuntimeSession | null = null
  private budgetTracker: BudgetTracker | null = null
  private contextCompactor: ContextCompactor | null = null

  constructor(private readonly options: AcpSessionRunnerOptions) {}

  get session(): RuntimeSession | null {
    return this.runtimeSession
  }

  get acpClient(): AcpClientBridge | null {
    return this.client
  }

  get sessionId(): string | null {
    return this.runtimeSession?.acpSessionId ?? null
  }

  get connectionStatus(): RuntimeConnectionStatus {
    return this.runtimeSession?.status ?? 'idle'
  }

  get budgetState(): RuntimeBudgetState | null {
    if (!this.budgetTracker) return null
    const check = this.budgetTracker.check()
    return {
      limits: this.budgetTracker.getLimits(),
      consumption: this.budgetTracker.getConsumption(),
      exceeded: check.exceeded,
      exceededReason: check.exceeded ? check.reason : undefined,
    }
  }

  get compactionState(): RuntimeCompactionState | null {
    if (!this.contextCompactor) return null
    return {
      lastCompactionTimestamp: this.contextCompactor.getLastCompactionTimestamp(),
      summary: null, // Would be populated after compaction
      rehydrationArtifacts: null,
      pendingCompaction: false,
    }
  }

  async create(): Promise<RuntimeSession> {
    assertAbsoluteCwd(this.options.cwd)
    const initResponse = await this.connectAndInitialize()
    const sessionResponse = await this.client!.newSession({
      cwd: this.options.cwd,
      mcpServers: [],
    })

    this.runtimeSession = {
      id: crypto.randomUUID(),
      agentName: this.options.agentName,
      acpSessionId: sessionResponse.sessionId,
      cwd: this.options.cwd,
      title: `Session ${new Date().toLocaleString()}`,
      supportsLoadSession: initResponse.agentCapabilities?.loadSession ?? false,
      status: 'connected',
      createdAt: Date.now(),
      lastUpdated: Date.now(),
    }

    return this.runtimeSession
  }

  async load(saved: SavedSession): Promise<RuntimeSession> {
    assertAbsoluteCwd(saved.cwd)
    const initResponse = await this.connectAndInitialize()

    try {
      await this.client!.loadSession({
        sessionId: saved.sessionId,
        cwd: saved.cwd,
        mcpServers: [],
      })
    } catch (error) {
      await this.disconnect()
      throw toRuntimeError(error, 'session-not-found')
    }

    this.runtimeSession = {
      id: saved.id,
      agentName: saved.agentName,
      acpSessionId: saved.sessionId,
      cwd: saved.cwd,
      title: saved.title,
      supportsLoadSession: saved.supportsLoadSession ?? initResponse.agentCapabilities?.loadSession ?? false,
      status: 'connected',
      createdAt: saved.lastUpdated,
      lastUpdated: Date.now(),
    }

    return this.runtimeSession
  }

  async prompt(options: RuntimePromptOptions): Promise<RuntimeOutput> {
    if (!this.client || !this.runtimeSession) {
      throw new RuntimeError('session-not-found', '没有可用会话。')
    }

    // Initialize budget tracker for this task
    this.budgetTracker = new BudgetTracker(options.budgetLimits || {})
    this.contextCompactor = createContextCompactor()

    const objective = options.objective || options.prompt.slice(0, 100)

    // Build prompt with compaction summary if available
    let promptText = options.prompt
    if (options.memories?.length) {
      promptText = ['以下是相关记忆：', ...options.memories.map((m) => `- ${m}`), '', options.prompt].join('\n')
    }

    // Check budget before starting
    const budgetCheck = this.budgetTracker.check()
    if (budgetCheck.exceeded) {
      const errorOutput = this.createBudgetErrorOutput(options.taskId, budgetCheck)
      this.options.onOutput?.(errorOutput)
      return errorOutput
    }

    const buffer = new OutputBuffer(options.taskId, this.runtimeSession.id, this.runtimeSession.agentName)
    const previousHandler = this.client.onSessionUpdate

    // Track tool calls for budget
    let toolCallCount = 0
    let toolResultChars = 0

    this.client.onSessionUpdate = (notification) => {
      const output = buffer.apply(notification)

      // Track tool calls in budget
      if (output.toolCalls.length > toolCallCount) {
        const newCalls = output.toolCalls.length - toolCallCount
        this.budgetTracker?.recordToolCalls(newCalls, 0)
        toolCallCount = output.toolCalls.length
      }

      // Track output size
      toolResultChars += output.content.length
      this.budgetTracker?.recordToolResultSize(output.content.length)

      // Check for compaction need
      const messages = output.messages
      const estimatedTokens = this.contextCompactor?.estimateTokens(messages) || 0
      if (this.contextCompactor?.needsCompaction(messages, output.toolCalls, estimatedTokens).needed) {
        // In a full implementation, we would pause and compact here
        // For MVP, we just log and continue
        console.warn('Context approaching limit, compaction recommended')
      }

      this.options.onOutput?.(output)
      previousHandler?.(notification)
    }

    try {
      // Record step
      this.budgetTracker.recordStep()

      await this.client.prompt({
        sessionId: this.runtimeSession.acpSessionId,
        prompt: [{ type: 'text', text: promptText }],
      })

      // Final budget check
      const finalCheck = this.budgetTracker.check()
      if (finalCheck.exceeded) {
        const errorOutput = this.createBudgetErrorOutput(options.taskId, finalCheck)
        this.options.onOutput?.(errorOutput)
        return errorOutput
      }

      const output = buffer.complete()
      this.options.onOutput?.(output)

      // Record final consumption - estimate tokens from output
      const finalEstimatedTokens = this.contextCompactor?.estimateTokens(output.messages) ?? output.content.length / 4
      this.budgetTracker.recordTokens(finalEstimatedTokens, output.content.length / 4)

      return output
    } catch (error) {
      const runtimeError = toRuntimeError(error, 'task-failed')
      const output = buffer.fail(runtimeError.message)
      this.options.onOutput?.(output)
      throw runtimeError
    } finally {
      this.client.onSessionUpdate = previousHandler
      this.runtimeSession.lastUpdated = Date.now()
    }
  }

  private createBudgetErrorOutput(taskId: string, budgetResult: { exceeded: true; reason: string; limitName: string; current: number; limit: number; nextSafeAction: string }): RuntimeOutput {
    return {
      taskId,
      sessionId: this.runtimeSession?.id || '',
      agentName: this.runtimeSession?.agentName || '',
      content: '',
      thought: '',
      messages: [{
        id: crypto.randomUUID(),
        role: 'assistant',
        content: `⚠️ Budget exceeded: ${budgetResult.reason}\n\nCurrent: ${budgetResult.current}, Limit: ${budgetResult.limit}\n\n${budgetResult.nextSafeAction}`,
        timestamp: Date.now(),
        toolCalls: [],
      }],
      toolCalls: [],
      status: 'stopped_budget_exceeded',
      error: budgetResult.reason,
    }
  }

  async cancel(): Promise<void> {
    if (!this.client || !this.runtimeSession) return
    await this.client.cancel({ sessionId: this.runtimeSession.acpSessionId })
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.disconnect()
      this.client = null
    }
    if (this.runtimeSession) {
      this.runtimeSession.status = 'disconnected'
    }
  }

  private async connectAndInitialize(): Promise<InitializeResponse> {
    this.client = await createAcpClient({
      name: this.options.agentName,
      config: this.options.agentConfig,
    })

    this.client.onTransportClose = (reason) => {
      if (this.runtimeSession) this.runtimeSession.status = 'disconnected'
      this.options.onTransportClose?.(reason)
    }

    return await this.client.initialize({
      protocolVersion: 1,
      clientCapabilities: {
        fs: {
          readTextFile: true,
          writeTextFile: true,
        },
      },
      clientInfo: {
        name: 'acp-ui',
        title: 'ACP UI',
        version: this.options.appVersion,
      },
    })
  }
}
