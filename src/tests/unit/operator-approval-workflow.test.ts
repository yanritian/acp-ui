// Operator Approval Workflow Tests
// Testing approval workflow and policies

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

describe('Operator Approval Workflow Tests', () => {
  describe('Approval Policies', () => {
    it('should apply safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'approval_001',
        status: 'running',
        event_stream: '',
        approval_policy: 'safe_default'
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'safe_default'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should apply permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'approval_002',
        status: 'running',
        event_stream: '',
        approval_policy: 'permissive'
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'permissive'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should apply strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'approval_003',
        status: 'waiting_approval',
        event_stream: '',
        approval_policy: 'strict'
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Approval Requests', () => {
    it('should create approval request for file modification', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'approval_004', level: 'approve', title: 'Modify File', reason: 'File modification required' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('approval_004')
      expect(approvals.length).toBe(1)
      expect(approvals[0].level).toBe('approve')
    })

    it('should create multiple approval requests', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'approval_005', level: 'approve', title: 'Modify 1', reason: 'File 1' },
        { approval_id: 'a2', task_id: 'approval_005', level: 'approve', title: 'Modify 2', reason: 'File 2' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('approval_005')
      expect(approvals.length).toBe(2)
    })

    it('should include preview in approval request', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'a1',
          task_id: 'approval_006',
          level: 'approve',
          title: 'Modify File',
          reason: 'File modification',
          preview: { files: ['test.gd'], diff: '+ new line' }
        }
      ])
      const approvals = await OperatorApi.getPendingApprovals('approval_006')
      expect(approvals[0].preview).toBeDefined()
    })
  })

  describe('Approval Decisions', () => {
    it('should approve request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'approval_007',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'Looks good'
      })
    })

    it('should reject request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'approval_008',
        approval_id: 'a1',
        decision: 'reject',
        reason: 'Not approved'
      })
    })

    it('should request changes', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'approval_009',
        approval_id: 'a1',
        decision: 'request_changes',
        reason: 'Needs modification'
      })
    })
  })

  describe('Approval Queue', () => {
    it('should list pending approvals', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'approval_010', level: 'approve', title: 'Request 1', reason: 'Test' },
        { approval_id: 'a2', task_id: 'approval_010', level: 'approve', title: 'Request 2', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('approval_010')
      expect(approvals.length).toBe(2)
    })

    it('should clear approval after decision', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: 'approval_011',
        approval_id: 'a1',
        decision: 'approve',
        reason: 'OK'
      })
    })

    it('should handle empty approval queue', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const approvals = await OperatorApi.getPendingApprovals('approval_012')
      expect(approvals.length).toBe(0)
    })
  })

  describe('Approval Events', () => {
    it('should generate approval_requested event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'approval_requested', title: 'Approval Requested', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'approval_013' }
      ])
      const events = await OperatorApi.listEvents('approval_013')
      expect(events[0].type).toBe('approval_requested')
    })

    it('should generate approval_granted event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e2', type: 'approval_granted', title: 'Approval Granted', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'approval_014' }
      ])
      const events = await OperatorApi.listEvents('approval_014')
      expect(events[0].type).toBe('approval_granted')
    })

    it('should generate approval_rejected event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e3', type: 'approval_rejected', title: 'Approval Rejected', level: 'warning', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'approval_015' }
      ])
      const events = await OperatorApi.listEvents('approval_015')
      expect(events[0].type).toBe('approval_rejected')
    })
  })

  describe('Approval Validation', () => {
    it('should validate approval ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval ID'))
      await expect(OperatorApi.approve({
        task_id: 'approval_016',
        approval_id: 'invalid',
        decision: 'approve',
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })

    it('should validate task ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID'))
      await expect(OperatorApi.getPendingApprovals('')).rejects.toThrow('Invalid')
    })

    it('should validate decision', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid decision'))
      await expect(OperatorApi.approve({
        task_id: 'approval_017',
        approval_id: 'a1',
        decision: 'invalid' as any,
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })
})