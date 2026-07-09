// Operator Complete Tests
// Complete testing of all operator capabilities

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

describe('Operator Complete Tests', () => {
  describe('Complete Task Management', () => {
    it('should manage task lifecycle completely', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_001',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete test'
      })
      expect(task.task_id).toBe('complete_001')

      // Get task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_001',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const getTask = await OperatorApi.getTask('complete_001')
      expect(getTask.status).toBe('running')

      // List tasks
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'complete_001', status: 'running', goal: 'Complete test' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1)

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('complete_001')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('complete_001')

      // Redirect
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 'complete_001',
        new_goal: 'New goal',
        preserve_completed_work: true
      })

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('complete_001')
    })
  })

  describe('Complete File Management', () => {
    it('should manage files completely', async () => {
      // List files
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        total: 3
      })
      const list = await OperatorApi.fileList('complete_002', 'scripts')
      expect(list.files.length).toBe(3)

      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const read = await OperatorApi.fileRead('complete_002', 'Player.gd')
      expect(read.content).toBeDefined()

      // Preview patch
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new function',
        lines_added: 1,
        lines_removed: 0
      })
      const preview = await OperatorApi.filePatchPreview('complete_002', 'Player.gd', 'new content')
      expect(preview.lines_added).toBe(1)

      // Apply patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      const patch = await OperatorApi.filePatch('complete_002', 'Player.gd', 'new content')
      expect(patch.success).toBe(true)
    })
  })

  describe('Complete Approval Management', () => {
    it('should manage approvals completely', async () => {
      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'complete_003', level: 'approve', title: 'Approve 1', reason: 'Test' },
        { approval_id: 'a2', task_id: 'complete_003', level: 'approve', title: 'Approve 2', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('complete_003')
      expect(approvals.length).toBe(2)

      // Approve first
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'complete_003',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Approved'
      })

      // Reject second
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'complete_003',
        approval_id: 'a2',
        decision: 'reject',
        reason: 'Rejected'
      })
    })
  })

  describe('Complete Event Management', () => {
    it('should manage events completely', async () => {
      // List events
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'complete_004' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'complete_004' },
        { event_id: 'e3', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'complete_004' }
      ])
      const events = await OperatorApi.listEvents('complete_004')
      expect(events.length).toBe(3)
      expect(events[0].type).toBe('task_created')
      expect(events[2].type).toBe('task_completed')
    })
  })

  describe('Complete Summary Management', () => {
    it('should manage summaries completely', async () => {
      // Get completed task summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_005',
        status: 'completed',
        goal: 'Complete task',
        summary: 'Successfully completed',
        files_changed: ['Player.gd', 'Enemy.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 120,
        iterations: 10,
        errors: [],
        warnings: ['Deprecated API used']
      })
      const completedSummary = await OperatorApi.getTaskSummary('complete_005')
      expect(completedSummary.status).toBe('completed')
      expect(completedSummary.files_changed.length).toBe(2)

      // Get failed task summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_006',
        status: 'failed',
        goal: 'Failed task',
        summary: 'Task failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: ['Compilation error'],
        warnings: []
      })
      const failedSummary = await OperatorApi.getTaskSummary('complete_006')
      expect(failedSummary.status).toBe('failed')
      expect(failedSummary.errors.length).toBe(1)
    })
  })
})