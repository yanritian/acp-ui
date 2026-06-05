<script setup lang="ts">
import { ref, computed } from 'vue';
import { usePermissionRulesStore, type PermissionRule, type PermissionAction, type PermissionScope } from '@/stores/permission-rules';
import { useI18n } from '@/locales';

const { t } = useI18n();
const rulesStore = usePermissionRulesStore();

const emit = defineEmits<{
  close: [];
}>();

// Form state
const showAddForm = ref(false);
const editingRule = ref<string | null>(null);
const formName = ref('');
const formDescription = ref('');
const formPattern = ref('');
const formAction = ref<PermissionAction>('allow');
const formScope = ref<PermissionScope>('tool');
const formToolKind = ref('');
const formEnabled = ref(true);
const formError = ref('');
const isSubmitting = ref(false);

// Filter state
const filterScope = ref<PermissionScope | 'all'>('all');
const filterAction = ref<PermissionAction | 'all'>('all');
const filterEnabled = ref<'all' | 'enabled' | 'disabled'>('all');

// Filtered rules
const filteredRules = computed(() => {
  return rulesStore.rules.filter(rule => {
    if (filterScope.value !== 'all' && rule.scope !== filterScope.value) return false;
    if (filterAction.value !== 'all' && rule.action !== filterAction.value) return false;
    if (filterEnabled.value === 'enabled' && !rule.enabled) return false;
    if (filterEnabled.value === 'disabled' && rule.enabled) return false;
    return true;
  });
});

// Statistics
const stats = computed(() => ({
  total: rulesStore.rules.length,
  enabled: rulesStore.enabledRules.length,
  allow: rulesStore.rules.filter(r => r.action === 'allow').length,
  reject: rulesStore.rules.filter(r => r.action === 'reject').length,
}));

// Reset form
function resetForm() {
  formName.value = '';
  formDescription.value = '';
  formPattern.value = '';
  formAction.value = 'allow';
  formScope.value = 'tool';
  formToolKind.value = '';
  formEnabled.value = true;
  formError.value = '';
  showAddForm.value = false;
  editingRule.value = null;
}

// Start add form
function startAdd() {
  resetForm();
  showAddForm.value = true;
}

// Start edit form
function startEdit(rule: PermissionRule) {
  resetForm();
  editingRule.value = rule.id;
  formName.value = rule.name;
  formDescription.value = rule.description || '';
  formPattern.value = rule.pattern;
  formAction.value = rule.action;
  formScope.value = rule.scope;
  formToolKind.value = rule.toolKind || '';
  formEnabled.value = rule.enabled;
}

// Validate regex pattern
function isValidRegex(pattern: string): boolean {
  try {
    new RegExp(pattern, 'i');
    return true;
  } catch {
    return false;
  }
}

// Submit form
async function handleSubmit() {
  formError.value = '';

  if (!formName.value.trim()) {
    formError.value = t('permission.ruleNameRequired');
    return;
  }

  if (!formPattern.value.trim()) {
    formError.value = t('permission.patternRequired');
    return;
  }

  if (!isValidRegex(formPattern.value)) {
    formError.value = t('permission.invalidPattern');
    return;
  }

  isSubmitting.value = true;

  try {
    if (editingRule.value) {
      rulesStore.updateRule(editingRule.value, {
        name: formName.value.trim(),
        description: formDescription.value.trim(),
        pattern: formPattern.value.trim(),
        action: formAction.value,
        scope: formScope.value,
        toolKind: formToolKind.value.trim() || undefined,
        enabled: formEnabled.value,
      });
    } else {
      rulesStore.addRule({
        name: formName.value.trim(),
        description: formDescription.value.trim(),
        pattern: formPattern.value.trim(),
        action: formAction.value,
        scope: formScope.value,
        toolKind: formToolKind.value.trim() || undefined,
        enabled: formEnabled.value,
      });
    }
    resetForm();
  } catch (e) {
    formError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isSubmitting.value = false;
  }
}

// Toggle rule enabled
function toggleRule(id: string) {
  rulesStore.toggleRule(id);
}

// Delete rule
function handleDelete(id: string) {
  if (!confirm(t('permission.deleteRuleConfirm'))) return;
  rulesStore.deleteRule(id);
}

// Reset to defaults
function handleResetDefaults() {
  if (!confirm(t('permission.resetDefaultsConfirm'))) return;
  rulesStore.resetToDefaults();
}

// Get scope label
function getScopeLabel(scope: PermissionScope): string {
  const labels: Record<PermissionScope, string> = {
    tool: t('permission.scopeTool'),
    path: t('permission.scopePath'),
    pattern: t('permission.scopePattern'),
  };
  return labels[scope];
}

// Get action label
function getActionLabel(action: PermissionAction): string {
  const labels: Record<PermissionAction, string> = {
    allow: t('permission.actionAllow'),
    reject: t('permission.actionReject'),
  };
  return labels[action];
}

// Get action icon
function getActionIcon(action: PermissionAction): string {
  return action === 'allow' ? '✓' : '✗';
}

// Format date
function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}
</script>

<template>
  <div class="rules-overlay" @click.self="emit('close')">
    <div class="rules-panel">
      <div class="rules-header">
        <h2>{{ t('permission.rulesTitle') }}</h2>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="rules-content">
        <!-- Statistics -->
        <div class="stats-section">
          <div class="stat-item">
            <span class="stat-value">{{ stats.total }}</span>
            <span class="stat-label">{{ t('permission.totalRules') }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ stats.enabled }}</span>
            <span class="stat-label">{{ t('permission.enabledRules') }}</span>
          </div>
          <div class="stat-item allow">
            <span class="stat-value">{{ stats.allow }}</span>
            <span class="stat-label">{{ t('permission.actionAllow') }}</span>
          </div>
          <div class="stat-item reject">
            <span class="stat-value">{{ stats.reject }}</span>
            <span class="stat-label">{{ t('permission.actionReject') }}</span>
          </div>
        </div>

        <!-- Filters -->
        <div class="filters-section">
          <select v-model="filterScope" class="filter-select">
            <option value="all">{{ t('permission.allScopes') }}</option>
            <option value="tool">{{ t('permission.scopeTool') }}</option>
            <option value="path">{{ t('permission.scopePath') }}</option>
            <option value="pattern">{{ t('permission.scopePattern') }}</option>
          </select>
          <select v-model="filterAction" class="filter-select">
            <option value="all">{{ t('permission.allActions') }}</option>
            <option value="allow">{{ t('permission.actionAllow') }}</option>
            <option value="reject">{{ t('permission.actionReject') }}</option>
          </select>
          <select v-model="filterEnabled" class="filter-select">
            <option value="all">{{ t('permission.allStatus') }}</option>
            <option value="enabled">{{ t('permission.enabledRules') }}</option>
            <option value="disabled">{{ t('permission.disabledRules') }}</option>
          </select>
        </div>

        <!-- Actions -->
        <div class="actions-section">
          <button class="add-btn" @click="startAdd" :disabled="showAddForm">
            {{ t('permission.addRule') }}
          </button>
          <button class="reset-btn" @click="handleResetDefaults">
            {{ t('permission.resetDefaults') }}
          </button>
        </div>

        <!-- Add/Edit Form -->
        <div v-if="showAddForm || editingRule" class="rule-form">
          <h4>{{ editingRule ? t('permission.editRule') : t('permission.addRule') }}</h4>

          <div class="form-group">
            <label>{{ t('permission.ruleName') }}</label>
            <input v-model="formName" type="text" placeholder="规则名称" />
          </div>

          <div class="form-group">
            <label>{{ t('permission.ruleDescription') }}</label>
            <input v-model="formDescription" type="text" placeholder="描述（可选）" />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>{{ t('permission.ruleScope') }}</label>
              <select v-model="formScope">
                <option value="tool">{{ t('permission.scopeTool') }}</option>
                <option value="path">{{ t('permission.scopePath') }}</option>
                <option value="pattern">{{ t('permission.scopePattern') }}</option>
              </select>
            </div>

            <div class="form-group">
              <label>{{ t('permission.ruleAction') }}</label>
              <select v-model="formAction">
                <option value="allow">{{ t('permission.actionAllow') }}</option>
                <option value="reject">{{ t('permission.actionReject') }}</option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label>{{ t('permission.rulePattern') }}</label>
            <input v-model="formPattern" type="text" placeholder="正则表达式，如: ^read.*" />
            <small>{{ t('permission.patternHint') }}</small>
          </div>

          <div class="form-group">
            <label>{{ t('permission.toolKind') }}</label>
            <input v-model="formToolKind" type="text" placeholder="read, write, bash 等（可选）" />
            <small>{{ t('permission.toolKindHint') }}</small>
          </div>

          <div class="form-group checkbox">
            <label>
              <input type="checkbox" v-model="formEnabled" />
              {{ t('permission.ruleEnabled') }}
            </label>
          </div>

          <div v-if="formError" class="form-error">{{ formError }}</div>

          <div class="form-actions">
            <button class="save-btn" @click="handleSubmit" :disabled="isSubmitting">
              {{ isSubmitting ? t('common.loading') : t('common.save') }}
            </button>
            <button class="cancel-btn" @click="resetForm">{{ t('common.cancel') }}</button>
          </div>
        </div>

        <!-- Rules List -->
        <div class="rules-list">
          <div
            v-for="rule in filteredRules"
            :key="rule.id"
            class="rule-item"
            :class="{ disabled: !rule.enabled }"
          >
            <div class="rule-header">
              <div class="rule-info">
                <span class="rule-name">{{ rule.name }}</span>
                <span class="rule-badges">
                  <span class="badge action" :class="rule.action">
                    {{ getActionIcon(rule.action) }} {{ getActionLabel(rule.action) }}
                  </span>
                  <span class="badge scope">{{ getScopeLabel(rule.scope) }}</span>
                  <span v-if="rule.toolKind" class="badge tool">{{ rule.toolKind }}</span>
                </span>
              </div>
              <div class="rule-actions">
                <input
                  type="checkbox"
                  :checked="rule.enabled"
                  @change="toggleRule(rule.id)"
                  title="启用/禁用"
                />
                <button class="edit-btn" @click="startEdit(rule)">{{ t('common.edit') }}</button>
                <button class="delete-btn" @click="handleDelete(rule.id)">{{ t('common.delete') }}</button>
              </div>
            </div>
            <div v-if="rule.description" class="rule-description">{{ rule.description }}</div>
            <div class="rule-pattern">
              <code>{{ rule.pattern }}</code>
            </div>
            <div class="rule-meta">
              <small>{{ t('permission.updatedAt') }}: {{ formatDate(rule.updatedAt) }}</small>
            </div>
          </div>

          <div v-if="filteredRules.length === 0" class="no-rules">
            {{ t('permission.noRules') }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rules-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
}

.rules-panel {
  background: var(--bg-main);
  border-radius: 8px;
  width: 90%;
  max-width: 700px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.rules-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border-color);
}

.rules-header h2 {
  margin: 0;
  font-size: 1.25rem;
}

.close-btn {
  border: none;
  background: transparent;
  font-size: 1.25rem;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0.25rem;
}

.close-btn:hover {
  color: var(--text-primary);
}

.rules-content {
  padding: 1.25rem;
  overflow-y: auto;
  flex: 1;
}

.stats-section {
  display: flex;
  gap: 1rem;
  padding: 0.75rem;
  background: var(--bg-sidebar);
  border-radius: 6px;
  margin-bottom: 1rem;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.5rem 1rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 600;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.stat-item.allow .stat-value {
  color: var(--bg-success, #28a745);
}

.stat-item.reject .stat-value {
  color: var(--bg-danger, #dc3545);
}

.filters-section {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.filter-select {
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-main);
  font-size: 0.85rem;
  cursor: pointer;
}

.actions-section {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.add-btn, .reset-btn {
  padding: 0.5rem 1rem;
  border: 1px solid;
  border-radius: 4px;
  font-size: 0.85rem;
  cursor: pointer;
}

.add-btn {
  background: var(--bg-primary);
  color: white;
  border-color: var(--bg-primary);
}

.add-btn:hover:not(:disabled) {
  background: var(--bg-primary-hover);
}

.add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.reset-btn {
  background: transparent;
  color: var(--text-secondary);
  border-color: var(--border-color);
}

.reset-btn:hover {
  background: var(--bg-hover);
}

.rule-form {
  background: var(--bg-sidebar);
  padding: 1rem;
  border-radius: 6px;
  margin-bottom: 1rem;
}

.rule-form h4 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
}

.form-group {
  margin-bottom: 0.75rem;
}

.form-group label {
  display: block;
  font-size: 0.85rem;
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.form-group input[type="text"],
.form-group select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 0.9rem;
  background: var(--bg-main);
  color: var(--text-primary);
}

.form-group small {
  display: block;
  margin-top: 0.25rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.form-row {
  display: flex;
  gap: 1rem;
}

.form-row .form-group {
  flex: 1;
}

.form-group.checkbox label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.form-error {
  color: var(--bg-danger);
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
}

.form-actions {
  display: flex;
  gap: 0.5rem;
}

.save-btn {
  padding: 0.5rem 1rem;
  background: var(--bg-primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.save-btn:hover:not(:disabled) {
  background: var(--bg-primary-hover);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cancel-btn {
  padding: 0.5rem 1rem;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.cancel-btn:hover {
  background: var(--bg-hover);
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.rule-item {
  padding: 1rem;
  background: var(--bg-sidebar);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.rule-item.disabled {
  opacity: 0.6;
  background: var(--bg-muted);
}

.rule-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.rule-info {
  flex: 1;
}

.rule-name {
  font-weight: 600;
  font-size: 1rem;
  display: block;
}

.rule-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.25rem;
}

.badge {
  font-size: 0.75rem;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-weight: 500;
}

.badge.action {
  background: var(--bg-light);
}

.badge.action.allow {
  background: var(--bg-success-light, #e8f5e9);
  color: var(--bg-success, #28a745);
}

.badge.action.reject {
  background: var(--bg-danger-light, #ffebee);
  color: var(--bg-danger, #dc3545);
}

.badge.scope {
  background: var(--bg-primary-light, #e3f2fd);
  color: var(--bg-primary, #0066cc);
}

.badge.tool {
  background: var(--bg-secondary-light, #f5f5f5);
  color: var(--text-secondary, #666);
}

.rule-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.rule-actions input[type="checkbox"] {
  cursor: pointer;
}

.edit-btn, .delete-btn {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  cursor: pointer;
}

.edit-btn {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.edit-btn:hover {
  background: var(--bg-hover);
}

.delete-btn {
  background: transparent;
  border: 1px solid var(--bg-danger);
  color: var(--bg-danger);
}

.delete-btn:hover {
  background: var(--bg-danger);
  color: white;
}

.rule-description {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-top: 0.5rem;
}

.rule-pattern {
  margin-top: 0.5rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-main);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.8rem;
}

.rule-pattern code {
  color: var(--text-primary);
}

.rule-meta {
  margin-top: 0.5rem;
  color: var(--text-muted);
}

.no-rules {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>