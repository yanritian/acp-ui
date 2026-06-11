<script setup lang="ts">
// WorkerHealthPanel.vue - Worker健康状态面板
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useSwarmStore } from '@/stores/swarm'

const swarmStore = useSwarmStore()
const autoRefresh = ref(true)
const refreshInterval = ref<number | null>(null)

// 健康状态颜色
const healthColors: Record<string, string> = {
  healthy: '#10b981',
  busy: '#f59e0b',
  offline: '#ef4444',
  starting: '#3b82f6',
  stopping: '#6b7280',
  unhealthy: '#ef4444',
}

// 状态标签
const healthLabels: Record<string, string> = {
  healthy: '健康',
  busy: '忙碌',
  offline: '离线',
  starting: '启动中',
  stopping: '停止中',
  unhealthy: '异常',
}

// 计算属性
const swarmStatusText = computed(() => {
  switch (swarmStore.swarmStatus) {
    case 'no_workers': return '无Worker'
    case 'no_queen': return '无Queen'
    case 'executing': return '执行中'
    case 'ready': return '就绪'
    default: return '未知'
  }
})

const swarmStatusColor = computed(() => {
  switch (swarmStore.swarmStatus) {
    case 'no_workers': return '#ef4444'
    case 'no_queen': return '#f59e0b'
    case 'executing': return '#3b82f6'
    case 'ready': return '#10b981'
    default: return '#6b7280'
  }
})

// 初始化
onMounted(async () => {
  await swarmStore.loadWorkers()

  // 自动刷新
  if (autoRefresh.value) {
    refreshInterval.value = window.setInterval(() => {
      swarmStore.loadWorkers()
    }, 5000)
  }
})

// 清理
onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
  }
})

// 方法
async function refresh() {
  await swarmStore.loadWorkers()
}

async function setQueen(workerId: string) {
  swarmStore.setQueen(workerId)
}
</script>

<template>
  <div class="worker-health-panel p-4">
    <!-- 蜂群状态头部 -->
    <div class="swarm-header mb-6">
      <div class="swarm-status">
        <span class="status-dot" :style="{ backgroundColor: swarmStatusColor }"></span>
        <span class="status-text">{{ swarmStatusText }}</span>
      </div>
      <div class="worker-counts flex gap-4">
        <span class="count-item">
          健康: {{ swarmStore.healthyWorkerCount }}
        </span>
        <span class="count-item">
          空闲: {{ swarmStore.idleWorkers.length }}
        </span>
        <span class="count-item">
          繁忙: {{ swarmStore.busyWorkers.length }}
        </span>
      </div>
      <button @click="refresh" class="refresh-btn">刷新</button>
    </div>

    <!-- Queen信息 -->
    <div class="queen-section mb-6" v-if="swarmStore.queenWorker">
      <div class="queen-badge">
        <span class="queen-icon">👑</span>
        <span class="queen-id">{{ swarmStore.queenWorker.id }}</span>
        <span class="queen-type">{{ swarmStore.queenWorker.capabilities.workerType }}</span>
      </div>
    </div>

    <!-- Worker列表 -->
    <div class="workers-list">
      <div class="worker-card" v-for="worker in swarmStore.workers" :key="worker.id">
        <div class="worker-header">
          <span class="worker-id">{{ worker.id }}</span>
          <span class="health-badge" :style="{ backgroundColor: healthColors[worker.status?.health || 'offline'] }">
            {{ healthLabels[worker.status?.health || 'offline'] }}
          </span>
          <button v-if="worker.id !== swarmStore.queenWorkerId" @click="setQueen(worker.id)" class="queen-btn">
            设为Queen
          </button>
        </div>

        <div class="worker-info">
          <span class="worker-type">类型: {{ worker.capabilities.workerType }}</span>
          <span class="worker-pid" v-if="worker.status?.pid">PID: {{ worker.status.pid }}</span>
        </div>

        <div class="worker-stats">
          <span class="stat">完成任务: {{ worker.status?.tasksCompleted || 0 }}</span>
          <span class="stat">失败任务: {{ worker.status?.tasksFailed || 0 }}</span>
        </div>

        <div class="worker-current-task" v-if="worker.status?.currentTask">
          当前任务: {{ worker.status.currentTask }}
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-state" v-if="swarmStore.workers.length === 0">
      <p>暂无Worker</p>
      <p class="hint">请注册Codex或Claude Code Worker</p>
    </div>
  </div>
</template>

<style scoped>
.worker-health-panel {
  background: #1a1a2e;
  border-radius: 8px;
}

.swarm-header {
  background: #16213e;
  padding: 12px;
  border-radius: 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.swarm-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.status-text {
  color: #e0e0e0;
  font-weight: bold;
}

.count-item {
  color: #a0a0a0;
  font-size: 12px;
}

.refresh-btn {
  background: #3b82f6;
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
}

.queen-section {
  background: #0f3460;
  padding: 12px;
  border-radius: 6px;
}

.queen-badge {
  display: flex;
  align-items: center;
  gap: 12px;
}

.queen-icon {
  font-size: 20px;
}

.queen-id {
  color: #e0e0e0;
  font-weight: bold;
}

.queen-type {
  color: #a0a0a0;
}

.worker-card {
  background: #16213e;
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 8px;
  border: 1px solid #0f3460;
}

.worker-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.worker-id {
  color: #e0e0e0;
  font-weight: bold;
}

.health-badge {
  padding: 2px 8px;
  border-radius: 4px;
  color: white;
  font-size: 12px;
}

.queen-btn {
  background: #f59e0b;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.worker-info {
  display: flex;
  gap: 16px;
  color: #6b7280;
  font-size: 12px;
  margin-top: 8px;
}

.worker-stats {
  display: flex;
  gap: 16px;
  color: #a0a0a0;
  font-size: 12px;
  margin-top: 8px;
}

.worker-current-task {
  background: #0f3460;
  padding: 8px;
  border-radius: 4px;
  color: #3b82f6;
  font-size: 12px;
  margin-top: 8px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #6b7280;
}

.hint {
  color: #a0a0a0;
  font-size: 12px;
}
</style>