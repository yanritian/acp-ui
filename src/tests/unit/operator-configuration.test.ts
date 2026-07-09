// Operator Configuration Tests
// Testing operator configuration and settings

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

describe('Operator Configuration Tests', () => {
  describe('Task Mode Configuration', () => {
    it('should use propose_then_apply mode', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_001',
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
        task_id: 'config_002',
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

  describe('Approval Policy Configuration', () => {
    it('should use safe_default policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_003',
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

    it('should use permissive policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_004',
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

    it('should use strict policy', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_005',
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

  describe('Domain Configuration', () => {
    it('should accept game.godot domain', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_006',
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
  })

  describe('Path Configuration', () => {
    it('should accept Windows path', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_007',
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
        task_id: 'config_008',
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
  })

  describe('Goal Configuration', () => {
    it('should accept Chinese goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_009',
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

    it('should accept English goal', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_010',
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

    it('should accept goal with emoji', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'config_011',
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
  })
})