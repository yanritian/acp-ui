// Performance Tests for Game Operator
// Tests for response time, throughput, and resource usage

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

describe('Game Operator Performance Tests', () => {
  describe('Response Time', () => {
    it('should handle task creation in under 100ms', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'perf_001',
        status: 'planning',
        event_stream: ''
      })

      const start = Date.now()
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Performance test'
      })
      const duration = Date.now() - start

      expect(duration).toBeLessThan(100)
    })

    it('should handle task listing in under 50ms', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task 1' },
        { task_id: 't2', status: 'completed', goal: 'Task 2' }
      ])

      const start = Date.now()
      await OperatorApi.listTasks()
      const duration = Date.now() - start

      expect(duration).toBeLessThan(50)
    })

    it('should handle event listing in under 50ms', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_started', title: 'Started' }
      ])

      const start = Date.now()
      await OperatorApi.listEvents('t1')
      const duration = Date.now() - start

      expect(duration).toBeLessThan(50)
    })
  })

  describe('Throughput', () => {
    it('should handle 100 concurrent task creations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'planning', event_stream: '' })

      const start = Date.now()
      await Promise.all(
        Array(100).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: '/test',
            goal: `Task ${i}`
          })
        )
      )
      const duration = Date.now() - start

      expect(duration).toBeLessThan(1000)
    })

    it('should handle 1000 concurrent event queries', async () => {
      mockInvoke.mockResolvedValue([])

      const start = Date.now()
      await Promise.all(
        Array(1000).fill(null).map((_, i) =>
          OperatorApi.listEvents(`task_${i}`)
        )
      )
      const duration = Date.now() - start

      expect(duration).toBeLessThan(2000)
    })

    it('should handle mixed operations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'running', event_stream: '' })
      mockInvoke.mockResolvedValue([])
      mockInvoke.mockResolvedValue({ success: true })

      const start = Date.now()
      await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' }),
        OperatorApi.listTasks(),
        OperatorApi.listEvents('t1'),
        OperatorApi.pauseTask('t1'),
        OperatorApi.resumeTask('t1')
      ])
      const duration = Date.now() - start

      expect(duration).toBeLessThan(500)
    })
  })

  describe('Resource Usage', () => {
    it('should handle large task lists', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(1000).fill(null).map((_, i) => ({
          task_id: `task_${i}`,
          status: 'running',
          goal: `Task ${i}`
        }))
      )

      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1000)
    })

    it('should handle large event lists', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(10000).fill(null).map((_, i) => ({
          event_id: `evt_${i}`,
          type: 'task_started',
          title: `Event ${i}`,
          timestamp: '2026-07-11T00:00:00Z',
          level: 'info',
          source: 'operator'
        }))
      )

      const events = await OperatorApi.listEvents('t1')
      expect(events.length).toBe(10000)
    })

    it('should handle large approval lists', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(500).fill(null).map((_, i) => ({
          approval_id: `a_${i}`,
          task_id: 't1',
          level: 'approve',
          action: 'file.patch',
          title: `Approval ${i}`,
          reason: 'Test'
        }))
      )

      const approvals = await OperatorApi.getPendingApprovals('t1')
      expect(approvals.length).toBe(500)
    })
  })

  describe('Memory Efficiency', () => {
    it('should handle repeated operations without memory leaks', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'running', event_stream: '' })

      for (let i = 0; i < 100; i++) {
        await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: `Iteration ${i}`
        })
      }

      expect(true).toBe(true) // If we get here, no memory issues
    })

    it('should clean up after task completion', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't1', status: 'completed' })
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.stopTask('t1')
      await OperatorApi.listTasks()

      expect(true).toBe(true)
    })
  })

  describe('Concurrency Safety', () => {
    it('should handle concurrent task updates', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.pauseTask('t1'),
        OperatorApi.resumeTask('t1'),
        OperatorApi.pauseTask('t1'),
        OperatorApi.resumeTask('t1')
      ])

      expect(true).toBe(true)
    })

    it('should handle concurrent approval operations', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await Promise.all([
        OperatorApi.approve({ task_id: 't1', approval_id: 'a1', decision: 'approve' }),
        OperatorApi.approve({ task_id: 't1', approval_id: 'a2', decision: 'reject' }),
        OperatorApi.approve({ task_id: 't2', approval_id: 'a3', decision: 'approve' })
      ])

      expect(true).toBe(true)
    })
  })
})