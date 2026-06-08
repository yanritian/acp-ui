<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { useSessionStore } from '@/stores/session'
import { useOrchestrationStore } from '@/stores/orchestration'
import { useI18n } from '@/locales'
import { trackBehavior } from '@/lib/self-improvement'
import AgentStatusCard from '@/features/agents/AgentStatusCard.vue'
import QuickTaskInput from '@/features/tasks/QuickTaskInput.vue'
import RecentTasks from '@/features/tasks/RecentTasks.vue'
import OnboardingFlow from '@/features/onboarding/OnboardingFlow.vue'
import { icons } from '@/shared/icons'

const { t } = useI18n()
const router = useRouter()
const configStore = useConfigStore()
const sessionStore = useSessionStore()
const orchestrationStore = useOrchestrationStore()

// Expose icons to template
const icon = icons

// State
const showOnboarding = ref(false)
const selectedAgent = ref<string | null>(null)
const loadingAgents = ref(false)

// Computed
const hasAgents = computed(() => configStore.agentNames.length > 0)
const agents = computed(() => configStore.agentNames)
const hasActiveSession = computed(() => sessionStore.isConnected)

// Check if we need to show onboarding
const needsOnboarding = computed(() => !hasAgents.value && !loadingAgents.value)

onMounted(async () => {
  loadingAgents.value = true
  await configStore.loadConfig()
  loadingAgents.value = false

  // 初始化 orchestration 数据
  orchestrationStore.refreshAll()

  // Track dashboard view
  trackBehavior('dashboard-viewed', { agentCount: agents.value.length })

  // Auto-select first agent if available
  if (agents.value.length > 0) {
    selectedAgent.value = agents.value[0]
  }

  // Show onboarding if no agents
  if (needsOnboarding.value) {
    showOnboarding.value = true
  }
})

function handleAgentSelect(agentName: string) {
  selectedAgent.value = agentName
  trackBehavior('dashboard-agent-selected', { agent: agentName })
}

function handleTaskSubmit(task: string) {
  if (!selectedAgent.value) {
    showOnboarding.value = true
    return
  }

  // Navigate to chat with task pre-filled
  router.push({
    path: '/chat',
    query: { task, agent: selectedAgent.value }
  })

  trackBehavior('dashboard-task-submitted', {
    agent: selectedAgent.value,
    taskLength: task.length
  })
}

function handleOnboardingComplete() {
  showOnboarding.value = false
  if (configStore.agentNames.length > 0) {
    selectedAgent.value = configStore.agentNames[0]
  }
}

function navigateToAgentConfig() {
  router.push('/agent-config')
  trackBehavior('dashboard-nav-agent-config', {})
}

function navigateToHistory() {
  router.push('/history')
  trackBehavior('dashboard-nav-history', {})
}
</script>

<template>
  <div class="dashboard-view">
    <!-- Header -->
    <header class="dashboard-header">
      <h1>{{ t('dashboard.title') }}</h1>
      <div class="header-actions">
        <button class="action-btn" @click="navigateToHistory">
          {{ t('dashboard.viewHistory') }}
        </button>
        <button class="action-btn primary" @click="navigateToAgentConfig">
          {{ t('dashboard.manageAgents') }}
        </button>
      </div>
    </header>

    <!-- Onboarding Flow (shown when no agents) -->
    <OnboardingFlow
      v-if="showOnboarding"
      @complete="handleOnboardingComplete"
      @skip="showOnboarding = false"
    />

    <!-- Main Dashboard Content -->
    <div v-else class="dashboard-content">
      <!-- Orchestration Summary Section -->
      <section class="dashboard-section orchestration-summary">
        <h2 class="section-title">
          <span class="section-icon">⚡</span>
          {{ t('dashboard.orchestration') }}
        </h2>
        <div class="orchestration-stats">
          <div class="stat-item">
            <span class="stat-value">{{ orchestrationStore.runningTaskCount }}</span>
            <span class="stat-label">{{ t('dashboard.activeTasks') }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ orchestrationStore.pendingApprovals.length }}</span>
            <span class="stat-label">{{ t('dashboard.pendingApprovals') }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ orchestrationStore.agents.length }}</span>
            <span class="stat-label">{{ t('dashboard.orchAgents') }}</span>
          </div>
        </div>
      </section>

      <!-- Agent Status Section -->
      <section class="dashboard-section">
        <h2 class="section-title">
          <span class="section-icon" v-html="icon.robot"></span>
          {{ t('dashboard.agentStatus') }}
        </h2>
        <AgentStatusCard
          :agents="agents"
          :selected-agent="selectedAgent"
          :loading="loadingAgents"
          @select="handleAgentSelect"
          @add="navigateToAgentConfig"
        />
      </section>

      <!-- Quick Task Section -->
      <section v-if="hasAgents" class="dashboard-section">
        <h2 class="section-title">
          <span class="section-icon" v-html="icon.fileText"></span>
          {{ t('dashboard.quickTask') }}
        </h2>
        <QuickTaskInput
          :selected-agent="selectedAgent"
          :agents="agents"
          @submit="handleTaskSubmit"
          @agent-change="handleAgentSelect"
        />
      </section>

      <!-- Recent Tasks Section -->
      <section v-if="hasAgents" class="dashboard-section">
        <h2 class="section-title">
          <span class="section-icon" v-html="icon.barChart"></span>
          {{ t('dashboard.recentTasks') }}
        </h2>
        <RecentTasks @view-all="navigateToHistory" />
      </section>

      <!-- Empty State (when agents exist but none selected) -->
      <div v-if="hasAgents && !selectedAgent" class="empty-prompt">
        <p>{{ t('dashboard.selectAgentPrompt') }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
  min-height: 100vh;
  animation: fadeInUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.dashboard-section {
  animation: fadeInUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  animation-fill-mode: both;
}

.dashboard-section:nth-child(2) { animation-delay: 0.05s; }
.dashboard-section:nth-child(3) { animation-delay: 0.1s; }

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.dashboard-header h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 12px;
}

.action-btn {
  padding: 10px 20px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.primary {
  background: var(--primary);
  color: var(--text-inverse);
  border-color: var(--primary);
}

.action-btn.primary:hover {
  background: var(--primary-hover);
}

.dashboard-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.dashboard-section {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 20px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
}

.section-icon {
  display: flex;
  align-items: center;
  color: var(--primary);
}

.section-icon :deep(svg) {
  width: 24px;
  height: 24px;
}

.section-icon :deep(svg) {
  width: 24px;
  height: 24px;
}

.empty-prompt {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.empty-prompt p {
  font-size: 16px;
}

/* Responsive breakpoints */
@media (max-width: 768px) {
  .dashboard-view {
    padding: 16px;
  }

  .dashboard-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 24px;
  }

  .dashboard-header h1 {
    font-size: 22px;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions .action-btn {
    flex: 1;
    padding: 8px 16px;
    font-size: 13px;
  }

  .dashboard-section {
    padding: 16px;
  }

  .section-title {
    font-size: 16px;
  }

  .section-icon {
    font-size: 20px;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .dashboard-view {
    padding: 20px;
  }

  .dashboard-header h1 {
    font-size: 24px;
  }
}
</style>