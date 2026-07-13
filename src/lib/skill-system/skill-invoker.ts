/**
 * Skill Invoker - Frontend wrapper for invoke_skill meta-tool
 *
 * Provides convenient interface for:
 * - Invoking skills with parameters
 * - Creating skills from natural language
 * - Rating skills for self-evolution
 * - Managing skill lifecycle
 */

import { invokeOrProxy } from '@/lib/host';
import type {
  Skill,
  SkillInvocation,
  SkillExecutionResult,
  SkillMeta,
  SkillContext,
  CreateSkillRequest,
  CreateSkillResult,
  SkillRatingRequest,
  EvolutionTriggers,
} from './types';

// ---------------------------------------------------------------------------
// Skill Invoker Class
// ---------------------------------------------------------------------------

export class SkillInvoker {
  /**
   * Invoke a skill by name
   *
   * This is the core meta-tool entry point.
   * All capabilities are delegated through this single interface.
   */
  async invokeSkill(invocation: SkillInvocation): Promise<SkillExecutionResult> {
    try {
      return await invokeOrProxy<SkillExecutionResult>('invoke_skill', {
        invocation: {
          skill_name: invocation.skillName,
          parameters: invocation.parameters,
          context: invocation.context || { cwd: '.' },
        },
      });

    } catch (error) {
      return {
        success: false,
        output: '',
        error: String(error),
        durationMs: 0,
        toolCallsCount: 0,
        evolved: false,
      };
    }
  }

  /**
   * Create a skill from natural language description
   *
   * Inspired by OpenClacky: "agent drafts SKILL.md, breaks down steps"
   */
  async createSkill(request: CreateSkillRequest): Promise<CreateSkillResult> {
    try {
      return await invokeOrProxy<CreateSkillResult>('create_skill', {
        name: request.name,
        description: request.description,
        natural_spec: request.naturalSpec,
      });

    } catch (error) {
      return {
        created: false,
        name: request.name,
        message: String(error),
      };
    }
  }

  /**
   * List all available skills
   */
  async listSkills(): Promise<SkillMeta[]> {
    try {
      return await invokeOrProxy<SkillMeta[]>('skills_list');
    } catch (error) {
      console.error('Failed to list skills:', error);
      return [];
    }
  }

  /**
   * View a specific skill's content
   */
  async viewSkill(name: string): Promise<string | null> {
    try {
      return await invokeOrProxy<string>('skill_view', { name });
    } catch (error) {
      console.error(`Failed to view skill ${name}:`, error);
      return null;
    }
  }

  /**
   * Manage skills (create/update/delete/self_improve)
   */
  async manageSkill(action: string, params: Record<string, unknown>): Promise<string> {
    try {
      const result = await invokeOrProxy<unknown>('skill_manage', { action, ...params });
      return typeof result === 'string' ? result : JSON.stringify(result);
    } catch (error) {
      return `Error: ${error}`;
    }
  }

  /**
   * Rate a skill (for self-evolution feedback)
   */
  async rateSkill(request: SkillRatingRequest): Promise<void> {
    await this.manageSkill('rate', {
      name: request.skillName,
      rating: request.rating,
    });
  }

  /**
   * Install built-in skills
   */
  async installBuiltinSkills(): Promise<string> {
    return this.manageSkill('install_builtins', {});
  }

  /**
   * Trigger self-improvement for a skill
   */
  async selfImprove(name: string, feedback: string): Promise<string> {
    return this.manageSkill('self_improve', { name, feedback });
  }

  /**
   * Sync skills with remote hub
   */
  async syncSkills(): Promise<string> {
    return this.manageSkill('sync', {});
  }
}

// ---------------------------------------------------------------------------
// Singleton Instance
// ---------------------------------------------------------------------------

export const skillInvoker = new SkillInvoker();

// ---------------------------------------------------------------------------
// Convenience Functions
// ---------------------------------------------------------------------------

/**
 * Quick invoke a skill by name
 * Supports both simple and context-aware invocations
 */
export async function invokeSkill(
  name: string,
  params?: Record<string, unknown>,
  context?: SkillContext,
): Promise<SkillExecutionResult> {
  return skillInvoker.invokeSkill({
    skillName: name,
    parameters: params || {},
    context: context || { cwd: '.' },
  });
}

/**
 * Quick create a skill from natural language
 */
export async function createSkill(
  name: string,
  description: string,
  naturalSpec: string,
): Promise<CreateSkillResult> {
  return skillInvoker.createSkill({
    name,
    description,
    naturalSpec,
  });
}

/**
 * Get evolution triggers configuration
 */
export const DEFAULT_EVOLUTION_TRIGGERS: EvolutionTriggers = {
  failureRateThreshold: 0.15,    // 15% failure rate triggers evolution
  minExecutions: 10,             // Need at least 10 runs before considering evolution
  errorPatternThreshold: 3,      // Same error 3 times triggers evolution
  minRatingThreshold: 2.5,       // Rating below 2.5/5 triggers evolution
  cooldownSeconds: 3600,         // 1 hour between evolutions
};
