<script setup lang="ts">
// GoalGraphView.vue - Goal状态图可视化
import { ref, computed } from 'vue'
import { useGoalStore } from '@/stores/goal'
import type { Goal, CompletionCondition } from '@/lib/goal-api'

const goalStore = useGoalStore()

// 新 Goal 表单状态
const showForm = ref(false)
const newGoal = ref({
  id: '',
  description: '',
  completionCondition: 'command_success',
  command: '',
  maxIterations: 5,
})

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

// 提交新 Goal
async function handleSubmitGoal() {
  if (!newGoal.value.id || !newGoal.value.description) {
    return
  }

  // 构建完成条件
  let completionCondition: CompletionCondition
  switch (newGoal.value.completionCondition) {
    case 'command_success':
      completionCondition = {
        type: 'command_success',
        command: newGoal.value.command || 'echo done',
        expectedExitCode: 0,
      }
      break
    case 'file_exists':
      completionCondition = {
        type: 'file_check',
        path: newGoal.value.command || 'output.txt',
        mustExist: true,
        contentContains: null,
      }
      break
    case 'output_contains':
      completionCondition = {
        type: 'output_contains',
        text: newGoal.value.command || 'success',
        caseSensitive: false,
      }
      break
    default:
      completionCondition = {
        type: 'command_success',
        command: 'echo done',
        expectedExitCode: 0,
      }
  }

  const goal: Goal = {
    id: newGoal.value.id,
    description: newGoal.value.description,
    status: { status: 'pending' },
    completionCondition,
    evaluator: { type: 'auto' },
    iterations: [],
    maxIterations: newGoal.value.maxIterations,
    dependencies: [],
    assignedWorker: null,
    tokenBudget: null,
    tokenUsed: 0,
    createdAt: Date.now(),
    startedAt: null,
    convergedAt: null,
    parentId: null,
  }

  await goalStore.submitGoal(goal)

  // 重置表单
  newGoal.value = {
    id: '',
    description: '',
    completionCondition: 'command_success',
    command: '',
    maxIterations: 5,
  }
  showForm.value = false
}

// 自动生成 ID
function generateId() {
  newGoal.value.id = `goal-${Date.now().toString(36)}`
}
</script>

<template>
  <div class="goal-graph-view p-4">
    <!-- 工具栏 -->
    <div class="toolbar flex gap-2 mb-4">
      <button @click="showForm = true" class="add-goal-btn">+ 新 Goal</button>
      <button @click="goalStore.refreshGoals" class="refresh-btn">刷新</button>
    </div>

    <!-- Goal 提交表单 -->
    <div class="goal-form" v-if="showForm">
      <div class="form-header">
        <h3>创建新 Goal</h3>
        <button @click="showForm = false" class="close-btn">×</button>
      </div>
      <div class="form-body">
        <div class="form-row">
          <label>ID:</label>
          <input v-model="newGoal.id" placeholder="goal-xxx" />
          <button @click="generateId" class="gen-btn">生成</button>
        </div>
        <div class="form-row">
          <label>描述:</label>
          <input v-model="newGoal.description" placeholder="Goal 描述" />
        </div>
        <div class="form-row">
          <label>完成条件:</label>
          <select v-model="newGoal.completionCondition">
            <option value="command_success">命令成功</option>
            <option value="file_exists">文件存在</option>
            <option value="output_contains">输出包含</option>
          </select>
        </div>
        <div class="form-row" v-if="newGoal.completionCondition === 'command_success'">
          <label>命令:</label>
          <input v-model="newGoal.command" placeholder="cargo test" />
        </div>
        <div class="form-row">
          <label>最大迭代:</label>
          <input type="number" v-model="newGoal.maxIterations" min="1" max="10" />
        </div>
        <button @click="handleSubmitGoal" class="submit-btn">提交</button>
      </div>
    </div>

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

.toolbar {
  display: flex;
  gap: 8px;
}

.add-goal-btn {
  background: #10b981;
  color: white;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.refresh-btn {
  background: #3b82f6;
  color: white;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.goal-form {
  background: #16213e;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  border: 1px solid #10b981;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.form-header h3 {
  color: #e0e0e0;
  margin: 0;
}

.close-btn {
  background: transparent;
  color: #6b7280;
  font-size: 20px;
  cursor: pointer;
}

.form-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.form-row label {
  color: #a0a0a0;
  min-width: 80px;
}

.form-row input,
.form-row select {
  background: #0f3460;
  color: #e0e0e0;
  padding: 8px;
  border-radius: 4px;
  border: 1px solid #1a1a2e;
  flex: 1;
}

.gen-btn {
  background: #6b7280;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
}

.submit-btn {
  background: #10b981;
  color: white;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  margin-top: 8px;
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