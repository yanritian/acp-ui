/**
 * Task Parser - parses user requests into structured task DAGs
 *
 * @deprecated This module is NOT USED by actual execution flow.
 *
 * The actual execution uses:
 * - AgentTeamsService for multi-agent tasks
 * - workflowStore for workflow execution
 * - swarmStore for swarm execution
 *
 * This file is kept for demo visualization (TaskGraphView) only.
 */

interface TaskTemplate {
  id: string;
  name: string;
  description: string;
  triggers: string[]; // Keywords that trigger this template
  steps: TaskStep[];
  parallelSteps: number[][]; // Indices of steps that can run in parallel
}

interface TaskStep {
  id: string;
  name: string;
  action: string;
  agentType: string;
  dependencies: string[]; // Step IDs this depends on
  params?: Record<string, unknown>;
  timeout?: number;
}

interface TaskNode {
  id: string;
  step: TaskStep;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'blocked';
  assignedAgent?: string;
  result?: unknown;
  error?: string;
}

interface TaskDAG {
  id: string;
  name: string;
  nodes: Map<string, TaskNode>;
  edges: Map<string, string[]>; // node_id -> dependent node_ids
  rootNodes: string[]; // Nodes with no dependencies
}

class TaskParser {
  private templates: Map<string, TaskTemplate> = new Map();
  private taskCounter = 0;

  constructor() {
    this.loadDefaultTemplates();
  }

  /**
   * Load default task templates
   */
  private loadDefaultTemplates(): void {
    const templates: TaskTemplate[] = [
      {
        id: 'miniprogram-dev',
        name: '小程序开发',
        description: '开发微信小程序功能',
        triggers: ['小程序', '微信小程序', 'miniprogram', '开发小程序'],
        steps: [
          { id: 'setup', name: '环境准备', action: 'setup', agentType: 'hbuilder', dependencies: [] },
          { id: 'code', name: '编写代码', action: 'write', agentType: 'code-reviewer', dependencies: ['setup'] },
          { id: 'review', name: '代码审查', action: 'review', agentType: 'code-reviewer', dependencies: ['code'] },
          { id: 'test', name: '测试验证', action: 'test', agentType: 'test-validator', dependencies: ['review'] },
          { id: 'build', name: '构建发布', action: 'build', agentType: 'hbuilder', dependencies: ['test'] },
        ],
        parallelSteps: [],
      },
      {
        id: 'android-build',
        name: 'Android构建',
        description: '构建Android应用',
        triggers: ['android', '安卓', 'apk', '构建android'],
        steps: [
          { id: 'sync', name: '代码同步', action: 'sync', agentType: 'android', dependencies: [] },
          { id: 'gradle', name: 'Gradle构建', action: 'build', agentType: 'android', dependencies: ['sync'] },
          { id: 'test', name: '单元测试', action: 'test', agentType: 'test-validator', dependencies: ['sync'] },
          { id: 'review', name: '代码审查', action: 'review', agentType: 'code-reviewer', dependencies: ['sync'] },
          { id: 'assemble', name: '打包APK', action: 'assemble', agentType: 'android', dependencies: ['gradle', 'test', 'review'] },
        ],
        parallelSteps: [[2, 3]], // test and review can run in parallel
      },
      {
        id: 'browser-control',
        name: '浏览器控制',
        description: '自动化浏览器操作',
        triggers: ['浏览器', 'browser', '网页', '自动化'],
        steps: [
          { id: 'connect', name: '连接浏览器', action: 'connect', agentType: 'browser', dependencies: [] },
          { id: 'navigate', name: '导航页面', action: 'navigate', agentType: 'browser', dependencies: ['connect'] },
          { id: 'interact', name: '执行交互', action: 'interact', agentType: 'browser', dependencies: ['navigate'] },
          { id: 'capture', name: '截图记录', action: 'capture', agentType: 'browser', dependencies: ['interact'] },
        ],
        parallelSteps: [],
      },
    ];

    for (const template of templates) {
      this.templates.set(template.id, template);
    }
  }

  /**
   * Parse user request into task DAG
   */
  parseRequest(request: string): TaskDAG | null {
    // Find matching template
    const template = this.matchTemplate(request);
    if (!template) {
      return null;
    }

    // Create task DAG from template
    return this.createDAG(template, request);
  }

  /**
   * Match user request to template
   */
  matchTemplate(request: string): TaskTemplate | null {
    const lowerRequest = request.toLowerCase();

    for (const template of this.templates.values()) {
      for (const trigger of template.triggers) {
        if (lowerRequest.includes(trigger.toLowerCase())) {
          return template;
        }
      }
    }

    // Default: return generic development template
    return this.templates.get('miniprogram-dev') || null;
  }

  /**
   * Create DAG from template
   */
  private createDAG(template: TaskTemplate, _request: string): TaskDAG {
    const dagId = `task-${Date.now()}-${this.taskCounter++}`;

    const nodes = new Map<string, TaskNode>();
    const edges = new Map<string, string[]>();
    const rootNodes: string[] = [];

    // Create nodes
    for (const step of template.steps) {
      const node: TaskNode = {
        id: `${dagId}-${step.id}`,
        step,
        status: 'pending',
      };
      nodes.set(node.id, node);

      // Find root nodes (no dependencies)
      if (step.dependencies.length === 0) {
        rootNodes.push(node.id);
      }
    }

    // Create edges (dependency graph)
    for (const step of template.steps) {
      const nodeId = `${dagId}-${step.id}`;
      for (const depId of step.dependencies) {
        const depNodeId = `${dagId}-${depId}`;
        if (!edges.has(depNodeId)) {
          edges.set(depNodeId, []);
        }
        edges.get(depNodeId)!.push(nodeId);
      }
    }

    return {
      id: dagId,
      name: template.name,
      nodes,
      edges,
      rootNodes,
    };
  }

  /**
   * Identify parallel-executable nodes
   */
  getParallelGroups(dag: TaskDAG): string[][] {
    const groups: string[][] = [];

    // Find nodes with same dependencies that are ready at same time
    const pendingNodes = Array.from(dag.nodes.values())
      .filter(n => n.status === 'pending');

    // Group by dependency set
    const byDeps = new Map<string, string[]>();
    for (const node of pendingNodes) {
      const depKey = node.step.dependencies.sort().join(',');
      if (!byDeps.has(depKey)) {
        byDeps.set(depKey, []);
      }
      byDeps.get(depKey)!.push(node.id);
    }

    // Groups with more than one node can run in parallel
    for (const group of byDeps.values()) {
      if (group.length > 1) {
        groups.push(group);
      }
    }

    return groups;
  }

  /**
   * Check if node is ready to execute (dependencies satisfied)
   */
  isNodeReady(dag: TaskDAG, nodeId: string): boolean {
    const node = dag.nodes.get(nodeId);
    if (!node || node.status !== 'pending') {
      return false;
    }

    // Check all dependencies are completed
    for (const depId of node.step.dependencies) {
      const depNodeId = `${dag.id}-${depId}`;
      const depNode = dag.nodes.get(depNodeId);
      if (!depNode || depNode.status !== 'completed') {
        return false;
      }
    }

    return true;
  }

  /**
   * Get all ready nodes
   */
  getReadyNodes(dag: TaskDAG): TaskNode[] {
    return Array.from(dag.nodes.values())
      .filter(n => this.isNodeReady(dag, n.id));
  }

  /**
   * Mark node as running
   */
  startNode(dag: TaskDAG, nodeId: string, agentId: string): void {
    const node = dag.nodes.get(nodeId);
    if (node) {
      node.status = 'running';
      node.assignedAgent = agentId;
    }
  }

  /**
   * Mark node as completed
   */
  completeNode(dag: TaskDAG, nodeId: string, result: unknown): void {
    const node = dag.nodes.get(nodeId);
    if (node) {
      node.status = 'completed';
      node.result = result;
    }
  }

  /**
   * Mark node as failed
   */
  failNode(dag: TaskDAG, nodeId: string, error: string): void {
    const node = dag.nodes.get(nodeId);
    if (node) {
      node.status = 'failed';
      node.error = error;
    }
  }

  /**
   * Get DAG progress
   */
  getProgress(dag: TaskDAG): { completed: number; total: number; percentage: number } {
    const total = dag.nodes.size;
    const completed = Array.from(dag.nodes.values())
      .filter(n => n.status === 'completed').length;
    return {
      completed,
      total,
      percentage: Math.round((completed / total) * 100),
    };
  }

  /**
   * Register custom template
   */
  registerTemplate(template: TaskTemplate): void {
    this.templates.set(template.id, template);
  }

  /**
   * List available templates
   */
  listTemplates(): TaskTemplate[] {
    return Array.from(this.templates.values());
  }
}

export const taskParser = new TaskParser();
export type { TaskTemplate, TaskStep, TaskNode, TaskDAG };