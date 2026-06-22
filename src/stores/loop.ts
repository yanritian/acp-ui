// Loop Store — Vue reactive state for Loop Engine
//
// Manages:
// - Loop state (goals, anomalies, healings, evolutions)
// - Statistics display
// - Goal execution actions
// - Health monitoring integration

import { ref, computed, type Ref } from 'vue';
import {
  loopGetState,
  loopGetStats,
  loopExecuteGoal,
  loopUpdateMetrics,
  type LoopState,
  type LoopStats,
  type GoalInfo,
  type AnomalyInfo,
  type EvolutionEventInfo,
} from '@/lib/loop-api';

// ============================================================================
// Store State
// ============================================================================

const state: Ref<LoopState | null> = ref(null);
const isLoading = ref(false);
const error: Ref<string | null> = ref(null);
const lastRefresh: Ref<number> = ref(0);

// ============================================================================
// Computed Properties
// ============================================================================

const activeGoals = computed(() => {
  if (!state.value) return [];
  return Object.entries(state.value.active_goals).map(([id, goal]) => ({
    ...goal,
    id,
  }));
});

const activeAnomalies = computed(() => {
  if (!state.value) return [];
  return state.value.anomalies;
});

const recentEvolutions = computed(() => {
  if (!state.value) return [];
  return state.value.evolution_events.slice(-10);
});

const stats = computed<LoopStats>(() => {
  if (!state.value) return {
    goals_total: 0,
    goals_converged: 0,
    goals_failed: 0,
    anomalies_detected: 0,
    healings_triggered: 0,
    healings_successful: 0,
    evolutions_applied: 0,
    patterns_learned: 0,
  };
  return state.value.stats;
});

const successRate = computed(() => {
  const s = stats.value;
  if (s.goals_total === 0) return 100;
  return Math.round((s.goals_converged / s.goals_total) * 100);
});

const healthScore = computed(() => {
  const s = stats.value;
  if (s.anomalies_detected === 0) return 100;
  const healed = s.healings_successful / s.anomalies_detected;
  return Math.round(healed * 100);
});

// ============================================================================
// Actions
// ============================================================================

/**
 * Refresh Loop state from backend.
 */
async function refreshState(): Promise<void> {
  isLoading.value = true;
  error.value = null;

  try {
    state.value = await loopGetState();
    lastRefresh.value = Date.now();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoading.value = false;
  }
}

/**
 * Execute a goal through the Loop Engine.
 */
async function executeGoal(
  goalId: string,
  description: string,
  completionCondition: string,
  executor?: string
): Promise<{ goal_id: string; status: string; iterations: number }> {
  isLoading.value = true;
  error.value = null;

  try {
    const result = await loopExecuteGoal(goalId, description, completionCondition, executor);
    // Refresh state after execution
    await refreshState();
    return result;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    throw e;
  } finally {
    isLoading.value = false;
  }
}

/**
 * Update health metrics for anomaly detection.
 */
async function updateHealthMetrics(metrics: Record<string, number>): Promise<void> {
  try {
    await loopUpdateMetrics(metrics);
    // Refresh to see new anomalies if detected
    await refreshState();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

/**
 * Start periodic health monitoring.
 * Collects metrics every 30 seconds.
 */
function startHealthMonitoring(intervalMs: number = 30000): () => void {
  const interval = setInterval(async () => {
    // Collect simulated metrics (in real app, would get from system)
    const metrics = {
      heartbeat: Math.random() * 100,
      memory: Math.random() * 50 + 50, // 50-100
      cpu: Math.random() * 30 + 20, // 20-50
      health: successRate.value,
      error_rate: stats.value.goals_failed / (stats.value.goals_total || 1) * 100,
    };
    await updateHealthMetrics(metrics);
  }, intervalMs);

  return () => clearInterval(interval);
}

// ============================================================================
// Store Export
// ============================================================================

export function useLoopStore() {
  return {
    // State
    state,
    isLoading,
    error,
    lastRefresh,

    // Computed
    activeGoals,
    activeAnomalies,
    recentEvolutions,
    stats,
    successRate,
    healthScore,

    // Actions
    refreshState,
    executeGoal,
    updateHealthMetrics,
    startHealthMonitoring,
  };
}

// Export types
export type { LoopState, LoopStats, GoalInfo, AnomalyInfo, EvolutionEventInfo };