<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { orchestrator, type Event } from '@/lib/orchestrator';
import { agentMatcher } from '@/lib/agent-matcher';
import { useI18n } from '@/locales';

const { t } = useI18n();

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
}

// State
const tasks = ref<TaskDisplay[]>([]);
const agents = ref<AgentDisplay[]>([]);
const qaStatus = ref<{
  lastReview?: { success: boolean; score: number; errors: number };
  lastValidation?: { success: boolean; coverage: number; passed: number };
}>({});
const selectedTask = ref<string | null>(null);
const autoRefresh = ref(true);
let intervalId: ReturnType<typeof setInterval> | null = null;

// Computed
const runningTasks = computed(() => tasks.value.filter(t => t.status === 'running').length);
const completedTasks = computed(() => tasks.value.filter(t => t.status === 'completed').length);
const overallProgress = computed(() => {
  const total = tasks.value.length;
  return total === 0 ? 0 : Math.round((completedTasks.value / total) * 100);
});
const systemLoad = computed(() => {
  const total = agents.value.reduce((sum, a) => sum + a.load, 0);
  const max = agents.value.reduce((sum, a) => sum + a.maxLoad, 0);
  return max === 0 ? 0 : Math.round((total / max) * 100);
});
const overloadedAgents = computed(() => agents.value.filter(a => a.status === 'overloaded').length);

const refreshData = () => {
  const registeredAgents = agentMatcher.getAllAgents();
  agents.value = registeredAgents.map(a => ({
    id: a.agentId,
    name: a.agentId.split('-')[0],
    load: a.currentLoad,
    maxLoad: a.maxLoad,
    status: a.currentLoad / a.maxLoad >= 0.8 ? 'overloaded' : a.currentLoad > 0 ? 'busy' : 'idle',
  }));
};

const handleTaskComplete = (event: Event) => {
  const payload = event.payload as { nodeId: string };
  const task = tasks.value.find(t => t.id === payload.nodeId);
  if (task) { task.status = 'completed'; task.progress = 100; }
  refreshData();
};

const handleTaskFailed = (event: Event) => {
  const payload = event.payload as { nodeId: string; error: string };
  const task = tasks.value.find(t => t.id === payload.nodeId);
  if (task) { task.status = 'failed'; task.error = payload.error; }
  refreshData();
};

const handleReviewResult = (event: Event) => {
  qaStatus.value.lastReview = event.payload as { success: boolean; score: number; errors: number };
};

const handleTestResult = (event: Event) => {
  qaStatus.value.lastValidation = event.payload as { success: boolean; coverage: number; passed: number };
};

const toggleAutoRefresh = () => {
  if (intervalId) { clearInterval(intervalId); intervalId = null; }
  if (autoRefresh.value) { intervalId = setInterval(refreshData, 2000); }
};

onMounted(() => {
  orchestrator.subscribe('task_complete', handleTaskComplete);
  orchestrator.subscribe('task_failed', handleTaskFailed);
  orchestrator.subscribe('review_result', handleReviewResult);
  orchestrator.subscribe('test_result', handleTestResult);
  refreshData();
  if (autoRefresh.value) { intervalId = setInterval(refreshData, 2000); }
});

function initializeMockData() {
  // Register mock agents
  const mockAgents = [
    { agentId: 'planner-001', capabilities: ['planning'], specialization: ['orchestration'], currentLoad: 2, maxLoad: 3, performanceScore: 90 },
    { agentId: 'architect-001', capabilities: ['architecture'], specialization: ['design'], currentLoad: 3, maxLoad: 4, performanceScore: 95 },
    { agentId: 'tddGuide-001', capabilities: ['testing'], specialization: ['quality'], currentLoad: 1, maxLoad: 3, performanceScore: 92 },
    { agentId: 'codeReviewer-001', capabilities: ['review'], specialization: ['qa'], currentLoad: 0, maxLoad: 2, performanceScore: 88 },
    { agentId: 'securityReviewer-001', capabilities: ['security'], specialization: ['audit'], currentLoad: 1, maxLoad: 2, performanceScore: 95 },
  ];

  mockAgents.forEach(agent => agentMatcher.registerAgent(agent));

  // Create mock tasks
  tasks.value = [
    { id: 'task-1', name: 'Parse Request', status: 'completed', agent: 'planner-001', progress: 100 },
    { id: 'task-2', name: 'Design Architecture', status: 'running', agent: 'architect-001', progress: 60 },
    { id: 'task-3', name: 'Write Tests', status: 'pending', agent: 'tddGuide-001', progress: 0 },
    { id: 'task-4', name: 'Implement Code', status: 'pending', agent: 'codeReviewer-001', progress: 0 },
    { id: 'task-5', name: 'Security Audit', status: 'pending', agent: 'securityReviewer-001', progress: 0 },
  ];

  // Set mock QA status
  qaStatus.value = {
    lastReview: { success: true, score: 95, errors: 2 },
    lastValidation: { success: true, coverage: 85, passed: 42 },
  };
}

onUnmounted(() => {
  orchestrator.unsubscribe('task_complete', handleTaskComplete);
  orchestrator.unsubscribe('task_failed', handleTaskFailed);
  orchestrator.unsubscribe('review_result', handleReviewResult);
  orchestrator.unsubscribe('test_result', handleTestResult);
  if (intervalId) { clearInterval(intervalId); }
});
</script>

<template>
  <div class="hermes-dashboard">
    <header class="hermes-header">
      <div>
        <h1 class="hermes-title">{{ t('hermes.title') }}</h1>
        <p class="hermes-subtitle">{{ t('hermes.subtitle') }}</p>
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
        <div class="stat-sub">{{ overloadedAgents }} {{ t('common.overloaded') }}</div>
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