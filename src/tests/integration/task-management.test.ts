// Task Management Integration Tests for Hermes Game Operator
// Tests comprehensive task lifecycle operations

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

describe('Task Management Integration Tests', () => {
  describe('Task Creation', () => {
    it('should create task with minimal params', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_min',
        status: 'planning',
        event_stream: 'operator://tasks/task_min/events'
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:/test/project',
        goal: 'Add feature'
      })

      expect(result.task_id).toBe('task_min')
      expect(result.status).toBe('planning')
    })

    it('should create task with all params', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_full',
        status: 'planning',
        event_stream: 'operator://tasks/task_full/events'
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:/test/project',
        goal: 'Implement player controller',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      })

      expect(result.task_id).toBeDefined()
      expect(mockInvoke).toHaveBeenCalledWith('operator_start_task', {
        request: {
          domain: 'game.godot',
          project_path: 'D:/test/project',
          goal: 'Implement player controller',
          mode: 'propose_then_apply',
          approval_policy: 'safe_default'
        }
      })
    })

    it('should create task with apply_directly mode', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_direct',
        status: 'running',
        event_stream: 'operator://tasks/task_direct/events'
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'apply_directly'
      })

      expect(result.task_id).toBeDefined()
    })
  })

  describe('Task Query Operations', () => {
    it('should get task by ID', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_query_1',
        status: 'running',
        domain: 'game.godot',
        project_path: 'D:/test/project',
        goal: 'Add feature',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T10:00:00Z',
        updated_at: '2026-07-09T10:05:00Z'
      })

      const task = await OperatorApi.getTask('task_query_1')
      expect(task.task_id).toBe('task_query_1')
      expect(task.status).toBe('running')
      expect(task.domain).toBe('game.godot')
    })

    it('should list all tasks', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_1', status: 'completed', goal: 'Goal 1' },
        { task_id: 'task_2', status: 'running', goal: 'Goal 2' },
        { task_id: 'task_3', status: 'planning', goal: 'Goal 3' }
      ])

      const tasks = await OperatorApi.listTasks()
      expect(tasks).toHaveLength(3)
      expect(tasks[0].status).toBe('completed')
      expect(tasks[1].status).toBe('running')
    })

    it('should return empty list when no tasks', async () => {
      mockInvoke.mockResolvedValueOnce([])

      const tasks = await OperatorApi.listTasks()
      expect(tasks).toHaveLength(0)
    })
  })

  describe('Task Control Operations', () => {
    it('should pause task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.pauseTask('task_pause')
      expect(mockInvoke).toHaveBeenCalledWith('operator_pause_task', {
        taskId: 'task_pause'
      })
    })

    it('should resume task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.resumeTask('task_resume')
      expect(mockInvoke).toHaveBeenCalledWith('operator_resume_task', {
        taskId: 'task_resume'
      })
    })

    it('should stop task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.stopTask('task_stop')
      expect(mockInvoke).toHaveBeenCalledWith('operator_stop_task', {
        taskId: 'task_stop'
      })
    })

    it('should redirect task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.redirectTask({
        task_id: 'task_redirect',
        new_goal: 'New implementation goal',
        preserve_completed_work: true
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_redirect_task', {
        request: {
          task_id: 'task_redirect',
          new_goal: 'New implementation goal',
          preserve_completed_work: true
        }
      })
    })

    it('should redirect without preserving work', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.redirectTask({
        task_id: 'task_redirect_2',
        new_goal: 'Fresh start',
        preserve_completed_work: false
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_redirect_task', {
        request: {
          task_id: 'task_redirect_2',
          new_goal: 'Fresh start',
          preserve_completed_work: false
        }
      })
    })
  })

  describe('Task Summary', () => {
    it('should get completed task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_summary_1',
        status: 'completed',
        goal: 'Add jump feature',
        summary: 'Successfully added jump feature to Player.gd',
        files_changed: ['scripts/Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 180,
        iterations: 12,
        errors: [],
        warnings: ['Minor style issue']
      })

      const summary = await OperatorApi.getTaskSummary('task_summary_1')
      expect(summary.status).toBe('completed')
      expect(summary.files_changed).toContain('scripts/Player.gd')
      expect(summary.duration_seconds).toBe(180)
    })

    it('should get failed task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_summary_failed',
        status: 'failed',
        goal: 'Add feature',
        summary: 'Task failed due to compilation error',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 45,
        iterations: 3,
        errors: ['Compilation failed: syntax error'],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_summary_failed')
      expect(summary.status).toBe('failed')
      expect(summary.errors).toHaveLength(1)
    })

    it('should get task summary with multiple file changes', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_summary_multi',
        status: 'completed',
        goal: 'Refactor',
        summary: 'Refactored multiple files',
        files_changed: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        files_created: ['Utils.gd'],
        files_deleted: ['OldScript.gd'],
        duration_seconds: 300,
        iterations: 25,
        errors: [],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_summary_multi')
      expect(summary.files_changed).toHaveLength(3)
      expect(summary.files_created).toHaveLength(1)
      expect(summary.files_deleted).toHaveLength(1)
    })
  })

  describe('Event Operations', () => {
    it('should list task events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_1', task_id: 'task_events', type: 'task_started', level: 'info', title: 'Started', timestamp: '2026-07-09T10:00:00Z', source: 'operator' },
        { event_id: 'evt_2', task_id: 'task_events', type: 'project_analyzed', level: 'info', title: 'Analyzed', timestamp: '2026-07-09T10:00:30Z', source: 'operator' },
        { event_id: 'evt_3', task_id: 'task_events', type: 'plan_ready', level: 'info', title: 'Plan ready', timestamp: '2026-07-09T10:01:00Z', source: 'operator' }
      ])

      const events = await OperatorApi.listEvents('task_events')
      expect(events).toHaveLength(3)
      expect(events[0].type).toBe('task_started')
    })

    it('should list events with limit', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_1', task_id: 'task_limit', type: 'task_started', level: 'info', title: 'Started', timestamp: '2026-07-09T10:00:00Z', source: 'operator' }
      ])

      const events = await OperatorApi.listEvents('task_limit', 1)
      expect(mockInvoke).toHaveBeenCalledWith('operator_list_events', {
        taskId: 'task_limit',
        limit: 1
      })
    })

    it('should return empty events list', async () => {
      mockInvoke.mockResolvedValueOnce([])

      const events = await OperatorApi.listEvents('task_no_events')
      expect(events).toHaveLength(0)
    })
  })

  describe('Task State Transitions', () => {
    it('should track state: idle -> planning -> running', async () => {
      const taskId = 'task_transitions'

      // Start (idle -> planning)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'planning', event_stream: '' })
      const start = await OperatorApi.startTask({ domain: 'game.godot', project_path: '/test', goal: 'Test' })
      expect(start.status).toBe('planning')

      // Get task (running)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:01:00Z' })
      const running = await OperatorApi.getTask(taskId)
      expect(running.status).toBe('running')
    })

    it('should track state: running -> paused -> running', async () => {
      const taskId = 'task_pause_resume'

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask(taskId)

      // Get task (paused)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'paused', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:02:00Z' })
      const paused = await OperatorApi.getTask(taskId)
      expect(paused.status).toBe('paused')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask(taskId)

      // Get task (running)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:03:00Z' })
      const resumed = await OperatorApi.getTask(taskId)
      expect(resumed.status).toBe('running')
    })

    it('should track state: running -> cancelled', async () => {
      const taskId = 'task_cancel'

      // Stop
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask(taskId)

      // Get task (cancelled)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'cancelled', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:05:00Z' })
      const cancelled = await OperatorApi.getTask(taskId)
      expect(cancelled.status).toBe('cancelled')
    })

    it('should track state: running -> completed', async () => {
      const taskId = 'task_complete'

      // Get task (completed)
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'completed', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:10:00Z', completed_at: '2026-07-09T10:10:00Z', summary: 'Task completed successfully' })
      const completed = await OperatorApi.getTask(taskId)
      expect(completed.status).toBe('completed')
      expect(completed.summary).toBeDefined()
    })
  })

  describe('Concurrent Task Operations', () => {
    it('should handle concurrent getTask calls', async () => {
      mockInvoke.mockResolvedValue({ task_id: 'task_concurrent', status: 'running', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:00:00Z' })

      const results = await Promise.all([
        OperatorApi.getTask('task_concurrent'),
        OperatorApi.getTask('task_concurrent'),
        OperatorApi.getTask('task_concurrent')
      ])

      expect(results).toHaveLength(3)
    })

    it('should handle concurrent listTasks and getTask', async () => {
      mockInvoke.mockResolvedValueOnce([{ task_id: 'task_1', status: 'running', goal: 'Test' }])
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_2', status: 'completed', domain: 'game.godot', project_path: '/test', goal: 'Test', mode: 'propose_then_apply', approval_policy: 'safe_default', created_at: '2026-07-09T10:00:00Z', updated_at: '2026-07-09T10:00:00Z' })

      const [tasks, task] = await Promise.all([
        OperatorApi.listTasks(),
        OperatorApi.getTask('task_2')
      ])

      expect(tasks).toHaveLength(1)
      expect(task.task_id).toBe('task_2')
    })
  })

  describe('Task Domain Support', () => {
    it('should create Godot domain task', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 'task_godot', status: 'planning', event_stream: '' })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:/games/my-game',
        goal: 'Add player movement'
      })

      expect(result.task_id).toBeDefined()
    })

    it('should validate domain format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid domain format'))

      await expect(OperatorApi.startTask({
        domain: 'invalid-domain',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Invalid domain')
    })
  })
})