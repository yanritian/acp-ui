<template>
  <div class="unified-settings view-container">
    <div class="settings-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.icon }} {{ t(tab.labelKey) }}
      </button>
    </div>

    <div class="settings-content">
      <!-- General Tab -->
      <div v-if="activeTab === 'general'" class="settings-section">
        <h3>{{ t('settings.general') }}</h3>
        <div class="setting-row">
          <label>{{ t('settings.language') }}</label>
          <LanguageSelector />
        </div>
        <div class="setting-row">
          <label>{{ t('settings.theme') }}</label>
          <select v-model="theme" class="form-select">
            <option value="light">{{ t('settings.themeLight') }}</option>
            <option value="dark">{{ t('settings.themeDark') }}</option>
            <option value="auto">{{ t('settings.themeAuto') }}</option>
          </select>
        </div>
      </div>

      <!-- Agent Tab -->
      <div v-if="activeTab === 'agent'" class="settings-section">
        <AgentConfigView />
      </div>

      <!-- Gateway Tab -->
      <div v-if="activeTab === 'gateway'" class="settings-section">
        <GatewaySettings />
      </div>

      <!-- Bot Tab -->
      <div v-if="activeTab === 'bot'" class="settings-section">
        <BotSettings />
      </div>

      <!-- Permissions Tab -->
      <div v-if="activeTab === 'permissions'" class="settings-section">
        <h3>{{ t('settings.permissions') }}</h3>
        <PermissionRulesView />
      </div>

      <!-- Advanced Tab -->
      <div v-if="activeTab === 'advanced'" class="settings-section">
        <h3>{{ t('settings.advanced') }}</h3>
        <div class="setting-row">
          <label>{{ t('settings.tokenBudget') }}</label>
          <input v-model.number="tokenBudget" type="number" class="form-input" placeholder="100000" />
        </div>
        <div class="setting-row">
          <label>{{ t('settings.logLevel') }}</label>
          <select v-model="logLevel" class="form-select">
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from '@/locales';
import LanguageSelector from './LanguageSelector.vue';
import AgentConfigView from '@/views/AgentConfigView.vue';
import GatewaySettings from './GatewaySettings.vue';
import BotSettings from './BotSettings.vue';
import PermissionRulesView from '@/features/config/PermissionRulesView.vue';

const { t } = useI18n();

const tabs = [
  { id: 'general', icon: '⚙️', labelKey: 'settings.general' },
  { id: 'agent', icon: '🤖', labelKey: 'settings.agent' },
  { id: 'gateway', icon: '🌐', labelKey: 'settings.gateway' },
  { id: 'bot', icon: '📱', labelKey: 'settings.bot' },
  { id: 'permissions', icon: '🔒', labelKey: 'settings.permissions' },
  { id: 'advanced', icon: '🔧', labelKey: 'settings.advanced' },
];

const activeTab = ref(localStorage.getItem('acp-ui:settings-tab') ?? 'general');
const theme = ref(localStorage.getItem('acp-ui:theme') ?? 'auto');
const tokenBudget = ref(Number(localStorage.getItem('acp-ui:tokenBudget')) || 100000);
const logLevel = ref(localStorage.getItem('acp-ui:logLevel') ?? 'info');

// Persist settings to localStorage
watch(activeTab, (v) => localStorage.setItem('acp-ui:settings-tab', v));
watch(theme, (v) => localStorage.setItem('acp-ui:theme', v));
watch(tokenBudget, (v) => localStorage.setItem('acp-ui:tokenBudget', String(v)));
watch(logLevel, (v) => localStorage.setItem('acp-ui:logLevel', v));
</script>

<style scoped>
.unified-settings {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.settings-tabs {
  display: flex;
  gap: 8px;
  padding: 16px;
  background: var(--bg-subtle);
  border-bottom: 1px solid var(--border-color);
}

.tab-btn {
  padding: 8px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: var(--bg-hover);
}

.tab-btn.active {
  background: var(--primary);
  color: var(--text-inverse);
  border-color: var(--primary);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  max-width: 800px;
}

.settings-section h3 {
  margin-bottom: 16px;
  color: var(--text-primary);
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-color);
}

.setting-row label {
  min-width: 150px;
  font-weight: 500;
}

.form-select, .form-input {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  min-width: 200px;
}
</style>