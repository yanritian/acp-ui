<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '@/locales'
import { usePluginRegistryStore } from '@/stores/plugin-registry'
import { PluginService, type PluginKind, type PluginStats, type PluginExecutionRecord } from '@/lib/plugin-system'

const { t } = useI18n()
const store = usePluginRegistryStore()

const expandedId = ref<string | null>(null)
const stats = ref<Record<string, PluginStats>>({})
const history = ref<Record<string, PluginExecutionRecord[]>>({})
const configEdits = ref<Record<string, string>>({})
const savingConfig = ref<string | null>(null)
const graphView = ref(false)

const kindIcons: Record<PluginKind, string> = {
  skill: '\uD83D\uDD27', mcp: '\uD83D\uDD0C', hook: '\uD83E\uDE9D', cli: '\uD81F\uDCBB', adapter: '\uD83D\uDCE1',
}
const kindFilters: Array<{ label: string; kind: PluginKind | null }> = [
  { label: 'All', kind: null },
  { label: 'Skills', kind: 'skill' },
  { label: 'MCP', kind: 'mcp' },
  { label: 'Hooks', kind: 'hook' },
  { label: 'CLI', kind: 'cli' },
  { label: 'Adapters', kind: 'adapter' },
]

const healthColors: Record<string, string> = {
  healthy: '#10b981', degraded: '#f59e0b', unhealthy: '#ef4444', unknown: '#94a3b8',
}

const filtered = computed(() => store.filteredPlugins)
const totalCount = computed(() => store.plugins.length)
const enabledCount = computed(() => store.enabledCount)
const healthyCount = computed(() => store.healthyCount)

async function toggleEnabled(id: string, current: boolean) {
  await store.togglePlugin(id, !current)
}

function toggleExpand(id: string) {
  if (expandedId.value === id) { expandedId.value = null; return }
  expandedId.value = id
  if (!stats.value[id]) loadStats(id)
  if (!history.value[id]) loadHistory(id)
}

async function loadStats(id: string) {
  try { stats.value[id] = await store.getPluginStats(id) } catch { /* ignore */ }
}

async function loadHistory(id: string) {
  try { history.value[id] = await PluginService.getHistory(id, 10) } catch { /* ignore */ }
}

function initConfigEdit(id: string) {
  const plugin = store.plugins.find(p => p.id === id)
  if (plugin) configEdits.value[id] = JSON.stringify(plugin.config, null, 2)
}

async function saveConfig(id: string) {
  savingConfig.value = id
  try {
    const parsed = JSON.parse(configEdits.value[id] || '{}')
    await PluginService.updateConfig(id, parsed)
    const plugin = store.plugins.find(p => p.id === id)
    if (plugin) plugin.config = parsed
  } catch (e) { console.error('Config save failed:', e) }
  finally { savingConfig.value = null }
}

function formatDuration(ms: number): string {
  return ms < 1000 ? `${ms}ms` : `${(ms / 1000).toFixed(1)}s`
}

function formatTokens(n: number): string {
  return n > 1000 ? `${(n / 1000).toFixed(1)}k` : String(n)
}

onMounted(() => { store.loadPlugins() })
</script>

<template>
  <div class="plugin-manager">
    <header class="pm-header">
      <div class="pm-header-left">
        <h1 class="pm-title">{{ t('pluginManager.title') }}</h1>
        <input
          class="pm-search"
          type="text"
          :placeholder="t('pluginManager.searchPlaceholder')"
          :value="store.searchQuery"
          @input="store.setSearch(($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="pm-header-right">
        <div class="pm-filters">
          <button
            v-for="f in kindFilters" :key="f.label"
            class="filter-btn" :class="{ active: store.filterKind === f.kind }"
            @click="store.setFilter(f.kind)"
          >{{ f.label }}</button>
        </div>
        <button class="btn btn-ghost" @click="graphView = !graphView">
          {{ graphView ? 'List View' : 'Graph View' }}
        </button>
      </div>
    </header>

    <div class="pm-stats-bar">
      <span>{{ totalCount }} {{ t('pluginManager.plugins') }}</span>
      <span class="sep">|</span>
      <span class="enabled">{{ enabledCount }} {{ t('pluginManager.enabled') }}</span>
      <span class="sep">|</span>
      <span class="healthy">{{ healthyCount }} {{ t('pluginManager.healthy') }}</span>
    </div>

    <div v-if="store.loading" class="pm-loading">{{ t('common.loading') }}...</div>
    <div v-else-if="store.error" class="pm-error">{{ store.error }}</div>

    <div v-else class="pm-grid">
      <div v-for="plugin in filtered" :key="plugin.id" class="plugin-card">
        <div class="card-top">
          <span class="plugin-icon">{{ kindIcons[plugin.kind] }}</span>
          <div class="plugin-info">
            <div class="plugin-name">{{ plugin.name }} <span class="plugin-version">v{{ plugin.version }}</span></div>
            <div class="plugin-desc">{{ plugin.description }}</div>
          </div>
          <span class="health-dot" :style="{ background: healthColors[plugin.health_status] || healthColors.unknown }"></span>
        </div>

        <div class="card-actions">
          <div class="toggle" :class="{ active: plugin.enabled }" @click="toggleEnabled(plugin.id, plugin.enabled)">
            <span class="toggle-knob"></span>
          </div>
          <button class="btn btn-sm btn-ghost" @click="toggleExpand(plugin.id)">
            {{ expandedId === plugin.id ? t('common.collapse') : t('pluginManager.details') }}
          </button>
        </div>

        <div v-if="expandedId === plugin.id" class="card-detail">
          <div class="detail-section">
            <h4>{{ t('pluginManager.capabilities') }}</h4>
            <ul class="cap-list">
              <li v-for="cap in plugin.capabilities" :key="cap.name">
                <strong>{{ cap.name }}</strong>: {{ cap.description }}
              </li>
            </ul>
            <div v-if="!plugin.capabilities.length" class="empty-text">{{ t('pluginManager.noCapabilities') }}</div>
          </div>

          <div class="detail-section" v-if="stats[plugin.id]">
            <h4>{{ t('pluginManager.executionStats') }}</h4>
            <div class="stats-row">
              <span>{{ t('pluginManager.successRate') }}: {{ (stats[plugin.id].success_rate * 100).toFixed(1) }}%</span>
              <span>{{ t('pluginManager.avgDuration') }}: {{ formatDuration(stats[plugin.id].average_duration_ms) }}</span>
              <span>{{ t('pluginManager.tokens') }}: {{ formatTokens(stats[plugin.id].total_input_tokens + stats[plugin.id].total_output_tokens) }}</span>
            </div>
          </div>

          <div class="detail-section">
            <h4>{{ t('pluginManager.config') }}</h4>
            <textarea
              class="config-editor"
              :value="configEdits[plugin.id] ?? JSON.stringify(plugin.config, null, 2)"
              @focus="initConfigEdit(plugin.id)"
              @input="configEdits[plugin.id] = ($event.target as HTMLTextAreaElement).value"
              rows="5"
            ></textarea>
            <button
              class="btn btn-sm btn-primary"
              :disabled="savingConfig === plugin.id"
              @click="saveConfig(plugin.id)"
            >{{ savingConfig === plugin.id ? t('common.saving') : t('common.save') }}</button>
          </div>

          <div class="detail-section" v-if="history[plugin.id]?.length">
            <h4>{{ t('pluginManager.recentHistory') }}</h4>
            <div class="history-list">
              <div v-for="rec in history[plugin.id].slice(0, 5)" :key="rec.id" class="history-item">
                <span class="hist-status" :class="rec.response.success ? 'ok' : 'fail'"></span>
                <span>{{ rec.capability }}</span>
                <span class="hist-time">{{ formatDuration(rec.response.duration_ms) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="!store.loading && !filtered.length" class="pm-empty">
      {{ t('pluginManager.noPlugins') }}
    </div>
  </div>
</template>

<style scoped>
.plugin-manager {
  --bg-main: #0f172a;
  --bg-surface: #1e293b;
  --bg-subtle: #334155;
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  --accent: #6366f1;
  --accent-hover: #4f46e5;
  --success: #10b981;
  --warning: #f59e0b;
  --error: #ef4444;
  --border: #334155;
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;

  padding: 20px;
  min-height: 100%;
  background: var(--bg-main);
  color: var(--text-primary);
  font-family: system-ui, -apple-system, sans-serif;
}

.pm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
}
.pm-header-left { display: flex; align-items: center; gap: 12px; }
.pm-header-right { display: flex; align-items: center; gap: 12px; }
.pm-title { font-size: 20px; font-weight: 600; white-space: nowrap; }
.pm-title::before { content: '\uD83E\uDDE9 '; }
.pm-search {
  padding: 6px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  width: 220px;
}
.pm-search:focus { border-color: var(--accent); outline: none; }
.pm-search::placeholder { color: var(--text-muted); }

.pm-filters { display: flex; gap: 4px; }
.filter-btn {
  padding: 4px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-full);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.filter-btn:hover { border-color: var(--accent); color: var(--text-primary); }
.filter-btn.active { background: var(--accent); color: white; border-color: var(--accent); }

.pm-stats-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--bg-surface);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 16px;
}
.pm-stats-bar .sep { color: var(--text-muted); }
.pm-stats-bar .enabled { color: var(--accent); }
.pm-stats-bar .healthy { color: var(--success); }

.pm-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 12px; }

.plugin-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 16px;
  transition: border-color 0.15s;
}
.plugin-card:hover { border-color: var(--accent); }

.card-top { display: flex; align-items: flex-start; gap: 10px; }
.plugin-icon { font-size: 28px; flex-shrink: 0; }
.plugin-info { flex: 1; min-width: 0; }
.plugin-name { font-size: 14px; font-weight: 600; }
.plugin-version { font-weight: 400; color: var(--text-muted); font-size: 12px; }
.plugin-desc { font-size: 12px; color: var(--text-secondary); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.health-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; margin-top: 4px; }

.card-actions { display: flex; align-items: center; justify-content: space-between; margin-top: 12px; }

.toggle {
  width: 36px; height: 20px;
  background: var(--bg-subtle); border-radius: var(--radius-full);
  position: relative; cursor: pointer; transition: background 0.2s;
}
.toggle.active { background: var(--accent); }
.toggle-knob {
  position: absolute; width: 16px; height: 16px;
  background: white; border-radius: 50%; top: 2px; left: 2px;
  transition: transform 0.2s;
}
.toggle.active .toggle-knob { transform: translateX(16px); }

.card-detail { margin-top: 14px; border-top: 1px solid var(--border); padding-top: 14px; }
.detail-section { margin-bottom: 12px; }
.detail-section h4 { font-size: 12px; font-weight: 600; color: var(--accent); margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px; }
.cap-list { margin: 0; padding-left: 16px; font-size: 12px; color: var(--text-secondary); }
.cap-list li { margin-bottom: 4px; }
.cap-list strong { color: var(--text-primary); }

.stats-row { display: flex; gap: 16px; font-size: 12px; color: var(--text-secondary); }

.config-editor {
  width: 100%; padding: 8px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text-primary); font-family: monospace; font-size: 11px;
  resize: vertical; margin-bottom: 6px;
}
.config-editor:focus { border-color: var(--accent); outline: none; }

.history-list { display: flex; flex-direction: column; gap: 4px; }
.history-item { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-secondary); }
.hist-status { width: 6px; height: 6px; border-radius: 50%; }
.hist-status.ok { background: var(--success); }
.hist-status.fail { background: var(--error); }
.hist-time { margin-left: auto; color: var(--text-muted); }

.btn {
  display: inline-flex; align-items: center; gap: 4px;
  padding: 6px 12px; font-size: 13px; font-weight: 500;
  border-radius: var(--radius-md); border: none; cursor: pointer; transition: all 0.15s;
}
.btn-primary { background: var(--accent); color: white; }
.btn-primary:hover { background: var(--accent-hover); }
.btn-ghost { background: transparent; color: var(--text-secondary); }
.btn-ghost:hover { color: var(--text-primary); }
.btn-sm { padding: 4px 10px; font-size: 12px; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.pm-loading, .pm-error, .pm-empty {
  text-align: center; padding: 48px 16px; color: var(--text-muted); font-size: 14px;
}
.pm-error { color: var(--error); }
</style>
