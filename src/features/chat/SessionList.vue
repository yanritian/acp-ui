<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { useSessionStore } from '@/stores/session';
import { useSessionLifecycleStore } from '@/stores/session-lifecycle';
import { useConfigStore } from '@/stores/config';
import type { SavedSession, ChatMessage } from '@/lib/types';
import { useI18n } from '@/locales';
import {
  exportToMarkdown,
  exportToJson,
  copyToClipboard,
  downloadFile,
  generateFilename,
  groupSessionsByTime,
  type TimeGroup,
} from '@/lib/session-export';
import { useSessionMessagesStore } from '@/stores/session-messages';

const { t } = useI18n();

const emit = defineEmits<{
  resume: [session: SavedSession];
  delete: [sessionId: string];
}>();

const sessionStore = useSessionStore();
const lifecycleStore = useSessionLifecycleStore();
const configStore = useConfigStore();
const messagesStore = useSessionMessagesStore();

// Search and filter state
const searchQuery = ref('');
const selectedAgentFilter = ref('');

// Export menu state
const showExportMenu = ref<string | null>(null); // session.id when menu is open

// Click outside handler reference
let clickOutsideHandler: ((e: MouseEvent) => void) | null = null;

// Get unique agent names for filter dropdown
const agentNames = computed(() => {
  const agents = new Set<string>();
  for (const session of sessionStore.savedSessions) {
    agents.add(session.agentName);
  }
  // Also add agents from config that might not have sessions yet
  const configAgents = configStore.config?.agents;
  if (configAgents) {
    for (const name of Object.keys(configAgents)) {
      agents.add(name);
    }
  }
  return Array.from(agents).sort();
});

// Filter and sort sessions
const filteredSessions = computed(() => {
  let sessions = [...sessionStore.resumableSessions];

  // Filter by search query
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    sessions = sessions.filter(
      s =>
        s.title.toLowerCase().includes(query) ||
        s.agentName.toLowerCase().includes(query)
    );
  }

  // Filter by agent
  if (selectedAgentFilter.value) {
    sessions = sessions.filter(s => s.agentName === selectedAgentFilter.value);
  }

  return sessions;
});

// Group sessions by time
const groupedSessions = computed(() => {
  return groupSessionsByTime(filteredSessions.value, t);
});

// Get groups that have sessions
const activeGroups = computed(() => {
  const groups: TimeGroup[] = ['pinned', 'today', 'yesterday', 'thisWeek', 'older'];
  return groups.filter(g => (groupedSessions.value.get(g)?.length ?? 0) > 0);
});

// Format date for display
function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

// Format relative time
function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  const minuteMs = 60 * 1000;
  const hourMs = 60 * minuteMs;
  const dayMs = 24 * hourMs;

  if (diff < minuteMs) {
    return t('sessionList.justNow');
  } else if (diff < hourMs) {
    const minutes = Math.floor(diff / minuteMs);
    return t('sessionList.minutesAgo').replace('{n}', String(minutes));
  } else if (diff < dayMs) {
    const hours = Math.floor(diff / hourMs);
    return t('sessionList.hoursAgo').replace('{n}', String(hours));
  } else {
    return formatDate(timestamp);
  }
}

function handleResume(session: SavedSession) {
  emit('resume', session);
}

function handleDelete(sessionId: string, event: Event) {
  event.stopPropagation();
  closeExportMenu();
  if (confirm(t('sessionList.deleteConfirm'))) {
    emit('delete', sessionId);
  }
}

// Pin/Unpin session
async function handlePin(session: SavedSession, event: Event) {
  event.stopPropagation();
  closeExportMenu();

  const updatedSession: SavedSession = {
    ...session,
    pinned: !session.pinned,
  };

  // Update session in lifecycle store
  const sessions = lifecycleStore.savedSessions;
  const index = sessions.findIndex(s => s.id === session.id);
  if (index >= 0) {
    sessions[index] = updatedSession;
    await lifecycleStore.saveSessionsToStore();
  }
}

// Export handlers
function openExportMenu(sessionId: string, event: Event) {
  event.stopPropagation();
  showExportMenu.value = sessionId;
}

function closeExportMenu() {
  showExportMenu.value = null;
}

async function handleExportMarkdown(session: SavedSession, event: Event) {
  event.stopPropagation();
  closeExportMenu();

  const messages = await loadSessionMessages(session.sessionId);
  const content = exportToMarkdown(session, messages);
  const filename = generateFilename(session, 'md');
  downloadFile(content, filename, 'text/markdown');
}

async function handleExportJson(session: SavedSession, event: Event) {
  event.stopPropagation();
  closeExportMenu();

  const messages = await loadSessionMessages(session.sessionId);
  const content = exportToJson(session, messages);
  const filename = generateFilename(session, 'json');
  downloadFile(content, filename, 'application/json');
}

async function handleCopyToClipboard(session: SavedSession, event: Event) {
  event.stopPropagation();
  closeExportMenu();

  const messages = await loadSessionMessages(session.sessionId);
  const content = exportToMarkdown(session, messages);
  const success = await copyToClipboard(content);

  // Could show a toast notification here
  if (!success) {
    alert(t('sessionList.copyFailed'));
  }
}

// Load messages for a session
async function loadSessionMessages(sessionId: string): Promise<ChatMessage[]> {
  try {
    const messages = await messagesStore.loadMessages(sessionId);
    return messages;
  } catch (e) {
    console.error('Failed to load messages for export:', e);
    return [];
  }
}

// Close export menu when clicking outside
watch(showExportMenu, (newValue, oldValue) => {
  if (newValue !== oldValue && newValue !== null) {
    // Add click-outside handler
    setTimeout(() => {
      document.addEventListener('click', closeExportMenu, { once: true });
    }, 0);
  }
});

// Clear search when component resets
function clearSearch() {
  searchQuery.value = '';
  selectedAgentFilter.value = '';
}
</script>

<template>
  <div class="session-list">
    <h3>{{ t('sessionList.title') }}</h3>

    <!-- Search and Filter Section -->
    <div class="search-filter-section">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="t('sessionList.searchPlaceholder')"
        @click.stop
      />
      <select
        v-model="selectedAgentFilter"
        class="agent-filter"
        @click.stop
      >
        <option value="">{{ t('sessionList.allAgents') }}</option>
        <option
          v-for="agent in agentNames"
          :key="agent"
          :value="agent"
        >
          {{ agent }}
        </option>
      </select>
      <button
        v-if="searchQuery || selectedAgentFilter"
        class="clear-filter-btn"
        @click="clearSearch"
        :title="t('sessionList.clearFilter')"
      >
        ×
      </button>
    </div>

    <!-- Empty State -->
    <div v-if="filteredSessions.length === 0" class="empty-state">
      <p>{{ searchQuery || selectedAgentFilter ? t('sessionList.noMatchingSessions') : t('sessionList.noSavedSessions') }}</p>
      <p class="hint">{{ searchQuery || selectedAgentFilter ? t('sessionList.tryDifferentSearch') : t('sessionList.createSessionHint') }}</p>
    </div>

    <!-- Session Groups -->
    <div v-else class="session-groups">
      <div
        v-for="group in activeGroups"
        :key="group"
        class="session-group"
      >
        <div class="group-header">
          <span class="group-label">{{ group }}</span>
          <span class="group-count">{{ groupedSessions.get(group)?.length }}</span>
        </div>
        <ul class="group-items">
          <li
            v-for="session in groupedSessions.get(group)"
            :key="session.id"
            class="session-item"
            :class="{ pinned: session.pinned }"
            @click="handleResume(session)"
          >
            <div class="session-info">
              <span class="session-title">
                <span v-if="session.pinned" class="pin-indicator">📌</span>
                {{ session.title }}
              </span>
              <span class="session-agent">{{ session.agentName }}</span>
              <span class="session-date">{{ formatRelativeTime(session.lastUpdated) }}</span>
            </div>
            <div class="session-actions">
              <!-- Pin Button -->
              <button
                class="pin-btn"
                :class="{ active: session.pinned }"
                @click="(e) => handlePin(session, e)"
                :title="session.pinned ? t('sessionList.unpinSession') : t('sessionList.pinSession')"
              >
                📌
              </button>
              <!-- Export Button -->
              <button
                class="export-btn"
                @click="(e) => openExportMenu(session.id, e)"
                :title="t('sessionList.exportSession')"
              >
                ⬇
              </button>
              <!-- Export Menu Dropdown -->
              <div
                v-if="showExportMenu === session.id"
                class="export-menu"
                @click.stop
              >
                <button @click="(e) => handleExportMarkdown(session, e)">
                  {{ t('sessionList.exportMarkdown') }}
                </button>
                <button @click="(e) => handleExportJson(session, e)">
                  {{ t('sessionList.exportJson') }}
                </button>
                <button @click="(e) => handleCopyToClipboard(session, e)">
                  {{ t('sessionList.copyToClipboard') }}
                </button>
              </div>
              <!-- Delete Button -->
              <button
                class="delete-btn"
                @click="(e) => handleDelete(session.id, e)"
                :title="t('sessionList.deleteSession')"
              >
                ×
              </button>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.session-list {
  padding: 1rem;
}

h3 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  color: var(--text-secondary, #666);
}

/* Search and Filter */
.search-filter-section {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.search-input {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--text-accent, #0066cc);
}

.agent-filter {
  padding: 0.5rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  font-size: 0.875rem;
  background: var(--bg-surface, #fff);
  min-width: 120px;
}

.clear-filter-btn {
  padding: 0.5rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted, #999);
  font-size: 1rem;
  cursor: pointer;
}

.clear-filter-btn:hover {
  background: var(--bg-hover, #f5f5f5);
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted, #999);
}

.empty-state .hint {
  font-size: 0.875rem;
  margin-top: 0.5rem;
}

/* Session Groups */
.session-groups {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.session-group {
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-hover, #f5f5f5);
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--text-secondary, #666);
}

.group-label {
  text-transform: uppercase;
}

.group-count {
  background: var(--bg-surface, #fff);
  padding: 0.125rem 0.5rem;
  border-radius: 10px;
  font-size: 0.75rem;
}

.group-items {
  list-style: none;
  padding: 0;
  margin: 0;
}

.session-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  border-top: 1px solid var(--border-color, #e0e0e0);
  cursor: pointer;
  transition: background 0.15s;
}

.session-item:first-child {
  border-top: none;
}

.session-item:hover {
  background: var(--bg-hover, #f5f5f5);
}

.session-item.pinned {
  background: var(--bg-pinned, #fff8e1);
}

.session-item.pinned:hover {
  background: var(--bg-pinned-hover, #fff3c4);
}

.session-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  overflow: hidden;
  min-width: 0;
}

.session-title {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.pin-indicator {
  font-size: 0.875rem;
}

.session-agent {
  font-size: 0.75rem;
  color: var(--text-accent, #0066cc);
}

.session-date {
  font-size: 0.75rem;
  color: var(--text-muted, #999);
}

.session-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  position: relative;
}

.pin-btn,
.export-btn,
.delete-btn {
  padding: 0.25rem 0.5rem;
  border: none;
  background: transparent;
  color: var(--text-muted, #999);
  font-size: 1rem;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s;
}

.pin-btn:hover,
.export-btn:hover {
  background: var(--bg-hover, #f0f0f0);
  color: var(--text-primary, #333);
}

.pin-btn.active {
  color: var(--text-accent, #0066cc);
}

.delete-btn:hover {
  background: var(--bg-danger, #fee);
  color: var(--text-danger, #c00);
}

/* Export Menu */
.export-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 10;
  background: var(--bg-surface, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  padding: 0.25rem 0;
  min-width: 150px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.export-menu button {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  background: transparent;
  text-align: left;
  font-size: 0.85rem;
  cursor: pointer;
}

.export-menu button:hover {
  background: var(--bg-hover, #f5f5f5);
}

/* Mobile */
@media (max-width: 800px) {
  .search-filter-section {
    flex-wrap: wrap;
  }

  .agent-filter {
    flex: 1;
    min-width: 0;
  }

  .session-actions {
    flex-wrap: wrap;
  }
}
</style>