// ACP Protocol Specification v1.0
// Extends JSON-RPC 2.0 with Agent Control Protocol features

export interface ACPCapabilities {
  version: string;                    // 协议版本，如 "1.0.0"
  features: ACPFeature[];             // 支持的特性列表
  maxSessions: number;                // 最大并发会话数
  supportsStreaming: boolean;         // 是否支持流式传输
  supportsMultiplexing: boolean;      // 是否支持多路复用
}

export interface ACPFeature {
  id: string;                         // 如 "skill.evolution", "mcp.discovery"
  version: string;
  required: boolean;                  // 连接双方都必须支持
}

// ACP Message（扩展 JSON-RPC 2.0）
export interface ACPMessage {
  jsonrpc: '2.0';
  id?: string | number;
  method?: string;                    // 请求方法
  params?: Record<string, unknown>;
  result?: unknown;
  error?: ACPError;
  // ACP 扩展字段
  sessionId: string;
  timestamp: number;
  source: 'client' | 'server' | 'agent';
  encryption?: {
    algorithm: 'aes-256-gcm';
    keyId: string;
    iv: string;
  };
}

export interface ACPError {
  code: number;
  message: string;
  data?: unknown;
}

// Standard ACP Methods
export const ACP_METHODS = {
  // Session management
  SESSION_CREATE: 'session.create',
  SESSION_RESUME: 'session.resume',
  SESSION_CLOSE: 'session.close',
  SESSION_LIST: 'session.list',

  // Agent management
  AGENT_REGISTER: 'agent.register',
  AGENT_DEREGISTER: 'agent.deregister',
  AGENT_LIST: 'agent.list',
  AGENT_STATUS: 'agent.status',

  // Task execution
  TASK_SUBMIT: 'task.submit',
  TASK_PROGRESS: 'task.progress',
  TASK_COMPLETE: 'task.complete',
  TASK_CANCEL: 'task.cancel',

  // Approval flow
  APPROVAL_REQUEST: 'approval.request',
  APPROVAL_RESPONSE: 'approval.response',

  // Sync events
  SYNC_EVENT: 'sync.event',
  SYNC_FLUSH: 'sync.flush',

  // Capability negotiation
  CAPABILITY_NEGOTIATE: 'capability.negotiate',
  CAPABILITY_QUERY: 'capability.query',

  // Heartbeat
  HEARTBEAT: 'heartbeat',
} as const;

// Standard ACP Error Codes
export const ACP_ERROR_CODES = {
  // JSON-RPC standard errors
  PARSE_ERROR: -32700,
  INVALID_REQUEST: -32600,
  METHOD_NOT_FOUND: -32601,
  INVALID_PARAMS: -32602,
  INTERNAL_ERROR: -32603,

  // ACP specific errors
  SESSION_NOT_FOUND: -32001,
  AGENT_NOT_FOUND: -32002,
  TASK_NOT_FOUND: -32003,
  CAPABILITY_NOT_SUPPORTED: -32004,
  ENCRYPTION_ERROR: -32005,
  SESSION_LIMIT_EXCEEDED: -32006,
  PERMISSION_DENIED: -32007,
} as const;

// Default capabilities for ACP-UI
export const DEFAULT_ACP_CAPABILITIES: ACPCapabilities = {
  version: '1.0.0',
  features: [
    { id: 'session.management', version: '1.0', required: true },
    { id: 'agent.stdio', version: '1.0', required: false },
    { id: 'agent.websocket', version: '1.0', required: false },
    { id: 'task.execution', version: '1.0', required: true },
    { id: 'approval.flow', version: '1.0', required: false },
    { id: 'sync.engine', version: '1.0', required: false },
    { id: 'skill.evolution', version: '1.0', required: false },
    { id: 'mcp.discovery', version: '1.0', required: false },
  ],
  maxSessions: 10,
  supportsStreaming: true,
  supportsMultiplexing: true,
};

// Capability negotiation helper
export function negotiateCapabilities(
  clientCaps: ACPCapabilities,
  serverCaps: ACPCapabilities
): ACPCapabilities {
  const negotiatedFeatures: ACPFeature[] = [];

  // Check required features
  for (const clientFeature of clientCaps.features) {
    if (clientFeature.required) {
      const serverFeature = serverCaps.features.find(f => f.id === clientFeature.id);
      if (!serverFeature) {
        throw new Error(`Required capability not supported: ${clientFeature.id}`);
      }
      negotiatedFeatures.push(clientFeature);
    }
  }

  // Add optional features that both support
  for (const clientFeature of clientCaps.features) {
    if (!clientFeature.required) {
      const serverFeature = serverCaps.features.find(f => f.id === clientFeature.id);
      if (serverFeature) {
        negotiatedFeatures.push(clientFeature);
      }
    }
  }

  return {
    version: Math.min(
      parseFloat(clientCaps.version),
      parseFloat(serverCaps.version)
    ).toFixed(1) + '.0',
    features: negotiatedFeatures,
    maxSessions: Math.min(clientCaps.maxSessions, serverCaps.maxSessions),
    supportsStreaming: clientCaps.supportsStreaming && serverCaps.supportsStreaming,
    supportsMultiplexing: clientCaps.supportsMultiplexing && serverCaps.supportsMultiplexing,
  };
}