// Agent类型定义
export type AgentType = 'planner' | 'architect' | 'tdd-guide' | 'code-reviewer' | 'security-reviewer' | 'general-purpose' | 'explore' | 'build-error-resolver' | 'go-build-resolver' | 'go-reviewer' | 'python-reviewer' | 'refactor-cleaner' | 'doc-updater' | 'e2e-runner' | 'database-reviewer' | 'statusline-setup' | 'planner' | 'init-architect' | 'get-current-datetime' | 'chief-of-staff' | 'claude-code-guide' | 'architect' | 'Plan'

/**
 * 协作网络节点 - 代表一个Agent
 */
export interface CollaborationNode {
  id: string
  agentId: string
  agentName: string
  agentType: AgentType
  position: { x: number; y: number }
  status: 'idle' | 'active' | 'waiting' | 'error'
  capabilities: AgentCapability[]
  currentLoad: number
  maxLoad: number
  description?: string
  avatarUrl?: string
}

/**
 * 协作边 - 代表任务流转或消息传递
 */
export interface CollaborationEdge {
  id: string
  sourceAgentId: string
  targetAgentId: string
  taskId: string
  taskDescription: string
  status: 'pending' | 'flowing' | 'completed' | 'failed'
  timestamp: number
  animationProgress: number // 0-1 流动动画进度
  messageType?: 'task_assign' | 'task_transfer' | 'request' | 'response' | 'notification'
  payloadPreview?: string
  duration?: number
}

/**
 * Agent能力定义
 */
export interface AgentCapability {
  id: string
  name: string
  description: string
  icon: string
  category: 'planning' | 'execution' | 'review' | 'testing' | 'communication' | 'orchestration'
  prerequisites?: string[]
  outputs?: string[]
  proficiency?: number // 0-100 能力熟练度
  examples?: string[]
}

/**
 * 协作协议定义
 */
export interface CollaborationProtocol {
  id: string
  name: string
  version: string
  participants: string[] // Agent IDs
  inputContract: ContractDefinition
  outputContract: ContractDefinition
  executionConditions: Condition[]
  constraints: Constraint[]
  examples: ProtocolExample[]
  metadata?: {
    author?: string
    createdAt?: number
    updatedAt?: number
    tags?: string[]
  }
}

/**
 * 契约定义 - 输入输出约定
 */
export interface ContractDefinition {
  type: 'data' | 'message' | 'task' | 'resource'
  schema: Record<string, unknown>
  required: boolean
  description?: string
  validationRules?: ValidationRule[]
}

/**
 * 执行条件
 */
export interface Condition {
  id: string
  type: 'precondition' | 'postcondition' | 'invariant'
  expression: string
  description: string
  priority?: number
}

/**
 * 约束规则
 */
export interface Constraint {
  id: string
  type: 'timeout' | 'resource_limit' | 'quality_threshold' | 'dependency'
  value: number | string
  unit?: string
  description: string
  enforcement: 'strict' | 'flexible'
}

/**
 * 协议示例
 */
export interface ProtocolExample {
  id: string
  name: string
  scenario: string
  input: Record<string, unknown>
  output: Record<string, unknown>
  notes?: string[]
}

/**
 * 验证规则
 */
export interface ValidationRule {
  id: string
  type: 'type_check' | 'range_check' | 'pattern_match' | 'custom'
  rule: string
  errorMessage: string
}

/**
 * 协作事件 - 所有协作活动的详细记录
 */
export interface CollaborationEvent {
  id: string
  type:
    | 'task_assign'
    | 'task_transfer'
    | 'task_complete'
    | 'task_fail'
    | 'message_sent'
    | 'message_received'
    | 'tool_call'
    | 'tool_result'
    | 'status_change'
    | 'protocol_invoked'
    | 'capability_match'
    | 'load_balance'
  timestamp: number
  sourceAgentId: string
  targetAgentId?: string
  taskId?: string
  sessionId?: string
  protocolId?: string
  capabilityId?: string
  payload: unknown
  details: EventDetails
  severity: 'info' | 'warning' | 'error' | 'critical'
}

/**
 * 事件详情
 */
export interface EventDetails {
  summary: string
  description?: string
  duration?: number
  success?: boolean
  error?: string
  metrics?: EventMetrics
  attachments?: EventAttachment[]
}

/**
 * 事件指标
 */
export interface EventMetrics {
  executionTime?: number
  resourceUsage?: number
  messageCount?: number
  toolCallCount?: number
  retryCount?: number
}

/**
 * 事件附件
 */
export interface EventAttachment {
  id: string
  type: 'code' | 'file' | 'image' | 'log' | 'data'
  name: string
  content?: string
  url?: string
  size?: number
}

/**
 * 时间线视图数据
 */
export interface TimelineData {
  startTime: number
  endTime: number
  agents: TimelineAgent[]
  events: CollaborationEvent[]
  milestones: TimelineMilestone[]
}

/**
 * 时间线Agent活动条
 */
export interface TimelineAgent {
  agentId: string
  agentName: string
  activities: TimelineActivity[]
  color: string
}

/**
 * 时间线活动段
 */
export interface TimelineActivity {
  id: string
  startTime: number
  endTime: number
  type: 'task' | 'waiting' | 'idle' | 'error'
  taskId?: string
  description: string
}

/**
 * 时间线里程碑
 */
export interface TimelineMilestone {
  id: string
  timestamp: number
  name: string
  description?: string
  type: 'start' | 'checkpoint' | 'completion' | 'error'
  relatedTaskIds?: string[]
}

/**
 * 看板视图数据
 */
export interface KanbanData {
  columns: KanbanColumn[]
  agents: KanbanAgent[]
}

/**
 * 看板列
 */
export interface KanbanColumn {
  id: string
  name: string
  status: 'pending' | 'in_progress' | 'reviewing' | 'testing' | 'completed' | 'failed'
  tasks: KanbanTask[]
  color: string
}

/**
 * 看板任务卡片
 */
export interface KanbanTask {
  id: string
  taskId: string
  title: string
  description?: string
  assignedAgentId?: string
  priority: 'low' | 'medium' | 'high' | 'critical'
  progress: number
  tags: string[]
  createdAt: number
  updatedAt: number
  estimatedTime?: number
  actualTime?: number
}

/**
 * 看板Agent列
 */
export interface KanbanAgent {
  agentId: string
  agentName: string
  tasks: KanbanTask[]
  capacity: number
  currentLoad: number
}

/**
 * 协作网络配置
 */
export interface CollaborationNetworkConfig {
  layoutAlgorithm: 'force-directed' | 'hierarchical' | 'circular' | 'grid'
  animationEnabled: boolean
  animationSpeed: number
  nodeSize: 'small' | 'medium' | 'large'
  edgeStyle: 'straight' | 'curved' | 'orthogonal'
  showCapabilities: boolean
  showProtocols: boolean
  showMetrics: boolean
  autoRefresh: boolean
  refreshInterval: number
}

/**
 * 协作网络统计
 */
export interface CollaborationNetworkStats {
  totalAgents: number
  activeAgents: number
  totalTasks: number
  runningTasks: number
  completedTasks: number
  failedTasks: number
  averageTaskDuration: number
  collaborationEfficiency: number
  messageCount: number
  toolCallCount: number
  protocolInvocations: number
}