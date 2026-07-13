/**
 * Skill System Types - OpenClacky-inspired Skill Meta-Tool Architecture
 *
 * Core concepts:
 * - A small set of immutable skills with concrete backend handlers
 * - invoke_skill meta-tool for capability delegation
 * - Self-evolving skills based on execution feedback
 */

// ---------------------------------------------------------------------------
// Skill Types
// ---------------------------------------------------------------------------

export interface Skill {
  name: string;
  content: string;           // Markdown execution instructions
  category?: string;
  description?: string;
  executionPattern: SkillExecutionPattern;
  version: SkillVersion;
  executionStats?: SkillExecutionStats;
  tags: string[];
  author?: string;
}

export type SkillExecutionPattern =
  | 'sequential'    // Execute steps in order
  | 'parallel'      // Multiple independent steps at once
  | 'conditional'   // Decide next step based on previous result
  | 'loop_until'    // Repeat until condition satisfied
  | 'interactive';  // Requires user input

export interface SkillVersion {
  semver: string;           // e.g., "1.0.0"
  contentHash: string;
  generation: number;       // Number of improvements since creation
  lastEvolvedAt?: string;
  lastEvolutionReason?: string;
}

export interface SkillExecutionStats {
  totalExecutions: number;
  successCount: number;
  failureCount: number;
  avgDurationMs: number;
  avgRating: number;        // 1-5 scale
  commonErrors: ErrorPattern[];
  lastExecutedAt?: string;
}

export interface ErrorPattern {
  pattern: string;
  count: number;
  suggestedFix?: string;
}

// ---------------------------------------------------------------------------
// Skill Invocation Types
// ---------------------------------------------------------------------------

export interface SkillInvocation {
  skillName: string;
  parameters: Record<string, unknown>;
  context?: SkillContext;
}

export interface SkillContext {
  sessionId?: string;
  agentName?: string;
  taskId?: string;
  cwd: string;
}

export interface SkillExecutionResult {
  success: boolean;
  output: string;
  error?: string;
  durationMs: number;
  toolCallsCount: number;
  evolved: boolean;         // Whether self-evolution was triggered
  evolutionSummary?: string;
}

// ---------------------------------------------------------------------------
// Evolution Types
// ---------------------------------------------------------------------------

export interface EvolutionTriggers {
  failureRateThreshold: number;    // Default: 0.15 (15%)
  minExecutions: number;            // Default: 10
  errorPatternThreshold: number;    // Default: 3
  minRatingThreshold: number;       // Default: 2.5
  cooldownSeconds: number;          // Default: 3600 (1 hour)
}

export type EvolutionReason =
  | { type: 'high_failure_rate'; rate: number }
  | { type: 'recurring_error'; pattern: ErrorPattern }
  | { type: 'low_rating'; rating: number }
  | { type: 'user_requested' };

export interface EvolutionSuggestion {
  reason: EvolutionReason;
  priority: number;
  description: string;
}

// ---------------------------------------------------------------------------
// Core Skills Registry (only skills with a concrete backend handler)
// ---------------------------------------------------------------------------

export const CORE_SKILLS: readonly string[] = [
  'godot-analyze',
  'godot-codegen',
];

// ---------------------------------------------------------------------------
// Skill Meta Interface
// ---------------------------------------------------------------------------

export interface SkillMeta {
  name: string;
  category?: string;
  description?: string;
  version?: string;
  generation?: number;
}

// ---------------------------------------------------------------------------
// Skill Creation Types
// ---------------------------------------------------------------------------

export interface CreateSkillRequest {
  name: string;           // kebab-case, e.g., 'my-custom-task'
  description: string;    // Brief description
  naturalSpec: string;    // Natural language specification
}

export interface CreateSkillResult {
  created: boolean;
  name: string;
  message: string;
}

// ---------------------------------------------------------------------------
// Skill Rating Types
// ---------------------------------------------------------------------------

export interface SkillRatingRequest {
  skillName: string;
  rating: number;         // 1-5
}

// ---------------------------------------------------------------------------
// Skill Version History Types
// ---------------------------------------------------------------------------

export interface SkillVersionHistory {
  skillName: string;
  versions: SkillVersionSnapshot[];
}

export interface SkillVersionSnapshot {
  semver: string;
  content: string;
  evolvedAt: string;
  reason: string;
  generation: number;
}
