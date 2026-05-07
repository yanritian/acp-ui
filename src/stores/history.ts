// History Store - Pinia store for history state
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { isTauriHost } from '../lib/platform'
import { getHistoryStore, type TaskRecord, type HistoryFilter, type TaskStatistics } from '../lib/storage/history-store'
import { getTauriHistoryStore } from '../lib/storage/history-store-tauri'

function getActiveStore() {
  return isTauriHost() ? getTauriHistoryStore() : getHistoryStore()
}

export const useHistoryStore = defineStore('history', () => {
  // State
  const records = ref<TaskRecord[]>([])
  const currentDetail = ref<TaskRecord | null>(null)
  const statistics = ref<TaskStatistics | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Filter state
  const filter = ref<HistoryFilter>({
    limit: 50,
  })

  // Computed
  const recentTasks = computed(() => records.value.slice(0, 10))
  const successTasks = computed(() =>
    records.value.filter(r => r.status === 'success')
  )
  const failedTasks = computed(() =>
    records.value.filter(r => r.status === 'failed')
  )
  const todayTasks = computed(() => {
    const todayStart = new Date().setHours(0, 0, 0, 0)
    return records.value.filter(r => r.createdAt >= todayStart)
  })

  // Actions
  async function loadRecords(newFilter?: HistoryFilter) {
    loading.value = true
    error.value = null

    try {
      if (newFilter) {
        filter.value = newFilter
      }

      const store = getActiveStore()
      records.value = await store.queryTasks(filter.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadTaskDetail(taskId: string) {
    loading.value = true
    error.value = null

    try {
      const store = getActiveStore()
      currentDetail.value = await store.getTaskDetail(taskId) ?? null
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadStatistics() {
    const store = getActiveStore()
    statistics.value = await store.getStatistics()
  }

  async function search(keyword: string) {
    loading.value = true
    error.value = null

    try {
      const store = getActiveStore()
      records.value = await store.search(keyword)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function exportHistory(format: 'json' | 'csv' | 'markdown'): Promise<string> {
    const store = getActiveStore()
    return await store.export(format)
  }

  async function deleteRecord(taskId: string) {
    const store = getActiveStore()
    await store.deleteTask(taskId)
    records.value = records.value.filter(r => r.id !== taskId)
  }

  async function clearAll() {
    const store = getActiveStore()
    await store.clear()
    records.value = []
    statistics.value = null
  }

  function setFilter(newFilter: Partial<HistoryFilter>) {
    filter.value = { ...filter.value, ...newFilter }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    records,
    currentDetail,
    statistics,
    loading,
    error,
    filter,

    // Computed
    recentTasks,
    successTasks,
    failedTasks,
    todayTasks,

    // Actions
    loadRecords,
    loadTaskDetail,
    loadStatistics,
    search,
    exportHistory,
    deleteRecord,
    clearAll,
    setFilter,
    clearError,
  }
})