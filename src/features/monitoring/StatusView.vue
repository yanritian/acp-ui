<script setup lang="ts">
import { useI18n } from '@/locales'
import { useTeamRuntimeStore } from '@/stores/team-runtime'

const { t } = useI18n()
const teamRuntime = useTeamRuntimeStore()
</script>

<template>
  <div class="view-container">
    <h3>{{ t('navigation.status') }}</h3>
    <div class="status-stats">
      <div class="stat-card">
        <span class="stat-value">{{ teamRuntime.activeTaskCount }}</span>
        <span class="stat-label">{{ t('common.runningTasks') }}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ teamRuntime.completedTaskCount }}</span>
        <span class="stat-label">{{ t('common.completedTasks') }}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ teamRuntime.taskList.length }}</span>
        <span class="stat-label">{{ t('common.totalTasks') }}</span>
      </div>
    </div>
    <div v-if="teamRuntime.taskList.length > 0" class="task-list">
      <div v-for="task in teamRuntime.taskList.slice(0, 10)" :key="task.id" class="task-item" :class="task.status">
        <span class="task-name">{{ task.title }}</span>
        <span class="task-source">{{ task.source }}</span>
        <span class="task-status">{{ task.status }}</span>
      </div>
    </div>
    <div v-else class="empty-state">
      <p>{{ t('common.noTasks') }}</p>
      <p class="hint">在"协作网络"或"Agent 团队"页面启动团队任务后，这里将显示实时状态。</p>
    </div>
  </div>
</template>

<style scoped>
.view-container {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.status-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
  margin: 16px 0;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--primary);
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
  font-size: 13px;
}

.task-item.running { border-left: 3px solid #3b82f6; }
.task-item.completed { border-left: 3px solid #22c55e; }
.task-item.failed { border-left: 3px solid #ef4444; }

.task-name { flex: 1; font-weight: 500; }
.task-source { color: var(--text-muted); font-size: 11px; }
.task-status {
  font-size: 11px;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--bg-subtle);
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: 12px;
  margin-top: 4px;
}
</style>
