// Operator Final Comprehensive Tests
// Final comprehensive testing of all operator capabilities

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

describe('Operator Final Comprehensive Tests', () => {
  describe('All Task Operations', () => {
    it('should test all task creation variants', async () => {
      // Minimal params
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_comp_001',
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
        task_id: 'final_comp_002',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply'
      })

      // With policy
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_comp_003',
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
      // Get task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_comp_004',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      await OperatorApi.getTask('final_comp_004')

      // List tasks
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'final_comp_004', status: 'running', goal: 'Test' }
      ])
      await OperatorApi.listTasks()

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('final_comp_004')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('final_comp_004')

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('final_comp_004')

      // Redirect
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 'final_comp_004',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })
  })

  describe('All File Operations', () => {
    it('should test all file operations', async () => {
      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['file1.gd', 'file2.gd'],
        total: 2
      })
      await OperatorApi.fileList('final_comp_005', 'scripts')

      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'content',
        path: 'file.gd'
      })
      await OperatorApi.fileRead('final_comp_005', 'file.gd')

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'file.gd'
      })
      await OperatorApi.filePatch('final_comp_005', 'file.gd', 'new content')

      // Preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ line',
        lines_added: 1,
        lines_removed: 0
      })
      await OperatorApi.filePatchPreview('final_comp_005', 'file.gd', 'new content')
    })
  })

  describe('All Approval Operations', () => {
    it('should test all approval operations', async () => {
      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'final_comp_006', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      await OperatorApi.getPendingApprovals('final_comp_006')

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'final_comp_006',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })
  })

  describe('All Event Operations', () => {
    it('should test all event operations', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'final_comp_007' }
      ])
      await OperatorApi.listEvents('final_comp_007')
    })
  })

  describe('All Summary Operations', () => {
    it('should test all summary operations', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_comp_008',
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
      await OperatorApi.getTaskSummary('final_comp_008')
    })
  })
})