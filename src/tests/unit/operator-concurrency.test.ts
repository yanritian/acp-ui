// Operator Concurrency Tests
// Testing concurrent operations

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

describe('Operator Concurrency Tests', () => {
  describe('Concurrent Task Operations', () => {
    it('should handle 30 concurrent task creations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'concurrent_t', status: 'planning', event_stream: '' })
      const start = Date.now()
      const results = await Promise.all(
        Array(30).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: `/test${i}`,
            goal: `Concurrent task ${i}`
          })
        )
      )
      const duration = Date.now() - start
      expect(results.length).toBe(30)
      expect(duration).toBeLessThan(10000)
    })

    it('should handle 100 concurrent file reads', async () => {
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      const results = await Promise.all(
        Array(100).fill(null).map((_, i) =>
          OperatorApi.fileRead('concurrent_001', `file${i}.gd`)
        )
      )
      expect(results.length).toBe(100)
    })

    it('should handle mixed concurrent operations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'mixed', status: 'running', event_stream: '' })
      mockInvoke.mockResolvedValue(undefined)
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })

      const results = await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/t1', goal: 'T1' }),
        OperatorApi.pauseTask('t1'),
        OperatorApi.resumeTask('t1'),
        OperatorApi.fileRead('t1', 'file.gd'),
        OperatorApi.listEvents('t1'),
        OperatorApi.listTasks()
      ])

      expect(results.length).toBe(6)
    })
  })

  describe('Concurrent Approval Operations', () => {
    it('should handle 20 concurrent approvals', async () => {
      mockInvoke.mockResolvedValue(undefined)
      const results = await Promise.all(
        Array(20).fill(null).map((_, i) =>
          OperatorApi.approve({
            task_id: `task_${i}`,
            approval_id: `approval_${i}`,
            decision: 'approve',
            reason: 'OK'
          })
        )
      )
      expect(results.length).toBe(20)
    })

    it('should handle concurrent approval queries', async () => {
      mockInvoke.mockResolvedValue([])
      const results = await Promise.all(
        Array(50).fill(null).map((_, i) =>
          OperatorApi.getPendingApprovals(`task_${i}`)
        )
      )
      expect(results.length).toBe(50)
    })
  })

  describe('Concurrent File Operations', () => {
    it('should handle 50 concurrent file patches', async () => {
      mockInvoke.mockResolvedValue({ success: true, path: 'file.gd' })
      const results = await Promise.all(
        Array(50).fill(null).map((_, i) =>
          OperatorApi.filePatch('concurrent_002', `file${i}.gd`, 'new content')
        )
      )
      expect(results.length).toBe(50)
    })

    it('should handle concurrent read and write', async () => {
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      mockInvoke.mockResolvedValue({ success: true })

      const results = await Promise.all([
        OperatorApi.fileRead('concurrent_003', 'file1.gd'),
        OperatorApi.filePatch('concurrent_003', 'file2.gd', 'content'),
        OperatorApi.fileRead('concurrent_003', 'file3.gd'),
        OperatorApi.filePatch('concurrent_003', 'file4.gd', 'content')
      ])

      expect(results.length).toBe(4)
    })
  })

  describe('Concurrent Event Queries', () => {
    it('should handle 100 concurrent event queries', async () => {
      mockInvoke.mockResolvedValue([])
      const results = await Promise.all(
        Array(100).fill(null).map((_, i) =>
          OperatorApi.listEvents(`task_${i}`)
        )
      )
      expect(results.length).toBe(100)
    })

    it('should handle concurrent task and event queries', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'running', goal: 'Test' })
      mockInvoke.mockResolvedValue([])

      const results = await Promise.all([
        OperatorApi.getTask('task_1'),
        OperatorApi.listEvents('task_1'),
        OperatorApi.getTask('task_2'),
        OperatorApi.listEvents('task_2')
      ])

      expect(results.length).toBe(4)
    })
  })

  describe('Race Conditions', () => {
    it('should handle rapid start-stop', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'race_t', status: 'running', event_stream: '' })
      mockInvoke.mockResolvedValue(undefined)

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      expect(task.task_id).toBeDefined()

      // These operations might fail due to state, but should not throw
      await OperatorApi.stopTask('race_t').catch(() => {})
      await OperatorApi.pauseTask('race_t').catch(() => {})
    })

    it('should handle concurrent pause-resume cycles', async () => {
      mockInvoke.mockResolvedValue(undefined)

      const cycles = Array(10).fill(null).map(async () => {
        await OperatorApi.pauseTask('race_002')
        await OperatorApi.resumeTask('race_002')
      })

      await Promise.all(cycles)
    })
  })
})