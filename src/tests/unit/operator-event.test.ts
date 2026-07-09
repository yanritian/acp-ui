// Operator Event Tests
// Testing event generation and handling

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

describe('Operator Event Tests', () => {
  describe('Task Events', () => {
    it('should generate task_created event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_001' }
      ])
      const events = await OperatorApi.listEvents('event_001')
      expect(events[0].type).toBe('task_created')
    })

    it('should generate task_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_002' }
      ])
      const events = await OperatorApi.listEvents('event_002')
      expect(events[0].type).toBe('task_started')
    })

    it('should generate task_paused event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e3', type: 'task_paused', title: 'Paused', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_003' }
      ])
      const events = await OperatorApi.listEvents('event_003')
      expect(events[0].type).toBe('task_paused')
    })

    it('should generate task_resumed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e4', type: 'task_resumed', title: 'Resumed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_004' }
      ])
      const events = await OperatorApi.listEvents('event_004')
      expect(events[0].type).toBe('task_resumed')
    })

    it('should generate task_completed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e5', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_005' }
      ])
      const events = await OperatorApi.listEvents('event_005')
      expect(events[0].type).toBe('task_completed')
    })

    it('should generate task_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e6', type: 'task_failed', title: 'Failed', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_006' }
      ])
      const events = await OperatorApi.listEvents('event_006')
      expect(events[0].type).toBe('task_failed')
    })

    it('should generate task_cancelled event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e7', type: 'task_cancelled', title: 'Cancelled', level: 'warning', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_007' }
      ])
      const events = await OperatorApi.listEvents('event_007')
      expect(events[0].type).toBe('task_cancelled')
    })
  })

  describe('Tool Events', () => {
    it('should generate tool_call_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 't1', type: 'tool_call_started', title: 'Tool Started', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_008' }
      ])
      const events = await OperatorApi.listEvents('event_008')
      expect(events[0].type).toBe('tool_call_started')
    })

    it('should generate tool_call_succeeded event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 't2', type: 'tool_call_succeeded', title: 'Tool Succeeded', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_009' }
      ])
      const events = await OperatorApi.listEvents('event_009')
      expect(events[0].type).toBe('tool_call_succeeded')
    })

    it('should generate tool_call_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 't3', type: 'tool_call_failed', title: 'Tool Failed', level: 'error', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_010' }
      ])
      const events = await OperatorApi.listEvents('event_010')
      expect(events[0].type).toBe('tool_call_failed')
    })
  })

  describe('File Events', () => {
    it('should generate file_read event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'f1', type: 'file_read', title: 'File Read', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_011' }
      ])
      const events = await OperatorApi.listEvents('event_011')
      expect(events[0].type).toBe('file_read')
    })

    it('should generate file_patch_applied event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'f2', type: 'file_patch_applied', title: 'File Patched', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_012' }
      ])
      const events = await OperatorApi.listEvents('event_012')
      expect(events[0].type).toBe('file_patch_applied')
    })
  })

  describe('Event Filtering', () => {
    it('should filter events by type', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_013' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'event_013' }
      ])
      const events = await OperatorApi.listEvents('event_013')
      const createdEvents = events.filter(e => e.type === 'task_created')
      expect(createdEvents.length).toBe(1)
    })

    it('should filter events by level', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_failed', title: 'Error', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_014' },
        { event_id: 'e2', type: 'task_completed', title: 'Success', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'event_014' }
      ])
      const events = await OperatorApi.listEvents('event_014')
      const errorEvents = events.filter(e => e.level === 'error')
      expect(errorEvents.length).toBe(1)
    })

    it('should filter events by source', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'tool', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_015' },
        { event_id: 'e2', type: 'task_started', title: 'Task', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'event_015' }
      ])
      const events = await OperatorApi.listEvents('event_015')
      const toolEvents = events.filter(e => e.source === 'tool')
      expect(toolEvents.length).toBe(1)
    })
  })

  describe('Event Ordering', () => {
    it('should return events in chronological order', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'task_created', title: 'Created', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'event_016' },
        { event_id: 'e2', type: 'task_started', title: 'Started', level: 'info', source: 'operator', timestamp: '2026-07-09T00:01:00Z', task_id: 'event_016' },
        { event_id: 'e3', type: 'task_completed', title: 'Completed', level: 'info', source: 'operator', timestamp: '2026-07-09T00:02:00Z', task_id: 'event_016' }
      ])
      const events = await OperatorApi.listEvents('event_016')
      expect(events[0].type).toBe('task_created')
      expect(events[2].type).toBe('task_completed')
    })
  })

  describe('Event Payload', () => {
    it('should include payload in events', async () => {
      mockInvoke.mockResolvedValueOnce([
        {
          event_id: 'e1',
          type: 'file_patch_applied',
          title: 'File Patched',
          level: 'info',
          source: 'tool',
          timestamp: '2026-07-09T00:00:00Z',
          task_id: 'event_017',
          payload: { path: 'test.gd', lines_changed: 10 }
        }
      ])
      const events = await OperatorApi.listEvents('event_017')
      expect(events[0]?.payload).toBeDefined()
      expect(events[0]?.payload?.path).toBe('test.gd')
    })
  })
})