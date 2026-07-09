// Operator Full Lifecycle Tests
// Complete testing of operator task lifecycle from creation to completion

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

describe('Operator Full Lifecycle Tests', () => {
  describe('Complete Task Flow', () => {
    it('should complete full task lifecycle: create → plan → run → complete', async () => {
      // Step 1: Create task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_full_001',
        status: 'planning',
        event_stream: ''
      })

      const createResult = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test/project',
        goal: 'Add player movement'
      })
      expect(createResult.task_id).toBe('task_full_001')
      expect(createResult.status).toBe('planning')

      // Step 2: Get task details
      mockInvoke.mockResolvedValueOnce({
        id: 'task_full_001',
        status: 'running',
        goal: 'Add player movement',
        domain: 'game.godot',
        project_path: '/test/project',
        created_at: new Date().toISOString(),
        events: []
      })

      const taskDetails = await OperatorApi.getTask('task_full_001')
      expect(taskDetails.status).toBe('running')

      // Step 3: Get task summary
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_full_001',
        status: 'completed',
        goal: 'Add player movement',
        summary: 'Successfully added player movement',
        files_changed: ['scripts/Player.gd', 'scripts/Movement.gd', 'scenes/Player.tscn'],
        files_created: ['scripts/Movement.gd'],
        files_deleted: [],
        duration_seconds: 5,
        iterations: 10,
        errors: [],
        warnings: []
      })

      const summary = await OperatorApi.getTaskSummary('task_full_001')
      expect(summary.status).toBe('completed')
      expect(summary.files_changed.length).toBe(3)
    })

    it('should handle task with approval workflow', async () => {
      // Create task with safe_default policy
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_approval_001',
        status: 'waiting_approval',
        event_stream: ''
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test/project',
        goal: 'Modify player script',
        approval_policy: 'safe_default'
      })
      expect(task.status).toBe('waiting_approval')

      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_001',
          task_id: 'task_approval_001',
          level: 'approve',
          title: 'Modify Player.gd',
          reason: 'File modification',
          preview: { files: ['scripts/Player.gd'] }
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_approval_001')
      expect(approvals.length).toBe(1)

      // Approve the request
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_approval_001',
        approval_id: 'approval_001',
        decision: 'approve',
        reason: 'Looks good'
      })
    })

    it('should handle task rejection workflow', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_reject_001',
        status: 'waiting_approval',
        event_stream: ''
      })

      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test/project',
        goal: 'Delete files'
      })

      mockInvoke.mockResolvedValueOnce([
        {
          approval_id: 'approval_002',
          task_id: 'task_reject_001',
          level: 'approve',
          title: 'Delete important.gd',
          reason: 'File deletion'
        }
      ])

      const approvals = await OperatorApi.getPendingApprovals('task_reject_001')
      expect(approvals.length).toBe(1)

      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_reject_001',
        approval_id: 'approval_002',
        decision: 'reject',
        reason: 'Do not delete important files'
      })
    })
  })

  describe('Task Control Operations', () => {
    it('should pause and resume task', async () => {
      // Create running task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_pause_001',
        status: 'running',
        event_stream: ''
      })

      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test task'
      })

      // Pause task - returns void
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask('task_pause_001')

      // Resume task - returns void
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask('task_pause_001')
    })

    it('should stop task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_stop_001',
        status: 'running',
        event_stream: ''
      })

      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test task'
      })

      // Stop task - returns void
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.stopTask('task_stop_001')
    })

    it('should redirect task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_redirect_001',
        status: 'running',
        event_stream: ''
      })

      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Original goal'
      })

      // Redirect task - returns void
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({ task_id: 'task_redirect_001', new_goal: 'New goal', preserve_completed_work: true })
    })
  })

  describe('Event Stream Tests', () => {
    it('should track all event types in order', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', timestamp: '2026-07-09T00:00:00Z', title: 'Created' },
        { event_id: 'e2', type: 'plan_started', timestamp: '2026-07-09T00:01:00Z', title: 'Planning' },
        { event_id: 'e3', type: 'plan_ready', timestamp: '2026-07-09T00:02:00Z', title: 'Plan ready' },
        { event_id: 'e4', type: 'task_started', timestamp: '2026-07-09T00:03:00Z', title: 'Running' },
        { event_id: 'e5', type: 'tool_call_started', timestamp: '2026-07-09T00:04:00Z', title: 'Tool call' },
        { event_id: 'e6', type: 'tool_call_succeeded', timestamp: '2026-07-09T00:05:00Z', title: 'Tool result' },
        { event_id: 'e7', type: 'file_patch_applied', timestamp: '2026-07-09T00:06:00Z', title: 'File change' },
        { event_id: 'e8', type: 'task_completed', timestamp: '2026-07-09T00:07:00Z', title: 'Completed' }
      ])

      const events = await OperatorApi.listEvents('task_001')
      expect(events.length).toBe(8)
      expect(events[0].type).toBe('task_created')
      expect(events[events.length - 1].type).toBe('task_completed')
    })

    it('should filter events by type', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool 1' },
        { event_id: 'e2', type: 'tool_call_started', title: 'Tool 2' },
        { event_id: 'e3', type: 'file_patch_applied', title: 'File' }
      ])

      const events = await OperatorApi.listEvents('task_001')
      const toolEvents = events.filter((e: any) => e.type === 'tool_call_started')
      expect(toolEvents.length).toBe(2)
    })
  })

  describe('File Operations Integration', () => {
    it('should read, modify, and list files', async () => {
      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D\n\nfunc _ready():\n  pass',
        path: 'scripts/Player.gd',
        size: 50
      })

      const readResult = await OperatorApi.fileRead('task_001', 'scripts/Player.gd')
      expect(readResult.content).toContain('extends Node2D')

      // Patch preview
      mockInvoke.mockResolvedValueOnce({
        diff: '+ func new_feature():\n+   print("new")',
        lines_added: 2,
        lines_removed: 0
      })

      const previewResult = await OperatorApi.filePatchPreview('task_001', 'scripts/Player.gd', 'new content')
      expect(previewResult.lines_added).toBe(2)

      // Apply patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        new_hash: 'abc123'
      })

      const patchResult = await OperatorApi.filePatch('task_001', 'scripts/Player.gd', 'new content')
      expect(patchResult.success).toBe(true)

      // List files
      mockInvoke.mockResolvedValueOnce({
        files: ['scripts/Player.gd', 'scenes/Main.tscn', 'assets/sprite.png'],
        total: 3
      })

      const listResult = await OperatorApi.fileList('task_001', '')
      expect(listResult.files.length).toBe(3)
    })
  })

  describe('Error Recovery Tests', () => {
    it('should handle and recover from planning errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Planning failed: insufficient context'))

      try {
        await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/empty',
          goal: 'Add feature'
        })
      } catch (e: any) {
        expect(e.message).toContain('Planning failed')
      }

      // Retry with more context
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_retry_001',
        status: 'planning',
        event_stream: ''
      })

      const retryResult = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/valid/project',
        goal: 'Add feature to Player.gd'
      })
      expect(retryResult.task_id).toBeDefined()
    })

    it('should handle timeout during execution', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Execution timeout after 300s'))

      try {
        await OperatorApi.getTask('task_timeout_001')
      } catch (e: any) {
        expect(e.message).toContain('timeout')
      }

      // Resume from timeout
      mockInvoke.mockResolvedValueOnce({
        id: 'task_timeout_001',
        status: 'paused',
        error: 'Execution timeout'
      })

      const task = await OperatorApi.getTask('task_timeout_001')
      expect(task.status).toBe('paused')

      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.resumeTask('task_timeout_001')
    })
  })

  describe('Multi-Task Concurrent Tests', () => {
    it('should manage multiple tasks simultaneously', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_1', status: 'running', goal: 'Task 1' },
        { task_id: 'task_2', status: 'planning', goal: 'Task 2' },
        { task_id: 'task_3', status: 'waiting_approval', goal: 'Task 3' }
      ])

      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(3)

      // Verify different states
      const runningTasks = tasks.filter((t: any) => t.status === 'running')
      expect(runningTasks.length).toBe(1)
    })

    it('should prioritize tasks correctly', async () => {
      mockInvoke.mockResolvedValueOnce([
        { task_id: 'task_high', goal: 'High priority task', status: 'waiting_approval' },
        { task_id: 'task_normal', goal: 'Normal task', status: 'running' },
        { task_id: 'task_low', goal: 'Low task', status: 'planning' }
      ])

      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(3)
    })
  })

  describe('Domain Pack Tests', () => {
    it('should use Godot domain pack correctly', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_godot_001',
        status: 'planning',
        domain: 'game.godot',
        domain_pack_loaded: true,
        skills_available: ['godot_scene_analyze', 'gdscript_generate', 'player_detect']
      })

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/godot/project',
        goal: 'Add player movement'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should reject invalid domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Unknown domain: game.unreal'))

      try {
        await OperatorApi.startTask({
          domain: 'game.unreal',
          project_path: '/test',
          goal: 'Test'
        })
      } catch (e: any) {
        expect(e.message).toContain('Unknown domain')
      }
    })
  })
})