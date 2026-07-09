// Operator Summary Tests
// Testing task summary generation and reporting

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

describe('Operator Summary Tests', () => {
  describe('Summary Generation', () => {
    it('should generate completed summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'completed',
        goal: 'Add player movement',
        summary: 'Successfully added player movement code',
        files_changed: ['scripts/Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t1')
      expect(summary.status).toBe('completed')
      expect(summary.files_changed.length).toBe(1)
    })

    it('should generate failed summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't2',
        status: 'failed',
        goal: 'Invalid task',
        summary: 'Task failed due to errors',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: ['Compilation error'],
        warnings: ['Deprecated API']
      })
      const summary = await OperatorApi.getTaskSummary('t2')
      expect(summary.status).toBe('failed')
      expect(summary.errors.length).toBe(1)
    })

    it('should include multiple file changes', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't3',
        status: 'completed',
        goal: 'Refactor',
        summary: 'Refactored multiple files',
        files_changed: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 120,
        iterations: 10,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t3')
      expect(summary.files_changed.length).toBe(3)
      expect(summary.files_created.length).toBe(1)
      expect(summary.files_deleted.length).toBe(1)
    })
  })

  describe('Duration Tracking', () => {
    it('should track short duration', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't4',
        status: 'completed',
        goal: 'Quick fix',
        summary: 'Fixed typo',
        files_changed: ['README.md'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 5,
        iterations: 1,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t4')
      expect(summary.duration_seconds).toBeLessThan(10)
    })

    it('should track long duration', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't5',
        status: 'completed',
        goal: 'Major refactor',
        summary: 'Completed major refactoring',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 3600,
        iterations: 50,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t5')
      expect(summary.duration_seconds).toBeGreaterThan(1000)
    })
  })

  describe('Iteration Counting', () => {
    it('should count single iteration', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't6',
        status: 'completed',
        goal: 'Simple task',
        summary: 'Done',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t6')
      expect(summary.iterations).toBe(1)
    })

    it('should count multiple iterations', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't7',
        status: 'completed',
        goal: 'Complex task',
        summary: 'Done after multiple tries',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 100,
        iterations: 25,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t7')
      expect(summary.iterations).toBeGreaterThan(10)
    })
  })

  describe('Error and Warning Collection', () => {
    it('should collect errors', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't8',
        status: 'failed',
        goal: 'Failing task',
        summary: 'Failed with errors',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: ['Error 1', 'Error 2', 'Error 3'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t8')
      expect(summary.errors.length).toBe(3)
    })

    it('should collect warnings', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't9',
        status: 'completed',
        goal: 'Task with warnings',
        summary: 'Completed with warnings',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 50,
        iterations: 5,
        errors: [],
        warnings: ['Warning 1', 'Warning 2']
      })
      const summary = await OperatorApi.getTaskSummary('t9')
      expect(summary.warnings.length).toBe(2)
    })

    it('should collect both errors and warnings', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't10',
        status: 'failed',
        goal: 'Problematic task',
        summary: 'Failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 20,
        iterations: 2,
        errors: ['Error 1'],
        warnings: ['Warning 1']
      })
      const summary = await OperatorApi.getTaskSummary('t10')
      expect(summary.errors.length).toBe(1)
      expect(summary.warnings.length).toBe(1)
    })
  })
})