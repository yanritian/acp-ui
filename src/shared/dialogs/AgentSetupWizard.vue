<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '@/locales'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'complete'): void
}>()

const { t } = useI18n()

// Wizard state
const currentStep = ref(1)
const isTesting = ref(false)
const testResult = ref<'success' | 'failed' | null>(null)

// Agent configuration
const agentName = ref('')
const agentDescription = ref('')
const selectedTemplate = ref<string | null>(null)

// Agent templates (could be loaded from config)
const agentTemplates = computed(() => [
  {
    id: 'claude-code',
    name: 'Claude Code Agent',
    description: 'Official Claude Code CLI Agent - Best for coding tasks',
    icon: '🤖',
    transport: 'stdio',
    recommended: true
  },
  {
    id: 'mcp-server',
    name: 'MCP Server Agent',
    description: 'Model Context Protocol Server - For tool integrations',
    icon: '🔧',
    transport: 'websocket'
  },
  {
    id: 'custom-stdio',
    name: 'Custom CLI Agent',
    description: 'Your own command-line Agent',
    icon: '⚙️',
    transport: 'stdio'
  },
  {
    id: 'custom-ws',
    name: 'WebSocket Agent',
    description: 'Connect to remote Agent via WebSocket',
    icon: '🌐',
    transport: 'websocket'
  }
])

// Parameters based on template
const commandParam = ref('')
const urlParam = ref('')
const argsParam = ref('')

function selectTemplate(templateId: string) {
  selectedTemplate.value = templateId
  const template = agentTemplates.value.find(t => t.id === templateId)
  if (template) {
    agentName.value = template.name
  }
}

function goBack() {
  if (currentStep.value > 1) {
    currentStep.value--
  }
}

function goNext() {
  if (currentStep.value < 3) {
    currentStep.value++
  }
}

async function testConnection() {
  isTesting.value = true
  testResult.value = null

  // Simulate connection test (in real implementation, this would actually test)
  await new Promise(resolve => setTimeout(resolve, 2000))

  // For demo purposes, randomly succeed or fail
  // In real implementation, this would actually attempt to connect
  testResult.value = Math.random() > 0.3 ? 'success' : 'failed'
  isTesting.value = false
}

async function finishWizard() {
  // In real implementation, save the agent configuration
  emit('complete')
}

function closeModal() {
  emit('close')
}
</script>

<template>
  <div class="wizard-overlay" @click.self="closeModal">
    <div class="wizard-modal">
      <!-- Header -->
      <div class="wizard-header">
        <h2>{{ t('onboarding.agentWizardTitle') }}</h2>
        <button class="close-btn" @click="closeModal">✕</button>
      </div>

      <!-- Progress Steps -->
      <div class="wizard-progress">
        <div
          v-for="step in 3"
          :key="step"
          :class="['progress-step', { active: currentStep >= step, completed: currentStep > step }]"
        >
          <div class="step-indicator">
            <span v-if="currentStep > step">✓</span>
            <span v-else>{{ step }}</span>
          </div>
          <div class="step-label">
            {{ step === 1 ? t('onboarding.wizardStep1Title') : step === 2 ? t('onboarding.wizardStep2Title') : t('onboarding.wizardStep3Title') }}
          </div>
        </div>
      </div>

      <!-- Step Content -->
      <div class="wizard-content">
        <!-- Step 1: Select Agent Type -->
        <div v-if="currentStep === 1" class="step-content">
          <h3>{{ t('onboarding.wizardSelectType') }}</h3>
          <div class="template-grid">
            <button
              v-for="template in agentTemplates"
              :key="template.id"
              :class="['template-card', { selected: selectedTemplate === template.id, recommended: template.recommended }]"
              @click="selectTemplate(template.id)"
            >
              <div class="template-icon">{{ template.icon }}</div>
              <div class="template-info">
                <div class="template-name">{{ template.name }}</div>
                <div class="template-desc">{{ template.description }}</div>
                <span v-if="template.recommended" class="recommended-badge">{{ t('agentProgress.recommended') }}</span>
              </div>
            </button>
          </div>
        </div>

        <!-- Step 2: Configure Parameters -->
        <div v-if="currentStep === 2" class="step-content">
          <h3>{{ t('onboarding.wizardConfigureParams') }}</h3>

          <div class="form-group">
            <label>{{ t('onboarding.wizardTemplateName') }}</label>
            <input
              v-model="agentName"
              type="text"
              :placeholder="t('onboarding.wizardTemplateNamePlaceholder')"
            />
          </div>

          <div class="form-group">
            <label>{{ t('onboarding.wizardTemplateDesc') }}</label>
            <input
              v-model="agentDescription"
              type="text"
              placeholder="Optional description..."
            />
          </div>

          <!-- Dynamic parameters based on template -->
          <template v-if="selectedTemplate?.includes('stdio') || selectedTemplate === 'claude-code'">
            <div class="form-group">
              <label>{{ t('settings.command') }}</label>
              <input
                v-model="commandParam"
                type="text"
                placeholder="e.g. claude-code"
              />
              <p class="form-hint">{{ t('settings.argsHint') }}</p>
            </div>

            <div class="form-group">
              <label>{{ t('settings.arguments') }}</label>
              <input
                v-model="argsParam"
                type="text"
                placeholder="e.g. --mode agent"
              />
            </div>
          </template>

          <template v-else-if="selectedTemplate?.includes('ws') || selectedTemplate === 'mcp-server'">
            <div class="form-group">
              <label>{{ t('settings.url') }}</label>
              <input
                v-model="urlParam"
                type="text"
                placeholder="ws://localhost:8080"
              />
              <p class="form-hint">{{ t('settings.wsUrlHint') }}</p>
            </div>
          </template>
        </div>

        <!-- Step 3: Test Connection -->
        <div v-if="currentStep === 3" class="step-content">
          <h3>{{ t('onboarding.wizardTestConnection') }}</h3>

          <div class="test-section">
            <div class="agent-summary">
              <p><strong>{{ t('settings.name') }}:</strong> {{ agentName }}</p>
              <p v-if="agentDescription"><strong>{{ t('workflow.description') }}:</strong> {{ agentDescription }}</p>
              <p v-if="commandParam"><strong>{{ t('settings.command') }}:</strong> {{ commandParam }}</p>
              <p v-if="urlParam"><strong>{{ t('settings.url') }}:</strong> {{ urlParam }}</p>
            </div>

            <button
              class="test-btn"
              :disabled="isTesting"
              @click="testConnection"
            >
              <span v-if="isTesting" class="spinner"></span>
              {{ isTesting ? t('onboarding.wizardTesting') : (testResult ? t('onboarding.wizardRetest') : t('gateway.testConnection')) }}
            </button>

            <div v-if="testResult" :class="['test-result', testResult]">
              <span v-if="testResult === 'success'">✓ {{ t('onboarding.wizardTestSuccess') }}</span>
              <span v-else>✗ {{ t('onboarding.wizardTestFailed') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="wizard-footer">
        <button
          v-if="currentStep > 1"
          class="btn-secondary"
          @click="goBack"
        >
          {{ t('onboarding.wizardBack') }}
        </button>

        <button
          v-if="currentStep < 3"
          class="btn-primary"
          :disabled="currentStep === 1 && !selectedTemplate"
          @click="goNext"
        >
          {{ t('onboarding.wizardNext') }}
        </button>

        <button
          v-if="currentStep === 3"
          class="btn-primary"
          :disabled="!testResult || testResult === 'failed'"
          @click="finishWizard"
        >
          {{ t('onboarding.wizardFinish') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wizard-overlay {
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

.wizard-modal {
  background: var(--bg-primary);
  border-radius: 16px;
  max-width: 600px;
  width: 90%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.wizard-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.wizard-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.5rem;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
}

.close-btn:hover {
  color: var(--text-primary);
}

/* Progress Steps */
.wizard-progress {
  padding: 1rem 1.5rem;
  display: flex;
  gap: 1rem;
  background: var(--bg-surface);
}

.progress-step {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.step-indicator {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--bg-hover);
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.875rem;
  font-weight: 600;
}

.progress-step.active .step-indicator {
  background: var(--primary);
  color: white;
}

.progress-step.completed .step-indicator {
  background: #10b981;
  color: white;
}

.step-label {
  font-size: 0.875rem;
  color: var(--text-muted);
}

.progress-step.active .step-label {
  color: var(--text-primary);
  font-weight: 500;
}

/* Content */
.wizard-content {
  padding: 1.5rem;
  overflow-y: auto;
}

.step-content h3 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  color: var(--text-primary);
}

/* Template Grid */
.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.template-card {
  padding: 1rem;
  background: var(--bg-surface);
  border: 2px solid var(--border-color);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  gap: 1rem;
  align-items: flex-start;
}

.template-card:hover {
  border-color: var(--primary);
}

.template-card.selected {
  border-color: var(--primary);
  background: rgba(var(--primary-rgb), 0.1);
}

.template-card.recommended {
  border-color: #10b981;
}

.template-icon {
  font-size: 2rem;
  flex-shrink: 0;
}

.template-info {
  flex: 1;
  min-width: 0;
}

.template-name {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.template-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.3;
}

.recommended-badge {
  display: inline-block;
  margin-top: 0.5rem;
  padding: 0.125rem 0.5rem;
  background: #10b981;
  color: white;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
}

/* Form */
.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  color: var(--text-primary);
}

.form-group input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 1rem;
  background: var(--bg-surface);
  color: var(--text-primary);
}

.form-group input:focus {
  outline: none;
  border-color: var(--primary);
}

.form-hint {
  margin-top: 0.25rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Test Section */
.test-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.agent-summary {
  background: var(--bg-surface);
  padding: 1rem;
  border-radius: 8px;
}

.agent-summary p {
  margin: 0.25rem 0;
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.test-btn {
  padding: 0.75rem 1.5rem;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.test-btn:hover:not(:disabled) {
  background: var(--primary-dark, #0056b3);
}

.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid white;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.test-result {
  padding: 1rem;
  border-radius: 8px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.test-result.success {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.test-result.failed {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

/* Footer */
.wizard-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}

.btn-primary {
  padding: 0.75rem 1.5rem;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-dark, #0056b3);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 0.75rem 1.5rem;
  background: var(--bg-surface);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  background: var(--bg-hover);
}

/* Mobile */
@media (max-width: 500px) {
  .template-grid {
    grid-template-columns: 1fr;
  }

  .wizard-progress {
    flex-direction: column;
    gap: 0.5rem;
  }

  .step-label {
    display: none;
  }
}
</style>