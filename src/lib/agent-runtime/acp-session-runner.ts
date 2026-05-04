import type { InitializeResponse } from '@agentclientprotocol/sdk'
import { AcpClientBridge, createAcpClient } from '../acp-bridge'
import type { AgentConfig, SavedSession } from '../types'
import { OutputBuffer } from './output-buffer'
import { RuntimeError, assertAbsoluteCwd, toRuntimeError } from './runtime-errors'
import type { RuntimeOutput, RuntimePromptOptions, RuntimeSession, RuntimeConnectionStatus } from './types'

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

    const promptText = options.memories?.length
      ? ['以下是相关记忆：', ...options.memories.map((m) => `- ${m}`), '', options.prompt].join('\n')
      : options.prompt

    const buffer = new OutputBuffer(options.taskId, this.runtimeSession.id, this.runtimeSession.agentName)
    const previousHandler = this.client.onSessionUpdate
    this.client.onSessionUpdate = (notification) => {
      const output = buffer.apply(notification)
      this.options.onOutput?.(output)
      previousHandler?.(notification)
    }

    try {
      await this.client.prompt({
        sessionId: this.runtimeSession.acpSessionId,
        prompt: [{ type: 'text', text: promptText }],
      })
      const output = buffer.complete()
      this.options.onOutput?.(output)
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
