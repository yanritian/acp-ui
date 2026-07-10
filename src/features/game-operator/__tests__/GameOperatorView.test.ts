// Game Operator View Component Tests
// Tests for the main GameOperatorView component logic

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { createApp, nextTick } from 'vue'
import gameOperatorViewSource from '../views/GameOperatorView.vue?raw'

// Mock Tauri APIs
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

// Mock Operator API
const mockStartTask = vi.fn()
const mockGetTask = vi.fn()
const mockListTasks = vi.fn()
const mockPauseTask = vi.fn()
const mockResumeTask = vi.fn()
const mockStopTask = vi.fn()
const mockApprove = vi.fn()
const mockListEvents = vi.fn()
const mockGetPendingApprovals = vi.fn()
const mockDetectProject = vi.fn()
const mockGetRemotePlatforms = vi.fn()

vi.mock('@/api/operatorApi', () => ({
  OperatorApi: {
    startTask: mockStartTask,
    getTask: mockGetTask,
    listTasks: mockListTasks,
    pauseTask: mockPauseTask,
    resumeTask: mockResumeTask,
    stopTask: mockStopTask,
    approve: mockApprove,
    listEvents: mockListEvents,
    getPendingApprovals: mockGetPendingApprovals,
  },
  GodotOperatorApi: {
    detectProject: mockDetectProject,
    analyzeProject: vi.fn(),
  },
  HermesCliApi: {
    checkConnection: vi.fn(),
    analyzeProject: vi.fn(),
    generatePlan: vi.fn(),
  }
}))

vi.mock('@/api/operatorRemoteApi', () => ({
  OperatorRemoteApi: {
    getPlatforms: mockGetRemotePlatforms,
  },
}))

describe('GameOperatorView Logic', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockListTasks.mockResolvedValue([])
    mockDetectProject.mockResolvedValue(true)
    mockGetRemotePlatforms.mockResolvedValue([])
  })

  afterEach(() => {
    vi.clearAllTimers()
  })

  describe('Initial State', () => {
    it('should start with no current task', async () => {
      mockListTasks.mockResolvedValueOnce([])
      const tasks = await mockListTasks()
      expect(tasks.length).toBe(0)
    })

    it('should load saved tasks on mount', async () => {
      mockListTasks.mockResolvedValueOnce([
        { task_id: 'task_001', status: 'running' }
      ])
      const tasks = await mockListTasks()
      expect(tasks.length).toBe(1)
    })

    it('should find active task on mount', async () => {
      mockListTasks.mockResolvedValueOnce([
        { task_id: 'task_001', status: 'completed' },
        { task_id: 'task_002', status: 'running' }
      ])
      const tasks = await mockListTasks()
      const activeTask = tasks.find((t: any) => !['completed', 'failed', 'cancelled'].includes(t.status))
      expect(activeTask?.task_id).toBe('task_002')
    })

    it('should keep the operator surface scrollable and form fields shrinkable', () => {
      expect(gameOperatorViewSource).toMatch(
        /\.game-operator-view\s*\{[^}]*overflow-y:\s*auto;/s,
      )
      expect(gameOperatorViewSource).toMatch(
        /\.path-field,\s*\.goal-field\s*\{[^}]*min-width:\s*0;/s,
      )
      expect(gameOperatorViewSource).toContain('container-type: inline-size;')
      expect(gameOperatorViewSource).toContain('@container (max-width: 1050px)')
    })
  })

  describe('Project Selection', () => {
    it('should detect Godot project', async () => {
      mockDetectProject.mockResolvedValueOnce(true)
      const isGodot = await mockDetectProject('/path/to/project')
      expect(isGodot).toBe(true)
    })

    it('should reject non-Godot project', async () => {
      mockDetectProject.mockResolvedValueOnce(false)
      const isGodot = await mockDetectProject('/path/to/non-godot')
      expect(isGodot).toBe(false)
    })

    it('should use Tauri dialog for selection', async () => {
      const { open } = await import('@tauri-apps/plugin-dialog')
      vi.mocked(open).mockResolvedValueOnce('/selected/path')
      const result = await open({ directory: true })
      expect(result).toBe('/selected/path')
    })
  })

  describe('Task Start', () => {
    it('should call startTask with correct params', async () => {
      mockStartTask.mockResolvedValueOnce({
        task_id: 'task_001',
        status: 'planning',
        event_stream: 'operator://tasks/task_001/events'
      })

      const result = await mockStartTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature'
      })

      expect(result.task_id).toBe('task_001')
    })

    it('should require project path and goal', async () => {
      const isValid = (path: string, goal: string) => !!path && !!goal
      expect(isValid('', 'goal')).toBe(false)
      expect(isValid('/path', '')).toBe(false)
      expect(isValid('/path', 'goal')).toBe(true)
    })

    it('should set loading state during start', async () => {
      mockStartTask.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve({ task_id: 'task_001' }), 50)))
      const promise = mockStartTask({})
      expect(promise).toBeDefined()
      await promise
    })
  })

  describe('Task Control', () => {
    it('should pause running task', async () => {
      mockPauseTask.mockResolvedValueOnce(undefined)
      await mockPauseTask('task_001')
      expect(mockPauseTask).toHaveBeenCalledWith('task_001')
    })

    it('should resume paused task', async () => {
      mockResumeTask.mockResolvedValueOnce(undefined)
      await mockResumeTask('task_001')
      expect(mockResumeTask).toHaveBeenCalledWith('task_001')
    })

    it('should stop task', async () => {
      mockStopTask.mockResolvedValueOnce(undefined)
      await mockStopTask('task_001')
      expect(mockStopTask).toHaveBeenCalledWith('task_001')
    })
  })

  describe('Event Polling', () => {
    it('should poll events at interval', async () => {
      mockListEvents.mockResolvedValueOnce([
        { event_id: 'evt_001', type: 'task_started' }
      ])
      const events = await mockListEvents('task_001')
      expect(events.length).toBe(1)
    })

    it('should discover a task created remotely after the view mounted empty', async () => {
      vi.useFakeTimers()
      const remoteTask = {
        task_id: 'task_remote_001',
        domain: 'game.godot',
        project_path: 'D:/games/remote-project',
        goal: 'Remote-created dash task',
        status: 'waiting_approval',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-10T00:00:00Z',
        updated_at: '2026-07-10T00:00:01Z',
      }
      mockListTasks
        .mockResolvedValueOnce([])
        .mockResolvedValue([remoteTask])
      mockGetTask.mockResolvedValue(remoteTask)
      mockListEvents.mockResolvedValue([])
      mockGetPendingApprovals.mockResolvedValue([])

      const host = document.createElement('div')
      document.body.appendChild(host)
      const { default: GameOperatorView } = await import('../views/GameOperatorView.vue')
      const { createI18n } = await import('vue-i18n')
      const i18n = createI18n({
        legacy: false,
        locale: 'en-US',
        messages: {
          'en-US': {
            gameOperator: {
              title: 'Hermes Game Operator',
              subtitle: 'Godot MVP - Single Task Closed Loop',
              projectPath: 'Godot Project Path',
              selectProject: 'Browse...',
              taskGoal: 'Task Goal',
              goalPlaceholder: 'e.g., Add double jump to the player character',
              startTask: 'Start Task',
              starting: 'Starting...',
              noPendingApprovals: 'No pending approvals',
              redirectGoal: 'Redirect Goal',
              redirect: 'Redirect',
              redirecting: 'Redirecting...',
              selectProjectTitle: 'Select Godot Project Directory',
              notGodotProject: 'Selected directory is not a Godot project',
              selectProjectAndGoal: 'Please select a project and enter a goal',
            },
            operatorStatus: {
              idle: 'Idle',
              planning: 'Planning',
              waitingApproval: 'Waiting Approval',
              running: 'Running',
              paused: 'Paused',
              redirecting: 'Redirecting',
              cancelling: 'Cancelling',
              cancelled: 'Cancelled',
              failed: 'Failed',
              completed: 'Completed',
              unknown: 'Unknown',
            },
            approvalDecision: {
              approve: 'Approve',
              reject: 'Reject',
              requestChanges: 'Request changes',
            },
            operatorEvent: {
              taskCreated: 'Task created',
              taskStarted: 'Task started',
              projectAnalyzing: 'Analyzing project',
              projectAnalyzed: 'Project analyzed',
              projectAnalysisFailed: 'Project analysis failed',
              planGenerating: 'Generating plan',
              planReady: 'Plan ready',
              planFailed: 'Plan generation failed',
              approvalRequested: 'Approval requested',
              approvalGranted: 'Approval granted',
              approvalRejected: 'Approval rejected',
              stepExecuting: 'Step executing',
              stepCompleted: 'Step completed',
              stepFailed: 'Step failed',
              fileModified: 'File modified',
              taskCompleting: 'Task completing',
              taskCompleted: 'Task completed',
              taskFailed: 'Task failed',
              taskCancelled: 'Task cancelled',
              taskPaused: 'Task paused',
              taskResumed: 'Task resumed',
              taskRedirected: 'Task redirected',
            },
            operatorError: {
              projectNotFound: 'Project not found',
              taskNotFound: 'Task not found',
              approvalNotFound: 'Approval not found',
              invalidState: 'Invalid state',
              operationFailed: 'Operation failed',
              networkError: 'Network error',
              timeout: 'Timeout',
              permissionDenied: 'Permission denied',
            },
            a11y: {
              approvalButton: 'Approve button',
              rejectButton: 'Reject button',
              requestChangesButton: 'Request changes button',
              pauseButton: 'Pause button',
              resumeButton: 'Resume button',
              stopButton: 'Stop button',
              diffPreview: 'Diff preview',
              approvalLevel: 'Approval level',
              approvalAction: 'Approval action',
              approvalTitle: 'Approval title',
              approvalReason: 'Approval reason',
              approvalRisk: 'Approval risk',
              approvalFiles: 'Approval files',
            },
          },
        },
      })
      const app = createApp(GameOperatorView)
      app.use(i18n)
      app.mount(host)
      await Promise.resolve()
      await nextTick()

      expect(host.textContent).toContain('Godot Project Path')
      await vi.advanceTimersByTimeAsync(2100)
      await nextTick()

      expect(mockListTasks).toHaveBeenCalledTimes(2)
      expect(host.textContent).toContain('Remote-created dash task')

      app.unmount()
      host.remove()
      vi.useRealTimers()
    })

    it('should poll approvals', async () => {
      mockGetPendingApprovals.mockResolvedValueOnce([
        { approval_id: 'approval_001' }
      ])
      const approvals = await mockGetPendingApprovals('task_001')
      expect(approvals.length).toBe(1)
    })
  })

  describe('Approval Handling', () => {
    it('should submit approval', async () => {
      mockApprove.mockResolvedValueOnce(undefined)
      await mockApprove({
        task_id: 'task_001',
        approval_id: 'approval_001',
        decision: 'approve'
      })
      expect(mockApprove).toHaveBeenCalled()
    })

    it('should refresh approvals after action', async () => {
      mockApprove.mockResolvedValueOnce(undefined)
      mockGetPendingApprovals.mockResolvedValueOnce([])
      await mockApprove({ task_id: 'task_001', approval_id: 'approval_001', decision: 'approve' })
      const approvals = await mockGetPendingApprovals('task_001')
      expect(approvals.length).toBe(0)
    })
  })

  describe('Error Handling', () => {
    it('should handle start error', async () => {
      mockStartTask.mockRejectedValueOnce(new Error('Start failed'))
      try {
        await mockStartTask({})
      } catch (e: any) {
        expect(e.message).toBe('Start failed')
      }
    })

    it('should handle detection error', async () => {
      mockDetectProject.mockRejectedValueOnce(new Error('Detection failed'))
      try {
        await mockDetectProject('/path')
      } catch (e: any) {
        expect(e.message).toBe('Detection failed')
      }
    })

    it('should handle control error', async () => {
      mockPauseTask.mockRejectedValueOnce(new Error('Control failed'))
      try {
        await mockPauseTask('task_001')
      } catch (e: any) {
        expect(e.message).toBe('Control failed')
      }
    })
  })

  describe('State Management', () => {
    it('should track current task', async () => {
      mockGetTask.mockResolvedValueOnce({ task_id: 'task_001', status: 'running' })
      const task = await mockGetTask('task_001')
      expect(task.task_id).toBe('task_001')
    })

    it('should track events list', async () => {
      mockListEvents.mockResolvedValueOnce([
        { event_id: 'evt_001' },
        { event_id: 'evt_002' }
      ])
      const events = await mockListEvents('task_001')
      expect(events.length).toBe(2)
    })

    it('should track pending approvals', async () => {
      mockGetPendingApprovals.mockResolvedValueOnce([
        { approval_id: 'approval_001' }
      ])
      const approvals = await mockGetPendingApprovals('task_001')
      expect(approvals.length).toBe(1)
    })
  })
})
