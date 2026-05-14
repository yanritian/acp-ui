/**
 * Agent Matcher - matches tasks to agents based on capabilities
 */

interface AgentCapabilities {
  agentId: string;
  capabilities: string[];
  currentLoad: number;
  maxLoad: number;
  performanceScore: number;
  specialization: string[];
}

interface TaskRequirements {
  requiredCapabilities: string[];
  preferredCapabilities: string[];
  priority: 'low' | 'medium' | 'high' | 'critical';
  estimatedLoad: number;
  preferredAgentType?: string;
}

interface MatchResult {
  agentId: string;
  score: number;
  jaccardSimilarity: number;
  capabilityScore: number;
  loadScore: number;
  performanceScore: number;
  matchedCapabilities: string[];
  missingCapabilities: string[];
}

interface MatcherConfig {
  minSimilarity: number;
  loadWeight: number;
  capabilityWeight: number;
  performanceWeight: number;
  overloadThreshold: number;
}

class AgentMatcher {
  private agents: Map<string, AgentCapabilities> = new Map();
  private config: MatcherConfig;

  constructor(config?: Partial<MatcherConfig>) {
    this.config = {
      minSimilarity: 0.3,
      loadWeight: 0.25,
      capabilityWeight: 0.45,
      performanceWeight: 0.3,
      overloadThreshold: 0.8,
      ...config,
    };
  }

  /**
   * Register agent capabilities
   */
  registerAgent(agent: AgentCapabilities): void {
    this.agents.set(agent.agentId, agent);
  }

  /**
   * Unregister agent
   */
  unregisterAgent(agentId: string): void {
    this.agents.delete(agentId);
  }

  /**
   * Update agent load
   */
  updateAgentLoad(agentId: string, currentLoad: number): void {
    const agent = this.agents.get(agentId);
    if (agent) {
      agent.currentLoad = currentLoad;
    }
  }

  /**
   * Find best agent for task
   */
  findBestAgent(requirements: TaskRequirements): MatchResult | null {
    const candidates = this.findCandidates(requirements);

    if (candidates.length === 0) {
      return null;
    }

    // Score and rank candidates
    const scoredCandidates = candidates.map(c => this.scoreCandidate(c, requirements));

    // Sort by score descending
    scoredCandidates.sort((a, b) => b.score - a.score);

    // Return best match
    return scoredCandidates[0];
  }

  /**
   * Find all candidate agents that meet minimum requirements
   */
  findCandidates(requirements: TaskRequirements): AgentCapabilities[] {
    const candidates: AgentCapabilities[] = [];

    for (const agent of this.agents.values()) {
      // Check if agent has all required capabilities
      const hasRequired = requirements.requiredCapabilities.every(
        cap => agent.capabilities.includes(cap) || agent.specialization.includes(cap)
      );

      if (!hasRequired) {
        continue;
      }

      // Check if agent is not overloaded
      const loadRatio = agent.currentLoad / agent.maxLoad;
      if (loadRatio >= this.config.overloadThreshold) {
        continue;
      }

      // Check preferred agent type if specified
      if (requirements.preferredAgentType) {
        const hasType = agent.capabilities.includes(requirements.preferredAgentType) ||
          agent.specialization.includes(requirements.preferredAgentType);
        if (!hasType) {
          continue;
        }
      }

      candidates.push(agent);
    }

    return candidates;
  }

  /**
   * Calculate Jaccard similarity between two sets
   */
  calculateJaccardSimilarity(setA: string[], setB: string[]): number {
    if (setA.length === 0 && setB.length === 0) {
      return 1.0;
    }

    const intersection = setA.filter(x => setB.includes(x));
    const union = [...new Set([...setA, ...setB])];

    return intersection.length / union.length;
  }

  /**
   * Calculate capability vector match
   */
  calculateCapabilityMatch(agent: AgentCapabilities, requirements: TaskRequirements): {
    matched: string[];
    missing: string[];
    score: number;
  } {
    const allRequired = [...requirements.requiredCapabilities, ...requirements.preferredCapabilities];

    const matched = allRequired.filter(
      cap => agent.capabilities.includes(cap) || agent.specialization.includes(cap)
    );

    const missing = allRequired.filter(
      cap => !agent.capabilities.includes(cap) && !agent.specialization.includes(cap)
    );

    // Required capabilities are mandatory, preferred are bonus
    const requiredMatched = requirements.requiredCapabilities.filter(
      cap => agent.capabilities.includes(cap) || agent.specialization.includes(cap)
    ).length;

    const preferredMatched = requirements.preferredCapabilities.filter(
      cap => agent.capabilities.includes(cap) || agent.specialization.includes(cap)
    ).length;

    // Score: required capabilities are weighted more
    const requiredWeight = requirements.requiredCapabilities.length > 0 ? 0.7 : 0;
    const preferredWeight = requirements.preferredCapabilities.length > 0 ? 0.3 : 0;

    const score = (
      (requiredMatched / requirements.requiredCapabilities.length || 0) * requiredWeight +
      (preferredMatched / requirements.preferredCapabilities.length || 0) * preferredWeight
    );

    return { matched, missing, score };
  }

  /**
   * Calculate load score (prefer less loaded agents)
   */
  calculateLoadScore(agent: AgentCapabilities): number {
    const loadRatio = agent.currentLoad / agent.maxLoad;

    // Score inversely proportional to load
    // Full score when load is 0, 0 score when at threshold
    const normalizedLoad = loadRatio / this.config.overloadThreshold;
    return Math.max(0, 1 - normalizedLoad);
  }

  /**
   * Score candidate agent
   */
  scoreCandidate(agent: AgentCapabilities, requirements: TaskRequirements): MatchResult {
    // Calculate component scores
    const allCapabilities = [...agent.capabilities, ...agent.specialization];
    const allRequired = [...requirements.requiredCapabilities, ...requirements.preferredCapabilities];

    const jaccardSimilarity = this.calculateJaccardSimilarity(allCapabilities, allRequired);
    const capabilityMatch = this.calculateCapabilityMatch(agent, requirements);
    const loadScore = this.calculateLoadScore(agent);

    // Weighted total score
    const score = (
      jaccardSimilarity * this.config.capabilityWeight +
      capabilityMatch.score * this.config.capabilityWeight +
      loadScore * this.config.loadWeight +
      (agent.performanceScore / 100) * this.config.performanceWeight
    );

    return {
      agentId: agent.agentId,
      score,
      jaccardSimilarity,
      capabilityScore: capabilityMatch.score,
      loadScore,
      performanceScore: agent.performanceScore / 100,
      matchedCapabilities: capabilityMatch.matched,
      missingCapabilities: capabilityMatch.missing,
    };
  }

  /**
   * Find agents by capability
   */
  findByCapability(capability: string): AgentCapabilities[] {
    return Array.from(this.agents.values())
      .filter(a => a.capabilities.includes(capability) || a.specialization.includes(capability));
  }

  /**
   * Get agent capabilities
   */
  getAgentCapabilities(agentId: string): AgentCapabilities | undefined {
    return this.agents.get(agentId);
  }

  /**
   * Get all registered agents
   */
  getAllAgents(): AgentCapabilities[] {
    return Array.from(this.agents.values());
  }

  /**
   * Get available agents (not overloaded)
   */
  getAvailableAgents(): AgentCapabilities[] {
    return Array.from(this.agents.values())
      .filter(a => a.currentLoad / a.maxLoad < this.config.overloadThreshold);
  }

  /**
   * Get agents sorted by load (for load balancing)
   */
  getAgentsByLoad(): AgentCapabilities[] {
    return Array.from(this.agents.values())
      .sort((a, b) => a.currentLoad - b.currentLoad);
  }

  /**
   * Calculate overall system load
   */
  getSystemLoad(): { total: number; average: number; overloaded: number } {
    const agents = Array.from(this.agents.values());

    if (agents.length === 0) {
      return { total: 0, average: 0, overloaded: 0 };
    }

    const total = agents.reduce((sum, a) => sum + a.currentLoad, 0);
    const average = total / agents.length;
    const overloaded = agents.filter(
      a => a.currentLoad / a.maxLoad >= this.config.overloadThreshold
    ).length;

    return { total, average, overloaded };
  }

  /**
   * Recommend agents for task type
   */
  recommendForTaskType(taskType: string, limit: number = 3): MatchResult[] {
    const requirements: TaskRequirements = {
      requiredCapabilities: [taskType],
      preferredCapabilities: [],
      priority: 'medium',
      estimatedLoad: 1,
    };

    const candidates = this.findCandidates(requirements);
    const scoredCandidates = candidates.map(c => this.scoreCandidate(c, requirements));

    scoredCandidates.sort((a, b) => b.score - a.score);

    return scoredCandidates.slice(0, limit);
  }

  /**
   * Check if any agent can handle task
   */
  canHandleTask(requirements: TaskRequirements): boolean {
    return this.findCandidates(requirements).length > 0;
  }

  /**
   * Get capability coverage analysis
   */
  getCapabilityCoverage(): Map<string, { agents: number; available: number }> {
    const coverage = new Map<string, { agents: number; available: number }>();

    // Collect all unique capabilities
    const allCapabilities = new Set<string>();
    for (const agent of this.agents.values()) {
      for (const cap of [...agent.capabilities, ...agent.specialization]) {
        allCapabilities.add(cap);
      }
    }

    // Count coverage for each capability
    for (const cap of allCapabilities) {
      const agents = this.findByCapability(cap);
      const available = agents.filter(
        a => a.currentLoad / a.maxLoad < this.config.overloadThreshold
      ).length;

      coverage.set(cap, { agents: agents.length, available });
    }

    return coverage;
  }

  /**
   * Generate matcher report
   */
  generateReport(): string {
    const lines: string[] = [];

    lines.push('# Agent Matcher Report');
    lines.push('\n## Registered Agents\n');

    for (const agent of this.agents.values()) {
      const loadRatio = agent.currentLoad / agent.maxLoad;
      const status = loadRatio >= this.config.overloadThreshold ? '⚠️ Overloaded' : '✅ Available';
      lines.push(`- **${agent.agentId}**: ${status}`);
      lines.push(`  - Capabilities: ${agent.capabilities.join(', ')}`);
      lines.push(`  - Specialization: ${agent.specialization.join(', ')}`);
      lines.push(`  - Load: ${agent.currentLoad}/${agent.maxLoad} (${(loadRatio * 100).toFixed(1)}%)`);
      lines.push(`  - Performance: ${agent.performanceScore}/100`);
    }

    const systemLoad = this.getSystemLoad();
    lines.push('\n## System Load\n');
    lines.push(`- Total Tasks: ${systemLoad.total}`);
    lines.push(`- Average Load: ${systemLoad.average.toFixed(2)}`);
    lines.push(`- Overloaded Agents: ${systemLoad.overloaded}`);

    lines.push('\n## Capability Coverage\n');
    const coverage = this.getCapabilityCoverage();
    for (const [cap, info] of coverage.entries()) {
      lines.push(`- ${cap}: ${info.agents} agents, ${info.available} available`);
    }

    return lines.join('\n');
  }
}

// Pre-defined agent types for registration with matcher
export const DEFAULT_AGENT_CAPABILITIES: Record<string, Partial<AgentCapabilities>> = {
  'hbuilder': {
    capabilities: ['miniprogram', 'uniapp', 'build', 'compile', 'run'],
    specialization: ['hbuilderx', 'wechat-dev'],
    performanceScore: 85,
  },
  'android': {
    capabilities: ['android', 'gradle', 'build', 'apk', 'adb', 'test'],
    specialization: ['android-studio', 'java', 'kotlin'],
    performanceScore: 90,
  },
  'browser': {
    capabilities: ['browser', 'navigate', 'click', 'evaluate', 'capture', 'automate'],
    specialization: ['chrome', 'playwright', 'puppeteer'],
    performanceScore: 88,
  },
  'code-reviewer': {
    capabilities: ['review', 'tsc', 'eslint', 'security', 'quality'],
    specialization: ['qa', 'code-analysis'],
    performanceScore: 95,
  },
  'test-validator': {
    capabilities: ['test', 'coverage', 'jest', 'vitest', 'validation'],
    specialization: ['qa', 'testing'],
    performanceScore: 92,
  },
};

export const agentMatcher = new AgentMatcher();
export type { AgentCapabilities, TaskRequirements, MatchResult, MatcherConfig };