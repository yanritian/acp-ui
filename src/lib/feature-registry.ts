export interface FeatureEntry {
  id: 'chat' | 'multi-agent' | 'multi-session' | 'status' | 'monitor' | 'history' | 'workflow' | 'gateway' | 'orchestration' | 'bot' | 'memory' | 'error' | 'evolution' | 'pattern' | 'hermes' | 'task-graph' | 'collaboration' | 'agent-teams' | 'executive-session' | 'skills' | 'agent-config' | 'plugins' | 'swarm-dashboard' | 'workflow-editor' | 'token-optimizer'
  labelKey: string
  icon: string
  requiresAgent?: boolean
  descriptionKey: string
  isCore?: boolean
}

export const FEATURES: FeatureEntry[] = [
  // Core features (shown in main navigation)
  { id: 'chat', labelKey: 'navigation.chat', icon: '💬', requiresAgent: true, descriptionKey: 'navigationDescriptions.chat', isCore: true },
  { id: 'multi-agent', labelKey: 'navigation.multiAgent', icon: '🤖', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiAgent', isCore: true },
  { id: 'collaboration', labelKey: 'navigation.collaboration', icon: '🕸️', requiresAgent: false, descriptionKey: 'navigationDescriptions.collaboration', isCore: true },
  { id: 'bot', labelKey: 'navigation.botConfig', icon: '🤖', requiresAgent: false, descriptionKey: 'navigationDescriptions.botConfig', isCore: true },
  { id: 'agent-config', labelKey: 'navigation.agentConfig', icon: '⚙️', requiresAgent: false, descriptionKey: 'navigationDescriptions.agentConfig', isCore: true },

  // Advanced features (shown in "More" menu)
  { id: 'executive-session', labelKey: 'navigation.executiveSession', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.executiveSession' },
  { id: 'multi-session', labelKey: 'navigation.multiSession', icon: '📋', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiSession' },
  { id: 'workflow', labelKey: 'navigation.workflow', icon: '⚡', requiresAgent: true, descriptionKey: 'navigationDescriptions.workflow' },
  { id: 'orchestration', labelKey: 'navigation.orchestration', icon: '🎬', requiresAgent: false, descriptionKey: 'navigationDescriptions.orchestration' },
  { id: 'agent-teams', labelKey: 'navigation.agentTeams', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.agentTeams' },
  { id: 'hermes', labelKey: 'navigation.hermes', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.hermes' },
  { id: 'task-graph', labelKey: 'navigation.taskGraph', icon: '🔗', requiresAgent: false, descriptionKey: 'navigationDescriptions.taskGraph' },
  { id: 'memory', labelKey: 'navigation.memory', icon: '💡', requiresAgent: false, descriptionKey: 'navigationDescriptions.memory' },
  { id: 'error', labelKey: 'navigation.errorMonitor', icon: '🚨', requiresAgent: false, descriptionKey: 'navigationDescriptions.errorMonitor' },
  { id: 'evolution', labelKey: 'navigation.evolution', icon: '🧬', requiresAgent: false, descriptionKey: 'navigationDescriptions.evolution' },
  { id: 'pattern', labelKey: 'navigation.pattern', icon: '📖', requiresAgent: false, descriptionKey: 'navigationDescriptions.pattern' },
  { id: 'gateway', labelKey: 'navigation.gateway', icon: '🌐', requiresAgent: false, descriptionKey: 'navigationDescriptions.gateway' },
  { id: 'status', labelKey: 'navigation.status', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.status' },
  { id: 'monitor', labelKey: 'navigation.monitor', icon: '📡', requiresAgent: false, descriptionKey: 'navigationDescriptions.monitor' },
  { id: 'history', labelKey: 'navigation.history', icon: '📚', requiresAgent: false, descriptionKey: 'navigationDescriptions.history' },
  { id: 'skills', labelKey: 'navigation.skills', icon: '🧩', requiresAgent: false, descriptionKey: 'navigationDescriptions.skills' },
  { id: 'plugins', labelKey: 'navigation.plugins', icon: '🔌', requiresAgent: false, descriptionKey: 'navigationDescriptions.plugins' },
  { id: 'swarm-dashboard', labelKey: 'navigation.swarmDashboard', icon: '🐝', requiresAgent: false, descriptionKey: 'navigationDescriptions.swarmDashboard' },
  { id: 'workflow-editor', labelKey: 'navigation.workflowEditor', icon: '🔧', requiresAgent: false, descriptionKey: 'navigationDescriptions.workflowEditor' },
  { id: 'token-optimizer', labelKey: 'navigation.tokenOptimizer', icon: '🪙', requiresAgent: false, descriptionKey: 'navigationDescriptions.tokenOptimizer' },
]

export const CORE_FEATURES = FEATURES.filter(f => f.isCore)
export const ADVANCED_FEATURES = FEATURES.filter(f => !f.isCore)
