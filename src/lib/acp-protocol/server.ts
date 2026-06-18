// ACP Server - WebSocket client for ACP protocol
// Connects to the Rust backend WebSocketServer (port 1421)
//
// NOTE: This module provides frontend WebSocket client functionality.
// The `start()` method is a scaffold — frontend should NOT start a server,
// instead use `connectToBackend()` to connect to the Rust backend.
// The Rust backend auto-starts WebSocketServer in src-tauri/src/lib.rs.

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

  /**
   * NOTE: Frontend does NOT start a WebSocket server.
   * The Rust backend (src-tauri/src/lib.rs) auto-starts WebSocketServer on port 1421.
   * Use connectToBackend() instead to connect as a client.
   *
   * This method is kept for API compatibility but does nothing.
   */
  async start(_port: number = 8765): Promise<void> {
    console.warn('[ACP Server] start() is deprecated — frontend should use connectToBackend() to connect to Rust backend')
  }

  /**
   * Connect to the backend Rust WebSocketServer (port 1421 by default).
   * This allows the frontend to receive events pushed from the backend.
   *
   * The Rust WebSocketServer is auto-started in src-tauri/src/lib.rs.
   */
  async connectToBackend(port: number = 1421): Promise<void> {
    const wsUrl = `ws://127.0.0.1:${port}`

    try {
      this.websocket = new WebSocket(wsUrl)

      this.websocket.onopen = () => {
        this.isConnected.value = true
        console.log(`[ACP Server] Connected to backend WebSocket at ${wsUrl}`)
      }

      this.websocket.onmessage = async (event) => {
        try {
          const rawData = JSON.parse(event.data as string)
          // Handle backend events - these are pushed from Rust WebSocketServer
          const msg: ACPMessage = {
            jsonrpc: '2.0',
            id: rawData.id,
            method: rawData.method,
            params: rawData.params,
            sessionId: rawData.sessionId || '',
            timestamp: rawData.timestamp || Date.now(),
            source: rawData.source || 'server',
            encryption: rawData.encryption,
          }

          // If it's a request from backend, handle it
          if (msg.method) {
            // Create a pseudo-connection for handling
            const pseudoConn: ACPConnection = {
              websocket: this.websocket!,
              clientId: 'backend',
              sessions: new Map(),
            }
            await this.handleMessage(msg, pseudoConn)
          }
        } catch (e) {
          console.error('[ACP Server] Backend message parse error:', e)
        }
      }

      this.websocket.onclose = () => {
        this.isConnected.value = false
        console.log('[ACP Server] Disconnected from backend WebSocket')
      }

      this.websocket.onerror = (error) => {
        console.error('[ACP Server] Backend WebSocket error:', error)
        this.isConnected.value = false
      }

      // Wait for connection to establish
      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('Connection timeout')), 5000)
        this.websocket!.onopen = () => {
          clearTimeout(timeout)
          resolve()
        }
        this.websocket!.onerror = (err) => {
          clearTimeout(timeout)
          reject(err)
        }
      })

    } catch (error) {
      console.error('[ACP Server] Failed to connect to backend:', error)
      this.isConnected.value = false
      throw error
    }
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

    // Agent list — proxy to swarm store
    this.messageHandlers.set(ACP_METHODS.AGENT_LIST, async (msg, _conn) => {
      // Import swarm-api dynamically to avoid circular deps
      const { swarmListWorkers, swarmGetWorkerStatus } = await import('@/lib/swarm-api');

      try {
        const workers = await swarmListWorkers();
        const agents = await Promise.all(
          workers.map(async (w) => {
            try {
              const status = await swarmGetWorkerStatus(w.workerId);
              return {
                id: w.workerId,
                type: w.workerType,
                capabilities: w.capabilities,
                health: status.health,
                tasksCompleted: status.tasksCompleted,
                currentTask: status.currentTask,
              };
            } catch {
              return {
                id: w.workerId,
                type: w.workerType,
                capabilities: w.capabilities,
                health: 'offline',
                tasksCompleted: 0,
                currentTask: null,
              };
            }
          })
        );

        return {
          jsonrpc: '2.0' as const,
          id: msg.id,
          result: { agents },
          sessionId: '',
          timestamp: Date.now(),
          source: 'server' as const,
        };
      } catch (e) {
        throw new Error(`Failed to list agents: ${e}`);
      }
    });

    // Task submit — execute via swarm
    this.messageHandlers.set(ACP_METHODS.TASK_SUBMIT, async (msg, _conn) => {
      const { swarmSendTask, swarmRegisterWorker } = await import('@/lib/swarm-api');
      const params = msg.params as Record<string, unknown>;

      try {
        // Get available worker or create one
        const workerType = ((params?.workerType as string) || 'claude_code') as 'codex' | 'claude_code';
        const workerId = (params?.workerId as string) || `worker-${Date.now()}`;
        const taskId = crypto.randomUUID();

        // Register worker if not already
        await swarmRegisterWorker(workerType, workerId);

        // Send task to worker
        const taskHandle = await swarmSendTask(
          workerId,
          taskId,
          (params?.prompt as string) || '',
          params?.workingDir as string | undefined,
          (params?.timeoutMs as number) || 60000
        );

        return {
          jsonrpc: '2.0' as const,
          id: msg.id,
          result: {
            taskId: taskHandle.taskId,
            workerId: taskHandle.workerId,
            status: taskHandle.status,
          },
          sessionId: '',
          timestamp: Date.now(),
          source: 'server' as const,
        };
      } catch (e) {
        throw new Error(`Task submit failed: ${e}`);
      }
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