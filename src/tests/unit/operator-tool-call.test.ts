// Operator Tool Call Tests
// Testing tool call execution and results

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

describe('Operator Tool Call Tests', () => {
  describe('File Read Tool', () => {
    it('should execute file_read tool', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D',
        path: 'Player.gd',
        size: 100
      })
      const result = await OperatorApi.fileRead('tool_001', 'Player.gd')
      expect(result.content).toContain('extends')
    })

    it('should handle tool error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found'))
      await expect(OperatorApi.fileRead('tool_001', 'missing.gd')).rejects.toThrow('not found')
    })
  })

  describe('File Patch Tool', () => {
    it('should execute file_patch tool', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'Player.gd',
        backup_created: true
      })
      const result = await OperatorApi.filePatch('tool_002', 'Player.gd', 'new content')
      expect(result.success).toBe(true)
    })

    it('should handle patch error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Patch failed'))
      await expect(OperatorApi.filePatch('tool_002', 'Player.gd', 'bad')).rejects.toThrow()
    })
  })

  describe('File List Tool', () => {
    it('should execute file_list tool', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd'],
        total: 2
      })
      const result = await OperatorApi.fileList('tool_003', 'scripts')
      expect(result.files.length).toBe(2)
    })

    it('should handle empty directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: [],
        total: 0
      })
      const result = await OperatorApi.fileList('tool_003', 'empty')
      expect(result.files.length).toBe(0)
    })
  })

  describe('Tool Call Events', () => {
    it('should generate tool_call_started event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_started', title: 'Tool', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'tool_004' }
      ])
      const events = await OperatorApi.listEvents('tool_004')
      expect(events[0].type).toBe('tool_call_started')
    })

    it('should generate tool_call_succeeded event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_succeeded', title: 'Success', level: 'info', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'tool_005' }
      ])
      const events = await OperatorApi.listEvents('tool_005')
      expect(events[0].type).toBe('tool_call_succeeded')
    })

    it('should generate tool_call_failed event', async () => {
      mockInvoke.mockResolvedValueOnce([
        { event_id: 'e1', type: 'tool_call_failed', title: 'Failed', level: 'error', source: 'operator', timestamp: '2026-07-09T00:00:00Z', task_id: 'tool_006' }
      ])
      const events = await OperatorApi.listEvents('tool_006')
      expect(events[0].type).toBe('tool_call_failed')
    })
  })

  describe('Tool Approval Levels', () => {
    it('should require approval for file_patch', async () => {
      mockInvoke.mockResolvedValueOnce([
        { approval_id: 'a1', task_id: 'tool_007', level: 'approve', title: 'Patch', reason: 'Modify file' }
      ])
      const approvals = await OperatorApi.getPendingApprovals('tool_007')
      expect(approvals.length).toBe(1)
    })

    it('should auto-approve file_read', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const approvals = await OperatorApi.getPendingApprovals('tool_008')
      expect(approvals.length).toBe(0)
    })
  })
})