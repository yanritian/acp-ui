// Integration Tests for Hermes Game Operator Full Flow
// Tests complete task lifecycle from start to completion

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

import { OperatorApi, GodotOperatorApi, HermesCliApi } from '@/api/operatorApi'

describe('Full Flow Integration Tests', () => {
  describe('Complete Task Lifecycle', () => {
    it('should complete full task flow: create -> run -> complete', async () => {
      const taskId = 'task_full_1'

      // 1. Start task
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'planning',
        event_stream: `operator://tasks/${taskId}/events`
      })
      const startResult = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:/tmp/test-godot-project',
        goal: 'Add double jump to player'
      })
      expect(startResult.task_id).toBe(taskId)

      // 2. Get task - running
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'running',
        domain: 'game.godot',
        project_path: 'D:/tmp/test-godot-project',
        goal: 'Add double jump to player',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:01:00Z'
      })
      const runningTask = await OperatorApi.getTask(taskId)
      expect(runningTask.status).toBe('running')

      // 3. List events
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'evt_1', task_id: taskId, type: 'task_started', level: 'info', title: 'Started', timestamp: '2026-07-09T00:00:00Z', source: 'operator' },
        { event_id: 'evt_2', task_id: taskId, type: 'project_analyzed', level: 'info', title: 'Analyzed', timestamp: '2026-07-09T00:00:30Z', source: 'operator' },
        { event_id: 'evt_3', task_id: taskId, type: 'plan_ready', level: 'info', title: 'Plan Ready', timestamp: '2026-07-09T00:01:00Z', source: 'operator' }
      ])
      const events = await OperatorApi.listEvents(taskId)
      expect(events.length).toBe(3)

      // 4. Get summary
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'completed',
        goal: 'Add double jump to player',
        summary: 'Successfully added double jump ability to Player.gd',
        files_changed: ['scripts/Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 120,
        iterations: 5,
        errors: [],
        warnings: []
      })
      const summary = await OperatorApi.getTaskSummary(taskId)
      expect(summary.status).toBe('completed')
      expect(summary.files_changed).toContain('scripts/Player.gd')
    })

    it('should handle pause-resume flow', async () => {
      const taskId = 'task_pause_1'

      // Start task
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', event_stream: '' })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      // Pause
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.pauseTask(taskId)

      // Verify paused
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'paused',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:02:00Z'
      })
      const pausedTask = await OperatorApi.getTask(taskId)
      expect(pausedTask.status).toBe('paused')

      // Resume
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.resumeTask(taskId)

      // Verify running
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'running',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:03:00Z'
      })
      const resumedTask = await OperatorApi.getTask(taskId)
      expect(resumedTask.status).toBe('running')
    })

    it('should handle redirect flow', async () => {
      const taskId = 'task_redirect_1'

      // Start task
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'running', event_stream: '' })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Original goal'
      })

      // Redirect
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.redirectTask({
        task_id: taskId,
        new_goal: 'New goal - add triple jump',
        preserve_completed_work: true
      })

      // Verify new goal
      mockInvoke.mockResolvedValueOnce({
        task_id: taskId,
        status: 'running',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'New goal - add triple jump',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:04:00Z'
      })
      const task = await OperatorApi.getTask(taskId)
      expect(task.goal).toBe('New goal - add triple jump')
    })
  })

  describe('Approval Flow', () => {
    it('should handle approval flow', async () => {
      const taskId = 'task_approval_1'

      // Start task
      mockInvoke.mockResolvedValueOnce({ task_id: taskId, status: 'waiting_approval', event_stream: '' })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })

      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_1',
        task_id: taskId,
        level: 'approve',
        action: 'file.patch',
        title: 'Modify Player.gd',
        reason: 'Adding double jump logic',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T00:00:00Z'
      }])
      const approvals = await OperatorApi.getPendingApprovals(taskId)
      expect(approvals.length).toBe(1)
      expect(approvals[0].action).toBe('file.patch')

      // Approve
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: taskId,
        approval_id: 'approval_1',
        decision: 'approve',
        comment: 'Looks good'
      })

      // Verify no more approvals
      mockInvoke.mockResolvedValueOnce([])
      const remainingApprovals = await OperatorApi.getPendingApprovals(taskId)
      expect(remainingApprovals.length).toBe(0)
    })

    it('should handle rejection flow', async () => {
      const taskId = 'task_reject_1'

      // Get pending approvals
      mockInvoke.mockResolvedValueOnce([{
        approval_id: 'approval_1',
        task_id: taskId,
        level: 'approve',
        action: 'file.patch',
        title: 'Modify file',
        reason: 'Test',
        options: ['approve', 'reject'],
        created_at: '2026-07-09T00:00:00Z'
      }])
      await OperatorApi.getPendingApprovals(taskId)

      // Reject
      mockInvoke.mockResolvedValueOnce(undefined)
      await OperatorApi.approve({
        task_id: taskId,
        approval_id: 'approval_1',
        decision: 'reject',
        comment: 'Not safe'
      })

      expect(mockInvoke).toHaveBeenCalled()
    })
  })

  describe('File Operations Flow', () => {
    it('should handle file read-modify-verify flow', async () => {
      const taskId = 'task_file_1'

      // Read file
      mockInvoke.mockResolvedValueOnce({
        content: 'extends CharacterBody2D\nconst SPEED = 300.0\n',
        path: 'scripts/Player.gd'
      })
      const readFile = await OperatorApi.fileRead(taskId, 'scripts/Player.gd')
      expect(readFile.content).toContain('SPEED')

      // Preview patch
      mockInvoke.mockResolvedValueOnce({
        old_content: 'extends CharacterBody2D\nconst SPEED = 300.0\n',
        new_content: 'extends CharacterBody2D\nconst SPEED = 300.0\nconst JUMP_VELOCITY = -400.0\n',
        diff: '--- old\n+++ new\n@@ -1,2 +1,3 @@\n extends CharacterBody2D\n const SPEED = 300.0\n+const JUMP_VELOCITY = -400.0\n'
      })
      const preview = await OperatorApi.filePatchPreview(taskId, 'scripts/Player.gd', 'new content')
      expect(preview.diff).toContain('JUMP_VELOCITY')

      // Apply patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        backup_path: 'scripts/Player.gd.bak'
      })
      const patchResult = await OperatorApi.filePatch(taskId, 'scripts/Player.gd', 'new content', true)
      expect(patchResult.success).toBe(true)
    })

    it('should handle file listing', async () => {
      const taskId = 'task_list_1'

      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        directories: ['scripts', 'scenes']
      })
      const listResult = await OperatorApi.fileList(taskId, 'scripts')
      expect(listResult.files.length).toBe(3)
      expect(listResult.files).toContain('Player.gd')
    })
  })

  describe('Godot Integration Flow', () => {
    it('should detect and analyze Godot project', async () => {
      // Detect project
      mockInvoke.mockResolvedValueOnce(true)
      const isGodot = await GodotOperatorApi.detectProject('D:/tmp/test-godot-project')
      expect(isGodot).toBe(true)

      // Analyze project
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test Godot Project',
        godot_version: '4.2',
        scripts: ['scripts/Player.gd'],
        scenes: ['main.tscn'],
        assets: [],
        main_scene: 'main.tscn'
      })
      const analysis = await GodotOperatorApi.analyzeProject('D:/tmp/test-godot-project')
      expect(analysis.project_name).toBe('Test Godot Project')
      expect(analysis.scripts).toContain('scripts/Player.gd')
    })
  })

  describe('Hermes CLI Integration Flow', () => {
    it('should check connection and generate plan', async () => {
      // Check connection
      mockInvoke.mockResolvedValueOnce({
        available: true,
        version: '1.0.0',
        path: '/usr/bin/hermes'
      })
      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(true)

      // Generate plan
      mockInvoke.mockResolvedValueOnce({
        taskId: 'task_hermes_1',
        goal: 'Add double jump',
        steps: [
          { id: 1, description: 'Analyze Player.gd', files: ['scripts/Player.gd'], estimated_time_seconds: 30, requires_approval: false },
          { id: 2, description: 'Add jump logic', files: ['scripts/Player.gd'], estimated_time_seconds: 60, requires_approval: true }
        ],
        total_estimated_time_seconds: 90
      })
      const plan = await HermesCliApi.generatePlan('task_hermes_1', 'Add double jump')
      expect(plan.steps.length).toBe(2)
      expect(plan.total_estimated_time_seconds).toBe(90)
    })
  })
})