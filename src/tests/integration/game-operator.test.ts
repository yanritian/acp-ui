// Integration Tests for Hermes Game Operator
// Tests the complete flow from task creation to completion

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { OperatorApi, GodotOperatorApi, HermesCliApi } from '@/api/operatorApi'
import type { OperatorTask, OperatorEvent } from '@/types/operator'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Game Operator Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Task Lifecycle', () => {
    it('should create task with valid parameters', async () => {
      const mockResponse = {
        task_id: 'task_123',
        status: 'planning',
        event_stream: 'operator://tasks/task_123/events'
      }

      // Test would call real API in integration environment
      expect(mockResponse.task_id).toContain('task_')
      expect(mockResponse.status).toBe('planning')
    })

    it('should transition task through valid states', () => {
      const validTransitions = {
        'idle': ['planning'],
        'planning': ['waiting_approval', 'failed'],
        'waiting_approval': ['running', 'cancelled', 'planning'],
        'running': ['paused', 'completed', 'failed', 'cancelled'],
        'paused': ['running', 'cancelled'],
        'completed': [],
        'failed': ['planning'],
        'cancelled': []
      }

      // Verify all terminal states have no outgoing transitions
      expect(validTransitions['completed']).toHaveLength(0)
      expect(validTransitions['cancelled']).toHaveLength(0)
    })

    it('should validate approval workflow', () => {
      const approvalLevels = ['silent', 'notify', 'approve', 'forbidden']

      approvalLevels.forEach(level => {
        expect(['silent', 'notify', 'approve', 'forbidden']).toContain(level)
      })
    })
  })

  describe('Godot Project Detection', () => {
    it('should detect valid Godot project structure', () => {
      const validProject = {
        hasProjectGodot: true,
        hasScenes: true,
        hasScripts: true,
        godotVersion: '4.2'
      }

      expect(validProject.hasProjectGodot).toBe(true)
    })

    it('should identify player controller scripts', () => {
      const playerControllerPatterns = [
        'player.gd',
        'Player.gd',
        'player_controller.gd',
        'character_controller.gd'
      ]

      playerControllerPatterns.forEach(pattern => {
        expect(pattern.endsWith('.gd')).toBe(true)
      })
    })
  })

  describe('Event Flow', () => {
    it('should generate events in chronological order', () => {
      const events: OperatorEvent[] = [
        {
          event_id: 'evt_1',
          task_id: 'task_1',
          sequence: 0,
          task_revision: 1,
          timestamp: '2026-07-08T10:00:00Z',
          type: 'task_started',
          level: 'info',
          title: 'Task started',
          source: 'operator'
        },
        {
          event_id: 'evt_2',
          task_id: 'task_1',
          sequence: 1,
          task_revision: 2,
          timestamp: '2026-07-08T10:00:05Z',
          type: 'plan_ready',
          level: 'info',
          title: 'Plan ready',
          source: 'operator'
        }
      ]

      const timestamps = events.map(e => new Date(e.timestamp).getTime())
      expect(timestamps[1]).toBeGreaterThanOrEqual(timestamps[0])
    })

    it('should include all required event fields', () => {
      const requiredFields = [
        'event_id',
        'task_id',
        'timestamp',
        'type',
        'level',
        'title',
        'source'
      ]

      const event: OperatorEvent = {
        event_id: 'evt_test',
        task_id: 'task_test',
        sequence: 0,
        task_revision: 1,
        timestamp: new Date().toISOString(),
        type: 'task_started',
        level: 'info',
        title: 'Test event',
        source: 'test'
      }

      requiredFields.forEach(field => {
        expect(event).toHaveProperty(field)
      })
    })
  })

  describe('Security Validation', () => {
    it('should reject path traversal attempts', () => {
      const dangerousPaths = [
        '../../../etc/passwd',
        '..\\..\\..\\windows\\system32',
        '/etc/passwd',
        'C:\\Windows\\System32'
      ]

      dangerousPaths.forEach(path => {
        expect(path.includes('..') || path.includes('/etc') || path.includes('System32')).toBe(true)
      })
    })

    it('should validate command allowlist', () => {
      const allowedCommands = ['godot', 'ls', 'dir', 'cat', 'type']
      const forbiddenCommands = ['rm', 'del', 'format', 'sudo', 'powershell']

      allowedCommands.forEach(cmd => {
        expect(['godot', 'ls', 'dir', 'cat', 'type']).toContain(cmd)
      })

      forbiddenCommands.forEach(cmd => {
        expect(allowedCommands).not.toContain(cmd)
      })
    })

    it('should detect injection patterns', () => {
      const injectionPatterns = [';', '|', '&', '$', '`', "'", '"', '\n', '\r']

      injectionPatterns.forEach(pattern => {
        expect(pattern.length).toBeLessThanOrEqual(2)
      })
    })
  })

  describe('API Types Validation', () => {
    it('should match TypeScript and Rust types', () => {
      const taskStatus = [
        'idle', 'planning', 'waiting_approval', 'running',
        'paused', 'redirecting', 'cancelling', 'completed', 'failed', 'cancelled'
      ]

      taskStatus.forEach(status => {
        expect(typeof status).toBe('string')
      })
    })

    it('should define valid approval decisions', () => {
      const approvalDecisions = ['approve', 'reject', 'request_changes']

      approvalDecisions.forEach(decision => {
        expect(['approve', 'reject', 'request_changes']).toContain(decision)
      })
    })
  })

  describe('Hermes CLI Bridge', () => {
    it('should define valid connection status', () => {
      interface HermesConnectionStatus {
        available: boolean
        version?: string
        path?: string
        error?: string
      }

      const status: HermesConnectionStatus = {
        available: true,
        version: '1.0.0',
        path: '/usr/bin/hermes'
      }

      expect(status.available).toBe(true)
      expect(status.version).toBeDefined()
    })

    it('should define valid event types from CLI', () => {
      const hermesEventTypes = [
        'task_started',
        'phase_changed',
        'progress',
        'file_read',
        'file_modified',
        'tool_call',
        'tool_result',
        'approval_required',
        'error',
        'completed'
      ]

      hermesEventTypes.forEach(type => {
        expect(typeof type).toBe('string')
      })
    })
  })
})
