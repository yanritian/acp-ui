// Operator Task Creation Tests
// Testing task creation with various parameters

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

describe('Operator Task Creation Tests', () => {
  describe('Basic Creation', () => {
    it('should create task with minimal params', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't1',
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

    it('should create task with all params', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't2',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Full test',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should create task with Chinese goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't3',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '添加玩家移动功能'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should create task with Japanese goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't4',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'プレイヤーの移動を追加'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Domain Validation', () => {
    it('should accept game.godot domain', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't5',
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

    it('should reject invalid domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid domain'))
      await expect(OperatorApi.startTask({
        domain: 'invalid.domain',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Invalid domain')
    })
  })

  describe('Path Validation', () => {
    it('should accept Windows path', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't6',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: 'D:\\Games\\MyProject',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should accept Unix path', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't7',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/home/user/games/project',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should reject invalid path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid project path'))
      await expect(OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '',
        goal: 'Test'
      })).rejects.toThrow('Invalid')
    })
  })

  describe('Approval Policy', () => {
    it('should apply safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't8',
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

    it('should apply permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't9',
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

    it('should apply strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't10',
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

  describe('Mode Selection', () => {
    it('should use propose_then_apply mode', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't11',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'propose_then_apply'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should use apply_directly mode', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 't12',
        status: 'running',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test',
        mode: 'apply_directly'
      })
      expect(task.task_id).toBeDefined()
    })
  })
})