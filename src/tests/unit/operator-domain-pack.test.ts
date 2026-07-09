// Operator Domain Pack Tests
// Testing domain pack integration and execution

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

describe('Operator Domain Pack Tests', () => {
  describe('Godot Domain Pack', () => {
    it('should create task with game.godot domain', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'domain_001',
        status: 'planning',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature'
      })
      expect(task.task_id).toBeDefined()
    })

    it('should analyze project with Godot domain', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'domain_002',
        status: 'running',
        event_stream: ''
      })
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: '/godot/project',
        goal: 'Analyze project'
      })
      expect(task.task_id).toBeDefined()
    })
  })

  describe('Domain Validation', () => {
    it('should reject invalid domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Unknown domain'))
      await expect(OperatorApi.startTask({
        domain: 'invalid.domain',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow('Unknown')
    })

    it('should reject empty domain', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Domain cannot be empty'))
      await expect(OperatorApi.startTask({
        domain: '',
        project_path: '/test',
        goal: 'Test'
      })).rejects.toThrow()
    })
  })

  describe('Domain-Specific Operations', () => {
    it('should handle Godot-specific file operations', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd'
      })
      const result = await OperatorApi.fileRead('domain_003', 'scripts/Player.gd')
      expect(result.content).toBeDefined()
    })

    it('should handle Godot scene files', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: '[gd_scene]',
        path: 'Main.tscn'
      })
      const result = await OperatorApi.fileRead('domain_004', 'scenes/Main.tscn')
      expect(result.content).toBeDefined()
    })
  })

  describe('Domain Pack Events', () => {
    it('should generate domain-specific events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'domain_005' },
        { event_id: 'e2', type: 'project_analyzed', title: 'Analyzed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'domain_005' }
      ])
      const events = await OperatorApi.listEvents('domain_005')
      expect(events.length).toBe(2)
    })
  })
})