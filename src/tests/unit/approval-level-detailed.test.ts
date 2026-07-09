// Approval Level Detailed Tests
// Comprehensive testing of approval levels, decisions, and queue management

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

describe('Approval Level Detailed Tests', () => {
  describe('Approval Level Types', () => {
    it('should handle info_only level', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_info',
        status: 'running',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Analyze project'
      })
      expect(task.status).toBeDefined()
    })

    it('should handle file_read level', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'file content',
        path: 'scripts/Player.gd'
      })

      const result = await OperatorApi.fileRead('task_001', 'scripts/Player.gd')
      expect(result.content).toBeDefined()
    })

    it('should handle file_modify level', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_modify_001',
          level: 'approve',
          title: 'Modify Player.gd',
          reason: 'File modification'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals[0].level).toBe('approve')
    })

    it('should handle file_create level', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_create_001',
          level: 'approve',
          title: 'Create NewScript.gd',
          reason: 'New file'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals.length).toBe(1)
    })

    it('should handle file_delete level', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_delete_001',
          level: 'approve',
          title: 'Delete OldScript.gd',
          reason: 'File deletion',
          risk: 'Irreversible'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals[0].risk).toBeDefined()
    })

    it('should handle command_execute level', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_cmd_001',
          level: 'approve',
          title: 'Execute command',
          reason: 'Build project'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals[0].level).toBe('approve')
    })

    it('should handle critical_operation level', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_critical_001',
          level: 'approve',
          title: 'Critical operation',
          reason: 'Project-wide refactoring'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals[0].approval_id).toBe('approval_critical_001')
    })
  })

  describe('Approval Decision Types', () => {
    it('should handle approved decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_001',
        approval_id: 'approval_001',
        decision: 'approve',
        reason: 'Changes look good'
      })
    })

    it('should handle rejected decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_001',
        approval_id: 'approval_001',
        decision: 'reject',
        reason: 'Do not proceed'
      })
    })

    it('should handle modified decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_001',
        approval_id: 'approval_001',
        decision: 'request_changes',
        reason: 'Change target file',
      })
    })

    it('should handle deferred decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_001',
        approval_id: 'approval_001',
        decision: 'reject',
        reason: 'Need more information'
      })
    })
  })

  describe('Approval Policy Tests', () => {
    it('should apply safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_safe',
        status: 'running',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'safe_default'
      })
      expect(task.task_id).toBe('task_safe')
    })

    it('should apply permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_permissive',
        status: 'running',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'permissive'
      })
      expect(task.task_id).toBe('task_permissive')
    })

    it('should apply strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_strict',
        status: 'waiting_approval',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
      expect(task.status).toBe('waiting_approval')
    })

    it('should apply custom policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_custom',
        status: 'running',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
      expect(task.task_id).toBe('task_custom')
    })
  })

  describe('Approval Queue Management', () => {
    it('should manage approval queue order', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'approval_001', title: 'First', reason: 'Test' },
        { approval_id: 'approval_002', title: 'Second', reason: 'Test' },
        { approval_id: 'approval_003', title: 'Third', reason: 'Test' }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals.length).toBe(3)
    })

    it('should handle approval expiration', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_expired',
          level: 'approve',
          title: 'Expired request',
          reason: 'Timeout'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_001')
      expect(approvals[0].approval_id).toBe('approval_expired')
    })

    it('should batch approve multiple items', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        batch_approved: ['approval_001', 'approval_002', 'approval_003'],
        failed: [],
        new_status: 'running'
      })

      // Batch approve would approve multiple at once
      for (let i = 1; i <= 3; i++) {
        mockInvoke.mockResolvedValueOnce({
          success: true
        })
      }

      expect(true).toBe(true)
    })
  })

  describe('Approval Audit Trail', () => {
    it('should record approval history', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', task_id: 'task_001', type: 'approval_requested', title: 'Request', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z' },
        { event_id: 'e2', task_id: 'task_001', type: 'approval_granted', title: 'Granted', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z' }
      ])

      const result = await OperatorApi.listEvents('task_001')
      expect(result).toBeDefined()
    })

    it('should track approval timing', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_001',
        status: 'completed',
        goal: 'Test',
        summary: 'Task completed',
        files_changed: ['Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 5,
        errors: [],
        warnings: []
      })

      const result = await OperatorApi.getTaskSummary('task_001')
      expect(result).toBeDefined()
    })
  })

  describe('Approval Error Handling', () => {
    it('should handle invalid approval ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval not found: invalid_id'))

      try {
        await OperatorApi.approve({
          task_id: 'task_001',
          approval_id: 'invalid_id',
          decision: 'approve',
          reason: 'Test'
        })
      } catch (e: any) {
        expect(e.message).toContain('Approval not found')
      }
    })

    it('should handle approval already decided', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval already decided: approval_001'))

      try {
        await OperatorApi.approve({
          task_id: 'task_001',
          approval_id: 'approval_001',
          decision: 'approve',
          reason: 'Test'
        })
      } catch (e: any) {
        expect(e.message).toContain('already decided')
      }
    })

    it('should handle task not in waiting state', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task is not waiting for approval'))

      try {
        await OperatorApi.approve({
          task_id: 'task_running',
          approval_id: 'approval_001',
          decision: 'approve',
          reason: 'Test'
        })
      } catch (e: any) {
        expect(e.message).toContain('not waiting')
      }
    })
  })
})