<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import type {
  AgentRealtimeStatus,
  AgentActivityType,
  RealtimeEvent
} from '@/lib/agent-runtime/realtime-progress-types'
import { ACTIVITY_INDICATORS } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'
import ActivityIndicator from './ActivityIndicator.vue'
import ThinkingDisplay from './ThinkingDisplay.vue'
import ToolExecutionMonitor from './ToolExecutionMonitor.vue'
import OutputTypewriter from './OutputTypewriter.vue'
import PermissionWaiting from './PermissionWaiting.vue'
import AgentPetAvatar from '@/components/agent-pet/AgentPetAvatar.vue'

const { t } = useI18n()

// Props
const props = defineProps<{
  agentId: string
  agentName: string
  initialStatus?: Partial<AgentRealtimeStatus>
}>()

// Emits
const emit = defineEmits<{
  (e: 'permission_response', optionId: string): void
  (e: 'interaction', type: string): void
  (e: 'status_change', status: AgentActivityType): void
}>()

// State
const status = ref<AgentRealtimeStatus>({
  agentId: props.agentId,
  agentName: props.agentName,
  currentActivity: {
    type: 'idle',
    startTime: Date.now(),
    duration: 0,
    progress: 0
  },
  thinking: {
    content: '',
    startTime: 0,
    depth: 0,
    chunks: [],
    isStreaming: false
  },
  toolExecution: null,
  output: {
    content: '',
    chunks: [],
    totalLength: 0,
    currentPosition: 0,
    isStreaming: false
  },
  permissionWaiting: null,
  stats: {
    tasksCompleted: 0,
    tasksFailed: 0,
    averageDuration: 0,
    successRate: 100,
    totalThinkingTime: 0,
    totalToolCalls: 0
  },
  lastUpdateTime: Date.now()
})

// Computed
const currentIndicator = computed(() => {
  return ACTIVITY_INDICATORS[status.value.currentActivity.type]
})

const isThinking = computed(() => {
  return status.value.currentActivity.type === 'thinking'
})

const isExecuting = computed(() => {
  return status.value.currentActivity.type === 'executing'
})

const isOutputting = computed(() => {
  return status.value.currentActivity.type === 'outputting'
})

const isWaiting = computed(() => {
  return status.value.permissionWaiting !== null
})

const elapsedTime = computed(() => {
  const duration = status.value.currentActivity.duration
  if (duration < 1000) return `${duration}ms`
  if (duration < 60000) return `${(duration / 1000).toFixed(1)}s`
  return `${(duration / 60000).toFixed(1)}m`
})

// Methods
function updateStatus(newStatus: Partial<AgentRealtimeStatus>) {
  status.value = {
    ...status.value,
    ...newStatus,
    lastUpdateTime: Date.now()
  }
}

function handleRealtimeEvent(event: RealtimeEvent) {
  switch (event.type) {
    case 'thinking_start':
      status.value.currentActivity = {
        type: 'thinking',
        startTime: event.timestamp,
        duration: 0,
        progress: 0
      }
      status.value.thinking.isStreaming = true
      status.value.thinking.startTime = event.timestamp
      break

    case 'thinking_chunk':
      if (event.data.chunk) {
        status.value.thinking.chunks.push(event.data.chunk)
        status.value.thinking.content += event.data.chunk.content
        status.value.thinking.depth = event.data.depth || status.value.thinking.depth
      }
      break

    case 'thinking_end':
      status.value.thinking.isStreaming = false
      status.value.stats.totalThinkingTime += status.value.currentActivity.duration
      break

    case 'tool_call_start':
      status.value.currentActivity = {
        type: 'executing',
        startTime: event.timestamp,
        duration: 0,
        progress: 0
      }
      status.value.toolExecution = event.data.toolExecution
      status.value.stats.totalToolCalls++
      break

    case 'tool_call_progress':
      if (status.value.toolExecution) {
        status.value.toolExecution.progress = event.data.progress
      }
      break

    case 'tool_call_result':
      if (status.value.toolExecution) {
        status.value.toolExecution.status = 'completed'
        status.value.toolExecution.result = event.data.result
      }
      break

    case 'tool_call_error':
      if (status.value.toolExecution) {
        status.value.toolExecution.status = 'failed'
        status.value.toolExecution.error = event.data.error
      }
      break

    case 'output_start':
      status.value.currentActivity = {
        type: 'outputting',
        startTime: event.timestamp,
        duration: 0,
        progress: 0
      }
      status.value.output.isStreaming = true
      break

    case 'output_chunk':
      if (event.data.chunk) {
        status.value.output.chunks.push(event.data.chunk)
        status.value.output.content += event.data.chunk.content
        status.value.output.currentPosition = status.value.output.content.length
      }
      break

    case 'output_end':
      status.value.output.isStreaming = false
      status.value.output.totalLength = status.value.output.content.length
      break

    case 'permission_request':
      status.value.permissionWaiting = event.data.permission
      status.value.currentActivity.type = 'waiting'
      break

    case 'permission_response':
      status.value.permissionWaiting = null
      emit('permission_response', event.data.optionId)
      break

    case 'status_change':
      status.value.currentActivity.type = event.data.newStatus
      emit('status_change', event.data.newStatus)
      break

    case 'level_up':
      // 处理升级事件（通过宠物组件）
      break
  }

  // 更新持续时间
  status.value.currentActivity.duration =
    Date.now() - status.value.currentActivity.startTime
}

function handlePermissionResponse(optionId: string) {
  if (status.value.permissionWaiting) {
    status.value.permissionWaiting = null
    emit('permission_response', optionId)
  }
}

function handleInteraction(type: string) {
  emit('interaction', type)
}

// Watch for initial status changes
watch(() => props.initialStatus, (newInitial) => {
  if (newInitial) {
    updateStatus(newInitial)
  }
}, { immediate: true, deep: true })

// Lifecycle - simulate real-time updates for demo
let updateInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  // 演示：模拟实时状态变化
  updateInterval = setInterval(() => {
    status.value.lastUpdateTime = Date.now()
    status.value.currentActivity.duration =
      Date.now() - status.value.currentActivity.startTime
  }, 100)
})

onUnmounted(() => {
  if (updateInterval) {
    clearInterval(updateInterval)
  }
})

// Expose methods for external control
defineExpose({
  updateStatus,
  handleRealtimeEvent
})
</script>

<template>
  <div class="agent-realtime-progress-panel">
    <!-- Header -->
    <header class="panel-header">
      <div class="agent-info">
        <AgentPetAvatar
          :agentId="agentId"
          :agentName="agentName"
          :currentActivity="status.currentActivity.type"
          :emotion="status.currentActivity.type === 'error' ? 'confused' : 'focused'"
          @interaction="handleInteraction"
        />
        <div class="agent-details">
          <h3 class="agent-name">{{ agentName }}</h3>
          <div class="elapsed-time">{{ elapsedTime }}</div>
        </div>
      </div>

      <ActivityIndicator
        :type="status.currentActivity.type"
        :progress="status.currentActivity.progress"
        :description="status.currentActivity.description"
      />
    </header>

    <!-- Main content -->
    <main class="panel-content">
      <!-- Thinking display -->
      <ThinkingDisplay
        v-if="isThinking"
        :content="status.thinking.content"
        :chunks="status.thinking.chunks"
        :depth="status.thinking.depth"
        :isStreaming="status.thinking.isStreaming"
      />

      <!-- Tool execution monitor -->
      <ToolExecutionMonitor
        v-if="isExecuting && status.toolExecution"
        :toolExecution="status.toolExecution"
      />

      <!-- Output typewriter -->
      <OutputTypewriter
        v-if="isOutputting"
        :content="status.output.content"
        :chunks="status.output.chunks"
        :isStreaming="status.output.isStreaming"
        :currentPosition="status.output.currentPosition"
      />

      <!-- Permission waiting -->
      <PermissionWaiting
        v-if="isWaiting && status.permissionWaiting"
        :permission="status.permissionWaiting"
        @response="handlePermissionResponse"
      />

      <!-- Idle state -->
      <div v-if="status.currentActivity.type === 'idle'" class="idle-state">
        <div class="idle-icon">😴</div>
        <div class="idle-text">{{ t('agentProgress.idleState') }}</div>
      </div>

      <!-- Error state -->
      <div v-if="status.currentActivity.type === 'error'" class="error-state">
        <div class="error-icon">❌</div>
        <div class="error-text">{{ t('agentProgress.errorState') }}</div>
      </div>
    </main>

    <!-- Stats footer -->
    <footer class="panel-footer">
      <div class="stats-row">
        <div class="stat-item">
          <span class="stat-icon">✅</span>
          <span class="stat-value">{{ status.stats.tasksCompleted }}</span>
          <span class="stat-label">{{ t('agentProgress.tasksCompleted') }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-icon">❌</span>
          <span class="stat-value">{{ status.stats.tasksFailed }}</span>
          <span class="stat-label">{{ t('agentProgress.tasksFailed') }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-icon">⚡</span>
          <span class="stat-value">{{ status.stats.totalToolCalls }}</span>
          <span class="stat-label">{{ t('agentProgress.toolCalls') }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-icon">🧠</span>
          <span class="stat-value">{{ Math.round(status.stats.totalThinkingTime / 1000) }}s</span>
          <span class="stat-label">{{ t('agentProgress.thinkingTime') }}</span>
        </div>
        <div class="stat-item success-rate">
          <span class="stat-icon">📊</span>
          <span class="stat-value">{{ status.stats.successRate }}%</span>
          <span class="stat-label">{{ t('agentProgress.successRate') }}</span>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.agent-realtime-progress-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border: 1px solid #3a3a5a;
  border-radius: 12px;
  overflow: hidden;
  font-family: 'Monaco', 'Menlo', monospace;
}

.panel-header {
  padding: 16px 20px;
  background: rgba(26, 26, 46, 0.8);
  border-bottom: 1px solid #3a3a5a;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.agent-details {
  display: flex;
  flex-direction: column;
}

.agent-name {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0;
}

.elapsed-time {
  font-size: 12px;
  color: #8b8b9b;
  font-family: 'Monaco', monospace;
}

.panel-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.idle-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
}

.idle-icon,
.error-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.idle-text {
  font-size: 14px;
  color: #8b8b9b;
}

.error-text {
  font-size: 14px;
  color: #ef4444;
}

.panel-footer {
  padding: 12px 20px;
  background: rgba(22, 33, 62, 0.9);
  border-top: 1px solid #3a3a5a;
}

.stats-row {
  display: flex;
  justify-content: space-around;
  gap: 12px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 6px;
  background: rgba(58, 58, 90, 0.3);
}

.stat-icon {
  font-size: 14px;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
}

.stat-label {
  font-size: 11px;
  color: #8b8b9b;
}

.success-rate .stat-value {
  color: #10b981;
}
</style>