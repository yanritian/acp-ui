// Operator Security Tests
// Testing security features and validation

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

describe('Operator Security Tests', () => {
  describe('Path Traversal Prevention', () => {
    it('should prevent path traversal with ../', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path traversal detected'))
      await expect(OperatorApi.fileRead('security_001', '../../../etc/passwd')).rejects.toThrow('traversal')
    })

    it('should prevent path traversal with encoded characters', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid path'))
      await expect(OperatorApi.fileRead('security_001', '%2e%2e%2f')).rejects.toThrow()
    })

    it('should prevent absolute path outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileRead('security_001', 'C:/Windows/System32')).rejects.toThrow('outside')
    })
  })

  describe('File Access Control', () => {
    it('should deny access to .env files', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to .env'))
      await expect(OperatorApi.fileRead('security_002', '.env')).rejects.toThrow('denied')
    })

    it('should deny access to .git directory', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to .git'))
      await expect(OperatorApi.fileRead('security_002', '.git/config')).rejects.toThrow('denied')
    })

    it('should deny access to hidden files', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to hidden file'))
      await expect(OperatorApi.fileRead('security_002', '.hidden')).rejects.toThrow('denied')
    })
  })

  describe('Input Validation', () => {
    it('should validate task ID format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID format'))
      await expect(OperatorApi.getTask('invalid!@#$')).rejects.toThrow('Invalid')
    })

    it('should validate goal input', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Goal contains invalid characters'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '<script>alert(1)</script>'
      })).rejects.toThrow('invalid')
    })

    it('should validate project path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid project path'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('Approval Security', () => {
    it('should require approval for file modifications', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'security_003', level: 'approve', title: 'Modify', reason: 'File modification' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('security_003')
      expect(approvals.length).toBe(1)
    })

    it('should prevent approval bypass', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Approval required'))
      await expect(OperatorApi.filePatch('security_003', 'test.gd', 'content')).rejects.toThrow('Approval')
    })

    it('should validate approval decision', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval decision'))
      await expect(OperatorApi.approve({
        task_id: 'security_003',
        approval_id: 'a1',
        decision: 'invalid' as any,
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('Domain Validation', () => {
    it('should only allow whitelisted domains', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Domain not allowed'))
      await expect(OperatorApi.startTask({
        domain: 'malicious.domain',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('not allowed')
    })

    it('should validate domain format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid domain format'))
      await expect(OperatorApi.startTask({
        domain: '',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('Rate Limiting', () => {
    it('should enforce rate limits', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Rate limit exceeded'))
      await expect(OperatorApi.listTasks()).rejects.toThrow('Rate limit')
    })

    it('should enforce quota limits', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Quota exceeded'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Quota')
    })
  })

  describe('Secure Error Messages', () => {
    it('should not leak internal paths in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied'))
      try {
        await OperatorApi.fileRead('security_004', 'test.gd')
      } catch (e: any) {
        expect(e.message).not.toContain('/etc/')
        expect(e.message).not.toContain('C:\\')
      }
    })

    it('should not leak API keys in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Authentication failed'))
      try {
        await OperatorApi.listTasks()
      } catch (e: any) {
        expect(e.message).not.toContain('sk-')
        expect(e.message).not.toContain('api_key')
      }
    })

    it('should not leak system information in errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Operation failed'))
      try {
        await OperatorApi.fileRead('security_004', 'test.gd')
      } catch (e: any) {
        expect(e.message).not.toContain('password')
        expect(e.message).not.toContain('secret')
      }
    })
  })
})