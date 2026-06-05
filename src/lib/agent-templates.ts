/**
 * Agent Templates - Pre-configured agent templates for quick setup
 */

export interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  command: string;
  args: string[];
  requiredEnv: string[];
  icon: string;
  tags: string[];
  transport?: 'stdio' | 'websocket';
  url?: string;
}

export const AGENT_TEMPLATES: AgentTemplate[] = [
  {
    id: 'claude-code',
    name: 'Claude Code',
    description: 'Anthropic 官方 AI 编程助手，支持多语言开发、代码审查、重构等',
    command: 'npx',
    args: ['@anthropic-ai/claude-code'],
    requiredEnv: ['ANTHROPIC_API_KEY'],
    icon: '🤖',
    tags: ['推荐', '官方', 'AI编程'],
    transport: 'stdio'
  },
  {
    id: 'copilot',
    name: 'GitHub Copilot',
    description: 'GitHub 官方 AI 编程助手，支持代码补全、代码生成、代码解释',
    command: 'npx',
    args: ['@github/copilot-language-server', '--acp'],
    requiredEnv: [],
    icon: '🟢',
    tags: ['推荐', '免费', 'AI编程'],
    transport: 'stdio'
  },
  {
    id: 'gemini-cli',
    name: 'Gemini CLI',
    description: 'Google Gemini AI 命令行工具，支持对话、代码生成、文档分析',
    command: 'gemini',
    args: [],
    requiredEnv: ['GOOGLE_API_KEY'],
    icon: '💎',
    tags: ['Google', 'AI编程'],
    transport: 'stdio'
  },
  {
    id: 'cursor-agent',
    name: 'Cursor Agent',
    description: 'Cursor AI 编程助手，强大的代码补全和重构功能',
    command: 'cursor-agent',
    args: ['--acp'],
    requiredEnv: [],
    icon: '✨',
    tags: ['AI编程', '代码补全'],
    transport: 'stdio'
  },
  {
    id: 'mcp-server',
    name: 'MCP Server (Generic)',
    description: '通用的 MCP 服务器模板，适用于各种自定义 MCP 服务',
    command: 'npx',
    args: ['-y', 'mcp-server'],
    requiredEnv: [],
    icon: '🔌',
    tags: ['MCP', '通用'],
    transport: 'stdio'
  },
  {
    id: 'mcp-filesystem',
    name: 'MCP Filesystem',
    description: 'MCP 文件系统服务器，提供安全的文件读写能力',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-filesystem', '/path/to/allowed/dir'],
    requiredEnv: [],
    icon: '📁',
    tags: ['MCP', '文件系统'],
    transport: 'stdio'
  },
  {
    id: 'mcp-github',
    name: 'MCP GitHub',
    description: 'MCP GitHub 服务器，提供 GitHub API 集成能力',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-github'],
    requiredEnv: ['GITHUB_TOKEN'],
    icon: '🐙',
    tags: ['MCP', 'GitHub'],
    transport: 'stdio'
  },
  {
    id: 'mcp-postgres',
    name: 'MCP PostgreSQL',
    description: 'MCP PostgreSQL 服务器，提供数据库查询能力',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-postgres'],
    requiredEnv: ['DATABASE_URL'],
    icon: '🐘',
    tags: ['MCP', '数据库'],
    transport: 'stdio'
  },
  {
    id: 'mcp-memory',
    name: 'MCP Memory',
    description: 'MCP 记忆服务器，提供持久化记忆存储能力',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-memory'],
    requiredEnv: [],
    icon: '🧠',
    tags: ['MCP', '记忆'],
    transport: 'stdio'
  },
  {
    id: 'websocket-agent',
    name: 'WebSocket Agent',
    description: 'WebSocket 连接的 Agent 模板，适用于远程部署的 AI 服务',
    command: '',
    args: [],
    requiredEnv: [],
    icon: '🌐',
    tags: ['WebSocket', '远程'],
    transport: 'websocket',
    url: 'ws://localhost:8080/ws'
  },
  {
    id: 'openai-compatible',
    name: 'OpenAI Compatible',
    description: '兼容 OpenAI API 格式的自定义 Agent 服务',
    command: '',
    args: [],
    requiredEnv: ['OPENAI_API_KEY'],
    icon: '🔥',
    tags: ['OpenAI', '兼容'],
    transport: 'websocket',
    url: 'ws://localhost:8000/v1/agent'
  },
  {
    id: 'deepseek-agent',
    name: 'DeepSeek Agent',
    description: 'DeepSeek AI 编程助手，高性价比的代码生成服务',
    command: 'deepseek-cli',
    args: ['--acp'],
    requiredEnv: ['DEEPSEEK_API_KEY'],
    icon: '🌊',
    tags: ['AI编程', '国产'],
    transport: 'stdio'
  }
];

/**
 * Get template by ID
 */
export function getTemplateById(id: string): AgentTemplate | undefined {
  return AGENT_TEMPLATES.find(t => t.id === id);
}

/**
 * Filter templates by tag
 */
export function getTemplatesByTag(tag: string): AgentTemplate[] {
  return AGENT_TEMPLATES.filter(t => t.tags.includes(tag));
}

/**
 * Get all unique tags from templates
 */
export function getAllTags(): string[] {
  const tagSet = new Set<string>();
  AGENT_TEMPLATES.forEach(t => t.tags.forEach(tag => tagSet.add(tag)));
  return Array.from(tagSet);
}

/**
 * Check if template is compatible with current platform
 */
export function isTemplateCompatible(template: AgentTemplate, isDesktop: boolean): boolean {
  // WebSocket templates work on all platforms
  if (template.transport === 'websocket') {
    return true;
  }
  // stdio templates only work on desktop
  return isDesktop;
}