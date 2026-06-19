/**
 * Orchestrator - manages task execution flow
 *
 * @deprecated This class is NOT USED by any component.
 *
 * The actual execution chain uses:
 * - MultiAgentChat.vue → teamRuntime.runTeamTask() → AgentTeamsService → AcpSessionRunner
 * - SwarmDashboard.vue → swarmStore → swarm-api → Tauri swarm commands
 *
 * This file is kept for reference only. Do not use in new code.
 * Consider removing in future cleanup.
 */

import { taskParser, TaskDAG, TaskNode } from './task-parser';

interface AgentInfo {
  id: string;
  name: string;
  capabilities: string[];
  currentTasks: number;
  maxTasks: number;
}

interface TaskAssignment {
  taskId: string;
  nodeId: string;
  agentId: string;
  timestamp: Date;
}

interface OrchestratorConfig {
  maxParallelTasks: number;
  taskTimeoutMs: number;
  retryAttempts: number;
  retryDelayMs: number;
}

type EventType = 'task_complete' | 'task_failed' | 'review_result' | 'test_result' | 'agent_available';

interface Event {
  type: EventType;
  payload: unknown;
  timestamp: Date;
}

class Orchestrator {
  private activeDAGs: Map<string, TaskDAG> = new Map();
  private agents: Map<string, AgentInfo> = new Map();
  private assignments: Map<string, TaskAssignment> = new Map();
  private eventListeners: Map<EventType, Set<(event: Event) => void>> = new Map();
  private config: OrchestratorConfig;

  constructor(config?: Partial<OrchestratorConfig>) {
    this.config = {
      maxParallelTasks: 3,
      taskTimeoutMs: 60000,
      retryAttempts: 2,
      retryDelayMs: 5000,
      ...config,
    };
  }

  /**
   * Register agent
   */
  registerAgent(agent: AgentInfo): void {
    this.agents.set(agent.id, agent);
    this.emit('agent_available', agent);
  }

  /**
   * Unregister agent
   */
  unregisterAgent(agentId: string): void {
    this.agents.delete(agentId);
  }

  /**
   * Start task execution
   */
  startTask(request: string): string | null {
    const dag = taskParser.parseRequest(request);
    if (!dag) {
      console.error('Failed to parse request:', request);
      return null;
    }

    this.activeDAGs.set(dag.id, dag);
    this.scheduleNext(dag.id);

    return dag.id;
  }

  /**
   * Schedule next available tasks
   */
  private scheduleNext(dagId: string): void {
    const dag = this.activeDAGs.get(dagId);
    if (!dag) return;

    // Get ready nodes
    const readyNodes = taskParser.getReadyNodes(dag);

    // Limit parallel execution
    const runningCount = Array.from(dag.nodes.values())
      .filter(n => n.status === 'running').length;

    const availableSlots = Math.max(0, this.config.maxParallelTasks - runningCount);
    const nodesToSchedule = readyNodes.slice(0, availableSlots);

    for (const node of nodesToSchedule) {
      // Find best agent
      const agent = this.findBestAgent(node.step.agentType);
      if (agent) {
        this.assignTask(dag, node, agent);
      } else {
        console.warn(`No agent available for task ${node.id}`);
      }
    }
  }

  /**
   * Find best agent for task type
   */
  private findBestAgent(agentType: string): AgentInfo | null {
    const candidates = Array.from(this.agents.values())
      .filter(a => a.capabilities.includes(agentType) && a.currentTasks < a.maxTasks);

    if (candidates.length === 0) return null;

    // Sort by load (prefer agents with fewer current tasks)
    candidates.sort((a, b) => a.currentTasks - b.currentTasks);

    return candidates[0];
  }

  /**
   * Assign task to agent
   */
  private assignTask(dag: TaskDAG, node: TaskNode, agent: AgentInfo): void {
    taskParser.startNode(dag, node.id, agent.id);

    const assignment: TaskAssignment = {
      taskId: dag.id,
      nodeId: node.id,
      agentId: agent.id,
      timestamp: new Date(),
    };
    this.assignments.set(node.id, assignment);

    // Update agent load
    agent.currentTasks++;

    console.log(`Assigned ${node.id} to agent ${agent.id}`);

    // Execute task
    this.executeTask(dag, node, agent);
  }

  /**
 * Execute task — now uses real swarm API (M-2 fix)
   */
  private async executeTask(dag: TaskDAG, node: TaskNode, agent: AgentInfo): Promise<void> {
    // Import swarm-api dynamically
    const { swarmRegisterWorker, swarmSendTask, swarmGetTaskOutput } = await import('./swarm-api');

    try {
      // Register worker if needed
      await swarmRegisterWorker('claude_code', agent.id);

      // Create task
      const taskId = `task-${node.id}-${Date.now()}`;
      const prompt = node.step?.action || node.step?.name || node.id;

      // Send task to worker
      await swarmSendTask(agent.id, taskId, prompt, undefined, 60000);

      // Poll for completion (non-blocking)
      const pollInterval = 1000;
      const maxPolls = 60; // 60 seconds max
      let polls = 0;
      let outputStr = '';

      while (polls < maxPolls) {
        try {
          const taskOutput = await swarmGetTaskOutput(agent.id, taskId);
          if (taskOutput && typeof taskOutput === 'string') {
            outputStr = taskOutput;
            break;
          }
        } catch {
          // Task not ready yet
        }

        await new Promise(resolve => setTimeout(resolve, pollInterval));
        polls++;
      }

      // Handle completion
      if (outputStr) {
        this.handleTaskComplete(dag.id, node.id, { success: true, output: outputStr });
      } else {
        this.handleTaskFailed(dag.id, node.id, 'Task timed out or no output');
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      this.handleTaskFailed(dag.id, node.id, errorMsg);
    }
  }

  /**
   * Handle task completion
   */
  handleTaskComplete(dagId: string, nodeId: string, result: unknown): void {
    const dag = this.activeDAGs.get(dagId);
    if (!dag) return;

    const node = dag.nodes.get(nodeId);
    if (!node) return;

    // Mark complete
    taskParser.completeNode(dag, nodeId, result);

    // Free agent
    const assignment = this.assignments.get(nodeId);
    if (assignment) {
      const agent = this.agents.get(assignment.agentId);
      if (agent) {
        agent.currentTasks--;
      }
      this.assignments.delete(nodeId);
    }

    // Emit event
    this.emit('task_complete', { dagId, nodeId, result });

    // Schedule next tasks
    this.scheduleNext(dagId);

    // Check if DAG is complete
    this.checkDAGComplete(dagId);
  }

  /**
   * Handle task failure
   */
  handleTaskFailed(dagId: string, nodeId: string, error: string): void {
    const dag = this.activeDAGs.get(dagId);
    if (!dag) return;

    const node = dag.nodes.get(nodeId);
    if (!node) return;

    // Check retry attempts
    const retryCount = (node.result as { retryCount?: number })?.retryCount || 0;

    if (retryCount < this.config.retryAttempts) {
      // Retry
      node.status = 'pending';
      node.result = { retryCount: retryCount + 1 };

      console.log(`Retrying task ${nodeId} (attempt ${retryCount + 1})`);

      setTimeout(() => {
        this.scheduleNext(dagId);
      }, this.config.retryDelayMs);
    } else {
      // Mark failed
      taskParser.failNode(dag, nodeId, error);

      // Free agent
      const assignment = this.assignments.get(nodeId);
      if (assignment) {
        const agent = this.agents.get(assignment.agentId);
        if (agent) {
          agent.currentTasks--;
        }
        this.assignments.delete(nodeId);
      }

      // Emit event
      this.emit('task_failed', { dagId, nodeId, error });

      // Block dependent tasks
      this.blockDependents(dag, nodeId);

      // Check DAG status
      this.checkDAGComplete(dagId);
    }
  }

  /**
   * Block dependent tasks when parent fails
   */
  private blockDependents(dag: TaskDAG, nodeId: string): void {
    const dependents = dag.edges.get(nodeId) || [];
    for (const depId of dependents) {
      const depNode = dag.nodes.get(depId);
      if (depNode && depNode.status === 'pending') {
        depNode.status = 'blocked';
        depNode.error = `Blocked by failed parent: ${nodeId}`;
      }
      // Recursively block
      this.blockDependents(dag, depId);
    }
  }

  /**
   * Check if DAG is complete
   */
  private checkDAGComplete(dagId: string): void {
    const dag = this.activeDAGs.get(dagId);
    if (!dag) return;

    const statuses = Array.from(dag.nodes.values()).map(n => n.status);
    const allComplete = statuses.every(s => s === 'completed' || s === 'failed' || s === 'blocked');
    const hasSuccess = statuses.some(s => s === 'completed');

    if (allComplete) {
      console.log(`DAG ${dagId} complete: ${hasSuccess ? 'success' : 'failed'}`);
      this.activeDAGs.delete(dagId);
    }
  }

  /**
   * Subscribe to events
   */
  subscribe(eventType: EventType, listener: (event: Event) => void): void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(listener);
  }

  /**
   * Unsubscribe from events
   */
  unsubscribe(eventType: EventType, listener: (event: Event) => void): void {
    this.eventListeners.get(eventType)?.delete(listener);
  }

  /**
   * Emit event
   */
  private emit(type: EventType, payload: unknown): void {
    const event: Event = {
      type,
      payload,
      timestamp: new Date(),
    };

    this.eventListeners.get(type)?.forEach(listener => listener(event));
  }

  /**
   * Get active tasks status
   */
  getStatus(): { activeDAGs: number; runningTasks: number; availableAgents: number } {
    const runningTasks = Array.from(this.assignments.values()).length;
    const availableAgents = Array.from(this.agents.values())
      .filter(a => a.currentTasks < a.maxTasks).length;

    return {
      activeDAGs: this.activeDAGs.size,
      runningTasks,
      availableAgents,
    };
  }
}

export const orchestrator = new Orchestrator();
export type { AgentInfo, TaskAssignment, OrchestratorConfig, EventType, Event };