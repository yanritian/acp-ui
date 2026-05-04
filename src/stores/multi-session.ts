import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { loadKvStore, type KVStore } from '../lib/host/storage'
import { useConfigStore } from '../stores/config'
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
  }

  async function saveMeta() {
    if (store) {
      await store.set('sessions', savedSessionsMeta.value)
      await store.save()
    }
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

  function switchSession(sessionId: string) {
    if (sessions.value.has(sessionId)) {
      activeSessionId.value = sessionId
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
    await saveMeta()
  }

  async function sendPrompt(text: string) {
    const session = activeSession.value
    if (!session) return
    const runner = runnerInstances.get(session.id)
    if (!runner) return

    session.isLoading = true
    session.error = null

    // Add user message immediately for instant feedback
    session.messages.push({
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: Date.now(),
    })

    try {
      await runner.prompt({
        taskId: crypto.randomUUID(),
        prompt: text,
        source: 'multi-session',
      })
    } catch (e) {
      session.error = e instanceof Error ? e.message : String(e)
    } finally {
      session.isLoading = false
    }
  }

  function applyOutputToSession(sessionId: string, output: RuntimeOutput) {
    const session = sessions.value.get(sessionId)
    if (!session) return

    // Sync messages from output
    session.messages = output.messages
    session.toolCalls.clear()
    for (const tc of output.toolCalls) {
      session.toolCalls.set(tc.toolCallId, tc)
    }
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
