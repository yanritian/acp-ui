<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { useI18n } from '@/locales'
import { addAgent, spawnAgent } from '@/lib/host'
import { trackBehavior } from '@/lib/self-improvement'
import {
  AGENT_TEMPLATES,
  type AgentTemplate
} from '@/lib/agent-templates'

const emit = defineEmits<{
  (e: 'complete'): void
  (e: 'skip'): void
}>()

const { t } = useI18n()
const router = useRouter()
const configStore = useConfigStore()

// State
const step = ref(0) // 0: welcome, 1: select template, 2: configure, 3: test
const selectedTemplate = ref<AgentTemplate | null>(null)
const agentName = ref('')
const adding = ref(false)
const testing = ref(false)
const testResult = ref<'pending' | 'success' | 'failed'>('pending')

// Recommended templates for beginners
const recommendedTemplates = computed(() =>
  AGENT_TEMPLATES.filter(t =>
    t.tags.includes('推荐') || t.tags.includes('AI编程')
  ).slice(0, 5)
)

// Step content
const stepTitles = [
  t('onboarding.welcome'),
  t('onboarding.selectTemplate'),
  t('onboarding.configure'),
  t('onboarding.testConnection')
]

function handleSelectTemplate(template: AgentTemplate) {
  selectedTemplate.value = template
  agentName.value = template.name
  step.value = 2
  trackBehavior('onboarding-template-selected', { template: template.id })
}

async function handleAddAgent() {
  if (!selectedTemplate.value || !agentName.value.trim()) return

  adding.value = true
  try {
    // Add agent using template
    const command = selectedTemplate.value.command
    const args = selectedTemplate.value.args

    await addAgent(
      agentName.value.trim(),
      command ? `${command} ${args.join(' ')}` : null,
      [],
      {},
      selectedTemplate.value.transport === 'websocket'
        ? { transport: 'websocket', url: selectedTemplate.value.url }
        : {}
    )

    // Reload config
    await configStore.loadConfig()

    // Move to test step
    step.value = 3
    trackBehavior('onboarding-agent-added', { agent: agentName.value })
  } catch (e: any) {
    console.error('Failed to add agent:', e)
    alert(t('onboarding.addFailed') + ': ' + e.message)
  } finally {
    adding.value = false
  }
}

async function handleTestConnection() {
  if (!agentName.value) return

  testing.value = true
  testResult.value = 'pending'

  try {
    // Try to spawn the agent
    await spawnAgent(agentName.value)

    // Wait a moment for connection
    await new Promise(r => setTimeout(r, 2000))

    testResult.value = 'success'
    trackBehavior('onboarding-test-success', { agent: agentName.value })
  } catch (e) {
    console.error('Test failed:', e)
    testResult.value = 'failed'
    trackBehavior('onboarding-test-failed', { agent: agentName.value })
  } finally {
    testing.value = false
  }
}

function handleComplete() {
  emit('complete')
  // Navigate to dashboard
  router.push('/')
}

function handleSkip() {
  emit('skip')
  router.push('/agent-config')
}

function handleBack() {
  if (step.value > 0) {
    step.value--
  }
}
</script>

<template>
  <div class="onboarding-flow">
    <!-- Progress Bar -->
    <div class="progress-bar">
      <div
        v-for="(title, idx) in stepTitles"
        :key="idx"
        :class="['progress-step', { active: step === idx, done: step > idx }]"
      >
        <span class="step-number">{{ idx + 1 }}</span>
        <span class="step-title">{{ title }}</span>
      </div>
    </div>

    <!-- Step 0: Welcome -->
    <div v-if="step === 0" class="step-content welcome-step">
      <div class="welcome-icon">🤖</div>
      <h2>{{ t('onboarding.welcomeTitle') }}</h2>
      <p>{{ t('onboarding.welcomeDesc') }}</p>
      <div class="welcome-actions">
        <button class="primary-btn" @click="step = 1">
          {{ t('onboarding.getStarted') }}
        </button>
        <button class="skip-btn" @click="handleSkip">
          {{ t('onboarding.skip') }}
        </button>
      </div>
    </div>

    <!-- Step 1: Select Template -->
    <div v-if="step === 1" class="step-content template-step">
      <h2>{{ t('onboarding.selectTemplateTitle') }}</h2>
      <p>{{ t('onboarding.selectTemplateDesc') }}</p>

      <div class="template-grid">
        <div
          v-for="template in recommendedTemplates"
          :key="template.id"
          class="template-card"
          @click="handleSelectTemplate(template)"
        >
          <div class="template-icon">{{ template.icon }}</div>
          <div class="template-info">
            <span class="template-name">{{ template.name }}</span>
            <span class="template-desc">{{ template.description }}</span>
          </div>
        </div>
      </div>

      <div class="step-actions">
        <button class="back-btn" @click="handleBack">
          {{ t('onboarding.back') }}
        </button>
        <button class="skip-btn" @click="handleSkip">
          {{ t('onboarding.customConfig') }}
        </button>
      </div>
    </div>

    <!-- Step 2: Configure -->
    <div v-if="step === 2" class="step-content configure-step">
      <h2>{{ t('onboarding.configureTitle') }}</h2>
      <p>{{ t('onboarding.configureDesc') }}</p>

      <div class="config-form">
        <div class="form-group">
          <label>{{ t('onboarding.agentName') }}</label>
          <input
            v-model="agentName"
            type="text"
            class="form-input"
            :placeholder="t('onboarding.agentNamePlaceholder')"
          />
        </div>

        <div v-if="selectedTemplate" class="template-preview">
          <div class="preview-label">{{ t('onboarding.templatePreview') }}</div>
          <div class="preview-content">
            <span class="preview-icon">{{ selectedTemplate.icon }}</span>
            <span class="preview-name">{{ selectedTemplate.name }}</span>
            <span class="preview-command">{{ selectedTemplate.command }} {{ selectedTemplate.args.join(' ') }}</span>
          </div>
        </div>
      </div>

      <div class="step-actions">
        <button class="back-btn" @click="handleBack">
          {{ t('onboarding.back') }}
        </button>
        <button
          class="primary-btn"
          :disabled="!agentName.trim() || adding"
          @click="handleAddAgent"
        >
          {{ adding ? t('onboarding.adding') : t('onboarding.addAgent') }}
        </button>
      </div>
    </div>

    <!-- Step 3: Test -->
    <div v-if="step === 3" class="step-content test-step">
      <h2>{{ t('onboarding.testTitle') }}</h2>
      <p>{{ t('onboarding.testDesc') }}</p>

      <div class="test-area">
        <div class="agent-preview">
          <span class="agent-icon">🤖</span>
          <span class="agent-name">{{ agentName }}</span>
        </div>

        <button
          class="test-btn"
          :disabled="testing"
          @click="handleTestConnection"
        >
          {{ testing ? t('onboarding.testing') : t('onboarding.testConnection') }}
        </button>

        <div v-if="testResult !== 'pending'" class="test-result">
          <div v-if="testResult === 'success'" class="result-success">
            <span class="result-icon">✓</span>
            <span>{{ t('onboarding.testSuccess') }}</span>
          </div>
          <div v-else class="result-failed">
            <span class="result-icon">✗</span>
            <span>{{ t('onboarding.testFailed') }}</span>
          </div>
        </div>
      </div>

      <div class="step-actions">
        <button class="back-btn" @click="handleBack">
          {{ t('onboarding.back') }}
        </button>
        <button class="primary-btn" @click="handleComplete">
          {{ t('onboarding.complete') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboarding-flow {
  max-width: 600px;
  margin: 0 auto;
  padding: 40px 20px;
}

.progress-bar {
  display: flex;
  justify-content: space-between;
  margin-bottom: 40px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  opacity: 0.5;
}

.progress-step.active {
  opacity: 1;
}

.progress-step.done {
  opacity: 0.8;
}

.step-number {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--bg-subtle, #f5f5f5);
  font-size: 14px;
  font-weight: 600;
}

.progress-step.active .step-number {
  background: var(--bg-primary, #0066cc);
  color: white;
}

.progress-step.done .step-number {
  background: #4CAF50;
  color: white;
}

.step-title {
  font-size: 12px;
  color: var(--text-secondary, #666);
}

.step-content {
  text-align: center;
}

.step-content h2 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
  color: var(--text-primary, #333);
}

.step-content p {
  color: var(--text-secondary, #666);
  margin-bottom: 24px;
}

/* Welcome Step */
.welcome-icon {
  font-size: 64px;
  margin-bottom: 24px;
}

.welcome-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

/* Template Step */
.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.template-card {
  display: flex;
  gap: 12px;
  padding: 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.template-card:hover {
  border-color: var(--bg-primary, #0066cc);
  background: rgba(0, 102, 204, 0.05);
}

.template-icon {
  font-size: 32px;
}

.template-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.template-name {
  font-size: 14px;
  font-weight: 600;
}

.template-desc {
  font-size: 12px;
  color: var(--text-muted, #999);
}

/* Configure Step */
.config-form {
  max-width: 400px;
  margin: 0 auto 24px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--text-secondary, #666);
}

.form-input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  font-size: 14px;
}

.form-input:focus {
  border-color: var(--bg-primary, #0066cc);
  outline: none;
}

.template-preview {
  padding: 16px;
  background: var(--bg-subtle, #f5f5f5);
  border-radius: 8px;
}

.preview-label {
  font-size: 12px;
  color: var(--text-muted, #999);
  margin-bottom: 8px;
}

.preview-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-icon {
  font-size: 24px;
}

.preview-name {
  font-weight: 600;
}

.preview-command {
  font-size: 12px;
  color: var(--text-muted, #999);
  font-family: monospace;
}

/* Test Step */
.test-area {
  padding: 24px;
  background: var(--bg-subtle, #f5f5f5);
  border-radius: 12px;
  margin-bottom: 24px;
}

.agent-preview {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.agent-icon {
  font-size: 32px;
}

.agent-name {
  font-size: 18px;
  font-weight: 600;
}

.test-btn {
  padding: 12px 24px;
  background: var(--bg-primary, #0066cc);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.test-btn:disabled {
  opacity: 0.5;
}

.test-result {
  margin-top: 16px;
  padding: 12px;
  border-radius: 8px;
}

.result-success {
  background: rgba(76, 175, 80, 0.1);
  color: #4CAF50;
}

.result-failed {
  background: rgba(244, 67, 54, 0.1);
  color: #f44336;
}

.result-icon {
  font-size: 18px;
  margin-right: 8px;
}

/* Action Buttons */
.step-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.primary-btn {
  padding: 12px 32px;
  background: var(--bg-primary, #0066cc);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.primary-btn:hover:not(:disabled) {
  background: var(--bg-primary-hover, #0052a3);
}

.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.back-btn,
.skip-btn {
  padding: 12px 24px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary, #666);
  font-size: 14px;
  cursor: pointer;
}

.back-btn:hover,
.skip-btn:hover {
  background: var(--bg-hover, #f0f0f0);
}
</style>