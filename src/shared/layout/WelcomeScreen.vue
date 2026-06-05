<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '@/locales'
import AgentSetupWizard from '@/shared/dialogs/AgentSetupWizard.vue'

defineProps<{
  hasAgents: boolean
}>()

const emit = defineEmits<{
  (e: 'open-settings'): void
  (e: 'start-demo'): void
  (e: 'import-config'): void
  (e: 'wizard-complete'): void
}>()

const { t } = useI18n()

// First time user detection
const isFirstTimeUser = ref(false)
const showWizard = ref(false)

onMounted(() => {
  // Check if this is the first time the user opens the app
  const hasVisitedBefore = localStorage.getItem('acp-ui:visited')
  if (!hasVisitedBefore) {
    isFirstTimeUser.value = true
    localStorage.setItem('acp-ui:visited', 'true')
  }
})

function handleQuickStart() {
  showWizard.value = true
}

function handleImportConfig() {
  emit('import-config')
}

function handleDemoExperience() {
  emit('start-demo')
}

function handleWizardComplete() {
  showWizard.value = false
  emit('wizard-complete')
}
</script>

<template>
  <div class="welcome-screen">
    <!-- First Time User Experience -->
    <div v-if="isFirstTimeUser && !hasAgents" class="first-time-section">
      <div class="welcome-header">
        <h2>{{ t('onboarding.firstTimeWelcome') }}</h2>
        <p class="welcome-subtitle">{{ t('onboarding.firstTimeSubtitle') }}</p>
      </div>

      <div class="onboarding-options">
        <button class="onboarding-card quick-start" @click="handleQuickStart">
          <div class="card-icon">🚀</div>
          <div class="card-content">
            <h3>{{ t('onboarding.quickStart') }}</h3>
            <p>{{ t('onboarding.quickStartDesc') }}</p>
          </div>
        </button>

        <button class="onboarding-card import-config" @click="handleImportConfig">
          <div class="card-icon">📁</div>
          <div class="card-content">
            <h3>{{ t('onboarding.importConfig') }}</h3>
            <p>{{ t('onboarding.importConfigDesc') }}</p>
          </div>
        </button>

        <button class="onboarding-card demo-mode" @click="handleDemoExperience">
          <div class="card-icon">🎯</div>
          <div class="card-content">
            <h3>{{ t('onboarding.demoExperience') }}</h3>
            <p>{{ t('onboarding.demoExperienceDesc') }}</p>
          </div>
        </button>
      </div>
    </div>

    <!-- Regular Welcome Screen -->
    <div v-else class="regular-welcome">
      <div class="welcome-header">
        <h2>{{ t('common.welcomeTitle') }}</h2>
        <p class="welcome-subtitle">{{ t('common.welcomeSubtitle') }}</p>
      </div>

      <!-- Quick Start Guide -->
      <div class="quick-start-guide">
        <h3>{{ t('common.quickStart') }}</h3>
        <div class="steps">
          <div class="step">
            <div class="step-number">1</div>
            <div class="step-content">
              <h4>{{ t('common.step1Title') }}</h4>
              <p>{{ t('common.step1Desc') }}</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">2</div>
            <div class="step-content">
              <h4>{{ t('common.step2Title') }}</h4>
              <p>{{ t('common.step2Desc') }}</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">3</div>
            <div class="step-content">
              <h4>{{ t('common.step3Title') }}</h4>
              <p>{{ t('common.step3Desc') }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Feature Highlights -->
      <div class="feature-highlights">
        <h3>{{ t('common.coreFeatures') }}</h3>
        <div class="features-grid">
          <div class="feature-card">
            <div class="feature-icon">💬</div>
            <h4>{{ t('common.featureChat') }}</h4>
            <p>{{ t('common.featureChatDesc') }}</p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">🤖</div>
            <h4>{{ t('common.featureMultiAgent') }}</h4>
            <p>{{ t('common.featureMultiAgentDesc') }}</p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">🕸️</div>
            <h4>{{ t('common.featureNetwork') }}</h4>
            <p>{{ t('common.featureNetworkDesc') }}</p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">🤖</div>
            <h4>{{ t('common.featureBot') }}</h4>
            <p>{{ t('common.featureBotDesc') }}</p>
          </div>
        </div>
      </div>

      <div v-if="!hasAgents" class="hint-section">
        <p class="hint">
          <strong>{{ t('common.tip') }}:</strong> {{ t('common.welcomeHint') }}
        </p>
        <button class="config-agents-btn" @click="emit('open-settings')">
          {{ t('common.configureAgents') }}
        </button>
      </div>
    </div>

    <!-- Agent Setup Wizard Dialog -->
    <AgentSetupWizard
      v-if="showWizard"
      @close="showWizard = false"
      @complete="handleWizardComplete"
    />
  </div>
</template>

<style scoped>
.welcome-screen {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 2rem;
  color: var(--text-secondary);
  overflow-y: auto;
  max-width: 900px;
  margin: 0 auto;
}

/* First Time User Styles */
.first-time-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.onboarding-options {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1.5rem;
  width: 100%;
  max-width: 800px;
  margin-top: 2rem;
}

.onboarding-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.5rem;
  background: var(--bg-surface);
  border: 2px solid var(--border-color);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: center;
}

.onboarding-card:hover {
  border-color: var(--primary);
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.onboarding-card.quick-start:hover {
  border-color: #10b981;
  background: rgba(16, 185, 129, 0.05);
}

.onboarding-card.import-config:hover {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

.onboarding-card.demo-mode:hover {
  border-color: #f59e0b;
  background: rgba(245, 158, 11, 0.05);
}

.card-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.card-content h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.card-content p {
  margin: 0;
  font-size: 0.9rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

/* Regular Welcome Styles */
.regular-welcome {
  width: 100%;
}

.welcome-header {
  text-align: center;
  margin-bottom: 2rem;
}

.welcome-screen h2 {
  margin-bottom: 0.5rem;
  color: var(--text-primary);
  font-size: 2rem;
}

.welcome-subtitle {
  font-size: 1.1rem;
  color: var(--text-muted);
}

/* Quick Start Guide */
.quick-start-guide {
  width: 100%;
  margin-bottom: 2rem;
  background: var(--bg-surface);
  border-radius: 12px;
  padding: 1.5rem;
  border: 1px solid var(--border-color);
}

.quick-start-guide h3 {
  margin: 0 0 1rem 0;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.step {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1rem;
  background: var(--bg-primary);
  border-radius: 8px;
  border-left: 3px solid var(--primary);
}

.step-number {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 1rem;
}

.step-content {
  flex: 1;
}

.step-content h4 {
  margin: 0 0 0.25rem 0;
  font-size: 1rem;
  color: var(--text-primary);
}

.step-content p {
  margin: 0;
  font-size: 0.9rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* Feature Highlights */
.feature-highlights {
  width: 100%;
  margin-bottom: 2rem;
}

.feature-highlights h3 {
  margin: 0 0 1rem 0;
  font-size: 1.25rem;
  color: var(--text-primary);
  text-align: center;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.feature-card {
  background: var(--bg-surface);
  border-radius: 8px;
  padding: 1.25rem;
  border: 1px solid var(--border-color);
  transition: all 0.2s ease;
}

.feature-card:hover {
  border-color: var(--primary);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.feature-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.feature-card h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
  color: var(--text-primary);
}

.feature-card p {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

/* Hint Section */
.hint-section {
  width: 100%;
  text-align: center;
  padding: 1.5rem;
  background: rgba(var(--primary-rgb), 0.05);
  border-radius: 8px;
  border: 1px solid rgba(var(--primary-rgb), 0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.welcome-screen .hint {
  margin: 0;
  font-size: 0.9rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.config-agents-btn {
  padding: 0.75rem 1.5rem;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.config-agents-btn:hover {
  background: var(--primary-dark, #0056b3);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.config-agents-btn:active {
  transform: translateY(0);
}

/* Mobile Styles */
@media (max-width: 600px) {
  .onboarding-options {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .card-icon {
    font-size: 2.5rem;
  }

  .welcome-screen h2 {
    font-size: 1.5rem;
  }
}
</style>