export interface FeatureEntry {
  id: 'chat' | 'multi-agent' | 'multi-session' | 'status' | 'monitor' | 'history' | 'workflow' | 'gateway' | 'orchestration' | 'bot' | 'memory'
  label: string
  icon: string
  requiresAgent?: boolean
  description: string
}

export const FEATURES: FeatureEntry[] = [
  { id: 'chat', label: '对话', icon: '💬', requiresAgent: true, description: '单 Agent 会话' },
  { id: 'multi-agent', label: '多Agent', icon: '🤖', requiresAgent: true, description: '广播、路由和对比多个 Agent 输出' },
  { id: 'multi-session', label: '总会话', icon: '📋', requiresAgent: true, description: '并行管理多个 Agent 会话' },
  { id: 'workflow', label: '工作流', icon: '⚡', requiresAgent: true, description: '保存并执行多步骤任务' },
  { id: 'orchestration', label: '编排监控', icon: '🎬', requiresAgent: false, description: '查看执行计划、节点和输出' },
  { id: 'bot', label: 'Bot 配置', icon: '🤖', requiresAgent: false, description: '配置远程指令入口' },
  { id: 'gateway', label: '远程控制', icon: '🌐', requiresAgent: false, description: 'WebSocket App 控制通道' },
  { id: 'memory', label: '记忆', icon: '💡', requiresAgent: false, description: '管理任务上下文记忆' },
  { id: 'status', label: '状态', icon: '📊', requiresAgent: false, description: '查看 Agent 连接池' },
  { id: 'monitor', label: '监控', icon: '📡', requiresAgent: false, description: '查看实时事件' },
  { id: 'history', label: '历史', icon: '📚', requiresAgent: false, description: '查询任务历史' },
]
