// src/stores/orchestration.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import {
  listOrchestratorAgents,
  listActiveTasks,
  listTaskHistory,
  getPendingApprovals,
  getApprovalStats,
  executeTaskAuto,
  cancelTask,
  decideApproval,
  seedDefaultAgents,
  type OrchestratorAgent,
  type OrchestratorTask,
  type ApprovalRequest,
  type ApprovalStats,
} from '@/lib/orchestration-api';

export const useOrchestrationStore = defineStore('orchestration', () => {
  // --- State ---
  const agents = ref<OrchestratorAgent[]>([]);
  const activeTasks = ref<OrchestratorTask[]>([]);
  const taskHistory = ref<OrchestratorTask[]>([]);
  const pendingApprovals = ref<ApprovalRequest[]>([]);
  const approvalStats = ref<ApprovalStats>({ total: 0, pending: 0, approved: 0, rejected: 0, expired: 0 });
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // --- Computed ---
  const runningTaskCount = computed(() =>
    activeTasks.value.filter(t => t.status === 'running').length
  );
  const hasPendingApprovals = computed(() => pendingApprovals.value.length > 0);

  // --- Actions ---
  async function refreshAgents() {
    try {
      agents.value = await listOrchestratorAgents();
    } catch (e) {
      console.warn('[Orchestration] Failed to load agents:', e);
    }
  }

  async function refreshActiveTasks() {
    try {
      activeTasks.value = await listActiveTasks();
    } catch (e) {
      console.warn('[Orchestration] Failed to load active tasks:', e);
    }
  }

  async function refreshTaskHistory() {
    try {
      taskHistory.value = await listTaskHistory();
    } catch (e) {
      console.warn('[Orchestration] Failed to load task history:', e);
    }
  }

  async function refreshApprovals() {
    try {
      pendingApprovals.value = await getPendingApprovals();
      approvalStats.value = await getApprovalStats();
    } catch (e) {
      console.warn('[Orchestration] Failed to load approvals:', e);
    }
  }

  async function refreshAll() {
    isLoading.value = true;
    error.value = null;
    try {
      await Promise.all([refreshAgents(), refreshActiveTasks(), refreshApprovals()]);
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function submitTask(description: string) {
    try {
      const task = await executeTaskAuto(description);
      activeTasks.value.push(task);
      return task;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function cancelActiveTask(taskId: string) {
    try {
      await cancelTask(taskId);
      const idx = activeTasks.value.findIndex(t => t.id === taskId);
      if (idx >= 0) activeTasks.value[idx].status = 'cancelled';
    } catch (e) {
      error.value = String(e);
    }
  }

  async function approveRequest(requestId: string, feedback?: string) {
    await decideApproval(requestId, 'approved', feedback);
    pendingApprovals.value = pendingApprovals.value.filter(r => r.id !== requestId);
    await refreshApprovals();
  }

  async function rejectRequest(requestId: string, feedback?: string) {
    await decideApproval(requestId, 'rejected', feedback);
    pendingApprovals.value = pendingApprovals.value.filter(r => r.id !== requestId);
    await refreshApprovals();
  }

  async function initDefaults() {
    try {
      await seedDefaultAgents();
      await refreshAgents();
    } catch (e) {
      console.warn('[Orchestration] Failed to seed defaults:', e);
    }
  }

  return {
    // state
    agents, activeTasks, taskHistory, pendingApprovals, approvalStats, isLoading, error,
    // computed
    runningTaskCount, hasPendingApprovals,
    // actions
    refreshAgents, refreshActiveTasks, refreshTaskHistory, refreshApprovals,
    refreshAll, submitTask, cancelActiveTask, approveRequest, rejectRequest, initDefaults,
  };
});