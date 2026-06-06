<template>
  <div class="agent-config-view view-container">
    <div class="config-header">
      <h2>
        <span class="header-icon" v-html="icon.robot"></span>
        {{ t('agentConfig.title') }}
      </h2>
      <div class="header-actions">
        <button class="template-btn" @click="showTemplatePanel = true">
          <span v-html="icon.package"></span> {{ t('agentConfig.fromTemplate') }}
        </button>
        <button class="add-btn" @click="showAddForm = true">
          <span v-html="icon.plus"></span> {{ t('agentConfig.addAgent') }}
        </button>
      </div>
    </div>

    <!-- Agent List -->
    <div class="agent-list">
      <div v-if="agents.length === 0" class="empty-state">
        <span class="empty-icon" v-html="icon.robot"></span>
        <h3>{{ t('agentConfig.noAgents') }}</h3>
        <p>{{ t('agentConfig.noAgentsHint') }}</p>
      </div>

      <div v-else class="agents-grid">
        <div
          v-for="agent in agents"
          :key="agent.name"
          class="agent-card"
          :class="{ connected: agent.running }"
        >
          <div class="agent-header">
            <div class="agent-name">{{ agent.name }}</div>
            <div class="agent-status" :class="agent.running ? 'status-connected' : 'status-disconnected'">
              {{ agent.running ? t('agentConfig.running') : t('agentConfig.stopped') }}
            </div>
          </div>

          <div class="agent-info">
            <div class="info-row">
              <span class="label">{{ t('agentConfig.type') }}:</span>
              <span class="value">{{ (agent.config as any).transport || 'stdio' }}</span>
            </div>
            <div class="info-row" v-if="(agent.config as any).transport === 'websocket'">
              <span class="label">URL:</span>
              <span class="value">{{ (agent.config as any).url }}</span>
            </div>
            <div class="info-row" v-if="(agent.config as any).command">
              <span class="label">{{ t('agentConfig.command') }}:</span>
              <span class="value">{{ (agent.config as any).command }}</span>
            </div>
          </div>

          <div class="agent-actions">
            <button
              v-if="!agent.running"
              class="connect-btn"
              @click="connectAgent(agent)"
            >
              {{ t('agentConfig.start') }}
            </button>
            <button
              v-else
              class="disconnect-btn"
              @click="disconnectAgent(agent)"
            >
              {{ t('agentConfig.stop') }}
            </button>
            <button class="edit-btn" @click="editAgent(agent)">{{ t('agentConfig.edit') }}</button>
            <button class="delete-btn" @click="deleteAgent(agent)">{{ t('agentConfig.delete') }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Form Modal -->
    <div v-if="showAddForm || editingAgent" class="modal-overlay" @click.self="closeForm">
      <div class="modal-content">
        <h3>{{ editingAgent ? t('agentConfig.editAgent') : t('agentConfig.addAgent') }}</h3>

        <!-- Platform restriction warning -->
        <div v-if="isRestrictedPlatform" class="platform-warning">
          <span class="warning-icon">⚠️</span>
          <span class="warning-text">{{ t('agentConfig.platformRestriction') }}</span>
        </div>

        <div class="form-group">
          <label>{{ t('agentConfig.agentName') }}</label>
          <input
            v-model="form.name"
            type="text"
            :placeholder="t('agentConfig.agentNamePlaceholder')"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>{{ t('agentConfig.connectionType') }}</label>
          <select v-model="form.type" class="form-select">
            <option value="websocket">WebSocket</option>
            <option value="stdio" :disabled="isRestrictedPlatform">
              {{ t('agentConfig.stdioOption') }}
              {{ isRestrictedPlatform ? `(${t('agentConfig.desktopOnly')})` : '' }}
            </option>
          </select>
          <small v-if="form.type === 'stdio' && isRestrictedPlatform" class="form-warning">
            {{ t('agentConfig.stdioNotAvailable') }}
          </small>
        </div>

        <div v-if="form.type === 'websocket'" class="form-group">
          <label>WebSocket URL</label>
          <input
            v-model="form.url"
            type="text"
            placeholder="ws://localhost:8080/ws"
            class="form-input"
          />
        </div>

        <div v-if="form.type === 'stdio'" class="form-group">
          <label>{{ t('agentConfig.startCommand') }}</label>
          <input
            v-model="form.command"
            type="text"
            placeholder="acp-server --port 8080"
            class="form-input"
          />
          <small class="form-help">{{ t('agentConfig.commandHelp') }}</small>
        </div>

        <div class="form-actions">
          <button class="cancel-btn" @click="closeForm">{{ t('agentConfig.cancel') }}</button>
          <button
            class="save-btn"
            @click="saveAgent"
            :disabled="!form.name || (!form.url && form.type === 'websocket') || (!form.command && form.type === 'stdio')"
          >
            {{ t('agentConfig.save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Template Selection Modal -->
    <div v-if="showTemplatePanel" class="modal-overlay" @click.self="showTemplatePanel = false">
      <div class="modal-content template-panel">
        <div class="template-panel-header">
          <h3><span v-html="icon.package"></span> {{ t('agentConfig.selectTemplate') }}</h3>
          <button class="close-btn" @click="showTemplatePanel = false">
            <span v-html="icon.x"></span>
          </button>
        </div>

        <!-- Tag filter -->
        <div class="tag-filter">
          <button
            class="tag-btn"
            :class="{ active: selectedTag === '' }"
            @click="selectedTag = ''"
          >
            {{ t('agentConfig.allTemplates') }}
          </button>
          <button
            v-for="tag in allTags"
            :key="tag"
            class="tag-btn"
            :class="{ active: selectedTag === tag }"
            @click="selectedTag = tag"
          >
            {{ tag }}
          </button>
        </div>

        <!-- Template grid -->
        <div class="template-grid">
          <div
            v-for="template in filteredTemplates"
            :key="template.id"
            class="template-card"
            :class="{ disabled: !isTemplateCompatible(template) }"
            @click="selectTemplate(template)"
          >
            <div class="template-icon">{{ template.icon }}</div>
            <div class="template-info">
              <div class="template-name">{{ template.name }}</div>
              <div class="template-desc">{{ template.description }}</div>
              <div class="template-tags">
                <span v-for="tag in template.tags" :key="tag" class="tag">{{ tag }}</span>
              </div>
            </div>
            <div v-if="!isTemplateCompatible(template)" class="template-warning">
              {{ t('agentConfig.desktopOnly') }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useConfigStore } from '@/stores/config';
import { useI18n } from '@/locales';
import { restrictedTransports, isDesktop } from '@/lib/platform';
import {
  addAgent,
  updateAgent,
  removeAgent,
  spawnAgent,
  killAgent,
  listRunningAgents
} from '@/lib/host';
import type { AgentConfig } from '@/lib/types';
import {
  AGENT_TEMPLATES,
  getAllTags,
  isTemplateCompatible as checkTemplateCompatible,
  type AgentTemplate
} from '@/lib/agent-templates';
import { icons } from '@/shared/icons';

const { t } = useI18n();
const isRestrictedPlatform = restrictedTransports();
const isDesktopPlatform = isDesktop();
const icon = icons;

interface AgentWithStatus {
  name: string;
  config: AgentConfig;
  running: boolean;
  agentId?: string;
}

const configStore = useConfigStore();
const showAddForm = ref(false);
const editingAgent = ref<string | null>(null);
const runningAgents = ref<string[]>([]);
const agentStatuses = ref<Record<string, boolean>>({});

const form = ref({
  name: '',
  type: 'websocket' as 'websocket' | 'stdio',
  url: 'ws://localhost:8080/ws',
  command: 'acp-server --port 8080',
  autoConnect: false
});

// Template state
const showTemplatePanel = ref(false);
const selectedTag = ref('');
const allTags = getAllTags();

// Filter templates by tag and platform compatibility
const filteredTemplates = computed(() => {
  let templates = AGENT_TEMPLATES;
  if (selectedTag.value) {
    templates = templates.filter(t => t.tags.includes(selectedTag.value));
  }
  return templates;
});

// Check if template is compatible with current platform
function isTemplateCompatible(template: AgentTemplate): boolean {
  return checkTemplateCompatible(template, isDesktopPlatform);
}

// Select a template and pre-fill the form
function selectTemplate(template: AgentTemplate) {
  if (!isTemplateCompatible(template)) {
    return;
  }

  showTemplatePanel.value = false;
  showAddForm.value = true;

  // Pre-fill form with template data
  form.value.name = template.name;
  form.value.type = template.transport || 'stdio';
  form.value.url = template.url || 'ws://localhost:8080/ws';

  // Build command string from command and args
  if (template.command) {
    const fullCommand = [template.command, ...template.args].join(' ');
    form.value.command = fullCommand;
  } else {
    form.value.command = '';
  }
}

// Get all configured agents with their status
const agents = computed<AgentWithStatus[]>(() => {
  return configStore.agentNames.map((name): AgentWithStatus | null => {
    const config = configStore.getAgent(name);
    if (!config) return null;
    return {
      name,
      config,
      running: agentStatuses.value[name] || false,
      agentId: runningAgents.value.find(id => id.includes(name))
    };
  }).filter((a): a is AgentWithStatus => a !== null);
});

// Load running agents status
async function loadStatus() {
  try {
    const running = await listRunningAgents();
    runningAgents.value = running;

    // Update status map
    const statusMap: Record<string, boolean> = {};
    configStore.agentNames.forEach(name => {
      statusMap[name] = running.some(id => id.includes(name));
    });
    agentStatuses.value = statusMap;
  } catch (e) {
    console.error('Failed to load agent status:', e);
  }
}

// Spawn (connect) agent
async function connectAgent(agent: AgentWithStatus) {
  try {
    await spawnAgent(agent.name);
    await loadStatus();
  } catch (e: any) {
    alert(t('agentConfig.startFailed') + ': ' + e.message);
  }
}

// Kill (disconnect) agent
async function disconnectAgent(agent: AgentWithStatus) {
  if (!agent.agentId) return;
  try {
    await killAgent(agent.agentId);
    await loadStatus();
  } catch (e) {
    console.error('Failed to kill agent:', e);
  }
}

// Save agent (add or edit)
async function saveAgent() {
  const agentData = {
    name: form.value.name,
    type: form.value.type,
    url: form.value.type === 'websocket' ? form.value.url : undefined,
    command: form.value.type === 'stdio' ? form.value.command : undefined,
  };

  try {
    if (editingAgent.value) {
      await updateAgent(
        agentData.name,
        agentData.command || null,
        [],
        {},
        agentData.type === 'websocket' ? { transport: 'websocket', url: agentData.url } : {}
      );
    } else {
      await addAgent(
        agentData.name,
        agentData.command || null,
        [],
        {},
        agentData.type === 'websocket' ? { transport: 'websocket', url: agentData.url } : {}
      );
    }
    await configStore.loadConfig();
    closeForm();
    await loadStatus();
  } catch (e: any) {
    alert(t('agentConfig.saveFailed') + ': ' + e.message);
  }
}

// Delete agent
async function deleteAgent(agent: AgentWithStatus) {
  if (!confirm(t('agentConfig.deleteConfirm', { name: agent.name }))) return;

  try {
    await removeAgent(agent.name);
    await configStore.loadConfig();
    await loadStatus();
  } catch (e: any) {
    alert(t('agentConfig.deleteFailed') + ': ' + e.message);
  }
}

// Edit agent
function editAgent(agent: AgentWithStatus) {
  editingAgent.value = agent.name;
  const config = agent.config;
  form.value = {
    name: agent.name,
    type: (config as any).transport || 'websocket',
    url: (config as any).url || 'ws://localhost:8080/ws',
    command: (config as any).command || 'acp-server --port 8080',
    autoConnect: false
  };
}

// Close form
function closeForm() {
  showAddForm.value = false;
  editingAgent.value = null;
  form.value = {
    name: '',
    type: 'websocket',
    url: 'ws://localhost:8080/ws',
    command: 'acp-server --port 8080',
    autoConnect: false
  };
}

// Load on mount
onMounted(async () => {
  await loadStatus();
  // Refresh status every 2 seconds
  setInterval(loadStatus, 2000);
});
</script>

<style scoped>
.agent-config-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
  animation: fadeInUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.config-header h2 {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 0;
  font-size: 24px;
}

.header-icon {
  color: var(--primary);
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  background: var(--success);
  color: var(--text-inverse);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.add-btn:hover {
  background: var(--success-hover);
}

.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
}

.agent-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 20px;
}

.agent-card.connected {
  border-color: var(--success);
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.1);
}

.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.agent-name {
  font-size: 18px;
  font-weight: 600;
}

.agent-status {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-connected {
  background: var(--success-light);
  color: var(--success-dark, #059669);
}

.status-disconnected {
  background: var(--error-light);
  color: var(--error-dark, #dc2626);
}

.agent-info {
  margin-bottom: 16px;
}

.info-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 14px;
}

.info-row .label {
  color: var(--text-secondary);
  min-width: 60px;
}

.info-row .value {
  color: var(--text-primary);
  word-break: break-all;
}

.agent-actions {
  display: flex;
  gap: 8px;
}

.connect-btn, .disconnect-btn, .edit-btn, .delete-btn {
  flex: 1;
  padding: 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.connect-btn {
  background: var(--primary);
  color: var(--text-inverse);
}

.disconnect-btn {
  background: var(--warning);
  color: var(--text-inverse);
}

.edit-btn {
  background: var(--text-subtle);
  color: var(--text-inverse);
}

.delete-btn {
  background: var(--error);
  color: var(--text-inverse);
}

.connect-btn:hover { background: var(--primary-hover); }
.disconnect-btn:hover { background: var(--warning-hover, #d97706); }
.edit-btn:hover { background: var(--text-muted); }
.delete-btn:hover { background: var(--error-hover, #dc2626); }

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.empty-icon {
  display: flex;
  justify-content: center;
  margin-bottom: 16px;
  color: var(--text-muted);
}

/* Modal styles */
.modal-overlay {
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

.modal-content {
  background: var(--bg-surface, #fff);
  border-radius: 12px;
  padding: 24px;
  min-width: 400px;
  max-width: 500px;
}

.modal-content h3 {
  margin: 0 0 20px 0;
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
  padding: 10px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  font-size: 14px;
}

.form-help {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.form-warning {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: var(--error);
  font-weight: 500;
}

.platform-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  margin-bottom: 20px;
  background: linear-gradient(90deg, var(--warning-light) 0%, var(--warning-light-hover, #fde68a) 100%);
  border-radius: 8px;
  border: 1px solid var(--warning);
}

.warning-icon {
  font-size: 18px;
}

.warning-text {
  font-size: 14px;
  color: var(--warning-dark, #92400e);
  font-weight: 500;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.cancel-btn {
  padding: 10px 20px;
  background: var(--text-subtle);
  color: var(--text-inverse);
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.save-btn {
  padding: 10px 20px;
  background: var(--success);
  color: var(--text-inverse);
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.save-btn:disabled {
  background: var(--bg-muted);
  cursor: not-allowed;
}

/* Header actions */
.header-actions {
  display: flex;
  gap: 12px;
}

.template-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  background: var(--accent, var(--primary));
  color: var(--text-inverse);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.template-btn:hover {
  background: var(--accent-hover, var(--primary-hover));
}

/* Template Panel */
.template-panel {
  min-width: 600px;
  max-width: 800px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.template-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.template-panel-header h3 {
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px 8px;
}

.close-btn:hover {
  color: var(--text-primary);
}

/* Tag filter */
.tag-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.tag-btn {
  padding: 6px 14px;
  background: var(--bg-subtle);
  border: 1px solid var(--border-default);
  border-radius: 16px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.tag-btn:hover {
  background: var(--bg-muted);
}

.tag-btn.active {
  background: var(--primary);
  color: var(--text-inverse);
  border-color: var(--primary);
}

/* Template grid */
.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  overflow-y: auto;
  padding-right: 8px;
}

.template-card {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.template-card:hover {
  border-color: var(--primary);
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.15);
}

.template-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.template-card.disabled:hover {
  border-color: var(--border-color);
  box-shadow: none;
}

.template-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.template-info {
  flex: 1;
  min-width: 0;
}

.template-name {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.template-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.template-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.template-tags .tag {
  padding: 2px 8px;
  background: rgba(99, 102, 241, 0.1);
  color: var(--primary);
  border-radius: 10px;
  font-size: 11px;
}

.template-warning {
  position: absolute;
  bottom: 8px;
  right: 8px;
  font-size: 11px;
  color: var(--error);
  background: var(--error-light);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Responsive breakpoints */
@media (max-width: 768px) {
  .agent-config-view {
    padding: 16px;
  }

  .config-header {
    flex-direction: column;
    gap: 12px;
  }

  .config-header h2 {
    font-size: 20px;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions button {
    flex: 1;
    padding: 8px 16px;
    font-size: 13px;
  }

  .agents-grid {
    grid-template-columns: 1fr;
  }

  .modal-content {
    min-width: auto;
    width: calc(100vw - 32px);
    margin: 16px;
  }

  .template-panel {
    min-width: auto;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .agents-grid {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }

  .template-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  }
}
</style>
