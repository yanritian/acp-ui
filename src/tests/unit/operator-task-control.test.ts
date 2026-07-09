// Operator Task Control Tests
// Testing pause, resume, stop, and redirect operations

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

describe('Operator Task Control Tests', () => {
  describe('Pause Operation', () => {
    it('should pause running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('t1')
    })

    it('should fail to pause completed task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot pause completed task'))
      await expect(OperatorApi.pauseTask('completed_task')).rejects.toThrow('Cannot pause')
    })

    it('should fail to pause idle task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot pause idle task'))
      await expect(OperatorApi.pauseTask('idle_task')).rejects.toThrow('Cannot pause')
    })
  })

  describe('Resume Operation', () => {
    it('should resume paused task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('t1')
    })

    it('should fail to resume running task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task is not paused'))
      await expect(OperatorApi.resumeTask('running_task')).rejects.toThrow('not paused')
    })

    it('should fail to resume cancelled task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot resume cancelled task'))
      await expect(OperatorApi.resumeTask('cancelled_task')).rejects.toThrow('Cannot resume')
    })
  })

  describe('Stop Operation', () => {
    it('should stop running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('t1')
    })

    it('should stop paused task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('paused_task')
    })

    it('should fail to stop completed task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot stop completed task'))
      await expect(OperatorApi.stopTask('completed_task')).rejects.toThrow('Cannot stop')
    })

    it('should fail to stop idle task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot stop idle task'))
      await expect(OperatorApi.stopTask('idle_task')).rejects.toThrow('Cannot stop')
    })
  })

  describe('Redirect Operation', () => {
    it('should redirect running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 't1',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })

    it('should redirect with work preservation', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 't1',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })

    it('should redirect without work preservation', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 't1',
        new_goal: 'New goal',
        preserve_completed_work: false
      })
    })

    it('should fail to redirect completed task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot redirect completed task'))
      await expect(OperatorApi.redirectTask({
        task_id: 'completed_task',
        new_goal: 'New goal',
        preserve_completed_work: true
      })).rejects.toThrow('Cannot redirect')
    })
  })

  describe('Get Task', () => {
    it('should get task details', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:01:00Z'
      })
      const task = await OperatorApi.getTask('t1')
      expect(task.task_id).toBe('t1')
    })

    it('should return error for non-existent task', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task not found'))
      await expect(OperatorApi.getTask('nonexistent')).rejects.toThrow('not found')
    })
  })

  describe('List Tasks', () => {
    it('should list all tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task 1' },
        { task_id: 't2', status: 'completed', goal: 'Task 2' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(2)
    })

    it('should return empty list when no tasks', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(0)
    })
  })
})