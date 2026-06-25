// Goal API Unit Tests
//
// Tests for goal-api.ts helper functions and type validation.

import { describe, it, expect } from 'vitest'
import {
  isTerminalStatus,
  statusLabel,
  statusColor,
  createFileGoal,
  createCommandGoal,
  type GoalStatus,
  type Goal,
} from '../goal-api'

describe('goal-api helpers', () => {
  describe('isTerminalStatus', () => {
    it('returns true for converged status', () => {
      const status: GoalStatus = { status: 'converged' }
      expect(isTerminalStatus(status)).toBe(true)
    })

    it('returns true for failed status', () => {
      const status: GoalStatus = { status: 'failed', reason: 'test failure' }
      expect(isTerminalStatus(status)).toBe(true)
    })

    it('returns true for budget_exhausted status', () => {
      const status: GoalStatus = { status: 'budget_exhausted' }
      expect(isTerminalStatus(status)).toBe(true)
    })

    it('returns true for max_iter_reached status', () => {
      const status: GoalStatus = { status: 'max_iter_reached' }
      expect(isTerminalStatus(status)).toBe(true)
    })

    it('returns true for cancelled status', () => {
      const status: GoalStatus = { status: 'cancelled' }
      expect(isTerminalStatus(status)).toBe(true)
    })

    it('returns false for pending status', () => {
      const status: GoalStatus = { status: 'pending' }
      expect(isTerminalStatus(status)).toBe(false)
    })

    it('returns false for active status', () => {
      const status: GoalStatus = { status: 'active' }
      expect(isTerminalStatus(status)).toBe(false)
    })

    it('returns false for evaluating status', () => {
      const status: GoalStatus = { status: 'evaluating' }
      expect(isTerminalStatus(status)).toBe(false)
    })

    it('returns false for iterating status', () => {
      const status: GoalStatus = { status: 'iterating', feedback: 'retry needed' }
      expect(isTerminalStatus(status)).toBe(false)
    })
  })

  describe('statusLabel', () => {
    it('returns correct labels for all statuses', () => {
      expect(statusLabel({ status: 'pending' })).toBe('Pending')
      expect(statusLabel({ status: 'active' })).toBe('Active')
      expect(statusLabel({ status: 'evaluating' })).toBe('Evaluating')
      expect(statusLabel({ status: 'converged' })).toBe('Converged')
      expect(statusLabel({ status: 'iterating', feedback: '' })).toBe('Iterating')
      expect(statusLabel({ status: 'failed', reason: '' })).toBe('Failed')
      expect(statusLabel({ status: 'budget_exhausted' })).toBe('Budget Exhausted')
      expect(statusLabel({ status: 'cancelled' })).toBe('Cancelled')
    })

    it('returns the status itself for unknown statuses', () => {
      // TypeScript won't allow invalid statuses, but testing fallback behavior
      const unknownStatus = { status: 'unknown' } as unknown as GoalStatus
      expect(statusLabel(unknownStatus)).toBe('unknown')
    })
  })

  describe('statusColor', () => {
    it('returns correct Tailwind classes for statuses', () => {
      expect(statusColor({ status: 'pending' })).toBe('text-gray-500')
      expect(statusColor({ status: 'active' })).toBe('text-blue-500')
      expect(statusColor({ status: 'evaluating' })).toBe('text-purple-500')
      expect(statusColor({ status: 'converged' })).toBe('text-green-500')
      expect(statusColor({ status: 'iterating', feedback: '' })).toBe('text-yellow-500')
      expect(statusColor({ status: 'failed', reason: '' })).toBe('text-red-500')
      expect(statusColor({ status: 'budget_exhausted' })).toBe('text-orange-500')
      expect(statusColor({ status: 'cancelled' })).toBe('text-gray-400')
    })

    it('returns gray-500 for unknown statuses', () => {
      const unknownStatus = { status: 'unknown' } as unknown as GoalStatus
      expect(statusColor(unknownStatus)).toBe('text-gray-500')
    })
  })

  describe('createFileGoal', () => {
    it('creates a goal with file_check condition', () => {
      const goal = createFileGoal('goal-001', 'Check README exists', 'README.md')

      expect(goal.id).toBe('goal-001')
      expect(goal.description).toBe('Check README exists')
      expect(goal.completion_condition.type).toBe('file_check')
      if (goal.completion_condition.type === 'file_check') {
        expect(goal.completion_condition.path).toBe('README.md')
        expect(goal.completion_condition.content_contains).toBeNull()
      }
      expect(goal.status.status).toBe('pending')
      expect(goal.iteration_log).toEqual([])
      expect(goal.max_iterations).toBe(5)
      expect(goal.token_budget).toBeNull()
      expect(goal.tokens_used).toBe(0)
      expect(goal.depends_on).toEqual([])
    })

    it('creates a goal with content_contains check', () => {
      const goal = createFileGoal(
        'goal-002',
        'Check config contains API key',
        'config.json',
        'API_KEY'
      )

      expect(goal.completion_condition.type).toBe('file_check')
      if (goal.completion_condition.type === 'file_check') {
        expect(goal.completion_condition.content_contains).toBe('API_KEY')
      }
    })

    it('sets correct timestamps', () => {
      const before = Date.now()
      const goal = createFileGoal('goal-003', 'Test', 'test.txt')
      const after = Date.now()

      expect(goal.created_at).toBeGreaterThanOrEqual(before)
      expect(goal.created_at).toBeLessThanOrEqual(after)
      expect(goal.started_at).toBeNull()
      expect(goal.converged_at).toBeNull()
    })
  })

  describe('createCommandGoal', () => {
    it('creates a goal with command_success condition', () => {
      const goal = createCommandGoal('goal-004', 'Run tests', 'npm test')

      expect(goal.id).toBe('goal-004')
      expect(goal.description).toBe('Run tests')
      expect(goal.completion_condition.type).toBe('command_success')
      if (goal.completion_condition.type === 'command_success') {
        expect(goal.completion_condition.command).toBe('npm test')
        expect(goal.completion_condition.args).toEqual([])
      }
      expect(goal.status.status).toBe('pending')
      expect(goal.max_iterations).toBe(3)
    })

    it('creates a goal with custom expected exit code', () => {
      const goal = createCommandGoal('goal-005', 'Run command expecting failure', 'false', 1)

      if (goal.completion_condition.type === 'command_success') {
        expect(goal.completion_condition.args).toEqual(['1'])
      }
    })
  })
})

describe('Goal type validation', () => {
  it('validates Goal structure', () => {
    const goal: Goal = {
      id: 'test-goal',
      description: 'Test goal',
      completion_condition: { type: 'command_success', command: 'echo', args: [], cwd: null },
      evaluator: { type: 'auto' },
      executor: null,
      status: { status: 'pending' },
      iteration_log: [],
      max_iterations: 5,
      token_budget: null,
      tokens_used: 0,
      created_at: Date.now(),
      started_at: null,
      converged_at: null,
      parent_goal_id: null,
      depends_on: [],
      current_iteration: 0,
      per_iteration_timeout_ms: 60000,
      parent_task_id: null,
      output_files: [],
    }

    expect(goal.id).toBe('test-goal')
    expect(goal.status.status).toBe('pending')
  })

  it('validates GoalStatus variants', () => {
    const statuses: GoalStatus[] = [
      { status: 'pending' },
      { status: 'active' },
      { status: 'evaluating' },
      { status: 'converged' },
      { status: 'iterating', feedback: 'test' },
      { status: 'failed', reason: 'error' },
      { status: 'budget_exhausted' },
      { status: 'max_iter_reached' },
      { status: 'cancelled' },
    ]

    statuses.forEach((status) => {
      expect(status.status).toBeDefined()
    })
  })

  it('validates CompletionCondition variants', () => {
    const conditions = [
      { type: 'command_success', command: 'echo', args: [], cwd: null },
      { type: 'output_contains', command: '', pattern: 'hello', case_sensitive: true },
      { type: 'output_matches', command: '', regex: 'test' },
      { type: 'file_check', path: 'test.txt', content_contains: null, max_size_bytes: null },
      { type: 'http_health_check', url: 'http://localhost', method: 'GET', expected_status: 200 },
      { type: 'queen_judgment', criteria: 'check result' },
      { type: 'all', conditions: [] },
      { type: 'any', conditions: [] },
    ]

    conditions.forEach((condition) => {
      expect(condition.type).toBeDefined()
    })
  })
})
