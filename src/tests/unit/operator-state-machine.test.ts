// Operator State Machine Tests
// Tests for state transitions and event handling

import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock Tauri invoke
const mockInvoke = vi.fn()

beforeEach(() => {
  mockInvoke.mockReset()
  ;(globalThis as any).__TAURI_INTERNALS__ = {
    invoke: mockInvoke
  }
})

describe('Operator State Machine', () => {
  describe('State Transitions', () => {
    it('should start in idle state', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_001',
        status: 'planning',
        event_stream: 'operator://tasks/task_001/events'
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test/project',
        goal: 'Test goal'
      })

      expect(result.status).toBe('planning')
      expect(result.task_id).toBeDefined()
    })

    it('should transition from planning to waiting_approval', async () => {
      // First call to get task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_002',
        status: 'waiting_approval',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T10:00:00Z',
        updated_at: '2026-07-09T10:01:00Z'
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const task = await OperatorApi.getTask('task_002')

      expect(task.status).toBe('waiting_approval')
    })

    it('should transition from waiting_approval to running on approve', async () => {
      mockInvoke.mockResolvedValueOnce(undefined) // approve returns void

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.approve({
        task_id: 'task_003',
        approval_id: 'approval_001',
        decision: 'approve'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_003',
          approval_id: 'approval_001',
          decision: 'approve'
        }
      })
    })

    it('should handle pause transition', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.pauseTask('task_004')

      expect(mockInvoke).toHaveBeenCalledWith('operator_pause_task', {
        taskId: 'task_004'
      })
    })

    it('should handle resume transition', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.resumeTask('task_005')

      expect(mockInvoke).toHaveBeenCalledWith('operator_resume_task', {
        taskId: 'task_005'
      })
    })

    it('should handle stop transition', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.stopTask('task_006')

      expect(mockInvoke).toHaveBeenCalledWith('operator_stop_task', {
        taskId: 'task_006'
      })
    })

    it('should handle redirect transition', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.redirectTask({
        task_id: 'task_007',
        new_goal: 'New goal',
        preserve_completed_work: true
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_redirect_task', {
        request: {
          task_id: 'task_007',
          new_goal: 'New goal',
          preserve_completed_work: true
        }
      })
    })
  })

  describe('Event History', () => {
    it('should track event history', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_001', event_type: 'task_created', title: 'Task created' },
        { event_id: 'evt_002', event_type: 'plan_started', title: 'Planning started' },
        { event_id: 'evt_003', event_type: 'plan_ready', title: 'Plan ready' }
      ])

      const { OperatorApi } = await import('@/api/operatorApi')
      const events = await OperatorApi.listEvents('task_008', 100)

      expect(events).toHaveLength(3)
      expect(events[0].event_type).toBe('task_created')
    })

    it('should limit event history', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_001', event_type: 'task_created', title: 'Task created' }
      ])

      const { OperatorApi } = await import('@/api/operatorApi')
      const events = await OperatorApi.listEvents('task_009', 1)

      expect(mockInvoke).toHaveBeenCalledWith('operator_list_events', {
        taskId: 'task_009',
        limit: 1
      })
    })
  })

  describe('Approval Workflow', () => {
    it('should get pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_001',
          task_id: 'task_010',
          level: 'approve',
          action: 'file_write',
          title: 'Modify Player.gd',
          reason: 'Adding double jump feature',
          options: ['approve', 'reject']
        }
      ])

      const { OperatorApi } = await import('@/api/operatorApi')
      const approvals = await OperatorApi.getPendingApprovals('task_010')

      expect(approvals).toHaveLength(1)
      expect(approvals[0].action).toBe('file_write')
    })

    it('should reject approval request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.approve({
        task_id: 'task_011',
        approval_id: 'approval_002',
        decision: 'reject',
        comment: 'Too risky'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_011',
          approval_id: 'approval_002',
          decision: 'reject',
          comment: 'Too risky'
        }
      })
    })

    it('should request changes on approval', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { OperatorApi } = await import('@/api/operatorApi')
      await OperatorApi.approve({
        task_id: 'task_012',
        approval_id: 'approval_003',
        decision: 'request_changes',
        comment: 'Need more tests'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_012',
          approval_id: 'approval_003',
          decision: 'request_changes',
          comment: 'Need more tests'
        }
      })
    })
  })

  describe('Task Summary', () => {
    it('should generate task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_013',
        status: 'completed',
        goal: 'Add double jump',
        summary: 'Successfully added double jump to Player.gd',
        files_changed: ['scripts/Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 120,
        iterations: 15,
        errors: [],
        warnings: []
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_013')

      expect(summary.status).toBe('completed')
      expect(summary.files_changed).toContain('scripts/Player.gd')
      expect(summary.duration_seconds).toBe(120)
    })

    it('should track errors in summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_014',
        status: 'failed',
        goal: 'Test goal',
        summary: 'Task failed with errors',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: ['File not found', 'Permission denied'],
        warnings: ['Deprecated API']
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_014')

      expect(summary.errors).toHaveLength(2)
      expect(summary.warnings).toHaveLength(1)
    })
  })

  describe('Task Listing', () => {
    it('should list all tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_001', status: 'completed', goal: 'Task 1' },
        { task_id: 'task_002', status: 'running', goal: 'Task 2' },
        { task_id: 'task_003', status: 'planning', goal: 'Task 3' }
      ])

      const { OperatorApi } = await import('@/api/operatorApi')
      const tasks = await OperatorApi.listTasks()

      expect(tasks).toHaveLength(3)
    })
  })
})