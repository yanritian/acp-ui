<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useHistoryStore } from '@/stores/history'
import { useI18n } from '@/locales'

const { t } = useI18n()

const historyStore = useHistoryStore()

const searchKeyword = ref('')
const selectedStatus = ref<string[]>([])
const showDetailModal = ref(false)
const selectedTaskId = ref<string | null>(null)

// 计算属性
const records = computed(() => historyStore.records)
const loading = computed(() => historyStore.loading)
const statistics = computed(() => historyStore.statistics)

// 状态选项
const statusOptions = [
  { value: 'success', label: t('history.statusSuccess'), color: 'green' },
  { value: 'failed', label: t('history.statusFailed'), color: 'red' },
  { value: 'running', label: t('history.statusRunning'), color: 'blue' },
  { value: 'pending', label: t('history.statusPending'), color: 'gray' },
]

// 加载记录
async function loadRecords() {
  await historyStore.loadRecords({
    status: selectedStatus.value.length > 0 ? selectedStatus.value as ('success' | 'failed' | 'running' | 'pending' | 'cancelled')[] : undefined,
    keyword: searchKeyword.value || undefined,
    limit: 50,
  })
}

// 搜索
async function handleSearch() {
  await historyStore.search(searchKeyword.value)
}

// 切换状态筛选
async function toggleStatus(status: string) {
  const index = selectedStatus.value.indexOf(status)
  if (index === -1) {
    selectedStatus.value.push(status)
  } else {
    selectedStatus.value.splice(index, 1)
  }
  await loadRecords()
}

// 查看详情
function viewDetail(taskId: string) {
  selectedTaskId.value = taskId
  showDetailModal.value = true
  historyStore.loadTaskDetail(taskId)
}

// 关闭详情
function closeDetail() {
  showDetailModal.value = false
  selectedTaskId.value = null
}

// 删除记录
async function deleteRecord(taskId: string) {
  await historyStore.deleteRecord(taskId)
}

// 导出
async function exportHistory(format: 'json' | 'csv' | 'markdown') {
  const content = await historyStore.exportHistory(format)
  const ext = format === 'json' ? 'json' : format === 'csv' ? 'csv' : 'md'
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `history-${Date.now()}.${ext}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

// 格式化时间
function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString()
}

// 获取状态图标
function getStatusIcon(status: string): string {
  switch (status) {
    case 'success': return '✅'
    case 'failed': return '❌'
    case 'running': return '🔄'
    case 'pending': return '⏳'
    case 'cancelled': return '⚠️'
    default: return '❓'
  }
}

// 获取状态类
function getStatusClass(status: string): string {
  return `status-${status}`
}

// 获取渠道图标
function getSourceIcon(source: string): string {
  switch (source) {
    case 'app': return '💻'
    case 'feishu': return '📱'
    case 'telegram': return '✈️'
    case 'discord': return '🎮'
    case 'web': return '🌐'
    default: return '❓'
  }
}

onMounted(() => {
  loadRecords()
  historyStore.loadStatistics()
})
</script>

<template>
  <div class="history-view">
    <!-- 搜索和筛选 -->
    <div class="search-bar">
      <input
        v-model="searchKeyword"
        type="text"
        :placeholder="t('history.searchPlaceholder')"
        @keyup.enter="handleSearch"
      />
      <button @click="handleSearch">{{ t('history.search') }}</button>

      <div class="filter-group">
        <span
          v-for="option in statusOptions"
          :key="option.value"
          :class="['filter-btn', { active: selectedStatus.includes(option.value) }]"
          :style="{ borderColor: option.color }"
          @click="toggleStatus(option.value)"
        >
          {{ option.label }}
        </span>
      </div>

      <div class="export-group">
        <button @click="exportHistory('json')">{{ t('history.exportJson') }}</button>
        <button @click="exportHistory('markdown')">{{ t('history.exportMarkdown') }}</button>
      </div>
    </div>

    <!-- 统计概览 -->
    <div v-if="statistics" class="statistics">
      <div class="stat-item">
        <span class="stat-value">{{ statistics.totalTasks }}</span>
        <span class="stat-label">{{ t('history.totalTasks') }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ statistics.successRate.toFixed(1) }}%</span>
        <span class="stat-label">{{ t('history.successRate') }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ Math.round(statistics.averageDurationMs / 1000) }}{{ t('history.seconds') }}</span>
        <span class="stat-label">{{ t('history.averageDuration') }}</span>
      </div>
    </div>

    <!-- 任务列表 -->
    <div class="task-list">
      <div v-if="loading" class="loading">{{ t('history.loading') }}</div>

      <div
        v-for="record in records"
        :key="record.id"
        class="task-card"
        :class="getStatusClass(record.status)"
      >
        <div class="task-header">
          <span class="task-status">{{ getStatusIcon(record.status) }}</span>
          <span class="task-name">{{ record.name }}</span>
          <span class="task-source">{{ getSourceIcon(record.source) }}</span>
        </div>

        <div class="task-meta">
          <span class="meta-item">{{ formatTime(record.createdAt) }}</span>
          <span class="meta-item" v-if="record.completedAt">
            {{ t('history.duration') }}: {{ Math.round((record.completedAt - record.createdAt) / 1000) }}{{ t('history.seconds') }}
          </span>
          <span class="meta-item" v-if="record.agents && record.agents.length > 0">
            {{ record.agents.map(a => a.agentName).join(', ') }}
          </span>
        </div>

        <div class="task-actions">
          <button @click="viewDetail(record.id)">{{ t('history.detail') }}</button>
          <button @click="deleteRecord(record.id)">{{ t('history.delete') }}</button>
        </div>
      </div>

      <div v-if="records.length === 0 && !loading" class="empty-state">
        {{ t('history.noRecords') }}
      </div>
    </div>

    <!-- 详情弹窗 -->
    <div v-if="showDetailModal" class="modal-overlay" @click.self="closeDetail">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ t('history.taskDetail') }}</h3>
          <button @click="closeDetail">✕</button>
        </div>

        <div v-if="historyStore.currentDetail" class="modal-body">
          <div class="detail-section">
            <h4>{{ t('history.basicInfo') }}</h4>
            <div class="detail-row">
              <span class="label">{{ t('history.taskId') }}:</span>
              <span class="value">{{ historyStore.currentDetail.id }}</span>
            </div>
            <div class="detail-row">
              <span class="label">{{ t('history.name') }}:</span>
              <span class="value">{{ historyStore.currentDetail.name }}</span>
            </div>
            <div class="detail-row">
              <span class="label">{{ t('history.status') }}:</span>
              <span class="value">{{ historyStore.currentDetail.status }}</span>
            </div>
            <div class="detail-row">
              <span class="label">{{ t('history.source') }}:</span>
              <span class="value">{{ historyStore.currentDetail.source }}</span>
            </div>
            <div class="detail-row">
              <span class="label">{{ t('history.createdAt') }}:</span>
              <span class="value">{{ formatTime(historyStore.currentDetail.createdAt) }}</span>
            </div>
            <div class="detail-row" v-if="historyStore.currentDetail.completedAt">
              <span class="label">{{ t('history.completedAt') }}:</span>
              <span class="value">{{ formatTime(historyStore.currentDetail.completedAt) }}</span>
            </div>
          </div>

          <div class="detail-section" v-if="historyStore.currentDetail.agents">
            <h4>{{ t('history.executionAgents') }}</h4>
            <div v-for="agent in historyStore.currentDetail.agents" :key="agent.agentId" class="agent-info">
              <span class="agent-name">{{ agent.agentName }}</span>
              <span class="agent-status">{{ agent.status }}</span>
            </div>
          </div>

          <div class="detail-section" v-if="historyStore.currentDetail.error">
            <h4>{{ t('history.errorMessage') }}</h4>
            <div class="error-message">
              {{ historyStore.currentDetail.error.message }}
            </div>
          </div>
        </div>

        <div v-else class="modal-body loading">
          {{ t('history.loading') }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.search-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.search-bar input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  width: 200px;
}

.search-bar button {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.filter-group {
  display: flex;
  gap: 4px;
}

.filter-btn {
  padding: 4px 12px;
  border-radius: 4px;
  border: 2px solid;
  background: white;
  cursor: pointer;
  font-size: 12px;
}

.filter-btn.active {
  background: #f0f0f0;
}

.export-group {
  display: flex;
  gap: 4px;
}

.export-group button {
  padding: 4px 12px;
  border-radius: 4px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  font-size: 12px;
}

.statistics {
  display: flex;
  gap: 24px;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 500;
}

.stat-label {
  font-size: 12px;
  color: #666;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.loading {
  text-align: center;
  padding: 24px;
  color: #666;
}

.task-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ddd;
  background: white;
}

.task-card.status-success {
  border-color: #4caf50;
}

.task-card.status-failed {
  border-color: #f44336;
}

.task-card.status-running {
  border-color: #2196f3;
}

.task-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.task-status {
  font-size: 16px;
}

.task-name {
  flex: 1;
  font-weight: 500;
}

.task-source {
  font-size: 16px;
}

.task-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #666;
  margin-bottom: 8px;
}

.task-actions {
  display: flex;
  gap: 8px;
}

.task-actions button {
  padding: 4px 12px;
  border-radius: 4px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  font-size: 12px;
}

.empty-state {
  text-align: center;
  padding: 48px;
  color: #666;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-content {
  background: white;
  border-radius: 12px;
  max-width: 600px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #ddd;
}

.modal-header h3 {
  margin: 0;
}

.modal-header button {
  padding: 4px 8px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 20px;
}

.modal-body {
  padding: 16px;
}

.detail-section {
  margin-bottom: 16px;
}

.detail-section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #666;
}

.detail-row {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
}

.label {
  color: #666;
  font-size: 12px;
  width: 100px;
}

.value {
  font-size: 12px;
}

.agent-info {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
}

.error-message {
  padding: 8px;
  background: #ffebee;
  border-radius: 4px;
  color: #f44336;
}
</style>