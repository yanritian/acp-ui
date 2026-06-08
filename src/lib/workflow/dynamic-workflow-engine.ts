// Dynamic Workflow Engine - Script-based workflow orchestration
// Supports JS/TS scripts for complex multi-agent workflows

import { ref, computed, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export interface WorkflowScript {
  id: string;
  name: string;
  description: string;
  script: string;             // JS/TS script content
  version: string;
  createdAt: number;
  updatedAt: number;
}

export interface WorkflowContext {
  sessionId: string;
  input: unknown;
  agents: string[];
  sharedMemory: Map<string, unknown>;
  parentContext?: WorkflowContext;
}

export interface WorkflowResult {
  success: boolean;
  output: unknown;
  duration: number;
  agentResults: Map<string, unknown>;
  errors: string[];
}

export interface WorkflowExecution {
  id: string;
  scriptId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'paused';
  startTime: number;
  endTime?: number;
  context: WorkflowContext;
  result?: WorkflowResult;
  parallelTasks: number;
}

export class DynamicWorkflowEngine {
  private scripts: Ref<Map<string, WorkflowScript>> = ref(new Map());
  private executions: Ref<Map<string, WorkflowExecution>> = ref(new Map());
  private maxConcurrency: Ref<number> = ref(50);  // Max parallel agents

  // State for UI
  public isRunning: Ref<boolean> = ref(false);
  public activeExecutions: Ref<number> = ref(0);

  // Register workflow script
  async registerScript(script: Omit<WorkflowScript, 'id' | 'createdAt' | 'updatedAt'>): Promise<WorkflowScript> {
    const id = `wf-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

    const fullScript: WorkflowScript = {
      id,
      ...script,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    // Validate script
    this.validateScript(fullScript.script);

    // Store in backend
    await invokeOrProxy('workflow_register_script', { script: fullScript });

    this.scripts.value.set(id, fullScript);
    return fullScript;
  }

  // Validate script syntax
  private validateScript(scriptContent: string): void {
    // Basic validation - check for dangerous patterns
    const dangerousPatterns = [
      /eval\s*\(/,
      /Function\s*\(/,
      /require\s*\(['"]child_process['"]\)/,
      /process\.exit/,
      /fs\.(unlink|rmdir|rm)/,
    ];

    for (const pattern of dangerousPatterns) {
      if (pattern.test(scriptContent)) {
        throw new Error(`Script contains dangerous pattern: ${pattern}`);
      }
    }
  }

  // Execute workflow script
  async runScript(scriptId: string, input: unknown, sessionId?: string): Promise<WorkflowExecution> {
    const script = this.scripts.value.get(scriptId);
    if (!script) {
      throw new Error(`Script ${scriptId} not found`);
    }

    const executionId = `exec-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

    const context: WorkflowContext = {
      sessionId: sessionId || `session-${Date.now()}`,
      input,
      agents: [],
      sharedMemory: new Map(),
    };

    const execution: WorkflowExecution = {
      id: executionId,
      scriptId,
      status: 'running',
      startTime: Date.now(),
      context,
      parallelTasks: 0,
    };

    this.executions.value.set(executionId, execution);
    this.activeExecutions.value++;
    this.isRunning.value = true;

    try {
      // Execute script in sandboxed environment
      const result = await this.executeScript(script, context);
      execution.status = 'completed';
      execution.endTime = Date.now();
      execution.result = result;
    } catch (e) {
      execution.status = 'failed';
      execution.endTime = Date.now();
      execution.result = {
        success: false,
        output: null,
        duration: execution.endTime - execution.startTime,
        agentResults: new Map(),
        errors: [String(e)],
      };
    }

    this.activeExecutions.value--;
    if (this.activeExecutions.value === 0) {
      this.isRunning.value = false;
    }

    return execution;
  }

  // Execute script in sandbox
  private async executeScript(script: WorkflowScript, context: WorkflowContext): Promise<WorkflowResult> {
    const startTime = Date.now();
    const agentResults = new Map<string, unknown>();
    const errors: string[] = [];

    // Create sandboxed execution context
    const sandbox = this.createSandbox(context, agentResults, errors);

    // Execute script
    try {
      // In production, this would use a proper sandbox (vm2, isolated-vm)
      // For now, we simulate execution
      const output = await this.simulateExecution(script.script, sandbox);

      return {
        success: errors.length === 0,
        output,
        duration: Date.now() - startTime,
        agentResults,
        errors,
      };
    } catch (e) {
      return {
        success: false,
        output: null,
        duration: Date.now() - startTime,
        agentResults,
        errors: [String(e)],
      };
    }
  }

  // Create sandbox environment
  private createSandbox(
    context: WorkflowContext,
    agentResults: Map<string, unknown>,
    errors: string[]
  ): Record<string, unknown> {
    // Define spawnAgent first
    const spawnAgentFn = async (agentId: string, task: string) => {
      context.agents.push(agentId);
      try {
        const result = await invokeOrProxy('workflow_spawn_agent', {
          agent_id: agentId,
          task,
          session_id: context.sessionId,
        });
        agentResults.set(agentId, result);
        return result;
      } catch (e) {
        errors.push(`Agent ${agentId} failed: ${e}`);
        return null;
      }
    };

    return {
      // Context
      context,
      input: context.input,
      sessionId: context.sessionId,

      // Agent control
      spawnAgent: spawnAgentFn,

      // Parallel execution
      parallel: async (tasks: Array<{ agentId: string; task: string }>) => {
        return Promise.allSettled(
          tasks.map(t => spawnAgentFn(t.agentId, t.task))
        );
      },

      // Memory access
      getMemory: (key: string) => context.sharedMemory.get(key),
      setMemory: (key: string, value: unknown) => context.sharedMemory.set(key, value),

      // Logging
      log: (msg: string) => console.log(`[Workflow ${context.sessionId}] ${msg}`),

      // Utils
      sleep: (ms: number) => new Promise(r => setTimeout(r, ms)),
      now: () => Date.now(),
    };
  }

  // Simulate script execution (placeholder for proper sandbox)
  private async simulateExecution(
    scriptContent: string,
    sandbox: Record<string, unknown>
  ): Promise<unknown> {
    // In production, use vm2 or isolated-vm
    // For now, parse script and execute simple operations

    // Detect parallel patterns
    const parallelMatch = scriptContent.match(/parallel\s*\(\s*\[([\s\S]*?)\]\s*\)/);
    if (parallelMatch) {
      // Extract tasks from parallel block
      const tasksBlock = parallelMatch[1];
      const tasks = this.parseParallelTasks(tasksBlock);
      const parallelFn = sandbox.parallel as (tasks: Array<{ agentId: string; task: string }>) => Promise<unknown[]>;
      return parallelFn(tasks);
    }

    // Detect spawn patterns
    const spawnMatch = scriptContent.match(/spawnAgent\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]\s*\)/);
    if (spawnMatch) {
      const agentId = spawnMatch[1];
      const task = spawnMatch[2];
      const spawnFn = sandbox.spawnAgent as (agentId: string, task: string) => Promise<unknown>;
      return spawnFn(agentId, task);
    }

    // Default: return input
    return sandbox.input;
  }

  // Parse parallel tasks from script
  private parseParallelTasks(block: string): Array<{ agentId: string; task: string }> {
    const tasks: Array<{ agentId: string; task: string }> = [];
    const taskMatches = block.matchAll(/\{\s*agentId:\s*['"]([^'"]+)['"]\s*,\s*task:\s*['"]([^'"]+)['"]\s*\}/g);

    for (const match of taskMatches) {
      tasks.push({ agentId: match[1], task: match[2] });
    }

    return tasks;
  }

  // Execute parallel batch with concurrency control
  async executeParallel(
    tasks: Array<{ agentId: string; task: string }>,
    concurrency: number = 50
  ): Promise<unknown[]> {
    const results: unknown[] = [];
    const batches = this.chunk(tasks, concurrency);

    for (const batch of batches) {
      const batchResults = await Promise.allSettled(
        batch.map(t => invokeOrProxy('workflow_spawn_agent', {
          agent_id: t.agentId,
          task: t.task,
        }))
      );
      results.push(...batchResults.map(r => r.status === 'fulfilled' ? r.value : null));
    }

    return results;
  }

  // Chunk array into batches
  private chunk<T>(arr: T[], size: number): T[][] {
    const chunks: T[][] = [];
    for (let i = 0; i < arr.length; i += size) {
      chunks.push(arr.slice(i, i + size));
    }
    return chunks;
  }

  // Pause execution
  async pause(executionId: string): Promise<void> {
    const execution = this.executions.value.get(executionId);
    if (execution && execution.status === 'running') {
      execution.status = 'paused';
      await invokeOrProxy('workflow_pause_execution', { execution_id: executionId });
    }
  }

  // Resume execution
  async resume(executionId: string): Promise<void> {
    const execution = this.executions.value.get(executionId);
    if (execution && execution.status === 'paused') {
      execution.status = 'running';
      await invokeOrProxy('workflow_resume_execution', { execution_id: executionId });
    }
  }

  // Cancel execution
  async cancel(executionId: string): Promise<void> {
    const execution = this.executions.value.get(executionId);
    if (execution) {
      execution.status = 'failed';
      execution.endTime = Date.now();
      await invokeOrProxy('workflow_cancel_execution', { execution_id: executionId });
    }
  }

  // Get scripts
  getScripts(): WorkflowScript[] {
    return Array.from(this.scripts.value.values());
  }

  // Get executions
  getExecutions(): WorkflowExecution[] {
    return Array.from(this.executions.value.values());
  }

  // Get execution by ID
  getExecution(executionId: string): WorkflowExecution | undefined {
    return this.executions.value.get(executionId);
  }

  // Set max concurrency
  setMaxConcurrency(max: number): void {
    this.maxConcurrency.value = max;
  }

  // Get max concurrency
  getMaxConcurrency(): number {
    return this.maxConcurrency.value;
  }
}

// Singleton instance
let dynamicWorkflowEngineInstance: DynamicWorkflowEngine | null = null;

export function useDynamicWorkflowEngine(): DynamicWorkflowEngine {
  if (!dynamicWorkflowEngineInstance) {
    dynamicWorkflowEngineInstance = new DynamicWorkflowEngine();
  }
  return dynamicWorkflowEngineInstance;
}

// Example workflow scripts
export const EXAMPLE_WORKFLOW_SCRIPTS: WorkflowScript[] = [
  {
    id: 'wf-multi-review',
    name: 'Multi-Agent Code Review',
    description: 'Run code review on multiple files in parallel',
    script: `
// Multi-Agent Code Review Workflow
const files = input.files || [];

const results = await parallel(
  files.map(file => ({
    agentId: 'code-reviewer',
    task: \`Review file: \${file}\`
  }))
);

// Aggregate results
const summary = {
  totalFiles: files.length,
  issues: results.filter(r => r?.issues?.length > 0).length,
  approved: results.filter(r => r?.approved).length
};

setMemory('review-summary', summary);
return summary;
`,
    version: '1.0.0',
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  {
    id: 'wf-tdd-cycle',
    name: 'TDD Cycle',
    description: 'Test-Driven Development cycle: write test, implement, refactor',
    script: `
// TDD Cycle Workflow
const feature = input.feature;

// Step 1: Write test
await spawnAgent('tdd-guide', \`Write test for: \${feature}\`);
await sleep(2000);

// Step 2: Implement
await spawnAgent('coder', \`Implement: \${feature}\`);
await sleep(5000);

// Step 3: Refactor
await spawnAgent('refactorer', \`Refactor implementation of: \${feature}\`);

return { feature, completed: true };
`,
    version: '1.0.0',
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
];