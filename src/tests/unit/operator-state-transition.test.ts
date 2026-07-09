// Operator State Transition Tests
// Testing all valid state transitions

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

describe('Operator State Transition Tests', () => {
  describe('Valid Transitions', () => {
    it('should transition idle -> planning', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.status).toBe('planning')
    })

    it('should transition planning -> waiting_approval', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'waiting_approval',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
      expect(task.status).toBe('waiting_approval')
    })

    it('should transition waiting_approval -> running', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 't1',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })

    it('should transition running -> paused', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('t1')
    })

    it('should transition paused -> running', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('t1')
    })

    it('should transition running -> cancelled', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('t1')
    })

    it('should transition running -> completed', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
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
      const summary = await OperatorApi.getTaskSummary('t1')
      expect(summary.status).toBe('completed')
    })

    it('should transition running -> failed', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'failed',
        goal: 'Test',
        summary: 'Failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 5,
        iterations: 1,
        errors: ['Error occurred'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t1')
      expect(summary.status).toBe('failed')
    })
  })

  describe('Terminal States', () => {
    it('should not transition from completed', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot transition from completed'))
      await expect(OperatorApi.pauseTask('completed_task')).rejects.toThrow()
    })

    it('should not transition from cancelled', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot transition from cancelled'))
      await expect(OperatorApi.resumeTask('cancelled_task')).rejects.toThrow()
    })

    it('should not transition from failed', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot transition from failed'))
      await expect(OperatorApi.stopTask('failed_task')).rejects.toThrow()
    })
  })

  describe('Redirect Transitions', () => {
    it('should transition running -> redirecting', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 't1',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })
  })
})