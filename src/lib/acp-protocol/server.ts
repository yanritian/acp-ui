// ACP Server - WebSocket server for ACP protocol
// Allows other ACP clients to connect to ACP-UI
//
// @deprecated This module is an interface definition / scaffold.
// Most methods are scaffold implementations — real WebSocket server
// requires Tauri plugin support or a Node.js sidecar.
// TODO(Phase 2): Implement real server via tauri-plugin-websocket or sidecar.

import { ref, type Ref } from 'vue';
import {
  type ACPMessage,
  type ACPCapabilities,
  ACP_METHODS,
  ACP_ERROR_CODES,
  DEFAULT_ACP_CAPABILITIES,
  negotiateCapabilities,
} from './index';

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

  // Start WebSocket server (requires Tauri plugin or sidecar)
  // TODO(Phase 2): Implement real server startup via tauri-plugin-websocket
  async start(_port: number = 8765): Promise<void> {
    // Scaffold — server not yet implemented in current runtime.
    // Callers should check isConnected before sending messages.
    console.warn('[ACP Server] start() is a scaffold — real server not yet wired');
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
        // Convert to ACPMessage format (JSON-RPC 2.0)
        const msg: ACPMessage = {
          jsonrpc: '2.0',
          id: rawData.id,
          method: rawData.method,
          params: rawData.params,
          sessionId: rawData.sessionId || '',
          timestamp: rawData.timestamp || Date.now(),
          source: rawData.source || 'client',
          encryption: rawData.encryption,
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

      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { capabilities: negotiated },
        sessionId: '',
        timestamp: Date.now(),
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

      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { sessionId },
        sessionId: '',
        timestamp: Date.now(),
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

      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { sessionId, closed: true },
        sessionId: '',
        timestamp: Date.now(),
        source: 'server' as const,
      };
    });

    // Heartbeat
    this.messageHandlers.set(ACP_METHODS.HEARTBEAT, async (msg, _conn) => {
      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { timestamp: Date.now() },
        sessionId: '',
        timestamp: Date.now(),
        source: 'server' as const,
      };
    });

    // Agent list — proxy to orchestration store
    this.messageHandlers.set(ACP_METHODS.AGENT_LIST, async (msg, _conn) => {
      // TODO(Phase 2): Populate from orchestration store
      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { agents: [] },
        sessionId: '',
        timestamp: Date.now(),
        source: 'server' as const,
      };
    });

    // Task submit
    this.messageHandlers.set(ACP_METHODS.TASK_SUBMIT, async (msg, _conn) => {
      const params = msg.params as Record<string, unknown>;
      const sessionId = params?.sessionId as string;

      // TODO(Phase 2): Call orchestration API to execute task
      const taskId = crypto.randomUUID();

      return {
        jsonrpc: '2.0' as const,
        id: msg.id,
        result: { taskId, sessionId, status: 'pending' },
        sessionId: '',
        timestamp: Date.now(),
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