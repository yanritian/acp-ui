import { defineStore } from 'pinia'
import { ref } from 'vue'
import { isTauriHost } from '../lib/platform'
import type { ErrorRecord, SolutionRecord, EvolutionRecord, PatternRecord } from './memory'

// Web-side localStorage storage for errors/solutions/evolutions/patterns
const WEB_ERRORS_KEY = 'acp-ui:errors'
const WEB_SOLUTIONS_KEY = 'acp-ui:solutions'
const WEB_EVOLUTIONS_KEY = 'acp-ui:evolutions'
const WEB_PATTERNS_KEY = 'acp-ui:patterns'

function loadWebRecords<T>(key: string): T[] {
  if (typeof localStorage === 'undefined') return []
  const raw = localStorage.getItem(key)
  if (!raw) return []
  try {
    return JSON.parse(raw)
  } catch {
    return []
  }
}

function saveWebRecords<T>(key: string, records: T[]): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(key, JSON.stringify(records))
  } catch (e) {
    console.warn(`Failed to persist ${key}:`, e)
  }
}

export const useErrorStore = defineStore('error', () => {
  const errors = ref<ErrorRecord[]>([])
  const solutions = ref<SolutionRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadErrors(status?: string, category?: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<ErrorRecord[]>('get_errors', {
          status: status ?? null,
          category: category ?? null,
          limit: 100,
        })
        errors.value = records
      } else {
        let filtered = loadWebRecords<ErrorRecord>(WEB_ERRORS_KEY)
        if (status) filtered = filtered.filter(e => e.status === status)
        if (category) filtered = filtered.filter(e => e.category === category)
        errors.value = filtered
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveError(
    category: string,
    message: string,
    context?: string,
    stackTrace?: string,
    agentId?: string,
    taskId?: string
  ): Promise<string> {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const id = await invoke<string>('save_error', {
          category,
          message,
          context: context ?? null,
          stackTrace: stackTrace ?? null,
          agentId: agentId ?? null,
          taskId: taskId ?? null,
        })
        await loadErrors()
        return id
      } else {
        const all = loadWebRecords<ErrorRecord>(WEB_ERRORS_KEY)
        const id = crypto.randomUUID()
        const record: ErrorRecord = {
          id,
          category,
          message,
          context: context ?? null,
          stackTrace: stackTrace ?? null,
          agentId: agentId ?? null,
          taskId: taskId ?? null,
          status: 'open',
          solutionId: null,
          createdAt: new Date().toISOString(),
          resolvedAt: null,
        }
        all.unshift(record)
        saveWebRecords(WEB_ERRORS_KEY, all)
        errors.value = all
        return id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function resolveError(errorId: string, solutionId: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('resolve_error', { errorId, solutionId })
      } else {
        const all = loadWebRecords<ErrorRecord>(WEB_ERRORS_KEY)
        const idx = all.findIndex(e => e.id === errorId)
        if (idx >= 0) {
          all[idx].status = 'resolved'
          all[idx].solutionId = solutionId
          all[idx].resolvedAt = new Date().toISOString()
          saveWebRecords(WEB_ERRORS_KEY, all)
        }
      }
      await loadErrors()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadSolutions(errorId?: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<SolutionRecord[]>('get_solutions', {
          errorId: errorId ?? null,
          limit: 100,
        })
        solutions.value = records
      } else {
        let filtered = loadWebRecords<SolutionRecord>(WEB_SOLUTIONS_KEY)
        if (errorId) filtered = filtered.filter(s => s.errorId === errorId)
        solutions.value = filtered
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveSolution(
    approach: string,
    result: string,
    errorId?: string,
    steps?: string,
    success?: boolean,
    evidence?: string
  ): Promise<string> {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const id = await invoke<string>('save_solution', {
          errorId: errorId ?? null,
          approach,
          steps: steps ?? null,
          result,
          success: success ?? null,
          evidence: evidence ?? null,
        })
        await loadSolutions()
        return id
      } else {
        const all = loadWebRecords<SolutionRecord>(WEB_SOLUTIONS_KEY)
        const id = crypto.randomUUID()
        const record: SolutionRecord = {
          id,
          errorId: errorId ?? null,
          approach,
          steps: steps ?? null,
          result,
          success: success ?? null,
          evidence: evidence ?? null,
          createdAt: new Date().toISOString(),
        }
        all.unshift(record)
        saveWebRecords(WEB_SOLUTIONS_KEY, all)
        solutions.value = all
        return id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    errors,
    solutions,
    loading,
    error,
    loadErrors,
    saveError,
    resolveError,
    loadSolutions,
    saveSolution,
    clearError,
  }
})

export const useEvolutionStore = defineStore('evolution', () => {
  const evolutions = ref<EvolutionRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadEvolutions(type?: string, domain?: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<EvolutionRecord[]>('get_evolutions', {
          evolutionType: type ?? null,
          domain: domain ?? null,
          limit: 100,
        })
        evolutions.value = records
      } else {
        let filtered = loadWebRecords<EvolutionRecord>(WEB_EVOLUTIONS_KEY)
        if (type) filtered = filtered.filter(e => e.evolutionType === type)
        if (domain) filtered = filtered.filter(e => e.domain === domain)
        evolutions.value = filtered
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveEvolution(
    evolutionType: string,
    domain: string,
    reason: string,
    before?: string,
    after?: string,
    evidence?: string
  ): Promise<string> {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const id = await invoke<string>('save_evolution', {
          evolutionType,
          domain,
          before: before ?? null,
          after: after ?? null,
          reason,
          evidence: evidence ?? null,
        })
        await loadEvolutions()
        return id
      } else {
        const all = loadWebRecords<EvolutionRecord>(WEB_EVOLUTIONS_KEY)
        const id = crypto.randomUUID()
        const record: EvolutionRecord = {
          id,
          evolutionType: evolutionType as 'improvement' | 'regression' | 'discovery',
          domain,
          before: before ?? null,
          after: after ?? null,
          reason,
          evidence: evidence ?? null,
          createdAt: new Date().toISOString(),
        }
        all.unshift(record)
        saveWebRecords(WEB_EVOLUTIONS_KEY, all)
        evolutions.value = all
        return id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    evolutions,
    loading,
    error,
    loadEvolutions,
    saveEvolution,
    clearError,
  }
})

export const usePatternStore = defineStore('pattern', () => {
  const patterns = ref<PatternRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadPatterns(category?: string) {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const records = await invoke<PatternRecord[]>('get_patterns', {
          category: category ?? null,
          limit: 100,
        })
        patterns.value = records
      } else {
        let filtered = loadWebRecords<PatternRecord>(WEB_PATTERNS_KEY)
        if (category) filtered = filtered.filter(p => p.category === category)
        patterns.value = filtered
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function savePattern(
    name: string,
    description: string,
    category: string,
    examples?: string
  ): Promise<string> {
    loading.value = true
    error.value = null

    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const id = await invoke<string>('save_pattern', {
          name,
          description,
          category,
          examples: examples ?? null,
        })
        await loadPatterns()
        return id
      } else {
        const all = loadWebRecords<PatternRecord>(WEB_PATTERNS_KEY)
        const id = crypto.randomUUID()
        const now = new Date().toISOString()
        const record: PatternRecord = {
          id,
          name,
          description,
          category,
          examples: examples ?? null,
          successRate: null,
          usageCount: 0,
          createdAt: now,
          updatedAt: now,
        }
        all.unshift(record)
        saveWebRecords(WEB_PATTERNS_KEY, all)
        patterns.value = all
        return id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updatePatternUsage(patternId: string, success: boolean) {
    try {
      if (isTauriHost()) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('update_pattern_usage', { patternId, success })
      } else {
        const all = loadWebRecords<PatternRecord>(WEB_PATTERNS_KEY)
        const idx = all.findIndex(p => p.id === patternId)
        if (idx >= 0) {
          all[idx].usageCount++
          const currentRate = all[idx].successRate ?? 0.5
          all[idx].successRate = success
            ? (currentRate * all[idx].usageCount + 1) / (all[idx].usageCount + 1)
            : (currentRate * all[idx].usageCount) / (all[idx].usageCount + 1)
          all[idx].updatedAt = new Date().toISOString()
          saveWebRecords(WEB_PATTERNS_KEY, all)
        }
      }
      await loadPatterns()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    patterns,
    loading,
    error,
    loadPatterns,
    savePattern,
    updatePatternUsage,
    clearError,
  }
})