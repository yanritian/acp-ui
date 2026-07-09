// Operator Concurrent Tasks Tests
// Testing concurrent task execution and management

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

describe('Operator Concurrent Tasks Tests', () => {
  describe('Multiple Task Creation', () => {
    it('should handle multiple task creation', async () => {
      mockInvoke.mockResolvedValue({ status: 'planning', event_stream: '' })

      const tasks = await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p1', goal: 'T1' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p2', goal: 'T2' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p3', goal: 'T3' })
      ])

      expect(tasks.length).toBe(3)
    })

    it('should list multiple tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task 1' },
        { task_id: 't2', status: 'planning', goal: 'Task 2' },
        { task_id: 't3', status: 'completed', goal: 'Task 3' }
      ])

      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(3)
    })

    it('should filter tasks by status', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task 1' },
        { task_id: 't2', status: 'running', goal: 'Task 2' },
        { task_id: 't3', status: 'completed', goal: 'Task 3' }
      ])

      const tasks = await OperatorApi.listTasks()
      const runningTasks = tasks.filter(t => t.status === 'running')
      expect(runningTasks.length).toBe(2)
    })
  })

  describe('Concurrent Control Operations', () => {
    it('should handle concurrent pause operations', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.pauseTask('t1'),
        OperatorApi.pauseTask('t2'),
        OperatorApi.pauseTask('t3')
      ])
    })

    it('should handle concurrent resume operations', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.resumeTask('t1'),
        OperatorApi.resumeTask('t2')
      ])
    })

    it('should handle concurrent stop operations', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.stopTask('t1'),
        OperatorApi.stopTask('t2')
      ])
    })
  })

  describe('Mixed Operations', () => {
    it('should handle mixed task operations', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'new_task', status: 'planning', event_stream: '' })
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task' }
      ])

      await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'New' })
      await OperatorApi.pauseTask('t1')
      const tasks = await OperatorApi.listTasks()

      expect(tasks).toBeDefined()
    })
  })

  describe('Task Limits', () => {
    it('should handle maximum concurrent tasks', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'planning' })

      const tasks = await Promise.all(
        Array(10).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: `/test${i}`,
            goal: `Task ${i}`
          })
        )
      )

      expect(tasks.length).toBe(10)
    })
  })
})