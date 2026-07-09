// Approval Workflow Tests for Hermes Game Operator
// Comprehensive tests for approval queue, levels, and decisions

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock Tauri invoke
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

describe('Approval Workflow Tests', () => {
  describe('Approval Levels', () => {
    it('should handle silent approval (auto-approve)', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_silent',
        approval_id: 'approval_silent',
        decision: 'approve'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_silent',
          approval_id: 'approval_silent',
          decision: 'approve'
        }
      })
    })

    it('should handle notify level (informational)', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_notify',
        task_id: 'task_notify',
        level: 'notify',
        action: 'file.read',
        title: 'File read notification',
        reason: 'Reading Player.gd',
        options: ['approve'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_notify')
      expect(approvals[0].level).toBe('notify')
    })

    it('should handle approve level (requires user action)', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_required',
        task_id: 'task_approve',
        level: 'approve',
        action: 'file.patch',
        title: 'Modify Player.gd',
        reason: 'Adding jump logic',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_approve')
      expect(approvals[0].level).toBe('approve')
      expect(approvals[0].options).toContain('reject')
    })

    it('should handle forbidden level (blocked)', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Operation forbidden'))

      await expect(OperatorApi.approve({
        task_id: 'task_forbidden',
        approval_id: 'approval_forbidden',
        decision: 'approve'
      })).rejects.toThrow('forbidden')
    })
  })

  describe('Approval Decisions', () => {
    it('should handle approve decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_approve_1',
        approval_id: 'approval_1',
        decision: 'approve',
        comment: 'Approved for implementation'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_approve_1',
          approval_id: 'approval_1',
          decision: 'approve',
          comment: 'Approved for implementation'
        }
      })
    })

    it('should handle reject decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_reject_1',
        approval_id: 'approval_2',
        decision: 'reject',
        comment: 'Rejected due to security concerns'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_reject_1',
          approval_id: 'approval_2',
          decision: 'reject',
          comment: 'Rejected due to security concerns'
        }
      })
    })

    it('should handle request_changes decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_changes_1',
        approval_id: 'approval_3',
        decision: 'request_changes',
        comment: 'Please add unit tests'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_changes_1',
          approval_id: 'approval_3',
          decision: 'request_changes',
          comment: 'Please add unit tests'
        }
      })
    })
  })

  describe('Approval Queue Operations', () => {
    it('should list pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_1',
          task_id: 'task_queue',
          level: 'approve',
          action: 'file.patch',
          title: 'Modify script',
          reason: 'Add feature',
          options: ['approve', 'reject'],
          created_at: '2026-07-09T10:00:00Z'
        },
        {
          approval_id: 'approval_2',
          task_id: 'task_queue',
          level: 'approve',
          action: 'file.write',
          title: 'Create new file',
          reason: 'New utility class',
          options: ['approve', 'reject'],
          created_at: '2026-07-09T10:01:00Z'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_queue')
      expect(approvals).toHaveLength(2)
      expect(approvals[0].action).toBe('file.patch')
      expect(approvals[1].action).toBe('file.write')
    })

    it('should return empty list when no approvals', async () => {
      mockInvoke.mockResolvedValueOnce([])

      const approvals = await OperatorApi.getPendingApprovals('task_no_approvals')
      expect(approvals).toHaveLength(0)
    })

    it('should process approvals in order', async () => {
      const taskId = 'task_order'

      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'approval_1', task_id: taskId, level: 'approve', action: 'file.patch', title: 'First', reason: '1', options: ['approve'], created_at: '2026-07-09T10:00:00Z' },
        { approval_id: 'approval_2', task_id: taskId, level: 'approve', action: 'file.patch', title: 'Second', reason: '2', options: ['approve'], created_at: '2026-07-09T10:01:00Z' }
      ])
      const approvals = await OperatorApi.getPendingApprovals(taskId)
      expect(approvals).toHaveLength(2)

      // Approve first
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({ task_id: taskId, approval_id: 'approval_1', decision: 'approve' })

      // Approve second
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({ task_id: taskId, approval_id: 'approval_2', decision: 'approve' })

      expect(mockInvoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('Approval with Preview', () => {
    it('should show file diff preview', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_diff',
        task_id: 'task_preview',
        level: 'approve',
        action: 'file.patch',
        title: 'Modify Player.gd',
        reason: 'Adding jump logic',
        preview: {
          files: ['scripts/Player.gd'],
          diff_id: 'diff_001'
        },
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_preview')
      expect(approvals[0].preview?.files).toContain('scripts/Player.gd')
    })

    it('should show command preview', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_cmd',
        task_id: 'task_cmd',
        level: 'approve',
        action: 'shell.command',
        title: 'Run build command',
        reason: 'Build project',
        preview: {
          command: 'cargo build --release'
        },
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_cmd')
      expect(approvals[0].preview?.command).toBe('cargo build --release')
    })
  })

  describe('Approval Risk Assessment', () => {
    it('should show risk level for dangerous operations', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_risk',
        task_id: 'task_risk',
        level: 'approve',
        action: 'file.delete',
        title: 'Delete file',
        reason: 'Remove obsolete code',
        risk: 'high - irreversible operation',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_risk')
      expect(approvals[0].risk).toContain('high')
    })

    it('should show low risk for safe operations', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_safe',
        task_id: 'task_safe',
        level: 'approve',
        action: 'file.read',
        title: 'Read file',
        reason: 'Analyze existing code',
        risk: 'low - read-only operation',
        options: ['approve'],
        created_at: '2026-07-09T10:00:00Z'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_safe')
      expect(approvals[0].risk).toContain('low')
    })
  })

  describe('Approval Resolution Tracking', () => {
    it('should track approval resolution', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_resolved',
        task_id: 'task_resolved',
        level: 'approve',
        action: 'file.patch',
        title: 'Modify file',
        reason: 'Add feature',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z',
        resolved_at: '2026-07-09T10:05:00Z',
        decision: 'approve',
        resolved_by: 'user_001'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_resolved')
      expect(approvals[0].decision).toBe('approve')
      expect(approvals[0].resolved_by).toBe('user_001')
    })

    it('should track rejection details', async () => {
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_rejected',
        task_id: 'task_rejected',
        level: 'approve',
        action: 'file.delete',
        title: 'Delete file',
        reason: 'Remove file',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T10:00:00Z',
        resolved_at: '2026-07-09T10:05:00Z',
        decision: 'reject'
      }])

      const approvals = await OperatorApi.getPendingApprovals('task_rejected')
      expect(approvals[0].decision).toBe('reject')
    })
  })

  describe('Approval Policy Modes', () => {
    it('should work with safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_safe_policy',
        status: 'waiting_approval',
        event_stream: ''
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'safe_default'
      })

      expect(result.task_id).toBeDefined()
    })

    it('should work with permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_permissive',
        status: 'running',
        event_stream: ''
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'permissive'
      })

      expect(result.task_id).toBeDefined()
    })

    it('should work with strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_strict',
        status: 'waiting_approval',
        event_stream: ''
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })

      expect(result.task_id).toBeDefined()
    })
  })

  describe('Batch Approval Operations', () => {
    it('should handle multiple approvals concurrently', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      await Promise.all([
        OperatorApi.approve({ task_id: 'task_1', approval_id: 'approval_1', decision: 'approve' }),
        OperatorApi.approve({ task_id: 'task_2', approval_id: 'approval_2', decision: 'approve' }),
        OperatorApi.approve({ task_id: 'task_3', approval_id: 'approval_3', decision: 'approve' })
      ])

      expect(mockInvoke).toHaveBeenCalledTimes(3)
    })

    it('should handle mixed approval decisions', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      await Promise.all([
        OperatorApi.approve({ task_id: 'task_mix', approval_id: 'approval_1', decision: 'approve' }),
        OperatorApi.approve({ task_id: 'task_mix', approval_id: 'approval_2', decision: 'reject' }),
        OperatorApi.approve({ task_id: 'task_mix', approval_id: 'approval_3', decision: 'request_changes' })
      ])

      expect(mockInvoke).toHaveBeenCalledTimes(3)
    })
  })
})