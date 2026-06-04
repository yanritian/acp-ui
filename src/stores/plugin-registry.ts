import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { PluginService, type PluginMeta, type PluginKind, type PluginStats } from '@/lib/plugin-system/plugin-service'

export const usePluginRegistryStore = defineStore('plugin-registry', () => {
  const plugins = ref<PluginMeta[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')
  const filterKind = ref<PluginKind | null>(null)

  // Computed
  const filteredPlugins = computed(() => {
    let result = plugins.value
    if (filterKind.value) {
      result = result.filter(p => p.kind === filterKind.value)
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      result = result.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.capabilities.some(c => c.name.toLowerCase().includes(q))
      )
    }
    return result
  })

  const skillPlugins = computed(() => plugins.value.filter(p => p.kind === 'skill'))
  const mcpPlugins = computed(() => plugins.value.filter(p => p.kind === 'mcp'))
  const hookPlugins = computed(() => plugins.value.filter(p => p.kind === 'hook'))
  const cliPlugins = computed(() => plugins.value.filter(p => p.kind === 'cli'))
  const adapterPlugins = computed(() => plugins.value.filter(p => p.kind === 'adapter'))

  const healthyCount = computed(() => plugins.value.filter(p => p.health_status === 'healthy').length)
  const enabledCount = computed(() => plugins.value.filter(p => p.enabled).length)

  // Actions
  async function loadPlugins() {
    loading.value = true
    error.value = null
    try {
      plugins.value = await PluginService.list()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function registerPlugin(meta: PluginMeta) {
    try {
      await PluginService.register(meta)
      await loadPlugins()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function unregisterPlugin(id: string) {
    try {
      await PluginService.unregister(id)
      await loadPlugins()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function togglePlugin(id: string, enabled: boolean) {
    try {
      await PluginService.setEnabled(id, enabled)
      const plugin = plugins.value.find(p => p.id === id)
      if (plugin) plugin.enabled = enabled
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function getPluginStats(id: string): Promise<PluginStats> {
    return PluginService.getStats(id)
  }

  function setFilter(kind: PluginKind | null) {
    filterKind.value = kind
  }

  function setSearch(query: string) {
    searchQuery.value = query
  }

  return {
    plugins, loading, error, searchQuery, filterKind,
    filteredPlugins, skillPlugins, mcpPlugins, hookPlugins, cliPlugins, adapterPlugins,
    healthyCount, enabledCount,
    loadPlugins, registerPlugin, unregisterPlugin, togglePlugin, getPluginStats,
    setFilter, setSearch,
  }
})
