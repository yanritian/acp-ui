// Operator API Tests
// Testing Operator API functionality

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

describe('Operator API Tests', () => {
  describe('Task Operations', () => {
    it('should create task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'api_001',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test API'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should get task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'api_002',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const task = await OperatorApi.getTask('api_002')
      expect(task.task_id).toBe('api_002')
    })

    it('should list tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 't1', status: 'running', goal: 'Task 1' },
        { task_id: 't2', status: 'completed', goal: 'Task 2' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(2)
    })
  })

  describe('Approval Operations', () => {
    it('should get pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 't1', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('t1')
      expect(approvals.length).toBe(1)
    })

    it('should approve request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 't1',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })

    it('should reject request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 't1',
        approval_id: 'a1',
        decision: 'reject',
        reason: 'Not approved'
      })
    })
  })

  describe('Event Operations', () => {
    it('should list events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events.length).toBe(1)
    })
  })

  describe('File Operations', () => {
    it('should read file', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'file content',
        path: 'test.gd'
      })
      const result = await OperatorApi.fileRead('t1', 'test.gd')
      expect(result.content).toBeDefined()
    })

    it('should patch file', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'test.gd'
      })
      const result = await OperatorApi.filePatch('t1', 'test.gd', 'new content')
      expect(result.success).toBe(true)
    })

    it('should preview patch', async () => {
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new line',
        lines_added: 1,
        lines_removed: 0
      })
      const result = await OperatorApi.filePatchPreview('t1', 'test.gd', 'new content')
      expect(result.lines_added).toBe(1)
    })

    it('should list files', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['file1.gd', 'file2.gd'],
        total: 2
      })
      const result = await OperatorApi.fileList('t1', 'scripts')
      expect(result.files.length).toBe(2)
    })
  })

  describe('Task Control Operations', () => {
    it('should pause task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('t1')
    })

    it('should resume task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('t1')
    })

    it('should stop task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('t1')
    })

    it('should redirect task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 't1',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })
  })

  describe('Summary Operations', () => {
    it('should get task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'completed',
        goal: 'Test',
        summary: 'Completed',
        files_changed: ['test.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('t1')
      expect(summary).toBeDefined()
    })
  })
})