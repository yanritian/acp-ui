/**
 * Skill System - OpenClacky-inspired Skill Meta-Tool Architecture
 *
 * This module implements the core skill system with:
 * - invoke_skill meta-tool (single entry point for all capabilities)
 * - Self-evolving skills based on execution feedback
 * - Natural language skill creation
 * - Version history and rollback
 *
 * Architecture inspired by OpenClacky:
 * - 12-16 immutable core tools
 * - All capabilities delegated through invoke_skill
 * - Skills auto-improve after runs based on context and results
 */

export * from './types';
export * from './skill-invoker';

// Re-export convenience functions
export {
  skillInvoker,
  invokeSkill,
  createSkill,
  DEFAULT_EVOLUTION_TRIGGERS,
} from './skill-invoker';

// Re-export core skills list
export { CORE_SKILLS } from './types';