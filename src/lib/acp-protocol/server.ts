// ACP Server - WebSocket server for ACP protocol
// Allows other ACP clients to connect to ACP-UI

import { ref, type Ref } from 'vue';
import {
  type ACPMessage as ACPMessageSpec,
  type ACPCapabilities,
  type ACPError,
  ACP_METHODS,
  ACP_ERROR_CODES,
  DEFAULT_ACP_CAPABILITIES,
  negotiateCapabilities,
} from './spec';
import { createACPMessage, encryptMessage, decryptMessage, type ACPMessage } from './index';

// Re-export for consistency
export { type ACPMessage } from './index';

export interface ACPSession {
  id: string;
  clientId: string;
  capabilities: ACPCapabilities;
  createdAt: number;
  lastActivity: number;
  status: 'active' | 'paused' | 'closed';
}

export interface ACPConnection {
  websocket: WebSocket;
  clientId: string;
  capabilities?: ACPCapabilities;
  sessions: Map<string, ACPSession>;
}

export class ACPServer {
  private websocket: WebSocket | null = null;
  private connections = new Map<string, ACPConnection>();
  private messageHandlers = new Map<string, (msg: ACPMessage, conn: ACPConnection) => Promise<ACPMessage>>();
  private encryptionKey: Uint8Array | null = null;

  // State refs for UI binding
  public isConnected: Ref<boolean> = ref(false);
  public activeConnections: Ref<number> = ref(0);
  public activeSessions: Ref<number> = ref(0);

  constructor() {
    this.registerDefaultHandlers();
  }

  // Start WebSocket server (in Tauri, this uses tauri-plugin-websocket)
  async start(port: number = 8765): Promise<void> {
    // In browser mode, we can't start a server
    // In Tauri mode, we would use tauri-plugin-websocket
    // For now, this is a placeholder that simulates server behavior
    console.log(`[ACP Server] Would start on port ${port} (requires Tauri)`);
    this.isConnected.value = true;
  }

  // Stop server
  async stop(): Promise<void> {
    for (const [clientId, conn] of this.connections) {
      conn.websocket.close();
    }
    this.connections.clear();
    this.isConnected.value = false;
    this.activeConnections.value = 0;
    this.activeSessions.value = 0;
  }

  // Handle incoming connection
  private async handleConnection(ws: WebSocket, clientId: string): Promise<void> {
    const conn: ACPConnection = {
      websocket: ws,
      clientId,
      sessions: new Map(),
    };

    this.connections.set(clientId, conn);
    this.activeConnections.value = this.connections.size;

    ws.onmessage = async (event) => {
      try {
        const rawData = JSON.parse(event.data as string);
        // Convert to ACPMessage format
        const msg: ACPMessage = {
          jsonrpc: '2.0',
          id: rawData.id,
          method: rawData.method,
          params: rawData.params,
          sessionId: rawData.sessionId || '',
          timestamp: rawData.timestamp || Date.now(),
          source: rawData.source || 'client',
          version: '1.0',
          messageId: rawData.messageId || crypto.randomUUID(),
          type: rawData.type || 'task_submit',
          payload: rawData.payload || rawData.params || {},
        };
        await this.handleMessage(msg, conn);
      } catch (e) {
        console.error('[ACP Server] Message parse error:', e);
        this.sendError(ws, ACP_ERROR_CODES.PARSE_ERROR, 'Parse error', undefined);
      }
    };

    ws.onclose = () => {
      this.connections.delete(clientId);
      this.activeConnections.value = this.connections.size;
      this.activeSessions.value = this.countSessions();
    };
  }

  // Handle incoming message
  private async handleMessage(msg: ACPMessage, conn: ACPConnection): Promise<void> {
    const handler = this.messageHandlers.get(msg.method || '');

    if (!handler) {
      this.sendError(conn.websocket, ACP_ERROR_CODES.METHOD_NOT_FOUND, 'Method not found', msg.id);
      return;
    }

    try {
      const response = await handler(msg, conn);
      this.send(conn.websocket, response);
    } catch (e) {
      const error = e as Error;
      this.sendError(conn.websocket, ACP_ERROR_CODES.INTERNAL_ERROR, error.message, msg.id);
    }
  }

  // Register default method handlers
  private registerDefaultHandlers(): void {
    // Capability negotiation
    this.messageHandlers.set(ACP_METHODS.CAPABILITY_NEGOTIATE, async (msg, conn) => {
      const clientCaps = (msg.params as Record<string, unknown>)?.capabilities as ACPCapabilities;
      const negotiated = negotiateCapabilities(clientCaps, DEFAULT_ACP_CAPABILITIES);
      conn.capabilities = negotiated;

      const baseMsg = createACPMessage('task_submit', {
        capabilities: negotiated,
        success: true,
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });

    // Session create
    this.messageHandlers.set(ACP_METHODS.SESSION_CREATE, async (msg, conn) => {
      const sessionId = crypto.randomUUID();
      const session: ACPSession = {
        id: sessionId,
        clientId: conn.clientId,
        capabilities: conn.capabilities || DEFAULT_ACP_CAPABILITIES,
        createdAt: Date.now(),
        lastActivity: Date.now(),
        status: 'active',
      };

      conn.sessions.set(sessionId, session);
      this.activeSessions.value = this.countSessions();

      const baseMsg = createACPMessage('task_submit', {
        sessionId,
        success: true,
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });

    // Session close
    this.messageHandlers.set(ACP_METHODS.SESSION_CLOSE, async (msg, conn) => {
      const sessionId = (msg.params as Record<string, unknown>)?.sessionId as string;
      const session = conn.sessions.get(sessionId);

      if (!session) {
        throw new Error('Session not found');
      }

      session.status = 'closed';
      conn.sessions.delete(sessionId);
      this.activeSessions.value = this.countSessions();

      const baseMsg = createACPMessage('task_submit', {
        sessionId,
        success: true,
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });

    // Heartbeat
    this.messageHandlers.set(ACP_METHODS.HEARTBEAT, async (msg, conn) => {
      const baseMsg = createACPMessage('task_submit', {
        timestamp: Date.now(),
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });

    // Agent list - proxy to orchestration store
    this.messageHandlers.set(ACP_METHODS.AGENT_LIST, async (msg, conn) => {
      // This would call the orchestration API
      const baseMsg = createACPMessage('task_submit', {
        agents: [], // Placeholder - would be populated from orchestration store
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });

    // Task submit
    this.messageHandlers.set(ACP_METHODS.TASK_SUBMIT, async (msg, conn) => {
      const params = msg.params as Record<string, unknown>;
      const sessionId = params?.sessionId as string;
      const taskDescription = params?.description as string;

      // This would call the orchestration API to execute task
      const taskId = crypto.randomUUID();

      const baseMsg = createACPMessage('task_submit', {
        taskId,
        sessionId,
        status: 'pending',
        success: true,
      });
      return {
        ...baseMsg,
        jsonrpc: '2.0' as const,
        sessionId: '',
        source: 'server' as const,
      };
    });
  }

  // Send message
  private send(ws: WebSocket, msg: ACPMessage): void {
    ws.send(JSON.stringify(msg));
  }

  // Send error
  private sendError(ws: WebSocket, code: number, message: string, id: string | number | undefined): void {
    const errorMsg: ACPMessage = {
      jsonrpc: '2.0',
      id,
      error: { code, message },
      sessionId: '',
      timestamp: Date.now(),
      source: 'server',
      version: '1.0',
      messageId: crypto.randomUUID(),
      type: 'task_error',
      payload: { error: { code, message } },
    };
    this.send(ws, errorMsg);
  }

  // Count total sessions
  private countSessions(): number {
    let count = 0;
    for (const conn of this.connections.values()) {
      count += conn.sessions.size;
    }
    return count;
  }

  // Set encryption key
  setEncryptionKey(key: Uint8Array): void {
    this.encryptionKey = key;
  }

  // Register custom handler
  registerHandler(method: string, handler: (msg: ACPMessage, conn: ACPConnection) => Promise<ACPMessage>): void {
    this.messageHandlers.set(method, handler);
  }
}

// Singleton instance
let acpServerInstance: ACPServer | null = null;

export function useACPServer(): ACPServer {
  if (!acpServerInstance) {
    acpServerInstance = new ACPServer();
  }
  return acpServerInstance;
}