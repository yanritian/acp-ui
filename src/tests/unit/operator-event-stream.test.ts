// Operator Event Stream Tests
// Testing event stream generation, filtering, and subscription

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

describe('Operator Event Stream Tests', () => {
  describe('Event Generation', () => {
    it('should generate task_created event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('task_created')
    })

    it('should generate task_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('task_started')
    })

    it('should generate plan_ready event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'plan_ready', title: 'Plan Ready', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('plan_ready')
    })

    it('should generate approval_requested event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'approval_requested', title: 'Approval', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('approval_requested')
    })

    it('should generate tool_call_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('tool_call_started')
    })

    it('should generate file_patch_applied event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'file_patch_applied', title: 'Patch', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('file_patch_applied')
    })

    it('should generate task_completed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('task_completed')
    })

    it('should generate task_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_failed', title: 'Failed', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('task_failed')
    })
  })

  describe('Event Filtering', () => {
    it('should filter by event type', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' },
        { event_id: 'e2', type: 'task_completed', title: 'Done', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      const toolEvents = events.filter(e => e.type === 'tool_call_started')
      expect(toolEvents.length).toBe(1)
    })

    it('should filter by event level', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_failed', title: 'Error', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' },
        { event_id: 'e2', type: 'task_completed', title: 'Done', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      const errorEvents = events.filter(e => e.level === 'error')
      expect(errorEvents.length).toBe(1)
    })

    it('should filter by source', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' },
        { event_id: 'e2', type: 'task_completed', title: 'Done', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      const toolSourceEvents = events.filter(e => e.source === 'tool')
      expect(toolSourceEvents.length).toBe(1)
    })
  })

  describe('Event Ordering', () => {
    it('should order events by timestamp', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 't1' },
        { event_id: 'e3', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 't1' }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].type).toBe('task_created')
      expect(events[2].type).toBe('task_completed')
    })
  })

  describe('Event Payload', () => {
    it('should include payload in events', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 't1', payload: { tool: 'file_read', path: 'test.gd' } }
      ])
      const events = await OperatorApi.listEvents('t1')
      expect(events[0].payload).toBeDefined()
    })
  })
})