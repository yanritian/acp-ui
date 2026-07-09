// Operator Task Summary Tests
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

describe('Operator Task Summary Tests', () => {
  describe('Summary Generation', () => {
    it('should generate summary for completed task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_001',
        status: 'completed',
        goal: 'Add feature',
        summary: 'Successfully added feature',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_001')
      expect(summary).toBeDefined()
      expect(summary.task_id).toBe('summary_001')
    })

    it('should generate summary for failed task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_002',
        status: 'failed',
        goal: 'Add feature',
        summary: 'Task failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: ['Compilation error'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_002')
      expect(summary).toBeDefined()
      expect(summary.status).toBe('failed')
    })

    it('should include file changes in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_003',
        status: 'completed',
        goal: 'Refactor',
        summary: 'Refactored files',
        files_changed: ['Player.gd', 'Enemy.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_003')
      expect(summary.files_changed).toBeDefined()
      expect(summary.files_created).toBeDefined()
      expect(summary.files_deleted).toBeDefined()
    })
  })

  describe('Summary Duration', () => {
    it('should include duration in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_004',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 120,
        iterations: 2,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_004')
      expect(summary.duration_seconds).toBeDefined()
    })

    it('should include iterations in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_005',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 10,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_005')
      expect(summary.iterations).toBeDefined()
    })
  })

  describe('Summary Errors and Warnings', () => {
    it('should include errors in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_006',
        status: 'failed',
        goal: 'Test',
        summary: 'Failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: ['Error 1', 'Error 2'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_006')
      expect(summary.errors).toBeDefined()
    })

    it('should include warnings in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_007',
        status: 'completed',
        goal: 'Test',
        summary: 'Done with warnings',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 2,
        errors: [],
        warnings: ['Warning 1']
      })
      const summary = await OperatorApi.getTaskSummary('summary_007')
      expect(summary.warnings).toBeDefined()
    })
  })

  describe('Summary Text', () => {
    it('should include summary text', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'summary_008',
        status: 'completed',
        goal: 'Add feature',
        summary: 'Successfully completed the task',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('summary_008')
      expect(summary.summary).toBeDefined()
    })
  })
})