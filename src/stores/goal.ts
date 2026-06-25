// src/stores/goal.ts
// Goal-Driven Architecture Pinia Store (RFC-001)

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import {
  goalList,
  goalGetStatus,
  goalSubmit,
  goalCancel,
  goalGetGraphSummary,
  goalGetReady,
  goalAssignWorker,
  goalAddIteration,
  type Goal,
  type GoalGraphSummary,
  type IterationRecord,
} from '@/lib/goal-api';

export const useGoalStore = defineStore('goal', () => {
  // --- State ---
  const goals = ref<Goal[]>([]);
  const summary = ref<GoalGraphSummary>({
    total: 0,
    pending: 0,
    active: 0,
    evaluating: 0,
    converged: 0,
    iterating: 0,
    failed: 0,
    budget_exhausted: 0,
    max_iter_reached: 0,
    cancelled: 0,
  });
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // --- Computed ---
  const activeGoals = computed(() =>
    goals.value.filter(g => g.status.status === 'active')
  );
  const convergedGoals = computed(() =>
    goals.value.filter(g => g.status.status === 'converged')
  );
  const pendingGoals = computed(() =>
    goals.value.filter(g => g.status.status === 'pending')
  );
  const failedGoals = computed(() =>
    goals.value.filter(g => g.status.status === 'failed')
  );
  const readyGoals = computed(() =>
    goals.value.filter(g => g.status.status === 'pending' && g.depends_on.length === 0)
  );

  // --- Actions ---
  async function refreshGoals() {
    isLoading.value = true;
    error.value = null;
    try {
      goals.value = await goalList();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function refreshSummary() {
    try {
      summary.value = await goalGetGraphSummary();
    } catch (e) {
      console.warn('[Goal] Failed to load summary:', e);
    }
  }

  async function refreshReadyGoals() {
    try {
      const ready = await goalGetReady();
      // Update existing goals' ready status
      goals.value = goals.value.map(g => {
        const isReady = ready.some(r => r.id === g.id);
        return isReady ? { ...g, status: { status: 'active' as const } } : g;
      });
    } catch (e) {
      console.warn('[Goal] Failed to get ready goals:', e);
    }
  }

  async function submitGoal(goal: Goal) {
    isLoading.value = true;
    error.value = null;
    try {
      await goalSubmit(goal);
      await refreshGoals();
      await refreshSummary();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function cancelGoal(id: string) {
    try {
      await goalCancel(id);
      await refreshGoals();
      await refreshSummary();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function assignWorker(id: string, workerId: string) {
    try {
      await goalAssignWorker(id, workerId);
      await refreshGoals();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function addIteration(id: string, record: IterationRecord) {
    try {
      await goalAddIteration(id, record);
      await refreshGoals();
      await refreshSummary();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function getGoalStatus(id: string): Promise<Goal | null> {
    try {
      return await goalGetStatus(id);
    } catch (e) {
      console.warn('[Goal] Failed to get status:', e);
      return null;
    }
  }

  // Initialize store
  async function initialize() {
    await refreshGoals();
    await refreshSummary();
  }

  return {
    // State
    goals,
    summary,
    isLoading,
    error,
    // Computed
    activeGoals,
    convergedGoals,
    pendingGoals,
    failedGoals,
    readyGoals,
    // Actions
    refreshGoals,
    refreshSummary,
    refreshReadyGoals,
    submitGoal,
    cancelGoal,
    assignWorker,
    addIteration,
    getGoalStatus,
    initialize,
  };
});