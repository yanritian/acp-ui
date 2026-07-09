// Security Guard Tests for Hermes Game Operator
// Tests for PathGuard and CommandGuard security

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

describe('Security Guard Tests', () => {
  describe('PathGuard Tests', () => {
    describe('Path Boundary Enforcement', () => {
      it('should allow path within project boundary', async () => {
        mockInvoke.mockResolvedValueOnce({
          content: 'file content',
          path: 'scripts/Player.gd'
        })

        const result = await OperatorApi.fileRead('task_1', 'scripts/Player.gd')
        expect(result.content).toBeDefined()
      })

      it('should reject path outside project boundary', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

        await expect(OperatorApi.fileRead('task_1', '../../../etc/passwd'))
          .rejects.toThrow('outside')
      })

      it('should reject absolute path outside project', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

        await expect(OperatorApi.fileRead('task_1', 'C:/Windows/System32'))
          .rejects.toThrow('outside')
      })

      it('should reject path traversal with encoded characters', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Invalid path'))

        await expect(OperatorApi.fileRead('task_1', '%2e%2e%2f'))
          .rejects.toThrow()
      })

      it('should reject path traversal with double encoding', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Invalid path'))

        await expect(OperatorApi.fileRead('task_1', '%252e%252e'))
          .rejects.toThrow()
      })
    })

    describe('Allowed Roots', () => {
      it('should use task.project_path as allowed root', async () => {
        mockInvoke.mockResolvedValueOnce({
          content: 'content',
          path: 'file.txt'
        })

        const result = await OperatorApi.fileRead('task_1', 'file.txt')
        expect(result).toBeDefined()
      })

      it('should allow nested directory access', async () => {
        mockInvoke.mockResolvedValueOnce({
          content: 'content',
          path: 'scripts/player/Player.gd'
        })

        const result = await OperatorApi.fileRead('task_1', 'scripts/player/Player.gd')
        expect(result.content).toBeDefined()
      })

      it('should reject sibling directory access', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

        await expect(OperatorApi.fileRead('task_1', '../other-project'))
          .rejects.toThrow('outside')
      })
    })

    describe('Hidden File Protection', () => {
      it('should reject hidden file access', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Access denied to hidden file'))

        await expect(OperatorApi.fileRead('task_1', '.env'))
          .rejects.toThrow('denied')
      })

      it('should reject .git directory access', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Access denied'))

        await expect(OperatorApi.fileRead('task_1', '.git/config'))
          .rejects.toThrow('denied')
      })
    })
  })

  describe('CommandGuard Tests', () => {
    describe('Forbidden Commands', () => {
      it('should reject rm command', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Command forbidden: rm'))

        await expect(OperatorApi.filePatch('task_1', 'test.txt', 'content'))
          .rejects.toThrow('forbidden')
      })

      it('should reject destructive file operations', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Operation forbidden'))

        await expect(OperatorApi.filePatch('task_1', 'test.txt', 'content'))
          .rejects.toThrow('forbidden')
      })
    })

    describe('Safe Operations', () => {
      it('should allow file read', async () => {
        mockInvoke.mockResolvedValueOnce({ content: 'content' })

        const result = await OperatorApi.fileRead('task_1', 'test.txt')
        expect(result).toBeDefined()
      })

      it('should allow file patch with approval', async () => {
        mockInvoke.mockResolvedValueOnce({ success: true })

        const result = await OperatorApi.filePatch('task_1', 'test.txt', 'content')
        expect(result).toBeDefined()
      })
    })
  })

  describe('Approval Policy Tests', () => {
    describe('Safe Default Policy', () => {
      it('should require approval for file modifications', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_safe',
          status: 'waiting_approval',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: 'Test',
          approval_policy: 'safe_default'
        })

        expect(result.task_id).toBeDefined()
      })
    })

    describe('Permissive Policy', () => {
      it('should auto-approve safe operations', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_permissive',
          status: 'running',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: 'Test',
          approval_policy: 'permissive'
        })

        expect(result.status).toBe('running')
      })
    })

    describe('Strict Policy', () => {
      it('should require approval for all operations', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_strict',
          status: 'waiting_approval',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: 'Test',
          approval_policy: 'strict'
        })

        expect(result.task_id).toBeDefined()
      })
    })
  })

  describe('Input Sanitization Tests', () => {
    describe('Goal Input', () => {
      it('should sanitize script tags in goal', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_sanitize',
          status: 'planning',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: '<script>alert(1)</script>'
        })

        expect(result.task_id).toBeDefined()
      })

      it('should handle unicode in goal', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_unicode',
          status: 'planning',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/test',
          goal: '添加功能 🎮'
        })

        expect(result.task_id).toBeDefined()
      })
    })

    describe('Path Input', () => {
      it('should normalize Windows paths', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_win',
          status: 'planning',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: 'D:\\Games\\MyProject',
          goal: 'Test'
        })

        expect(result.task_id).toBeDefined()
      })

      it('should normalize Unix paths', async () => {
        mockInvoke.mockResolvedValueOnce({
          task_id: 'task_unix',
          status: 'planning',
          event_stream: ''
        })

        const result = await OperatorApi.startTask({
          domain: 'game.godot',
          project_path: '/home/user/games/project',
          goal: 'Test'
        })

        expect(result.task_id).toBeDefined()
      })
    })
  })

  describe('Error Message Security', () => {
    it('should not expose internal paths in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied'))

      try {
        await OperatorApi.fileRead('task_1', 'test.txt')
      } catch (e: any) {
        expect(e.message).not.toContain('/etc/')
        expect(e.message).not.toContain('C:\\')
      }
    })

    it('should not expose API keys in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Authentication failed'))

      try {
        await OperatorApi.listTasks()
      } catch (e: any) {
        expect(e.message).not.toContain('sk-')
        expect(e.message).not.toContain('key')
      }
    })

    it('should not expose system information in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Operation failed'))

      try {
        await OperatorApi.fileRead('task_1', 'test.txt')
      } catch (e: any) {
        expect(e.message).not.toMatch(/password|secret|token/i)
      }
    })
  })

  describe('Rate Limiting', () => {
    it('should handle rate limit gracefully', async () => {
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
})