<template>
  <div class="permission-rules-view">
    <div class="rules-header">
      <h4>{{ t('permissionRules.title') }}</h4>
      <button class="add-btn" @click="showAddRule = true">➕ {{ t('permissionRules.addRule') }}</button>
    </div>

    <div v-if="rules.length === 0" class="empty-state">
      <p>{{ t('permissionRules.noRules') }}</p>
      <p class="hint">{{ t('permissionRules.noRulesHint') }}</p>
    </div>

    <div v-else class="rules-list">
      <div v-for="rule in rules" :key="rule.id" class="rule-card">
        <div class="rule-info">
          <span class="rule-pattern">{{ rule.pattern }}</span>
          <span class="rule-action" :class="rule.action">{{ rule.action }}</span>
        </div>
        <div class="rule-actions">
          <button class="edit-btn" @click="editRule(rule)">✏️</button>
          <button class="delete-btn" @click="deleteRule(rule.id)">🗑️</button>
        </div>
      </div>
    </div>

    <!-- Add/Edit Rule Modal -->
    <div v-if="showAddRule || editingRule" class="modal-overlay" @click.self="closeModal">
      <div class="modal-content">
        <h3>{{ editingRule ? t('permissionRules.editRule') : t('permissionRules.addRule') }}</h3>
        <div class="form-group">
          <label>{{ t('permissionRules.pattern') }}</label>
          <input v-model="form.pattern" type="text" placeholder="fs/read_text_file:*" class="form-input" />
          <small class="form-help">{{ t('permissionRules.patternHelp') }}</small>
        </div>
        <div class="form-group">
          <label>{{ t('permissionRules.action') }}</label>
          <select v-model="form.action" class="form-select">
            <option value="allow">{{ t('permissionRules.allow') }}</option>
            <option value="deny">{{ t('permissionRules.deny') }}</option>
            <option value="ask">{{ t('permissionRules.ask') }}</option>
          </select>
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
import { usePermissionRulesStore } from '@/stores/permission-rules';
import { useI18n } from '@/locales';

const { t } = useI18n();
const store = usePermissionRulesStore();

const rules = computed(() => store.rules);
const showAddRule = ref(false);
const editingRule = ref<string | null>(null);

const form = ref({
  pattern: '',
  action: 'allow' as 'allow' | 'deny' | 'ask',
});

function editRule(rule: any) {
  editingRule.value = rule.id;
  form.value = { pattern: rule.pattern, action: rule.action };
  showAddRule.value = false;
}

function deleteRule(id: string) {
  store.removeRule(id);
}

function saveRule() {
  if (!form.value.pattern) return;
  if (editingRule.value) {
    store.updateRule(editingRule.value, form.value);
  } else {
    store.addRule(form.value);
  }
  closeModal();
}

function closeModal() {
  showAddRule.value = false;
  editingRule.value = null;
  form.value = { pattern: '', action: 'allow' };
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
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.empty-state {
  text-align: center;
  padding: 32px;
  color: #666;
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
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-surface, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
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
.rule-action.deny { background: #fee2e2; color: #991b1b; }
.rule-action.ask { background: #fef3c7; color: #92400e; }

.rule-actions {
  display: flex;
  gap: 8px;
}

.edit-btn, .delete-btn {
  padding: 4px 8px;
  background: none;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  cursor: pointer;
}

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