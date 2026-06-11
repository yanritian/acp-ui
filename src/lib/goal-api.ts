// Goal-Driven Architecture — TypeScript Types + Tauri API (RFC-001)
//
// Mirrors the Rust Goal data structures for frontend integration.
// These types align with src-tauri/src/goal.rs.

import { invokeOrProxy } from './host';

// ---------------------------------------------------------------------------
// Goal
// ---------------------------------------------------------------------------

export interface Goal {
  id: string
  description: string
  completionCondition: CompletionCondition
  evaluator: Evaluator
  assignedWorker: string | null
  status: GoalStatus
  iterations: IterationRecord[]
  maxIterations: number
  tokenBudget: number | null
  tokenUsed: number
  createdAt: number
  startedAt: number | null
  convergedAt: number | null
  parentId: string | null
  dependencies: string[]
}

// ---------------------------------------------------------------------------
// CompletionCondition
// ---------------------------------------------------------------------------

export type CompletionCondition =
  | { type: 'command_success'; command: string; expectedExitCode: number }
  | { type: 'output_contains'; text: string; caseSensitive: boolean }
  | { type: 'output_matches'; pattern: string }
  | { type: 'all'; conditions: CompletionCondition[] }
  | { type: 'any'; conditions: CompletionCondition[] }
  | { type: 'file_check'; path: string; mustExist: boolean; contentContains: string | null }
  | { type: 'http_health_check'; url: string; expectedStatus: number }
  | { type: 'queen_judgment'; criteria: string }

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

export type Evaluator =
  | { type: 'auto' }
  | { type: 'queen'; queenWorkerId: string }
  | { type: 'adversarial'; primary: string; adversary: string }
  | {
      type: 'hybrid'
      autoConditions: CompletionCondition[]
      queenCriteria: string
    }

// ---------------------------------------------------------------------------
// GoalStatus
// ---------------------------------------------------------------------------

export type GoalStatus =
  | { status: 'pending' }
  | { status: 'active' }
  | { status: 'evaluating' }
  | { status: 'converged' }
  | { status: 'iterating'; feedback: string }
  | { status: 'failed'; reason: string }
  | { status: 'budget_exhausted' }
  | { status: 'cancelled' }

// ---------------------------------------------------------------------------
// IterationRecord
// ---------------------------------------------------------------------------

export interface IterationRecord {
  iteration: number
  workerOutput: string
  evaluation: EvaluationResult
  feedback: string | null
  tokensUsed: number
  timestamp: number
  durationMs: number
}

export interface EvaluationResult {
  passed: boolean
  explanation: string
  details: ConditionResult[]
}

export interface ConditionResult {
  description: string
  passed: boolean
  evidence: string
}

// ---------------------------------------------------------------------------
// GoalGraph Summary
// ---------------------------------------------------------------------------

export interface GoalGraphSummary {
  total: number
  pending: number
  active: number
  evaluating: number
  converged: number
  iterating: number
  failed: number
  budget_exhausted: number
  cancelled: number
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/** Check if a GoalStatus is terminal (no more iterations possible) */
export function isTerminalStatus(status: GoalStatus): boolean {
  return ['converged', 'failed', 'budget_exhausted', 'cancelled'].includes(status.status)
}

/** Get a human-readable label for a goal status */
export function statusLabel(status: GoalStatus): string {
  const labels: Record<string, string> = {
    pending: 'Pending',
    active: 'Active',
    evaluating: 'Evaluating',
    converged: 'Converged',
    iterating: 'Iterating',
    failed: 'Failed',
    budget_exhausted: 'Budget Exhausted',
    cancelled: 'Cancelled',
  }
  return labels[status.status] || status.status
}

/** Get a color class for a goal status (Tailwind) */
export function statusColor(status: GoalStatus): string {
  const colors: Record<string, string> = {
    pending: 'text-gray-500',
    active: 'text-blue-500',
    evaluating: 'text-purple-500',
    converged: 'text-green-500',
    iterating: 'text-yellow-500',
    failed: 'text-red-500',
    budget_exhausted: 'text-orange-500',
    cancelled: 'text-gray-400',
  }
  return colors[status.status] || 'text-gray-500'
}

/** Create a simple file-check goal */
export function createFileGoal(
  id: string,
  description: string,
  filePath: string,
  contentContains?: string
): Goal {
  return {
    id,
    description,
    completionCondition: {
      type: 'file_check',
      path: filePath,
      mustExist: true,
      contentContains: contentContains ?? null,
    },
    evaluator: { type: 'auto' },
    assignedWorker: null,
    status: { status: 'pending' },
    iterations: [],
    maxIterations: 5,
    tokenBudget: null,
    tokenUsed: 0,
    createdAt: Date.now(),
    startedAt: null,
    convergedAt: null,
    parentId: null,
    dependencies: [],
  }
}

/** Create a simple command-success goal */
export function createCommandGoal(
  id: string,
  description: string,
  command: string,
  expectedExitCode: number = 0
): Goal {
  return {
    id,
    description,
    completionCondition: {
      type: 'command_success',
      command,
      expectedExitCode,
    },
    evaluator: { type: 'auto' },
    assignedWorker: null,
    status: { status: 'pending' },
    iterations: [],
    maxIterations: 3,
    tokenBudget: null,
    tokenUsed: 0,
    createdAt: Date.now(),
    startedAt: null,
    convergedAt: null,
    parentId: null,
    dependencies: [],
  }
}

// ---------------------------------------------------------------------------
// Tauri API Functions
// ---------------------------------------------------------------------------

/** Submit a new goal to the GoalGraph */
export async function goalSubmit(goal: Goal): Promise<void> {
  return invokeOrProxy('goal_submit', { goal })
}

/** Get goal status by ID */
export async function goalGetStatus(id: string): Promise<Goal> {
  return invokeOrProxy<Goal>('goal_get_status', { id })
}

/** Cancel a goal */
export async function goalCancel(id: string): Promise<void> {
  return invokeOrProxy('goal_cancel', { id })
}

/** Get graph summary statistics */
export async function goalGetGraphSummary(): Promise<GoalGraphSummary> {
  return invokeOrProxy<GoalGraphSummary>('goal_get_graph_summary')
}

/** List all goals */
export async function goalList(): Promise<Goal[]> {
  return invokeOrProxy<Goal[]>('goal_list')
}

/** Get goals ready to execute (dependencies converged) */
export async function goalGetReady(): Promise<Goal[]> {
  return invokeOrProxy<Goal[]>('goal_get_ready')
}

/** Assign a worker to a goal */
export async function goalAssignWorker(id: string, workerId: string): Promise<void> {
  return invokeOrProxy('goal_assign_worker', { id, workerId })
}

/** Add an iteration record to a goal */
export async function goalAddIteration(id: string, record: IterationRecord): Promise<void> {
  return invokeOrProxy('goal_add_iteration', { id, record })
}
