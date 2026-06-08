// Remote Control Server - WebSocket server for mobile/IM remote control
// Mobile -> WebSocket -> Desktop Agent

import { ref, type Ref } from 'vue';
import { type ACPMessage, ACP_METHODS } from '../acp-protocol/spec';
import { createACPMessage } from '../acp-protocol/index';

export type RemoteCommandType =
  | 'start_agent'
  | 'stop_agent'
  | 'send_message'
  | 'approve'
  | 'reject'
  | 'get_status'
  | 'pause_agent'
  | 'resume_agent';

export type RemoteEventType =
  | 'agent_status'
  | 'task_complete'
  | 'approval_needed'
  | 'error'
  | 'heartbeat';

export interface RemoteCommand {
  type: RemoteCommandType;
  payload: Record<string, unknown>;
  timestamp: number;
  source: 'mobile' | 'im' | 'web';
}

export interface RemoteEvent {
  type: RemoteEventType;
  payload: Record<string, unknown>;
  timestamp: number;
}

export interface RemoteClient {
  id: string;
  type: 'mobile' | 'im' | 'web';
  connected: boolean;
  lastActivity: number;
}

export class RemoteControlServer {
  private clients: Ref<Map<string, RemoteClient>> = ref(new Map());
  private commandQueue: RemoteCommand[] = [];
  private eventListeners: ((event: RemoteEvent) => void)[] = [];

  // State for UI
  public isRunning: Ref<boolean> = ref(false);
  public clientCount: Ref<number> = ref(0);
  public lastCommand: Ref<RemoteCommand | null> = ref(null);

  // Start server
  async start(port: number = 8766): Promise<void> {
    // In Tauri mode, use tauri-plugin-websocket
    // For now, simulate server behavior
    this.isRunning.value = true;
    console.log(`[RemoteControl] Server started on port ${port}`);

    // Start heartbeat interval
    setInterval(() => {
      this.broadcastEvent({
        type: 'heartbeat',
        payload: { timestamp: Date.now() },
        timestamp: Date.now(),
      });
    }, 30000);
  }

  // Stop server
  async stop(): Promise<void> {
    this.isRunning.value = false;
    this.clients.value.clear();
    this.clientCount.value = 0;
  }

  // Handle incoming connection
  handleConnection(clientId: string, type: 'mobile' | 'im' | 'web'): void {
    this.clients.value.set(clientId, {
      id: clientId,
      type,
      connected: true,
      lastActivity: Date.now(),
    });
    this.clientCount.value = this.clients.value.size;
  }

  // Handle disconnection
  handleDisconnection(clientId: string): void {
    const client = this.clients.value.get(clientId);
    if (client) {
      client.connected = false;
      this.clients.value.delete(clientId);
    }
    this.clientCount.value = this.clients.value.size;
  }

  // Execute command from remote client
  async executeCommand(cmd: RemoteCommand): Promise<RemoteEvent> {
    this.lastCommand.value = cmd;

    let result: Record<string, unknown>;

    switch (cmd.type) {
      case 'start_agent':
        result = await this.startAgent(cmd.payload);
        break;
      case 'stop_agent':
        result = await this.stopAgent(cmd.payload);
        break;
      case 'send_message':
        result = await this.sendMessageToAgent(cmd.payload);
        break;
      case 'approve':
        result = await this.handleApprove(cmd.payload);
        break;
      case 'reject':
        result = await this.handleReject(cmd.payload);
        break;
      case 'get_status':
        result = await this.getStatus(cmd.payload);
        break;
      case 'pause_agent':
        result = await this.pauseAgent(cmd.payload);
        break;
      case 'resume_agent':
        result = await this.resumeAgent(cmd.payload);
        break;
      default:
        result = { error: 'Unknown command type' };
    }

    const event: RemoteEvent = {
      type: 'agent_status',
      payload: result,
      timestamp: Date.now(),
    };

    this.broadcastEvent(event);
    return event;
  }

  // Start agent
  private async startAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    // Call orchestration API
    return {
      agentId: payload.agentId as string,
      status: 'started',
      timestamp: Date.now(),
    };
  }

  // Stop agent
  private async stopAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      agentId: payload.agentId as string,
      status: 'stopped',
      timestamp: Date.now(),
    };
  }

  // Send message to agent
  private async sendMessageToAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      agentId: payload.agentId as string,
      messageSent: true,
      timestamp: Date.now(),
    };
  }

  // Handle approve
  private async handleApprove(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      requestId: payload.requestId as string,
      approved: true,
      timestamp: Date.now(),
    };
  }

  // Handle reject
  private async handleReject(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      requestId: payload.requestId as string,
      rejected: true,
      timestamp: Date.now(),
    };
  }

  // Get status
  private async getStatus(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      agents: [],
      tasks: [],
      approvals: [],
      timestamp: Date.now(),
    };
  }

  // Pause agent
  private async pauseAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      agentId: payload.agentId as string,
      status: 'paused',
      timestamp: Date.now(),
    };
  }

  // Resume agent
  private async resumeAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return {
      agentId: payload.agentId as string,
      status: 'resumed',
      timestamp: Date.now(),
    };
  }

  // Broadcast event to all clients
  broadcastEvent(event: RemoteEvent): void {
    this.eventListeners.forEach(listener => listener(event));
  }

  // Add event listener
  addEventListener(listener: (event: RemoteEvent) => void): void {
    this.eventListeners.push(listener);
  }

  // Remove event listener
  removeEventListener(listener: (event: RemoteEvent) => void): void {
    this.eventListeners = this.eventListeners.filter(l => l !== listener);
  }

  // Get connected clients
  getConnectedClients(): RemoteClient[] {
    return Array.from(this.clients.value.values()).filter(c => c.connected);
  }

  // Push agent status update
  pushAgentStatus(agentId: string, status: string, details?: Record<string, unknown>): void {
    this.broadcastEvent({
      type: 'agent_status',
      payload: { agentId, status, details },
      timestamp: Date.now(),
    });
  }

  // Push task completion
  pushTaskComplete(taskId: string, result: string): void {
    this.broadcastEvent({
      type: 'task_complete',
      payload: { taskId, result },
      timestamp: Date.now(),
    });
  }

  // Push approval needed
  pushApprovalNeeded(requestId: string, description: string): void {
    this.broadcastEvent({
      type: 'approval_needed',
      payload: { requestId, description },
      timestamp: Date.now(),
    });
  }

  // Push error
  pushError(error: string, context?: Record<string, unknown>): void {
    this.broadcastEvent({
      type: 'error',
      payload: { error, context },
      timestamp: Date.now(),
    });
  }
}

// Singleton instance
let remoteControlServerInstance: RemoteControlServer | null = null;

export function useRemoteControlServer(): RemoteControlServer {
  if (!remoteControlServerInstance) {
    remoteControlServerInstance = new RemoteControlServer();
  }
  return remoteControlServerInstance;
}