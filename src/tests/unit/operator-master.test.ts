// Operator Master Tests
// Master comprehensive testing of all features

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

describe('Operator Master Tests', () => {
  describe('All API Methods', () => {
    it('should test all start task variants', async () => {
      // Minimal
      mockInvoke.mockResolvedValueOnce({
        task_id: 'master_001',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      // With mode
      mockInvoke.mockResolvedValueOnce({
        task_id: 'master_002',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'apply_directly'
      })

      // With policy
      mockInvoke.mockResolvedValueOnce({
        task_id: 'master_003',
        status: 'waiting_approval',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
    })

    it('should test all task control operations', async () => {
      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('master_004')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('master_004')

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('master_004')

      // Redirect
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 'master_004',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })

    it('should test all file operations', async () => {
      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'content',
        path: 'file.gd'
      })
      await OperatorApi.fileRead('master_005', 'file.gd')

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'file.gd'
      })
      await OperatorApi.filePatch('master_005', 'file.gd', 'content')

      // Preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ line',
        lines_added: 1,
        lines_removed: 0
      })
      await OperatorApi.filePatchPreview('master_005', 'file.gd', 'content')

      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['file.gd'],
        total: 1
      })
      await OperatorApi.fileList('master_005', 'scripts')
    })

    it('should test all approval operations', async () => {
      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'master_006', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      await OperatorApi.getPendingApprovals('master_006')

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'master_006',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })

    it('should test all event operations', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'master_007' }
      ])
      await OperatorApi.listEvents('master_007')
    })

    it('should test all summary operations', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'master_008',
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
      await OperatorApi.getTaskSummary('master_008')
    })
  })
})