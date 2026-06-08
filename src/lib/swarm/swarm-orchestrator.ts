// Swarm Orchestrator - Top-level Agent蜂群编排
// Supports 5 topology types: Star, Chain, Mesh, Tree, Ring

import { ref, computed, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export type SwarmTopologyType = 'star' | 'chain' | 'mesh' | 'tree' | 'ring';
export type SwarmRole = 'queen' | 'worker' | 'observer';

export interface SwarmAgent {
  id: string;
  name: string;
  type: 'codex' | 'claude-code' | 'custom';
  role: SwarmRole;
  command: string;           // CLI command to invoke
  args: string[];
  capabilities: string[];
  status: 'idle' | 'busy' | 'error' | 'offline';
  currentTask?: string;
  load: number;              // 0-1, current workload
  maxLoad: number;           // Max concurrent tasks
}

export interface SwarmTopology {
  type: SwarmTopologyType;
  queen?: string;            // Queen agent ID (for star/tree)
  order?: string[];          // Order for chain/ring
  depth?: number;            // Tree depth
  workers: string[];         // Worker agent IDs
}

export interface SwarmTask {
  id: string;
  description: string;
  status: 'pending' | 'assigned' | 'running' | 'completed' | 'failed';
  assignedTo?: string;       // Agent ID
  createdAt: number;
  completedAt?: number;
  result?: string;
  error?: string;
}

export interface SwarmStatus {
  topology: SwarmTopology;
  agents: SwarmAgent[];
  tasks: SwarmTask[];
  stats: {
    totalAgents: number;
    activeAgents: number;
    pendingTasks: number;
    runningTasks: number;
    completedTasks: number;
    failedTasks: number;
  };
}

export class SwarmOrchestrator {
  private topology: Ref<SwarmTopology | null> = ref(null);
  private agents: Ref<SwarmAgent[]> = ref([]);
  private tasks: Ref<SwarmTask[]> = ref([]);
  private queenId: Ref<string | null> = ref(null);

  // State for UI
  public isInitialized: Ref<boolean> = ref(false);
  public status: Ref<SwarmStatus | null> = ref(null);

  // Initialize swarm with topology
  async initialize(topologyType: SwarmTopologyType, config?: {
    queenType?: 'codex' | 'claude-code' | 'custom';
    workerCount?: number;
    customAgents?: SwarmAgent[];
  }): Promise<void> {
    // Create queen agent if needed
    if (topologyType === 'star' || topologyType === 'tree') {
      const queen = await this.createQueenAgent(config?.queenType || 'codex');
      this.queenId.value = queen.id;
    }

    // Create worker agents
    const workers: SwarmAgent[] = [];
    const workerCount = config?.workerCount || 3;

    for (let i = 0; i < workerCount; i++) {
      const worker = await this.createWorkerAgent('claude-code', i);
      workers.push(worker);
    }

    // Add custom agents if provided
    if (config?.customAgents) {
      workers.push(...config.customAgents);
    }

    this.agents.value = workers;

    // Set topology
    this.topology.value = this.createTopology(topologyType, workers);
    this.isInitialized.value = true;

    // Update status
    this.updateStatus();
  }

  // Create queen agent
  private async createQueenAgent(type: 'codex' | 'claude-code' | 'custom'): Promise<SwarmAgent> {
    const commands: Record<string, { command: string; args: string[] }> = {
      codex: { command: 'codex', args: ['--full-auto'] },
      'claude-code': { command: 'claude', args: ['--dangerously-skip-permissions'] },
      custom: { command: 'custom-agent', args: [] },
    };

    const { command, args } = commands[type];

    // Register with backend
    await invokeOrProxy('swarm_register_agent', {
      agent_type: type,
      role: 'queen',
      command,
      args,
    });

    const agent: SwarmAgent = {
      id: `queen-${type}-${Date.now()}`,
      name: `Queen ${type}`,
      type,
      role: 'queen',
      command,
      args,
      capabilities: ['orchestration', 'task-assignment', 'monitoring'],
      status: 'idle',
      load: 0,
      maxLoad: 5,
    };

    this.agents.value.push(agent);
    return agent;
  }

  // Create worker agent
  private async createWorkerAgent(type: 'codex' | 'claude-code' | 'custom', index: number): Promise<SwarmAgent> {
    const commands: Record<string, { command: string; args: string[] }> = {
      codex: { command: 'codex', args: ['--auto'] },
      'claude-code': { command: 'claude', args: [] },
      custom: { command: 'custom-agent', args: [] },
    };

    const { command, args } = commands[type];

    // Register with backend
    await invokeOrProxy('swarm_register_agent', {
      agent_type: type,
      role: 'worker',
      command,
      args,
    });

    return {
      id: `worker-${type}-${index}`,
      name: `Worker ${type} #${index}`,
      type,
      role: 'worker',
      command,
      args,
      capabilities: ['execution', 'code-generation', 'analysis'],
      status: 'idle',
      load: 0,
      maxLoad: 3,
    };
  }

  // Create topology configuration
  private createTopology(type: SwarmTopologyType, workers: SwarmAgent[]): SwarmTopology {
    const workerIds = workers.map(w => w.id);

    switch (type) {
      case 'star':
        return {
          type: 'star',
          queen: this.queenId.value || undefined,
          workers: workerIds,
        };
      case 'chain':
        return {
          type: 'chain',
          order: workerIds,
          workers: workerIds,
        };
      case 'mesh':
        return {
          type: 'mesh',
          workers: workerIds,
        };
      case 'tree':
        return {
          type: 'tree',
          queen: this.queenId.value || undefined,
          depth: 2,
          workers: workerIds,
        };
      case 'ring':
        return {
          type: 'ring',
          order: workerIds,
          workers: workerIds,
        };
    }
  }

  // Submit task to swarm
  async submitTask(description: string): Promise<SwarmTask> {
    const task: SwarmTask = {
      id: `task-${Date.now()}`,
      description,
      status: 'pending',
      createdAt: Date.now(),
    };

    this.tasks.value.push(task);

    // Assign based on topology
    await this.assignTask(task);

    this.updateStatus();
    return task;
  }

  // Assign task based on topology
  private async assignTask(task: SwarmTask): Promise<void> {
    if (!this.topology.value) return;

    switch (this.topology.value.type) {
      case 'star':
        // Queen assigns to least loaded worker
        const leastLoaded = this.findLeastLoadedWorker();
        if (leastLoaded) {
          await this.executeOnAgent(task, leastLoaded);
        }
        break;

      case 'chain':
        // First agent in chain starts
        const firstAgent = this.agents.value.find(a => a.id === this.topology.value?.order?.[0]);
        if (firstAgent) {
          await this.executeOnAgent(task, firstAgent);
        }
        break;

      case 'mesh':
        // Broadcast to all agents, fastest responds
        await this.broadcastTask(task);
        break;

      case 'tree':
        // Queen delegates to branch workers
        const worker = this.findLeastLoadedWorker();
        if (worker) {
          await this.executeOnAgent(task, worker);
        }
        break;

      case 'ring':
        // Round-robin assignment
        const nextWorker = this.getNextRingWorker();
        if (nextWorker) {
          await this.executeOnAgent(task, nextWorker);
        }
        break;
    }
  }

  // Find least loaded worker
  private findLeastLoadedWorker(): SwarmAgent | null {
    const workers = this.agents.value.filter(a => a.role === 'worker');
    if (workers.length === 0) return null;

    return workers.reduce((min, w) =>
      w.load < min.load ? w : min
    , workers[0]);
  }

  // Get next worker in ring
  private getNextRingWorker(): SwarmAgent | null {
    const order = this.topology.value?.order || [];
    const lastTask = this.tasks.value.filter(t => t.assignedTo).pop();
    const lastAgentId = lastTask?.assignedTo;

    if (!lastAgentId) {
      return this.agents.value.find(a => a.id === order[0]) || null;
    }

    const lastIndex = order.indexOf(lastAgentId);
    const nextIndex = (lastIndex + 1) % order.length;
    return this.agents.value.find(a => a.id === order[nextIndex]) || null;
  }

  // Execute task on specific agent
  private async executeOnAgent(task: SwarmTask, agent: SwarmAgent): Promise<void> {
    task.status = 'assigned';
    task.assignedTo = agent.id;
    agent.status = 'busy';
    agent.currentTask = task.id;
    agent.load += 1 / agent.maxLoad;

    // Call backend to execute
    try {
      const result = await invokeOrProxy('swarm_execute_task', {
        agent_id: agent.id,
        task_id: task.id,
        description: task.description,
      });

      task.status = 'running';

      // Poll for completion (or use event listener)
      await this.waitForCompletion(task);
    } catch (e) {
      task.status = 'failed';
      task.error = String(e);
      agent.status = 'error';
    }
  }

  // Broadcast task to all agents
  private async broadcastTask(task: SwarmTask): Promise<void> {
    await invokeOrProxy('swarm_broadcast_task', {
      task_id: task.id,
      description: task.description,
    });

    task.status = 'assigned';
  }

  // Wait for task completion
  private async waitForCompletion(task: SwarmTask, timeoutMs = 60000): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      const status = await invokeOrProxy('swarm_get_task_status', { task_id: task.id });

      if (status === 'completed') {
        task.status = 'completed';
        task.completedAt = Date.now();

        // Update agent status
        const agent = this.agents.value.find(a => a.id === task.assignedTo);
        if (agent) {
          agent.status = 'idle';
          agent.currentTask = undefined;
          agent.load -= 1 / agent.maxLoad;
        }
        return;
      }

      if (status === 'failed') {
        task.status = 'failed';
        return;
      }

      await new Promise(r => setTimeout(r, 1000));
    }

    task.status = 'failed';
    task.error = 'Timeout';
  }

  // Update status summary
  private updateStatus(): void {
    if (!this.topology.value) return;

    this.status.value = {
      topology: this.topology.value,
      agents: this.agents.value,
      tasks: this.tasks.value,
      stats: {
        totalAgents: this.agents.value.length,
        activeAgents: this.agents.value.filter(a => a.status === 'busy').length,
        pendingTasks: this.tasks.value.filter(t => t.status === 'pending').length,
        runningTasks: this.tasks.value.filter(t => t.status === 'running').length,
        completedTasks: this.tasks.value.filter(t => t.status === 'completed').length,
        failedTasks: this.tasks.value.filter(t => t.status === 'failed').length,
      },
    };
  }

  // Get current status
  getStatus(): SwarmStatus | null {
    return this.status.value;
  }

  // Get agents
  getAgents(): SwarmAgent[] {
    return this.agents.value;
  }

  // Get tasks
  getTasks(): SwarmTask[] {
    return this.tasks.value;
  }

  // Adaptive Queen election (for self-healing)
  async electNewQueen(): Promise<SwarmAgent | null> {
    // Find best candidate among workers
    const workers = this.agents.value.filter(a => a.role === 'worker');

    // Sort by: lowest load, highest success rate
    const candidate = workers.reduce((best, w) => {
      const wScore = 1 - w.load;
      const bScore = best ? 1 - best.load : 0;
      return wScore > bScore ? w : best;
    }, null as SwarmAgent | null);

    if (candidate) {
      candidate.role = 'queen';
      this.queenId.value = candidate.id;

      if (this.topology.value) {
        this.topology.value.queen = candidate.id;
      }

      this.updateStatus();
    }

    return candidate;
  }

  // Shutdown swarm
  async shutdown(): Promise<void> {
    // Cancel all running tasks
    for (const task of this.tasks.value.filter(t => t.status === 'running')) {
      await invokeOrProxy('swarm_cancel_task', { task_id: task.id });
    }

    // Deregister all agents
    for (const agent of this.agents.value) {
      await invokeOrProxy('swarm_deregister_agent', { agent_id: agent.id });
    }

    this.agents.value = [];
    this.tasks.value = [];
    this.topology.value = null;
    this.isInitialized.value = false;
    this.status.value = null;
  }
}

// Singleton instance
let swarmOrchestratorInstance: SwarmOrchestrator | null = null;

export function useSwarmOrchestrator(): SwarmOrchestrator {
  if (!swarmOrchestratorInstance) {
    swarmOrchestratorInstance = new SwarmOrchestrator();
  }
  return swarmOrchestratorInstance;
}