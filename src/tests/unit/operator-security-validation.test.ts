// Operator Security Validation Tests
// Testing input validation and security checks

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

describe('Operator Security Validation Tests', () => {
  describe('Goal Input Validation', () => {
    it('should accept normal goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add player movement'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept Chinese goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't2',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '添加玩家移动'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept emoji in goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't3',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature 🎮'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept long goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't4',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add a new player movement system that includes walking, running, jumping, and swimming'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Path Validation', () => {
    it('should accept valid project path', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't5',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/valid/project',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should reject path traversal', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid path: path traversal detected'))
      await expect(OperatorApi.fileRead('t1', '../../../etc/passwd')).rejects.toThrow()
    })

    it('should reject absolute path outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))
      await expect(OperatorApi.fileRead('t1', '/etc/passwd')).rejects.toThrow()
    })

    it('should reject hidden file access', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to hidden file'))
      await expect(OperatorApi.fileRead('t1', '.env')).rejects.toThrow()
    })

    it('should reject .git directory access', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied'))
      await expect(OperatorApi.fileRead('t1', '.git/config')).rejects.toThrow()
    })
  })

  describe('Domain Validation', () => {
    it('should accept valid domain', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't6',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should reject unknown domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Unknown domain'))
      await expect(OperatorApi.startTask({
        domain: 'game.unreal',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Unknown domain')
    })
  })

  describe('Approval Policy Validation', () => {
    it('should accept safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't7',
        status: 'running',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'safe_default'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't8',
        status: 'running',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'permissive'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't9',
        status: 'waiting_approval',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        approval_policy: 'strict'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('File Operation Validation', () => {
    it('should reject file read outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileRead('t1', '../outside.txt')).rejects.toThrow()
    })

    it('should reject file patch outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.filePatch('t1', '../outside.txt', 'content')).rejects.toThrow()
    })

    it('should reject file list outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileList('t1', '../outside')).rejects.toThrow()
    })
  })
})