<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { orchestrator, type Event } from '@/lib/orchestrator';
import { agentMatcher } from '@/lib/agent-matcher';

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

const taskStatusColor = (status: string): string => {
  const colors: Record<string, string> = {
    pending: 'text-gray-500', running: 'text-blue-500', completed: 'text-green-500',
    failed: 'text-red-500', blocked: 'text-orange-500',
  };
  return colors[status] || 'text-gray-500';
};

const agentStatusColor = (status: string): string => {
  const colors: Record<string, string> = {
    idle: 'bg-green-100 text-green-800', busy: 'bg-blue-100 text-blue-800', overloaded: 'bg-red-100 text-red-800',
  };
  return colors[status] || 'bg-gray-100 text-gray-800';
};

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

onUnmounted(() => {
  orchestrator.unsubscribe('task_complete', handleTaskComplete);
  orchestrator.unsubscribe('task_failed', handleTaskFailed);
  orchestrator.unsubscribe('review_result', handleReviewResult);
  orchestrator.unsubscribe('test_result', handleTestResult);
  if (intervalId) { clearInterval(intervalId); }
});
</script>

<template>
  <div class="hermes-dashboard p-4 bg-gray-50 min-h-screen">
    <header class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-900">Hermes Dashboard</h1>
        <p class="text-sm text-gray-500">Task Orchestration & Agent Management</p>
      </div>
      <div class="flex items-center gap-4">
        <label class="flex items-center gap-2">
          <span class="text-sm text-gray-600">Auto Refresh</span>
          <input type="checkbox" v-model="autoRefresh" @change="toggleAutoRefresh" class="rounded" />
        </label>
        <button @click="refreshData" class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600">Refresh</button>
      </div>
    </header>

    <div class="grid grid-cols-4 gap-4 mb-6">
      <div class="bg-white rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 mb-1">Overall Progress</div>
        <div class="text-3xl font-bold text-gray-900">{{ overallProgress }}%</div>
        <div class="mt-2 h-2 bg-gray-200 rounded">
          <div class="h-2 rounded bg-blue-500" :style="{ width: overallProgress + '%' }"></div>
        </div>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 mb-1">Running Tasks</div>
        <div class="text-3xl font-bold text-blue-600">{{ runningTasks }}</div>
        <div class="text-xs text-gray-400 mt-1">{{ tasks.length }} total</div>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 mb-1">System Load</div>
        <div class="text-3xl font-bold text-gray-900">{{ systemLoad }}%</div>
        <div class="text-xs text-gray-400 mt-1">{{ overloadedAgents }} overloaded</div>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 mb-1">QA Status</div>
        <div class="flex items-center gap-2 mt-2">
          <span class="px-2 py-1 rounded text-sm" :class="qaStatus.lastReview?.success ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'">
            Review: {{ qaStatus.lastReview?.success ? 'Pass' : (qaStatus.lastReview ? 'Fail' : 'N/A') }}
          </span>
          <span class="px-2 py-1 rounded text-sm" :class="qaStatus.lastValidation?.success ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'">
            Test: {{ qaStatus.lastValidation?.success ? 'Pass' : (qaStatus.lastValidation ? 'Fail' : 'N/A') }}
          </span>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-3 gap-6">
      <div class="col-span-2 bg-white rounded-lg shadow">
        <div class="p-4 border-b"><h2 class="text-lg font-semibold">Tasks</h2></div>
        <div class="p-4">
          <div v-if="tasks.length === 0" class="text-center text-gray-400 py-8">No active tasks.</div>
          <div v-else class="space-y-2">
            <div v-for="task in tasks" :key="task.id" @click="selectedTask = task.id"
              class="p-3 border rounded cursor-pointer hover:bg-gray-50"
              :class="selectedTask === task.id ? 'border-blue-500 bg-blue-50' : 'border-gray-200'">
              <div class="flex items-center justify-between">
                <span class="font-medium" :class="taskStatusColor(task.status)">{{ task.name }}</span>
                <span class="px-2 py-1 rounded text-xs" :class="taskStatusColor(task.status)">{{ task.status }}</span>
              </div>
              <div class="mt-2 text-xs text-gray-400">Progress: {{ task.progress }}%</div>
            </div>
          </div>
        </div>
      </div>
      <div class="bg-white rounded-lg shadow">
        <div class="p-4 border-b"><h2 class="text-lg font-semibold">Agents</h2></div>
        <div class="p-4">
          <div v-if="agents.length === 0" class="text-center text-gray-400 py-8">No registered agents.</div>
          <div v-else class="space-y-2">
            <div v-for="agent in agents" :key="agent.id" class="p-3 border rounded border-gray-200">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ agent.name }}</span>
                <span class="px-2 py-1 rounded text-xs" :class="agentStatusColor(agent.status)">{{ agent.status }}</span>
              </div>
              <div class="mt-2 text-xs text-gray-500">Load: {{ agent.load }} / {{ agent.maxLoad }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="selectedTask" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="selectedTask = null">
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md p-4">
        <div class="flex items-center justify-between mb-3">
          <h4 class="font-semibold">Task Detail</h4>
          <button @click="selectedTask = null" class="text-gray-400 hover:text-gray-600">✕</button>
        </div>
        <div class="text-sm"><span class="text-gray-500">ID:</span> <span class="ml-2">{{ selectedTask }}</span></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hermes-dashboard { font-family: system-ui, -apple-system, sans-serif; }
</style>