<script setup lang="ts">
import type { ApprovalRequest } from '@/types/operator'

const props = defineProps<{
  approvals: ApprovalRequest[]
}>()

const emit = defineEmits<{
  approve: [approvalId: string, decision: 'approve' | 'reject']
}>()

const levelColors: Record<string, string> = {
  silent: '#6B7280',
  notify: '#3B82F6',
  approve: '#F59E0B',
  forbidden: '#EF4444',
}
</script>

<template>
  <div class="approval-drawer">
    <h3>Pending Approvals</h3>
    <div v-if="approvals.length === 0" class="empty-state">
      No pending approvals
    </div>
    <div v-else class="approval-list">
      <div
        v-for="approval in approvals"
        :key="approval.approval_id"
        class="approval-card"
      >
        <div class="approval-header">
          <div
            class="approval-level"
            :style="{ background: levelColors[approval.level] }"
          >
            {{ approval.level.toUpperCase() }}
          </div>
          <div class="approval-action">{{ approval.action }}</div>
        </div>
        <div class="approval-title">{{ approval.title }}</div>
        <div class="approval-reason">{{ approval.reason }}</div>
        <div v-if="approval.risk" class="approval-risk">
          <strong>Risk:</strong> {{ approval.risk }}
        </div>
        <div v-if="approval.preview?.files" class="approval-files">
          <strong>Files:</strong>
          <ul>
            <li v-for="file in approval.preview.files" :key="file">
              {{ file }}
            </li>
          </ul>
        </div>
        <div class="approval-actions">
          <button
            @click="emit('approve', approval.approval_id, 'approve')"
            class="btn-approve"
          >
            ✓ Approve
          </button>
          <button
            @click="emit('approve', approval.approval_id, 'reject')"
            class="btn-reject"
          >
            ✗ Reject
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.approval-drawer {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.approval-drawer h3 {
  margin-bottom: 1rem;
  font-size: 1.1rem;
}

.empty-state {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.approval-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.approval-card {
  padding: 1rem;
  background: var(--bg-main);
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.approval-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.approval-level {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  color: white;
  font-size: 0.75rem;
  font-weight: 600;
}

.approval-action {
  font-family: monospace;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.approval-title {
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.approval-reason {
  font-size: 0.9rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.approval-risk {
  padding: 0.5rem;
  background: #FEF3C7;
  border-radius: 4px;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
}

.approval-files {
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
}

.approval-files ul {
  margin-top: 0.25rem;
  padding-left: 1.5rem;
}

.approval-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-approve,
.btn-reject {
  flex: 1;
  padding: 0.5rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.btn-approve {
  background: #10B981;
  color: white;
}

.btn-reject {
  background: #EF4444;
  color: white;
}

.btn-approve:hover,
.btn-reject:hover {
  opacity: 0.9;
}
</style>
