// Operator Final Ultimate Tests
// Final ultimate comprehensive testing

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

describe('Operator Final Ultimate Tests', () => {
  describe('Complete Task Lifecycle', () => {
    it('should complete full task lifecycle with all operations', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_ultimate_001',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Final ultimate test'
      })
      expect(task.task_id).toBe('final_ultimate_001')

      // Get task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_ultimate_001',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Final ultimate test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const getTask = await OperatorApi.getTask('final_ultimate_001')
      expect(getTask.status).toBe('running')

      // List tasks
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'final_ultimate_001', status: 'running', goal: 'Final ultimate test' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(1)
    })
  })

  describe('Complete File Operations', () => {
    it('should complete full file operations cycle', async () => {
      // List files
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd'],
        total: 2
      })
      const list = await OperatorApi.fileList('final_ultimate_002', 'scripts')
      expect(list.files.length).toBe(2)

      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const read = await OperatorApi.fileRead('final_ultimate_002', 'Player.gd')
      expect(read.content).toBeDefined()

      // Preview patch
      mockInvoke.mockResolvedValueOnce({
        diff: '+ new function',
        lines_added: 1,
        lines_removed: 0
      })
      const preview = await OperatorApi.filePatchPreview('final_ultimate_002', 'Player.gd', 'new content')
      expect(preview.lines_added).toBe(1)

      // Apply patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      const patch = await OperatorApi.filePatch('final_ultimate_002', 'Player.gd', 'new content')
      expect(patch.success).toBe(true)
    })
  })

  describe('Complete Approval Workflow', () => {
    it('should complete full approval workflow', async () => {
      // Get approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'final_ultimate_003', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('final_ultimate_003')
      expect(approvals.length).toBe(1)

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'final_ultimate_003',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Approved'
      })
    })
  })

  describe('Complete Event Tracking', () => {
    it('should track all events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'final_ultimate_004' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'final_ultimate_004' },
        { event_id: 'e3', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'final_ultimate_004' }
      ])
      const events = await OperatorApi.listEvents('final_ultimate_004')
      expect(events.length).toBe(3)
    })
  })

  describe('Complete Summary Generation', () => {
    it('should generate complete summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_ultimate_005',
        status: 'completed',
        goal: 'Complete task',
        summary: 'Successfully completed',
        files_changed: ['Player.gd'],
        files_created: ['NewScript.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('final_ultimate_005')
      expect(summary).toBeDefined()
    })
  })
})