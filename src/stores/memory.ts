import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { isTauriHost } from '../lib/platform'

export interface MemoryRecord {
  id: string
  scope: 'global' | 'agent' | 'session' | 'task'
  agentId: string | null
  sessionId: string | null
  taskId: string | null
  memoryType: 'fact' | 'decision' | 'error' | 'solution' | 'pattern' | 'preference'
  content: string
  tags: string | null
  importance: number
  createdAt: string
  lastAccessed: string | null
  accessCount: number
  expiresAt: string | null
}

export type MemoryScope = MemoryRecord['scope']
export type MemoryType = MemoryRecord['memoryType']

export interface ErrorRecord {
  id: string
  category: string
  message: string
  context: string | null
  stackTrace: string | null
  agentId: string | null
  taskId: string | null
  status: 'open' | 'resolved' | 'ignored'
  solutionId: string | null
  createdAt: string
  resolvedAt: string | null
}

export interface SolutionRecord {
  id: string
  errorId: string | null
  approach: string
  steps: string | null
  result: string
  success: boolean | null
  evidence: string | null
  createdAt: string
}

export interface EvolutionRecord {
  id: string
  evolutionType: 'improvement' | 'regression' | 'discovery'
  domain: string
  before: string | null
  after: string | null
  reason: string
  evidence: string | null
  createdAt: string
}

export interface PatternRecord {
  id: string
  name: string
  description: string
  category: string
  examples: string | null
  successRate: number | null
  usageCount: number
  createdAt: string
  updatedAt: string
}

// Web-side in-memory storage backed by localStorage
const WEB_MEMORIES_KEY = 'acp-ui:memories'

function loadWebMemories(): MemoryRecord[] {
  if (typeof localStorage === 'undefined') return []
  const raw = localStorage.getItem(WEB_MEMORIES_KEY)
  if (!raw) return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

function saveWebMemories(memories: MemoryRecord[]): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(WEB_MEMORIES_KEY, JSON.stringify(memories))
  } catch (e) {
    console.warn('Failed to persist memories:', e)
  }
}

function webLoadAgentMemories(agentId: string | null, limit: number = 100): MemoryRecord[] {
  const all = loadWebMemories()
  let filtered = all
  if (agentId) {
    filtered = all.filter(m => m.agentId === agentId)
  }
  return filtered.slice(0, limit)
}

function webSearchMemories(query: string, agentId: string | null = null, limit: number = 50): MemoryRecord[] {
  const all = loadWebMemories()
  const q = query.toLowerCase()
  let filtered = all.filter(m => m.content.toLowerCase().includes(q))
  if (agentId) {
    filtered = filtered.filter(m => m.agentId === agentId)
  }
  return filtered.slice(0, limit)
}

function webSaveMemory(
  content: string,
  tags: string | null = null,
  scope: MemoryScope = 'global',
  memoryType: MemoryType = 'fact',
  agentId: string | null = null,
  sessionId: string | null = null,
  taskId: string | null = null,
  importance: number = 0.5,
  expiresAt: string | null = null
): string {
  const all = loadWebMemories()
  const id = crypto.randomUUID()
  const record: MemoryRecord = {
    id,
    scope,
    agentId,
    sessionId,
    taskId,
    memoryType,
    content,
    tags,
    importance,
    createdAt: new Date().toISOString(),
    lastAccessed: null,
    accessCount: 0,
    expiresAt,
  }
  all.unshift(record)
  saveWebMemories(all)
  return id
}

function webDeleteMemory(memoryId: string): boolean {
  const all = loadWebMemories()
  const filtered = all.filter(m => m.id !== memoryId)
  if (filtered.length === all.length) return false
  saveWebMemories(filtered)
  return true
}

export const useMemoryStore = defineStore('memory', () => {
  const memories = ref<MemoryRecord[]>([])
  const searchResults = ref<MemoryRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const searchKeyword = ref('')
  const selectedAgentId = ref<string | null>(null)

  const hasResults = computed(() => searchResults.value.length > 0 || memories.value.length > 0)

  async function loadAgentMemories(agentId: string | null) {
    loading.value = true
    error.value = null

    try {
      selectedAgentId.value = agentId
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        if (agentId) {
          const records = await invoke<MemoryRecord[]>('get_agent_memories', { agentId, limit: 100 })
          memories.value = records
        } else {
          const records = await invoke<MemoryRecord[]>('get_shared_memories', { limit: 100 })
          memories.value = records
        }
      } else {
        memories.value = webLoadAgentMemories(agentId)
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function searchMemories(keyword: string, agentId: string | null = null) {
    loading.value = true
    error.value = null
    searchKeyword.value = keyword

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<MemoryRecord[]>('search_memories', {
          query: keyword,
          agentId,
          limit: 50,
        })
        searchResults.value = records
      } else {
        searchResults.value = webSearchMemories(keyword, agentId)
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveMemory(
    content: string,
    tags: string | null = null,
    scope: MemoryScope = 'global',
    memoryType: MemoryType = 'fact',
    agentId: string | null = null,
    sessionId: string | null = null,
    taskId: string | null = null,
    importance: number = 0.5,
    expiresAt: string | null = null
  ) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke<string>('save_memory', {
          content,
          tags,
          scope,
          memoryType,
          agentId,
          sessionId,
          taskId,
          importance,
          expiresAt,
        })
      } else {
        webSaveMemory(content, tags, scope, memoryType, agentId, sessionId, taskId, importance, expiresAt)
      }
      await loadAgentMemories(selectedAgentId.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function deleteMemory(memoryId: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('delete_memory', { memoryId })
      } else {
        webDeleteMemory(memoryId)
      }
      memories.value = memories.value.filter(m => m.id !== memoryId)
      searchResults.value = searchResults.value.filter(m => m.id !== memoryId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  /** Load memories relevant to a given prompt for injection into tasks */
  async function loadRelevantMemories(options: {
    prompt: string
    agentName?: string
    sessionId?: string
    limit?: number
  }): Promise<MemoryRecord[]> {
    const limit = options.limit ?? 5
    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        // Use the first few words of the prompt as search keyword
        const keyword = options.prompt.split(/\s+/).slice(0, 3).join(' ')
        const records = await invoke<MemoryRecord[]>('search_memories', {
          query: keyword,
          agentId: options.agentName ?? null,
          limit,
        })
        return records
      } else {
        const keyword = options.prompt.split(/\s+/).slice(0, 3).join(' ')
        return webSearchMemories(keyword, options.agentName ?? null, limit)
      }
    } catch {
      return []
    }
  }

  return {
    memories,
    searchResults,
    loading,
    error,
    searchKeyword,
    selectedAgentId,
    hasResults,
    loadAgentMemories,
    searchMemories,
    saveMemory,
    deleteMemory,
    clearError,
    loadRelevantMemories,
  }
})
