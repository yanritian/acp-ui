<script setup lang="ts">
import { ref, computed } from 'vue'
import { useOrchestrationStore } from '@/stores/orchestration'
import { useI18n } from '@/locales'

const { t } = useI18n()
const orchestrationStore = useOrchestrationStore()

const pendingCount = computed(() => orchestrationStore.pendingApprovals.length)
const hasApprovals = computed(() => orchestrationStore.hasPendingApprovals)
const expanded = ref(false)

const approvals = computed(() => orchestrationStore.pendingApprovals)

function toggleExpand() {
  expanded.value = !expanded.value
}

async function approve(id: string) {
  await orchestrationStore.approveRequest(id)
}

async function reject(id: string) {
  await orchestrationStore.rejectRequest(id)
}
</script>

<template>
  <div v-if="hasApprovals" class="approval-banner">
    <div class="banner-header" @click="toggleExpand">
      <span class="banner-icon">🔔</span>
      <span class="banner-text">
        {{ t('approvals.pendingCount', { count: pendingCount }) }}
      </span>
      <span class="expand-icon">{{ expanded ? '▼' : '▶' }}</span>
    </div>

    <div v-if="expanded" class="approval-list">
      <div v-for="approval in approvals" :key="approval.id" class="approval-item">
        <div class="approval-info">
          <span class="approval-type">{{ approval.request_type }}</span>
          <span class="approval-desc">{{ approval.description }}</span>
        </div>
        <div class="approval-actions">
          <button class="approve-btn" @click="approve(approval.id)">
            {{ t('approvals.approve') }}
          </button>
          <button class="reject-btn" @click="reject(approval.id)">
            {{ t('approvals.reject') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.approval-banner {
  position: sticky;
  top: 0;
  z-index: 100;
  background: linear-gradient(90deg, #DBEAFE 0%, #BFDBFE 100%);
  border-bottom: 1px solid #3B82F6;
}

.banner-header {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.banner-icon {
  font-size: 18px;
}

.banner-text {
  color: #1E40AF;
  font-size: 14px;
  font-weight: 500;
}

.expand-icon {
  color: #1E40AF;
  font-size: 12px;
}

.approval-list {
  padding: 8px 16px;
  background: white;
  border-top: 1px solid #E2E8F0;
}

.approval-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  border-radius: 4px;
  background: var(--bg-surface);
  margin-bottom: 4px;
}

.approval-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.approval-type {
  font-weight: 600;
  font-size: 12px;
  color: var(--text-primary);
}

.approval-desc {
  font-size: 11px;
  color: var(--text-secondary);
}

.approval-actions {
  display: flex;
  gap: 4px;
}

.approve-btn {
  padding: 4px 8px;
  background: #10B981;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.reject-btn {
  padding: 4px 8px;
  background: #EF4444;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}
</style>