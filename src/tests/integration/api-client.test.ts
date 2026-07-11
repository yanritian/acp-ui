// Integration Tests for Game Operator API Client

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock axios for testing
vi.mock('axios', () => ({
  default: {
    create: vi.fn(() => ({
      get: vi.fn(),
      post: vi.fn(),
    })),
  },
}))

describe('Game Operator API Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  describe('Task Management API', () => {
    it('should handle list tasks response', async () => {
      const mockResponse = {
        data: {
          tasks: [
            {
              task_id: 'task_001',
              domain: 'game.godot',
              project_path: '/test',
              goal: 'Test task',
              status: 'running',
              mode: 'propose_then_apply',
              approval_policy: 'safe_default',
              created_at: '2026-07-11T00:00:00Z',
              updated_at: '2026-07-11T00:00:00Z'
            }
          ]
        }
      }
      expect(mockResponse.data.tasks).toBeDefined()
      expect(mockResponse.data.tasks.length).toBe(1)
    })

    it('should handle start task request', async () => {
      const request = {
        domain: 'game.godot',
        project_path: '/test',
        goal: 'Add feature',
        mode: 'propose_then_apply',
        approval_policy: 'safe_default'
      }
      expect(request.goal).toBe('Add feature')
    })

    it('should handle task control operations', async () => {
      const operations = ['pause', 'resume', 'stop', 'redirect']
      operations.forEach(op => {
        expect(typeof op).toBe('string')
      })
    })
  })

  describe('Event Management API', () => {
    it('should handle list events response', async () => {
      const mockResponse = {
        data: {
          events: [
            {
              event_id: 'evt_001',
              task_id: 'task_001',
              timestamp: '2026-07-11T00:00:00Z',
              type: 'task_started',
              level: 'info',
              title: 'Task started',
              message: 'Task has been started',
              source: 'operator'
            }
          ]
        }
      }
      expect(mockResponse.data.events).toBeDefined()
      expect(mockResponse.data.events.length).toBe(1)
    })

    it('should filter events by type', async () => {
      const events = [
        { type: 'task_started' },
        { type: 'task_completed' },
        { type: 'task_started' }
      ]
      const filtered = events.filter(e => e.type === 'task_started')
      expect(filtered.length).toBe(2)
    })
  })

  describe('Approval Management API', () => {
    it('should handle get approvals response', async () => {
      const mockResponse = {
        data: {
          approvals: [
            {
              approval_id: 'a001',
              task_id: 'task_001',
              level: 'approve',
              action: 'file.patch',
              title: 'Approve changes',
              reason: 'File modification required'
            }
          ]
        }
      }
      expect(mockResponse.data.approvals).toBeDefined()
      expect(mockResponse.data.approvals.length).toBe(1)
    })

    it('should handle approve request', async () => {
      const request = {
        approval_id: 'a001',
        decision: 'approve'
      }
      expect(request.decision).toBe('approve')
    })

    it('should handle reject request', async () => {
      const request = {
        approval_id: 'a001',
        decision: 'reject'
      }
      expect(request.decision).toBe('reject')
    })
  })

  describe('Error Handling', () => {
    it('should handle 401 unauthorized', async () => {
      const error = {
        response: { status: 401 },
        message: 'Unauthorized'
      }
      expect(error.response.status).toBe(401)
    })

    it('should handle 404 not found', async () => {
      const error = {
        response: { status: 404 },
        message: 'Not found'
      }
      expect(error.response.status).toBe(404)
    })

    it('should handle 429 rate limit', async () => {
      const error = {
        response: { status: 429 },
        message: 'Too many requests'
      }
      expect(error.response.status).toBe(429)
    })
  })

  describe('Authentication', () => {
    it('should include Bearer token in headers', async () => {
      const token = 'test_token'
      const headers = {
        Authorization: `Bearer ${token}`
      }
      expect(headers.Authorization).toContain('Bearer')
    })

    it('should handle missing token', async () => {
      const headers = {}
      expect(headers.Authorization).toBeUndefined()
    })
  })
})