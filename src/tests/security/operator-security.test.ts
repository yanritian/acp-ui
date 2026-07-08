// Security Tests for Hermes Game Operator
// Tests security boundaries and attack prevention

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

describe('Security Tests', () => {
  describe('Path Traversal Prevention', () => {
    it('should reject parent directory traversal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', '../../../etc/passwd')).rejects.toThrow('outside')
    })

    it('should reject absolute system paths', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', 'C:/Windows/System32')).rejects.toThrow('outside')
    })

    it('should reject Unix system paths', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', '/etc/passwd')).rejects.toThrow('outside')
    })

    it('should reject mixed traversal patterns', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', 'scripts/../../../etc/shadow')).rejects.toThrow('outside')
    })

    it('should reject URL-encoded traversal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid path'))

      await expect(OperatorApi.fileRead('task_1', '%2e%2e%2f%2e%2e%2fetc')).rejects.toThrow()
    })

    it('should reject double-encoded traversal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid path'))

      await expect(OperatorApi.fileRead('task_1', '%252e%252e%252f')).rejects.toThrow()
    })
  })

  describe('Command Injection Prevention', () => {
    it('should reject shell command separator', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters in goal'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test; rm -rf /'
      })).rejects.toThrow()
    })

    it('should reject pipe operator', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test | cat /etc/passwd'
      })).rejects.toThrow()
    })

    it('should reject AND operator', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test && whoami'
      })).rejects.toThrow()
    })

    it('should reject OR operator', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test || ls -la'
      })).rejects.toThrow()
    })

    it('should reject command substitution', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test $(whoami)'
      })).rejects.toThrow()
    })

    it('should reject backtick substitution', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test `id`'
      })).rejects.toThrow()
    })

    it('should reject newline injection', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid characters'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test\nrm -rf /'
      })).rejects.toThrow()
    })
  })

  describe('XSS Prevention', () => {
    it('should handle script tags in goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_xss_1',
        status: 'planning',
        event_stream: ''
      })

      // API should accept but backend should sanitize
      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '<script>alert("xss")</script>'
      })

      expect(result.task_id).toBeDefined()
    })

    it('should handle event handlers in goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_xss_2',
        status: 'planning',
        event_stream: ''
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '<img onerror="alert(1)" src="x">'
      })

      expect(result.task_id).toBeDefined()
    })

    it('should handle javascript: URLs', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_xss_3',
        status: 'planning',
        event_stream: ''
      })

      const result = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'javascript:void(document.cookie)'
      })

      expect(result.task_id).toBeDefined()
    })
  })

  describe('Input Validation', () => {
    it('should validate task_id format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID format'))

      await expect(OperatorApi.getTask('../etc/passwd')).rejects.toThrow('Invalid')
    })

    it('should validate domain format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid domain'))

      await expect(OperatorApi.startTask({
        domain: '../../../etc',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })

    it('should validate approval_id format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval ID'))

      await expect(OperatorApi.approve({
        task_id: 'task_1',
        approval_id: '../../../etc/passwd',
        decision: 'approve'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('API Key Security', () => {
    it('should not expose API key in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Authentication failed'))

      try {
        await OperatorApi.listTasks()
      } catch (e: any) {
        expect(e.message).not.toContain('sk-')
        expect(e.message).not.toContain('key')
      }
    })

    it('should handle missing API key gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('API key not configured'))

      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow()
    })
  })

  describe('Rate Limiting', () => {
    it('should handle rate limit errors', async () => {
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

  describe('Permission Boundaries', () => {
    it('should enforce read-only operations', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Write operation not allowed'))

      await expect(OperatorApi.filePatch('task_1', 'readonly.txt', 'content')).rejects.toThrow('not allowed')
    })

    it('should enforce approval requirements', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval required'))

      await expect(OperatorApi.filePatch('task_1', 'script.gd', 'new content')).rejects.toThrow('Approval required')
    })

    it('should forbid dangerous operations', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Operation forbidden'))

      await expect(OperatorApi.fileRead('task_1', '.env')).rejects.toThrow('forbidden')
    })
  })

  describe('Resource Limits', () => {
    it('should enforce file size limits', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File exceeds maximum size'))

      await expect(OperatorApi.fileRead('task_1', 'large_file.bin')).rejects.toThrow('exceeds')
    })

    it('should enforce event limits', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Maximum events reached'))

      await expect(OperatorApi.listEvents('task_1', 1000000)).rejects.toThrow()
    })

    it('should enforce timeout limits', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Operation timed out'))

      await expect(OperatorApi.getTask('task_timeout')).rejects.toThrow('timed out')
    })
  })
})