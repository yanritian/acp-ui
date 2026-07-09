// Operator Batch Operations Tests
// Testing batch and bulk operations

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

describe('Operator Batch Operations Tests', () => {
  describe('Batch Task Operations', () => {
    it('should create multiple tasks', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'batch_t', status: 'planning', event_stream: '' })
      const results = await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p1', goal: 'G1' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p2', goal: 'G2' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/p3', goal: 'G3' })
      ])
      expect(results.length).toBe(3)
    })

    it('should stop multiple tasks', async () => {
      mockInvoke.mockResolvedValue(undefined)
      await Promise.all([
        OperatorApi.stopTask('t1'),
        OperatorApi.stopTask('t2'),
        OperatorApi.stopTask('t3')
      ])
    })

    it('should pause multiple tasks', async () => {
      mockInvoke.mockResolvedValue(undefined)
      await Promise.all([
        OperatorApi.pauseTask('t1'),
        OperatorApi.pauseTask('t2')
      ])
    })

    it('should resume multiple tasks', async () => {
      mockInvoke.mockResolvedValue(undefined)
      await Promise.all([
        OperatorApi.resumeTask('t1'),
        OperatorApi.resumeTask('t2')
      ])
    })
  })

  describe('Batch File Operations', () => {
    it('should read multiple files', async () => {
      mockInvoke.mockResolvedValue({ content: 'test', path: 'file.gd' })
      const results = await Promise.all([
        OperatorApi.fileRead('batch_001', 'file1.gd'),
        OperatorApi.fileRead('batch_001', 'file2.gd'),
        OperatorApi.fileRead('batch_001', 'file3.gd')
      ])
      expect(results.length).toBe(3)
    })

    it('should patch multiple files', async () => {
      mockInvoke.mockResolvedValue({ success: true })
      const results = await Promise.all([
        OperatorApi.filePatch('batch_001', 'file1.gd', 'content1'),
        OperatorApi.filePatch('batch_001', 'file2.gd', 'content2')
      ])
      expect(results.length).toBe(2)
    })

    it('should preview multiple patches', async () => {
      mockInvoke.mockResolvedValue({ lines_added: 1, lines_removed: 0 })
      const results = await Promise.all([
        OperatorApi.filePatchPreview('batch_001', 'file1.gd', 'new1'),
        OperatorApi.filePatchPreview('batch_001', 'file2.gd', 'new2')
      ])
      expect(results.length).toBe(2)
    })
  })

  describe('Batch Approval Operations', () => {
    it('should approve multiple requests', async () => {
      mockInvoke.mockResolvedValue(undefined)
      await Promise.all([
        OperatorApi.approve({ task_id: 't1', approval_id: 'a1', decision: 'approve', reason: 'OK' }),
        OperatorApi.approve({ task_id: 't2', approval_id: 'a2', decision: 'approve', reason: 'OK' })
      ])
    })

    it('should get approvals for multiple tasks', async () => {
      mockInvoke.mockResolvedValue([])
      const results = await Promise.all([
        OperatorApi.getPendingApprovals('t1'),
        OperatorApi.getPendingApprovals('t2')
      ])
      expect(results.length).toBe(2)
    })
  })

  describe('Batch Event Operations', () => {
    it('should get events for multiple tasks', async () => {
      mockInvoke.mockResolvedValue([])
      const results = await Promise.all([
        OperatorApi.listEvents('t1'),
        OperatorApi.listEvents('t2'),
        OperatorApi.listEvents('t3')
      ])
      expect(results.length).toBe(3)
    })

    it('should get summaries for multiple tasks', async () => {
      mockInvoke.mockResolvedValue({
        task_id: 't',
        status: 'completed',
        goal: 'Test',
        summary: 'Done',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: [],
        warnings: []
      })
      const results = await Promise.all([
        OperatorApi.getTaskSummary('t1'),
        OperatorApi.getTaskSummary('t2')
      ])
      expect(results.length).toBe(2)
    })
  })
})