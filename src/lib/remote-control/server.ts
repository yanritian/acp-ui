// Remote Control Server - WebSocket server for mobile/IM remote control
// Mobile -> WebSocket -> Desktop Agent
//
// @deprecated This module is a scaffold / interface definition.
// All command handlers are scaffold implementations — TODO(Phase 2): wire to real orchestration API.
// TODO(Phase 2): Wire to real WebSocket transport and orchestration API.

import { ref, type Ref } from 'vue';

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
  private eventListeners: ((event: RemoteEvent) => void)[] = [];
  private heartbeatIntervalId: ReturnType<typeof setInterval> | null = null;

  // State for UI
  public isRunning: Ref<boolean> = ref(false);
  public clientCount: Ref<number> = ref(0);
  public lastCommand: Ref<RemoteCommand | null> = ref(null);

  // Start server — scaffold, requires Tauri plugin or sidecar
  // TODO(Phase 2): Implement real WebSocket server
  async start(_port: number = 8766): Promise<void> {
    console.warn('[RemoteControl] start() is a scaffold — real server not yet wired');
    // Note: does NOT set isRunning to true (no fake state)

    // Heartbeat would be started here when real server is implemented
    // this.heartbeatIntervalId = setInterval(...)
  }

  // Stop server
  async stop(): Promise<void> {
    if (this.heartbeatIntervalId) {
      clearInterval(this.heartbeatIntervalId);
      this.heartbeatIntervalId = null;
    }
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

    try {
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
    } catch (e) {
      const err = e as Error;
      console.warn(`[RemoteControl] Command '${cmd.type}' failed:`, err.message);
      result = { error: err.message };
    }

    const event: RemoteEvent = {
      type: 'agent_status',
      payload: result,
      timestamp: Date.now(),
    };

    this.broadcastEvent(event);
    return event;
  }

  // --- Command handlers (wired to swarm API) ---

  private async startAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmRegisterWorker, swarmGetWorkerStatus } = await import('@/lib/swarm-api');
    const workerType = ((payload.workerType as string) || 'claude_code') as 'codex' | 'claude_code';
    const workerId = (payload.workerId as string) || `worker-${Date.now()}`;

    const capabilities = await swarmRegisterWorker(workerType, workerId);
    const status = await swarmGetWorkerStatus(capabilities.workerId);

    return {
      workerId,
      workerType,
      health: status.health,
      capabilities,
    };
  }

  private async stopAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmShutdownWorker } = await import('@/lib/swarm-api');
    const workerId = payload.workerId as string;

    await swarmShutdownWorker(workerId);
    return { workerId, stopped: true };
  }

  private async sendMessageToAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmSendTask, swarmGetTaskOutput } = await import('@/lib/swarm-api');
    const workerId = payload.workerId as string;
    const message = payload.message as string;
    const taskId = crypto.randomUUID();

    const taskHandle = await swarmSendTask(
      workerId,
      taskId,
      message,
      payload.workingDir as string | undefined,
      (payload.timeoutMs as number) || 60000
    );

    // Poll for output (simplified)
    const output = await swarmGetTaskOutput(workerId, taskHandle.taskId);

    return {
      taskId: taskHandle.taskId,
      workerId,
      output,
      status: taskHandle.status,
    };
  }

  private async handleApprove(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    // TODO: Wire to approval queue system
    const requestId = payload.requestId as string;
    return { requestId, approved: true, message: 'Approval recorded (stub)' };
  }

  private async handleReject(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    // TODO: Wire to approval queue system
    const requestId = payload.requestId as string;
    return { requestId, rejected: true, message: 'Rejection recorded (stub)' };
  }

  private async getStatus(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmListWorkers, swarmGetWorkerStatus, queenGetLease } = await import('@/lib/swarm-api');

    const workers = await swarmListWorkers();
    const queenLease = await queenGetLease();

    const workerStatuses = await Promise.all(
      workers.map(async (w) => {
        try {
          const status = await swarmGetWorkerStatus(w.workerId);
          return { id: w.workerId, type: w.workerType, health: status.health, tasksCompleted: status.tasksCompleted };
        } catch {
          return { id: w.workerId, type: w.workerType, health: 'offline', tasksCompleted: 0 };
        }
      })
    );

    return {
      workers: workerStatuses,
      queen: queenLease ? { queenId: queenLease.queenId, isValid: queenLease.isValid } : null,
      timestamp: Date.now(),
    };
  }

  private async pauseAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmGetWorkerStatus } = await import('@/lib/swarm-api');
    const workerId = payload.workerId as string;

    // Note: No direct pause API - return current status
    const status = await swarmGetWorkerStatus(workerId);
    return {
      workerId,
      paused: false,
      message: 'Pause not directly supported - worker continues',
      currentHealth: status.health,
    };
  }

  private async resumeAgent(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    const { swarmGetWorkerStatus } = await import('@/lib/swarm-api');
    const workerId = payload.workerId as string;

    const status = await swarmGetWorkerStatus(workerId);
    return {
      workerId,
      resumed: true,
      currentHealth: status.health,
    };
  }

  // Broadcast event to all listeners
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
