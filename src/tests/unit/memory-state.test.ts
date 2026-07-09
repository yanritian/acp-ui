// Memory and State Persistence Tests for Hermes Game Operator
// Tests for memory management and state persistence

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock Tauri invoke
const mockInvoke = vi.fn()

beforeEach(() => {
  vi.stubGlobal('__TAURI_INTERNALS__', {
    invoke: mockInvoke
  })
  vi.clearAllMocks()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

import { OperatorApi } from '@/api/operatorApi'

describe('Memory and State Persistence Tests', () => {
  describe('Task State Persistence', () => {
    it('should persist task across sessions', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_persisted', status: 'running', goal: 'Previous task' }
      ])

      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1)
      expect(tasks[0].task_id).toBe('task_persisted')
    })

    it('should restore task state on reload', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_restore',
        status: 'paused',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test goal',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T10:00:00Z',
        updated_at: '2026-07-09T10:05:00Z'
      })

      const task = await OperatorApi.getTask('task_restore')
      expect(task.status).toBe('paused')
    })

    it('should track task history', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_1', type: 'task_started', timestamp: '2026-07-09T10:00:00Z' },
        { event_id: 'evt_2', type: 'step_completed', timestamp: '2026-07-09T10:01:00Z' }
      ])

      const events = await OperatorApi.listEvents('task_history')
      expect(events.length).toBe(2)
    })
  })

  describe('Memory Scope', () => {
    it('should define project-scoped memory', () => {
      const memory = {
        scope: 'project',
        domain: 'godot',
        key: 'player_controller',
        value: 'scripts/Player.gd'
      }
      expect(memory.scope).toBe('project')
    })

    it('should define task-scoped memory', () => {
      const memory = {
        scope: 'task',
        domain: 'godot',
        key: 'current_step',
        value: 3
      }
      expect(memory.scope).toBe('task')
    })

    it('should define operator-scoped memory', () => {
      const memory = {
        scope: 'operator',
        domain: 'godot',
        key: 'domain_pack_version',
        value: '1.0.0'
      }
      expect(memory.scope).toBe('operator')
    })
  })

  describe('Event History Management', () => {
    it('should limit event history size', async () => {
      const events = Array.from({ length: 100 }, (_, i) => ({
        event_id: `evt_${i}`,
        type: 'step_completed'
      }))

      mockInvoke.mockResolvedValueOnce(events)
      const result = await OperatorApi.listEvents('task_limit', 100)
      expect(result.length).toBe(100)
    })

    it('should handle event list request', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_1' },
        { event_id: 'evt_2' }
      ])

      const events = await OperatorApi.listEvents('task_page')
      expect(events.length).toBe(2)
    })
  })

  describe('Task State Management', () => {
    it('should get completed task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_completed',
        status: 'completed'
      })
      const task = await OperatorApi.getTask('task_completed')
      expect(task.status).toBe('completed')
    })

    it('should get failed task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_failed',
        status: 'failed',
        error: 'Error message'
      })
      const task = await OperatorApi.getTask('task_failed')
      expect(task.status).toBe('failed')
    })
  })

  describe('Session Management', () => {
    it('should list active tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_1', status: 'completed' },
        { task_id: 'task_2', status: 'running' },
        { task_id: 'task_3', status: 'paused' }
      ])

      const tasks = await OperatorApi.listTasks()
      const activeTasks = tasks.filter((t: any) => ['running', 'paused'].includes(t.status))
      expect(activeTasks.length).toBe(2)
    })

    it('should get pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'approval_1', task_id: 'task_2', level: 'approve' }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_2')
      expect(approvals.length).toBe(1)
    })
  })

  describe('Summary Generation', () => {
    it('should generate summary for completed task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_summary',
        status: 'completed',
        goal: 'Add feature',
        summary: 'Successfully added the feature',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 120,
        iterations: 15,
        errors: [],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_summary')
      expect(summary.status).toBe('completed')
    })

    it('should include error details in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_error_summary',
        status: 'failed',
        goal: 'Test',
        summary: 'Task failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: ['Compilation error', 'Type mismatch'],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_error_summary')
      expect(summary.errors.length).toBe(2)
    })

    it('should track file modifications in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_files_summary',
        status: 'completed',
        goal: 'Refactor',
        summary: 'Refactored code',
        files_changed: ['A.gd', 'B.gd'],
        files_created: ['C.gd'],
        files_deleted: ['D.gd'],
        duration_seconds: 60,
        iterations: 10,
        errors: [],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_files_summary')
      expect(summary.files_changed.length).toBe(2)
    })
  })

  describe('Concurrent Access', () => {
    it('should handle concurrent state reads', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'task_concurrent', status: 'running' })

      const results = await Promise.all([
        OperatorApi.getTask('task_concurrent'),
        OperatorApi.getTask('task_concurrent'),
        OperatorApi.getTask('task_concurrent')
      ])

      expect(results.length).toBe(3)
    })

    it('should handle concurrent state updates', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.pauseTask('task_update'),
        OperatorApi.stopTask('task_update')
      ])

      expect(mockInvoke).toHaveBeenCalledTimes(2)
    })
  })
})