// Operator Error Recovery Tests
// Testing error handling and recovery scenarios

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

describe('Operator Error Recovery Tests', () => {
  describe('Connection Errors', () => {
    it('should handle connection timeout', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection timeout'))
      await expect(OperatorApi.listTasks()).rejects.toThrow('timeout')
    })

    it('should handle connection refused', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection refused'))
      await expect(OperatorApi.getTask('t1')).rejects.toThrow('refused')
    })

    it('should handle network unreachable', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network unreachable'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('unreachable')
    })
  })

  describe('Validation Errors', () => {
    it('should handle invalid task ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID format'))
      await expect(OperatorApi.getTask('invalid!@#')).rejects.toThrow('Invalid')
    })

    it('should handle empty goal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Goal cannot be empty'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: ''
      })).rejects.toThrow('empty')
    })

    it('should handle invalid domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Unknown domain'))
      await expect(OperatorApi.startTask({
        domain: 'invalid',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Unknown')
    })
  })

  describe('Permission Errors', () => {
    it('should handle file read permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))
      await expect(OperatorApi.fileRead('t1', 'protected.gd')).rejects.toThrow('Permission')
    })

    it('should handle file write permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Write permission denied'))
      await expect(OperatorApi.filePatch('t1', 'readonly.gd', 'content')).rejects.toThrow('permission')
    })
  })

  describe('State Errors', () => {
    it('should handle invalid state transition', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid state transition'))
      await expect(OperatorApi.pauseTask('completed_task')).rejects.toThrow('Invalid')
    })

    it('should handle task not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task not found'))
      await expect(OperatorApi.getTask('nonexistent')).rejects.toThrow('not found')
    })
  })

  describe('Rate Limiting', () => {
    it('should handle rate limit exceeded', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Rate limit exceeded'))
      await expect(OperatorApi.listTasks()).rejects.toThrow('Rate limit')
    })

    it('should handle quota exceeded', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Quota exceeded'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Quota')
    })
  })

  describe('Recovery Scenarios', () => {
    it('should recover from temporary error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Temporary error'))
      mockInvoke.mockResolvedValueOnce({ task_id: 't1', status: 'planning', event_stream: '' })

      try {
        await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: 'Test'
        })
      } catch (e) {
        // First attempt failed, retry
      }

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should continue after pause-resume', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.pauseTask('t1')
      await OperatorApi.resumeTask('t1')
    })
  })
})