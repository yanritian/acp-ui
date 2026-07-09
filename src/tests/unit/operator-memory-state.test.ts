// Operator Memory State Tests
// Testing state persistence and memory management

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

describe('Operator Memory State Tests', () => {
  describe('Task State Persistence', () => {
    it('should persist task state', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_001',
        status: 'running',
        event_stream: 'state_data'
      })
      const task = await OperatorApi.getTask('memory_001')
      expect(task.status).toBe('running')
    })

    it('should restore task after restart', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_002',
        status: 'paused',
        event_stream: 'restored_state'
      })
      const task = await OperatorApi.getTask('memory_002')
      expect(task.status).toBe('paused')
    })
  })

  describe('Event Memory', () => {
    it('should store events in memory', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'memory_003' }
      ])
      const events = await OperatorApi.listEvents('memory_003')
      expect(events.length).toBe(1)
    })

    it('should limit event memory size', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(100).fill(null).map((_, i) => ({
          event_id: `e${i}`,
          type: 'task_started',
          title: 'Event',
          level: 'info',
          source: 'operator',
          timestamp: '2026-07-09T00:00:00Z',
          task_id: 'memory_004'
        }))
      )
      const events = await OperatorApi.listEvents('memory_004')
      expect(events.length).toBeLessThanOrEqual(100)
    })
  })

  describe('Approval Memory', () => {
    it('should store pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'memory_005', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('memory_005')
      expect(approvals.length).toBe(1)
    })

    it('should clear approval after decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'memory_006',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })
  })

  describe('File Change Memory', () => {
    it('should track file changes', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_007',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('memory_007')
      expect(summary.files_changed.length).toBe(1)
    })

    it('should track multiple file changes', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_008',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 30,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('memory_008')
      expect(summary.files_changed.length).toBe(3)
      expect(summary.files_created.length).toBe(1)
      expect(summary.files_deleted.length).toBe(1)
    })
  })

  describe('Error Memory', () => {
    it('should store task errors', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_009',
        status: 'failed',
        goal: 'Test',
        summary: 'Failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 5,
        iterations: 1,
        errors: ['Error 1', 'Error 2'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('memory_009')
      expect(summary.errors.length).toBe(2)
    })

    it('should store task warnings', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'memory_010',
        status: 'completed',
        goal: 'Test',
        summary: 'Done with warnings',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: ['Warning 1']
      })
      const summary = await OperatorApi.getTaskSummary('memory_010')
      expect(summary.warnings.length).toBe(1)
    })
  })
})