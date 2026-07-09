// Operator Performance Tests
// Testing performance characteristics

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

describe('Operator Performance Tests', () => {
  describe('Response Time', () => {
    it('should measure task creation time', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'perf_001', status: 'planning', event_stream: '' })
      const start = Date.now()
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      const duration = Date.now() - start
      expect(duration).toBeLessThan(1000)
    })

    it('should measure list tasks time', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const start = Date.now()
      await OperatorApi.listTasks()
      const duration = Date.now() - start
      expect(duration).toBeLessThan(500)
    })

    it('should measure file read time', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'test', path: 'file.gd' })
      const start = Date.now()
      await OperatorApi.fileRead('perf_001', 'file.gd')
      const duration = Date.now() - start
      expect(duration).toBeLessThan(500)
    })
  })

  describe('Throughput', () => {
    it('should handle 10 concurrent task creations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 't', status: 'planning', event_stream: '' })
      const start = Date.now()
      await Promise.all(
        Array(10).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: '/test',
            goal: `Task ${i}`
          })
        )
      )
      const duration = Date.now() - start
      expect(duration).toBeLessThan(5000)
    })

    it('should handle 50 concurrent file reads', async () => {
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      const start = Date.now()
      await Promise.all(
        Array(50).fill(null).map((_, i) =>
          OperatorApi.fileRead('perf_001', `file${i}.gd`)
        )
      )
      const duration = Date.now() - start
      expect(duration).toBeLessThan(5000)
    })
  })

  describe('Memory', () => {
    it('should handle large file list', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: Array(1000).fill('file.gd'),
        total: 1000
      })
      const result = await OperatorApi.fileList('perf_001', 'large_dir')
      expect(result.files.length).toBe(1000)
    })

    it('should handle large event list', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(500).fill(null).map((_, i) => ({
          event_id: `e${i}`,
          type: 'task_started',
          title: 'Event',
          level: 'info',
          source: 'operator',
          timestamp: '2026-07-09T00:00:00Z',
          task_id: 'perf_001'
        }))
      )
      const events = await OperatorApi.listEvents('perf_001')
      expect(events.length).toBe(500)
    })
  })
})