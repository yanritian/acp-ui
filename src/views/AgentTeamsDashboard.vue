<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useAgentRealtimeStore } from '@/stores/agent-realtime'
import { useAgentPetStore } from '@/stores/agent-pet'
import { useCollaborationStore } from '@/stores/collaboration'
import AgentRealtimeProgressPanel from '@/features/agent-teams/components/agent-progress/AgentRealtimeProgressPanel.vue'
import AgentPetAvatar from '@/features/agent-teams/components/agent-pet/AgentPetAvatar.vue'
import EmotionDisplay from '@/features/agent-teams/components/agent-pet/EmotionDisplay.vue'
import GrowthSystem from '@/features/agent-teams/components/agent-pet/GrowthSystem.vue'
import InteractionPanel from '@/features/agent-teams/components/agent-pet/InteractionPanel.vue'
import EnhancedHermesDashboard from '@/features/hermes/EnhancedHermesDashboard.vue'
import { useI18n } from '@/locales'

const { t } = useI18n()

// Stores
const realtimeStore = useAgentRealtimeStore()
const petStore = useAgentPetStore()
const collaborationStore = useCollaborationStore()

// State
const activeView = ref<'progress' | 'collaboration' | 'pets' | 'all'>('all')
const selectedAgentId = ref<string | null>(null)
const showSyncPanel = ref(false)
const autoRefresh = ref(true)
let refreshInterval: ReturnType<typeof setInterval> | null = null

// Computed
const allAgents = computed(() => realtimeStore.allAgents)
const allPets = computed(() => petStore.allPets)
const globalStats = computed(() => realtimeStore.globalStats)
const activeAgent = computed(() => realtimeStore.activeAgent)
const activePet = computed(() => petStore.activePet)

// Methods
function selectAgent(agentId: string) {
  selectedAgentId.value = agentId
  realtimeStore.setActiveAgent(agentId)
  petStore.setActivePet(agentId)
}

function handlePermissionResponse(optionId: string) {
  if (activeAgent.value) {
    realtimeStore.respondToPermission(activeAgent.value.agentId, optionId)
  }
}

function handleInteraction(type: string) {
  if (activePet.value) {
    petStore.handleInteraction(activePet.value.agentId, type as any)

    // 同步到其他端（Phase 4功能）
    // syncService.sendInteraction(activePet.value.agentId, type)
  }
}

function handleStatusChange(status: string) {
  if (activePet.value) {
    petStore.autoAdjustEmotion(activePet.value.agentId, status as any)
  }
}

function refreshData() {
  // 刷新数据
}

function toggleAutoRefresh() {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }

  if (autoRefresh.value) {
    refreshInterval = setInterval(refreshData, 5000)
  }
}

// Lifecycle
onMounted(() => {
  // 选择第一个Agent
  if (realtimeStore.allAgents.length > 0) {
    selectAgent(realtimeStore.allAgents[0].agentId)
  }

  // 启动自动刷新
  if (autoRefresh.value) {
    refreshInterval = setInterval(refreshData, 5000)
  }
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<template>
  <div class="agent-teams-dashboard">
    <!-- Header -->
    <header class="dashboard-header">
      <div class="header-left">
        <h1 class="dashboard-title">
          <span class="title-icon">🤖</span>
          {{ t('agentTeams.title') }}
        </h1>
        <p class="dashboard-subtitle">{{ t('agentTeams.subtitle') }}</p>
      </div>

      <div class="header-right">
        <!-- Global stats -->
        <div class="global-stats">
          <div class="stat-badge">
            <span class="stat-icon">🤖</span>
            <span class="stat-value">{{ globalStats.totalAgents }}</span>
            <span class="stat-label">{{ t('agentTeams.agents') }}</span>
          </div>
          <div class="stat-badge">
            <span class="stat-icon">💭</span>
            <span class="stat-value">{{ globalStats.thinking }}</span>
            <span class="stat-label">{{ t('agentTeams.thinking') }}</span>
          </div>
          <div class="stat-badge">
            <span class="stat-icon">⚡</span>
            <span class="stat-value">{{ globalStats.executing }}</span>
            <span class="stat-label">{{ t('agentTeams.executing') }}</span>
          </div>
          <div class="stat-badge success">
            <span class="stat-icon">📊</span>
            <span class="stat-value">{{ Math.round(globalStats.averageSuccessRate) }}%</span>
            <span class="stat-label">{{ t('agentTeams.successRate') }}</span>
          </div>
        </div>

        <!-- View mode selector -->
        <div class="view-selector">
          <button
            :class="['view-btn', { active: activeView === 'all' }]"
            @click="activeView = 'all'"
          >
            📊 {{ t('agentTeams.viewAll') }}
          </button>
          <button
            :class="['view-btn', { active: activeView === 'progress' }]"
            @click="activeView = 'progress'"
          >
            ⚡ {{ t('agentTeams.viewProgress') }}
          </button>
          <button
            :class="['view-btn', { active: activeView === 'collaboration' }]"
            @click="activeView = 'collaboration'"
          >
            🕸️ {{ t('agentTeams.viewCollaboration') }}
          </button>
          <button
            :class="['view-btn', { active: activeView === 'pets' }]"
            @click="activeView = 'pets'"
          >
            🐱 {{ t('agentTeams.viewPets') }}
          </button>
        </div>

        <!-- Sync status -->
        <div class="sync-status" @click="showSyncPanel = !showSyncPanel">
          <span class="sync-icon">🔄</span>
          <span class="sync-text">{{ t('agentTeams.syncStatus') }}</span>
          <span class="sync-badge connected">{{ t('agentTeams.syncConnected') }}</span>
        </div>
      </div>
    </header>

    <!-- Main content -->
    <div class="dashboard-content">
      <!-- All view - 综合视图 -->
      <template v-if="activeView === 'all'">
        <!-- Agent grid -->
        <div class="agents-section">
          <div class="section-header">
            <span class="section-icon">🤖</span>
            <span class="section-title">{{ t('agentTeams.agentTeam') }}</span>
          </div>

          <div class="agents-grid">
            <div
              v-for="agent in allAgents"
              :key="agent.agentId"
              :class="['agent-card', { selected: selectedAgentId === agent.agentId }]"
              @click="selectAgent(agent.agentId)"
            >
              <!-- Pet avatar -->
              <AgentPetAvatar
                v-if="petStore.pets.get(agent.agentId)"
                :agentId="agent.agentId"
                :agentName="agent.agentName"
                :currentActivity="agent.currentActivity.type"
                :emotion="petStore.pets.get(agent.agentId)?.emotion.current || 'focused'"
                @interaction="handleInteraction"
              />

              <!-- Mini progress indicator -->
              <div class="mini-progress">
                <span class="activity-icon">
                  {{ agent.currentActivity.type === 'thinking' ? '💭' :
                     agent.currentActivity.type === 'executing' ? '⚡' :
                     agent.currentActivity.type === 'outputting' ? '💬' : '😴' }}
                </span>
                <span class="activity-text">{{ agent.agentName }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Split view -->
        <div class="split-view">
          <!-- Left: Progress panel -->
          <div class="progress-section">
            <AgentRealtimeProgressPanel
              v-if="activeAgent"
              :agentId="activeAgent.agentId"
              :agentName="activeAgent.agentName"
              :initialStatus="activeAgent"
              @permission_response="handlePermissionResponse"
              @interaction="handleInteraction"
              @status_change="handleStatusChange"
            />
          </div>

          <!-- Right: Collaboration network -->
          <div class="collaboration-section">
            <EnhancedHermesDashboard />
          </div>
        </div>
      </template>

      <!-- Progress view -->
      <template v-else-if="activeView === 'progress'">
        <div class="progress-full-view">
          <AgentRealtimeProgressPanel
            v-if="activeAgent"
            :agentId="activeAgent.agentId"
            :agentName="activeAgent.agentName"
            :initialStatus="activeAgent"
            @permission_response="handlePermissionResponse"
            @interaction="handleInteraction"
            @status_change="handleStatusChange"
          />
        </div>
      </template>

      <!-- Collaboration view -->
      <template v-else-if="activeView === 'collaboration'">
        <div class="collaboration-full-view">
          <EnhancedHermesDashboard />
        </div>
      </template>

      <!-- Pets view -->
      <template v-else-if="activeView === 'pets'">
        <div class="pets-full-view">
          <!-- Pet grid -->
          <div class="pets-grid">
            <div
              v-for="pet in allPets"
              :key="pet.agentId"
              :class="['pet-card', { selected: selectedAgentId === pet.agentId }]"
              @click="selectAgent(pet.agentId)"
            >
              <AgentPetAvatar
                :agentId="pet.agentId"
                :agentName="pet.name"
                :currentActivity="realtimeStore.agents.get(pet.agentId)?.currentActivity.type || 'idle'"
                :emotion="pet.emotion.current"
                :petType="pet.appearance.type"
                @interaction="handleInteraction"
              />

              <EmotionDisplay
                :emotion="pet.emotion.current"
                :intensity="pet.emotion.intensity"
                :transitionDuration="pet.emotion.transitionDuration"
              />

              <GrowthSystem
                :level="pet.growth.level"
                :experience="pet.growth.experience"
                :experienceToNextLevel="pet.growth.experienceToNextLevel"
                :completedTasks="pet.growth.completedTasks"
                :successRate="pet.growth.successRate"
                :achievements="pet.growth.achievements"
              />

              <InteractionPanel
                :happinessLevel="pet.interaction.happinessLevel"
                :affinityScore="pet.interaction.affinityScore"
                :petCount="pet.interaction.petCount"
                :lastInteractionType="pet.interaction.lastInteractionType"
                @pet="handleInteraction('pet')"
                @poke="handleInteraction('poke')"
                @feed="handleInteraction('feed')"
                @play="handleInteraction('play')"
              />
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Sync panel (popup) -->
    <div v-if="showSyncPanel" class="sync-panel-overlay" @click="showSyncPanel = false">
      <div class="sync-panel" @click.stop>
        <div class="panel-header">
          <span class="panel-icon">🔄</span>
          <span class="panel-title">{{ t('agentTeams.syncPanelTitle') }}</span>
        </div>

        <div class="sync-devices">
          <div class="device-item web">
            <span class="device-icon">🌐</span>
            <span class="device-name">{{ t('agentTeams.webPlatform') }}</span>
            <span class="device-status connected">{{ t('agentTeams.syncConnected') }}</span>
          </div>
          <div class="device-item desktop">
            <span class="device-icon">🖥️</span>
            <span class="device-name">{{ t('agentTeams.desktopPlatform') }}</span>
            <span class="device-status pending">{{ t('agentTeams.syncPending') }}</span>
          </div>
          <div class="device-item mobile">
            <span class="device-icon">📱</span>
            <span class="device-name">{{ t('agentTeams.mobilePlatform') }}</span>
            <span class="device-status pending">{{ t('agentTeams.syncPending') }}</span>
          </div>
        </div>

        <div class="sync-info">
          <div class="info-row">
            <span class="info-label">{{ t('agentTeams.syncProtocol') }}</span>
            <span class="info-value">WebSocket</span>
          </div>
          <div class="info-row">
            <span class="info-label">{{ t('agentTeams.lastSync') }}</span>
            <span class="info-value">{{ new Date().toLocaleTimeString() }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-teams-dashboard {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--bg-base-dark, #0f172a) 0%, var(--bg-dark-mid, #1e293b) 50%, var(--bg-dark-deep, #0f172a) 100%);
  color: var(--text-primary);
  font-family: var(--font-mono, 'JetBrains Mono', monospace);
  animation: fadeIn 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.dashboard-header {
  padding: 16px 24px;
  background: rgba(30, 41, 59, 0.9);
  border-bottom: 1px solid var(--border-dark);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  flex: 1;
}

.dashboard-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.title-icon {
  font-size: 28px;
}

.dashboard-subtitle {
  font-size: 13px;
  color: var(--text-muted);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 24px;
}

.global-stats {
  display: flex;
  gap: 16px;
}

.stat-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(51, 65, 85, 0.3);
  border-radius: 8px;
  border: 1px solid var(--border-dark);
}

.stat-icon {
  font-size: 18px;
}

.stat-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-label {
  font-size: 11px;
  color: var(--text-muted);
}

.stat-badge.success .stat-value {
  color: var(--success);
}

.view-selector {
  display: flex;
  gap: 8px;
}

.view-btn {
  padding: 8px 16px;
  background: rgba(51, 65, 85, 0.2);
  border: 1px solid var(--border-dark);
  border-radius: 8px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.view-btn:hover {
  background: rgba(51, 65, 85, 0.4);
  color: var(--text-primary);
}

.view-btn.active {
  background: rgba(139, 92, 246, 0.2);
  border-color: var(--primary);
  color: var(--primary);
}

.sync-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.sync-status:hover {
  background: rgba(16, 185, 129, 0.2);
}

.sync-icon {
  font-size: 18px;
}

.sync-text {
  font-size: 12px;
  color: var(--success);
}

.sync-badge {
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 600;
}

.sync-badge.connected {
  background: rgba(16, 185, 129, 0.2);
  color: var(--success);
}

.dashboard-content {
  flex: 1;
  overflow: hidden;
  padding: 20px;
}

.agents-section {
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.section-icon {
  font-size: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.agents-grid {
  display: flex;
  gap: 16px;
  overflow-x: auto;
  padding-bottom: 8px;
}

.agent-card {
  padding: 16px;
  background: rgba(15, 23, 42, 0.6);
  border: 2px solid var(--border-dark);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.agent-card:hover {
  background: rgba(51, 65, 85, 0.4);
  border-color: var(--primary);
}

.agent-card.selected {
  background: rgba(139, 92, 246, 0.2);
  border-color: var(--primary);
}

.mini-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}

.activity-icon {
  font-size: 20px;
}

.activity-text {
  font-size: 12px;
  color: var(--text-primary);
}

.split-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  height: calc(100vh - 240px);
}

.progress-section,
.collaboration-section {
  overflow: hidden;
}

.progress-full-view,
.collaboration-full-view {
  height: calc(100vh - 120px);
}

.pets-full-view {
  height: calc(100vh - 120px);
  overflow-y: auto;
}

.pets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
}

.pet-card {
  padding: 20px;
  background: rgba(26, 26, 46, 0.6);
  border: 2px solid #3a3a5a;
  border-radius: 16px;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.pet-card:hover {
  background: rgba(58, 58, 90, 0.4);
}

.pet-card.selected {
  background: rgba(139, 92, 246, 0.2);
  border-color: #8b5cf6;
}

.sync-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.sync-panel {
  padding: 24px;
  background: rgba(26, 26, 46, 0.95);
  border: 2px solid #8b5cf6;
  border-radius: 16px;
  width: 400px;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.panel-icon {
  font-size: 24px;
}

.panel-title {
  font-size: 18px;
  font-weight: 600;
  color: #e0e0e0;
}

.sync-devices {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.device-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: rgba(58, 58, 90, 0.2);
  border-radius: 8px;
}

.device-icon {
  font-size: 20px;
}

.device-name {
  font-size: 14px;
  color: #e0e0e0;
  flex: 1;
}

.device-status {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
}

.device-status.connected {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.device-status.pending {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.sync-info {
  padding: 16px;
  background: rgba(58, 58, 90, 0.2);
  border-radius: 8px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.info-label {
  font-size: 12px;
  color: var(--text-muted);
}

.info-value {
  font-size: 13px;
  color: var(--text-primary);
}

/* Responsive breakpoints */
@media (max-width: 768px) {
  .dashboard-header {
    flex-direction: column;
    gap: 12px;
    padding: 12px 16px;
  }

  .header-right {
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .global-stats {
    width: 100%;
    justify-content: space-around;
  }

  .view-selector {
    width: 100%;
    justify-content: space-around;
  }

  .split-view {
    grid-template-columns: 1fr;
  }

  .pets-grid {
    grid-template-columns: 1fr;
  }
}
</style>