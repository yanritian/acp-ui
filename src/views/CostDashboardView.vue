<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { costApi } from '@/api/agent-platform'
import { WebSocketService } from '@/api/agent-platform'
import type { CostSummary } from '@/api/types'

const costSummary = ref<CostSummary | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)
const wsService = new WebSocketService()

const budgetPercent = computed(() => {
  if (!costSummary.value) return 0
  return costSummary.value.budget_remaining_percent
})

const progressColor = computed(() => {
  const percent = budgetPercent.value
  if (percent > 50) return '#4caf50'
  if (percent > 20) return '#ff9800'
  return '#f44336'
})

async function fetchCost() {
  isLoading.value = true
  error.value = null

  try {
    costSummary.value = await costApi.getSummary()
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  fetchCost()

  // WebSocket for real-time updates
  wsService.connect()
  wsService.on('cost_update', (data) => {
    costSummary.value = data
  })
})

onUnmounted(() => {
  wsService.disconnect()
})

// Refresh every 30 seconds
const refreshInterval = setInterval(fetchCost, 30000)
onUnmounted(() => clearInterval(refreshInterval))
</script>

<template>
  <div class="cost-dashboard">
    <h2>成本追踪</h2>

    <div v-if="isLoading" class="loading">
      加载中...
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
      <button @click="fetchCost">重试</button>
    </div>

    <div v-else-if="costSummary" class="summary">
      <!-- Monthly Total -->
      <div class="cost-card monthly">
        <h3>本月成本</h3>
        <div class="amount">
          <span class="currency">¥</span>
          <span class="value">{{ costSummary.monthly_total.toFixed(2) }}</span>
        </div>
        <div class="remaining">
          预算剩余: {{ budgetPercent.toFixed(1) }}%
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{ width: (100 - budgetPercent) + '%', background: progressColor }"
          ></div>
        </div>
      </div>

      <!-- Daily Total -->
      <div class="cost-card daily">
        <h3>今日成本</h3>
        <div class="amount">
          <span class="currency">¥</span>
          <span class="value">{{ costSummary.daily_total.toFixed(4) }}</span>
        </div>
      </div>

      <!-- Currency -->
      <div class="cost-card currency-info">
        <h3>货币</h3>
        <span class="currency-value">{{ costSummary.currency }}</span>
      </div>
    </div>

    <!-- Budget Thresholds -->
    <div class="thresholds">
      <div class="threshold-item green">
        <span class="label">&gt; 50%</span>
        <span class="desc">预算充足</span>
      </div>
      <div class="threshold-item orange">
        <span class="label">20-50%</span>
        <span class="desc">注意监控</span>
      </div>
      <div class="threshold-item red">
        <span class="label">&lt; 20%</span>
        <span class="desc">即将耗尽</span>
      </div>
    </div>

    <!-- Refresh Button -->
    <button class="refresh-btn" @click="fetchCost">
      刷新数据
    </button>
  </div>
</template>

<style scoped>
.cost-dashboard {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}

.cost-dashboard h2 {
  text-align: center;
  margin-bottom: 20px;
}

.loading, .error {
  text-align: center;
  padding: 20px;
}

.error {
  color: #c00;
}

.error button {
  margin-top: 10px;
}

.summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 15px;
  margin-bottom: 20px;
}

.cost-card {
  background: white;
  padding: 15px;
  border-radius: 12px;
  border: 1px solid #e0e0e0;
  text-align: center;
}

.cost-card h3 {
  font-size: 14px;
  margin-bottom: 10px;
  color: #666;
}

.amount {
  font-size: 24px;
  font-weight: bold;
}

.amount .currency {
  font-size: 16px;
  color: #666;
}

.remaining {
  font-size: 12px;
  color: #888;
  margin-top: 8px;
}

.progress-bar {
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  margin-top: 10px;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: all 0.3s;
}

.thresholds {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.threshold-item {
  flex: 1;
  padding: 10px;
  border-radius: 8px;
  text-align: center;
}

.threshold-item.green {
  background: #e8f5e9;
}

.threshold-item.orange {
  background: #fff3e0;
}

.threshold-item.red {
  background: #ffebee;
}

.threshold-item .label {
  font-weight: bold;
  font-size: 14px;
}

.threshold-item .desc {
  font-size: 12px;
  color: #666;
}

.refresh-btn {
  width: 100%;
  padding: 12px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
}

.refresh-btn:hover {
  background: #e8e8e8;
}
</style>