// Operator Workflow Tests
// Testing complete workflow scenarios

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

describe('Operator Workflow Tests', () => {
  describe('Complete Workflow', () => {
    it('should complete create-analyze-plan-execute workflow', async () => {
      // Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'workflow_001',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete workflow test'
      })

      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      await OperatorApi.fileRead('workflow_001', 'Player.gd')

      // Patch file
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      await OperatorApi.filePatch('workflow_001', 'Player.gd', 'new content')

      // Get summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'workflow_001',
        status: 'completed',
        goal: 'Complete workflow test',
        summary: 'Completed',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('workflow_001')
      expect(summary).toBeDefined()
    })
  })

  describe('Approval Workflow', () => {
    it('should complete approval workflow', async () => {
      // Create task with strict policy
      mockInvoke.mockResolvedValueOnce({
        task_id: 'approval_workflow_001',
        status: 'waiting_approval',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Approval workflow',
        approval_policy: 'strict'
      })

      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'approval_workflow_001', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('approval_workflow_001')
      expect(approvals.length).toBe(1)

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'approval_workflow_001',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Approved'
      })
    })
  })

  describe('Error Recovery Workflow', () => {
    it('should handle error and recover', async () => {
      // Try to read file that doesn't exist
      mockInvoke.mockRejectedValueOnce(new Error('File not found'))
      try {
        await OperatorApi.fileRead('error_workflow_001', 'missing.gd')
      } catch (e) {
        // Expected error
      }

      // Read existing file
      mockInvoke.mockResolvedValueOnce({
        content: 'file content',
        path: 'existing.gd'
      })
      const result = await OperatorApi.fileRead('error_workflow_001', 'existing.gd')
      expect(result.content).toBeDefined()
    })
  })

  describe('Multi-Task Workflow', () => {
    it('should handle multiple tasks concurrently', async () => {
      // Create multiple tasks
      mockInvoke.mockResolvedValue({ task_id: 'multi', status: 'running', event_stream: '' })

      await Promise.all([
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/t1', goal: 'Task 1' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/t2', goal: 'Task 2' }),
        OperatorApi.startTask({ domain: 'game.godot', project_path: '/t3', goal: 'Task 3' })
      ])

      // List tasks
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'multi', status: 'running', goal: 'Task 1' },
        { task_id: 'multi', status: 'running', goal: 'Task 2' },
        { task_id: 'multi', status: 'running', goal: 'Task 3' }
      ])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(3)
    })
  })
})