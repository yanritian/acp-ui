/**
 * Intelligent Memory Extraction Engine
 *
 * Automatically extracts valuable memories from conversations
 */

import type { MemoryType, MemoryScope } from '../stores/memory'

export interface ExtractedMemory {
  content: string
  type: MemoryType
  importance: number
  tags: string[]
  scope: MemoryScope
}

// Keywords that indicate important information worth remembering
const FACT_KEYWORDS = [
  '技术栈', '架构', '框架', '语言', '版本', '配置', '环境',
  'API', '接口', '协议', '格式', '规范', '约定',
  '重要', '关键', '核心', '基础', '必需',
]

const DECISION_KEYWORDS = [
  '决定', '选择', '采用', '使用', '方案', '策略',
  '因为', '由于', '考虑到', '基于', '权衡',
  '最终', '确定', '确认', '选定',
]

const ERROR_KEYWORDS = [
  '错误', '失败', '异常', '崩溃', '问题', 'bug',
  '报错', '出错', '无法', '不支持', '冲突',
]

const SOLUTION_KEYWORDS = [
  '解决', '修复', '改正', '补丁', '更新', '升级',
  '通过', '使用', '采用', '实施', '应用',
  '成功', '完成', '正常', '正常工作',
]

const PATTERN_KEYWORDS = [
  '模式', '规律', '习惯', '惯例', '最佳实践',
  '总是', '通常', '一般', '常见', '典型',
]

const PREFERENCE_KEYWORDS = [
  '喜欢', '偏好', '习惯', '更倾向于', '更喜欢',
  '建议', '推荐', '希望', '期望', '想要',
]

/**
 * Analyze message content and extract potential memories
 */
export function analyzeMessage(content: string, context: {
  agentId?: string
  sessionId?: string
  taskId?: string
}): ExtractedMemory[] {
  const memories: ExtractedMemory[] = []
  const lines = content.split('\n').filter(l => l.trim().length > 20)

  for (const line of lines) {
    const extracted = extractFromLine(line, context)
    if (extracted) {
      memories.push(extracted)
    }
  }

  return memories
}

function extractFromLine(line: string, context: {
  agentId?: string
  sessionId?: string
  taskId?: string
}): ExtractedMemory | null {
  const lowerLine = line.toLowerCase()

  // Determine memory type based on keywords
  let type: MemoryType | null = null
  let importance = 0.5

  if (FACT_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'fact'
    importance = 0.7
  } else if (DECISION_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'decision'
    importance = 0.6
  } else if (ERROR_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'error'
    importance = 0.4
  } else if (SOLUTION_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'solution'
    importance = 0.5
  } else if (PATTERN_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'pattern'
    importance = 0.6
  } else if (PREFERENCE_KEYWORDS.some(k => lowerLine.includes(k.toLowerCase()))) {
    type = 'preference'
    importance = 0.5
  }

  if (!type) return null

  // Extract tags from the line
  const tags = extractTags(line)

  // Determine scope based on context
  let scope: MemoryScope = 'global'
  if (context.taskId) {
    scope = 'task'
  } else if (context.sessionId) {
    scope = 'session'
  } else if (context.agentId) {
    scope = 'agent'
  }

  return {
    content: line.trim(),
    type,
    importance,
    tags,
    scope,
  }
}

function extractTags(content: string): string[] {
  const tags: string[] = []

  // Extract tech keywords
  const techKeywords = [
    'React', 'Vue', 'TypeScript', 'JavaScript', 'Python', 'Go', 'Rust',
    'Node', 'Webpack', 'Vite', 'Docker', 'Kubernetes', 'AWS', 'GCP',
    'API', 'REST', 'GraphQL', 'WebSocket', 'HTTP', 'SQL', 'NoSQL',
    'Git', 'CI/CD', 'TDD', 'BDD', 'MVP', 'MVC',
  ]

  for (const keyword of techKeywords) {
    if (content.includes(keyword)) {
      tags.push(keyword)
    }
  }

  // Extract domain keywords
  const domainKeywords = [
    '前端', '后端', '全栈', '架构', '安全', '性能', '测试',
    '部署', '运维', '数据', '算法', '网络', '系统', '设计',
  ]

  for (const keyword of domainKeywords) {
    if (content.includes(keyword)) {
      tags.push(keyword)
    }
  }

  return tags.slice(0, 5) // Limit to 5 tags
}

/**
 * Calculate importance score based on multiple factors
 */
export function calculateImportance(factors: {
  accessCount: number
  ageInDays: number
  type: MemoryType
}): number {
  const { accessCount, ageInDays, type } = factors

  // Base importance by type
  const typeWeights: Record<MemoryType, number> = {
    fact: 0.8,
    decision: 0.7,
    pattern: 0.6,
    preference: 0.5,
    solution: 0.4,
    error: 0.3,
  }

  let base = typeWeights[type] ?? 0.5

  // Access frequency bonus (more accessed = more important)
  const accessBonus = Math.min(0.2, accessCount * 0.02)

  // Age decay (older memories lose importance, except facts)
  let ageDecay = 0
  if (type !== 'fact' && ageInDays > 30) {
    ageDecay = Math.min(0.3, (ageInDays - 30) * 0.01)
  }

  return Math.max(0.1, Math.min(1.0, base + accessBonus - ageDecay))
}

/**
 * Batch analyze messages and return top candidates
 */
export function batchExtract(messages: string[], context: {
  agentId?: string
  sessionId?: string
  taskId?: string
}): ExtractedMemory[] {
  const allExtracted: ExtractedMemory[] = []

  for (const msg of messages) {
    const extracted = analyzeMessage(msg, context)
    allExtracted.push(...extracted)
  }

  // Deduplicate by content similarity
  const unique = deduplicateMemories(allExtracted)

  // Sort by importance and return top candidates
  return unique.sort((a, b) => b.importance - a.importance).slice(0, 10)
}

function deduplicateMemories(memories: ExtractedMemory[]): ExtractedMemory[] {
  const seen = new Set<string>()
  const unique: ExtractedMemory[] = []

  for (const m of memories) {
    // Simple dedup by content hash (first 50 chars)
    const hash = m.content.slice(0, 50).toLowerCase()
    if (!seen.has(hash)) {
      seen.add(hash)
      unique.push(m)
    }
  }

  return unique
}