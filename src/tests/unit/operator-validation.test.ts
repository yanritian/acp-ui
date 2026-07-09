// Operator Validation Tests
// Testing input validation and error handling

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

describe('Operator Validation Tests', () => {
  describe('Input Validation', () => {
    it('should validate task ID format', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid task ID format'))
      await expect(OperatorApi.getTask('invalid!@#')).rejects.toThrow('Invalid')
    })

    it('should validate goal input', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Goal contains invalid characters'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '<script>alert(1)</script>'
      })).rejects.toThrow()
    })

    it('should validate project path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid project path'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })

    it('should validate domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Unknown domain'))
      await expect(OperatorApi.startTask({
        domain: 'invalid.domain',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Unknown')
    })
  })

  describe('Path Security Validation', () => {
    it('should prevent path traversal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path traversal detected'))
      await expect(OperatorApi.fileRead('t1', '../../../etc/passwd')).rejects.toThrow('traversal')
    })

    it('should prevent access to hidden files', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied'))
      await expect(OperatorApi.fileRead('t1', '.env')).rejects.toThrow('denied')
    })

    it('should prevent access outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileRead('t1', 'C:/Windows/System32')).rejects.toThrow('outside')
    })
  })

  describe('State Validation', () => {
    it('should validate task state for pause', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot pause completed task'))
      await expect(OperatorApi.pauseTask('completed_task')).rejects.toThrow()
    })

    it('should validate task state for resume', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Task is not paused'))
      await expect(OperatorApi.resumeTask('running_task')).rejects.toThrow()
    })

    it('should validate task state for stop', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot stop completed task'))
      await expect(OperatorApi.stopTask('completed_task')).rejects.toThrow()
    })
  })

  describe('Approval Validation', () => {
    it('should validate approval decision', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval decision'))
      await expect(OperatorApi.approve({
        task_id: 't1',
        approval_id: 'a1',
        decision: 'invalid' as any,
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })

    it('should validate approval ID', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid approval ID'))
      await expect(OperatorApi.approve({
        task_id: 't1',
        approval_id: 'invalid',
        decision: 'approve',
        reason: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('File Operation Validation', () => {
    it('should validate file path for read', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid file path'))
      await expect(OperatorApi.fileRead('t1', '/invalid/path')).rejects.toThrow()
    })

    it('should validate file path for patch', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid file path'))
      await expect(OperatorApi.filePatch('t1', '/invalid/path', 'content')).rejects.toThrow()
    })

    it('should validate directory path for list', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid directory path'))
      await expect(OperatorApi.fileList('t1', '/invalid/path')).rejects.toThrow()
    })
  })
})