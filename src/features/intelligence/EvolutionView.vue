<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useEvolutionStore } from '@/stores/error'
import { useI18n } from '@/locales'

const { t } = useI18n()

const evolutionStore = useEvolutionStore()

const selectedType = ref<string>('all')
const selectedDomain = ref<string>('all')
const showAddForm = ref(false)
const newEvolutionType = ref<string>('improvement')
const newDomain = ref('')
const newReason = ref('')
const newBefore = ref('')
const newAfter = ref('')

const types = [
  { value: 'all', label: t('errorMonitor.all') },
  { value: 'improvement', label: t('evolution.improvement') },
  { value: 'regression', label: t('evolution.regression') },
  { value: 'discovery', label: t('evolution.discovery') },
]

const domains = computed(() => {
  const unique = new Set(evolutionStore.evolutions.map(e => e.domain))
  return [{ value: 'all', label: t('errorMonitor.all') }, ...Array.from(unique).map(d => ({ value: d, label: d }))]
})

const filteredEvolutions = computed(() => {
  let filtered = evolutionStore.evolutions
  if (selectedType.value !== 'all') {
    filtered = filtered.filter(e => e.evolutionType === selectedType.value)
  }
  if (selectedDomain.value !== 'all') {
    filtered = filtered.filter(e => e.domain === selectedDomain.value)
  }
  return filtered
})

const improvementCount = computed(() => evolutionStore.evolutions.filter(e => e.evolutionType === 'improvement').length)
const regressionCount = computed(() => evolutionStore.evolutions.filter(e => e.evolutionType === 'regression').length)
const discoveryCount = computed(() => evolutionStore.evolutions.filter(e => e.evolutionType === 'discovery').length)

async function loadEvolutions() {
  await evolutionStore.loadEvolutions()
}

async function handleAddEvolution() {
  if (!newReason.value.trim() || !newDomain.value.trim()) return

  await evolutionStore.saveEvolution(
    newEvolutionType.value,
    newDomain.value.trim(),
    newReason.value.trim(),
    newBefore.value.trim() || undefined,
    newAfter.value.trim() || undefined,
    undefined
  )

  showAddForm.value = false
  newEvolutionType.value = 'improvement'
  newDomain.value = ''
  newReason.value = ''
  newBefore.value = ''
  newAfter.value = ''
}

function formatTime(s: string): string {
  return new Date(s).toLocaleString()
}

function getTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    improvement: t('evolution.improvement'),
    regression: t('evolution.regression'),
    discovery: t('evolution.discovery'),
  }
  return labels[type] || type
}

onMounted(() => {
  loadEvolutions()
})
</script>

<template>
  <div class="evolution-view">
    <!-- Statistics -->
    <div class="stats-bar">
      <div class="stat-item improvement">
        <span class="stat-label">{{ t('evolution.improvement') }}</span>
        <span class="stat-value">{{ improvementCount }}</span>
      </div>
      <div class="stat-item regression">
        <span class="stat-label">{{ t('evolution.regression') }}</span>
        <span class="stat-value">{{ regressionCount }}</span>
      </div>
      <div class="stat-item discovery">
        <span class="stat-label">{{ t('evolution.discovery') }}</span>
        <span class="stat-value">{{ discoveryCount }}</span>
      </div>
    </div>

    <!-- Filters -->
    <div class="filters">
      <div class="filter-group">
        <label>{{ t('evolution.type') }}:</label>
        <select v-model="selectedType">
          <option v-for="t_item in types" :key="t_item.value" :value="t_item.value">{{ t_item.label }}</option>
        </select>
      </div>
      <div class="filter-group">
        <label>{{ t('evolution.domain') }}:</label>
        <select v-model="selectedDomain">
          <option v-for="d in domains" :key="d.value" :value="d.value">{{ d.label }}</option>
        </select>
      </div>
      <button class="btn-add" @click="showAddForm = !showAddForm">
        {{ showAddForm ? t('memory.cancel') : t('evolution.newEvolution') }}
      </button>
    </div>

    <!-- Add Form -->
    <div v-if="showAddForm" class="add-form">
      <div class="form-row">
        <label>{{ t('evolution.type') }}:</label>
        <select v-model="newEvolutionType">
          <option value="improvement">{{ t('evolution.improvement') }}</option>
          <option value="regression">{{ t('evolution.regression') }}</option>
          <option value="discovery">{{ t('evolution.discovery') }}</option>
        </select>
      </div>
      <input
        v-model="newDomain"
        :placeholder="t('evolution.domainPlaceholder')"
      />
      <textarea
        v-model="newReason"
        :placeholder="t('evolution.reasonPlaceholder')"
        rows="2"
      ></textarea>
      <textarea
        v-model="newBefore"
        :placeholder="t('evolution.beforePlaceholder')"
        rows="2"
      ></textarea>
      <textarea
        v-model="newAfter"
        :placeholder="t('evolution.afterPlaceholder')"
        rows="2"
      ></textarea>
      <div class="form-actions">
        <button class="btn-submit" @click="handleAddEvolution" :disabled="!newReason.trim() || !newDomain.trim()">
          {{ t('memory.save') }}
        </button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="evolutionStore.error" class="error-banner">
      <span>{{ evolutionStore.error }}</span>
      <button @click="evolutionStore.clearError">✕</button>
    </div>

    <!-- Loading -->
    <div v-if="evolutionStore.loading" class="loading">{{ t('evolution.loading') }}</div>

    <!-- Evolution List -->
    <div class="evolution-list">
      <div v-if="filteredEvolutions.length === 0 && !evolutionStore.loading" class="empty-state">
        <p>{{ t('evolution.noEvolutions') }}</p>
        <p class="hint">{{ t('evolution.noEvolutionsHint') }}</p>
      </div>

      <div v-for="evo in filteredEvolutions" :key="evo.id" class="evolution-card">
        <div class="evolution-header">
          <span class="evolution-type" :class="evo.evolutionType">{{ getTypeLabel(evo.evolutionType) }}</span>
          <span class="evolution-domain">{{ evo.domain }}</span>
          <span class="evolution-time">{{ formatTime(evo.createdAt) }}</span>
        </div>
        <div class="evolution-reason">{{ evo.reason }}</div>
        <div v-if="evo.before || evo.after" class="evolution-diff">
          <div v-if="evo.before" class="before">
            <span class="label">{{ t('evolution.before') }}:</span>
            <pre>{{ evo.before }}</pre>
          </div>
          <div v-if="evo.after" class="after">
            <span class="label">{{ t('evolution.after') }}:</span>
            <pre>{{ evo.after }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.evolution-view {
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

.stat-item.improvement {
  background: #f0fdf4;
  color: #16a34a;
}

.stat-item.regression {
  background: #fef2f2;
  color: #dc2626;
}

.stat-item.discovery {
  background: #eff6ff;
  color: #2563eb;
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
  font-size: 14px;
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

.evolution-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state {
  text-align: center;
  padding: 48px;
  color: #666;
}

.evolution-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ddd;
  background: white;
}

.evolution-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 12px;
}

.evolution-type {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.evolution-type.improvement {
  background: #f0fdf4;
  color: #16a34a;
}

.evolution-type.regression {
  background: #fef2f2;
  color: #dc2626;
}

.evolution-type.discovery {
  background: #eff6ff;
  color: #2563eb;
}

.evolution-domain {
  padding: 2px 8px;
  background: #e5e7eb;
  border-radius: 12px;
  font-size: 11px;
}

.evolution-time {
  color: #999;
}

.evolution-reason {
  white-space: pre-wrap;
  word-break: break-word;
  margin-bottom: 8px;
  font-size: 14px;
}

.evolution-diff {
  display: flex;
  gap: 16px;
}

.evolution-diff .before,
.evolution-diff .after {
  flex: 1;
  padding: 8px;
  border-radius: 4px;
}

.evolution-diff .before {
  background: #fef2f2;
}

.evolution-diff .after {
  background: #f0fdf4;
}

.evolution-diff .label {
  font-size: 11px;
  color: #666;
  margin-bottom: 4px;
}

.evolution-diff pre {
  margin: 0;
  font-size: 12px;
  white-space: pre-wrap;
}
</style>