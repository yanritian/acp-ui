// Operator Stress Tests
// Testing system under heavy load

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

describe('Operator Stress Tests', () => {
  describe('Concurrent Task Creation', () => {
    it('should handle 20 concurrent task creations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'stress_t', status: 'planning', event_stream: '' })
      const start = Date.now()
      const results = await Promise.all(
        Array(20).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: `/test${i}`,
            goal: `Stress test ${i}`
          })
        )
      )
      const duration = Date.now() - start
      expect(results.length).toBe(20)
      expect(duration).toBeLessThan(10000)
    })

    it('should handle 50 concurrent file reads', async () => {
      mockInvoke.mockResolvedValue({ content: 'test content', path: 'file.gd' })
      const start = Date.now()
      const results = await Promise.all(
        Array(50).fill(null).map((_, i) =>
          OperatorApi.fileRead('stress_001', `file${i}.gd`)
        )
      )
      const duration = Date.now() - start
      expect(results.length).toBe(50)
      expect(duration).toBeLessThan(10000)
    })

    it('should handle 100 concurrent event queries', async () => {
      mockInvoke.mockResolvedValue([])
      const start = Date.now()
      const results = await Promise.all(
        Array(100).fill(null).map((_, i) =>
          OperatorApi.listEvents(`task_${i}`)
        )
      )
      const duration = Date.now() - start
      expect(results.length).toBe(100)
      expect(duration).toBeLessThan(10000)
    })
  })

  describe('Large Data Handling', () => {
    it('should handle 1000 files in list', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: Array(1000).fill('file.gd'),
        total: 1000
      })
      const result = await OperatorApi.fileList('stress_002', 'large_dir')
      expect(result.files.length).toBe(1000)
    })

    it('should handle 500 events', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(500).fill(null).map((_, i) => ({
          event_id: `e${i}`,
          type: 'task_started',
          title: 'Event',
          level: 'info',
          source: 'operator',
          timestamp: '2026-07-09T00:00:00Z',
          task_id: 'stress_003'
        }))
      )
      const events = await OperatorApi.listEvents('stress_003')
      expect(events.length).toBe(500)
    })

    it('should handle long goal text (5000 chars)', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'stress_004', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'A'.repeat(5000)
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Mixed Operations', () => {
    it('should handle mixed operations concurrently', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'mixed', status: 'planning', event_stream: '' })
      mockInvoke.mockResolvedValue(undefined)
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })

      const results = await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/t1', goal: 'T1' }),
        OperatorApi.pauseTask('t1'),
        OperatorApi.resumeTask('t1'),
        OperatorApi.fileRead('t1', 'file.gd'),
        OperatorApi.listEvents('t1')
      ])

      expect(results.length).toBe(5)
    })
  })
})