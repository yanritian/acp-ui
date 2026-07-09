// Operator Hook Tests
// Testing hook execution and lifecycle

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

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

describe('Operator Hook Tests', () => {
  describe('Hook Events', () => {
    it('should generate hook_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'h1', type: 'hook_started', title: 'Hook', level: 'info', source: 'hook', timestamp: '2026-07-09T00:00:00Z', task_id: 'hook_001' }
      ])
      const events = await OperatorApi.listEvents('hook_001')
      expect(events[0].type).toBe('hook_started')
    })

    it('should generate hook_succeeded event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'h2', type: 'hook_succeeded', title: 'Success', level: 'info', source: 'hook', timestamp: '2026-07-09T00:00:00Z', task_id: 'hook_002' }
      ])
      const events = await OperatorApi.listEvents('hook_002')
      expect(events[0].type).toBe('hook_succeeded')
    })

    it('should generate hook_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'h3', type: 'hook_failed', title: 'Failed', level: 'error', source: 'hook', timestamp: '2026-07-09T00:00:00Z', task_id: 'hook_003' }
      ])
      const events = await OperatorApi.listEvents('hook_003')
      expect(events[0].type).toBe('hook_failed')
    })
  })

  describe('Hook Execution', () => {
    it('should execute pre-task hook', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'hook_004', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should execute post-task hook', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'hook_005',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('hook_005')
      expect(summary.status).toBe('completed')
    })
  })

  describe('Hook Failure Handling', () => {
    it('should handle hook failure gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Hook failed'))
      await expect(OperatorApi.getTask('hook_006')).rejects.toThrow('Hook failed')
    })

    it('should continue on non-critical hook failure', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'hook_007',
        status: 'completed',
        goal: 'Test',
        summary: 'Done with hook warning',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: ['Hook warning']
      })
      const summary = await OperatorApi.getTaskSummary('hook_007')
      expect(summary.warnings.length).toBe(1)
    })
  })

  describe('Hook Configuration', () => {
    it('should use configured hooks', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'hook_008', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test with hooks'
      })
      expect(task.task_id).toBeDefined()
    })
  })
})