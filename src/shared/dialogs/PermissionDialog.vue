<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { PermissionRequest, PermissionOption } from '@/lib/types';
import { usePermissionRulesStore, type PermissionAction } from '@/stores/permission-rules';
import { useI18n } from '@/locales';

const { t } = useI18n();
const rulesStore = usePermissionRulesStore();

// Props - now supports multiple requests
const props = defineProps<{
  requests: PermissionRequest[];
  showRuleManager?: boolean;
}>();

const emit = defineEmits<{
  select: [requestId: string, optionId: string];
  batchSelect: [selectedItems: { requestId: string; optionId: string }[]];
  cancel: [];
  createRule: [toolKind: string, toolTitle: string, action: PermissionAction, locations?: { path: string }[]];
  openRuleManager: [];
}>();

// Internal state for batch selection
const selectedItems = ref<Set<string>>(new Set());
const selectedAction = ref<string>('allow_once');
const showBatchMode = ref(false);

// Check if any rule matches any request
const matchedRules = computed(() => {
  return props.requests.map(req => {
    const rule = rulesStore.matchRule(
      req.toolCall.kind,
      req.toolCall.title,
      req.toolCall.locations
    );
    return {
      requestId: req.sessionId,
      rule,
      action: rule?.action,
    };
  });
});

// Auto-approve requests that have matching "allow" rules
watch(matchedRules, (matches) => {
  for (const match of matches) {
    if (match.action === 'allow') {
      // Auto-select the allow option for this request
      const req = props.requests.find(r => r.sessionId === match.requestId);
      if (req) {
        const allowOption = req.options.find(o =>
          o.kind === 'allow_once' || o.kind === 'allow_always'
        );
        if (allowOption) {
          emit('select', match.requestId, allowOption.optionId);
        }
      }
    }
  }
}, { immediate: true });

// Group requests by tool kind for batch display
const groupedRequests = computed(() => {
  const groups: Record<string, PermissionRequest[]> = {};
  for (const req of props.requests) {
    const kind = req.toolCall.kind;
    if (!groups[kind]) {
      groups[kind] = [];
    }
    groups[kind].push(req);
  }
  return groups;
});

// Toggle selection for a request
function toggleSelection(requestId: string) {
  const newSet = new Set(selectedItems.value);
  if (newSet.has(requestId)) {
    newSet.delete(requestId);
  } else {
    newSet.add(requestId);
  }
  selectedItems.value = newSet;
}

// Toggle all requests in a group
function toggleGroup(kind: string) {
  const groupRequests = groupedRequests.value[kind] || [];
  const allSelected = groupRequests.every(r => selectedItems.value.has(r.sessionId));

  const newSet = new Set(selectedItems.value);
  if (allSelected) {
    // Remove all from group
    for (const req of groupRequests) {
      newSet.delete(req.sessionId);
    }
  } else {
    // Add all from group
    for (const req of groupRequests) {
      newSet.add(req.sessionId);
    }
  }
  selectedItems.value = newSet;
}

// Handle single request selection
function handleSelect(requestId: string, optionId: string) {
  emit('select', requestId, optionId);
}

// Handle batch approval
function handleBatchApprove() {
  if (selectedItems.value.size === 0) return;

  const items: { requestId: string; optionId: string }[] = [];
  for (const requestId of selectedItems.value) {
    const req = props.requests.find(r => r.sessionId === requestId);
    if (req) {
      // Find matching option for selected action
      const option = req.options.find(o => o.kind === selectedAction.value);
      if (option) {
        items.push({ requestId, optionId: option.optionId });
      }
    }
  }

  emit('batchSelect', items);
  selectedItems.value = new Set();
}

// Handle cancel
function handleCancel() {
  emit('cancel');
}

// Create rule from selection
function handleCreateRule(request: PermissionRequest, action: PermissionAction) {
  emit('createRule',
    request.toolCall.kind,
    request.toolCall.title,
    action,
    request.toolCall.locations
  );
}

// Check if request has a matching "reject" rule (should show warning)
function hasRejectRule(request: PermissionRequest): boolean {
  const match = matchedRules.value.find(m => m.requestId === request.sessionId);
  return match?.action === 'reject';
}

// Get option button class based on kind
function getOptionClass(kind: string): string {
  if (kind.startsWith('allow')) {
    return 'option-allow';
  }
  if (kind.startsWith('reject')) {
    return 'option-reject';
  }
  return 'option-other';
}

// Get recommended option for a request
function getRecommendedOption(request: PermissionRequest): PermissionOption | undefined {
  // For read operations, recommend allow_once
  if (request.toolCall.kind === 'read') {
    return request.options.find(o => o.kind === 'allow_once');
  }
  // For write operations, recommend reject_once (be cautious)
  if (request.toolCall.kind === 'write' || request.toolCall.kind === 'bash') {
    // Unless it's a known safe operation
    const title = request.toolCall.title.toLowerCase();
    if (title.includes('git status') || title.includes('ls') || title.includes('cat')) {
      return request.options.find(o => o.kind === 'allow_once');
    }
    return request.options.find(o => o.kind === 'reject_once');
  }
  return request.options.find(o => o.kind === 'allow_once');
}
</script>

<template>
  <div class="permission-overlay">
    <div class="permission-dialog" :class="{ 'batch-mode': showBatchMode || requests.length > 1 }">
      <div class="dialog-header">
        <span class="icon">🔐</span>
        <h3>{{ requests.length > 1 ? t('permission.batchTitle') : t('permission.title') }}</h3>
        <div class="header-actions">
          <button
            v-if="requests.length > 1"
            class="toggle-btn"
            @click="showBatchMode = !showBatchMode"
          >
            {{ showBatchMode ? t('permission.singleMode') : t('permission.batchMode') }}
          </button>
          <button
            class="rules-btn"
            @click="emit('openRuleManager')"
            title="管理权限规则"
          >
            ⚙️
          </button>
        </div>
      </div>

      <div class="dialog-content">
        <!-- Batch mode: grouped requests -->
        <template v-if="showBatchMode && requests.length > 1">
          <div class="batch-header">
            <p>{{ t('permission.batchHint') }}</p>
            <div class="batch-actions">
              <select v-model="selectedAction" class="action-select">
                <option value="allow_once">{{ t('permission.allowOnce') }}</option>
                <option value="allow_always">{{ t('permission.allowAlways') }}</option>
                <option value="reject_once">{{ t('permission.rejectOnce') }}</option>
                <option value="reject_always">{{ t('permission.rejectAlways') }}</option>
              </select>
              <button
                class="batch-btn"
                :disabled="selectedItems.size === 0"
                @click="handleBatchApprove"
              >
                {{ t('permission.approveSelected') }} ({{ selectedItems.size }})
              </button>
            </div>
          </div>

          <div class="grouped-requests">
            <div
              v-for="(groupRequests, kind) in groupedRequests"
              :key="kind"
              class="request-group"
            >
              <div class="group-header">
                <input
                  type="checkbox"
                  :checked="groupRequests.every(r => selectedItems.has(r.sessionId))"
                  @change="toggleGroup(kind)"
                />
                <span class="group-kind">{{ kind }}</span>
                <span class="group-count">{{ groupRequests.length }}</span>
              </div>
              <div class="group-items">
                <div
                  v-for="req in groupRequests"
                  :key="req.sessionId"
                  class="request-item"
                  :class="{ 'has-reject-rule': hasRejectRule(req), 'selected': selectedItems.has(req.sessionId) }"
                >
                  <input
                    type="checkbox"
                    :checked="selectedItems.has(req.sessionId)"
                    @change="toggleSelection(req.sessionId)"
                  />
                  <div class="item-info">
                    <span class="item-title">{{ req.toolCall.title }}</span>
                    <div v-if="req.toolCall.locations?.length" class="item-locations">
                      <span v-for="(loc, i) in req.toolCall.locations.slice(0, 2)" :key="i">
                        📁 {{ loc.path }}
                      </span>
                      <span v-if="req.toolCall.locations.length > 2" class="more-locations">
                        +{{ req.toolCall.locations.length - 2 }} {{ t('permission.moreFiles') }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>

        <!-- Single mode: detailed request display -->
        <template v-else>
          <div
            v-for="request in requests"
            :key="request.sessionId"
            class="request-detail"
            :class="{ 'has-reject-rule': hasRejectRule(request) }"
          >
            <div v-if="hasRejectRule(request)" class="rule-warning">
              ⚠️ {{ t('permission.ruleWarning') }}
            </div>

            <div class="tool-info">
              <span class="tool-title">{{ request.toolCall.title }}</span>
              <span class="tool-kind">{{ request.toolCall.kind }}</span>
            </div>

            <div v-if="request.toolCall.locations?.length" class="locations">
              <div
                v-for="(loc, index) in request.toolCall.locations"
                :key="index"
                class="location"
              >
                📁 {{ loc.path }}
              </div>
            </div>

            <!-- Quick rule creation -->
            <div class="quick-rule-actions">
              <button
                class="quick-rule-btn allow"
                @click="handleCreateRule(request, 'allow')"
                title="创建允许规则"
              >
                + {{ t('permission.createAllowRule') }}
              </button>
              <button
                class="quick-rule-btn reject"
                @click="handleCreateRule(request, 'reject')"
                title="创建拒绝规则"
              >
                + {{ t('permission.createRejectRule') }}
              </button>
            </div>
          </div>
        </template>
      </div>

      <div class="dialog-actions">
        <!-- Single mode actions -->
        <template v-if="!showBatchMode || requests.length <= 1">
          <template v-for="request in requests" :key="request.sessionId">
            <div class="request-actions">
              <button
                v-for="option in request.options"
                :key="option.optionId"
                :class="['option-btn', getOptionClass(option.kind), { 'recommended': option === getRecommendedOption(request) }]"
                @click="handleSelect(request.sessionId, option.optionId)"
              >
                {{ option.name }}
                <span v-if="option === getRecommendedOption(request)" class="recommended-badge">
                  {{ t('agentProgress.recommended') }}
                </span>
              </button>
            </div>
          </template>
          <button class="cancel-btn" @click="handleCancel">
            {{ t('common.cancel') }}
          </button>
        </template>

        <!-- Batch mode actions -->
        <template v-else>
          <button class="cancel-btn" @click="handleCancel">
            {{ t('permission.rejectAll') }}
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.permission-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.permission-dialog {
  background: var(--bg-dialog, #fff);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  max-width: 480px;
  width: 90%;
  overflow: hidden;
}

.permission-dialog.batch-mode {
  max-width: 640px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  background: var(--bg-header, #f5f5f5);
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.dialog-header .icon {
  font-size: 1.5rem;
}

.dialog-header h3 {
  margin: 0;
  font-size: 1.1rem;
  flex: 1;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
}

.toggle-btn, .rules-btn {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color, #e0e0e0);
  background: transparent;
  border-radius: 4px;
  font-size: 0.8rem;
  cursor: pointer;
}

.toggle-btn:hover, .rules-btn:hover {
  background: var(--bg-hover, #f0f0f0);
}

.dialog-content {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.batch-header {
  margin-bottom: 1rem;
  padding: 0.75rem;
  background: var(--bg-sidebar, #f5f5f5);
  border-radius: 6px;
}

.batch-header p {
  margin: 0 0 0.5rem 0;
  font-size: 0.85rem;
  color: var(--text-muted, #666);
}

.batch-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.action-select {
  padding: 0.5rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 4px;
  background: var(--bg-main, #fff);
  font-size: 0.85rem;
}

.batch-btn {
  padding: 0.5rem 1rem;
  background: var(--bg-success, #28a745);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.85rem;
  cursor: pointer;
}

.batch-btn:disabled {
  background: var(--bg-muted, #ccc);
  cursor: not-allowed;
}

.grouped-requests {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.request-group {
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-sidebar, #f5f5f5);
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.group-header input[type="checkbox"] {
  cursor: pointer;
}

.group-kind {
  font-weight: 600;
  text-transform: capitalize;
}

.group-count {
  font-size: 0.8rem;
  color: var(--text-muted, #666);
  background: var(--bg-badge, #e0e0e0);
  padding: 0.1rem 0.4rem;
  border-radius: 10px;
}

.group-items {
  display: flex;
  flex-direction: column;
}

.request-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--border-color-light, #f0f0f0);
}

.request-item:last-child {
  border-bottom: none;
}

.request-item.selected {
  background: var(--bg-selected, #e6f7ff);
}

.request-item.has-reject-rule {
  background: var(--bg-danger-light, #fff5f5);
}

.request-item input[type="checkbox"] {
  margin-top: 0.2rem;
  cursor: pointer;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-weight: 500;
  font-size: 0.9rem;
  display: block;
}

.item-locations {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  margin-top: 0.25rem;
}

.item-locations span {
  font-family: monospace;
  font-size: 0.75rem;
  color: var(--text-muted, #666);
}

.more-locations {
  color: var(--text-secondary, #999);
}

.request-detail {
  margin-bottom: 1rem;
}

.request-detail:last-child {
  margin-bottom: 0;
}

.request-detail.has-reject-rule {
  border: 1px solid var(--border-danger, #dc3545);
  border-radius: 4px;
  padding: 0.5rem;
  background: var(--bg-danger-light, #fff5f5);
}

.rule-warning {
  padding: 0.5rem;
  background: var(--bg-warning, #fff3cd);
  border-radius: 4px;
  margin-bottom: 0.5rem;
  font-size: 0.85rem;
  color: var(--text-warning, #856404);
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tool-title {
  font-weight: 600;
  font-size: 1rem;
}

.tool-kind {
  font-size: 0.875rem;
  color: var(--text-muted, #666);
  text-transform: capitalize;
}

.locations {
  margin-top: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-hover, #f5f5f5);
  border-radius: 4px;
  border: 1px solid var(--border-color, #e0e0e0);
}

.location {
  font-family: monospace;
  font-size: 0.8rem;
  padding: 0.125rem 0;
  color: var(--text-primary, #333);
  word-break: break-all;
}

.quick-rule-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.quick-rule-btn {
  padding: 0.25rem 0.5rem;
  border: 1px solid;
  border-radius: 4px;
  font-size: 0.75rem;
  cursor: pointer;
  background: transparent;
}

.quick-rule-btn.allow {
  border-color: var(--bg-success, #28a745);
  color: var(--bg-success, #28a745);
}

.quick-rule-btn.allow:hover {
  background: var(--bg-success, #28a745);
  color: white;
}

.quick-rule-btn.reject {
  border-color: var(--bg-danger, #dc3545);
  color: var(--bg-danger, #dc3545);
}

.quick-rule-btn.reject:hover {
  background: var(--bg-danger, #dc3545);
  color: white;
}

.dialog-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 1rem;
  border-top: 1px solid var(--border-color, #e0e0e0);
  background: var(--bg-footer, #fafafa);
}

.request-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  width: 100%;
}

.option-btn {
  flex: 1;
  min-width: 100px;
  padding: 0.625rem 1rem;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
}

.option-allow {
  background: var(--bg-success, #28a745);
  color: white;
}

.option-allow:hover {
  background: var(--bg-success-hover, #218838);
}

.option-reject {
  background: var(--bg-danger, #dc3545);
  color: white;
}

.option-reject:hover {
  background: var(--bg-danger-hover, #c82333);
}

.option-other {
  background: var(--bg-secondary, #6c757d);
  color: white;
}

.option-btn.recommended {
  border: 2px solid var(--bg-primary, #0066cc);
  box-shadow: 0 0 0 2px rgba(0, 102, 204, 0.2);
}

.recommended-badge {
  font-size: 0.7rem;
  background: var(--bg-primary, #0066cc);
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
}

.cancel-btn {
  flex: 1;
  min-width: 120px;
  padding: 0.625rem 1rem;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  background: var(--bg-button, #fff);
  font-size: 0.9rem;
  cursor: pointer;
}

.cancel-btn:hover {
  background: var(--bg-hover, #f0f0f0);
}
</style>