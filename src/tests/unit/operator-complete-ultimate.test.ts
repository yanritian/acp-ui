// Operator Complete Ultimate Tests
// Complete ultimate comprehensive testing

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

describe('Operator Complete Ultimate Tests', () => {
  describe('Ultimate Task Lifecycle', () => {
    it('should complete ultimate task lifecycle', async () => {
      // Create
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_ultimate_001',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete ultimate test'
      })
      expect(task.task_id).toBe('complete_ultimate_001')

      // Get
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_ultimate_001',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete ultimate test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const getTask = await OperatorApi.getTask('complete_ultimate_001')
      expect(getTask.status).toBe('running')

      // List
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'complete_ultimate_001', status: 'running', goal: 'Complete ultimate test' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1)

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('complete_ultimate_001')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('complete_ultimate_001')

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('complete_ultimate_001')
    })
  })

  describe('Ultimate File Operations', () => {
    it('should complete ultimate file operations', async () => {
      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        total: 3
      })
      const list = await OperatorApi.fileList('complete_ultimate_002', 'scripts')
      expect(list.files.length).toBe(3)

      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const read = await OperatorApi.fileRead('complete_ultimate_002', 'Player.gd')
      expect(read.content).toBeDefined()

      // Preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new function',
        lines_added: 1,
        lines_removed: 0
      })
      const preview = await OperatorApi.filePatchPreview('complete_ultimate_002', 'Player.gd', 'new content')
      expect(preview.lines_added).toBe(1)

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      const patch = await OperatorApi.filePatch('complete_ultimate_002', 'Player.gd', 'new content')
      expect(patch.success).toBe(true)
    })
  })

  describe('Ultimate Approval Workflow', () => {
    it('should complete ultimate approval workflow', async () => {
      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'complete_ultimate_003', level: 'approve', title: 'Approve', reason: 'Test' },
        { approval_id: 'a2', task_id: 'complete_ultimate_003', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('complete_ultimate_003')
      expect(approvals.length).toBe(2)

      // Approve first
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'complete_ultimate_003',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Approved'
      })

      // Reject second
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'complete_ultimate_003',
        approval_id: 'a2',
        decision: 'reject',
        reason: 'Rejected'
      })
    })
  })

  describe('Ultimate Event Tracking', () => {
    it('should track ultimate events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'complete_ultimate_004' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'complete_ultimate_004' },
        { event_id: 'e3', type: 'approval_requested', title: 'Approval', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'complete_ultimate_004' },
        { event_id: 'e4', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:03:00Z', task_id: 'complete_ultimate_004' }
      ])
      const events = await OperatorApi.listEvents('complete_ultimate_004')
      expect(events.length).toBe(4)
    })
  })

  describe('Ultimate Summary Generation', () => {
    it('should generate ultimate summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'complete_ultimate_005',
        status: 'completed',
        goal: 'Complete ultimate task',
        summary: 'Successfully completed ultimate task',
        files_changed: ['Player.gd', 'Enemy.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 120,
        iterations: 10,
        errors: [],
        warnings: ['Deprecated API used']
      })
      const summary = await OperatorApi.getTaskSummary('complete_ultimate_005')
      expect(summary).toBeDefined()
    })
  })
})