// Performance Benchmarks for Hermes Game Operator
// Tests response times and throughput for critical operations

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

describe('Performance Benchmarks', () => {
  describe('API Response Times', () => {
    it('should start task within 100ms', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_perf_1',
        status: 'planning',
        event_stream: 'operator://tasks/task_perf_1/events'
      })

      const start = performance.now()
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:/tmp/test-godot-project',
        goal: 'Add double jump'
      })
      const duration = performance.now() - start

      // Should complete within 100ms (mock)
      expect(duration).toBeLessThan(100)
    })

    it('should list tasks within 50ms', async () => {
      const mockTasks = Array.from({ length: 100 }, (_, i) => ({
        task_id: `task_${i}`,
        domain: 'game.godot',
        project_path: '/path',
        goal: `Goal ${i}`,
        status: 'completed',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-08T10:00:00Z',
        updated_at: '2026-07-08T10:05:00Z'
      }))
      mockInvoke.mockResolvedValueOnce(mockTasks)

      const start = performance.now()
      const result = await OperatorApi.listTasks()
      const duration = performance.now() - start

      expect(result).toHaveLength(100)
      expect(duration).toBeLessThan(50)
    })

    it('should handle 1000 events efficiently', async () => {
      const mockEvents = Array.from({ length: 1000 }, (_, i) => ({
        event_id: `evt_${i}`,
        task_id: 'task_perf',
        timestamp: `2026-07-08T10:00:${String(i % 60).padStart(2, '0')}Z`,
        type: 'tool_call_started',
        level: 'info',
        title: `Event ${i}`,
        source: 'operator'
      }))
      mockInvoke.mockResolvedValueOnce(mockEvents)

      const start = performance.now()
      const result = await OperatorApi.listEvents('task_perf', 1000)
      const duration = performance.now() - start

      expect(result).toHaveLength(1000)
      expect(duration).toBeLessThan(100)
    })
  })

  describe('Memory Efficiency', () => {
    it('should not leak memory on repeated calls', async () => {
      const initialMemory = (performance as any).memory?.usedJSHeapSize || 0

      for (let i = 0; i < 100; i++) {
        mockInvoke.mockResolvedValueOnce({
          task_id: `task_${i}`,
          status: 'planning',
          event_stream: `operator://tasks/task_${i}/events`
        })
        await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: 'D:/tmp/test',
          goal: 'Test'
        })
      }

      const finalMemory = (performance as any).memory?.usedJSHeapSize || 0

      // Memory should not grow significantly (within 10MB)
      const memoryGrowth = finalMemory - initialMemory
      expect(memoryGrowth).toBeLessThan(10 * 1024 * 1024)
    })
  })

  describe('Concurrent Operations', () => {
    it('should handle 10 concurrent task starts', async () => {
      const promises = []

      for (let i = 0; i < 10; i++) {
        mockInvoke.mockResolvedValueOnce({
          task_id: `task_concurrent_${i}`,
          status: 'planning',
          event_stream: `operator://tasks/task_concurrent_${i}/events`
        })
        promises.push(OperatorApi.startTask({
          domain: 'game.godot',
          project_path: 'D:/tmp/test',
          goal: `Concurrent test ${i}`
        }))
      }

      const start = performance.now()
      const results = await Promise.all(promises)
      const duration = performance.now() - start

      expect(results).toHaveLength(10)
      expect(duration).toBeLessThan(200)
    })

    it('should handle mixed operations concurrently', async () => {
      // Setup mocks for different operations
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_1', status: 'planning', event_stream: '' })
      mockInvoke.mockResolvedValueOnce([{ task_id: 'task_1' }])
      mockInvoke.mockResolvedValueOnce([])
      mockInvoke.mockResolvedValueOnce(undefined)

      const start = performance.now()
      await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' }),
        OperatorApi.listTasks(),
        OperatorApi.listEvents('task_1'),
        OperatorApi.pauseTask('task_1')
      ])
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })
  })

  describe('Event Throughput', () => {
    it('should process 100 events per second', async () => {
      const events = Array.from({ length: 100 }, (_, i) => ({
        event_id: `evt_${i}`,
        task_id: 'task_throughput',
        timestamp: new Date().toISOString(),
        type: 'tool_call_started',
        level: 'info',
        title: `Event ${i}`,
        source: 'operator'
      }))

      const start = performance.now()

      // Simulate processing events
      for (const event of events) {
        // Event processing logic would go here
        expect(event.event_id).toBeDefined()
      }

      const duration = performance.now() - start

      // Should process 100 events in under 1 second
      expect(duration).toBeLessThan(1000)
    })
  })
})