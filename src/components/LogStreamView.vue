<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from '@/locales';

const { t } = useI18n();

interface LogEntry {
  id: string;
  agentId: string;
  logType: string;
  content: string;
  timestamp: string;
  source: string;
  metadata?: string;
}

const emit = defineEmits<{
  close: [];
  resize: [height: number];
}>();

// State
const logs = ref<LogEntry[]>([]);
const filteredLogs = computed(() => {
  let result = logs.value;

  // Filter by type
  if (selectedType.value !== 'all') {
    result = result.filter(log => log.logType === selectedType.value);
  }

  // Filter by agent
  if (selectedAgent.value !== 'all') {
    result = result.filter(log => log.agentId === selectedAgent.value);
  }

  // Search keyword
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    result = result.filter(log => log.content.toLowerCase().includes(keyword));
  }

  return result;
});

const selectedType = ref<string>('all');
const selectedAgent = ref<string>('all');
const searchKeyword = ref<string>('');
const isPaused = ref(false);
const autoScroll = ref(true);
const logContainer = ref<HTMLElement | null>(null);

// Resize handling
const isResizing = ref(false);
const panelHeight = ref(280);
const MIN_HEIGHT = 150;
const MAX_HEIGHT = window.innerHeight * 0.7;

// Virtual scroll state
const scrollTop = ref(0);
const ITEM_HEIGHT = 32;
const BUFFER_SIZE = 10;

const visibleRange = computed(() => {
  const startIndex = Math.max(0, Math.floor(scrollTop.value / ITEM_HEIGHT) - BUFFER_SIZE);
  const endIndex = Math.min(
    filteredLogs.value.length,
    Math.ceil(scrollTop.value / ITEM_HEIGHT) + Math.ceil(panelHeight.value / ITEM_HEIGHT) + BUFFER_SIZE
  );
  return { startIndex, endIndex };
});

const visibleLogs = computed(() => {
  return filteredLogs.value.slice(visibleRange.value.startIndex, visibleRange.value.endIndex);
});

const totalHeight = computed(() => filteredLogs.value.length * ITEM_HEIGHT);

function startResize(e: MouseEvent) {
  isResizing.value = true;
  e.preventDefault();
  document.addEventListener('mousemove', doResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'ns-resize';
  document.body.style.userSelect = 'none';
}

function doResize(e: MouseEvent) {
  if (!isResizing.value) return;
  const newHeight = window.innerHeight - e.clientY;
  panelHeight.value = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, newHeight));
  emit('resize', panelHeight.value);
}

function stopResize() {
  isResizing.value = false;
  document.removeEventListener('mousemove', doResize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }) + '.' + String(date.getMilliseconds()).padStart(3, '0');
}

function getTypeIcon(type: string): string {
  switch (type) {
    case 'compile': return '🔨';
    case 'debug': return '🔍';
    case 'network': return '🌐';
    case 'error': return '❌';
    case 'warning': return '⚠️';
    case 'info': return 'ℹ️';
    case 'system': return '⚙️';
    default: return '📄';
  }
}

function highlightKeyword(content: string): string {
  if (!searchKeyword.value) return content;
  const keyword = searchKeyword.value;
  const regex = new RegExp(`(${keyword})`, 'gi');
  return content.replace(regex, '<mark>$1</mark>');
}

function handleScroll() {
  if (!logContainer.value) return;
  scrollTop.value = logContainer.value.scrollTop;
  const { scrollTop: st, scrollHeight, clientHeight } = logContainer.value;
  autoScroll.value = st + clientHeight >= scrollHeight - 50;
}

async function loadLogs() {
  try {
    const result = await invoke<LogEntry[]>('get_logs', { limit: 500 });
    logs.value = result;
  } catch (e) {
    console.error('Failed to load logs:', e);
  }
}

async function searchLogsInDb() {
  if (!searchKeyword.value) {
    loadLogs();
    return;
  }
  try {
    const result = await invoke<LogEntry[]>('search_logs', {
      keyword: searchKeyword.value,
      limit: 200
    });
    logs.value = result;
  } catch (e) {
    console.error('Failed to search logs:', e);
  }
}

async function clearLogs() {
  try {
    await invoke('clear_log_buffer');
    logs.value = [];
  } catch (e) {
    console.error('Failed to clear logs:', e);
  }
}

function togglePause() {
  isPaused.value = !isPaused.value;
}

// Auto-scroll when new logs arrive
watch(() => logs.value.length, async () => {
  if (autoScroll.value && !isPaused.value && logContainer.value) {
    await nextTick();
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
});

// Debounce search
let searchTimeout: ReturnType<typeof setTimeout>;
watch(searchKeyword, () => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    searchLogsInDb();
  }, 300);
});

// Listen for log-batch events
let unlisten: (() => void) | null = null;

onMounted(async () => {
  await loadLogs();
  unlisten = await listen<LogEntry[]>('log-batch', (event) => {
    if (!isPaused.value) {
      logs.value.push(...event.payload);
      // Keep only last 1000 logs in memory
      if (logs.value.length > 1000) {
        logs.value = logs.value.slice(-1000);
      }
    }
  });
});

onUnmounted(() => {
  document.removeEventListener('mousemove', doResize);
  document.removeEventListener('mouseup', stopResize);
  if (unlisten) unlisten();
});
</script>

<template>
  <div class="log-stream-view" :style="{ height: panelHeight + 'px' }">
    <!-- Resize Handle -->
    <div
      class="resize-handle"
      @mousedown="startResize"
      title="Drag to resize"
    ></div>

    <div class="log-header">
      <span class="title">{{ t('logStream.title') }}</span>

      <div class="controls">
        <button
          class="control-btn"
          :class="{ active: isPaused }"
          @click="togglePause()"
          :title="isPaused ? t('logStream.resume') : t('logStream.pause')"
        >
          {{ isPaused ? '▶' : '⏸' }}
        </button>

        <button
          class="control-btn"
          @click="clearLogs()"
          :title="t('logStream.clear')"
        >
          🗑
        </button>

        <div class="search-container">
          <span class="search-icon">🔍</span>
          <input
            type="text"
            class="search-input"
            :placeholder="t('logStream.searchPlaceholder')"
            v-model="searchKeyword"
          />
          <button
            v-if="searchKeyword"
            class="search-clear-btn"
            @click="searchKeyword = ''"
            title="Clear search"
          >
            ×
          </button>
        </div>

        <span v-if="searchKeyword" class="match-count">
          {{ filteredLogs.length }} {{ filteredLogs.length === 1 ? t('logStream.match') : t('logStream.matches') }}
        </span>

        <select
          class="filter-select"
          v-model="selectedType"
        >
          <option value="all">{{ t('logStream.allTypes') }}</option>
          <option value="compile">Compile</option>
          <option value="debug">Debug</option>
          <option value="network">Network</option>
          <option value="error">Error</option>
          <option value="warning">Warning</option>
          <option value="info">Info</option>
          <option value="system">System</option>
        </select>

        <select
          class="filter-select agent-select"
          v-model="selectedAgent"
        >
          <option value="all">{{ t('logStream.allAgents') }}</option>
          <!-- Agent options would be populated dynamically -->
        </select>
      </div>

      <button class="close-btn" @click="emit('close')" title="Close">×</button>
    </div>

    <div
      class="log-container"
      ref="logContainer"
      @scroll="handleScroll"
    >
      <div
        class="log-list"
        :style="{ height: totalHeight + 'px' }"
      >
        <div
          v-for="(log, index) in visibleLogs"
          :key="log.id"
          class="log-entry"
          :class="log.logType"
          :style="{
            position: 'absolute',
            top: (visibleRange.startIndex + index) * ITEM_HEIGHT + 'px',
            height: ITEM_HEIGHT + 'px'
          }"
        >
          <span class="type-icon">{{ getTypeIcon(log.logType) }}</span>
          <span class="timestamp">{{ formatTime(log.timestamp) }}</span>
          <span class="agent-id">{{ log.agentId.slice(0, 8) }}</span>
          <span class="source-badge" :class="log.source">{{ log.source }}</span>
          <span
            class="log-content"
            v-html="highlightKeyword(log.content)"
          ></span>
        </div>
      </div>

      <div v-if="filteredLogs.length === 0" class="empty-state">
        {{ t('logStream.noLogs') }}
      </div>
    </div>

    <div v-if="isPaused" class="paused-indicator">
      {{ t('logStream.paused') }} - {{ logs.length }} {{ t('logStream.buffered') }}
    </div>

    <div class="log-count">
      {{ filteredLogs.length }} {{ t('logStream.logsCount') }}
    </div>
  </div>
</template>

<style scoped>
.log-stream-view {
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-top: 1px solid var(--border-color);
  font-size: 0.8rem;
  position: relative;
}

.resize-handle {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 6px;
  cursor: ns-resize;
  background: transparent;
  z-index: 10;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--bg-primary);
  opacity: 0.5;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-main);
  border-bottom: 1px solid var(--border-color);
}

.title {
  font-weight: 600;
  margin-right: auto;
}

.controls {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.control-btn {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 0.75rem;
}

.control-btn:hover {
  background: var(--bg-hover);
}

.control-btn.active {
  background: var(--bg-warning);
}

.search-container {
  display: flex;
  align-items: center;
  position: relative;
  background: var(--bg-main);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 0 0.25rem;
}

.search-icon {
  font-size: 0.7rem;
  opacity: 0.6;
}

.search-input {
  width: 120px;
  padding: 0.25rem 0.4rem;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 0.75rem;
  outline: none;
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-clear-btn {
  padding: 0 0.25rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.9rem;
  line-height: 1;
}

.search-clear-btn:hover {
  color: var(--text-primary);
}

.match-count {
  font-size: 0.7rem;
  color: var(--text-muted);
  white-space: nowrap;
}

.filter-select {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-main);
  color: var(--text-primary);
  font-size: 0.75rem;
  cursor: pointer;
}

.agent-select {
  max-width: 100px;
}

.close-btn {
  padding: 0.25rem 0.5rem;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 1rem;
  color: var(--text-muted);
}

.close-btn:hover {
  color: var(--text-primary);
}

.log-container {
  flex: 1;
  overflow-y: auto;
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  position: relative;
}

.log-list {
  position: relative;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  border-bottom: 1px solid var(--border-color);
  white-space: nowrap;
  overflow: hidden;
}

.log-entry:hover {
  background: var(--bg-hover);
}

.type-icon {
  width: 1.2em;
}

.timestamp {
  color: var(--text-muted);
  font-size: 0.7rem;
}

.agent-id {
  color: var(--text-accent);
  font-size: 0.7rem;
}

.source-badge {
  padding: 0 0.25rem;
  border-radius: 3px;
  font-size: 0.65rem;
  text-transform: uppercase;
}

.source-badge.stdout {
  background: rgba(40, 167, 69, 0.2);
  color: #28a745;
}

.source-badge.stderr {
  background: rgba(220, 53, 69, 0.2);
  color: #dc3545;
}

.log-content {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
}

.log-content mark {
  background: rgba(255, 193, 7, 0.3);
  color: inherit;
  padding: 0;
}

/* Log type colors */
.log-entry.compile { border-left: 3px solid #6a5acd; }
.log-entry.debug { border-left: 3px solid #888; }
.log-entry.network { border-left: 3px solid #0066cc; }
.log-entry.error {
  border-left: 3px solid #dc3545;
  background: rgba(220, 53, 69, 0.05);
}
.log-entry.warning { border-left: 3px solid #ffc107; }
.log-entry.info { border-left: 3px solid #28a745; }
.log-entry.system { border-left: 3px solid #6c757d; }

@media (prefers-color-scheme: dark) {
  .log-entry.compile { border-left-color: #9370db; }
  .log-entry.network { border-left-color: #4da6ff; }
  .log-entry.error {
    border-left-color: #ff6b6b;
    background: rgba(255, 107, 107, 0.05);
  }
  .log-entry.info { border-left-color: #5cb85c; }
}

.empty-state {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.paused-indicator {
  position: absolute;
  bottom: 0.5rem;
  right: 0.5rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-warning);
  border-radius: 4px;
  font-size: 0.7rem;
  color: #856404;
}

.log-count {
  position: absolute;
  bottom: 0.5rem;
  left: 0.75rem;
  font-size: 0.7rem;
  color: var(--text-muted);
}
</style>