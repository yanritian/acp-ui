<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useConfigStore } from '@/stores/config'
import { useI18n } from '@/locales'
import { listRunningAgents } from '@/lib/host'
import { trackBehavior } from '@/lib/self-improvement'

interface AgentStatus {
  name: string
  running: boolean
  type: string
}

const props = defineProps<{
  agents: string[]
  selectedAgent: string | null
  loading: boolean
}>()

const emit = defineEmits<{
  (e: 'select', agentName: string): void
  (e: 'add'): void
}>()

const { t } = useI18n()
const configStore = useConfigStore()

// State
const agentStatuses = ref<AgentStatus[]>([])
const refreshing = ref(false)
let statusTimer: ReturnType<typeof setInterval> | null = null

// Computed
const runningCount = computed(() => agentStatuses.value.filter(a => a.running).length)
const stoppedCount = computed(() => agentStatuses.value.filter(a => !a.running).length)

async function loadAgentStatuses() {
  if (props.loading) return

  refreshing.value = true
  try {
    const running = await listRunningAgents()

    // Build status list
    const statuses: AgentStatus[] = props.agents.map(name => {
      const config = configStore.getAgent(name)
      return {
        name,
        running: running.some(id => id.includes(name)),
        type: (config as any)?.transport || 'stdio'
      }
    })

    agentStatuses.value = statuses
  } catch (e) {
    console.error('Failed to load agent statuses:', e)
  } finally {
    refreshing.value = false
  }
}

function handleSelect(agent: AgentStatus) {
  emit('select', agent.name)
}

function handleAdd() {
  emit('add')
}

onMounted(() => {
  loadAgentStatuses()
  // Refresh every 2 seconds
  statusTimer = setInterval(loadAgentStatuses, 2000)
})

onUnmounted(() => {
  if (statusTimer) {
    clearInterval(statusTimer)
  }
})

// Watch for agent list changes
const prevAgentsLength = ref(props.agents.length)
if (props.agents.length !== prevAgentsLength.value) {
  loadAgentStatuses()
  prevAgentsLength.value = props.agents.length
}
</script>

<template>
  <div class="agent-status-card">
    <!-- Loading State -->
    <div v-if="loading" class="loading-state">
      <span class="loading-spinner">🔄</span>
      <span>{{ t('dashboard.loadingAgents') }}</span>
    </div>

    <!-- Empty State -->
    <div v-else-if="agents.length === 0" class="empty-state">
      <div class="empty-icon">🤖</div>
      <h3>{{ t('dashboard.noAgents') }}</h3>
      <p>{{ t('dashboard.noAgentsHint') }}</p>
      <button class="add-btn" @click="handleAdd">
        {{ t('dashboard.addFirstAgent') }}
      </button>
    </div>

    <!-- Agent Grid -->
    <div v-else class="agents-grid">
      <!-- Status Summary -->
      <div class="status-summary">
        <span class="summary-item">
          <span class="summary-dot running"></span>
          {{ runningCount }} {{ t('dashboard.running') }}
        </span>
        <span class="summary-item">
          <span class="summary-dot stopped"></span>
          {{ stoppedCount }} {{ t('dashboard.stopped') }}
        </span>
        <button class="refresh-btn" @click="loadAgentStatuses" :disabled="refreshing">
          {{ refreshing ? '🔄' : '↻' }}
        </button>
      </div>

      <!-- Agent Cards -->
      <div class="agent-list">
        <div
          v-for="agent in agentStatuses"
          :key="agent.name"
          class="agent-item"
          :class="{
            running: agent.running,
            selected: selectedAgent === agent.name
          }"
          @click="handleSelect(agent)"
        >
          <div class="agent-status-indicator">
            <span :class="['status-dot', agent.running ? 'running' : 'stopped']"></span>
          </div>
          <div class="agent-info">
            <span class="agent-name">{{ agent.name }}</span>
            <span class="agent-type">{{ agent.type }}</span>
          </div>
          <div class="agent-state">
            {{ agent.running ? t('dashboard.running') : t('dashboard.stopped') }}
          </div>
        </div>
      </div>

      <!-- Add Button -->
      <button class="add-btn" @click="handleAdd">
        + {{ t('dashboard.addAgent') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.agent-status-card {
  min-height: 200px;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
  color: var(--text-muted, #999);
}

.loading-spinner {
  font-size: 24px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-primary, #333);
}

.empty-state p {
  color: var(--text-muted, #999);
  margin-bottom: 20px;
}

.add-btn {
  padding: 12px 24px;
  background: var(--bg-primary, #0066cc);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.add-btn:hover {
  background: var(--bg-primary-hover, #0052a3);
}

.agents-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-summary {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: var(--bg-subtle, #f5f5f5);
  border-radius: 8px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-secondary, #666);
}

.summary-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.summary-dot.running {
  background: #4CAF50;
}

.summary-dot.stopped {
  background: #9E9E9E;
}

.refresh-btn {
  padding: 6px 10px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
}

.refresh-btn:hover:not(:disabled) {
  background: var(--bg-hover, #f0f0f0);
}

.refresh-btn:disabled {
  opacity: 0.5;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.agent-item:hover {
  background: var(--bg-hover, #f0f0f0);
}

.agent-item.selected {
  border-color: var(--bg-primary, #0066cc);
  background: rgba(0, 102, 204, 0.05);
}

.agent-item.running {
  border-left: 3px solid #4CAF50;
}

.agent-status-indicator {
  flex-shrink: 0;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.status-dot.running {
  background: #4CAF50;
  box-shadow: 0 0 4px rgba(76, 175, 80, 0.5);
}

.status-dot.stopped {
  background: #9E9E9E;
}

.agent-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.agent-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.agent-type {
  font-size: 12px;
  color: var(--text-muted, #999);
}

.agent-state {
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.agent-item.running .agent-state {
  background: rgba(76, 175, 80, 0.1);
  color: #4CAF50;
}

.agent-item:not(.running) .agent-state {
  background: rgba(158, 158, 158, 0.1);
  color: #9E9E9E;
}
</style>