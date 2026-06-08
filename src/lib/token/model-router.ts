// Model Router - Smart routing based on task complexity
// Routes simple tasks to cheaper models, complex tasks to powerful models

import { ref, computed, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export interface ModelInfo {
  id: string;
  name: string;
  provider: 'anthropic' | 'openai' | 'google' | 'custom';
  costPerToken: number;      // USD per 1K tokens
  maxTokens: number;
  capabilities: string[];
  latencyMs: number;         // Average response latency
}

export interface TaskComplexity {
  score: number;             // 0-1, higher = more complex
  factors: {
    promptLength: number;    // 0-1
    toolCount: number;       // 0-1
    contextDepth: number;    // 0-1
    reasoningRequired: boolean;
    multiStep: boolean;
  };
  recommendation: string;
}

export interface ModelSelection {
  model: ModelInfo;
  reason: string;
  estimatedCost: number;
  estimatedLatency: number;
}

// Available models
const AVAILABLE_MODELS: ModelInfo[] = [
  {
    id: 'claude-opus-4-7',
    name: 'Claude Opus 4.7',
    provider: 'anthropic',
    costPerToken: 0.015,
    maxTokens: 200000,
    capabilities: ['reasoning', 'code', 'analysis', 'vision', 'long-context'],
    latencyMs: 5000,
  },
  {
    id: 'claude-sonnet-4-6',
    name: 'Claude Sonnet 4.6',
    provider: 'anthropic',
    costPerToken: 0.003,
    maxTokens: 200000,
    capabilities: ['code', 'analysis', 'fast-response'],
    latencyMs: 2000,
  },
  {
    id: 'claude-haiku-4-5',
    name: 'Claude Haiku 4.5',
    provider: 'anthropic',
    costPerToken: 0.00025,
    maxTokens: 200000,
    capabilities: ['fast-response', 'simple-tasks'],
    latencyMs: 500,
  },
  {
    id: 'gpt-4o-mini',
    name: 'GPT-4o Mini',
    provider: 'openai',
    costPerToken: 0.00015,
    maxTokens: 128000,
    capabilities: ['fast-response', 'simple-tasks'],
    latencyMs: 300,
  },
];

export class ModelRouter {
  private currentSelection: Ref<ModelSelection | null> = ref(null);
  public lastComplexity: Ref<TaskComplexity | null> = ref(null);
  public routingHistory: Ref<ModelSelection[]> = ref([]);

  // Evaluate task complexity
  async evaluateComplexity(task: {
    prompt: string;
    tools?: string[];
    context?: string;
    hasReasoning?: boolean;
    isMultiStep?: boolean;
  }): Promise<TaskComplexity> {
    // Use smart_router.rs if available (Tauri mode)
    try {
      const result = await invokeOrProxy('smart_router_evaluate', {
        prompt: task.prompt,
        tools: task.tools || [],
        context: task.context || '',
      }) as TaskComplexity | null;

      if (result && typeof result.score === 'number') {
        return result;
      }
    } catch (e) {
      // Fall back to local evaluation
    }

    // Local complexity evaluation
    const promptLength = Math.min(task.prompt.length / 2000, 1);
    const toolCount = Math.min((task.tools?.length || 0) / 10, 1);
    const contextDepth = task.context ? Math.min(task.context.length / 5000, 1) : 0;

    const score = (
      promptLength * 0.3 +
      toolCount * 0.25 +
      contextDepth * 0.2 +
      (task.hasReasoning ? 0.15 : 0) +
      (task.isMultiStep ? 0.1 : 0)
    );

    const recommendation = score < 0.3
      ? 'Use fast/cheap model (Haiku/GPT-4o-mini)'
      : score < 0.7
        ? 'Use balanced model (Sonnet)'
        : 'Use powerful model (Opus)';

    return {
      score,
      factors: {
        promptLength,
        toolCount,
        contextDepth,
        reasoningRequired: task.hasReasoning || false,
        multiStep: task.isMultiStep || false,
      },
      recommendation,
    };
  }

  // Select best model for task
  async selectModel(task: {
    prompt: string;
    tools?: string[];
    context?: string;
    hasReasoning?: boolean;
    isMultiStep?: boolean;
    preferSpeed?: boolean;
    budgetLimit?: number;
  }): Promise<ModelSelection> {
    const complexity = await this.evaluateComplexity(task);
    this.lastComplexity.value = complexity;

    // Filter by budget if specified
    let candidates = AVAILABLE_MODELS;
    if (task.budgetLimit !== undefined) {
      candidates = candidates.filter(m => m.costPerToken <= task.budgetLimit!);
    }

    // Filter by speed preference
    if (task.preferSpeed) {
      candidates = candidates.filter(m => m.latencyMs < 1000);
    }

    // Select based on complexity
    let selected: ModelInfo;
    let reason: string;

    if (complexity.score < 0.3) {
      // Simple task - use cheapest model
      selected = candidates.find(m => m.costPerToken < 0.001) || candidates[0];
      reason = 'Simple task routed to cost-effective model';
    } else if (complexity.score < 0.7) {
      // Moderate task - use balanced model
      selected = candidates.find(m => m.costPerToken < 0.01 && m.costPerToken > 0.001) || candidates[1];
      reason = 'Moderate complexity routed to balanced model';
    } else {
      // Complex task - use most capable model
      selected = candidates.find(m => m.capabilities.includes('reasoning')) || candidates[0];
      reason = 'Complex task routed to high-capability model';
    }

    const estimatedTokens = Math.ceil(task.prompt.length / 4 + (task.context?.length || 0) / 4);
    const estimatedCost = estimatedTokens * selected.costPerToken;

    const selection: ModelSelection = {
      model: selected,
      reason,
      estimatedCost,
      estimatedLatency: selected.latencyMs,
    };

    this.currentSelection.value = selection;
    this.routingHistory.value.push(selection);

    return selection;
  }

  // Get current selection
  getCurrentSelection(): ModelSelection | null {
    return this.currentSelection.value;
  }

  // Get routing history
  getRoutingHistory(): ModelSelection[] {
    return this.routingHistory.value;
  }

  // Clear history
  clearHistory(): void {
    this.routingHistory.value = [];
  }

  // Get available models
  getAvailableModels(): ModelInfo[] {
    return AVAILABLE_MODELS;
  }
}

// Singleton instance
let modelRouterInstance: ModelRouter | null = null;

export function useModelRouter(): ModelRouter {
  if (!modelRouterInstance) {
    modelRouterInstance = new ModelRouter();
  }
  return modelRouterInstance;
}