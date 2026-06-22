// Loop Engine API — Frontend interface for Loop Engine Tauri commands
//
// Provides unified access to:
// - Goal execution with loop triggers (Self-Healing, Evolution)
// - Loop state visualization
// - Health metrics collection

import { invokeOrProxy } from './host';

// ============================================================================
// Types
// ============================================================================

export interface LoopState {
  active_goals: Record<string, GoalInfo>;
  anomalies: AnomalyInfo[];
  healing_actions: HealingActionInfo[];
  evolution_events: EvolutionEventInfo[];
  stats: LoopStats;
}

export interface GoalInfo {
  id: string;
  description: string;
  status: string;
  executor?: string;
  current_iteration: number;
}

export interface AnomalyInfo {
  id: string;
  anomaly_type: string;
  severity: string;
  target: string;
  health_score: number;
  detected_at: string;
  resolved_at?: string;
}

export interface HealingActionInfo {
  id: string;
  anomaly_id: string;
  action_type: string;
  status: string;
  result?: string;
}

export interface EvolutionEventInfo {
  id: string;
  event_type: string;
  goal_id: string;
  pattern?: string;
  fitness_score: number;
}

export interface LoopStats {
  goals_total: number;
  goals_converged: number;
  goals_failed: number;
  anomalies_detected: number;
  healings_triggered: number;
  healings_successful: number;
  evolutions_applied: number;
  patterns_learned: number;
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Execute a goal through the Loop Engine pipeline.
 * This triggers Self-Healing on failure and Evolution on success.
 */
export async function loopExecuteGoal(
  goalId: string,
  description: string,
  completionCondition: string,
  executor?: string
): Promise<{ goal_id: string; status: string; iterations: number }> {
  return invokeOrProxy('loop_execute_goal', {
    goal_id: goalId,
    description,
    completion_condition: completionCondition,
    executor,
  });
}

/**
 * Get the current Loop Engine state for visualization.
 */
export async function loopGetState(): Promise<LoopState> {
  return invokeOrProxy('loop_get_state');
}

/**
 * Get Loop Engine statistics summary.
 */
export async function loopGetStats(): Promise<LoopStats> {
  return invokeOrProxy('loop_get_stats');
}

/**
 * Update health metrics for anomaly detection.
 * Call this periodically (e.g., every 30 seconds) with current metrics.
 */
export async function loopUpdateMetrics(
  metrics: Record<string, number>
): Promise<void> {
  return invokeOrProxy('loop_update_metrics', { metrics });
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Check if Loop Engine is healthy (no active anomalies).
 */
export async function loopIsHealthy(): Promise<boolean> {
  const state = await loopGetState();
  return state.anomalies.filter(a => !a.resolved_at).length === 0;
}

/**
 * Get recent evolution events (last N).
 */
export async function loopGetRecentEvolutions(limit: number = 10): Promise<EvolutionEventInfo[]> {
  const state = await loopGetState();
  return state.evolution_events.slice(-limit);
}

/**
 * Get success rate (converged / total goals).
 */
export async function loopGetSuccessRate(): Promise<number> {
  const stats = await loopGetStats();
  if (stats.goals_total === 0) return 1.0;
  return stats.goals_converged / stats.goals_total;
}