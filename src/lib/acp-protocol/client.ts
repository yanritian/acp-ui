// ACP Client - WebSocket client for connecting to ACP servers
// Also handles multiplexing and backpressure

import { ref, type Ref } from 'vue';
import {
  type ACPMessage as ACPMessageSpec,
  type ACPCapabilities,
  ACP_METHODS,
  ACP_ERROR_CODES,
  DEFAULT_ACP_CAPABILITIES,
  negotiateCapabilities,
} from './spec';
import { createACPMessage, encryptMessage, decryptMessage, type ACPMessage } from './index';

// Use index.ts ACPMessage as the primary type
type ClientACPMessage = ACPMessage;

export interface ACPSessionHandle {
  id: string;
  status: 'active' | 'paused' | 'closed';
  pendingMessages: ACPMessage[];
  lastActivity: number;
}

export class ACPClient {
  private websocket: WebSocket | null = null;
  private serverCapabilities: ACPCapabilities | null = null;
  private sessions = new Map<string, ACPSessionHandle>();
  private pendingRequests = new Map<string | number, { resolve: Function; reject: Function }>();
  private messageQueue: ACPMessage[] = [];
  private backpressureThreshold = 100; // Max queued messages before pause
  private encryptionKey: Uint8Array | null = null;

  // State refs for UI binding
  public isConnected: Ref<boolean> = ref(false);
  public serverUrl: Ref<string> = ref('');
  public activeSessions: Ref<number> = ref(0);
  public queuedMessages: Ref<number> = ref(0);

  // Connect to ACP server
  async connect(url: string): Promise<void> {
    this.serverUrl.value = url;

    this.websocket = new WebSocket(url);

    this.websocket.onopen = async () => {
      this.isConnected.value = true;
      // Perform capability negotiation
      await this.negotiateCapabilities();
    };

    this.websocket.onmessage = async (event) => {
      try {
        const rawData = JSON.parse(event.data as string);
        // Convert to ACPMessage format
        let msg: ACPMessage = {
          jsonrpc: '2.0',
          id: rawData.id,
          method: rawData.method,
          params: rawData.params,
          result: rawData.result,
          error: rawData.error,
          sessionId: rawData.sessionId || '',
          timestamp: rawData.timestamp || Date.now(),
          source: rawData.source || 'server',
          version: '1.0',
          messageId: rawData.messageId || crypto.randomUUID(),
          type: rawData.type || 'task_submit',
          payload: rawData.payload || rawData.params || {},
          encryption: rawData.encryption,
        };

        // Decrypt if needed
        if (msg.encryption && this.encryptionKey) {
          msg = await decryptMessage(msg, this.encryptionKey);
        }

        this.handleResponse(msg);
      } catch (e) {
        console.error('[ACP Client] Message parse error:', e);
      }
    };

    this.websocket.onclose = () => {
      this.isConnected.value = false;
      this.sessions.clear();
      this.activeSessions.value = 0;
    };

    this.websocket.onerror = (error) => {
      console.error('[ACP Client] WebSocket error:', error);
    };
  }

  // Disconnect
  async disconnect(): Promise<void> {
    // Close all sessions
    for (const [sessionId] of this.sessions) {
      await this.closeSession(sessionId);
    }

    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }

    this.isConnected.value = false;
  }

  // Capability negotiation
  private async negotiateCapabilities(): Promise<ACPCapabilities> {
    const response = await this.sendRequest(ACP_METHODS.CAPABILITY_NEGOTIATE, {
      capabilities: DEFAULT_ACP_CAPABILITIES,
    });

    const result = response.result as Record<string, unknown> | undefined;
    this.serverCapabilities = result?.capabilities as ACPCapabilities;
    return this.serverCapabilities!;
  }

  // Create session
  async createSession(): Promise<string> {
    const response = await this.sendRequest(ACP_METHODS.SESSION_CREATE, {});
    const result = response.result as Record<string, unknown> | undefined;
    const sessionId = result?.sessionId as string;

    this.sessions.set(sessionId, {
      id: sessionId,
      status: 'active',
      pendingMessages: [],
      lastActivity: Date.now(),
    });

    this.activeSessions.value = this.sessions.size;
    return sessionId;
  }

  // Close session
  async closeSession(sessionId: string): Promise<void> {
    await this.sendRequest(ACP_METHODS.SESSION_CLOSE, { sessionId });

    const session = this.sessions.get(sessionId);
    if (session) {
      session.status = 'closed';
      this.sessions.delete(sessionId);
    }

    this.activeSessions.value = this.sessions.size;
  }

  // Submit task
  async submitTask(sessionId: string, description: string): Promise<string> {
    const response = await this.sendRequest(ACP_METHODS.TASK_SUBMIT, {
      sessionId,
      description,
    });

    const result = response.result as Record<string, unknown> | undefined;
    return result?.taskId as string;
  }

  // Send request with multiplexing support
  private async sendRequest(method: string, params: Record<string, unknown>): Promise<ACPMessage> {
    const id = crypto.randomUUID();

    const msg: ACPMessage = {
      jsonrpc: '2.0',
      id,
      method,
      params,
      sessionId: this.sessions.keys().next().value || '',
      timestamp: Date.now(),
      source: 'client',
      version: '1.0',
      messageId: crypto.randomUUID(),
      type: 'task_submit',
      payload: params,
    };

    // Encrypt if needed
    let finalMsg = msg;
    if (this.encryptionKey && this.serverCapabilities?.features.some(f => f.id === 'encryption')) {
      finalMsg = await encryptMessage(msg, this.encryptionKey);
    }

    // Check backpressure
    if (this.messageQueue.length >= this.backpressureThreshold) {
      console.warn('[ACP Client] Backpressure threshold reached, waiting...');
      await this.flushQueue();
    }

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      this.websocket?.send(JSON.stringify(finalMsg));
    });
  }

  // Handle response
  private handleResponse(msg: ACPMessage): void {
    // Handle pending request
    if (msg.id && this.pendingRequests.has(msg.id)) {
      const { resolve, reject } = this.pendingRequests.get(msg.id)!;
      this.pendingRequests.delete(msg.id);

      if (msg.error) {
        reject(new Error(msg.error.message));
      } else {
        resolve(msg);
      }
      return;
    }

    // Handle notification (no id)
    if (!msg.id) {
      this.handleNotification(msg);
    }
  }

  // Handle notification
  private handleNotification(msg: ACPMessage): void {
    const method = msg.method;

    switch (method) {
      case ACP_METHODS.TASK_PROGRESS:
        // Emit progress event
        console.log('[ACP Client] Task progress:', msg.params);
        break;
      case ACP_METHODS.TASK_COMPLETE:
        // Emit complete event
        console.log('[ACP Client] Task complete:', msg.params);
        break;
      case ACP_METHODS.APPROVAL_REQUEST:
        // Emit approval request
        console.log('[ACP Client] Approval request:', msg.params);
        break;
      default:
        console.log('[ACP Client] Unknown notification:', method);
    }
  }

  // Flush queued messages
  private async flushQueue(): Promise<void> {
    while (this.messageQueue.length > 0) {
      const msg = this.messageQueue.shift();
      if (msg) {
        this.websocket?.send(JSON.stringify(msg));
      }
      await new Promise(r => setTimeout(r, 10)); // Small delay for backpressure
    }
    this.queuedMessages.value = 0;
  }

  // Set encryption key
  setEncryptionKey(key: Uint8Array): void {
    this.encryptionKey = key;
  }

  // Get server capabilities
  getServerCapabilities(): ACPCapabilities | null {
    return this.serverCapabilities;
  }

  // Check if multiplexing is supported
  supportsMultiplexing(): boolean {
    return this.serverCapabilities?.supportsMultiplexing ?? false;
  }
}

// Singleton instance
let acpClientInstance: ACPClient | null = null;

export function useACPClient(): ACPClient {
  if (!acpClientInstance) {
    acpClientInstance = new ACPClient();
  }
  return acpClientInstance;
}