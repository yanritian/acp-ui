// Capability Evolver - Self-evolving Skill system
// Pre-execution hook: check missing skills -> auto-generate
// Post-execution hook: extract patterns -> evolve skills

import { ref, computed, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export interface EvolutionEvent {
  id?: string;
  type: 'skill_generated' | 'skill_evolved' | 'pattern_extracted' | 'fitness_evaluated';
  timestamp?: number;
  skillId?: string;
  skillName?: string;
  details: Record<string, unknown>;
  fitnessScore?: number;
}

export interface SkillPattern {
  id: string;
  name: string;
  operations: string[];       // High-frequency operations
  frequency: number;          // How often observed
  contextPatterns: string[];  // When this pattern applies
  suggestedSkill: string;     // Name for the evolved skill
}

export interface FitnessScore {
  skillId: string;
  successRate: number;        // 0-1
  avgDuration: number;        // ms
  userSatisfaction: number;   // 0-1 (from feedback)
  tokenEfficiency: number;    // 0-1
  compositeScore: number;     // Weighted average
}

export class CapabilityEvolver {
  private evolutionLog: Ref<EvolutionEvent[]> = ref([]);
  private patterns: Ref<SkillPattern[]> = ref([]);
  private fitnessScores: Ref<FitnessScore[]> = ref([]);
  private pendingSkills: Ref<string[]> = ref([]);

  // State for UI
  public isEvolutionEnabled: Ref<boolean> = ref(true);
  public lastEvolutionTime: Ref<number> = ref(0);
  public evolutionCount: Ref<number> = ref(0);

  // Pre-execution hook: Check if required skills are missing
  async preExecutionHook(task: {
    description: string;
    requiredCapabilities?: string[];
    context?: Record<string, unknown>;
  }): Promise<{
    skillsLoaded: string[];
    skillsGenerated: string[];
    ready: boolean;
  }> {
    if (!this.isEvolutionEnabled.value) {
      return { skillsLoaded: [], skillsGenerated: [], ready: true };
    }

    // Analyze task to determine required skills
    const requiredSkills = task.requiredCapabilities || await this.analyzeRequiredSkills(task.description);

    // Get available skills from skill invoker
    const availableSkills = await this.listAvailableSkills();

    // Find missing skills
    const missing = requiredSkills.filter(s => !availableSkills.includes(s));

    const generated: string[] = [];

    // Auto-generate missing skills
    for (const skillName of missing) {
      try {
        const newSkill = await this.generateSkill(skillName, task.context || {});
        generated.push(newSkill.id);
        this.logEvolution({
          type: 'skill_generated',
          skillId: newSkill.id,
          skillName: skillName,
          details: { context: task.context },
        });
      } catch (e) {
        console.warn('[CapabilityEvolver] Failed to generate skill:', skillName, e);
      }
    }

    return {
      skillsLoaded: requiredSkills,
      skillsGenerated: generated,
      ready: missing.length === 0 || generated.length === missing.length,
    };
  }

  // Post-execution hook: Extract patterns from successful execution
  async postExecutionHook(result: {
    success: boolean;
    taskDescription: string;
    operations: string[];
    duration: number;
    tokensUsed: number;
    feedback?: 'positive' | 'negative' | 'neutral';
  }): Promise<void> {
    if (!this.isEvolutionEnabled.value || !result.success) {
      return;
    }

    // Extract operation patterns
    const pattern = await this.extractOperationPattern(result);

    if (pattern && pattern.frequency >= 3) {
      // High-frequency pattern - evolve into skill
      const evolvedSkill = await this.evolveSkill(pattern);
      this.logEvolution({
        type: 'skill_evolved',
        skillId: evolvedSkill.id,
        skillName: pattern.suggestedSkill,
        details: { pattern },
        fitnessScore: 0.8, // Initial fitness for evolved skill
      });
    }

    // Update fitness scores for used skills
    await this.updateFitnessScores(result);

    this.lastEvolutionTime.value = Date.now();
  }

  // Analyze task to determine required skills
  private async analyzeRequiredSkills(description: string): Promise<string[]> {
    // Use NLP-like pattern matching
    const patterns: { pattern: RegExp; skill: string }[] = [
      { pattern: /review|analyze|audit/i, skill: 'code-review' },
      { pattern: /test|unit|integration/i, skill: 'testing' },
      { pattern: /refactor|clean|optimize/i, skill: 'refactoring' },
      { pattern: /document|write docs/i, skill: 'documentation' },
      { pattern: /debug|fix|repair/i, skill: 'debugging' },
      { pattern: /security|audit|vulnerability/i, skill: 'security-review' },
      { pattern: /deploy|release|publish/i, skill: 'deployment' },
      { pattern: /design|architect|plan/i, skill: 'architecture' },
    ];

    const skills: string[] = [];
    for (const { pattern, skill } of patterns) {
      if (pattern.test(description)) {
        skills.push(skill);
      }
    }

    // If no patterns match, add generic problem-solving skill
    if (skills.length === 0) {
      skills.push('problem-solving');
    }

    return skills;
  }

  // List available skills
  private async listAvailableSkills(): Promise<string[]> {
    try {
      const skills = await invokeOrProxy('skill_list_available');
      return (skills as string[]) || [];
    } catch (e) {
      return [];
    }
  }

  // Generate new skill
  private async generateSkill(
    skillName: string,
    context: Record<string, unknown>
  ): Promise<{ id: string; name: string }> {
    // Call skill generation API
    const result = await invokeOrProxy('skill_generate', {
      name: skillName,
      context,
      auto_register: true,
    });

    return result as { id: string; name: string };
  }

  // Extract operation pattern from result
  private async extractOperationPattern(result: {
    taskDescription: string;
    operations: string[];
    duration: number;
  }): Promise<SkillPattern | null> {
    // Count operation frequencies
    const opCounts = new Map<string, number>();
    for (const op of result.operations) {
      opCounts.set(op, (opCounts.get(op) || 0) + 1);
    }

    // Find high-frequency operations
    const highFreqOps = Array.from(opCounts.entries())
      .filter(([_, count]) => count >= 3)
      .map(([op]) => op);

    if (highFreqOps.length < 2) {
      return null; // Not enough pattern
    }

    // Generate skill name from pattern
    const suggestedSkill = `auto-${highFreqOps.slice(0, 2).join('-')}`;

    return {
      id: crypto.randomUUID(),
      name: suggestedSkill,
      operations: highFreqOps,
      frequency: Math.max(...Array.from(opCounts.values())),
      contextPatterns: [result.taskDescription.slice(0, 50)],
      suggestedSkill,
    };
  }

  // Evolve pattern into skill
  private async evolveSkill(pattern: SkillPattern): Promise<{ id: string; name: string }> {
    const result = await invokeOrProxy('skill_evolve', {
      pattern,
      auto_register: true,
    });

    return result as { id: string; name: string };
  }

  // Update fitness scores
  private async updateFitnessScores(result: {
    operations: string[];
    duration: number;
    tokensUsed: number;
    feedback?: 'positive' | 'negative' | 'neutral';
  }): Promise<void> {
    // For each operation, update fitness
    for (const op of result.operations) {
      const existing = this.fitnessScores.value.find(s => s.skillId === op);

      if (existing) {
        // Update existing score
        existing.avgDuration = (existing.avgDuration + result.duration) / 2;
        existing.tokenEfficiency = Math.min(existing.tokenEfficiency + 0.1, 1);
        if (result.feedback === 'positive') {
          existing.userSatisfaction = Math.min(existing.userSatisfaction + 0.1, 1);
        }
        existing.compositeScore = this.calculateCompositeScore(existing);
      } else {
        // Create new score entry
        this.fitnessScores.value.push({
          skillId: op,
          successRate: 1,
          avgDuration: result.duration,
          userSatisfaction: result.feedback === 'positive' ? 0.8 : 0.5,
          tokenEfficiency: 0.5,
          compositeScore: 0.6,
        });
      }
    }
  }

  // Calculate composite fitness score
  private calculateCompositeScore(score: FitnessScore): number {
    // Clamp duration component to [0, 1] so long durations don't produce negative values
    const durationComponent = Math.max(0, 1 - score.avgDuration / 10000);
    const raw = (
      score.successRate * 0.3 +
      durationComponent * 0.2 +
      score.userSatisfaction * 0.25 +
      score.tokenEfficiency * 0.25
    );
    // Clamp final score to [0, 1]
    return Math.max(0, Math.min(1, raw));
  }

  // Log evolution event
  private logEvolution(event: EvolutionEvent): void {
    this.evolutionLog.value.push({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      ...event,
    });
    this.evolutionCount.value++;
  }

  // Get evolution log
  getEvolutionLog(): EvolutionEvent[] {
    return this.evolutionLog.value;
  }

  // Get patterns
  getPatterns(): SkillPattern[] {
    return this.patterns.value;
  }

  // Get fitness scores
  getFitnessScores(): FitnessScore[] {
    return this.fitnessScores.value;
  }

  // Enable/disable evolution
  setEvolutionEnabled(enabled: boolean): void {
    this.isEvolutionEnabled.value = enabled;
  }

  // Trigger manual evolution
  async triggerEvolution(): Promise<EvolutionEvent[]> {
    // Analyze all patterns and evolve high-frequency ones
    const events: EvolutionEvent[] = [];

    for (const pattern of this.patterns.value) {
      if (pattern.frequency >= 5) {
        const evolved = await this.evolveSkill(pattern);
        events.push({
          id: crypto.randomUUID(),
          type: 'skill_evolved',
          timestamp: Date.now(),
          skillId: evolved.id,
          skillName: evolved.name,
          details: { pattern },
        });
      }
    }

    return events;
  }
}

// Singleton instance
let capabilityEvolverInstance: CapabilityEvolver | null = null;

export function useCapabilityEvolver(): CapabilityEvolver {
  if (!capabilityEvolverInstance) {
    capabilityEvolverInstance = new CapabilityEvolver();
  }
  return capabilityEvolverInstance;
}