<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMultiSessionStore } from '../stores/multi-session'
import NewSessionDialog from './multi-session/NewSessionDialog.vue'
import { useI18n } from '@/locales'

const { t } = useI18n()

const multiSession = useMultiSessionStore()

const sessions = computed(() => multiSession.sessionList)
const activeId = computed(() => multiSession.activeSessionId)
const showNewDialog = ref(false)

function getAgentIcon(agentName: string): string {
  const name = agentName.toLowerCase()
  if (name.includes('claude')) return '🤖'
  if (name.includes('codex')) return '💻'
  if (name.includes('gemini')) return '✨'
  if (name.includes('copilot')) return '🐙'
  return '🤖'
}

async function handleCreateSession(agentName: string, cwd: string) {
  await multiSession.createSession(agentName, cwd)
  showNewDialog.value = false
}

async function handleCloseSession(sessionId: string) {
  await multiSession.closeSession(sessionId)
}
</script>

<template>
  <div class="session-tabs">
    <div
      v-for="session in sessions"
      :key="session.id"
      class="tab"
      :class="{ active: session.id === activeId }"
      @click="multiSession.switchSession(session.id)"
    >
      <span class="tab-icon">{{ getAgentIcon(session.agentName) }}</span>
      <span class="tab-title">{{ session.title }}</span>
      <span class="tab-status" :class="session.status">
        {{ session.status === 'connected' ? '●' : session.status === 'connecting' ? '◌' : '○' }}
      </span>
      <button class="tab-close" @click.stop="handleCloseSession(session.id)" :title="t('common.close')">
        ✕
      </button>
    </div>

    <button class="tab-add" @click="showNewDialog = true" :title="t('common.newSession')">
      +
    </button>
  </div>

  <NewSessionDialog
    v-if="showNewDialog"
    @create="handleCreateSession"
    @cancel="showNewDialog = false"
  />
</template>

<style scoped>
.session-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 8px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
  min-height: 36px;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  user-select: none;
  border: 1px solid transparent;
}

.tab:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tab.active {
  background: var(--bg-primary);
  color: white;
  border-color: var(--bg-primary);
}

.tab-icon {
  font-size: 14px;
}

.tab-title {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-status {
  font-size: 10px;
  opacity: 0.6;
}

.tab-status.connected {
  color: #10b981;
}

.tab-status.connecting {
  color: #f59e0b;
}

.tab-status.disconnected,
.tab-status.error {
  color: #ef4444;
}

.tab-close {
  display: none;
  padding: 0 2px;
  border: none;
  background: transparent;
  color: inherit;
  font-size: 12px;
  cursor: pointer;
  opacity: 0.5;
  border-radius: 3px;
}

.tab:hover .tab-close {
  display: inline-flex;
}

.tab-close:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.2);
}

.tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px dashed var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-add:hover {
  border-color: var(--primary);
  color: var(--primary);
  background: rgba(99, 102, 241, 0.05);
}
</style>
