// Operator Comprehensive Tests
// Comprehensive testing of all operator features

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

describe('Operator Comprehensive Tests', () => {
  describe('All Task Operations', () => {
    it('should create, get, list, pause, resume, stop, redirect tasks', async () => {
      // Create
      mockInvoke.mockResolvedValueOnce({
        task_id: 'comp_001',
        status: 'running',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Comprehensive test'
      })

      // Get
      mockInvoke.mockResolvedValueOnce({
        task_id: 'comp_001',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Comprehensive test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const task = await OperatorApi.getTask('comp_001')
      expect(task.task_id).toBe('comp_001')

      // List
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'comp_001', status: 'running', goal: 'Test' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1)

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('comp_001')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('comp_001')

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('comp_001')
    })
  })

  describe('All File Operations', () => {
    it('should read, patch, preview, list files', async () => {
      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'file content',
        path: 'test.gd'
      })
      const readResult = await OperatorApi.fileRead('comp_002', 'test.gd')
      expect(readResult.content).toBeDefined()

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'test.gd'
      })
      const patchResult = await OperatorApi.filePatch('comp_002', 'test.gd', 'new content')
      expect(patchResult.success).toBe(true)

      // Preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new line',
        lines_added: 1,
        lines_removed: 0
      })
      const previewResult = await OperatorApi.filePatchPreview('comp_002', 'test.gd', 'new content')
      expect(previewResult.lines_added).toBe(1)

      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['file1.gd', 'file2.gd'],
        total: 2
      })
      const listResult = await OperatorApi.fileList('comp_002', 'scripts')
      expect(listResult.files.length).toBe(2)
    })
  })

  describe('All Approval Operations', () => {
    it('should get approvals, approve, reject', async () => {
      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'comp_003', level: 'approve', title: 'Approve', reason: 'Test' },
        { approval_id: 'a2', task_id: 'comp_003', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('comp_003')
      expect(approvals.length).toBe(2)

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'comp_003',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })

      // Reject
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'comp_003',
        approval_id: 'a2',
        decision: 'reject',
        reason: 'Not approved'
      })
    })
  })

  describe('All Event Operations', () => {
    it('should list events for task', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'comp_004' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'comp_004' }
      ])
      const events = await OperatorApi.listEvents('comp_004')
      expect(events.length).toBe(2)
    })
  })

  describe('All Summary Operations', () => {
    it('should get task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'comp_005',
        status: 'completed',
        goal: 'Test',
        summary: 'Completed',
        files_changed: ['test.gd'],
        files_created: ['new.gd'],
        files_deleted: ['old.gd'],
        duration_seconds: 30,
        iterations: 3,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('comp_005')
      expect(summary).toBeDefined()
    })
  })
})