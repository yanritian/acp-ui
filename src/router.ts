import { createRouter, createWebHashHistory } from 'vue-router'
import { createMockTaskDag } from './lib/mock-task-dag'

// Use lazy loading for code splitting
const routes = [
  { path: '/', redirect: '/chat' },
  { path: '/chat', name: 'chat', component: () => import('./components/ChatView.vue') },
  { path: '/multi-agent', name: 'multi-agent', component: () => import('./components/MultiAgentChat.vue') },
  { path: '/multi-session', name: 'multi-session', component: () => import('./components/MultiSessionChat.vue') },
  { path: '/status', name: 'status', component: () => import('./components/StatusView.vue') },
  { path: '/monitor', name: 'monitor', component: () => import('./components/MonitorView.vue') },
  { path: '/history', name: 'history', component: () => import('./components/HistoryView.vue') },
  { path: '/workflow', name: 'workflow', component: () => import('./components/WorkflowView.vue') },
  { path: '/gateway', name: 'gateway', component: () => import('./components/GatewaySettings.vue') },
  { path: '/orchestration', name: 'orchestration', component: () => import('./components/TeamOrchestrationView.vue') },
  { path: '/bot', name: 'bot', component: () => import('./components/BotSettings.vue') },
  { path: '/memory', name: 'memory', component: () => import('./components/MemoryView.vue') },
  { path: '/error', name: 'error', component: () => import('./components/ErrorView.vue') },
  { path: '/evolution', name: 'evolution', component: () => import('./components/EvolutionView.vue') },
  { path: '/pattern', name: 'pattern', component: () => import('./components/PatternView.vue') },
  { path: '/hermes', name: 'hermes', component: () => import('./components/HermesDashboard.vue') },
  { 
    path: '/task-graph', 
    name: 'task-graph', 
    component: () => import('./components/TaskGraphView.vue'),
    props: () => ({ dag: createMockTaskDag(), showAgents: true, orientation: 'vertical' })
  },
  { path: '/collaboration', name: 'collaboration', component: () => import('./components/EnhancedHermesDashboard.vue') },
  { path: '/agent-teams', name: 'agent-teams', component: () => import('./views/AgentTeamsDashboard.vue') },
  { path: '/executive-session', name: 'executive-session', component: () => import('./components/ExecutiveSessionView.vue') },
  { path: '/skills', name: 'skills', component: () => import('./components/skills/SkillManager.vue') },
  { path: '/agent-config', name: 'agent-config', component: () => import('./views/AgentConfigView.vue') },
  { path: '/plugins', name: 'plugins', component: () => import('./components/PluginManagerView.vue') },
  { path: '/swarm-dashboard', name: 'swarm-dashboard', component: () => import('./components/SwarmDashboard.vue') },
  { path: '/workflow-editor', name: 'workflow-editor', component: () => import('./components/WorkflowEditor.vue') },
  { path: '/token-optimizer', name: 'token-optimizer', component: () => import('./components/TokenOptimizerPanel.vue') },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})
