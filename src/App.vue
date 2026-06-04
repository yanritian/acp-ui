<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useConfigStore } from './stores/config'
import { useSessionStore } from './stores/session'
import { useMultiSessionStore } from './stores/multi-session'
import { useI18n } from './locales'
import { trackBehavior } from './lib/self-improvement'
import { startEvolutionEngine } from './lib/self-improvement'
import type { SavedSession } from './lib/types'
import './assets/modern.css'

// Composables
import { useResponsiveLayout } from './composables/useResponsiveLayout'
import { useReconnect } from './composables/useReconnect'
import { usePreferences } from './composables/usePreferences'
import { useBotCommand } from './composables/useBotCommand'

// Components
import AppSidebar from './components/AppSidebar.vue'
import ConnectionBanner from './components/ConnectionBanner.vue'
import WelcomeScreen from './components/WelcomeScreen.vue'
import PermissionDialog from './components/PermissionDialog.vue'
import SettingsView from './components/SettingsView.vue'
import AuthMethodDialog from './components/AuthMethodDialog.vue'
import TrafficMonitor from './components/TrafficMonitor.vue'
import LogStreamView from './components/LogStreamView.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

// Stores
const configStore = useConfigStore()
const sessionStore = useSessionStore()
const multiSession = useMultiSessionStore()

// Composables
const { showSidebar, isNarrowLayout, toggleSidebar, handleBackdropClick } = useResponsiveLayout()
const { handleManualReconnect } = useReconnect()
const { selectedCwd, folderPickerAvailable, loadPreferences, handleSelectFolder, handleCwdInput } = usePreferences()
useBotCommand()

// Local state
const selectedAgent = ref('')
const showSettings = ref(false)
const showTrafficMonitor = ref(false)
const showStartupDetails = ref(false)
const showLogStream = ref(false)

// Computed properties from stores
const isConnected = computed(() => sessionStore.isConnected)
const isLoading = computed(() => sessionStore.isLoading)
const isConnecting = computed(() => sessionStore.isConnecting)
const isReconnecting = computed(() => sessionStore.isReconnecting)
const error = computed(() => sessionStore.error || configStore.error)
const hasAgents = computed(() => configStore.hasAgents)

// Name of the agent the reconnect banner refers to. Falls back to the
// generic "agent" if the saved session has no name (shouldn't happen).
const reconnectingAgentName = computed(
  () => sessionStore.currentSession?.agentName ?? 'agent'
)

// True when there is a saved session we *could* reconnect to but the
// transport is currently down. Surfaces a manual "Reconnect" affordance on
// the error banner so users don't have to background+foreground the app to
// trigger the auto path.
const canManuallyReconnect = computed(
  () =>
    !isConnected.value &&
    !isReconnecting.value &&
    !isConnecting.value &&
    !!sessionStore.currentSession?.supportsLoadSession
)

// Watch for permission requests from session store
const pendingPermission = computed(() => sessionStore.pendingPermission)

// Watch for auth method selection requests
const pendingAuthMethods = computed(() => sessionStore.pendingAuthMethods)
const pendingAuthAgentName = computed(() => sessionStore.pendingAuthAgentName)

// Current view name from route
const currentViewName = computed(() => route.name as string)

// Is the current view the chat view?
const isChatView = computed(() => currentViewName.value === 'chat')

onMounted(async () => {
  // Load persisted preferences
  await loadPreferences()
  
  // Initialize stores
  await configStore.loadConfig()
  await configStore.setupHotReload()
  await sessionStore.initStore()
  await multiSession.initStore()

  // Start self-improvement engine (analyzes patterns every 5 minutes)
  startEvolutionEngine()

  // Track initial feature usage
  trackBehavior('app-started', { agentCount: configStore.hasAgents ? 'yes' : 'no' })
})

async function handleAgentSelect(agentName: string) {
  selectedAgent.value = agentName
}

async function handleNewSession() {
  if (!selectedAgent.value) return

  // ACP requires an absolute working directory; passing '.' is rejected by
  // most agents. On desktop the folder picker always returns an absolute
  // path, but on mobile the user types it, so validate up-front and surface
  // a helpful error rather than letting the agent reject `session/new`.
  const cwd = selectedCwd.value.trim()
  if (!cwd) {
    sessionStore.error = t('errors.workingDirectoryRequired')
    return
  }
  const isAbsolute = cwd.startsWith('/') || /^[A-Za-z]:[\\/]/.test(cwd)
  if (!isAbsolute) {
    sessionStore.error = t('errors.workingDirectoryNotAbsolute', { path: cwd })
    return
  }

  try {
    await sessionStore.createSession(selectedAgent.value, cwd)
    // Navigate to chat view after successful connection
    router.push('/chat')
  } catch (e) {
    console.error('Failed to create session:', e)
  }
}

async function handleResumeSession(session: SavedSession) {
  selectedAgent.value = session.agentName
  try {
    await sessionStore.resumeSession(session)
    // Navigate to chat view after resuming
    router.push('/chat')
  } catch (e) {
    console.error('Failed to resume session:', e)
  }
}

async function handleDeleteSession(sessionId: string) {
  await sessionStore.deleteSession(sessionId)
}

async function handleDisconnect() {
  await sessionStore.disconnect()
}

async function handleCancelConnection() {
  await sessionStore.cancelConnection()
}

function handlePermissionSelect(optionId: string) {
  sessionStore.resolvePermission(optionId)
}

function handlePermissionCancel() {
  sessionStore.cancelPermission()
}

function handleAuthMethodSelect(methodId: string) {
  sessionStore.selectAuthMethod(methodId)
}

function handleAuthMethodCancel() {
  sessionStore.cancelAuthSelection()
}

function clearError() {
  sessionStore.clearError()
  configStore.clearError()
}
</script>

<template>
  <div class="app-container" :class="{ 'narrow-layout': isNarrowLayout }">
    <!-- Sidebar (slides in as a drawer on narrow viewports) -->
    <AppSidebar
      v-model:selectedAgent="selectedAgent"
      :show-sidebar="showSidebar"
      :is-narrow-layout="isNarrowLayout"
      :is-drawer="isNarrowLayout"
      :selected-cwd="selectedCwd"
      :folder-picker-available="folderPickerAvailable"
      :is-connected="isConnected"
      :is-connecting="isConnecting"
      :is-loading="isLoading"
      :has-agents="hasAgents"
      :show-log-stream="showLogStream"
      :show-traffic-monitor="showTrafficMonitor"
      :startup-phase="sessionStore.startupPhase"
      :startup-logs="sessionStore.startupLogs"
      :startup-elapsed="sessionStore.startupElapsed"
      @toggle-sidebar="toggleSidebar"
      @toggle-log-stream="showLogStream = !showLogStream"
      @toggle-traffic-monitor="showTrafficMonitor = !showTrafficMonitor"
      @open-settings="showSettings = true"
      @agent-select="handleAgentSelect"
      @select-folder="handleSelectFolder"
      @cwd-input="handleCwdInput"
      @new-session="handleNewSession"
      @cancel-connection="handleCancelConnection"
      @toggle-startup-details="showStartupDetails = !showStartupDetails"
      @disconnect="handleDisconnect"
      @resume-session="handleResumeSession"
      @delete-session="handleDeleteSession"
    />
    
    <!-- Backdrop behind the drawer on narrow viewports. Only intercepts
         taps when the layout is narrow; on desktop it's display:none. -->
    <div
      v-show="isNarrowLayout && showSidebar"
      class="drawer-backdrop"
      @click="handleBackdropClick"
    />

    <!-- Mobile hamburger to open the drawer when collapsed. The desktop
         chevron toggle (`.sidebar-toggle-collapsed`) is hidden on narrow
         viewports via CSS so we don't show two affordances. -->
    <button
      v-show="isNarrowLayout && !showSidebar"
      class="mobile-hamburger"
      @click="showSidebar = true"
      aria-label="Open menu"
    >☰</button>

    <!-- Collapsed sidebar toggle -->
    <button 
      v-if="!showSidebar" 
      class="sidebar-toggle-collapsed"
      @click="toggleSidebar"
    >
      ▶
    </button>
    
    <!-- Main Content Area -->
    <div class="main-area">
      <main class="main-content">
        <!-- Connection banners (reconnect / error) -->
        <ConnectionBanner
          :is-reconnecting="isReconnecting"
          :reconnecting-agent-name="reconnectingAgentName"
          :error="error"
          :can-manually-reconnect="canManuallyReconnect"
          @manual-reconnect="handleManualReconnect"
          @clear-error="clearError"
        />
        
        <!-- Special handling for chat view when not connected (show welcome screen) -->
        <WelcomeScreen
          v-if="isChatView && !isConnected"
          :has-agents="hasAgents"
          @open-settings="showSettings = true"
        />
        
        <!-- Default router view for all other routes -->
        <router-view v-else />
      </main>

      <!-- Traffic Monitor Panel -->
      <div v-if="showTrafficMonitor" class="traffic-panel">
        <TrafficMonitor @close="showTrafficMonitor = false" />
      </div>
    </div>

    <!-- Permission Dialog -->
    <PermissionDialog
      v-if="pendingPermission"
      :request="pendingPermission"
      @select="handlePermissionSelect"
      @cancel="handlePermissionCancel"
    />

    <!-- Auth Method Dialog -->
    <AuthMethodDialog 
      v-if="pendingAuthMethods.length > 0"
      :auth-methods="pendingAuthMethods"
      :agent-name="pendingAuthAgentName"
      @select="handleAuthMethodSelect"
      @cancel="handleAuthMethodCancel"
    />

    <!-- Settings -->
    <SettingsView
      v-if="showSettings"
      @close="showSettings = false"
    />

    <!-- Log Stream Panel (Agent Real-time Logs) -->
    <LogStreamView
      v-if="showLogStream"
      @close="showLogStream = false"
      @resize="() => {}"
    />
  </div>
</template>

<style>
:root {
  --bg-primary: #0066cc;
  --bg-primary-hover: #0052a3;
  --bg-sidebar: #f8f9fa;
  --bg-main: #ffffff;
  --bg-hover: #f0f0f0;
  --bg-user: #e3f2fd;
  --bg-assistant: #f5f5f5;
  --bg-code: #282c34;
  --bg-success: #28a745;
  --bg-danger: #dc3545;
  --bg-warning: #fff3cd;
  --text-primary: #333;
  --text-secondary: #666;
  --text-muted: #999;
  --text-accent: #0066cc;
  --text-code: #abb2bf;
  --border-color: #e0e0e0;
  
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: var(--text-primary);
  background-color: #ffffff;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #4da6ff;
    --bg-primary-hover: #3399ff;
    --bg-sidebar: #1e1e1e;
    --bg-main: #252525;
    --bg-hover: #333;
    --bg-user: #1a3a5c;
    --bg-assistant: #2d2d2d;
    --text-primary: #e0e0e0;
    --text-secondary: #a0a0a0;
    --text-muted: #707070;
    --text-accent: #4da6ff;
    --border-color: #404040;
    background-color: #252525;
  }
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body, #app {
  height: 100%;
}
</style>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.drawer-backdrop {
  display: none;
}

.mobile-hamburger {
  display: none;
}

.sidebar-toggle-collapsed {
  position: fixed;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-left: none;
  border-radius: 0 4px 4px 0;
  background: var(--bg-sidebar);
  cursor: pointer;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-main);
}

.traffic-panel {
  flex-shrink: 0;
  border-top: 2px solid var(--border-color);
}

/* ---------- Mobile / narrow-viewport layout ---------- */

@media (max-width: 800px) {
  .app-container {
    /* Prevent the off-screen drawer from causing horizontal scroll. */
    overflow-x: hidden;
  }

  .drawer-backdrop {
    display: block;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 90;
  }

  /* Hide the desktop chevron when the mobile hamburger is in play. */
  .sidebar-toggle-collapsed {
    display: none;
  }

  .mobile-hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
    position: fixed;
    top: calc(env(safe-area-inset-top, 0px) + 0.5rem);
    left: 0.5rem;
    z-index: 50;
    width: 44px;
    height: 44px;
    padding: 0;
    background: var(--bg-sidebar);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    font-size: 1.25rem;
    cursor: pointer;
    color: var(--text-primary);
  }

  /* Honour the iOS home indicator at the bottom of the main area. */
  .main-area {
    padding-bottom: env(safe-area-inset-bottom, 0px);
  }
}
</style>
