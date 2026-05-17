<script setup lang="ts">
import { computed } from 'vue'
import { useTeamRuntimeStore } from '../stores/team-runtime'
import { useI18n } from '@/locales'

const { t } = useI18n()

const teamRuntime = useTeamRuntimeStore()

const tasks = computed(() => teamRuntime.taskList)

function getTaskOutputs(taskId: string) {
  return teamRuntime.getTaskOutputs(taskId)
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleString()
}

function statusIcon(status: string): string {
  switch (status) {
    case 'running': return '▶️'
    case 'completed': return '✅'
    case 'failed': return '❌'
    case 'cancelled': return '⏹️'
    default: return '⏸️'
  }
}
</script>

<template>
  <div class="plan-visualization">
    <!-- Header -->
    <header class="plan-header">
      <h2>{{ t('planVisualization.title') }}</h2>
    </header>

    <!-- Task List -->
    <div v-if="tasks.length > 0" class="task-list">
      <div v-for="task in tasks" :key="task.id" class="task-card" :class="task.status">
        <div class="task-title">
          <span>{{ statusIcon(task.status) }}</span>
          <span class="task-name">{{ task.title }}</span>
          <span class="task-source">[{{ task.source }}]</span>
        </div>
        <div class="task-meta">
          <span>{{ t('planVisualization.sessionsCount', { count: task.targetSessionIds.length }) }}</span>
          <span>{{ formatTime(task.createdAt) }}</span>
        </div>
        <!-- Outputs per agent -->
        <div v-for="output in getTaskOutputs(task.id)" :key="output.sessionId" class="task-output">
          <span class="output-agent">{{ output.agentName }}:</span>
          <span class="output-status">{{ output.status === 'completed' ? '✅' : output.status === 'failed' ? '❌' : '⏳' }}</span>
          <span class="output-text">{{ output.content ? output.content.substring(0, 100) + (output.content.length > 100 ? '...' : '') : t('planVisualization.waitingOutput') }}</span>
        </div>
        <div v-if="task.error" class="task-error">{{ task.error }}</div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <span class="icon-large">📊</span>
      <p>{{ t('planVisualization.noTasks') }}</p>
      <p class="hint">{{ t('planVisualization.noTasksHint') }}</p>
    </div>
  </div>
</template>

<style scoped>
.plan-visualization {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-surface);
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-light);
}

.plan-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-card {
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-surface);
}

.task-card.running {
  border-left: 3px solid #3b82f6;
}

.task-card.completed {
  border-left: 3px solid #22c55e;
}

.task-card.failed {
  border-left: 3px solid #ef4444;
}

.task-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.task-name {
  font-weight: 500;
  font-size: 14px;
}

.task-source {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-subtle);
  padding: 1px 6px;
  border-radius: 4px;
}

.task-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.task-output {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 4px 0;
}

.output-agent {
  font-weight: 600;
  color: var(--primary);
  min-width: 80px;
}

.output-status {
  min-width: 20px;
}

.output-text {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-error {
  color: #ef4444;
  font-size: 12px;
  margin-top: 4px;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.icon-large {
  font-size: 64px;
  margin-bottom: 16px;
}

.hint {
  font-size: 13px;
  color: var(--text-subtle);
}
</style>
