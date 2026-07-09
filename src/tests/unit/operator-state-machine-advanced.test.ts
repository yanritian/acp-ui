// Operator State Machine Advanced Tests
// Tests for edge cases and advanced state transitions

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

describe('Operator State Machine Advanced Tests', () => {
  describe('Complex State Transitions', () => {
    it('should handle idle -> planning -> waiting_approval -> running -> completed', async () => {
      const taskId = 'task_complex_1'

      // idle -> planning
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'planning', event_stream: '' })
      const start = await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' })
      expect(start.status).toBe('planning')

      // planning -> waiting_approval
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'waiting_approval', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:01:00Z' })
      const waiting = await OperatorApi.getTask(taskId)
      expect(waiting.status).toBe('waiting_approval')

      // waiting_approval -> running (after approval)
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({ task_id: taskId, approval_id: 'approval_1', decision: 'approve' })

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:02:00Z' })
      const running = await OperatorApi.getTask(taskId)
      expect(running.status).toBe('running')

      // running -> completed
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'completed', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:10:00Z', completed_at: '2026-07-09T10:10:00Z', summary: 'Done' })
      const completed = await OperatorApi.getTask(taskId)
      expect(completed.status).toBe('completed')
    })

    it('should handle idle -> planning -> running -> paused -> running -> cancelled', async () => {
      const taskId = 'task_complex_2'

      // idle -> planning
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'planning', event_stream: '' })
      await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' })

      // planning -> running
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:01:00Z' })
      const running1 = await OperatorApi.getTask(taskId)
      expect(running1.status).toBe('running')

      // running -> paused
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask(taskId)

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'paused', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:02:00Z' })
      const paused = await OperatorApi.getTask(taskId)
      expect(paused.status).toBe('paused')

      // paused -> running
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask(taskId)

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:03:00Z' })
      const running2 = await OperatorApi.getTask(taskId)
      expect(running2.status).toBe('running')

      // running -> cancelled
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask(taskId)

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'cancelled', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:04:00Z' })
      const cancelled = await OperatorApi.getTask(taskId)
      expect(cancelled.status).toBe('cancelled')
    })

    it('should handle failed state transition', async () => {
      const taskId = 'task_failed_1'

      // idle -> planning
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'planning', event_stream: '' })
      await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' })

      // planning -> running
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:01:00Z' })
      await OperatorApi.getTask(taskId)

      // running -> failed
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'failed', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:05:00Z', error: 'Compilation failed' })
      const failed = await OperatorApi.getTask(taskId)
      expect(failed.status).toBe('failed')
      expect(failed.error).toContain('Compilation')
    })
  })

  describe('Redirect State Transitions', () => {
    it('should handle redirect from running', async () => {
      const taskId = 'task_redirect_running'

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', event_stream: '' })
      await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Original' })

      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({ task_id: taskId, new_goal: 'New goal', preserve_completed_work: true })

      expect(mockInvoke).toHaveBeenCalledWith('operator_redirect_task', {
        request: { task_id: taskId, new_goal: 'New goal', preserve_completed_work: true }
      })
    })

    it('should handle redirect from paused', async () => {
      const taskId = 'task_redirect_paused'

      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'paused', event_stream: '' })
      await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Original' })

      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({ task_id: taskId, new_goal: 'Redirected goal', preserve_completed_work: false })

      expect(mockInvoke).toHaveBeenCalled()
    })

    it('should handle multiple redirects', async () => {
      const taskId = 'task_multi_redirect'

      for (let i = 0; i < 5; i++) {
        mockInvoke.mockResolvedValueOnce(undefined)
        await OperatorApi.redirectTask({
          task_id: taskId,
          new_goal: `Goal ${i}`,
          preserve_completed_work: true
        })
      }

      expect(mockInvoke).toHaveBeenCalledTimes(5)
    })
  })

  describe('Terminal State Handling', () => {
    it('should not allow resume from completed', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid state transition: completed -> running'))

      await expect(OperatorApi.resumeTask('task_completed')).rejects.toThrow('Invalid state')
    })

    it('should not allow pause from cancelled', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid state transition: cancelled -> paused'))

      await expect(OperatorApi.pauseTask('task_cancelled')).rejects.toThrow('Invalid state')
    })

    it('should not allow redirect from failed', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot redirect failed task'))

      await expect(OperatorApi.redirectTask({
        task_id: 'task_failed',
        new_goal: 'New',
        preserve_completed_work: false
      })).rejects.toThrow('Cannot redirect')
    })
  })

  describe('Event Stream Management', () => {
    it('should handle 100 events in order', async () => {
      const events = Array.from({ length: 100 }, (_, i) => ({
        event_id: `evt_${i}`,
        task_id: 'task_events',
        type: i === 0 ? 'task_started' : 'step_completed',
        timestamp: `2026-07-09T10:00:${String(i % 60).padStart(2, '0')}Z`,
        level: 'info',
        title: `Event ${i}`,
        source: 'operator'
      }))

      mockInvoke.mockResolvedValueOnce(events)
      const result = await OperatorApi.listEvents('task_events', 100)
      expect(result.length).toBe(100)
      expect(result[0].type).toBe('task_started')
    })

    it('should handle event with complex payload', async () => {
      const complexEvent = {
        event_id: 'evt_complex',
        task_id: 'task_complex',
        timestamp: '2026-07-09T10:00:00Z',
        type: 'tool_call_succeeded',
        level: 'info',
        title: 'Tool call',
        source: 'operator',
        payload: {
          tool: 'file.patch',
          input: { path: 'test.gd', content: 'new code' },
          output: { success: true, backup: 'test.gd.bak' },
          duration_ms: 150
        }
      }

      mockInvoke.mockResolvedValueOnce([complexEvent])
      const events = await OperatorApi.listEvents('task_complex')
      expect(events[0].payload).toBeDefined()
    })
  })

  describe('Concurrent Task Management', () => {
    it('should handle 10 concurrent tasks', async () => {
      for (let i = 0; i < 10; i++) {
        mockInvoke.mockResolvedValueOnce({
          task_id: `task_concurrent_${i}`,
          status: 'planning',
          event_stream: ''
        })
      }

      const tasks = await Promise.all(
        Array.from({ length: 10 }, (_, i) =>
          OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: `Task ${i}` })
        )
      )

      expect(tasks.length).toBe(10)
    })

    it('should list all running tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_1', status: 'running', goal: 'Goal 1' },
        { task_id: 'task_2', status: 'running', goal: 'Goal 2' },
        { task_id: 'task_3', status: 'paused', goal: 'Goal 3' },
        { task_id: 'task_4', status: 'completed', goal: 'Goal 4' }
      ])

      const tasks = await OperatorApi.listTasks()
      const running = tasks.filter((t: any) => t.status === 'running')
      expect(running.length).toBe(2)
    })
  })

  describe('Error Recovery', () => {
    it('should recover from transient error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Temporary failure'))
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_1', status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:00:00Z' })

      // First call fails
      await expect(OperatorApi.getTask('task_1')).rejects.toThrow('Temporary')

      // Second call succeeds
      const task = await OperatorApi.getTask('task_1')
      expect(task.task_id).toBe('task_1')
    })

    it('should preserve task state on API error', async () => {
      // Get task successfully
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_1', status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:00:00Z' })
      const before = await OperatorApi.getTask('task_1')
      expect(before.status).toBe('running')

      // Pause fails
      mockInvoke.mockRejectedValueOnce(new Error('Pause failed'))
      await expect(OperatorApi.pauseTask('task_1')).rejects.toThrow('Pause failed')

      // State should be unchanged
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_1', status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:00:00Z' })
      const after = await OperatorApi.getTask('task_1')
      expect(after.status).toBe('running')
    })
  })
})