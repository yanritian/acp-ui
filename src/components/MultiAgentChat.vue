<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTeamRuntimeStore } from '../stores/team-runtime'
import { useConfigStore } from '../stores/config'
import type { TeamRoutingMode, TeamAgentSelection } from '../lib/team-service/types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const teamRuntime = useTeamRuntimeStore()
const configStore = useConfigStore()

const inputText = ref('')
const selectedAgents = ref<Set<string>>(new Set())
const routingMode = ref<TeamRoutingMode>('single')

const agentNames = computed(() => Object.keys(configStore.config.agents))
const hasAgents = computed(() => agentNames.value.length > 0)

const taskResults = computed(() => {
  const results: Array<{ taskId: string; agentName: string; content: string; thought?: string; status: string; error?: string }> = []
  for (const [, outputs] of teamRuntime.outputs) {
    for (const out of outputs) {
      results.push({
        taskId: out.taskId,
        agentName: out.agentName,
        content: out.content,
        thought: out.thought,
        status: out.status,
        error: out.error,
      })
    }
  }
  return results
})

function toggleAgent(agentName: string) {
  if (selectedAgents.value.has(agentName)) {
    selectedAgents.value.delete(agentName)
  } else {
    selectedAgents.value.add(agentName)
  }
}

function selectAll() {
  for (const name of agentNames.value) {
    selectedAgents.value.add(name)
  }
}

function clearAll() {
  selectedAgents.value.clear()
}

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || !hasAgents.value) return

  const cwd = configStore.getDefaultCwd(agentNames.value[0])
  const agents: TeamAgentSelection[] = Array.from(selectedAgents.value).map(name => ({
    agentName: name,
    cwd,
  }))

  if (agents.length === 0) {
    // Default to first agent if none selected
    agents.push({ agentName: agentNames.value[0], cwd })
  }

  inputText.value = ''

  await teamRuntime.runTeamTask({
    title: text.substring(0, 50),
    prompt: text,
    source: 'multi-agent',
    routing: routingMode.value,
    agents,
  })
}
</script>

<template>
  <div class="multi-agent-chat">
    <!-- Agent selection -->
    <div class="agent-selector">
      <div class="selector-header">
        <span>{{ t('multiAgent.selectAgent') }}</span>
        <div class="selector-actions">
          <button class="action-btn" @click="selectAll">{{ t('multiAgent.selectAll') }}</button>
          <button class="action-btn" @click="clearAll">{{ t('multiAgent.clearAll') }}</button>
        </div>
      </div>
      <div class="agent-chips">
        <button
          v-for="name in agentNames"
          :key="name"
          class="agent-chip"
          :class="{ selected: selectedAgents.has(name) }"
          @click="toggleAgent(name)"
        >
          {{ name }}
        </button>
        <p v-if="!hasAgents" class="no-agents">
          {{ t('multiAgent.addAgentHint') }}
        </p>
      </div>
    </div>

    <!-- Routing mode -->
    <div class="routing-mode">
      <label>
        <input type="radio" v-model="routingMode" value="single" /> {{ t('multiAgent.singleAgent') }}
      </label>
      <label>
        <input type="radio" v-model="routingMode" value="broadcast" /> {{ t('multiAgent.broadcast') }}
      </label>
    </div>

    <!-- Task results -->
    <div class="results-area">
      <div v-if="taskResults.length === 0 && !teamRuntime.isRunning" class="empty-state">
        <p>{{ t('multiAgent.startConversation') }}</p>
        <p v-if="!hasAgents" class="warning">{{ t('multiAgent.addAgentWarning') }}</p>
      </div>

      <div v-if="teamRuntime.isRunning" class="loading-indicator">
        <span class="spinner"></span> {{ t('multiAgent.processing') }}
      </div>

      <div v-for="result in taskResults" :key="result.taskId" class="result-card" :class="result.status">
        <div class="result-header">
          <span class="result-agent">{{ result.agentName }}</span>
          <span class="result-status">{{ result.status === 'completed' ? '✓' : result.status === 'failed' ? '✕' : '...' }}</span>
        </div>
        <div v-if="result.thought" class="result-thought">
          <em>{{ result.thought }}</em>
        </div>
        <div class="result-content">{{ result.content }}</div>
        <div v-if="result.error" class="result-error">{{ result.error }}</div>
      </div>
    </div>

    <!-- Error banner -->
    <div v-if="teamRuntime.error" class="error-banner">
      <span>{{ teamRuntime.error }}</span>
      <button @click="teamRuntime.clearError">✕</button>
    </div>

    <!-- Input -->
    <div class="input-area">
      <textarea
        v-model="inputText"
        class="chat-input"
        :placeholder="t('multiAgent.inputPlaceholder')"
        :disabled="!hasAgents || teamRuntime.isRunning"
        @keydown.enter.exact.prevent="sendMessage"
        rows="2"
      ></textarea>
      <button
        class="send-btn"
        :disabled="!inputText.trim() || !hasAgents || teamRuntime.isRunning"
        @click="sendMessage"
      >
        {{ t('multiAgent.send') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.multi-agent-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 8px;
  padding: 12px;
}

.agent-selector {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px 12px;
}

.selector-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
}

.selector-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 11px;
}

.agent-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-chip {
  padding: 4px 12px;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.agent-chip.selected {
  background: var(--bg-primary);
  color: white;
  border-color: var(--bg-primary);
}

.no-agents {
  font-size: 12px;
  color: var(--text-muted);
}

.routing-mode {
  display: flex;
  gap: 16px;
  font-size: 13px;
  padding: 4px 0;
}

.routing-mode label {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.results-area {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
}

.empty-state .warning {
  color: #ef4444;
}

.loading-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  color: var(--text-muted);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.result-card {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 10px 14px;
}

.result-card.completed {
  border-left: 3px solid #10b981;
}

.result-card.failed {
  border-left: 3px solid #ef4444;
}

.result-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
  font-size: 13px;
}

.result-agent {
  font-weight: 600;
  color: var(--primary);
}

.result-status {
  font-size: 14px;
}

.result-thought {
  font-size: 12px;
  opacity: 0.6;
  margin-bottom: 4px;
  padding-left: 8px;
  border-left: 2px solid currentColor;
}

.result-content {
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 14px;
}

.result-error {
  color: #ef4444;
  font-size: 12px;
  margin-top: 4px;
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #fee;
  color: #c00;
  font-size: 13px;
}

.error-banner button {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}

.input-area {
  display: flex;
  gap: 8px;
  padding: 8px 0;
}

.chat-input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-main);
  color: var(--text-primary);
  font-size: 14px;
  resize: none;
  font-family: inherit;
}

.chat-input:focus {
  border-color: var(--primary);
  outline: none;
}

.chat-input:disabled {
  opacity: 0.5;
}

.send-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background: var(--bg-primary);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
