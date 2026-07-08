export interface FeatureEntry {
  id: string
  labelKey: string
  icon: string
  requiresAgent?: boolean
  descriptionKey: string
  isCore?: boolean
  group?: 'chat' | 'agent' | 'workflow' | 'monitor' | 'lab'  // New: navigation group
  experimental?: boolean  // New: mark experimental features
}

export const FEATURE_GROUPS = {
  chat: { labelKey: 'navigationGroup.chat', icon: '📁', order: 1 },
  agent: { labelKey: 'navigationGroup.agent', icon: '🤖', order: 2 },
  workflow: { labelKey: 'navigationGroup.workflow', icon: '⚡', order: 3 },
  monitor: { labelKey: 'navigationGroup.monitor', icon: '📊', order: 4 },
  lab: { labelKey: 'navigationGroup.lab', icon: '🧪', order: 5 },
}

export const FEATURES: FeatureEntry[] = [
  // === Game Operator (Primary Entry) ===
  { id: 'games', labelKey: 'navigation.games', icon: '🎮', requiresAgent: false, descriptionKey: 'navigationDescriptions.games', group: 'workflow', isCore: true },

  // === Dashboard (Home) ===
  { id: 'dashboard', labelKey: 'navigation.dashboard', icon: '🏠', requiresAgent: false, descriptionKey: 'navigationDescriptions.dashboard', group: 'chat', isCore: true },

  // === Chat Group ===
  { id: 'chat', labelKey: 'navigation.chat', icon: '💬', requiresAgent: true, descriptionKey: 'navigationDescriptions.chat', group: 'chat', isCore: true },
  { id: 'multi-agent', labelKey: 'navigation.multiAgent', icon: '🤖', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiAgent', group: 'chat', isCore: true },
  { id: 'multi-session', labelKey: 'navigation.multiSession', icon: '📋', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiSession', group: 'chat' },
  { id: 'collaboration', labelKey: 'navigation.collaboration', icon: '🕸️', requiresAgent: false, descriptionKey: 'navigationDescriptions.collaboration', group: 'chat' },

  // === Agent Group ===
  { id: 'agent-config', labelKey: 'navigation.agentConfig', icon: '⚙️', requiresAgent: false, descriptionKey: 'navigationDescriptions.agentConfig', group: 'agent', isCore: true },
  { id: 'gateway', labelKey: 'navigation.gateway', icon: '🌐', requiresAgent: false, descriptionKey: 'navigationDescriptions.gateway', group: 'agent', isCore: true },
  { id: 'bot', labelKey: 'navigation.botConfig', icon: '🤖', requiresAgent: false, descriptionKey: 'navigationDescriptions.botConfig', group: 'agent', isCore: true },

  // === Workflow Group (moved to experiments) ===
  { id: 'workflow', labelKey: 'navigation.workflow', icon: '⚡', requiresAgent: true, descriptionKey: 'navigationDescriptions.workflow', group: 'lab' },
  { id: 'orchestration', labelKey: 'navigation.orchestration', icon: '🎬', requiresAgent: false, descriptionKey: 'navigationDescriptions.orchestration', group: 'lab' },
  { id: 'workflow-editor', labelKey: 'navigation.workflowEditor', icon: '🔧', requiresAgent: false, descriptionKey: 'navigationDescriptions.workflowEditor', group: 'lab' },
  { id: 'task-graph', labelKey: 'navigation.taskGraph', icon: '🔗', requiresAgent: false, descriptionKey: 'navigationDescriptions.taskGraph', group: 'lab' },
  { id: 'agent-teams', labelKey: 'navigation.agentTeams', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.agentTeams', group: 'lab' },
  { id: 'executive-session', labelKey: 'navigation.executiveSession', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.executiveSession', group: 'lab' },

  // === Monitor Group (moved to experiments) ===
  { id: 'status', labelKey: 'navigation.status', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.status', group: 'lab' },
  { id: 'monitor', labelKey: 'navigation.monitor', icon: '📡', requiresAgent: false, descriptionKey: 'navigationDescriptions.monitor', group: 'lab' },
  { id: 'history', labelKey: 'navigation.history', icon: '📚', requiresAgent: false, descriptionKey: 'navigationDescriptions.history', group: 'lab' },
  { id: 'hermes', labelKey: 'navigation.hermes', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.hermes', group: 'lab' },
  { id: 'token-optimizer', labelKey: 'navigation.tokenOptimizer', icon: '🪙', requiresAgent: false, descriptionKey: 'navigationDescriptions.tokenOptimizer', group: 'lab' },

  // === Lab Group (Experimental) ===
  { id: 'memory', labelKey: 'navigation.memory', icon: '💡', requiresAgent: false, descriptionKey: 'navigationDescriptions.memory', group: 'lab', experimental: true },
  { id: 'evolution', labelKey: 'navigation.evolution', icon: '🧬', requiresAgent: false, descriptionKey: 'navigationDescriptions.evolution', group: 'lab', experimental: true },
  { id: 'pattern', labelKey: 'navigation.pattern', icon: '📖', requiresAgent: false, descriptionKey: 'navigationDescriptions.pattern', group: 'lab', experimental: true },
  { id: 'error', labelKey: 'navigation.errorMonitor', icon: '🚨', requiresAgent: false, descriptionKey: 'navigationDescriptions.errorMonitor', group: 'lab', experimental: true },
  { id: 'skills', labelKey: 'navigation.skills', icon: '🧩', requiresAgent: false, descriptionKey: 'navigationDescriptions.skills', group: 'lab' },
  { id: 'plugins', labelKey: 'navigation.plugins', icon: '🔌', requiresAgent: false, descriptionKey: 'navigationDescriptions.plugins', group: 'lab' },
  { id: 'swarm-dashboard', labelKey: 'navigation.swarmDashboard', icon: '🐝', requiresAgent: false, descriptionKey: 'navigationDescriptions.swarmDashboard', group: 'lab' },
  { id: 'games-renpy', labelKey: 'navigation.gamesRenpy', icon: '🎭', requiresAgent: false, descriptionKey: 'navigationDescriptions.gamesRenpy', group: 'lab' },
]

export const CORE_FEATURES = FEATURES.filter(f => f.isCore)
export const ADVANCED_FEATURES = FEATURES.filter(f => !f.isCore)

// Grouped features for sidebar rendering
export const GROUPED_FEATURES = Object.entries(FEATURE_GROUPS)
  .sort((a, b) => a[1].order - b[1].order)
  .map(([groupKey, groupMeta]) => ({
    key: groupKey as keyof typeof FEATURE_GROUPS,
    ...groupMeta,
    features: FEATURES.filter(f => f.group === groupKey),
  }))
