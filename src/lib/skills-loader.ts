/**
 * Skills Loader - dynamically loads and manages skills for agents
 */

import { ref, computed } from 'vue';

interface Skill {
  id: string;
  name: string;
  description: string;
  category: string;
  handler: string; // Path to skill handler module
  enabled: boolean;
}

interface SkillCall {
  skillId: string;
  agentId: string;
  params: Record<string, unknown>;
}

interface SkillResult {
  success: boolean;
  output?: unknown;
  error?: string;
}

class SkillsLoader {
  private skills: Map<string, Skill> = new Map();
  private agentSkills: Map<string, Set<string>> = new Map(); // agent_id -> skill_ids
  private skillHandlers: Map<string, (...args: unknown[]) => Promise<unknown>> = new Map();

  /**
   * Register a skill
   */
  registerSkill(skill: Skill): void {
    this.skills.set(skill.id, skill);
    console.log(`Registered skill: ${skill.name}`);
  }

  /**
   * Load skill handler dynamically
   */
  async loadSkillHandler(skillId: string, handlerPath: string): Promise<void> {
    try {
      // Dynamic import of skill handler
      const module = await import(/* @vite-ignore */ handlerPath);
      if (module.default && typeof module.default === 'function') {
        this.skillHandlers.set(skillId, module.default);
        console.log(`Loaded skill handler: ${skillId}`);
      } else if (module.handler && typeof module.handler === 'function') {
        this.skillHandlers.set(skillId, module.handler);
        console.log(`Loaded skill handler: ${skillId}`);
      } else {
        console.warn(`Skill handler not found in module: ${handlerPath}`);
      }
    } catch (e) {
      console.error(`Failed to load skill handler: ${handlerPath}`, e);
      // Store fallback handler that returns error
      this.skillHandlers.set(skillId, async () => {
        throw new Error(`Skill handler not available: ${handlerPath}`);
      });
    }
  }

  /**
   * Register skills for an agent
   */
  registerAgentSkills(agentId: string, skillIds: string[]): void {
    const skillsSet = new Set<string>();
    for (const skillId of skillIds) {
      if (this.skills.has(skillId)) {
        skillsSet.add(skillId);
      } else {
        console.warn(`Skill '${skillId}' not found, skipping for agent '${agentId}'`);
      }
    }
    this.agentSkills.set(agentId, skillsSet);
    console.log(`Registered ${skillsSet.size} skills for agent: ${agentId}`);
  }

  /**
   * Unregister all skills for an agent
   */
  unregisterAgentSkills(agentId: string): void {
    this.agentSkills.delete(agentId);
    console.log(`Unregistered all skills for agent: ${agentId}`);
  }

  /**
   * Validate agent can call skill (isolation check)
   */
  validateAccess(agentId: string, skillId: string): boolean {
    const agentSkills = this.agentSkills.get(agentId);
    if (!agentSkills) {
      console.warn(`Agent '${agentId}' has no skills registered`);
      return false;
    }
    if (!agentSkills.has(skillId)) {
      console.warn(`Agent '${agentId}' cannot access skill '${skillId}'`);
      return false;
    }
    return true;
  }

  /**
   * Call a skill for an agent
   */
  async callSkill(call: SkillCall): Promise<SkillResult> {
    // Validate access
    if (!this.validateAccess(call.agentId, call.skillId)) {
      return {
        success: false,
        error: `Agent '${call.agentId}' cannot access skill '${call.skillId}'`
      };
    }

    // Get skill info
    const skill = this.skills.get(call.skillId);
    if (!skill) {
      return {
        success: false,
        error: `Skill '${call.skillId}' not found`
      };
    }

    // Get handler
    const handler = this.skillHandlers.get(call.skillId);
    if (!handler) {
      // Try to load handler if not loaded
      if (skill.handler) {
        await this.loadSkillHandler(call.skillId, skill.handler);
        const loadedHandler = this.skillHandlers.get(call.skillId);
        if (!loadedHandler) {
          return {
            success: false,
            error: `Skill handler not available`
          };
        }
      } else {
        return {
          success: false,
          error: `Skill handler path not specified`
        };
      }
    }

    // Execute skill
    try {
      const result = await this.skillHandlers.get(call.skillId)!(call.params);
      return {
        success: true,
        output: result
      };
    } catch (e) {
      return {
        success: false,
        error: String(e)
      };
    }
  }

  /**
   * Get list of skills for an agent
   */
  getAgentSkills(agentId: string): Skill[] {
    const skillIds = this.agentSkills.get(agentId) || new Set();
    return Array.from(skillIds)
      .map(id => this.skills.get(id))
      .filter((s): s is Skill => s !== undefined);
  }

  /**
   * Get all registered skills
   */
  getAllSkills(): Skill[] {
    return Array.from(this.skills.values());
  }

  /**
   * Get skill by ID
   */
  getSkill(skillId: string): Skill | undefined {
    return this.skills.get(skillId);
  }

  /**
   * Check if skill exists
   */
  hasSkill(skillId: string): boolean {
    return this.skills.has(skillId);
  }

  /**
   * List skills by category
   */
  getSkillsByCategory(category: string): Skill[] {
    return Array.from(this.skills.values())
      .filter(s => s.category === category);
  }
}

// Singleton instance
export const skillsLoader = new SkillsLoader();

// Default skills registration
export function registerDefaultSkills(): void {
  const defaultSkills: Skill[] = [
    {
      id: 'code-review',
      name: 'Code Review',
      description: 'Review code for quality, security, and best practices',
      category: 'qa',
      handler: './skills/code-review.ts',
      enabled: true
    },
    {
      id: 'test-validator',
      name: 'Test Validator',
      description: 'Run tests and validate coverage',
      category: 'qa',
      handler: './skills/test-validator.ts',
      enabled: true
    },
    {
      id: 'file-read',
      name: 'File Read',
      description: 'Read file contents',
      category: 'file',
      handler: './skills/file-read.ts',
      enabled: true
    },
    {
      id: 'file-write',
      name: 'File Write',
      description: 'Write file contents',
      category: 'file',
      handler: './skills/file-write.ts',
      enabled: true
    },
    {
      id: 'bash-exec',
      name: 'Bash Execute',
      description: 'Execute bash commands',
      category: 'exec',
      handler: './skills/bash-exec.ts',
      enabled: true
    },
    {
      id: 'browser-control',
      name: 'Browser Control',
      description: 'Control browser via CDP',
      category: 'browser',
      handler: './skills/browser-control.ts',
      enabled: true
    }
  ];

  for (const skill of defaultSkills) {
    skillsLoader.registerSkill(skill);
  }
}

// Vue store for skills management
export const useSkillsStore = () => {
  const allSkills = ref<Skill[]>([]);
  const agentSkillsMap = ref<Map<string, Skill[]>>(new Map());

  const refreshSkills = () => {
    allSkills.value = skillsLoader.getAllSkills();
  };

  const getSkillsForAgent = (agentId: string): Skill[] => {
    return skillsLoader.getAgentSkills(agentId);
  };

  const canAgentUseSkill = (agentId: string, skillId: string): boolean => {
    return skillsLoader.validateAccess(agentId, skillId);
  };

  const callSkill = async (agentId: string, skillId: string, params: Record<string, unknown>): Promise<SkillResult> => {
    return skillsLoader.callSkill({
      skillId,
      agentId,
      params
    });
  };

  return {
    allSkills: computed(() => allSkills.value),
    agentSkillsMap: computed(() => agentSkillsMap.value),
    refreshSkills,
    getSkillsForAgent,
    canAgentUseSkill,
    callSkill
  };
};