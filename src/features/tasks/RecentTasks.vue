<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSessionStore } from '@/stores/session'
import { useI18n } from '@/locales'
import type { SavedSession } from '@/lib/types'

const emit = defineEmits<{
  (e: 'view-all'): void
}>()

const { t } = useI18n()
const sessionStore = useSessionStore()

// State
const recentSessions = ref<SavedSession[]>([])

onMounted(async () => {
  // Load recent sessions from store
  await sessionStore.initStore()
  recentSessions.value = sessionStore.savedSessions.slice(0, 5)
})

function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function getStatusIcon(session: SavedSession): string {
  // Saved sessions are completed by definition
  return '✓'
}

function handleViewAll() {
  emit('view-all')
}
</script>

<template>
  <div class="recent-tasks">
    <!-- Empty State -->
    <div v-if="recentSessions.length === 0" class="empty-state">
      <p>{{ t('dashboard.noRecentTasks') }}</p>
      <p class="hint">{{ t('dashboard.startFirstTask') }}</p>
    </div>

    <!-- Task List -->
    <div v-else class="task-list">
      <div
        v-for="session in recentSessions"
        :key="session.id"
        class="task-item"
      >
        <div class="task-status">
          <span class="status-icon">{{ getStatusIcon(session) }}</span>
        </div>
        <div class="task-info">
          <span class="task-agent">{{ session.agentName }}</span>
          <span class="task-title">{{ session.cwd.split(/[\\/]/).pop() || session.cwd }}</span>
        </div>
        <div class="task-time">
          {{ formatTime(session.lastUpdated) }}
        </div>
      </div>
    </div>

    <!-- View All Button -->
    <button
      v-if="recentSessions.length > 0"
      class="view-all-btn"
      @click="handleViewAll"
    >
      {{ t('dashboard.viewAllTasks') }}
    </button>
  </div>
</template>

<style scoped>
.recent-tasks {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.empty-state {
  text-align: center;
  padding: 24px;
  color: var(--text-muted, #999);
}

.empty-state p {
  margin-bottom: 8px;
}

.hint {
  font-size: 12px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  transition: background 0.2s ease;
}

.task-item:hover {
  background: var(--bg-hover, #f0f0f0);
}

.task-status {
  flex-shrink: 0;
}

.status-icon {
  font-size: 16px;
}

.task-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-agent {
  font-size: 12px;
  color: var(--text-muted, #999);
}

.task-title {
  font-size: 14px;
  color: var(--text-primary, #333);
  font-weight: 500;
}

.task-time {
  font-size: 12px;
  color: var(--text-muted, #999);
}

.view-all-btn {
  padding: 10px 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary, #666);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.view-all-btn:hover {
  background: var(--bg-hover, #f0f0f0);
  color: var(--text-primary, #333);
}
</style>