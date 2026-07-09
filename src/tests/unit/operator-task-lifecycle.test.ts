// Operator Task Lifecycle Tests
// Testing complete task lifecycle from creation to completion

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

describe('Operator Task Lifecycle Tests', () => {
  describe('Task Creation Phase', () => {
    it('should create task with minimal parameters', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_001',
        status: 'idle',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test goal'
      })
      expect(task.task_id).toBe('lifecycle_001')
      expect(task.status).toBe('idle')
    })

    it('should create task with all parameters', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_002',
        status: 'planning',
        event_stream: '',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Complete goal',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      })
      expect(task.task_id).toBe('lifecycle_002')
      expect(task.status).toBe('planning')
    })
  })

  describe('Planning Phase', () => {
    it('should transition to planning state', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_003',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Plan test'
      })
      expect(task.status).toBe('planning')
    })

    it('should generate plan_ready event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'plan_ready', title: 'Plan Ready', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'lifecycle_004' }
      ])
      const events = await OperatorApi.listEvents('lifecycle_004')
      expect(events[0].type).toBe('plan_ready')
    })
  })

  describe('Waiting Approval Phase', () => {
    it('should transition to waiting_approval', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_005',
        status: 'waiting_approval',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Approval test',
        approval_policy: 'strict'
      })
      expect(task.status).toBe('waiting_approval')
    })

    it('should request approval', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'lifecycle_006', level: 'approve', title: 'Approve', reason: 'Test' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('lifecycle_006')
      expect(approvals.length).toBe(1)
    })
  })

  describe('Running Phase', () => {
    it('should transition to running', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_007',
        status: 'running',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Running test',
        approval_policy: 'permissive'
      })
      expect(task.status).toBe('running')
    })

    it('should execute tools', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'test content',
        path: 'test.gd'
      })
      const result = await OperatorApi.fileRead('lifecycle_008', 'test.gd')
      expect(result.content).toBeDefined()
    })

    it('should modify files', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'test.gd'
      })
      const result = await OperatorApi.filePatch('lifecycle_009', 'test.gd', 'new content')
      expect(result.success).toBe(true)
    })
  })

  describe('Pausing and Resuming', () => {
    it('should pause running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('lifecycle_010')
    })

    it('should resume paused task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('lifecycle_011')
    })

    it('should pause and resume multiple times', async () => {
      mockInvoke.mockResolvedValue(undefined)
      await OperatorApi.pauseTask('lifecycle_012')
      await OperatorApi.resumeTask('lifecycle_012')
      await OperatorApi.pauseTask('lifecycle_012')
      await OperatorApi.resumeTask('lifecycle_012')
    })
  })

  describe('Completion Phase', () => {
    it('should transition to completed', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_013',
        status: 'completed',
        goal: 'Complete test',
        summary: 'Successfully completed',
        files_changed: ['test.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 60,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('lifecycle_013')
      expect(summary.status).toBe('completed')
    })

    it('should generate task_completed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'lifecycle_014' }
      ])
      const events = await OperatorApi.listEvents('lifecycle_014')
      expect(events[0].type).toBe('task_completed')
    })
  })

  describe('Failure Phase', () => {
    it('should transition to failed', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'lifecycle_015',
        status: 'failed',
        goal: 'Failed test',
        summary: 'Task failed',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 10,
        iterations: 1,
        errors: ['Critical error'],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary('lifecycle_015')
      expect(summary.status).toBe('failed')
    })

    it('should generate task_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_failed', title: 'Failed', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'lifecycle_016' }
      ])
      const events = await OperatorApi.listEvents('lifecycle_016')
      expect(events[0].type).toBe('task_failed')
    })
  })

  describe('Cancellation Phase', () => {
    it('should cancel running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('lifecycle_017')
    })

    it('should generate task_cancelled event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_cancelled', title: 'Cancelled', level: 'warning', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'lifecycle_018' }
      ])
      const events = await OperatorApi.listEvents('lifecycle_018')
      expect(events[0].type).toBe('task_cancelled')
    })
  })

  describe('Redirect Phase', () => {
    it('should redirect running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: 'lifecycle_019',
        new_goal: 'New goal',
        preserve_completed_work: true
      })
    })

    it('should generate task_redirected event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_redirected', title: 'Redirected', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'lifecycle_020' }
      ])
      const events = await OperatorApi.listEvents('lifecycle_020')
      expect(events[0].type).toBe('task_redirected')
    })
  })
})