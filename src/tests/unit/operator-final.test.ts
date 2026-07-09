// Operator Final Tests
// Final comprehensive tests

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

describe('Operator Final Tests', () => {
  describe('Comprehensive Task Lifecycle', () => {
    it('should complete full task lifecycle', async () => {
      // Create
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_001',
        status: 'planning',
        event_stream: ''
      })
      await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Final test'
      })

      // Get task
      mockInvoke.mockResolvedValueOnce({
        task_id: 'final_001',
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Final test',
        status: 'running',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default',
        created_at: '2026-07-09T00:00:00Z',
        updated_at: '2026-07-09T00:00:00Z'
      })
      const task = await OperatorApi.getTask('final_001')
      expect(task.task_id).toBe('final_001')
    })
  })

  describe('Comprehensive File Operations', () => {
    it('should complete file read-patch-list cycle', async () => {
      // Read
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const readResult = await OperatorApi.fileRead('final_002', 'Player.gd')
      expect(readResult.content).toBeDefined()

      // Patch
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd'
      })
      const patchResult = await OperatorApi.filePatch('final_002', 'Player.gd', 'new content')
      expect(patchResult.success).toBe(true)

      // List
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd'],
        total: 2
      })
      const listResult = await OperatorApi.fileList('final_002', 'scripts')
      expect(listResult.files.length).toBe(2)
    })
  })

  describe('Comprehensive Event Tracking', () => {
    it('should track events throughout task', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'final_003' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'final_003' },
        { event_id: 'e3', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'final_003' }
      ])
      const events = await OperatorApi.listEvents('final_003')
      expect(events.length).toBe(3)
    })
  })
})