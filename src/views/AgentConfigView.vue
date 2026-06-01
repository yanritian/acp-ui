<template>
  <div class="agent-config-view view-container">
    <div class="config-header">
      <h2>🤖 {{ t('agentConfig.title') }}</h2>
      <button class="add-btn" @click="showAddForm = true">
        ➕ {{ t('agentConfig.addAgent') }}
      </button>
    </div>

    <!-- Agent List -->
    <div class="agent-list">
      <div v-if="agents.length === 0" class="empty-state">
        <div class="empty-icon">🤖</div>
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
            <option value="stdio">{{ t('agentConfig.stdioOption') }}</option>
          </select>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useConfigStore } from '@/stores/config';
import { useI18n } from '@/locales';
import {
  addAgent,
  updateAgent,
  removeAgent,
  spawnAgent,
  killAgent,
  listRunningAgents
} from '@/lib/host';
import type { AgentConfig } from '@/lib/types';

const { t } = useI18n();

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
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.config-header h2 {
  margin: 0;
  font-size: 24px;
}

.add-btn {
  padding: 10px 20px;
  background: #4CAF50;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.add-btn:hover {
  background: #45a049;
}

.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
}

.agent-card {
  background: var(--bg-surface, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  padding: 20px;
}

.agent-card.connected {
  border-color: #4CAF50;
  box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.1);
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
  background: #e8f5e9;
  color: #2e7d32;
}

.status-disconnected {
  background: #ffebee;
  color: #c62828;
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
  color: #666;
  min-width: 60px;
}

.info-row .value {
  color: #333;
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
  background: #2196F3;
  color: white;
}

.disconnect-btn {
  background: #FF9800;
  color: white;
}

.edit-btn {
  background: #9E9E9E;
  color: white;
}

.delete-btn {
  background: #f44336;
  color: white;
}

.connect-btn:hover { background: #0b7dda; }
.disconnect-btn:hover { background: #e68900; }
.edit-btn:hover { background: #757575; }
.delete-btn:hover { background: #da190b; }

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #666;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
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
  color: #666;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.cancel-btn {
  padding: 10px 20px;
  background: #9E9E9E;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.save-btn {
  padding: 10px 20px;
  background: #4CAF50;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.save-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}
</style>
