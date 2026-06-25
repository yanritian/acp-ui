<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from '@/locales'
import { CORE_FEATURES, ADVANCED_FEATURES } from '@/lib/feature-registry'
import { trackBehavior } from '@/lib/self-improvement'
import AgentSelector from '@/features/config/AgentSelector.vue'
import SessionList from '@/features/chat/SessionList.vue'
import StartupProgress from '@/shared/ui/StartupProgress.vue'
import LanguageSelector from '@/shared/ui/LanguageSelector.vue'
import { icons } from '@/shared/icons'
import type { SavedSession } from '@/lib/types'

const props = defineProps<{
  showSidebar: boolean
  isNarrowLayout: boolean
  isDrawer: boolean
  selectedAgent: string
  selectedCwd: string
  folderPickerAvailable: boolean
  isConnected: boolean
  isConnecting: boolean
  isLoading: boolean
  hasAgents: boolean
  showLogStream: boolean
  showTrafficMonitor: boolean
  startupPhase?: string
  startupLogs?: string[]
  startupElapsed?: number
}>()

const icon = icons

const emit = defineEmits<{
  (e: 'toggle-sidebar'): void
  (e: 'toggle-log-stream'): void
  (e: 'toggle-traffic-monitor'): void
  (e: 'open-settings'): void
  (e: 'agent-select', agentName: string): void
  (e: 'select-folder'): void
  (e: 'cwd-input', event: Event): void
  (e: 'new-session'): void
  (e: 'cancel-connection'): void
  (e: 'toggle-startup-details'): void
  (e: 'disconnect'): void
  (e: 'resume-session', session: SavedSession): void
  (e: 'delete-session', sessionId: string): void
  (e: 'update:selectedAgent', value: string): void
  (e: 'open-tank-game'): void
}>()

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const showAdvancedMenu = ref(false)

// Translated features - grouped into core and advanced
const coreFeatures = computed(() =>
  CORE_FEATURES.map(f => ({
    ...f,
    label: t(f.labelKey as any),
    description: t(f.descriptionKey as any),
  }))
)

const advancedFeatures = computed(() =>
  ADVANCED_FEATURES.map(f => ({
    ...f,
    label: t(f.labelKey as any),
    description: t(f.descriptionKey as any),
  }))
)

// Current view name from route
const currentViewName = computed(() => route.name as string)

function navigateToFeature(featureId: string) {
  router.push('/' + featureId)
  trackBehavior(`feature-${featureId}`, { from: currentViewName.value })
}

function handleAgentSelect(agentName: string) {
  emit('update:selectedAgent', agentName)
  emit('agent-select', agentName)
}
</script>

<template>
  <aside
    v-show="showSidebar"
    class="sidebar"
    :class="{ 'is-drawer': isDrawer }"
  >
    <div class="sidebar-header">
      <h1>ACP UI</h1>
      <div class="header-actions">
        <button
          class="settings-btn"
          :class="{ active: showLogStream }"
          @click="emit('toggle-log-stream')"
          title="Agent Log Stream"
        >
          <span v-html="icon.fileText"></span>
        </button>
        <button
          class="settings-btn"
          :class="{ active: showTrafficMonitor }"
          @click="emit('toggle-traffic-monitor')"
          title="ACP Traffic Monitor"
        >
          <span v-html="icon.activity"></span>
        </button>
        <button
          class="settings-btn game-btn"
          @click="emit('open-tank-game')"
          title="坦克大战 Demo"
        >
          🎮
        </button>
        <button class="settings-btn" @click="emit('open-settings')" title="Settings">
          <span v-html="icon.settings"></span>
        </button>
        <button class="toggle-btn" @click="emit('toggle-sidebar')">
          <span class="toggle-icon" v-html="icon.arrowRight"></span>
        </button>
      </div>
    </div>
    
    <div class="sidebar-content">
      <!-- Agent Selection -->
      <div class="section">
        <AgentSelector 
          :selected="selectedAgent"
          @select="handleAgentSelect"
          @update:selected="handleAgentSelect"
        />
        
        <!-- Working Directory Picker -->
        <div class="cwd-picker">
          <label>{{ t('common.workingDirectory') }}</label>
          <!-- Desktop: read-only display + folder picker. -->
          <div v-if="folderPickerAvailable" class="cwd-row">
            <span class="cwd-path" :title="selectedCwd || 'Current directory'">
              {{ selectedCwd ? selectedCwd.split(/[\\/]/).pop() : '.' }}
            </span>
            <button
              class="cwd-btn"
              @click="emit('select-folder')"
              :title="t('common.selectFolder')"
              :disabled="isConnecting || isConnected"
            >
              📁
            </button>
          </div>
          <!-- Mobile / web: free-text input. The cwd is interpreted by
               the remote agent, so the path must exist on the agent's
               machine. -->
          <input
            v-else
            class="cwd-input"
            type="text"
            :value="selectedCwd"
            @input="emit('cwd-input', $event)"
            :disabled="isConnecting || isConnected"
            :placeholder="t('common.inputPlaceholder')"
            autocapitalize="none"
            autocorrect="off"
            spellcheck="false"
          />
        </div>

        <button
          v-if="hasAgents && !isConnected && !isConnecting"
          class="new-session-btn"
          :disabled="!selectedAgent || isLoading"
          @click="emit('new-session')"
        >
          {{ isLoading ? t('common.connecting') : t('common.newSession') }}
        </button>
        
        <!-- Startup Progress -->
        <StartupProgress 
          v-if="isConnecting"
          :agent-name="selectedAgent"
          :phase="startupPhase || ''"
          :logs="startupLogs || []"
          :elapsed-seconds="startupElapsed || 0"
          :show-details="false"
          @cancel="emit('cancel-connection')"
          @toggle-details="emit('toggle-startup-details')"
        />
        
        <button 
          v-if="isConnected"
          class="disconnect-btn"
          @click="emit('disconnect')"
        >
          {{ t('common.disconnect') }}
        </button>
      </div>

      <!-- Session List -->
      <div class="section">
        <SessionList
          @resume="(session: SavedSession) => emit('resume-session', session)"
          @delete="(sessionId: string) => emit('delete-session', sessionId)"
        />
      </div>

      <!-- View Navigation -->
      <div class="section view-nav">
        <h3 class="nav-title">{{ t('common.featureNavigation') }}</h3>
        <nav class="nav-buttons">
          <!-- Core features (always visible) -->
          <button
            v-for="feature in coreFeatures"
            :key="feature.id"
            :class="['nav-btn', { active: currentViewName === feature.id }]"
            @click="navigateToFeature(feature.id)"
            :title="feature.description"
          >
            <span class="nav-icon">{{ feature.icon }}</span>
            <span class="nav-text">{{ feature.label }}</span>
          </button>

          <!-- More menu for advanced features -->
          <div class="more-menu">
            <button
              class="nav-btn more-btn"
              @click="showAdvancedMenu = !showAdvancedMenu"
              :title="t('common.moreFeatures')"
            >
              <span class="nav-icon" v-html="icon.settings"></span>
              <span class="nav-text">{{ t('common.moreFeatures') }} {{ showAdvancedMenu ? '▼' : '▶' }}</span>
            </button>

            <!-- Advanced features (collapsible) -->
            <div v-if="showAdvancedMenu" class="advanced-features">
              <button
                v-for="feature in advancedFeatures"
                :key="feature.id"
                :class="['nav-btn advanced-btn', { active: currentViewName === feature.id }]"
                @click="navigateToFeature(feature.id)"
                :title="feature.description"
              >
                <span class="nav-icon">{{ feature.icon }}</span>
                <span class="nav-text">{{ feature.label }}</span>
              </button>
            </div>
          </div>
        </nav>
      </div>

      <!-- Language Selector -->
      <div class="section language-section">
        <LanguageSelector />
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 320px;
  min-width: 320px;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, rgba(0, 102, 204, 0.05), rgba(99, 102, 241, 0.05));
}

.sidebar-header h1 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 0.25rem;
}

.settings-btn,
.toggle-btn {
  padding: 0.25rem 0.5rem;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-muted);
}

.settings-btn:hover,
.toggle-btn:hover {
  color: var(--text-primary);
}

.settings-btn.active {
  color: var(--text-accent);
  background: var(--bg-hover);
  border-radius: 4px;
}

.game-btn {
  font-size: 1.1rem;
}

.game-btn:hover {
  color: #4CAF50;
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
}

.section {
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.language-section {
  padding: 0.5rem 1rem;
  border-bottom: none;
  margin-top: auto;
}

.new-session-btn,
.disconnect-btn {
  width: 100%;
  margin-top: 0.75rem;
  padding: 0.625rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
}

.new-session-btn {
  background: var(--bg-primary);
  color: white;
}

.new-session-btn:hover:not(:disabled) {
  background: var(--bg-primary-hover);
}

.new-session-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cwd-picker {
  margin-top: 0.75rem;
}

.cwd-picker label {
  display: block;
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.cwd-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.cwd-path {
  flex: 1;
  padding: 0.375rem 0.5rem;
  background: var(--bg-main);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 0.8rem;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cwd-btn {
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 1rem;
}

.cwd-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.cwd-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Mobile-only free-text cwd input. */
.cwd-input {
  width: 100%;
  padding: 0.5rem 0.6rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-main);
  color: var(--text-primary);
  font-size: 16px; /* 16px = no iOS auto-zoom */
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.cwd-input:disabled {
  opacity: 0.5;
}

.view-nav {
  padding-top: 12px;
}

.nav-title {
  margin: 0 0 12px 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.nav-buttons {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

/* Core features - slightly larger and more prominent */
.nav-btn.core-feature {
  font-weight: 500;
}

/* More menu button */
.more-btn {
  border-top: 1px solid var(--border-color);
  margin-top: 8px;
  padding-top: 12px;
  font-weight: 500;
}

/* Advanced features container */
.advanced-features {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 4px;
  padding-left: 12px;
  border-left: 2px solid var(--border-color);
}

.advanced-btn {
  font-size: 13px;
  padding: 8px 12px;
  opacity: 0.85;
}

.advanced-btn:hover {
  opacity: 1;
}

.nav-icon {
  font-size: 18px;
  width: 24px;
  text-align: center;
}

.nav-text {
  flex: 1;
}

.nav-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-btn.active {
  background: linear-gradient(135deg, var(--bg-primary), var(--bg-primary-hover));
  color: white;
  box-shadow: 0 2px 8px rgba(0, 102, 204, 0.3);
}

.nav-btn.highlight {
  border: 1px solid var(--bg-primary);
}

.nav-btn.highlight:hover {
  background: rgba(0, 102, 204, 0.1);
}

.nav-btn.highlight.active {
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border: none;
}

.disconnect-btn {
  background: var(--bg-danger);
  color: white;
}

.disconnect-btn:hover {
  background: #c82333;
}

/* Mobile / narrow-viewport layout */
@media (max-width: 800px) {
  .sidebar.is-drawer {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 85vw;
    max-width: 360px;
    z-index: 100;
    box-shadow: 2px 0 16px rgba(0, 0, 0, 0.3);
    /* Honour iOS notch / Android status bar. */
    padding-top: env(safe-area-inset-top, 0px);
  }

  /* Tap-target sizing for the icon buttons inside the sidebar header. */
  .settings-btn,
  .toggle-btn {
    min-width: 40px;
    min-height: 40px;
    font-size: 1rem;
  }
}
</style>
