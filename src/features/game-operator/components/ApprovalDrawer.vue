<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ApprovalDecision, ApprovalRequest } from '@/types/operator'

const { t } = useI18n()

defineProps<{
  approvals: ApprovalRequest[]
}>()

const emit = defineEmits<{
  approve: [approvalId: string, decision: ApprovalDecision]
}>()

const levelColors: Record<string, string> = {
  silent: '#6B7280',
  notify: '#3B82F6',
  approve: '#F59E0B',
  forbidden: '#EF4444',
}

function decisionLabel(decision: ApprovalDecision): string {
  return t(`approvalDecision.${decision}`)
}

function decisionClass(decision: ApprovalDecision): string {
  if (decision === 'approve') return 'btn-approve'
  if (decision === 'reject') return 'btn-reject'
  return 'btn-change'
}

function approvalOptions(approval: ApprovalRequest): ApprovalDecision[] {
  return approval.options?.length ? approval.options : ['approve', 'reject']
}
</script>

<template>
  <div class="approval-drawer">
    <h3>{{ t('gameOperator.title') }}</h3>
    <div v-if="approvals.length === 0" class="empty-state">
      {{ t('gameOperator.noPendingApprovals') }}
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
            :aria-label="`${t('a11y.approvalLevel')}: ${approval.level}`"
          >
            {{ approval.level.toUpperCase() }}
          </div>
          <div
            class="approval-action"
            :aria-label="`${t('a11y.approvalAction')}: ${approval.action}`"
          >
            {{ approval.action }}
          </div>
        </div>
        <div
          class="approval-title"
          :aria-label="`${t('a11y.approvalTitle')}: ${approval.title}`"
        >
          {{ approval.title }}
        </div>
        <div
          class="approval-reason"
          :aria-label="`${t('a11y.approvalReason')}: ${approval.reason}`"
        >
          {{ approval.reason }}
        </div>
        <div v-if="approval.risk" class="approval-risk">
          <strong>{{ t('a11y.approvalRisk') }}:</strong> {{ approval.risk }}
        </div>
        <div v-if="approval.preview?.files" class="approval-files">
          <strong>{{ t('a11y.approvalFiles') }}:</strong>
          <ul>
            <li v-for="file in approval.preview.files" :key="file">
              {{ file }}
            </li>
          </ul>
        </div>
        <div
          v-if="approval.preview?.diffs?.length"
          class="approval-diffs"
        >
          <details
            v-for="fileDiff in approval.preview.diffs"
            :key="`${approval.approval_id}:${fileDiff.path}`"
            class="diff-file"
          >
            <summary :aria-label="`${t('a11y.diffPreview')}: ${fileDiff.path}`">
              <span class="diff-operation" :data-operation="fileDiff.operation">
                {{ fileDiff.operation }}
              </span>
              <code>{{ fileDiff.path }}</code>
            </summary>
            <pre><code>{{ fileDiff.diff }}</code></pre>
          </details>
        </div>
        <div class="approval-actions">
          <button
            v-for="decision in approvalOptions(approval)"
            :key="decision"
            @click="emit('approve', approval.approval_id, decision)"
            class="approval-btn"
            :class="decisionClass(decision)"
            :aria-label="`${t(`a11y.${decision}Button`)}: ${approval.title}`"
          >
            {{ decisionLabel(decision) }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.approval-drawer {
  height: auto;
  min-width: 0;
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
  min-width: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-height: calc(100vh - 200px);
}

.approval-card {
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  padding: 1rem;
  background: var(--bg-main);
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.approval-header {
  display: flex;
  align-items: center;
  min-width: 0;
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
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.approval-title {
  font-weight: 500;
  margin-bottom: 0.5rem;
  overflow-wrap: anywhere;
}

.approval-reason {
  font-size: 0.9rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
  overflow-wrap: anywhere;
}

.approval-risk {
  padding: 0.5rem;
  background: #FEF3C7;
  border-radius: 4px;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  overflow-wrap: anywhere;
}

.approval-files {
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
  overflow-wrap: anywhere;
}

.approval-files ul {
  margin-top: 0.25rem;
  padding-left: 1.5rem;
}

.approval-files li {
  overflow-wrap: anywhere;
}

.approval-diffs {
  margin-bottom: 0.75rem;
  border-top: 1px solid var(--border-color);
}

.diff-file {
  border-bottom: 1px solid var(--border-color);
}

.diff-file summary {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.5rem;
  padding: 0.55rem 0;
  cursor: pointer;
  font-size: 0.8rem;
}

.diff-file summary code {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-operation {
  flex: 0 0 auto;
  color: #166534;
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
}

.diff-operation[data-operation='replace'] {
  color: #92400E;
}

.diff-file pre {
  max-height: 280px;
  margin: 0 0 0.65rem;
  padding: 0.75rem;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: #111827;
  color: #E5E7EB;
  font-size: 0.72rem;
  line-height: 1.5;
  white-space: pre;
}

.approval-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.approval-btn {
  flex: 1 1 auto;
  min-width: 100px;
  min-height: 36px;
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

.btn-change {
  background: #F59E0B;
  color: white;
}

.approval-btn:hover {
  opacity: 0.9;
}

.approval-btn:focus {
  outline: 2px solid #3B82F6;
  outline-offset: 2px;
}

@media (max-width: 1024px) {
  .approval-drawer h3 {
    font-size: 1rem;
  }

  .approval-card {
    padding: 0.75rem;
  }

  .approval-actions {
    flex-direction: column;
  }

  .approval-btn {
    width: 100%;
  }
}

</style>
