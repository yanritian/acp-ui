// Operator Edge Cases Tests
// Testing boundary conditions and edge cases

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

describe('Operator Edge Cases Tests', () => {
  describe('Empty Inputs', () => {
    it('should handle empty task list', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const tasks = await OperatorApi.listTasks()
      expect(tasks.length).toBe(0)
    })

    it('should handle empty event list', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const events = await OperatorApi.listEvents('t1')
      expect(events.length).toBe(0)
    })

    it('should handle empty approval list', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const approvals = await OperatorApi.getPendingApprovals('t1')
      expect(approvals.length).toBe(0)
    })

    it('should handle empty file list', async () => {
      mockInvoke.mockResolvedValueOnce({ files: [], directories: [], total: 0 })
      const result = await OperatorApi.fileList('t1', 'empty_dir')
      expect(result.files.length).toBe(0)
    })
  })

  describe('Maximum Limits', () => {
    it('should handle long goal text', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't1', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'A'.repeat(1000)
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle long file path', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'test', path: 'long_path' })
      const result = await OperatorApi.fileRead('t1', 'a/'.repeat(50) + 'file.gd')
      expect(result.content).toBeDefined()
    })

    it('should handle many files in list', async () => {
      mockInvoke.mockResolvedValueOnce({ files: Array(100).fill('file.gd'), total: 100 })
      const result = await OperatorApi.fileList('t1', 'many_files')
      expect(result.files.length).toBe(100)
    })
  })

  describe('Unicode Handling', () => {
    it('should handle Chinese characters', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't1', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '添加玩家移动功能，包括行走、跑步和跳跃'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle Japanese characters', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't2', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'プレイヤーの移動を追加する'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle Korean characters', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't3', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: '플레이어 이동 추가'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle emoji', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't4', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature 🎮 🕹️ 🎯'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Special Characters', () => {
    it('should handle paths with spaces', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't5', status: 'planning', event_stream: '' })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/path with spaces/project',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should handle paths with dots', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'test', path: 'file.test.gd' })
      const result = await OperatorApi.fileRead('t1', 'scripts/file.test.gd')
      expect(result.content).toBeDefined()
    })

    it('should handle paths with dashes', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'test', path: 'my-file.gd' })
      const result = await OperatorApi.fileRead('t1', 'scripts/my-file.gd')
      expect(result.content).toBeDefined()
    })

    it('should handle paths with underscores', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'test', path: 'my_file.gd' })
      const result = await OperatorApi.fileRead('t1', 'scripts/my_file.gd')
      expect(result.content).toBeDefined()
    })
  })

  describe('Rapid Operations', () => {
    it('should handle rapid start-stop', async () => {
      mockInvoke.mockResolvedValueOnce({ task_id: 't_rapid', status: 'planning', event_stream: '' })
      mockInvoke.mockResolvedValueOnce(undefined)

      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Test'
      })
      expect(task.task_id).toBeDefined()
      await OperatorApi.stopTask('t_rapid')
    })

    it('should handle rapid pause-resume', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      await OperatorApi.pauseTask('t1')
      await OperatorApi.resumeTask('t1')
    })
  })
})