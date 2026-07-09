// Operator Ultimate Tests
// Ultimate comprehensive testing

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

describe('Operator Ultimate Tests', () => {
  describe('Complete Task Lifecycle', () => {
    it('should complete full lifecycle: create -> plan -> approve -> run -> complete', async () => {
      // Create
      mockInvoke.mockResolvedValueOnce({
        task_id: 'ultimate_001',
        status: 'waiting_approval',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Ultimate test',
        approval_policy: 'strict'
      })

      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'ultimate_001', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('ultimate_001')
      expect(approvals.length).toBe(1)

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'ultimate_001',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })

      // Get summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'ultimate_001',
        status: 'completed',
        goal: 'Ultimate test',
        summary: 'Completed',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('ultimate_001')
      expect(summary).toBeDefined()
    })
  })

  describe('Complete File Operations', () => {
    it('should complete full file cycle: list -> read -> preview -> patch', async () => {
      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd'],
        total: 2
      })
      const listResult = await OperatorApi.fileList('ultimate_002', 'scripts')
      expect(listResult.files.length).toBe(2)

      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const readResult = await OperatorApi.fileRead('ultimate_002', 'Player.gd')
      expect(readResult.content).toBeDefined()

      // Preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new line',
        lines_added: 1,
        lines_removed: 0
      })
      const previewResult = await OperatorApi.filePatchPreview('ultimate_002', 'Player.gd', 'new content')
      expect(previewResult.lines_added).toBe(1)

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      const patchResult = await OperatorApi.filePatch('ultimate_002', 'Player.gd', 'new content')
      expect(patchResult.success).toBe(true)
    })
  })

  describe('Complete Error Recovery', () => {
    it('should handle and recover from errors', async () => {
      // Error
      mockInvoke.mockRejectedValueOnce(new Error('File not found'))
      try {
        await OperatorApi.fileRead('ultimate_003', 'missing.gd')
      } catch (e) {
        // Expected
      }

      // Recovery
      mockInvoke.mockResolvedValueOnce({
        content: 'found',
        path: 'existing.gd'
      })
      const result = await OperatorApi.fileRead('ultimate_003', 'existing.gd')
      expect(result.content).toBeDefined()
    })
  })

  describe('Complete Event Tracking', () => {
    it('should track all events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'ultimate_004' },
        { event_id: 'e2', type: 'plan_ready', title: 'Plan', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'ultimate_004' },
        { event_id: 'e3', type: 'approval_requested', title: 'Approval', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'ultimate_004' },
        { event_id: 'e4', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:03:00Z', task_id: 'ultimate_004' }
      ])
      const events = await OperatorApi.listEvents('ultimate_004')
      expect(events.length).toBe(4)
    })
  })
})