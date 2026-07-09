// Operator Integration Tests
// Testing complete workflows

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

describe('Operator Integration Tests', () => {
  describe('Complete Workflow', () => {
    it('should complete full workflow: create -> approve -> complete', async () => {
      // Step 1: Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'workflow_001',
        status: 'waiting_approval',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature',
        approval_policy: 'strict'
      })
      expect(task.status).toBe('waiting_approval')

      // Step 2: Get pending approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'workflow_001', level: 'approve', title: 'Approve' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('workflow_001')
      expect(approvals.length).toBe(1)

      // Step 3: Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'workflow_001',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Approved'
      })

      // Step 4: Get summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'workflow_001',
        status: 'completed',
        goal: 'Add feature',
        summary: 'Completed',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('workflow_001')
      expect(summary.status).toBe('completed')
    })

    it('should complete workflow with pause-resume', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'pause_001',
        status: 'running',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('pause_001')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('pause_001')

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('pause_001')
    })

    it('should complete workflow with redirect', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'redirect_001',
        status: 'running',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Original goal'
      })

      // Redirect
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 'redirect_001',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })
  })

  describe('File Workflow', () => {
    it('should complete file workflow: read -> preview -> patch', async () => {
      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'original content',
        path: 'Player.gd'
      })
      const readResult = await OperatorApi.fileRead('file_001', 'Player.gd')
      expect(readResult.content).toBeDefined()

      // Preview patch
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new line',
        lines_added: 1,
        lines_removed: 0
      })
      const previewResult = await OperatorApi.filePatchPreview('file_001', 'Player.gd', 'new content')
      expect(previewResult.lines_added).toBe(1)

      // Apply patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        backup_created: true
      })
      const patchResult = await OperatorApi.filePatch('file_001', 'Player.gd', 'new content')
      expect(patchResult.success).toBe(true)
    })

    it('should list files and read selected', async () => {
      // List files
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        total: 3
      })
      const listResult = await OperatorApi.fileList('file_002', 'scripts')
      expect(listResult.files.length).toBe(3)

      // Read first file
      mockInvoke.mockResolvedValueOnce({
        content: 'file content',
        path: 'Player.gd'
      })
      const readResult = await OperatorApi.fileRead('file_002', listResult.files[0])
      expect(readResult.content).toBeDefined()
    })
  })

  describe('Event Workflow', () => {
    it('should track events throughout task lifecycle', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'event_001',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      // List events
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_001' },
        { event_id: 'e2', type: 'plan_ready', title: 'Plan', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'event_001' }
      ])
      const events = await OperatorApi.listEvents('event_001')
      expect(events.length).toBe(2)
    })
  })
})