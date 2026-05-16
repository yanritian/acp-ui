<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTeamRuntimeStore } from '../stores/team-runtime'
import PlanVisualization from './PlanVisualization.vue'
import { useI18n } from '@/locales'

const { t } = useI18n()

const teamRuntime = useTeamRuntimeStore()

const selectedTaskId = ref<string | null>(null)

const taskList = computed(() => teamRuntime.taskList)
const runningTasks = computed(() => taskList.value.filter(t => t.status === 'running'))

const selectedTask = computed(() => {
  if (!selectedTaskId.value) return null
  return teamRuntime.tasks.get(selectedTaskId.value) ?? null
})

const selectedTaskOutputs = computed(() => {
  if (!selectedTaskId.value) return []
  return teamRuntime.getTaskOutputs(selectedTaskId.value)
})

function selectTask(taskId: string) {
  selectedTaskId.value = taskId
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
  <div class="orchestration-view">
    <!-- Header -->
    <header class="view-header">
      <h1>{{ t('orchestration.title') }}</h1>
      <p class="subtitle">{{ t('orchestration.subtitle') }}</p>
    </header>

    <!-- Running tasks summary -->
    <section class="tasks-summary">
      <h2 class="section-title">
        {{ t('orchestration.runningCompleted', { running: runningTasks.length, completed: teamRuntime.completedTaskCount }) }}
      </h2>
      <div class="task-cards">
        <div
          v-for="task in taskList"
          :key="task.id"
          class="task-card"
          :class="[task.status, { selected: selectedTaskId === task.id }]"
          @click="selectTask(task.id)"
        >
          <div class="card-status">
            <span>{{ statusIcon(task.status) }}</span>
            <span class="status-label">{{ task.status }}</span>
          </div>
          <div class="card-title">{{ task.title }}</div>
          <div class="card-meta">
            <span>{{ task.source }}</span>
            <span>{{ formatTime(task.createdAt) }}</span>
          </div>
        </div>
        <div v-if="taskList.length === 0" class="empty-state">
          <span class="icon-large">🤖</span>
          <p>{{ t('orchestration.noTasks') }}</p>
          <p class="hint">{{ t('orchestration.noTasksHint') }}</p>
        </div>
      </div>
    </section>

    <!-- Plan Visualization (execution timeline) -->
    <section class="plan-section">
      <h2 class="section-title">{{ t('orchestration.executionTimeline') }}</h2>
      <PlanVisualization />
    </section>

    <!-- Selected Task Detail -->
    <section v-if="selectedTask" class="task-detail-section">
      <div class="detail-header">
        <h2>{{ selectedTask.title }}</h2>
        <button class="btn-close" @click="selectedTaskId = null">✕</button>
      </div>
      <div class="detail-content">
        <div class="detail-row">
          <span class="label">{{ t('orchestration.source') }}:</span>
          <span class="value">{{ selectedTask.source }}</span>
        </div>
        <div class="detail-row">
          <span class="label">{{ t('orchestration.status') }}:</span>
          <span class="value">{{ selectedTask.status }}</span>
        </div>
        <div class="detail-row">
          <span class="label">{{ t('orchestration.targetSessions') }}:</span>
          <span class="value">{{ t('orchestration.sessionsCount', { count: selectedTask.targetSessionIds.length }) }}</span>
        </div>
        <div v-if="selectedTask.error" class="detail-row error-row">
          <span class="label">{{ t('orchestration.error') }}:</span>
          <span class="value error">{{ selectedTask.error }}</span>
        </div>
        <!-- Outputs -->
        <div v-if="selectedTaskOutputs.length > 0" class="outputs-list">
          <h4>{{ t('orchestration.agentOutputs') }}</h4>
          <div v-for="output in selectedTaskOutputs" :key="output.sessionId" class="output-card" :class="output.status">
            <div class="output-header">
              <span class="output-agent">{{ output.agentName }}</span>
              <span>{{ output.status === 'completed' ? '✅' : output.status === 'failed' ? '❌' : '⏳' }}</span>
            </div>
            <div v-if="output.thought" class="output-thought"><em>{{ output.thought }}</em></div>
            <div class="output-content">{{ output.content }}</div>
            <div v-if="output.error" class="output-error">{{ output.error }}</div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.orchestration-view {
  padding: 16px;
  max-width: 1400px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
  overflow-y: auto;
}

.view-header {
  display: flex;
  align-items: center;
  gap: 16px;
}

.view-header h1 {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.subtitle {
  color: var(--text-muted);
  font-size: 14px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.tasks-summary {
  margin-bottom: 8px;
}

.task-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 8px;
}

.task-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-light);
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.task-card:hover {
  border-color: var(--primary);
  transform: translateY(-1px);
}

.task-card.selected {
  border-color: var(--primary);
  background: var(--primary-light);
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

.card-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.status-label {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-muted);
}

.card-title {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 4px;
}

.card-meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-muted);
}

.empty-state {
  text-align: center;
  padding: 32px;
  color: var(--text-muted);
  grid-column: 1 / -1;
}

.icon-large {
  font-size: 48px;
  display: block;
  margin-bottom: 12px;
}

.hint {
  font-size: 13px;
  color: var(--text-subtle);
}

.plan-section {
  background: var(--bg-surface);
  border-radius: 12px;
  padding: 12px;
  min-height: 200px;
}

.task-detail-section {
  background: var(--bg-surface);
  border-radius: 12px;
  padding: 16px;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.detail-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.btn-close {
  border: none;
  background: none;
  cursor: pointer;
  font-size: 20px;
  color: var(--text-muted);
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-row {
  display: flex;
  gap: 8px;
  font-size: 13px;
}

.detail-row .label {
  color: var(--text-muted);
  min-width: 80px;
}

.detail-row .value {
  color: var(--text-primary);
}

.error-row .value.error {
  color: #ef4444;
}

.outputs-list {
  margin-top: 12px;
}

.outputs-list h4 {
  font-size: 14px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.output-card {
  border: 1px solid var(--border-light);
  border-radius: 6px;
  padding: 8px 12px;
  margin-bottom: 8px;
}

.output-card.completed {
  border-left: 3px solid #22c55e;
}

.output-card.failed {
  border-left: 3px solid #ef4444;
}

.output-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 13px;
}

.output-agent {
  font-weight: 600;
  color: var(--primary);
}

.output-thought {
  font-size: 12px;
  opacity: 0.6;
  margin-bottom: 4px;
  padding-left: 8px;
  border-left: 2px solid currentColor;
}

.output-content {
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 13px;
}

.output-error {
  color: #ef4444;
  font-size: 12px;
  margin-top: 4px;
}
</style>
