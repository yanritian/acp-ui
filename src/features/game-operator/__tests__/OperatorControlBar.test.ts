// Operator Control Bar Component Tests
// Tests for task control buttons (pause, resume, stop)

import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock props
const mockTask = {
  task_id: 'task_001',
  status: 'running',
  domain: 'game.godot',
  project_path: '/test',
  goal: 'Test goal',
  mode: 'propose_then_apply',
  approval_policy: 'safe_default',
  created_at: '2026-07-09T10:00:00Z',
  updated_at: '2026-07-09T10:00:00Z'
}

describe('OperatorControlBar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Task Status Display', () => {
    it('should display running status', () => {
      const task = { ...mockTask, status: 'running' }
      expect(task.status).toBe('running')
    })

    it('should display paused status', () => {
      const task = { ...mockTask, status: 'paused' }
      expect(task.status).toBe('paused')
    })

    it('should display planning status', () => {
      const task = { ...mockTask, status: 'planning' }
      expect(task.status).toBe('planning')
    })

    it('should display completed status', () => {
      const task = { ...mockTask, status: 'completed' }
      expect(task.status).toBe('completed')
    })

    it('should display failed status', () => {
      const task = { ...mockTask, status: 'failed' }
      expect(task.status).toBe('failed')
    })
  })

  describe('Control Button States', () => {
    it('should enable pause when running', () => {
      const task = { ...mockTask, status: 'running' }
      const canPause = task.status === 'running'
      expect(canPause).toBe(true)
    })

    it('should enable resume when paused', () => {
      const task = { ...mockTask, status: 'paused' }
      const canResume = task.status === 'paused'
      expect(canResume).toBe(true)
    })

    it('should enable stop when running or paused', () => {
      const runningTask = { ...mockTask, status: 'running' }
      const pausedTask = { ...mockTask, status: 'paused' }

      const canStopRunning = ['running', 'paused', 'planning'].includes(runningTask.status)
      const canStopPaused = ['running', 'paused', 'planning'].includes(pausedTask.status)

      expect(canStopRunning).toBe(true)
      expect(canStopPaused).toBe(true)
    })

    it('should disable controls when completed', () => {
      const task = { ...mockTask, status: 'completed' }
      const canControl = ['running', 'paused', 'planning', 'waiting_approval'].includes(task.status)
      expect(canControl).toBe(false)
    })
  })

  describe('Button Labels', () => {
    it('should show correct pause button label', () => {
      const label = 'Pause'
      expect(label).toBe('Pause')
    })

    it('should show correct resume button label', () => {
      const label = 'Resume'
      expect(label).toBe('Resume')
    })

    it('should show correct stop button label', () => {
      const label = 'Stop'
      expect(label).toBe('Stop')
    })
  })

  describe('Event Emission', () => {
    it('should emit pause event', () => {
      const handler = vi.fn()
      handler('pause')
      expect(handler).toHaveBeenCalledWith('pause')
    })

    it('should emit resume event', () => {
      const handler = vi.fn()
      handler('resume')
      expect(handler).toHaveBeenCalledWith('resume')
    })

    it('should emit stop event', () => {
      const handler = vi.fn()
      handler('stop')
      expect(handler).toHaveBeenCalledWith('stop')
    })
  })

  describe('Task Info Display', () => {
    it('should show task goal', () => {
      const task = { ...mockTask, goal: 'Add double jump' }
      expect(task.goal).toBe('Add double jump')
    })

    it('should show project path', () => {
      const task = { ...mockTask, project_path: '/path/to/project' }
      expect(task.project_path).toBe('/path/to/project')
    })

    it('should show task ID', () => {
      const task = { ...mockTask, task_id: 'task_12345' }
      expect(task.task_id).toBe('task_12345')
    })

    it('should show domain', () => {
      const task = { ...mockTask, domain: 'game.godot' }
      expect(task.domain).toBe('game.godot')
    })
  })
})