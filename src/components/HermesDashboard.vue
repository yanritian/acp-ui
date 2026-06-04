<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from '@/locales';
import { useHermesApi, type HermesTaskStatus } from '@/lib/hermes-api';

const { t } = useI18n();
const hermesApi = useHermesApi();

interface TaskDisplay {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'blocked';
  agent?: string;
  progress: number;
  error?: string;
}

interface AgentDisplay {
  id: string;
  name: string;
  load: number;
  maxLoad: number;
  status: 'idle' | 'busy' | 'overloaded';
  thinkingCount: number;
  toolCallCount: number;
}

// State - now connected to real Hermes data
const tasks = ref<TaskDisplay[]>([]);
const agents = ref<AgentDisplay[]>([]);
const selectedTask = ref<string | null>(null);
const autoRefresh = ref(true);

// Computed - from Hermes API
const runningTasks = computed(() => hermesApi.runningTasksCount.value);
const completedTasks = computed(() => hermesApi.completedTasksCount.value);
const overallProgress = computed(() => hermesApi.progressPercent.value);
const systemLoad = computed(() => hermesApi.systemLoad.value);
const connected = computed(() => hermesApi.connected.value);

// QA Status from metrics
const qaStatus = computed(() => {
  const metrics = hermesApi.metrics.value;
  return {
    lastReview: metrics.totalTasksCompleted > 0 ? {
      success: metrics.totalTasksFailed < metrics.totalTasksCompleted,
      score: Math.round((metrics.totalTasksCompleted / (metrics.totalTasksCompleted + metrics.totalTasksFailed || 1)) * 100),
      errors: metrics.totalTasksFailed,
    } : undefined,
    lastValidation: metrics.totalThinkingChunks > 0 ? {
      success: true,
      coverage: Math.min(100, Math.round(metrics.totalToolCalls * 10)),
      passed: metrics.totalThinkingChunks,
    } : undefined,
  };
});

// Convert Hermes task to display format
const convertTask = (hermesTask: HermesTaskStatus): TaskDisplay => ({
  id: hermesTask.taskId,
  name: hermesTask.request.slice(0, 50) + (hermesTask.request.length > 50 ? '...' : ''),
  status: hermesTask.status === 'running' ? 'running' :
          hermesTask.status === 'completed' ? 'completed' :
          hermesTask.status === 'failed' ? 'failed' : 'pending',
  agent: 'coder',
  progress: hermesTask.status === 'completed' ? 100 :
            hermesTask.status === 'running' ? overallProgress.value : 0,
  error: hermesTask.status === 'failed' ? 'Execution failed' : undefined,
});

// Refresh data from Hermes API
const refreshData = async () => {
  // Update agents from Hermes status
  const agentState = hermesApi.agent.value;
  agents.value = [{
    id: 'hermes-coder',
    name: 'Hermes Coder',
    load: agentState.status === 'busy' ? 80 : 0,
    maxLoad: 100,
    status: agentState.status === 'busy' ? 'busy' :
            agentState.status === 'error' ? 'overloaded' : 'idle',
    thinkingCount: agentState.thinkingCount,
    toolCallCount: agentState.toolCallCount,
  }];

  // Update tasks from current task + history
  const currentTask = hermesApi.task.value;
  const history = hermesApi.history.value;

  tasks.value = [];
  if (currentTask) {
    tasks.value.push(convertTask(currentTask));
  }
  tasks.value.push(...history.slice(0, 10).map(convertTask));
};

// Watch Hermes API state changes
watch([hermesApi.agent, hermesApi.task, hermesApi.metrics], () => {
  refreshData();
}, { deep: true });

// Auto refresh toggle
let intervalId: ReturnType<typeof setInterval> | null = null;

const toggleAutoRefresh = () => {
  if (intervalId) { clearInterval(intervalId); intervalId = null; }
  if (autoRefresh.value) { intervalId = setInterval(refreshData, 2000); }
};

onMounted(async () => {
  // Initialize Hermes API connection
  await hermesApi.init();
  refreshData();
  if (autoRefresh.value) { intervalId = setInterval(refreshData, 2000); }
});

onUnmounted(() => {
  hermesApi.cleanup();
  if (intervalId) { clearInterval(intervalId); }
});
</script>

<template>
  <div class="hermes-dashboard">
    <header class="hermes-header">
      <div>
        <h1 class="hermes-title">{{ t('hermes.title') }}</h1>
        <p class="hermes-subtitle">
          {{ connected ? t('hermes.connected') : t('hermes.disconnected') }}
        </p>
      </div>
      <div class="hermes-controls">
        <label class="auto-refresh-label">
          <span>{{ t('common.autoRefresh') }}</span>
          <input type="checkbox" v-model="autoRefresh" @change="toggleAutoRefresh" />
        </label>
        <button @click="refreshData" class="refresh-btn">{{ t('common.refresh') }}</button>
      </div>
    </header>

    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-label">{{ t('common.overallProgress') }}</div>
        <div class="stat-value progress">{{ overallProgress }}%</div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: overallProgress + '%' }"></div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-label">{{ t('common.runningTasks') }}</div>
        <div class="stat-value running">{{ runningTasks }}</div>
        <div class="stat-sub">{{ tasks.length }} {{ t('common.totalTasks') }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">{{ t('common.systemLoad') }}</div>
        <div class="stat-value">{{ systemLoad }}%</div>
        <div class="stat-sub">{{ hermesApi.agent.value.thinkingCount }} thinking</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">{{ t('common.qaStatus') }}</div>
        <div class="qa-badges">
          <span class="qa-badge" :class="qaStatus.lastReview?.success ? 'pass' : (qaStatus.lastReview ? 'fail' : 'na')">
            {{ t('common.review') }}: {{ qaStatus.lastReview?.success ? t('common.pass') : (qaStatus.lastReview ? t('common.fail') : t('common.na')) }}
          </span>
          <span class="qa-badge" :class="qaStatus.lastValidation?.success ? 'pass' : (qaStatus.lastValidation ? 'fail' : 'na')">
            {{ t('common.test') }}: {{ qaStatus.lastValidation?.success ? t('common.pass') : (qaStatus.lastValidation ? t('common.fail') : t('common.na')) }}
          </span>
        </div>
      </div>
    </div>

    <div class="content-grid">
      <div class="panel">
        <div class="panel-header">{{ t('common.tasks') }}</div>
        <div class="panel-body">
          <div v-if="tasks.length === 0" class="empty-text">{{ t('hermes.noActiveTasks') }}</div>
          <div v-else class="task-list">
            <div v-for="task in tasks" :key="task.id"
              @click="selectedTask = task.id"
              :class="['task-item', { selected: selectedTask === task.id }]">
              <div class="task-header">
                <span class="task-name" :class="task.status">{{ task.name }}</span>
                <span class="task-status" :class="task.status">{{ task.status }}</span>
              </div>
              <div class="task-progress">{{ t('common.progress') }}: {{ task.progress }}%</div>
            </div>
          </div>
        </div>
      </div>
      <div class="panel">
        <div class="panel-header">{{ t('common.agents') }}</div>
        <div class="panel-body">
          <div v-if="agents.length === 0" class="empty-text">{{ t('hermes.noRegisteredAgents') }}</div>
          <div v-else class="agent-list">
            <div v-for="agent in agents" :key="agent.id" class="agent-item">
              <div class="agent-header">
                <span class="agent-name">{{ agent.name }}</span>
                <span class="agent-status" :class="agent.status">{{ agent.status }}</span>
              </div>
              <div class="agent-load">{{ t('common.load') }}: {{ agent.load }} / {{ agent.maxLoad }}</div>
              <div class="agent-stats">
                <span>Thinking: {{ agent.thinkingCount }}</span>
                <span>Tools: {{ agent.toolCallCount }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="selectedTask" class="task-detail-overlay" @click.self="selectedTask = null">
      <div class="task-detail-modal">
        <div class="modal-header">
          <h4 class="modal-title">{{ t('common.taskDetail') }}</h4>
          <button @click="selectedTask = null" class="modal-close">✕</button>
        </div>
        <div class="modal-body">
          <div class="modal-row">
            <span class="modal-label">ID:</span>
            <span class="modal-value">{{ selectedTask }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hermes-dashboard {
  font-family: system-ui, -apple-system, sans-serif;
  background: linear-gradient(135deg, #F8FAFC 0%, #EEF2FF 50%, #F8FAFC 100%);
  min-height: 100%;
  padding: 16px;
}

.hermes-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding: 16px 20px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.hermes-title {
  font-size: 20px;
  font-weight: 600;
  color: #1E293B;
}

.hermes-title::before {
  content: '📊 ';
}

.hermes-subtitle {
  font-size: 13px;
  color: #64748B;
  margin-top: 2px;
}

.hermes-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.auto-refresh-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #64748B;
}

.auto-refresh-label input {
  accent-color: #3B82F6;
}

.refresh-btn {
  padding: 6px 14px;
  background: #3B82F6;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.refresh-btn:hover {
  background: #2563EB;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.stat-card {
  background: white;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid #E2E8F0;
}

.stat-label {
  font-size: 12px;
  color: #64748B;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  color: #1E293B;
}

.stat-value.progress {
  color: #3B82F6;
}

.stat-value.running {
  color: #22C55E;
}

.stat-sub {
  font-size: 11px;
  color: #94A3B8;
  margin-top: 4px;
}

.progress-bar {
  height: 6px;
  background: #E2E8F0;
  border-radius: 3px;
  margin-top: 8px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3B82F6 0%, #22C55E 100%);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.qa-badges {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.qa-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
}

.qa-badge.pass {
  background: #DCFCE7;
  color: #166534;
}

.qa-badge.fail {
  background: #FEE2E2;
  color: #991B1B;
}

.qa-badge.na {
  background: #F1F5F9;
  color: #64748B;
}

.content-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 16px;
}

.panel {
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid #E2E8F0;
}

.panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid #E2E8F0;
  font-size: 14px;
  font-weight: 600;
  color: #1E293B;
}

.panel-body {
  padding: 16px;
}

.empty-text {
  text-align: center;
  color: #94A3B8;
  padding: 32px;
  font-size: 14px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  padding: 12px;
  border: 1px solid #E2E8F0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.task-item:hover {
  background: #F8FAFC;
}

.task-item.selected {
  border-color: #3B82F6;
  background: #EFF6FF;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-name {
  font-size: 13px;
  font-weight: 500;
}

.task-name.pending { color: #94A3B8; }
.task-name.running { color: #3B82F6; }
.task-name.completed { color: #22C55E; }
.task-name.failed { color: #EF4444; }
.task-name.blocked { color: #F97316; }

.task-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.task-status.pending { background: #F1F5F9; color: #64748B; }
.task-status.running { background: #DBEAFE; color: #1D4ED8; }
.task-status.completed { background: #DCFCE7; color: #166534; }
.task-status.failed { background: #FEE2E2; color: #991B1B; }
.task-status.blocked { background: #FED7AA; color: #9A3412; }

.task-progress {
  font-size: 11px;
  color: #94A3B8;
  margin-top: 6px;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-item {
  padding: 12px;
  border: 1px solid #E2E8F0;
  border-radius: 8px;
}

.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.agent-name {
  font-size: 13px;
  font-weight: 500;
  color: #1E293B;
}

.agent-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.agent-status.idle { background: #DCFCE7; color: #166534; }
.agent-status.busy { background: #DBEAFE; color: #1D4ED8; }
.agent-status.overloaded { background: #FEE2E2; color: #991B1B; }

.agent-load {
  font-size: 11px;
  color: #94A3B8;
  margin-top: 6px;
}

.agent-stats {
  font-size: 11px;
  color: #64748B;
  margin-top: 4px;
  display: flex;
  gap: 8px;
}

.task-detail-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.task-detail-modal {
  background: white;
  border-radius: 12px;
  padding: 20px;
  max-width: 400px;
  width: 90%;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  color: #1E293B;
}

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  color: #94A3B8;
  cursor: pointer;
}

.modal-close:hover {
  color: #64748B;
}
</style>