<template>
  <div class="permission-rules-view">
    <div class="rules-header">
      <h4>{{ t('permissionRules.title') }}</h4>
      <button class="add-btn" @click="showAddRule = true">+ {{ t('permissionRules.addRule') }}</button>
    </div>

    <div v-if="rules.length === 0" class="empty-state">
      <p>{{ t('permissionRules.noRules') }}</p>
      <p class="hint">{{ t('permissionRules.noRulesHint') }}</p>
    </div>

    <div v-else class="rules-list">
      <div v-for="rule in rules" :key="rule.id" class="rule-card">
        <div class="rule-info">
          <span class="rule-pattern">{{ rule.name }}</span>
          <span class="rule-action" :class="rule.action">{{ rule.action === 'allow' ? t('permissionRules.allow') : t('permissionRules.deny') }}</span>
        </div>
        <div class="rule-details">
          <code>{{ rule.pattern }}</code>
        </div>
        <div class="rule-actions">
          <button class="toggle-btn" @click="toggleRule(rule.id)">
            {{ rule.enabled ? '✓' : '○' }}
          </button>
          <button class="edit-btn" @click="editRule(rule)">{{ t('common.edit') }}</button>
          <button class="delete-btn" @click="deleteRule(rule.id)">{{ t('common.delete') }}</button>
        </div>
      </div>
    </div>

    <!-- Add/Edit Rule Modal -->
    <div v-if="showAddRule || editingRule" class="modal-overlay" @click.self="closeModal">
      <div class="modal-content">
        <h3>{{ editingRule ? t('permissionRules.editRule') : t('permissionRules.addRule') }}</h3>
        <div class="form-group">
          <label>{{ t('permission.ruleName') }}</label>
          <input v-model="form.name" type="text" placeholder="规则名称" class="form-input" />
        </div>
        <div class="form-group">
          <label>{{ t('permissionRules.pattern') }}</label>
          <input v-model="form.pattern" type="text" placeholder="正则表达式，如 ^read.*" class="form-input" />
          <small class="form-help">{{ t('permission.patternHint') }}</small>
        </div>
        <div class="form-group">
          <label>{{ t('permission.ruleScope') }}</label>
          <select v-model="form.scope" class="form-select">
            <option value="tool">{{ t('permission.scopeTool') }}</option>
            <option value="path">{{ t('permission.scopePath') }}</option>
            <option value="pattern">{{ t('permission.scopePattern') }}</option>
          </select>
        </div>
        <div class="form-group">
          <label>{{ t('permissionRules.action') }}</label>
          <select v-model="form.action" class="form-select">
            <option value="allow">{{ t('permissionRules.allow') }}</option>
            <option value="reject">{{ t('permissionRules.deny') }}</option>
          </select>
        </div>
        <div class="form-group">
          <label>{{ t('permission.toolKind') }}</label>
          <input v-model="form.toolKind" type="text" placeholder="read, write, bash 等（可选）" class="form-input" />
          <small class="form-help">{{ t('permission.toolKindHint') }}</small>
        </div>
        <div class="form-actions">
          <button class="cancel-btn" @click="closeModal">{{ t('common.cancel') }}</button>
          <button class="save-btn" @click="saveRule">{{ t('common.save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { usePermissionRulesStore, type PermissionAction, type PermissionScope } from '@/stores/permission-rules';
import { useI18n } from '@/locales';

const { t } = useI18n();
const store = usePermissionRulesStore();

const rules = computed(() => store.rules);
const showAddRule = ref(false);
const editingRule = ref<string | null>(null);

const form = ref({
  name: '',
  pattern: '',
  action: 'allow' as PermissionAction,
  scope: 'tool' as PermissionScope,
  toolKind: '',
});

function editRule(rule: any) {
  editingRule.value = rule.id;
  form.value = {
    name: rule.name,
    pattern: rule.pattern,
    action: rule.action,
    scope: rule.scope,
    toolKind: rule.toolKind || '',
  };
  showAddRule.value = false;
}

function deleteRule(id: string) {
  store.deleteRule(id);
}

function toggleRule(id: string) {
  store.toggleRule(id);
}

function saveRule() {
  if (!form.value.pattern || !form.value.name) return;

  const ruleData = {
    name: form.value.name,
    pattern: form.value.pattern,
    action: form.value.action,
    scope: form.value.scope,
    toolKind: form.value.toolKind || undefined,
    enabled: true,
  };

  if (editingRule.value) {
    store.updateRule(editingRule.value, ruleData);
  } else {
    store.addRule(ruleData);
  }
  closeModal();
}

function closeModal() {
  showAddRule.value = false;
  editingRule.value = null;
  form.value = {
    name: '',
    pattern: '',
    action: 'allow',
    scope: 'tool',
    toolKind: '',
  };
}
</script>

<style scoped>
.permission-rules-view {
  padding: 16px;
}

.rules-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.add-btn {
  padding: 8px 16px;
  background: var(--bg-primary, #3b82f6);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.empty-state {
  text-align: center;
  padding: 32px;
  color: var(--text-muted, #666);
}

.hint {
  font-size: 12px;
  margin-top: 8px;
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rule-card {
  display: flex;
  flex-direction: column;
  padding: 12px 16px;
  background: var(--bg-surface, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
}

.rule-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.rule-pattern {
  font-weight: 500;
}

.rule-action {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.rule-action.allow { background: #dcfce7; color: #166534; }
.rule-action.reject { background: #fee2e2; color: #991b1b; }

.rule-details {
  margin-top: 8px;
}

.rule-details code {
  font-size: 0.85rem;
  color: var(--text-muted, #666);
}

.rule-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  justify-content: flex-end;
}

.toggle-btn, .edit-btn, .delete-btn {
  padding: 4px 8px;
  background: none;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  cursor: pointer;
}

.toggle-btn { font-weight: bold; }
.delete-btn:hover { background: #fee2e2; }

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-surface, #fff);
  padding: 24px;
  border-radius: 12px;
  min-width: 400px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.form-input, .form-select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  background: var(--bg-main, #fff);
  color: var(--text-primary);
}

.form-help {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: #666;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.cancel-btn, .save-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.cancel-btn { background: #9e9e9e; color: white; }
.save-btn { background: #4caf50; color: white; }
</style>