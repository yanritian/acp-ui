/**
 * Agent实时进度面板类型定义 (Phase 2)
 * Claude Code风格的实时状态可视化
 */

// ===== Agent活动状态 =====
export type AgentActivityType = 'thinking' | 'executing' | 'outputting' | 'waiting' | 'idle' | 'error'

// ===== Agent情绪状态 =====
export type AgentEmotionType = 'happy' | 'focused' | 'confused' | 'tired' | 'bored' | 'excited'

// ===== 思考块类型 =====
export interface ThinkingChunk {
  id: string
  content: string
  timestamp: number
  duration: number // 毫秒
  depth: number // 思考深度 1-5
}

// ===== 工具执行状态 =====
export type ToolExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

// ===== 工具执行信息 =====
export interface ToolExecution {
  id: string
  toolName: string
  parameters: Record<string, any>
  status: ToolExecutionStatus
  progress: number // 0-100
  startTime: number
  duration?: number
  result?: any
  error?: string
  preview?: string // 结果预览
}

// ===== 输出块类型 =====
export interface OutputChunk {
  id: string
  content: string
  timestamp: number
  type: 'text' | 'code' | 'file' | 'command' | 'result'
}

// ===== 权限等待类型 =====
export interface PermissionWaiting {
  type: 'tool_use' | 'file_write' | 'command_execute' | 'network_access'
  description: string
  options: PermissionOption[]
  waitingSince: number
  timeout?: number
}

// ===== 权限选项 =====
export interface PermissionOption {
  id: string
  label: string
  action: 'allow' | 'deny' | 'allow_always' | 'deny_always'
  recommended?: boolean
}

// ===== Agent实时状态 =====
export interface AgentRealtimeStatus {
  agentId: string
  agentName: string

  // 当前活动
  currentActivity: {
    type: AgentActivityType
    startTime: number
    duration: number
    progress: number // 0-100
    description?: string
  }

  // 思考过程
  thinking: {
    content: string
    startTime: number
    depth: number
    chunks: ThinkingChunk[]
    isStreaming: boolean
  }

  // 工具执行
  toolExecution: ToolExecution | null

  // 输出内容
  output: {
    content: string
    chunks: OutputChunk[]
    totalLength: number
    currentPosition: number
    isStreaming: boolean
  }

  // 权限等待
  permissionWaiting: PermissionWaiting | null

  // 统计数据
  stats: {
    tasksCompleted: number
    tasksFailed: number
    averageDuration: number
    successRate: number
    totalThinkingTime: number
    totalToolCalls: number
  }

  // 最后更新时间
  lastUpdateTime: number
}

// ===== Agent宠物模型 =====
export interface AgentPet {
  id: string
  agentId: string
  name: string
  avatarUrl?: string

  // 外观特征
  appearance: {
    type: 'cat' | 'dog' | 'robot' | 'dragon' | 'custom'
    color: string
    accessories: string[]
    customImageUrl?: string
  }

  // 情绪状态
  emotion: {
    current: AgentEmotionType
    intensity: number // 0-100
    lastChangeTime: number
    transitionDuration: number // 情绪切换动画时长
  }

  // 成长系统
  growth: {
    level: number
    experience: number
    experienceToNextLevel: number
    completedTasks: number
    successRate: number
    achievements: string[]
  }

  // 个性化
  personality: {
    name: string
    traits: string[] // ['careful', 'creative', 'fast', 'thorough']
    workStyle: 'methodical' | 'creative' | 'aggressive' | 'careful'
    preferences: string[]
    catchphrase?: string
  }

  // 互动状态
  interaction: {
    lastPetTime: number
    happinessLevel: number // 0-100
    affinityScore: number // 与用户的亲密度 0-100
    petCount: number
    lastInteractionType: 'pet' | 'poke' | 'feed' | 'play'
  }

  // 动画状态
  animation: {
    currentAnimation: 'idle' | 'blink' | 'think' | 'happy' | 'confused' | 'tired' | 'working' | 'levelup' | 'celebrate'
    animationStartTime: number
    animationDuration: number
    isAnimating: boolean
  }
}

// ===== 实时事件类型 =====
export type RealtimeEventType =
  | 'thinking_start'
  | 'thinking_chunk'
  | 'thinking_end'
  | 'tool_call_start'
  | 'tool_call_progress'
  | 'tool_call_result'
  | 'tool_call_error'
  | 'output_start'
  | 'output_chunk'
  | 'output_end'
  | 'permission_request'
  | 'permission_response'
  | 'status_change'
  | 'emotion_change'
  | 'level_up'
  | 'achievement_unlocked'
  | 'interaction'

// ===== 实时事件 =====
export interface RealtimeEvent {
  id: string
  type: RealtimeEventType
  agentId: string
  timestamp: number
  data: any
  severity: 'info' | 'warning' | 'error' | 'success' | 'critical'
}

// ===== Agent宠物动画配置 =====
export const AGENT_PET_ANIMATIONS = {
  idle: { frames: 1, duration: 0, expression: '😐', descriptionKey: 'petAnimations.idle' },
  blink: { frames: 2, duration: 200, expression: '😐', descriptionKey: 'petAnimations.blink' },
  think: { frames: 3, duration: 800, expression: '🤔', descriptionKey: 'petAnimations.think' },
  happy: { frames: 4, duration: 600, expression: '😊', descriptionKey: 'petAnimations.happy' },
  confused: { frames: 3, duration: 500, expression: '😕', descriptionKey: 'petAnimations.confused' },
  tired: { frames: 2, duration: 400, expression: '😫', descriptionKey: 'petAnimations.tired' },
  working: { frames: 3, duration: 1000, expression: '😐', descriptionKey: 'petAnimations.working' },
  levelup: { frames: 6, duration: 1500, expression: '🎉', descriptionKey: 'petAnimations.levelup' },
  celebrate: { frames: 5, duration: 800, expression: '✨', descriptionKey: 'petAnimations.celebrate' }
}

// ===== 活动状态指示器配置 =====
export const ACTIVITY_INDICATORS = {
  thinking: { icon: '💭', color: '#8B5CF6', animation: 'pulse', descriptionKey: 'activityIndicators.thinking' },
  executing: { icon: '⚡', color: '#F59E0B', animation: 'progress', descriptionKey: 'activityIndicators.executing' },
  outputting: { icon: '💬', color: '#10B981', animation: 'typewriter', descriptionKey: 'activityIndicators.outputting' },
  waiting: { icon: '⏸️', color: '#6B7280', animation: 'blink', descriptionKey: 'activityIndicators.waiting' },
  idle: { icon: '😴', color: '#9CA3AF', animation: 'none', descriptionKey: 'activityIndicators.idle' },
  error: { icon: '❌', color: '#EF4444', animation: 'shake', descriptionKey: 'activityIndicators.error' }
}

// ===== 情绪表情映射 =====
export const EMOTION_EXPRESSIONS = {
  happy: { emoji: '😊', color: '#10B981', animation: 'bounce' },
  focused: { emoji: '😐', color: '#3B82F6', animation: 'pulse' },
  confused: { emoji: '😕', color: '#F59E0B', animation: 'shake' },
  tired: { emoji: '😫', color: '#6B7280', animation: 'droop' },
  bored: { emoji: '🙄', color: '#9CA3AF', animation: 'look-around' },
  excited: { emoji: '🎉', color: '#8B5CF6', animation: 'jump' }
}

// ===== 成长等级配置 =====
export const LEVEL_CONFIG = {
  maxLevel: 100,
  baseExperience: 100,
  experienceMultiplier: 1.5,
  achievements: {
    first_task: { nameKey: 'achievements.firstTaskName', descriptionKey: 'achievements.firstTaskDesc', icon: '🌟', requirement: 1 },
    ten_tasks: { nameKey: 'achievements.tenTasksName', descriptionKey: 'achievements.tenTasksDesc', icon: '⭐', requirement: 10 },
    fifty_tasks: { nameKey: 'achievements.fiftyTasksName', descriptionKey: 'achievements.fiftyTasksDesc', icon: '🏅', requirement: 50 },
    hundred_tasks: { nameKey: 'achievements.hundredTasksName', descriptionKey: 'achievements.hundredTasksDesc', icon: '🏆', requirement: 100 },
    perfect_streak: { nameKey: 'achievements.perfectStreakName', descriptionKey: 'achievements.perfectStreakDesc', icon: '💎', requirement: 10 },
    fast_executor: { nameKey: 'achievements.fastExecutorName', descriptionKey: 'achievements.fastExecutorDesc', icon: '⚡', requirement: 30 },
    deep_thinker: { nameKey: 'achievements.deepThinkerName', descriptionKey: 'achievements.deepThinkerDesc', icon: '🧠', requirement: 300 }
  }
}