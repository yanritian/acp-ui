import { createRouter, createWebHashHistory } from 'vue-router'
import { createMockTaskDag } from './lib/mock-task-dag'

// Use lazy loading for code splitting
const routes = [
  { path: '/', redirect: '/chat' },
  { path: '/chat', name: 'chat', component: () => import('./features/chat/ChatView.vue') },
  { path: '/multi-agent', name: 'multi-agent', component: () => import('./features/chat/MultiAgentChat.vue') },
  { path: '/multi-session', name: 'multi-session', component: () => import('./features/chat/MultiSessionChat.vue') },
  { path: '/status', name: 'status', component: () => import('./features/monitoring/StatusView.vue') },
  { path: '/monitor', name: 'monitor', component: () => import('./features/monitoring/MonitorView.vue') },
  { path: '/history', name: 'history', component: () => import('./shared/ui/HistoryView.vue') },
  { path: '/workflow', name: 'workflow', component: () => import('./features/workflow/WorkflowView.vue') },
  { path: '/gateway', name: 'gateway', component: () => import('./shared/ui/GatewaySettings.vue') },
  { path: '/orchestration', name: 'orchestration', component: () => import('./features/workflow/TeamOrchestrationView.vue') },
  { path: '/bot', name: 'bot', component: () => import('./shared/ui/BotSettings.vue') },
  { path: '/memory', name: 'memory', component: () => import('./features/intelligence/MemoryView.vue') },
  { path: '/error', name: 'error', component: () => import('./shared/ui/ErrorView.vue') },
  { path: '/evolution', name: 'evolution', component: () => import('./features/intelligence/EvolutionView.vue') },
  { path: '/pattern', name: 'pattern', component: () => import('./features/intelligence/PatternView.vue') },
  { path: '/hermes', name: 'hermes', component: () => import('./features/hermes/HermesDashboard.vue') },
  {
    path: '/task-graph',
    name: 'task-graph',
    component: () => import('./features/workflow/TaskGraphView.vue'),
    props: () => ({ dag: createMockTaskDag(), showAgents: true, orientation: 'vertical', isDemo: true })
  },
  { path: '/collaboration', name: 'collaboration', component: () => import('./features/hermes/EnhancedHermesDashboard.vue') },
  { path: '/agent-teams', name: 'agent-teams', component: () => import('./views/AgentTeamsDashboard.vue') },
  { path: '/executive-session', name: 'executive-session', component: () => import('./features/agent-teams/ExecutiveSessionView.vue') },
  { path: '/skills', name: 'skills', component: () => import('./features/plugins/skills/SkillManager.vue') },
  { path: '/agent-config', name: 'agent-config', component: () => import('./views/AgentConfigView.vue') },
  { path: '/plugins', name: 'plugins', component: () => import('./features/plugins/PluginManagerView.vue') },
  { path: '/swarm-dashboard', name: 'swarm-dashboard', component: () => import('./features/plugins/SwarmDashboard.vue') },
  { path: '/workflow-editor', name: 'workflow-editor', component: () => import('./features/workflow/WorkflowEditor.vue') },
  { path: '/token-optimizer', name: 'token-optimizer', component: () => import('./features/plugins/TokenOptimizerPanel.vue') },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})