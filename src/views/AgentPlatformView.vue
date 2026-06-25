<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAgentPlatformStore } from '@/stores/agent-platform'
import type { AgentSummary } from '@/api/types'

const store = useAgentPlatformStore()

const input = ref('')
const preferredAgent = ref('')
const maxCost = ref<number | undefined>(undefined)

const roleLabels: Record<string, string> = {
  developer: '开发者',
  marketer: '营销人员',
  designer: '设计师',
  finance: '财务人员',
  gamer: '游戏开发者',
  writer: '内容创作者',
  analyst: '数据分析师',
  general: '普通用户'
}

const sceneLabels: Record<string, string> = {
  code_generation: '代码生成',
  code_review: '代码审查',
  bug_fix: 'Bug修复',
  image_generation: '图片生成',
  video_generation: '视频生成',
  copywriting: '文案创作',
  translation: '翻译',
  document_generation: '文档生成',
  excel_analysis: 'Excel分析',
  game_build: '游戏构建',
  general_task: '通用任务'
}

const detectedRoleLabel = computed(() => {
  if (!store.oneShotResult) return ''
  return roleLabels[store.oneShotResult.detected_role] || store.oneShotResult.detected_role
})

const detectedSceneLabel = computed(() => {
  if (!store.oneShotResult) return ''
  return sceneLabels[store.oneShotResult.detected_scene] || store.oneShotResult.detected_scene
})

const selectedAgentInfo = computed(() => {
  if (!store.oneShotResult) return null
  return store.agentById(store.oneShotResult.selected_agent)
})

const canExecute = computed(() => {
  return input.value.trim().length > 0 && !store.isExecuting
})

async function execute() {
  if (!canExecute.value) return

  await store.executeOneShot(input.value, {
    preferred_agent: preferredAgent.value || undefined,
    max_cost: maxCost.value
  })
}

function clearInput() {
  input.value = ''
  preferredAgent.value = ''
  maxCost.value = undefined
  store.oneShotResult = null
}

// Load agents on mount
store.fetchAgents()
</script>

<template>
  <div class="oneshot-container">
    <div class="header">
      <h1>Agent Platform</h1>
      <p class="subtitle">一键智能任务执行</p>
    </div>

    <!-- Main Input Area -->
    <div class="input-section">
      <textarea
        v-model="input"
        placeholder="输入你的需求，例如：帮我写一个 Rust 函数、生成一张营销图片、分析这个 Excel..."
        class="main-input"
        :disabled="store.isExecuting"
        @keydown.ctrl.enter="execute"
      />

      <div class="options-row">
        <select v-model="preferredAgent" class="agent-select">
          <option value="">自动选择 Agent</option>
          <option v-for="agent in store.availableAgents" :key="agent.id" :value="agent.id">
            {{ agent.name }}
          </option>
        </select>

        <input
          v-model.number="maxCost"
          type="number"
          placeholder="最大成本 (¥)"
          class="cost-input"
          min="0"
          step="0.01"
        />
      </div>

      <div class="action-buttons">
        <button
          class="execute-btn"
          :disabled="!canExecute"
          @click="execute"
        >
          {{ store.isExecuting ? '执行中...' : '执行' }}
        </button>
        <button class="clear-btn" @click="clearInput">清空</button>
      </div>
    </div>

    <!-- Execution Result -->
    <div v-if="store.oneShotResult" class="result-section">
      <div class="result-header">
        <span class="detected-role">
          检测角色: <strong>{{ detectedRoleLabel }}</strong>
        </span>
        <span class="detected-scene">
          检测场景: <strong>{{ detectedSceneLabel }}</strong>
        </span>
      </div>

      <div class="agent-selection">
        <div class="selected-agent">
          <span class="agent-icon">🤖</span>
          <span class="agent-name">{{ store.oneShotResult.selected_agent }}</span>
          <span class="agent-reason">{{ store.oneShotResult.selection_reason }}</span>
        </div>

        <div v-if="selectedAgentInfo" class="agent-details">
          <span class="health">健康度: {{ selectedAgentInfo.health_score.toFixed(0) }}%</span>
          <span class="type">类型: {{ selectedAgentInfo.adapter_type }}</span>
        </div>
      </div>

      <div class="result-preview">
        <h3>执行结果预览</h3>
        <div class="preview-content">{{ store.oneShotResult.result_preview }}</div>
      </div>

      <div class="cost-info">
        <span>预估成本: ¥{{ store.oneShotResult.estimated_cost.toFixed(4) }}</span>
        <span>预计耗时: {{ store.oneShotResult.transparency.estimated_duration_ms / 1000 }}s</span>
        <span>隐私级别: {{ store.oneShotResult.transparency.privacy_level }}</span>
      </div>

      <div class="dag-visualization">
        <code>{{ store.oneShotResult.transparency.dag_visualization }}</code>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="store.error" class="error-section">
      <span class="error-icon">⚠️</span>
      <span class="error-message">{{ store.error }}</span>
      <button class="retry-btn" @click="store.clearError">清除</button>
    </div>

    <!-- Available Agents List -->
    <div class="agents-section">
      <h3>可用 Agent ({{ store.agents.length }})</h3>
      <div class="agents-grid">
        <div
          v-for="agent in store.agents"
          :key="agent.id"
          class="agent-card"
          :class="{ selected: preferredAgent === agent.id }"
          @click="preferredAgent = agent.id"
        >
          <div class="agent-header">
            <span class="agent-name">{{ agent.name }}</span>
            <span class="agent-status" :class="agent.status">{{ agent.status }}</span>
          </div>
          <div class="agent-health">
            <div class="health-bar">
              <div class="health-fill" :style="{ width: agent.health_score + '%' }"></div>
            </div>
            <span class="health-value">{{ agent.health_score.toFixed(0) }}%</span>
          </div>
          <div class="agent-caps">
            <span v-for="cap in agent.capabilities.slice(0, 3)" :key="cap" class="cap-tag">
              {{ cap }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.oneshot-container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.header {
  text-align: center;
  margin-bottom: 30px;
}

.header h1 {
  font-size: 2rem;
  margin-bottom: 8px;
}

.subtitle {
  color: #666;
  font-size: 1rem;
}

.input-section {
  background: #f5f5f5;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 20px;
}

.main-input {
  width: 100%;
  min-height: 120px;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 14px;
  resize: vertical;
  font-family: inherit;
}

.options-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.agent-select, .cost-input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.agent-select {
  flex: 2;
}

.cost-input {
  flex: 1;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.execute-btn {
  padding: 12px 24px;
  background: #4a90d9;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 16px;
}

.execute-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.clear-btn {
  padding: 12px 24px;
  background: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
}

.result-section {
  background: white;
  padding: 20px;
  border-radius: 12px;
  border: 1px solid #e0e0e0;
  margin-bottom: 20px;
}

.result-header {
  display: flex;
  gap: 20px;
  margin-bottom: 15px;
  padding-bottom: 15px;
  border-bottom: 1px solid #eee;
}

.detected-role, .detected-scene {
  font-size: 14px;
}

.detected-role strong, .detected-scene strong {
  color: #4a90d9;
}

.agent-selection {
  display: flex;
  align-items: center;
  gap: 15px;
  margin-bottom: 15px;
}

.selected-agent {
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-icon {
  font-size: 24px;
}

.agent-name {
  font-weight: bold;
  font-size: 16px;
}

.agent-reason {
  color: #666;
  font-size: 12px;
}

.agent-details {
  display: flex;
  gap: 10px;
  font-size: 12px;
  color: #888;
}

.result-preview {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 15px;
}

.result-preview h3 {
  margin-bottom: 10px;
  font-size: 14px;
}

.preview-content {
  font-size: 14px;
  line-height: 1.5;
}

.cost-info {
  display: flex;
  gap: 20px;
  font-size: 13px;
  color: #666;
  margin-bottom: 10px;
}

.dag-visualization {
  background: #f0f0f0;
  padding: 10px;
  border-radius: 6px;
  font-size: 12px;
  font-family: monospace;
}

.error-section {
  background: #fff5f5;
  padding: 15px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}

.error-icon {
  font-size: 20px;
}

.error-message {
  color: #c00;
  flex: 1;
}

.retry-btn {
  padding: 6px 12px;
  background: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
}

.agents-section h3 {
  margin-bottom: 15px;
}

.agents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}

.agent-card {
  background: white;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
  cursor: pointer;
  transition: all 0.2s;
}

.agent-card:hover {
  border-color: #4a90d9;
}

.agent-card.selected {
  border-color: #4a90d9;
  background: #f0f7ff;
}

.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.agent-name {
  font-weight: 500;
  font-size: 13px;
}

.agent-status {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  background: #e8f5e9;
  color: #2e7d32;
}

.agent-status.unavailable {
  background: #ffebee;
  color: #c62828;
}

.agent-health {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.health-bar {
  flex: 1;
  height: 4px;
  background: #e0e0e0;
  border-radius: 2px;
}

.health-fill {
  height: 100%;
  background: #4caf50;
  border-radius: 2px;
}

.health-value {
  font-size: 11px;
  color: #666;
}

.agent-caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cap-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: #f5f5f5;
  border-radius: 4px;
  color: #666;
}
</style>