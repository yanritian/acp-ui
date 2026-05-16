<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useErrorStore } from '../stores/error'
import { useI18n } from '@/locales'

const { t } = useI18n()

const errorStore = useErrorStore()

const selectedStatus = ref<string>('all')
const selectedCategory = ref<string>('all')
const showSolutionForm = ref(false)
const selectedErrorId = ref<string | null>(null)
const solutionApproach = ref('')
const solutionResult = ref('')
const solutionSuccess = ref(true)

const statuses = [
  { value: 'all', label: t('errorMonitor.all') },
  { value: 'open', label: t('errorMonitor.pending') },
  { value: 'resolved', label: t('errorMonitor.resolved') },
  { value: 'ignored', label: t('errorMonitor.ignored') },
]

const categories = [
  { value: 'all', label: t('errorMonitor.all') },
  { value: 'build_error', label: t('errorMonitor.buildError') },
  { value: 'runtime_error', label: t('errorMonitor.runtimeError') },
  { value: 'logic_error', label: t('errorMonitor.logicError') },
  { value: 'dependency_error', label: t('errorMonitor.dependencyError') },
  { value: 'config_error', label: t('errorMonitor.configError') },
]

const filteredErrors = computed(() => {
  let filtered = errorStore.errors
  if (selectedStatus.value !== 'all') {
    filtered = filtered.filter(e => e.status === selectedStatus.value)
  }
  if (selectedCategory.value !== 'all') {
    filtered = filtered.filter(e => e.category === selectedCategory.value)
  }
  return filtered
})

const openCount = computed(() => errorStore.errors.filter(e => e.status === 'open').length)
const resolvedCount = computed(() => errorStore.errors.filter(e => e.status === 'resolved').length)

async function loadErrors() {
  await errorStore.loadErrors()
}

async function handleResolveError(errorId: string) {
  selectedErrorId.value = errorId
  showSolutionForm.value = true
}

async function handleSubmitSolution() {
  if (!selectedErrorId.value || !solutionApproach.value.trim()) return

  const solutionId = await errorStore.saveSolution(
    solutionApproach.value.trim(),
    solutionResult.value.trim() || t('errorMonitor.successfullyResolved'),
    selectedErrorId.value,
    undefined,
    solutionSuccess.value,
    undefined
  )

  await errorStore.resolveError(selectedErrorId.value, solutionId)

  showSolutionForm.value = false
  selectedErrorId.value = null
  solutionApproach.value = ''
  solutionResult.value = ''
  solutionSuccess.value = true
}

function formatTime(s: string): string {
  return new Date(s).toLocaleString()
}

function getStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    open: t('errorMonitor.pending'),
    resolved: t('errorMonitor.resolved'),
    ignored: t('errorMonitor.ignored'),
  }
  return labels[status] || status
}

function getCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    build_error: t('errorMonitor.build'),
    runtime_error: t('errorMonitor.runtime'),
    logic_error: t('errorMonitor.logic'),
    dependency_error: t('errorMonitor.dependency'),
    config_error: t('errorMonitor.config'),
  }
  return labels[category] || category
}

onMounted(() => {
  loadErrors()
})
</script>

<template>
  <div class="error-view">
    <!-- Statistics -->
    <div class="stats-bar">
      <div class="stat-item open">
        <span class="stat-label">{{ t('errorMonitor.pending') }}</span>
        <span class="stat-value">{{ openCount }}</span>
      </div>
      <div class="stat-item resolved">
        <span class="stat-label">{{ t('errorMonitor.resolved') }}</span>
        <span class="stat-value">{{ resolvedCount }}</span>
      </div>
    </div>

    <!-- Filters -->
    <div class="filters">
      <div class="filter-group">
        <label>{{ t('errorMonitor.statusFilter') }}:</label>
        <select v-model="selectedStatus">
          <option v-for="s in statuses" :key="s.value" :value="s.value">{{ s.label }}</option>
        </select>
      </div>
      <div class="filter-group">
        <label>{{ t('errorMonitor.typeFilter') }}:</label>
        <select v-model="selectedCategory">
          <option v-for="c in categories" :key="c.value" :value="c.value">{{ c.label }}</option>
        </select>
      </div>
    </div>

    <!-- Solution Form -->
    <div v-if="showSolutionForm" class="solution-form">
      <h3>{{ t('errorMonitor.addSolution') }}</h3>
      <textarea
        v-model="solutionApproach"
        :placeholder="t('errorMonitor.solutionPlaceholder')"
        rows="3"
      ></textarea>
      <input
        v-model="solutionResult"
        :placeholder="t('errorMonitor.resultPlaceholder')"
      />
      <div class="form-row">
        <label>
          <input type="checkbox" v-model="solutionSuccess" />
          {{ t('errorMonitor.successfullyResolved') }}
        </label>
      </div>
      <div class="form-actions">
        <button class="btn-cancel" @click="showSolutionForm = false">{{ t('common.cancel') }}</button>
        <button class="btn-submit" @click="handleSubmitSolution" :disabled="!solutionApproach.trim()">{{ t('errorMonitor.submit') }}</button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="errorStore.error" class="error-banner">
      <span>{{ errorStore.error }}</span>
      <button @click="errorStore.clearError">✕</button>
    </div>

    <!-- Loading -->
    <div v-if="errorStore.loading" class="loading">{{ t('errorMonitor.loading') }}</div>

    <!-- Error List -->
    <div class="error-list">
      <div v-if="filteredErrors.length === 0 && !errorStore.loading" class="empty-state">
        <p>{{ t('errorMonitor.noErrors') }}</p>
        <p class="hint">{{ t('errorMonitor.normalOperation') }}</p>
      </div>

      <div v-for="err in filteredErrors" :key="err.id" class="error-card">
        <div class="error-header">
          <span class="error-category">{{ getCategoryLabel(err.category) }}</span>
          <span class="error-status" :class="err.status">{{ getStatusLabel(err.status) }}</span>
          <span class="error-time">{{ formatTime(err.createdAt) }}</span>
        </div>
        <div class="error-message">{{ err.message }}</div>
        <div v-if="err.context" class="error-context">
          <pre>{{ err.context }}</pre>
        </div>
        <div v-if="err.status === 'open'" class="error-actions">
          <button class="btn-resolve" @click="handleResolveError(err.id)">{{ t('errorMonitor.resolve') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.error-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.stats-bar {
  display: flex;
  gap: 16px;
}

.stat-item {
  padding: 8px 16px;
  border-radius: 8px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.stat-item.open {
  background: #fef2f2;
  color: #dc2626;
}

.stat-item.resolved {
  background: #f0fdf4;
  color: #16a34a;
}

.stat-label {
  font-size: 13px;
}

.stat-value {
  font-weight: 600;
  font-size: 18px;
}

.filters {
  display: flex;
  gap: 16px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group label {
  font-size: 13px;
  color: #666;
}

.filter-group select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.solution-form {
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.solution-form h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
}

.solution-form textarea,
.solution-form input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: inherit;
  font-size: 14px;
  margin-bottom: 8px;
}

.form-row {
  margin-bottom: 12px;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-cancel {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
}

.btn-submit {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #fee;
  color: #c00;
  font-size: 13px;
  border-radius: 4px;
}

.error-banner button {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}

.loading {
  text-align: center;
  padding: 24px;
  color: #666;
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state {
  text-align: center;
  padding: 48px;
  color: #666;
}

.empty-state .hint {
  font-size: 13px;
  margin-top: 8px;
  color: #16a34a;
}

.error-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ddd;
  background: white;
}

.error-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 12px;
}

.error-category {
  padding: 2px 8px;
  background: #e5e7eb;
  border-radius: 12px;
  font-size: 11px;
}

.error-status {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
}

.error-status.open {
  background: #fef2f2;
  color: #dc2626;
}

.error-status.resolved {
  background: #f0fdf4;
  color: #16a34a;
}

.error-status.ignored {
  background: #f3f4f6;
  color: #6b7280;
}

.error-time {
  color: #999;
}

.error-message {
  white-space: pre-wrap;
  word-break: break-word;
  margin-bottom: 8px;
  font-size: 14px;
}

.error-context {
  padding: 8px;
  background: #f3f4f6;
  border-radius: 4px;
  overflow-x: auto;
}

.error-context pre {
  margin: 0;
  font-size: 12px;
  white-space: pre-wrap;
}

.error-actions {
  display: flex;
  justify-content: flex-end;
}

.btn-resolve {
  padding: 4px 12px;
  border: 1px solid #4a90d9;
  border-radius: 4px;
  background: transparent;
  color: #4a90d9;
  cursor: pointer;
  font-size: 12px;
}

.btn-resolve:hover {
  background: #4a90d9;
  color: white;
}
</style>