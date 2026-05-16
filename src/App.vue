<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { canPickFolder, pickFolder, loadKvStore, type KVStore } from './lib/host';
import { useConfigStore } from './stores/config';
import { useSessionStore } from './stores/session';
import { useMultiSessionStore } from './stores/multi-session';
import { useTeamRuntimeStore } from './stores/team-runtime';
import { initTelemetry } from './lib/telemetry';
import AgentSelector from './components/AgentSelector.vue';
import SessionList from './components/SessionList.vue';
import ChatView from './components/ChatView.vue';
import PermissionDialog from './components/PermissionDialog.vue';
import SettingsView from './components/SettingsView.vue';
import AuthMethodDialog from './components/AuthMethodDialog.vue';
import TrafficMonitor from './components/TrafficMonitor.vue';
import StartupProgress from './components/StartupProgress.vue';
import MultiAgentChat from './components/MultiAgentChat.vue';
import HistoryView from './components/HistoryView.vue';
import WorkflowView from './components/WorkflowView.vue';
import GatewaySettings from './components/GatewaySettings.vue';
import TeamOrchestrationView from './components/TeamOrchestrationView.vue';
import MultiSessionChat from './components/MultiSessionChat.vue';
import MemoryView from './components/MemoryView.vue';
import ErrorView from './components/ErrorView.vue';
import EvolutionView from './components/EvolutionView.vue';
import PatternView from './components/PatternView.vue';
import HermesDashboard from './components/HermesDashboard.vue';
import EnhancedHermesDashboard from './components/EnhancedHermesDashboard.vue';
import TaskGraphView from './components/TaskGraphView.vue';
import LogStreamView from './components/LogStreamView.vue';
import AgentTeamsDashboard from './views/AgentTeamsDashboard.vue';
import { FEATURES } from './lib/feature-registry'
import { startEvolutionEngine, trackBehavior } from './lib/self-improvement'
import { taskParser, type TaskDAG } from './lib/task-parser'
import './assets/modern.css'
import type { SavedSession } from './lib/types';

const configStore = useConfigStore();
const sessionStore = useSessionStore();
const multiSession = useMultiSessionStore();
const teamRuntime = useTeamRuntimeStore();

const selectedAgent = ref('');
const selectedCwd = ref('');
// On mobile / web there is no native folder picker (and the cwd refers to
// a path on the *agent's* machine, not the local device), so we expose a
// free-text field instead of the picker button.
const folderPickerAvailable = canPickFolder();
const showSidebar = ref(true);
const showSettings = ref(false);
const showTrafficMonitor = ref(false);
const showStartupDetails = ref(false);
// View types
const currentView = ref<'chat' | 'multi-agent' | 'multi-session' | 'status' | 'monitor' | 'history' | 'workflow' | 'gateway' | 'orchestration' | 'bot' | 'memory' | 'error' | 'evolution' | 'pattern' | 'hermes' | 'task-graph' | 'collaboration' | 'agent-teams'>('chat');
const showLogStream = ref(false);

// Mock Task DAG for demo
const mockTaskDag = ref<TaskDAG | null>(null);

function initializeMockDag() {
  // Create a sample DAG for visualization
  const nodes = new Map<string, any>();
  const edges = new Map<string, string[]>();

  const steps = [
    { id: 'step-1', name: 'Parse Request', status: 'completed', agent: 'planner-001', deps: [] },
    { id: 'step-2', name: 'Design Architecture', status: 'running', agent: 'architect-001', deps: ['step-1'] },
    { id: 'step-3', name: 'Write Tests', status: 'pending', agent: 'tddGuide-001', deps: ['step-2'] },
    { id: 'step-4', name: 'Implement Code', status: 'pending', agent: 'codeReviewer-001', deps: ['step-2'] },
    { id: 'step-5', name: 'Security Audit', status: 'pending', agent: 'securityReviewer-001', deps: ['step-3', 'step-4'] },
    { id: 'step-6', name: 'Build & Deploy', status: 'pending', agent: 'build-001', deps: ['step-5'] },
  ];

  for (const step of steps) {
    // Use dag.id prefix for node IDs to match edge building logic
    const nodeId = `demo-dag-001-${step.id}`;
    nodes.set(nodeId, {
      id: nodeId,
      step: {
        id: step.id,
        name: step.name,
        action: 'task',
        agentType: 'general',
        // Dependencies use short IDs - TaskGraphView will add dag.id prefix
        dependencies: step.deps
      },
      status: step.status as 'pending' | 'running' | 'completed' | 'failed' | 'blocked',
      assignedAgent: step.agent,
    });

    // Build reverse edges: dependency -> dependent
    for (const dep of step.deps) {
      const depNodeId = `demo-dag-001-${dep}`;
      if (!edges.has(depNodeId)) {
        edges.set(depNodeId, []);
      }
      edges.get(depNodeId)!.push(nodeId);
    }
  }

  mockTaskDag.value = {
    id: 'demo-dag-001',
    name: 'Feature Development Workflow',
    nodes,
    edges,
    rootNodes: ['demo-dag-001-step-1'],
  };
}

// Reactive flag tracking whether the viewport is narrow enough to show the
// sidebar as a slide-in drawer (mobile / very narrow desktop windows). Used
// by the template to decide when the backdrop is interactive and by
// onMounted to default the drawer closed.
const isNarrowLayout = ref(false);
let narrowMql: MediaQueryList | null = null;
function syncNarrowLayout() {
  if (narrowMql) isNarrowLayout.value = narrowMql.matches;
}

// Foreground-reconnect plumbing. Mobile OSes freeze the WebView when the
// app is backgrounded and routers may drop the idle TCP connection while
// we're away. When the user returns we ask the session store to silently
// reattach to the last session if we have one.
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
function scheduleReconnect() {
  // Coalesce rapid visibility/online flips into a single attempt.
  if (reconnectTimer) return;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    // Skip if the browser still thinks we're offline; we'll be re-triggered
    // by the `online` event when connectivity returns.
    if (typeof navigator !== 'undefined' && navigator.onLine === false) return;
    void sessionStore.tryReconnect();
  }, 250);
}
function handleVisibilityChange() {
  if (typeof document !== 'undefined' && !document.hidden) scheduleReconnect();
}
function handleOnline() {
  scheduleReconnect();
}

// Preferences store for persisting user selections
let prefsStore: KVStore | null = null;

const isConnected = computed(() => sessionStore.isConnected);
const isLoading = computed(() => sessionStore.isLoading);
const isConnecting = computed(() => sessionStore.isConnecting);
const isReconnecting = computed(() => sessionStore.isReconnecting);
const error = computed(() => sessionStore.error || configStore.error);
const hasAgents = computed(() => configStore.hasAgents);

// Name of the agent the reconnect banner refers to. Falls back to the
// generic "agent" if the saved session has no name (shouldn't happen).
const reconnectingAgentName = computed(
  () => sessionStore.currentSession?.agentName ?? 'agent'
);

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
);

async function handleManualReconnect() {
  await sessionStore.tryReconnect();
}

// Watch for permission requests from session store
const pendingPermission = computed(() => sessionStore.pendingPermission);

// Watch for auth method selection requests
const pendingAuthMethods = computed(() => sessionStore.pendingAuthMethods);
const pendingAuthAgentName = computed(() => sessionStore.pendingAuthAgentName);

onMounted(async () => {
  // Initialize mock Task DAG for demo
  initializeMockDag();

  // Track viewport width so the sidebar can default-collapse into a drawer
  // on phones / narrow windows. We watch a MediaQueryList rather than
  // resize for correctness across orientation changes on iOS.
  if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
    narrowMql = window.matchMedia('(max-width: 800px)');
    syncNarrowLayout();
    narrowMql.addEventListener('change', syncNarrowLayout);
    if (isNarrowLayout.value) showSidebar.value = false;
  }

  // Load persisted preferences first
  prefsStore = await loadKvStore('preferences.json');
  
  // Initialize telemetry (check user preference)
  const telemetryEnabled = await prefsStore.get<boolean>('telemetryEnabled') ?? true;
  await initTelemetry(telemetryEnabled);
  
  // Initialize stores
  await configStore.loadConfig();
  await configStore.setupHotReload();
  await sessionStore.initStore();
  await multiSession.initStore();

  // Start self-improvement engine (analyzes patterns every 5 minutes)
  startEvolutionEngine();

  // Track initial feature usage
  trackBehavior('app-started', { agentCount: configStore.hasAgents ? 'yes' : 'no' });
  
  const savedCwd = await prefsStore.get<string>('lastCwd');
  if (savedCwd) {
    selectedCwd.value = savedCwd;
  }

  // Hook foreground-reconnect listeners. `pageshow` fires both on initial
  // navigation and when iOS restores a frozen WebView from the back/forward
  // cache, so it complements `visibilitychange` on Safari/iOS.
  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', handleVisibilityChange);
  }
  if (typeof window !== 'undefined') {
    window.addEventListener('pageshow', scheduleReconnect);
    window.addEventListener('online', handleOnline);
  }
});

async function handleAgentSelect(agentName: string) {
  selectedAgent.value = agentName;
}

async function handleSelectFolder() {
  const folder = await pickFolder('Select Working Directory');
  if (folder) {
    selectedCwd.value = folder;
    // Persist the selection
    if (prefsStore) {
      await prefsStore.set('lastCwd', folder);
      await prefsStore.save();
    }
  }
}

/** Persist a typed cwd as the user edits it (mobile / web field). */
async function handleCwdInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  selectedCwd.value = value;
  if (prefsStore) {
    await prefsStore.set('lastCwd', value);
    await prefsStore.save();
  }
}

async function handleNewSession() {
  if (!selectedAgent.value) return;

  // ACP requires an absolute working directory; passing '.' is rejected by
  // most agents. On desktop the folder picker always returns an absolute
  // path, but on mobile the user types it, so validate up-front and surface
  // a helpful error rather than letting the agent reject `session/new`.
  const cwd = selectedCwd.value.trim();
  if (!cwd) {
    sessionStore.error = 'Please enter a working directory (absolute path on the agent\u2019s machine).';
    return;
  }
  const isAbsolute = cwd.startsWith('/') || /^[A-Za-z]:[\\/]/.test(cwd);
  if (!isAbsolute) {
    sessionStore.error = `Working directory must be an absolute path (got: ${cwd}).`;
    return;
  }

  try {
    await sessionStore.createSession(selectedAgent.value, cwd);
  } catch (e) {
    console.error('Failed to create session:', e);
  }
}

async function handleResumeSession(session: SavedSession) {
  selectedAgent.value = session.agentName;
  try {
    await sessionStore.resumeSession(session);
  } catch (e) {
    console.error('Failed to resume session:', e);
  }
}

async function handleDeleteSession(sessionId: string) {
  await sessionStore.deleteSession(sessionId);
}

async function handleDisconnect() {
  await sessionStore.disconnect();
}

async function handleCancelConnection() {
  await sessionStore.cancelConnection();
}

function handlePermissionSelect(optionId: string) {
  sessionStore.resolvePermission(optionId);
}

function handlePermissionCancel() {
  sessionStore.cancelPermission();
}

function handleAuthMethodSelect(methodId: string) {
  sessionStore.selectAuthMethod(methodId);
}

function handleAuthMethodCancel() {
  sessionStore.cancelAuthSelection();
}

function toggleSidebar() {
  showSidebar.value = !showSidebar.value;
}

function navigateToFeature(featureId: string) {
  currentView.value = featureId as typeof currentView.value
  trackBehavior(`feature-${featureId}`, { from: currentView.value })
}

/** Close the drawer when the user taps the backdrop on a narrow viewport. */
function handleBackdropClick() {
  if (isNarrowLayout.value) showSidebar.value = false;
}

onBeforeUnmount(() => {
  if (narrowMql) {
    narrowMql.removeEventListener('change', syncNarrowLayout);
    narrowMql = null;
  }
  if (typeof document !== 'undefined') {
    document.removeEventListener('visibilitychange', handleVisibilityChange);
  }
  if (typeof window !== 'undefined') {
    window.removeEventListener('pageshow', scheduleReconnect);
    window.removeEventListener('online', handleOnline);
  }
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
});

function clearError() {
  sessionStore.clearError();
  configStore.clearError();
}
</script>

<template>
  <div class="app-container" :class="{ 'narrow-layout': isNarrowLayout }">
    <!-- Sidebar (slides in as a drawer on narrow viewports) -->
    <aside
      v-show="showSidebar"
      class="sidebar"
      :class="{ 'is-drawer': isNarrowLayout }"
    >
      <div class="sidebar-header">
        <h1>ACP UI</h1>
        <div class="header-actions">
          <button
            class="settings-btn"
            :class="{ active: showLogStream }"
            @click="showLogStream = !showLogStream"
            title="Agent Log Stream"
          >📋</button>
          <button
            class="settings-btn"
            :class="{ active: showTrafficMonitor }"
            @click="showTrafficMonitor = !showTrafficMonitor"
            title="ACP Traffic Monitor"
          >📡</button>
          <button class="settings-btn" @click="showSettings = true" title="Settings">⚙</button>
          <button class="toggle-btn" @click="toggleSidebar">◀</button>
        </div>
      </div>
      
      <div class="sidebar-content">
        <!-- Agent Selection -->
        <div class="section">
          <AgentSelector 
            v-model:selected="selectedAgent"
            @select="handleAgentSelect"
          />
          
          <!-- Working Directory Picker -->
          <div class="cwd-picker">
            <label>Working Directory:</label>
            <!-- Desktop: read-only display + folder picker. -->
            <div v-if="folderPickerAvailable" class="cwd-row">
              <span class="cwd-path" :title="selectedCwd || 'Current directory'">
                {{ selectedCwd ? selectedCwd.split(/[\\/]/).pop() : '.' }}
              </span>
              <button 
                class="cwd-btn" 
                @click="handleSelectFolder"
                title="Select folder"
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
              @input="handleCwdInput"
              :disabled="isConnecting || isConnected"
              placeholder="/absolute/path/on/agent"
              autocapitalize="none"
              autocorrect="off"
              spellcheck="false"
            />
          </div>
          
          <button 
            v-if="hasAgents && !isConnected && !isConnecting"
            class="new-session-btn"
            :disabled="!selectedAgent || isLoading"
            @click="handleNewSession"
          >
            {{ isLoading ? 'Connecting...' : 'New Session' }}
          </button>
          
          <!-- Startup Progress -->
          <StartupProgress 
            v-if="isConnecting"
            :agent-name="selectedAgent"
            :phase="sessionStore.startupPhase"
            :logs="sessionStore.startupLogs"
            :elapsed-seconds="sessionStore.startupElapsed"
            :show-details="showStartupDetails"
            @cancel="handleCancelConnection"
            @toggle-details="showStartupDetails = !showStartupDetails"
          />
          
          <button 
            v-if="isConnected"
            class="disconnect-btn"
            @click="handleDisconnect"
          >
            Disconnect
          </button>
        </div>
        
        <!-- Session List -->
        <div class="section">
          <SessionList
            @resume="handleResumeSession"
            @delete="handleDeleteSession"
          />
        </div>

        <!-- View Navigation -->
        <div class="section view-nav">
          <h3 class="nav-title">功能导航</h3>
          <nav class="nav-buttons">
            <button
              v-for="feature in FEATURES"
              :key="feature.id"
              :class="['nav-btn', { active: currentView === feature.id }]"
              @click="navigateToFeature(feature.id)"
              :title="feature.description"
            >
              <span class="nav-icon">{{ feature.icon }}</span>
              <span class="nav-text">{{ feature.label }}</span>
            </button>
          </nav>
        </div>
      </div>
    </aside>
    
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
        <!-- Reconnect banner takes priority over the error banner: while a
             reconnect is in progress we don't want a contradictory red
             "Connection lost" pill. -->
        <div v-if="isReconnecting" class="reconnect-banner">
          <span class="reconnect-spinner" aria-hidden="true"></span>
          <span class="reconnect-text">
            Reconnecting to <strong>{{ reconnectingAgentName }}</strong>…
          </span>
        </div>

        <!-- Error display (suppressed while reconnecting). -->
        <div v-else-if="error" class="error-banner">
          <span class="error-icon">⚠</span>
          <span class="error-text">{{ error }}</span>
          <button
            v-if="canManuallyReconnect"
            class="error-action"
            @click="handleManualReconnect"
            title="Reconnect"
          >Reconnect</button>
          <button class="error-close" @click="clearError" title="Dismiss">×</button>
        </div>
        
        <!-- Chat View when connected (single agent) -->
        <ChatView v-if="isConnected && currentView === 'chat'" />

        <!-- Multi-Agent Chat View -->
        <MultiAgentChat v-else-if="currentView === 'multi-agent'" />

        <!-- Multi-Session View (总会话) -->
        <MultiSessionChat v-else-if="currentView === 'multi-session'" />

        <!-- Agent Status Panel -->
        <div v-else-if="currentView === 'status'" class="view-container">
          <h3>Agent 连接池</h3>
          <div class="status-stats">
            <div class="stat-card">
              <span class="stat-value">{{ teamRuntime.activeTaskCount }}</span>
              <span class="stat-label">运行中任务</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ teamRuntime.completedTaskCount }}</span>
              <span class="stat-label">已完成任务</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ teamRuntime.taskList.length }}</span>
              <span class="stat-label">总任务数</span>
            </div>
          </div>
          <div v-if="teamRuntime.taskList.length > 0" class="task-list">
            <div v-for="task in teamRuntime.taskList.slice(0, 10)" :key="task.id" class="task-item" :class="task.status">
              <span class="task-name">{{ task.title }}</span>
              <span class="task-source">{{ task.source }}</span>
              <span class="task-status">{{ task.status }}</span>
            </div>
          </div>
          <div v-else class="empty-state">
            <p>暂无任务</p>
          </div>
        </div>

        <!-- Realtime Monitor -->
        <div v-else-if="currentView === 'monitor'" class="view-container">
          <h3>实时事件</h3>
          <div v-if="teamRuntime.events.length > 0" class="event-list">
            <div v-for="event in teamRuntime.events.slice(0, 50)" :key="event.id" class="event-item">
              <span class="event-type">{{ event.type }}</span>
              <span class="event-message">{{ event.message }}</span>
              <span class="event-time">{{ new Date(event.timestamp).toLocaleTimeString() }}</span>
            </div>
          </div>
          <div v-else class="empty-state">
            <p>暂无事件</p>
          </div>
          <TrafficMonitor />
        </div>

        <!-- History View -->
        <HistoryView v-else-if="currentView === 'history'" />

        <!-- Workflow View -->
        <WorkflowView v-else-if="currentView === 'workflow'" />

        <!-- Team Orchestration View (实时可视化编排) -->
        <TeamOrchestrationView v-else-if="currentView === 'orchestration'" />

        <!-- Bot Settings View -->
        <div v-else-if="currentView === 'bot'" class="view-container">
          <h3>Bot 配置</h3>
          <div class="empty-state">
            <p>Bot 功能正在开发中</p>
            <p class="hint">当前 BotManager 仅支持事件发射，实际执行功能将在后续版本实现</p>
          </div>
        </div>

        <!-- Gateway Settings View -->
        <GatewaySettings v-else-if="currentView === 'gateway'" />

        <!-- Memory View -->
        <MemoryView v-else-if="currentView === 'memory'" />

        <!-- Error View -->
        <ErrorView v-else-if="currentView === 'error'" />

        <!-- Evolution View -->
        <EvolutionView v-else-if="currentView === 'evolution'" />

        <!-- Pattern View -->
        <PatternView v-else-if="currentView === 'pattern'" />

        <!-- Hermes Dashboard (Agent Progress Monitor) -->
        <HermesDashboard v-else-if="currentView === 'hermes'" />

        <!-- Collaboration Network View (Network Visualization) -->
        <EnhancedHermesDashboard v-else-if="currentView === 'collaboration'" />

        <!-- Agent Teams Platform Dashboard (Phase 2-4: 实时进度 + 类人宠物 + 三端同步) -->
        <AgentTeamsDashboard v-else-if="currentView === 'agent-teams'" />

        <!-- Task Graph View (DAG Visualization) -->
        <TaskGraphView v-else-if="currentView === 'task-graph'" :dag="mockTaskDag" :show-agents="true" orientation="vertical" />

        <!-- Welcome screen when not connected in chat view -->
        <div v-else-if="currentView === 'chat' && !isConnected" class="welcome-screen">
          <h2>Welcome to ACP UI</h2>
          <p>Select an agent and create a new session to get started.</p>
          <p v-if="!hasAgents" class="hint">
            Configure agents in your config file to begin.
          </p>
        </div>

        <!-- Default state for other views -->
        <div v-else class="welcome-screen">
          <h2>Agent Teams Platform</h2>
          <p>请先连接代理以使用此功能</p>
        </div>
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

.sidebar-content {
  flex: 1;
  overflow-y: auto;
}

.section {
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
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

.error-banner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #fee;
  color: #c00;
  border-bottom: 1px solid #fcc;
}

.error-icon {
  flex-shrink: 0;
}

.error-text {
  flex: 1;
}

.error-close {
  flex-shrink: 0;
  padding: 0.25rem 0.5rem;
  border: none;
  background: transparent;
  color: #c00;
  font-size: 1.25rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0.6;
  border-radius: 4px;
}

.error-close:hover {
  opacity: 1;
  background: rgba(204, 0, 0, 0.1);
}

/* Inline "Reconnect" affordance shown next to a stale error when we have a
   saved session we could reattach to. */
.error-action {
  flex-shrink: 0;
  padding: 0.25rem 0.6rem;
  margin-right: 0.25rem;
  border: 1px solid #c00;
  border-radius: 4px;
  background: transparent;
  color: #c00;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
}

.error-action:hover {
  background: rgba(204, 0, 0, 0.1);
}

/* Foreground-reconnect banner. Distinct visual style from the red error
   banner so users immediately read it as transient progress, not failure. */
.reconnect-banner {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.75rem 1rem;
  background: #e0f2fe;
  color: #0369a1;
  border-bottom: 1px solid #bae6fd;
}

.reconnect-text {
  flex: 1;
  font-size: 0.9rem;
}

.reconnect-text strong {
  font-weight: 600;
}

.reconnect-spinner {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: reconnect-spin 0.9s linear infinite;
}

@keyframes reconnect-spin {
  to { transform: rotate(360deg); }
}

@media (prefers-color-scheme: dark) {
  .reconnect-banner {
    background: #082f49;
    color: #7dd3fc;
    border-bottom-color: #0c4a6e;
  }
}

.welcome-screen {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
}

.welcome-screen h2 {
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}

.welcome-screen .hint {
  margin-top: 1rem;
  font-size: 0.875rem;
  color: var(--text-muted);
}

/* ---------- Inline View Containers (status, monitor, bot) ---------- */

.view-container {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.status-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
  margin: 16px 0;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--primary);
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
  font-size: 13px;
}

.task-item.running { border-left: 3px solid #3b82f6; }
.task-item.completed { border-left: 3px solid #22c55e; }
.task-item.failed { border-left: 3px solid #ef4444; }

.task-name { flex: 1; font-weight: 500; }
.task-source { color: var(--text-muted); font-size: 11px; }
.task-status {
  font-size: 11px;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--bg-subtle);
}

.event-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 400px;
  overflow-y: auto;
}

.event-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 10px;
  border-radius: 4px;
  font-size: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
}

.event-type {
  font-weight: 600;
  color: var(--primary);
  min-width: 100px;
  font-size: 11px;
}

.event-message { flex: 1; }
.event-time { color: var(--text-muted); min-width: 80px; text-align: right; }

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: 12px;
  margin-top: 4px;
}

/* ---------- Multi-Session View ---------- */

.multi-session-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.session-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ---------- Mobile / narrow-viewport layout ---------- */

.drawer-backdrop {
  display: none;
}

.mobile-hamburger {
  display: none;
}

@media (max-width: 800px) {
  .app-container {
    /* Prevent the off-screen drawer from causing horizontal scroll. */
    overflow-x: hidden;
  }

  /* Banners sit at the very top of the main area, where the OS status bar
     / camera notch overlap on phones. Extend the banner colour through the
     safe-area inset and push the text below it so the status bar reads as
     a tinted continuation of the banner instead of clipping its content. */
  .reconnect-banner,
  .error-banner {
    padding-top: calc(0.75rem + env(safe-area-inset-top, 0px));
  }

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

  /* Tap-target sizing for the icon buttons inside the sidebar header. */
  .settings-btn,
  .toggle-btn {
    min-width: 40px;
    min-height: 40px;
    font-size: 1rem;
  }

  /* Honour the iOS home indicator at the bottom of the main area. */
  .main-area {
    padding-bottom: env(safe-area-inset-bottom, 0px);
  }
}
</style>