<script setup lang="ts">
// GoalGraphView.vue - Goal状态图可视化
import { computed } from 'vue'
import { useGoalStore } from '@/stores/goal'

const goalStore = useGoalStore()

// 计算属性：按状态分组
const goalsByStatus = computed(() => ({
  pending: goalStore.pendingGoals,
  active: goalStore.activeGoals,
  converged: goalStore.convergedGoals,
  failed: goalStore.failedGoals,
}))

// 状态颜色映射
const statusColors: Record<string, string> = {
  pending: '#6b7280',
  active: '#3b82f6',
  evaluating: '#f59e0b',
  converged: '#10b981',
  iterating: '#8b5cf6',
  failed: '#ef4444',
  budget_exhausted: '#f97316',
  cancelled: '#94a3b8',
}

// 状态标签映射
const statusLabels: Record<string, string> = {
  pending: '等待',
  active: '执行',
  evaluating: '评估',
  converged: '收敛',
  iterating: '迭代',
  failed: '失败',
  budget_exhausted: '预算耗尽',
  cancelled: '取消',
}

// 计算依赖关系
const dependencies = computed(() => {
  const deps: Array<{ from: string; to: string }> = []
  for (const goal of goalStore.goals) {
    for (const depId of goal.dependencies) {
      deps.push({ from: depId, to: goal.id })
    }
  }
  return deps
})
</script>

<template>
  <div class="goal-graph-view p-4">
    <!-- 状态摘要 -->
    <div class="summary-bar flex gap-4 mb-6">
      <div class="summary-item" v-for="[status, count] in Object.entries(goalStore.summary)" :key="status">
        <span class="status-dot" :style="{ backgroundColor: statusColors[status] || '#gray' }"></span>
        <span class="status-label">{{ statusLabels[status] || status }}</span>
        <span class="status-count">{{ count }}</span>
      </div>
    </div>

    <!-- Goal节点列表 -->
    <div class="goals-grid grid grid-cols-2 gap-4">
      <div class="goal-card" v-for="goal in goalStore.goals" :key="goal.id">
        <div class="goal-header">
          <span class="goal-id">{{ goal.id }}</span>
          <span class="goal-status" :style="{ backgroundColor: statusColors[goal.status.status] }">
            {{ statusLabels[goal.status.status] || goal.status.status }}
          </span>
        </div>
        <div class="goal-description">{{ goal.description }}</div>
        <div class="goal-meta">
          <span class="iteration">迭代: {{ goal.iterations.length }}/{{ goal.maxIterations }}</span>
          <span class="executor">执行器: {{ goal.assignedWorker || '未分配' }}</span>
        </div>
        <!-- 依赖关系 -->
        <div class="dependencies" v-if="goal.dependencies.length > 0">
          <span class="dep-label">依赖:</span>
          <span class="dep-item" v-for="dep in goal.dependencies" :key="dep">{{ dep }}</span>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-state" v-if="goalStore.goals.length === 0">
      <p>暂无Goal</p>
      <button @click="goalStore.initialize" class="refresh-btn">刷新</button>
    </div>
  </div>
</template>

<style scoped>
.goal-graph-view {
  background: #1a1a2e;
  border-radius: 8px;
}

.summary-bar {
  background: #16213e;
  padding: 12px;
  border-radius: 6px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  background: #0f3460;
  border-radius: 4px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-label {
  color: #a0a0a0;
  font-size: 12px;
}

.status-count {
  color: #e0e0e0;
  font-weight: bold;
}

.goal-card {
  background: #16213e;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #0f3460;
}

.goal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.goal-id {
  color: #e0e0e0;
  font-weight: bold;
}

.goal-status {
  padding: 2px 8px;
  border-radius: 4px;
  color: white;
  font-size: 12px;
}

.goal-description {
  color: #a0a0a0;
  font-size: 14px;
  margin-bottom: 12px;
}

.goal-meta {
  display: flex;
  gap: 16px;
  color: #6b7280;
  font-size: 12px;
}

.dependencies {
  margin-top: 12px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.dep-label {
  color: #6b7280;
  font-size: 12px;
}

.dep-item {
  background: #0f3460;
  padding: 2px 8px;
  border-radius: 4px;
  color: #a0a0a0;
  font-size: 12px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #6b7280;
}

.refresh-btn {
  background: #3b82f6;
  color: white;
  padding: 8px 16px;
  border-radius: 6px;
  margin-top: 16px;
}
</style>