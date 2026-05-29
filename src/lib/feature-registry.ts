export interface FeatureEntry {
  id: 'chat' | 'multi-agent' | 'multi-session' | 'status' | 'monitor' | 'history' | 'workflow' | 'gateway' | 'orchestration' | 'bot' | 'memory' | 'error' | 'evolution' | 'pattern' | 'hermes' | 'task-graph' | 'collaboration' | 'agent-teams' | 'executive-session' | 'skills'
  labelKey: string
  icon: string
  requiresAgent?: boolean
  descriptionKey: string
}

export const FEATURES: FeatureEntry[] = [
  { id: 'chat', labelKey: 'navigation.chat', icon: '💬', requiresAgent: true, descriptionKey: 'navigationDescriptions.chat' },
  { id: 'multi-agent', labelKey: 'navigation.multiAgent', icon: '🤖', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiAgent' },
  { id: 'executive-session', labelKey: 'navigation.executiveSession', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.executiveSession' },
  { id: 'multi-session', labelKey: 'navigation.multiSession', icon: '📋', requiresAgent: true, descriptionKey: 'navigationDescriptions.multiSession' },
  { id: 'workflow', labelKey: 'navigation.workflow', icon: '⚡', requiresAgent: true, descriptionKey: 'navigationDescriptions.workflow' },
  { id: 'orchestration', labelKey: 'navigation.orchestration', icon: '🎬', requiresAgent: false, descriptionKey: 'navigationDescriptions.orchestration' },
  { id: 'agent-teams', labelKey: 'navigation.agentTeams', icon: '🚀', requiresAgent: false, descriptionKey: 'navigationDescriptions.agentTeams' },
  { id: 'collaboration', labelKey: 'navigation.collaboration', icon: '🕸️', requiresAgent: false, descriptionKey: 'navigationDescriptions.collaboration' },
  { id: 'hermes', labelKey: 'navigation.hermes', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.hermes' },
  { id: 'task-graph', labelKey: 'navigation.taskGraph', icon: '🔗', requiresAgent: false, descriptionKey: 'navigationDescriptions.taskGraph' },
  { id: 'memory', labelKey: 'navigation.memory', icon: '💡', requiresAgent: false, descriptionKey: 'navigationDescriptions.memory' },
  { id: 'error', labelKey: 'navigation.errorMonitor', icon: '🚨', requiresAgent: false, descriptionKey: 'navigationDescriptions.errorMonitor' },
  { id: 'evolution', labelKey: 'navigation.evolution', icon: '🧬', requiresAgent: false, descriptionKey: 'navigationDescriptions.evolution' },
  { id: 'pattern', labelKey: 'navigation.pattern', icon: '📖', requiresAgent: false, descriptionKey: 'navigationDescriptions.pattern' },
  { id: 'bot', labelKey: 'navigation.botConfig', icon: '🤖', requiresAgent: false, descriptionKey: 'navigationDescriptions.botConfig' },
  { id: 'gateway', labelKey: 'navigation.gateway', icon: '🌐', requiresAgent: false, descriptionKey: 'navigationDescriptions.gateway' },
  { id: 'status', labelKey: 'navigation.status', icon: '📊', requiresAgent: false, descriptionKey: 'navigationDescriptions.status' },
  { id: 'monitor', labelKey: 'navigation.monitor', icon: '📡', requiresAgent: false, descriptionKey: 'navigationDescriptions.monitor' },
  { id: 'history', labelKey: 'navigation.history', icon: '📚', requiresAgent: false, descriptionKey: 'navigationDescriptions.history' },
  { id: 'skills', labelKey: 'navigation.skills', icon: '🧩', requiresAgent: false, descriptionKey: 'navigationDescriptions.skills' },
]
