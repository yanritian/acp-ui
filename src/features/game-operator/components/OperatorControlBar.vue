<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { OperatorTask } from '@/types/operator'
import { operatorStatusKey } from '../operatorStatus'

const { t } = useI18n()

defineProps<{
  task: OperatorTask
}>()

const emit = defineEmits<{
  pause: []
  resume: []
  stop: []
}>()

const statusColors: Record<string, string> = {
  idle: '#94A3B8',
  planning: '#3B82F6',
  waiting_approval: '#F59E0B',
  running: '#10B981',
  paused: '#F59E0B',
  redirecting: '#8B5CF6',
  cancelling: '#EF4444',
  cancelled: '#6B7280',
  failed: '#EF4444',
  completed: '#10B981',
}

const stoppableStatuses = ['planning', 'waiting_approval', 'running', 'paused', 'redirecting']

function statusLabel(status: string): string {
  return t(`operatorStatus.${operatorStatusKey(status)}`)
}
</script>

<template>
  <div class="operator-control-bar">
    <div class="task-info">
      <div
        class="task-status"
        :style="{ background: statusColors[task.status] }"
        :aria-label="`${t('a11y.approvalLevel')}: ${statusLabel(task.status)}`"
      >
        {{ statusLabel(task.status) }}
      </div>
      <div class="task-details">
        <div class="task-goal">{{ task.goal }}</div>
        <div class="task-meta">
          <span>{{ t('history.taskId') }}: {{ task.task_id }}</span>
          <span>{{ t('evolution.domain') }}: {{ task.domain }}</span>
        </div>
      </div>
    </div>

    <div class="control-buttons">
      <button
        v-if="task.status === 'running'"
        @click="emit('pause')"
        class="control-btn pause-btn"
        :aria-label="t('a11y.pauseButton')"
      >
        {{ t('logStream.pause') }}
      </button>
      <button
        v-if="task.status === 'paused'"
        @click="emit('resume')"
        class="control-btn resume-btn"
        :aria-label="t('a11y.resumeButton')"
      >
        {{ t('logStream.resume') }}
      </button>
      <button
        v-if="stoppableStatuses.includes(task.status)"
        @click="emit('stop')"
        class="control-btn stop-btn"
        :aria-label="t('a11y.stopButton')"
      >
        {{ t('agentConfig.stop') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.operator-control-bar {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
  padding: 1rem;
  background: var(--bg-sidebar);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.task-info {
  display: flex;
  align-items: center;
  flex: 1 1 420px;
  max-width: 100%;
  min-width: 0;
  gap: 1rem;
}

.task-status {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  color: white;
  font-weight: 600;
  font-size: 0.85rem;
  white-space: nowrap;
}

.task-details {
  display: flex;
  flex: 1 1 220px;
  min-width: 0;
  flex-direction: column;
  gap: 0.25rem;
}

.task-goal {
  font-weight: 500;
  overflow-wrap: anywhere;
}

.task-meta {
  display: flex;
  flex-wrap: wrap;
  min-width: 0;
  gap: 1rem;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.task-meta span {
  min-width: 0;
  overflow-wrap: anywhere;
}

.control-buttons {
  display: flex;
  flex-shrink: 0;
  margin-left: auto;
  gap: 0.5rem;
}

.control-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: opacity 0.2s;
}

.control-btn:hover {
  opacity: 0.8;
}

.pause-btn {
  background: #F59E0B;
  color: white;
}

.resume-btn {
  background: #10B981;
  color: white;
}

.stop-btn {
  background: #EF4444;
  color: white;
}

@container (max-width: 700px) {
  .operator-control-bar {
    align-items: flex-start;
    flex-direction: column;
  }

  .task-info {
    align-items: flex-start;
    flex: 0 1 auto;
    flex-direction: column;
    width: 100%;
  }

  .task-details {
    flex: 0 1 auto;
    width: 100%;
  }

  .control-buttons {
    margin-left: 0;
  }
}
</style>
