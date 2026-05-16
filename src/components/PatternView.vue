<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { usePatternStore } from '../stores/error'
import { useI18n } from '@/locales'

const { t } = useI18n()

const patternStore = usePatternStore()

const selectedCategory = ref<string>('all')
const showAddForm = ref(false)
const newPatternName = ref('')
const newPatternDescription = ref('')
const newPatternCategory = ref('coding')
const newPatternExamples = ref('')

const categories = [
  { value: 'all', label: t('errorMonitor.all') },
  { value: 'coding', label: t('pattern.categoryCoding') },
  { value: 'architecture', label: t('pattern.categoryArchitecture') },
  { value: 'testing', label: t('pattern.categoryTesting') },
  { value: 'security', label: t('pattern.categorySecurity') },
  { value: 'performance', label: t('pattern.categoryPerformance') },
  { value: 'workflow', label: t('pattern.categoryWorkflow') },
]

const filteredPatterns = computed(() => {
  let filtered = patternStore.patterns
  if (selectedCategory.value !== 'all') {
    filtered = filtered.filter(p => p.category === selectedCategory.value)
  }
  return filtered.sort((a, b) => (b.usageCount ?? 0) - (a.usageCount ?? 0))
})

const topPatterns = computed(() => filteredPatterns.value.slice(0, 5))

async function loadPatterns() {
  await patternStore.loadPatterns()
}

async function handleAddPattern() {
  if (!newPatternName.value.trim() || !newPatternDescription.value.trim()) return

  await patternStore.savePattern(
    newPatternName.value.trim(),
    newPatternDescription.value.trim(),
    newPatternCategory.value,
    newPatternExamples.value.trim() || undefined
  )

  showAddForm.value = false
  newPatternName.value = ''
  newPatternDescription.value = ''
  newPatternCategory.value = 'coding'
  newPatternExamples.value = ''
}

async function handleUsePattern(patternId: string, success: boolean) {
  await patternStore.updatePatternUsage(patternId, success)
}

function getCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    coding: t('pattern.categoryCoding'),
    architecture: t('pattern.categoryArchitecture'),
    testing: t('pattern.categoryTesting'),
    security: t('pattern.categorySecurity'),
    performance: t('pattern.categoryPerformance'),
    workflow: t('pattern.categoryWorkflow'),
  }
  return labels[category] || category
}

function getSuccessRate(rate: number | null): string {
  if (rate === null) return 'N/A'
  return `${Math.round(rate * 100)}%`
}

onMounted(() => {
  loadPatterns()
})
</script>

<template>
  <div class="pattern-view">
    <!-- Top Patterns -->
    <div v-if="topPatterns.length > 0" class="top-patterns">
      <h3>{{ t('pattern.highFrequencyPatterns') }}</h3>
      <div class="top-list">
        <div v-for="p in topPatterns" :key="p.id" class="top-item">
          <span class="top-name">{{ p.name }}</span>
          <span class="top-usage">{{ p.usageCount }} {{ t('pattern.usageCount') }}</span>
          <span class="top-rate">{{ getSuccessRate(p.successRate) }}</span>
        </div>
      </div>
    </div>

    <!-- Filters -->
    <div class="filters">
      <div class="filter-group">
        <label>{{ t('pattern.category') }}:</label>
        <select v-model="selectedCategory">
          <option v-for="c in categories" :key="c.value" :value="c.value">{{ c.label }}</option>
        </select>
      </div>
      <button class="btn-add" @click="showAddForm = !showAddForm">
        {{ showAddForm ? t('memory.cancel') : t('pattern.newPattern') }}
      </button>
    </div>

    <!-- Add Form -->
    <div v-if="showAddForm" class="add-form">
      <input
        v-model="newPatternName"
        :placeholder="t('pattern.patternName')"
      />
      <div class="form-row">
        <label>{{ t('pattern.category') }}:</label>
        <select v-model="newPatternCategory">
          <option value="coding">{{ t('pattern.categoryCoding') }}</option>
          <option value="architecture">{{ t('pattern.categoryArchitecture') }}</option>
          <option value="testing">{{ t('pattern.categoryTesting') }}</option>
          <option value="security">{{ t('pattern.categorySecurity') }}</option>
          <option value="performance">{{ t('pattern.categoryPerformance') }}</option>
          <option value="workflow">{{ t('pattern.categoryWorkflow') }}</option>
        </select>
      </div>
      <textarea
        v-model="newPatternDescription"
        :placeholder="t('pattern.patternDescription')"
        rows="3"
      ></textarea>
      <textarea
        v-model="newPatternExamples"
        :placeholder="t('pattern.patternExamplesOptional')"
        rows="2"
      ></textarea>
      <div class="form-actions">
        <button class="btn-submit" @click="handleAddPattern" :disabled="!newPatternName.trim() || !newPatternDescription.trim()">
          {{ t('memory.save') }}
        </button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="patternStore.error" class="error-banner">
      <span>{{ patternStore.error }}</span>
      <button @click="patternStore.clearError">✕</button>
    </div>

    <!-- Loading -->
    <div v-if="patternStore.loading" class="loading">{{ t('pattern.loading') }}</div>

    <!-- Pattern List -->
    <div class="pattern-list">
      <div v-if="filteredPatterns.length === 0 && !patternStore.loading" class="empty-state">
        <p>{{ t('pattern.noPatterns') }}</p>
        <p class="hint">{{ t('pattern.noPatternsHint') }}</p>
      </div>

      <div v-for="pattern in filteredPatterns" :key="pattern.id" class="pattern-card">
        <div class="pattern-header">
          <span class="pattern-category">{{ getCategoryLabel(pattern.category) }}</span>
          <span class="pattern-usage">{{ pattern.usageCount }} {{ t('pattern.usageCount') }}</span>
          <span class="pattern-rate" :class="{ good: (pattern.successRate ?? 0) >= 0.7 }">
            {{ getSuccessRate(pattern.successRate) }}
          </span>
        </div>
        <div class="pattern-name">{{ pattern.name }}</div>
        <div class="pattern-description">{{ pattern.description }}</div>
        <div v-if="pattern.examples" class="pattern-examples">
          <span class="label">{{ t('pattern.example') }}:</span>
          <pre>{{ pattern.examples }}</pre>
        </div>
        <div class="pattern-actions">
          <button class="btn-use success" @click="handleUsePattern(pattern.id, true)">{{ t('pattern.success') }} ✓</button>
          <button class="btn-use fail" @click="handleUsePattern(pattern.id, false)">{{ t('pattern.fail') }} ✗</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pattern-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.top-patterns {
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
}

.top-patterns h3 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #666;
}

.top-list {
  display: flex;
  gap: 12px;
}

.top-item {
  flex: 1;
  padding: 8px;
  background: white;
  border-radius: 4px;
  border: 1px solid #ddd;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.top-name {
  font-weight: 600;
  font-size: 13px;
}

.top-usage {
  font-size: 11px;
  color: #666;
}

.top-rate {
  font-size: 11px;
  color: #4a90d9;
}

.filters {
  display: flex;
  gap: 16px;
  align-items: center;
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

.btn-add {
  padding: 8px 16px;
  border: 1px dashed #4a90d9;
  border-radius: 4px;
  background: transparent;
  color: #4a90d9;
  cursor: pointer;
}

.add-form {
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.add-form input,
.add-form textarea,
.add-form select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: inherit;
  font-size: 14px;
  margin-bottom: 8px;
}

.form-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-row label {
  font-size: 13px;
  color: #666;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
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

.loading {
  text-align: center;
  padding: 24px;
  color: #666;
}

.pattern-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state {
  text-align: center;
  padding: 48px;
  color: #666;
}

.pattern-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ddd;
  background: white;
}

.pattern-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 12px;
}

.pattern-category {
  padding: 2px 8px;
  background: #e5e7eb;
  border-radius: 12px;
  font-size: 11px;
}

.pattern-usage {
  color: #666;
}

.pattern-rate {
  padding: 2px 8px;
  background: #fef2f2;
  color: #dc2626;
  border-radius: 12px;
  font-size: 11px;
}

.pattern-rate.good {
  background: #f0fdf4;
  color: #16a34a;
}

.pattern-name {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 8px;
}

.pattern-description {
  white-space: pre-wrap;
  word-break: break-word;
  margin-bottom: 8px;
  font-size: 14px;
  color: #333;
}

.pattern-examples {
  padding: 8px;
  background: #f3f4f6;
  border-radius: 4px;
  margin-bottom: 8px;
}

.pattern-examples .label {
  font-size: 11px;
  color: #666;
  margin-bottom: 4px;
}

.pattern-examples pre {
  margin: 0;
  font-size: 12px;
  white-space: pre-wrap;
}

.pattern-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-use {
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.btn-use.success {
  border: 1px solid #16a34a;
  background: transparent;
  color: #16a34a;
}

.btn-use.success:hover {
  background: #16a34a;
  color: white;
}

.btn-use.fail {
  border: 1px solid #dc2626;
  background: transparent;
  color: #dc2626;
}

.btn-use.fail:hover {
  background: #dc2626;
  color: white;
}
</style>