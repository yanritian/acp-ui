<script setup lang="ts">
import { computed } from 'vue'
import type { ToolExecution, ToolExecutionStatus } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  toolExecution: ToolExecution
}>()

const emit = defineEmits<{
  (e: 'cancel'): void
}>()

// Computed
const statusColor = computed(() => {
  const colors: Record<ToolExecutionStatus, string> = {
    pending: '#6B7280',
    running: '#F59E0B',
    completed: '#10B981',
    failed: '#EF4444',
    cancelled: '#9CA3AF'
  }
  return colors[props.toolExecution.status]
})

const statusIcon = computed(() => {
  const icons: Record<ToolExecutionStatus, string> = {
    pending: '⏳',
    running: '⚡',
    completed: '✅',
    failed: '❌',
    cancelled: '🚫'
  }
  return icons[props.toolExecution.status]
})

const executionTime = computed(() => {
  const duration = props.toolExecution.duration || 0
  if (duration < 1000) return `${duration}ms`
  return `${(duration / 1000).toFixed(1)}s`
})

const parameterPreview = computed(() => {
  const params = props.toolExecution.parameters
  const preview = JSON.stringify(params, null, 2)
  return preview.length > 200 ? preview.slice(0, 200) + '...' : preview
})

const resultPreview = computed(() => {
  if (!props.toolExecution.result) return null
  const result = JSON.stringify(props.toolExecution.result, null, 2)
  return result.length > 300 ? result.slice(0, 300) + '...' : result
})
</script>

<template>
  <div class="tool-execution-monitor">
    <!-- Header -->
    <div class="tool-header">
      <div class="tool-info">
        <span class="tool-icon">🔧</span>
        <span class="tool-name">{{ toolExecution.toolName }}</span>
        <span
          :class="['status-badge', toolExecution.status]"
          :style="{ backgroundColor: statusColor }"
        >
          <span class="status-icon">{{ statusIcon }}</span>
          <span class="status-text">{{ toolExecution.status }}</span>
        </span>
      </div>
      <div class="execution-time">{{ executionTime }}</div>
    </div>

    <!-- Progress bar -->
    <div v-if="toolExecution.status === 'running'" class="progress-section">
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${toolExecution.progress}%`, backgroundColor: statusColor }"
        />
      </div>
      <div class="progress-label">{{ toolExecution.progress }}%</div>
    </div>

    <!-- Parameters -->
    <div class="parameters-section">
      <div class="section-label">{{ t('agentProgress.parameters') }}</div>
      <pre class="parameters-preview">{{ parameterPreview }}</pre>
    </div>

    <!-- Result -->
    <div v-if="toolExecution.status === 'completed' && resultPreview" class="result-section">
      <div class="section-label">{{ t('agentProgress.executionResult') }}</div>
      <pre class="result-preview">{{ resultPreview }}</pre>
    </div>

    <!-- Error -->
    <div v-if="toolExecution.status === 'failed'" class="error-section">
      <div class="section-label error">{{ t('agentProgress.errorMessage') }}</div>
      <div class="error-message">{{ toolExecution.error }}</div>
    </div>

    <!-- Cancel button -->
    <div v-if="toolExecution.status === 'running'" class="actions-section">
      <button class="cancel-button" @click="emit('cancel')">
        🚫 {{ t('agentProgress.cancelExecution') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.tool-execution-monitor {
  padding: 16px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 12px;
}

.tool-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(245, 158, 11, 0.2);
}

.tool-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tool-icon {
  font-size: 20px;
}

.tool-name {
  font-size: 14px;
  font-weight: 600;
  color: #f59e0b;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 6px;
  color: white;
  font-size: 11px;
}

.status-icon {
  font-size: 12px;
}

.status-text {
  font-size: 10px;
  font-weight: 500;
}

.execution-time {
  font-size: 12px;
  color: #8b8b9b;
  font-family: 'Monaco', monospace;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.progress-bar {
  flex: 1;
  height: 10px;
  background: rgba(139, 139, 155, 0.2);
  border-radius: 5px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 5px;
  transition: width 0.3s ease;
}

.progress-label {
  font-size: 12px;
  color: #e0e0e0;
  font-family: 'Monaco', monospace;
  min-width: 40px;
}

.parameters-section,
.result-section,
.error-section {
  margin-bottom: 12px;
}

.section-label {
  font-size: 12px;
  color: #8b8b9b;
  margin-bottom: 6px;
  font-weight: 500;
}

.section-label.error {
  color: #ef4444;
}

.parameters-preview,
.result-preview {
  padding: 12px;
  background: rgba(26, 26, 46, 0.5);
  border-radius: 8px;
  border: 1px solid #3a3a5a;
  font-size: 12px;
  color: #e0e0e0;
  font-family: 'Monaco', monospace;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.error-message {
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  font-size: 13px;
  color: #ef4444;
}

.actions-section {
  padding-top: 12px;
  border-top: 1px solid rgba(245, 158, 11, 0.2);
}

.cancel-button {
  padding: 8px 16px;
  background: rgba(239, 68, 68, 0.2);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: #ef4444;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cancel-button:hover {
  background: rgba(239, 68, 68, 0.3);
}
</style>