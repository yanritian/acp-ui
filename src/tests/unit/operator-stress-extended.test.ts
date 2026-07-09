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

describe('Operator Stress Tests Extended', () => {
  describe('Heavy Load Operations', () => {
    it('should handle 40 concurrent task creations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'stress_t', status: 'planning', event_stream: '' })
      const start = Date.now()
      const results = await Promise.all(
        Array(40).fill(null).map((_, i) =>
          OperatorApi.startTask({
            domain: 'game.godot',
            project_path: `/test${i}`,
            goal: `Stress test ${i}`
          })
        )
      )
      const duration = Date.now() - start
      expect(results.length).toBe(40)
      expect(duration).toBeLessThan(10000)
    })

    it('should handle 200 concurrent file reads', async () => {
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      const results = await Promise.all(
        Array(200).fill(null).map((_, i) =>
          OperatorApi.fileRead('stress_001', `file${i}.gd`)
        )
      )
      expect(results.length).toBe(200)
    })

    it('should handle 1000 event queries', async () => {
      mockInvoke.mockResolvedValue([])
      const results = await Promise.all(
        Array(1000).fill(null).map((_, i) =>
          OperatorApi.listEvents(`task_${i}`)
        )
      )
      expect(results.length).toBe(1000)
    })
  })

  describe('Memory Stress', () => {
    it('should handle large file list (5000 files)', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: Array(5000).fill('file.gd'),
        total: 5000
      })
      const result = await OperatorApi.fileList('stress_002', 'large_dir')
      expect(result.files.length).toBe(5000)
    })

    it('should handle large event list (10000 events)', async () => {
      mockInvoke.mockResolvedValueOnce(
        Array(10000).fill(null).map((_, i) => ({
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
      expect(events.length).toBe(10000)
    })
  })

  describe('Concurrent Mixed Operations', () => {
    it('should handle 50 mixed concurrent operations', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'mixed', status: 'running', event_stream: '' })
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      mockInvoke.mockResolvedValue({ success: true })

      const operations = []
      for (let i = 0; i < 50; i++) {
        if (i % 3 === 0) {
          operations.push(OperatorApi.startTask({ domain: 'game.godot', project_path: `/t${i}`, goal: `Task ${i}` }))
        } else if (i % 3 === 1) {
          operations.push(OperatorApi.fileRead('t1', 'file.gd'))
        } else {
          operations.push(OperatorApi.filePatch('t1', 'file.gd', 'content'))
        }
      }

      const results = await Promise.all(operations)
      expect(results.length).toBe(50)
    })
  })
})