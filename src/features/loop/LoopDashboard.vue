<script setup lang="ts">
/**
 * Loop Dashboard — Visualization for Loop Engine status
 *
 * Displays:
 * - Goal execution statistics (total, converged, failed)
 * - Anomaly detection stats
 * - Healing actions
 * - Evolution events
 * - Real-time health score
 */

import { onMounted, onUnmounted, ref } from 'vue';
import { useLoopStore } from '@/stores/loop';

const {
  stats,
  successRate,
  healthScore,
  activeGoals,
  activeAnomalies,
  recentEvolutions,
  isLoading,
  error,
  refreshState,
  startHealthMonitoring,
} = useLoopStore();

// Health monitoring
const stopMonitoring = ref<(() => void) | null>(null);

onMounted(async () => {
  await refreshState();
  stopMonitoring.value = startHealthMonitoring(30000);
});

onUnmounted(() => {
  if (stopMonitoring.value) {
    stopMonitoring.value();
  }
});

// Manual refresh
async function handleRefresh() {
  await refreshState();
}

// Format helpers
function formatTimestamp(ts: number): string {
  return new Date(ts).toLocaleTimeString();
}

function formatEventType(type: string): string {
  const labels: Record<string, string> = {
    GoalConverged: '🎯 Goal Converged',
    PatternExtracted: '📝 Pattern Extracted',
    SkillGenerated: '⚡ Skill Generated',
    SkillEvolved: '🔄 Skill Evolved',
  };
  return labels[type] || type;
}

function formatSeverity(severity: string): string {
  const colors: Record<string, string> = {
    low: 'text-yellow-500',
    medium: 'text-orange-500',
    high: 'text-red-500',
    critical: 'text-red-700 font-bold',
  };
  return colors[severity] || 'text-gray-500';
}
</script>

<template>
  <div class="p-6 bg-gray-900 text-white rounded-lg">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-2xl font-bold">Loop Engine Dashboard</h2>
      <button
        @click="handleRefresh"
        :disabled="isLoading"
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded disabled:opacity-50"
      >
        {{ isLoading ? 'Refreshing...' : 'Refresh' }}
      </button>
    </div>

    <!-- Error display -->
    <div v-if="error" class="p-4 bg-red-900 rounded mb-4">
      Error: {{ error }}
    </div>

    <!-- Stats Grid -->
    <div class="grid grid-cols-4 gap-4 mb-6">
      <!-- Goals Stats -->
      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Goals</h3>
        <div class="text-3xl font-bold">{{ stats.goals_total }}</div>
        <div class="text-sm text-green-400">{{ stats.goals_converged }} converged</div>
        <div class="text-sm text-red-400">{{ stats.goals_failed }} failed</div>
      </div>

      <!-- Success Rate -->
      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Success Rate</h3>
        <div class="text-3xl font-bold text-green-400">{{ successRate }}%</div>
        <div class="text-sm text-gray-400">Goals converged</div>
      </div>

      <!-- Anomalies -->
      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Anomalies</h3>
        <div class="text-3xl font-bold text-orange-400">{{ stats.anomalies_detected }}</div>
        <div class="text-sm text-green-400">{{ stats.healings_successful }} healed</div>
        <div class="text-sm text-gray-400">{{ stats.healings_triggered }} triggered</div>
      </div>

      <!-- Health Score -->
      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Health Score</h3>
        <div class="text-3xl font-bold">{{ healthScore }}%</div>
        <div class="text-sm text-gray-400">System health</div>
      </div>
    </div>

    <!-- Evolution Stats -->
    <div class="grid grid-cols-2 gap-4 mb-6">
      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Evolutions</h3>
        <div class="text-xl font-bold text-blue-400">{{ stats.evolutions_applied }}</div>
        <div class="text-sm text-gray-400">Patterns learned: {{ stats.patterns_learned }}</div>
      </div>

      <div class="p-4 bg-gray-800 rounded">
        <h3 class="text-sm text-gray-400 mb-2">Active Goals</h3>
        <div class="text-xl font-bold">{{ activeGoals.length }}</div>
        <div class="text-sm text-gray-400">Currently executing</div>
      </div>
    </div>

    <!-- Active Anomalies -->
    <div v-if="activeAnomalies.length > 0" class="mb-6">
      <h3 class="text-lg font-bold mb-3 text-orange-400">⚠️ Active Anomalies</h3>
      <div class="space-y-2">
        <div
          v-for="anomaly in activeAnomalies"
          :key="anomaly.id"
          class="p-3 bg-gray-800 rounded flex items-center gap-4"
        >
          <span :class="formatSeverity(anomaly.severity)">{{ anomaly.severity.toUpperCase() }}</span>
          <span class="text-gray-300">{{ anomaly.anomaly_type }}</span>
          <span class="text-gray-400">→ {{ anomaly.target }}</span>
          <span class="text-gray-500">Health: {{ anomaly.health_score.toFixed(0) }}%</span>
        </div>
      </div>
    </div>

    <!-- Recent Evolutions -->
    <div v-if="recentEvolutions.length > 0" class="mb-6">
      <h3 class="text-lg font-bold mb-3 text-blue-400">🚀 Recent Evolutions</h3>
      <div class="space-y-2">
        <div
          v-for="event in recentEvolutions"
          :key="event.id"
          class="p-3 bg-gray-800 rounded"
        >
          <div class="flex items-center gap-3">
            <span class="text-blue-300">{{ formatEventType(event.event_type) }}</span>
            <span class="text-gray-400">Goal: {{ event.goal_id }}</span>
            <span class="text-green-400">Fitness: {{ event.fitness_score.toFixed(2) }}</span>
          </div>
          <div v-if="event.pattern" class="mt-1 text-sm text-gray-500">
            Pattern: {{ event.pattern }}
          </div>
        </div>
      </div>
    </div>

    <!-- Active Goals List -->
    <div v-if="activeGoals.length > 0">
      <h3 class="text-lg font-bold mb-3">📋 Active Goals</h3>
      <div class="space-y-2">
        <div
          v-for="goal in activeGoals"
          :key="goal.id"
          class="p-3 bg-gray-800 rounded"
        >
          <div class="flex items-center gap-3">
            <span class="text-blue-300">{{ goal.id }}</span>
            <span class="text-gray-400">{{ goal.status }}</span>
            <span class="text-gray-500">Iteration: {{ goal.current_iteration }}</span>
          </div>
          <div class="mt-1 text-sm text-gray-400">
            {{ goal.description }}
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="stats.goals_total === 0" class="text-center text-gray-400 py-8">
      No goals executed yet. Start by executing a goal through the Loop Engine.
    </div>
  </div>
</template>

<style scoped>
/* Additional styling if needed */
</style>