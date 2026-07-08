// File Change Tracking Tests
// Tests for the file change tracking feature in OperatorState

import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock Tauri invoke for testing
const mockInvoke = vi.fn()

// Setup global mock
beforeEach(() => {
  mockInvoke.mockReset()
  ;(globalThis as any).__TAURI_INTERNALS__ = {
    invoke: mockInvoke
  }
})

describe('File Change Tracking', () => {
  describe('Task Summary with File Changes', () => {
    it('should return empty file changes for new task', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_001',
        status: 'planning',
        goal: 'Test goal',
        summary: 'Task in progress',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 0,
        iterations: 5,
        errors: [],
        warnings: []
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_001')

      expect(summary.files_changed).toEqual([])
      expect(summary.files_created).toEqual([])
      expect(summary.files_deleted).toEqual([])
    })

    it('should track modified files in task summary', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_002',
        status: 'completed',
        goal: 'Add double jump',
        summary: 'Task completed successfully',
        files_changed: ['scripts/Player.gd'],
        files_created: [],
        files_deleted: [],
        duration_seconds: 120,
        iterations: 15,
        errors: [],
        warnings: []
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_002')

      expect(summary.files_changed).toContain('scripts/Player.gd')
      expect(summary.files_created).toEqual([])
      expect(summary.duration_seconds).toBe(120)
    })

    it('should categorize different file change types', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_003',
        status: 'completed',
        goal: 'Refactor player',
        summary: 'Refactoring complete',
        files_changed: ['scripts/Player.gd', 'scripts/Movement.gd'],
        files_created: ['scripts/NewAbility.gd'],
        files_deleted: ['scripts/OldCode.gd'],
        duration_seconds: 300,
        iterations: 45,
        errors: [],
        warnings: ['Deprecated API used']
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_003')

      expect(summary.files_changed).toHaveLength(2)
      expect(summary.files_created).toHaveLength(1)
      expect(summary.files_deleted).toHaveLength(1)
      expect(summary.warnings).toContain('Deprecated API used')
    })

    it('should extract errors from events', async () => {
      mockInvoke.mockResolvedValueOnce({
        task_id: 'task_004',
        status: 'failed',
        goal: 'Invalid operation',
        summary: 'Task failed with errors',
        files_changed: [],
        files_created: [],
        files_deleted: [],
        duration_seconds: 30,
        iterations: 3,
        errors: ['File not found: test.gd', 'Permission denied'],
        warnings: []
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const summary = await OperatorApi.getTaskSummary('task_004')

      expect(summary.errors).toHaveLength(2)
      expect(summary.errors[0]).toContain('File not found')
      expect(summary.status).toBe('failed')
    })
  })

  describe('File Patch with Change Recording', () => {
    it('should record file modification', async () => {
      mockInvoke.mockResolvedValueOnce({
        path: 'scripts/Player.gd',
        success: true,
        lines_changed: 5,
        backup_path: 'scripts/Player.gd.bak'
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const result = await OperatorApi.filePatch(
        'task_001',
        'scripts/Player.gd',
        'new content',
        true
      )

      expect(result.success).toBe(true)
      expect(result.lines_changed).toBe(5)
    })

    it('should handle patch preview without modifying', async () => {
      mockInvoke.mockResolvedValueOnce({
        path: 'scripts/Player.gd',
        original_content: 'original',
        new_content: 'new',
        diff: '--- original\n+++ new\n@@ -1 +1 @@\n-original\n+new'
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const preview = await OperatorApi.filePatchPreview(
        'task_001',
        'scripts/Player.gd',
        'new content'
      )

      expect(preview.diff).toContain('--- original')
      expect(preview.path).toBe('scripts/Player.gd')
    })
  })

  describe('File List Operation', () => {
    it('should list files in directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        path: 'scripts',
        files: ['Player.gd', 'Enemy.gd', 'Utils.gd'],
        directories: []
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const result = await OperatorApi.fileList('task_001', 'scripts')

      expect(result.files).toHaveLength(3)
      expect(result.files).toContain('Player.gd')
    })

    it('should list directories', async () => {
      mockInvoke.mockResolvedValueOnce({
        path: '.',
        files: ['project.godot'],
        directories: ['scripts', 'scenes', 'assets']
      })

      const { OperatorApi } = await import('@/api/operatorApi')
      const result = await OperatorApi.fileList('task_001', '.')

      expect(result.directories).toHaveLength(3)
      expect(result.directories).toContain('scripts')
    })
  })
})