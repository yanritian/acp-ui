/**
 * Skill System - OpenClacky-inspired Skill Meta-Tool Architecture
 *
 * This module implements the core skill system with:
 * - invoke_skill meta-tool (single entry point for all capabilities)
 * - Self-evolving skills based on execution feedback
 * - Natural language skill creation
 * - Version history and rollback
 * - Development flow skill chain (Phase 2)
 *
 * Architecture inspired by OpenClacky:
 * - 12-16 immutable core tools
 * - All capabilities delegated through invoke_skill
 * - Skills auto-improve after runs based on context and results
 */

export * from './types';
export * from './skill-invoker';
export * from './dev-flow-skills';

// Re-export convenience functions
export {
  skillInvoker,
  invokeSkill,
  createSkill,
  DEFAULT_EVOLUTION_TRIGGERS,
} from './skill-invoker';

// Re-export core skills list
export { CORE_SKILLS } from './types';

// Re-export development flow
export {
  DEVELOPMENT_FLOW_SKILLS,
  DEVELOPMENT_FLOW_STAGES,
  DevelopmentFlowOrchestrator,
  executeDevelopmentFlow,
  getFlowStatus,
  type FlowStage,
  type FlowContext,
  type FlowError,
  type StageExecution,
  type AnalysisResult,
  type PlanResult,
  type CodeResult,
  type ReviewResult,
  type AdjustmentsResult,
  type TestResult,
  type DocumentationResult,
} from './dev-flow-skills';