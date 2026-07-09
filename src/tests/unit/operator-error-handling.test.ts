// Operator Error Handling Tests
// Testing error handling and recovery

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

describe('Operator Error Handling Tests', () => {
  describe('Connection Errors', () => {
    it('should handle connection timeout', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection timeout'))
      await expect(OperatorApi.listTasks()).rejects.toThrow('Connection timeout')
    })

    it('should handle network error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))
      await expect(OperatorApi.getTask('error_001')).rejects.toThrow('Network error')
    })
  })

  describe('Validation Errors', () => {
    it('should handle invalid task ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID'))
      await expect(OperatorApi.getTask('')).rejects.toThrow('Invalid')
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
      mockInvoke.mockRejectedValueOnce(new Error('Invalid domain'))
      await expect(OperatorApi.startTask({
        domain: '',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('Permission Errors', () => {
    it('should handle file read permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))
      await expect(OperatorApi.fileRead('error_002', 'protected.gd')).rejects.toThrow('Permission')
    })

    it('should handle file write permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Write permission denied'))
      await expect(OperatorApi.filePatch('error_002', 'readonly.gd', 'content')).rejects.toThrow('permission')
    })
  })

  describe('File System Errors', () => {
    it('should handle file not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found'))
      await expect(OperatorApi.fileRead('error_003', 'missing.gd')).rejects.toThrow('not found')
    })

    it('should handle directory not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Directory not found'))
      await expect(OperatorApi.fileList('error_003', 'missing_dir')).rejects.toThrow('not found')
    })

    it('should handle invalid file path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid file path'))
      await expect(OperatorApi.fileRead('error_003', '/invalid/path')).rejects.toThrow('Invalid')
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

    it('should handle task already completed', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task already completed'))
      await expect(OperatorApi.pauseTask('completed_task')).rejects.toThrow('already completed')
    })
  })

  describe('Approval Errors', () => {
    it('should handle invalid approval ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval ID'))
      await expect(OperatorApi.approve({
        task_id: 'error_004',
        approval_id: 'invalid',
        decision: 'approve',
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })

    it('should handle approval not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval not found'))
      await expect(OperatorApi.approve({
        task_id: 'error_004',
        approval_id: 'missing',
        decision: 'approve',
        reason: 'Test'
      })).rejects.toThrow('not found')
    })
  })

  describe('Tool Execution Errors', () => {
    it('should handle tool execution failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Tool execution failed'))
      await expect(OperatorApi.fileRead('error_005', 'bad_file.gd')).rejects.toThrow('failed')
    })

    it('should handle tool timeout', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Tool timeout'))
      await expect(OperatorApi.filePatch('error_005', 'slow.gd', 'content')).rejects.toThrow('timeout')
    })
  })

  describe('Error Recovery', () => {
    it('should recover from temporary error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Temporary error'))
      mockInvoke.mockResolvedValueOnce({ task_id: 'error_006', status: 'running', event_stream: '' })

      try {
        await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: 'Test'
        })
      } catch (e) {
        // First attempt failed
      }

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle retry on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Error 1'))
      mockInvoke.mockRejectedValueOnce(new Error('Error 2'))
      mockInvoke.mockResolvedValueOnce({ task_id: 'error_007', status: 'running', event_stream: '' })

      let retries = 0
      for (let i = 0; i < 3; i++) {
        try {
          await OperatorApi.startTask({
            domain: 'game.godot',
            project_path: '/test',
            goal: 'Test'
          })
          break
        } catch (e) {
          retries++
        }
      }

      expect(retries).toBe(2)
    })
  })

  describe('Error Messages', () => {
    it('should provide descriptive error messages', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File read failed: Permission denied for /etc/passwd'))
      try {
        await OperatorApi.fileRead('error_008', '/etc/passwd')
      } catch (e: any) {
        expect(e.message).toContain('Permission denied')
        expect(e.message).toContain('/etc/passwd')
      }
    })

    it('should include context in error messages', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task execution failed at step 3 of 5'))
      try {
        await OperatorApi.getTask('error_009')
      } catch (e: any) {
        expect(e.message).toContain('step 3')
      }
    })
  })
})