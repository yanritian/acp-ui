// Error Handling Tests for Hermes Game Operator
// Tests error scenarios and recovery mechanisms

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

describe('Error Handling', () => {
  describe('Network Errors', () => {
    it('should handle connection refused', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection refused'))

      await expect(OperatorApi.listTasks()).rejects.toThrow('Connection refused')
    })

    it('should handle timeout errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Request timeout'))

      await expect(OperatorApi.getTask('task_1')).rejects.toThrow('Request timeout')
    })

    it('should handle network unreachable', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network unreachable'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Network unreachable')
    })
  })

  describe('Validation Errors', () => {
    it('should handle invalid task ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task not found: invalid_id'))

      await expect(OperatorApi.getTask('invalid_id')).rejects.toThrow('Task not found')
    })

    it('should handle invalid project path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid path: /nonexistent'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/nonexistent',
        goal: 'Test'
      })).rejects.toThrow('Invalid path')
    })

    it('should handle empty goal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Goal cannot be empty'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: ''
      })).rejects.toThrow('Goal cannot be empty')
    })
  })

  describe('State Machine Errors', () => {
    it('should handle invalid state transition', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid state transition: completed -> running'))

      await expect(OperatorApi.resumeTask('task_completed')).rejects.toThrow('Invalid state transition')
    })

    it('should handle task already running', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task is already running'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('already running')
    })

    it('should handle task not paused', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task is not paused'))

      await expect(OperatorApi.resumeTask('task_running')).rejects.toThrow('not paused')
    })
  })

  describe('Permission Errors', () => {
    it('should handle path outside boundary', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', '../../../etc/passwd')).rejects.toThrow('outside project boundary')
    })

    it('should handle forbidden command', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Command forbidden: rm'))

      await expect(OperatorApi.filePatch('task_1', 'test.txt', 'content')).rejects.toThrow('forbidden')
    })

    it('should handle approval required', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval required for this action'))

      await expect(OperatorApi.filePatch('task_1', 'script.gd', 'new content')).rejects.toThrow('Approval required')
    })
  })

  describe('Resource Errors', () => {
    it('should handle file not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found: missing.gd'))

      await expect(OperatorApi.fileRead('task_1', 'missing.gd')).rejects.toThrow('File not found')
    })

    it('should handle disk full', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Disk full'))

      await expect(OperatorApi.filePatch('task_1', 'test.txt', 'content')).rejects.toThrow('Disk full')
    })

    it('should handle file too large', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File exceeds maximum size'))

      await expect(OperatorApi.fileRead('task_1', 'large_file.bin')).rejects.toThrow('exceeds maximum size')
    })
  })

  describe('Hermes CLI Errors', () => {
    it('should handle Hermes CLI not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Hermes CLI not found'))

      const result = await HermesCliApi.checkConnection()
      expect(result.available).toBe(false)
      expect(result.error).toContain('not found')
    })

    it('should handle Hermes API error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Hermes API error: rate limit exceeded'))

      await expect(HermesCliApi.generatePlan('task_1', 'goal')).rejects.toThrow('rate limit')
    })
  })

  describe('Godot Detection Errors', () => {
    it('should handle not a Godot project', async () => {
      mockInvoke.mockResolvedValueOnce(false)

      const result = await GodotOperatorApi.detectProject('/non-godot-project')
      expect(result).toBe(false)
    })

    it('should handle corrupted project file', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed to parse project.godot'))

      await expect(GodotOperatorApi.analyzeProject('/corrupted-project')).rejects.toThrow('parse')
    })
  })

  describe('Error Recovery', () => {
    it('should allow retry after error', async () => {
      // First call fails
      mockInvoke.mockRejectedValueOnce(new Error('Temporary error'))
      await expect(OperatorApi.listTasks()).rejects.toThrow('Temporary error')

      // Second call succeeds
      mockInvoke.mockResolvedValueOnce([])
      const result = await OperatorApi.listTasks()
      expect(result).toEqual([])
    })

    it('should preserve state after error', async () => {
      // Get task succeeds
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_1',
        status: 'running',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-08T10:00:00Z',
        updated_at: '2026-07-08T10:00:00Z'
      })

      const task = await OperatorApi.getTask('task_1')
      expect(task.status).toBe('running')

      // Pause fails
      mockInvoke.mockRejectedValueOnce(new Error('Pause failed'))
      await expect(OperatorApi.pauseTask('task_1')).rejects.toThrow('Pause failed')

      // Task state should be unchanged
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_1',
        status: 'running',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-08T10:00:00Z',
        updated_at: '2026-07-08T10:00:00Z'
      })

      const taskAfter = await OperatorApi.getTask('task_1')
      expect(taskAfter.status).toBe('running')
    })
  })
})