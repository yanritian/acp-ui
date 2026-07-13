// Unit Tests for Hermes Operator API Layer
// Tests the frontend API wrapper functions

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock window.__TAURI_INTERNALS__ for testing
const mockInvoke = vi.fn()

// Setup global mock before tests
beforeEach(() => {
  vi.stubGlobal('__TAURI_INTERNALS__', {
    invoke: mockInvoke
  })
  vi.clearAllMocks()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

// Import after mock setup
import { OperatorApi, GodotOperatorApi, HermesCliApi } from '@/api/operatorApi'
import type {
  OperatorTask,
  OperatorEvent,
  ApprovalRequest,
  StartTaskRequest,
  StartTaskResponse
} from '@/types/operator'

describe('OperatorApi', () => {
  describe('Task Management', () => {
    it('should start task with valid request', async () => {
      const mockResponse: StartTaskResponse = {
        task_id: 'task_123',
        revision: 1,
        status: 'planning',
        event_stream: 'operator://tasks/task_123/events'
      }
      mockInvoke.mockResolvedValueOnce(mockResponse)

      const request: StartTaskRequest = {
        domain: 'game.godot',
        project_path: 'D:/tmp/test-godot-project',
        goal: 'Add double jump to player',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      }

      const result = await OperatorApi.startTask(request)

      expect(mockInvoke).toHaveBeenCalledWith('operator_start_task', { request })
      expect(result.task_id).toBe('task_123')
      expect(result.status).toBe('planning')
    })

    it('should get task by id', async () => {
      const mockTask: OperatorTask = {
        task_id: 'task_123',
        revision: 1,
        domain: 'game.godot',
        project_path: 'D:/tmp/test-godot-project',
        goal: 'Add double jump',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-08T10:00:00Z',
        updated_at: '2026-07-08T10:05:00Z'
      }
      mockInvoke.mockResolvedValueOnce(mockTask)

      const result = await OperatorApi.getTask('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_get_task', { taskId: 'task_123' })
      expect(result.task_id).toBe('task_123')
      expect(result.status).toBe('running')
    })

    it('should list all tasks', async () => {
      const mockTasks: OperatorTask[] = [
        {
          task_id: 'task_1',
          revision: 4,
          domain: 'game.godot',
          project_path: '/path/a',
          goal: 'Goal A',
          status: 'completed',
          mode: 'propose_then_apply',
          approval_policy: 'safe_default',
          created_at: '2026-07-08T09:00:00Z',
          updated_at: '2026-07-08T10:00:00Z'
        },
        {
          task_id: 'task_2',
          revision: 5,
          domain: 'game.godot',
          project_path: '/path/b',
          goal: 'Goal B',
          status: 'running',
          mode: 'propose_then_apply',
          approval_policy: 'safe_default',
          created_at: '2026-07-08T10:00:00Z',
          updated_at: '2026-07-08T10:05:00Z'
        }
      ]
      mockInvoke.mockResolvedValueOnce(mockTasks)

      const result = await OperatorApi.listTasks()

      expect(mockInvoke).toHaveBeenCalledWith('operator_list_tasks', undefined)
      expect(result).toHaveLength(2)
    })

    it('should pause running task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.pauseTask('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_pause_task', { taskId: 'task_123' })
    })

    it('should resume paused task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.resumeTask('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_resume_task', { taskId: 'task_123' })
    })

    it('should stop task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.stopTask('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_stop_task', { taskId: 'task_123' })
    })

    it('should redirect task', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.redirectTask({
        task_id: 'task_123',
        new_goal: 'New goal',
        preserve_completed_work: true
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_redirect_task', {
        request: {
          task_id: 'task_123',
          new_goal: 'New goal',
          preserve_completed_work: true
        }
      })
    })
  })

  describe('Approval', () => {
    it('should approve action', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_123',
        approval_id: 'approval_1',
        decision: 'approve',
        comment: 'Looks good'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_123',
          approval_id: 'approval_1',
          decision: 'approve',
          comment: 'Looks good'
        }
      })
    })

    it('should reject action', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.approve({
        task_id: 'task_123',
        approval_id: 'approval_1',
        decision: 'reject'
      })

      expect(mockInvoke).toHaveBeenCalledWith('operator_approve', {
        request: {
          task_id: 'task_123',
          approval_id: 'approval_1',
          decision: 'reject'
        }
      })
    })

    it('should get pending approvals', async () => {
      const mockApprovals: ApprovalRequest[] = [
        {
          approval_id: 'approval_1',
          task_id: 'task_123',
          task_revision: 1,
          level: 'approve',
          action: 'file.patch',
          title: 'Modify Player.gd',
          reason: 'Adding double jump logic',
          options: ['approve', 'reject'],
          created_at: '2026-07-08T10:00:00Z'
        }
      ]
      mockInvoke.mockResolvedValueOnce(mockApprovals)

      const result = await OperatorApi.getPendingApprovals('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_get_pending_approvals', { taskId: 'task_123' })
      expect(result).toHaveLength(1)
      expect(result[0].action).toBe('file.patch')
    })
  })

  describe('Events', () => {
    it('should list events', async () => {
      const mockEvents: OperatorEvent[] = [
        {
          event_id: 'evt_1',
          task_id: 'task_123',
          sequence: 0,
          task_revision: 1,
          timestamp: '2026-07-08T10:00:00Z',
          type: 'task_started',
          level: 'info',
          title: 'Task started',
          source: 'operator'
        },
        {
          event_id: 'evt_2',
          task_id: 'task_123',
          sequence: 1,
          task_revision: 2,
          timestamp: '2026-07-08T10:05:00Z',
          type: 'plan_ready',
          level: 'info',
          title: 'Plan ready',
          source: 'hermes'
        }
      ]
      mockInvoke.mockResolvedValueOnce(mockEvents)

      const result = await OperatorApi.listEvents('task_123', 100)

      expect(mockInvoke).toHaveBeenCalledWith('operator_list_events', { taskId: 'task_123', limit: 100 })
      expect(result).toHaveLength(2)
    })

    it('should list events without limit', async () => {
      mockInvoke.mockResolvedValueOnce([])

      await OperatorApi.listEvents('task_123')

      expect(mockInvoke).toHaveBeenCalledWith('operator_list_events', { taskId: 'task_123', limit: undefined })
    })
  })

  describe('File Tools', () => {
    it('should read file', async () => {
      const mockContent = 'extends CharacterBody2D\nconst SPEED = 300.0'
      mockInvoke.mockResolvedValueOnce({ content: mockContent })

      const result = await OperatorApi.fileRead('task_123', 'scripts/Player.gd')

      expect(mockInvoke).toHaveBeenCalledWith('operator_file_read', {
        taskId: 'task_123',
        path: 'scripts/Player.gd'
      })
    })

    it('should patch file', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      await OperatorApi.filePatch('task_123', 'scripts/Player.gd', 'new content', true)

      expect(mockInvoke).toHaveBeenCalledWith('operator_file_patch', {
        taskId: 'task_123',
        path: 'scripts/Player.gd',
        newContent: 'new content',
        createBackup: true
      })
    })

    it('should preview patch', async () => {
      const mockPatch = {
        old_content: 'old',
        new_content: 'new',
        diff: '--- old\n+++ new'
      }
      mockInvoke.mockResolvedValueOnce(mockPatch)

      const result = await OperatorApi.filePatchPreview('task_123', 'scripts/Player.gd', 'new')

      expect(mockInvoke).toHaveBeenCalledWith('operator_file_patch_preview', {
        taskId: 'task_123',
        path: 'scripts/Player.gd',
        newContent: 'new'
      })
    })

    it('should list files', async () => {
      const mockFiles = ['Player.gd', 'Enemy.gd', 'UI.gd']
      mockInvoke.mockResolvedValueOnce({ files: mockFiles })

      const result = await OperatorApi.fileList('task_123', 'scripts')

      expect(mockInvoke).toHaveBeenCalledWith('operator_file_list', {
        taskId: 'task_123',
        path: 'scripts'
      })
    })
  })
})

describe('GodotOperatorApi', () => {
  it('should detect Godot project', async () => {
    mockInvoke.mockResolvedValueOnce(true)

    const result = await GodotOperatorApi.detectProject('D:/tmp/test-godot-project')

    expect(mockInvoke).toHaveBeenCalledWith('godot_detect_project', {
      path: 'D:/tmp/test-godot-project'
    })
    expect(result).toBe(true)
  })

  it('should reject non-Godot project', async () => {
    mockInvoke.mockResolvedValueOnce(false)

    const result = await GodotOperatorApi.detectProject('D:/tmp/other-project')

    expect(result).toBe(false)
  })

  it('should analyze Godot project', async () => {
    const mockAnalysis = {
      project_name: 'Test Godot Project',
      godot_version: '4.2',
      scripts: ['scripts/Player.gd'],
      scenes: ['main.tscn'],
      assets: [],
      main_scene: 'main.tscn'
    }
    mockInvoke.mockResolvedValueOnce(mockAnalysis)

    const result = await GodotOperatorApi.analyzeProject('D:/tmp/test-godot-project')

    expect(mockInvoke).toHaveBeenCalledWith('godot_analyze_project', {
      projectPath: 'D:/tmp/test-godot-project'
    })
    expect(result.project_name).toBe('Test Godot Project')
    expect(result.scripts).toContain('scripts/Player.gd')
  })
})

describe('HermesCliApi', () => {
  it('should check Hermes connection', async () => {
    const mockStatus = {
      available: true,
      version: '1.0.0',
      path: '/usr/local/bin/hermes'
    }
    mockInvoke.mockResolvedValueOnce(mockStatus)

    const result = await HermesCliApi.checkConnection()

    expect(mockInvoke).toHaveBeenCalledWith('hermes_check_connection', undefined)
    expect(result.available).toBe(true)
    expect(result.version).toBe('1.0.0')
  })

  it('should handle Hermes unavailable', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Hermes CLI not found'))

    const result = await HermesCliApi.checkConnection()

    expect(result.available).toBe(false)
    expect(result.error).toContain('Hermes CLI not found')
  })

  it('should generate plan', async () => {
    const mockPlan = {
      taskId: 'task_123',
      goal: 'Add double jump',
      steps: [
        { id: 1, description: 'Analyze Player.gd', files: ['scripts/Player.gd'], estimatedTimeSeconds: 30, requiresApproval: false },
        { id: 2, description: 'Add jump_count variable', files: ['scripts/Player.gd'], estimatedTimeSeconds: 60, requiresApproval: true }
      ],
      totalEstimatedTimeSeconds: 90
    }
    mockInvoke.mockResolvedValueOnce(mockPlan)

    const result = await HermesCliApi.generatePlan('task_123', 'Add double jump')

    expect(mockInvoke).toHaveBeenCalledWith('hermes_generate_plan', {
      taskId: 'task_123',
      goal: 'Add double jump'
    })
    expect(result.steps).toHaveLength(2)
  })
})
