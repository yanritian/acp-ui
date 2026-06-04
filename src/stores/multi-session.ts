import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { loadKvStore, type KVStore } from '../lib/host/storage'
import { useConfigStore } from '../stores/config'
import { useHistoryStore } from '../stores/history'
import type { TaskRecord } from '../lib/storage/history-store'
import type {
  SavedSession,
  ChatMessage,
  ToolCallInfo,
  SessionMode,
  SlashCommand,
  ModelInfo,
} from '../lib/types'
import { AcpSessionRunner } from '../lib/agent-runtime/acp-session-runner'
import { getAppVersion } from '../lib/host'
import type { RuntimeOutput } from '../lib/agent-runtime/types'

const STORE_PATH = 'multi-sessions.json'
const MESSAGES_STORE_PATH = 'session-messages.json'

export interface SessionState {
  id: string
  agentName: string
  cwd: string
  acpSessionId: string | null
  status: 'connecting' | 'connected' | 'disconnected' | 'error'
  messages: ChatMessage[]
  toolCalls: Map<string, ToolCallInfo>
  isConnected: boolean
  isLoading: boolean
  isConnecting: boolean
  error: string | null
  availableModes: SessionMode[]
  currentModeId: string
  availableCommands: SlashCommand[]
  availableModels: ModelInfo[]
  currentModelId: string
  savedSession: SavedSession | null
  title: string
}

function createEmptySession(id: string, agentName: string, cwd: string): SessionState {
  return {
    id,
    agentName,
    cwd,
    acpSessionId: null,
    status: 'disconnected',
    messages: [],
    toolCalls: new Map(),
    isConnected: false,
    isLoading: false,
    isConnecting: false,
    error: null,
    availableModes: [],
    currentModeId: '',
    availableCommands: [],
    availableModels: [],
    currentModelId: '',
    savedSession: null,
    title: agentName,
  }
}

// Non-reactive map for AcpSessionRunner instances (avoid Vue reactivity proxy issues)
const runnerInstances = new Map<string, AcpSessionRunner>()

export const useMultiSessionStore = defineStore('multiSession', () => {
  const sessions = ref<Map<string, SessionState>>(new Map())
  const activeSessionId = ref<string | null>(null)
  const savedSessionsMeta = ref<SavedSession[]>([])

  let store: KVStore | null = null
  let messagesStore: KVStore | null = null

  const activeSession = computed(() => {
    if (!activeSessionId.value) return null
    return sessions.value.get(activeSessionId.value) || null
  })

  const sessionList = computed(() => Array.from(sessions.value.values()))

  const isConnected = computed(() => activeSession.value?.isConnected ?? false)
  const isLoading = computed(() => activeSession.value?.isLoading ?? false)
  const isConnecting = computed(() => activeSession.value?.isConnecting ?? false)
  const error = computed(() => activeSession.value?.error ?? null)
  const messages = computed(() => activeSession.value?.messages ?? [])
  const availableModes = computed(() => activeSession.value?.availableModes ?? [])
  const currentModeId = computed(() => activeSession.value?.currentModeId ?? '')
  const availableCommands = computed(() => activeSession.value?.availableCommands ?? [])
  const availableModels = computed(() => activeSession.value?.availableModels ?? [])
  const currentModelId = computed(() => activeSession.value?.currentModelId ?? '')
  const hasActiveSession = computed(() => activeSession.value !== null)

  async function initStore() {
    store = await loadKvStore(STORE_PATH)
    const saved = await store.get<SavedSession[]>('sessions')
    if (saved) {
      savedSessionsMeta.value = saved
    }

    // Load messages store
    messagesStore = await loadKvStore(MESSAGES_STORE_PATH)
  }

  async function saveMeta() {
    if (store) {
      await store.set('sessions', savedSessionsMeta.value)
      await store.save()
    }
  }

  async function saveMessages(sessionId: string, msgs: ChatMessage[]) {
    if (messagesStore) {
      await messagesStore.set(sessionId, msgs)
      await messagesStore.save()
    }
  }

  async function loadMessages(sessionId: string): Promise<ChatMessage[]> {
    if (messagesStore) {
      const saved = await messagesStore.get<ChatMessage[]>(sessionId)
      return saved ?? []
    }
    return []
  }

  async function createSession(agentName: string, cwd: string): Promise<string> {
    const sessionId = crypto.randomUUID()
    const session = createEmptySession(sessionId, agentName, cwd)
    sessions.value.set(sessionId, session)
    activeSessionId.value = sessionId

    await connectSession(sessionId)
    return sessionId
  }

  async function connectSession(sessionId: string) {
    const session = sessions.value.get(sessionId)
    if (!session) return

    session.status = 'connecting'
    session.isConnecting = true
    session.error = null

    try {
      const configStore = useConfigStore()
      const agentConfig = configStore.config.agents[session.agentName]

      if (!agentConfig) {
        throw new Error(`Agent '${session.agentName}' not found in config`)
      }

      const appVersion = await getAppVersion()
      const runner = new AcpSessionRunner({
        agentName: session.agentName,
        agentConfig,
        cwd: session.cwd,
        appVersion,
        onOutput: (output: RuntimeOutput) => {
          applyOutputToSession(sessionId, output)
        },
        onTransportClose: (reason?: string) => {
          handleUnexpectedClose(sessionId, reason)
        },
      })

      runnerInstances.set(sessionId, runner)

      const runtimeSession = await runner.create()
      session.acpSessionId = runtimeSession.acpSessionId

      session.isConnected = true
      session.isConnecting = false
      session.status = 'connected'

      // Save session meta
      const meta: SavedSession = {
        id: sessionId,
        agentName: session.agentName,
        sessionId: runtimeSession.acpSessionId,
        title: session.title,
        lastUpdated: Date.now(),
        cwd: session.cwd,
        supportsLoadSession: runtimeSession.supportsLoadSession,
      }
      session.savedSession = meta

      const existingIndex = savedSessionsMeta.value.findIndex(s => s.id === sessionId)
      if (existingIndex >= 0) {
        savedSessionsMeta.value[existingIndex] = meta
      } else {
        savedSessionsMeta.value.unshift(meta)
      }
      await saveMeta()
    } catch (e) {
      session.status = 'error'
      session.isConnecting = false
      session.error = e instanceof Error ? e.message : String(e)
    }
  }

  async function switchSession(sessionId: string) {
    if (sessions.value.has(sessionId)) {
      activeSessionId.value = sessionId
      // Load persisted messages for this session
      const session = sessions.value.get(sessionId)
      if (session && session.messages.length === 0) {
        const savedMsgs = await loadMessages(sessionId)
        if (savedMsgs.length > 0) {
          session.messages = savedMsgs
        }
      }
    }
  }

  async function closeSession(sessionId: string) {
    const runner = runnerInstances.get(sessionId)
    if (runner) {
      await runner.disconnect()
      runnerInstances.delete(sessionId)
    }

    sessions.value.delete(sessionId)
    if (activeSessionId.value === sessionId) {
      const next = sessions.value.keys().next()
      activeSessionId.value = next.done ? null : next.value
    }

    savedSessionsMeta.value = savedSessionsMeta.value.filter(s => s.id !== sessionId)

    // Clean up messages store for this session
    if (messagesStore) {
      await messagesStore.set(sessionId, undefined)
      await messagesStore.save()
    }

    try {
      await saveMeta()
    } catch (e) {
      console.warn('Failed to persist session metadata:', e)
    }
  }

  async function sendPrompt(text: string) {
    const session = activeSession.value
    if (!session) return
    const runner = runnerInstances.get(session.id)
    if (!runner) return

    session.isLoading = true
    session.error = null

    const taskId = crypto.randomUUID()
    const startTime = Date.now()

    // Add user message immediately for instant feedback
    const userMsgId = crypto.randomUUID()
    session.messages.push({
      id: userMsgId,
      role: 'user',
      content: text,
      timestamp: startTime,
    })

    let output: RuntimeOutput | null = null
    try {
      output = await runner.prompt({
        taskId,
        prompt: text,
        source: 'multi-session',
      })
    } catch (e) {
      session.error = e instanceof Error ? e.message : String(e)
      // Rollback: remove user message since it wasn't actually sent
      session.messages = session.messages.filter(m => m.id !== userMsgId)
    } finally {
      session.isLoading = false
    }

    // Save to history after completion
    if (output) {
      try {
        const historyStore = useHistoryStore()
        const record: TaskRecord = {
          id: taskId,
          name: text.slice(0, 50) + (text.length > 50 ? '...' : ''),
          status: output.status === 'completed' ? 'success' : output.status === 'failed' ? 'failed' : 'running',
          createdAt: startTime,
          completedAt: output.status === 'completed' || output.status === 'failed' ? Date.now() : undefined,
          source: 'app',
          agents: [{
            agentId: session.id,
            agentName: session.agentName,
            role: 'executor',
            status: output.status,
            startTime,
            endTime: output.status !== 'running' ? Date.now() : undefined,
            outputFiles: [],
          }],
          conversations: output.messages.map(m => ({
            timestamp: m.timestamp,
            speaker: m.role,
            message: m.content,
            toolCalls: m.toolCalls,
          })),
          outputs: [{
            type: 'message',
            content: output.content,
            agentId: session.id,
          }],
          syncRecords: [],
          error: output.error ? {
            message: output.error,
            timestamp: Date.now(),
          } : undefined,
        }
        await historyStore.loadRecords()
        // Use internal store save method
        const store = historyStore.records
        // Direct save via the internal HistoryStore instance
        const { getHistoryStore } = await import('../lib/storage/history-store')
        const internalStore = getHistoryStore()
        await internalStore.saveTask(record)
      } catch (e) {
        console.warn('Failed to save task to history:', e)
      }
    }
  }

  function applyOutputToSession(sessionId: string, output: RuntimeOutput) {
    const session = sessions.value.get(sessionId)
    if (!session) return

    // Merge messages by ID: append new ones, update existing ones
    // Skip user messages — they're already added by sendPrompt before calling the runner
    for (const msg of output.messages) {
      if (msg.role === 'user') continue

      const existing = session.messages.find(m => m.id === msg.id)
      if (existing) {
        if (msg.content !== existing.content) {
          existing.content = msg.content
        }
        if (msg.thought && msg.thought !== existing.thought) {
          existing.thought = msg.thought
        }
      } else {
        session.messages.push(msg)
      }
    }

    // Merge toolCalls
    for (const tc of output.toolCalls) {
      const existing = session.toolCalls.get(tc.toolCallId)
      if (!existing) {
        session.toolCalls.set(tc.toolCallId, tc)
      }
    }

    // Persist messages after each update
    saveMessages(sessionId, session.messages).catch(e => {
      console.warn('Failed to persist messages:', e)
    })
  }

  function handleUnexpectedClose(sessionId: string, reason?: string) {
    runnerInstances.delete(sessionId)
    const session = sessions.value.get(sessionId)
    if (!session) return
    session.isConnected = false
    session.isLoading = false
    session.status = 'disconnected'
    session.error = `Connection lost: ${reason ?? 'transport closed'}`
  }

  async function disconnect() {
    const session = activeSession.value
    if (!session) return

    const runner = runnerInstances.get(session.id)
    if (runner) {
      await runner.disconnect()
      runnerInstances.delete(session.id)
    }
    session.isConnected = false
    session.status = 'disconnected'
  }

  function clearError() {
    const session = activeSession.value
    if (session) session.error = null
  }

  return {
    sessions,
    activeSessionId,
    activeSession,
    sessionList,
    savedSessionsMeta,
    isConnected,
    isLoading,
    isConnecting,
    error,
    messages,
    availableModes,
    currentModeId,
    availableCommands,
    availableModels,
    currentModelId,
    hasActiveSession,
    initStore,
    createSession,
    switchSession,
    closeSession,
    sendPrompt,
    disconnect,
    clearError,
  }
})
