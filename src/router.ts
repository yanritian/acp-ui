import { createRouter, createWebHashHistory } from 'vue-router'
import { createMockTaskDag } from './lib/mock-task-dag'

// Use lazy loading for code splitting
const routes = [
  { path: '/', redirect: '/dashboard' },
  { path: '/dashboard', name: 'dashboard', component: () => import('./views/DashboardView.vue') },
  // Agent Platform - One-Shot Interface
  { path: '/agent-platform', name: 'agent-platform', component: () => import('./views/AgentPlatformView.vue') },
  { path: '/cost-tracker', name: 'cost-tracker', component: () => import('./views/CostDashboardView.vue') },
  { path: '/chat', name: 'chat', component: () => import('./features/chat/ChatView.vue') },
  { path: '/multi-agent', name: 'multi-agent', component: () => import('./features/chat/MultiAgentChat.vue') },
  { path: '/multi-session', name: 'multi-session', component: () => import('./features/chat/MultiSessionChat.vue') },
  // Demo pages - placeholder data, not connected to real backend
  { path: '/status', name: 'status', component: () => import('./features/monitoring/StatusView.vue'), meta: { isDemo: true } },
  { path: '/monitor', name: 'monitor', component: () => import('./features/monitoring/MonitorView.vue'), meta: { isDemo: true } },
  { path: '/history', name: 'history', component: () => import('./shared/ui/HistoryView.vue') },
  { path: '/workflow', name: 'workflow', component: () => import('./features/workflow/WorkflowView.vue') },
  { path: '/gateway', name: 'gateway', component: () => import('./shared/ui/GatewaySettings.vue') },
  { path: '/orchestration', name: 'orchestration', component: () => import('./features/workflow/TeamOrchestrationView.vue') },
  { path: '/bot', name: 'bot', component: () => import('./shared/ui/BotSettings.vue') },
  { path: '/memory', name: 'memory', component: () => import('./features/intelligence/MemoryView.vue') },
  { path: '/error', name: 'error', component: () => import('./shared/ui/ErrorView.vue') },
  { path: '/evolution', name: 'evolution', component: () => import('./features/intelligence/EvolutionView.vue') },
  { path: '/pattern', name: 'pattern', component: () => import('./features/intelligence/PatternView.vue') },
  // Demo page - mock Hermes dashboard
  { path: '/hermes', name: 'hermes', component: () => import('./features/hermes/HermesDashboard.vue'), meta: { isDemo: true } },
  {
    path: '/task-graph',
    name: 'task-graph',
    component: () => import('./features/workflow/TaskGraphView.vue'),
    props: () => ({ dag: createMockTaskDag(), showAgents: true, orientation: 'vertical', isDemo: true }),
    meta: { isDemo: true }
  },
  // Demo page - mock collaboration dashboard
  { path: '/collaboration', name: 'collaboration', component: () => import('./features/hermes/EnhancedHermesDashboard.vue'), meta: { isDemo: true } },
  { path: '/agent-teams', name: 'agent-teams', component: () => import('./views/AgentTeamsDashboard.vue') },
  { path: '/executive-session', name: 'executive-session', component: () => import('./features/agent-teams/ExecutiveSessionView.vue') },
  { path: '/skills', name: 'skills', component: () => import('./features/plugins/skills/SkillManager.vue') },
  { path: '/agent-config', name: 'agent-config', component: () => import('./views/AgentConfigView.vue') },
  { path: '/plugins', name: 'plugins', component: () => import('./features/plugins/PluginManagerView.vue') },
  { path: '/swarm', name: 'swarm', component: () => import('./features/swarm/SwarmDashboard.vue') },
  { path: '/goal-graph', name: 'goal-graph', component: () => import('./features/swarm/GoalGraphView.vue') },
  { path: '/worker-health', name: 'worker-health', component: () => import('./features/swarm/WorkerHealthPanel.vue') },
  { path: '/workflow-editor', name: 'workflow-editor', component: () => import('./features/workflow/WorkflowEditor.vue') },
  // Demo page - token optimizer backend commands not implemented
  { path: '/token-optimizer', name: 'token-optimizer', component: () => import('./features/plugins/TokenOptimizerPanel.vue'), meta: { isDemo: true } },
  { path: '/games', name: 'games', component: () => import('./features/games/GameManager.vue') },
  { path: '/games/renpy', name: 'games-renpy', component: () => import('./features/games/RenPyManager.vue') },
  { path: '/games/designer', name: 'games-designer', component: () => import('./features/games/GameDesigner.vue') },
  { path: '/games/developer', name: 'games-developer', component: () => import('./features/games/GameDeveloper.vue') },
  { path: '/settings', name: 'settings', component: () => import('./shared/ui/UnifiedSettingsView.vue') },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})