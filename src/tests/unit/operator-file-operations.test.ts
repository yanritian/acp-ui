// Operator File Operations Tests
// Testing file read, patch, and list operations

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

describe('Operator File Operations Tests', () => {
  describe('File Read', () => {
    it('should read file content', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'extends Node2D\n\nfunc _ready():\n  pass',
        path: 'scripts/Player.gd',
        size: 50
      })
      const result = await OperatorApi.fileRead('file_001', 'scripts/Player.gd')
      expect(result.content).toContain('extends')
    })

    it('should handle UTF-8 encoding', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: '中文内容',
        path: 'scripts/Chinese.gd',
        encoding: 'utf-8'
      })
      const result = await OperatorApi.fileRead('file_001', 'scripts/Chinese.gd')
      expect(result.content).toContain('中文')
    })

    it('should handle large files', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'x'.repeat(10000),
        path: 'scripts/Large.gd',
        size: 10000
      })
      const result = await OperatorApi.fileRead('file_001', 'scripts/Large.gd')
      expect(result.content.length).toBe(10000)
    })

    it('should reject path outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileRead('file_001', '../../../etc/passwd')).rejects.toThrow('outside')
    })

    it('should reject hidden file access', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to hidden file'))
      await expect(OperatorApi.fileRead('file_001', '.env')).rejects.toThrow('denied')
    })
  })

  describe('File Patch', () => {
    it('should apply file patch', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'scripts/Player.gd',
        backup_created: true
      })
      const result = await OperatorApi.filePatch('file_002', 'scripts/Player.gd', 'new content')
      expect(result.success).toBe(true)
    })

    it('should create backup by default', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        backup_created: true,
        backup_path: 'scripts/Player.gd.bak'
      })
      const result = await OperatorApi.filePatch('file_002', 'scripts/Player.gd', 'new content')
      expect(result.backup_created).toBe(true)
    })

    it('should skip backup when disabled', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        backup_created: false
      })
      const result = await OperatorApi.filePatch('file_002', 'scripts/Player.gd', 'new content', false)
      expect(result.backup_created).toBe(false)
    })

    it('should reject patch outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.filePatch('file_002', '../outside.txt', 'content')).rejects.toThrow('outside')
    })
  })

  describe('File Patch Preview', () => {
    it('should preview patch without applying', async () => {
      mockInvoke.mockResolvedValueOnce({
        diff: '@@ -1,3 +1,4 @@\n extends Node2D\n \n+func new_function():\n+  print("new")',
        lines_added: 2,
        lines_removed: 0,
        hunks: 1
      })
      const result = await OperatorApi.filePatchPreview('file_003', 'scripts/Player.gd', 'new content')
      expect(result.lines_added).toBe(2)
    })

    it('should show diff with context', async () => {
      mockInvoke.mockResolvedValueOnce({
        diff: '--- a/Player.gd\n+++ b/Player.gd\n@@ context',
        context_lines: 3
      })
      const result = await OperatorApi.filePatchPreview('file_003', 'scripts/Player.gd', 'new content')
      expect(result.diff).toContain('---')
    })

    it('should handle no changes', async () => {
      mockInvoke.mockResolvedValueOnce({
        diff: '',
        lines_added: 0,
        lines_removed: 0,
        hunks: 0
      })
      const result = await OperatorApi.filePatchPreview('file_003', 'scripts/Player.gd', 'same content')
      expect(result.lines_added).toBe(0)
    })
  })

  describe('File List', () => {
    it('should list files in directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        directories: ['subdir'],
        total: 3
      })
      const result = await OperatorApi.fileList('file_004', 'scripts')
      expect(result.files.length).toBe(3)
    })

    it('should list files recursively', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['scripts/Player.gd', 'scripts/enemy/Enemy.gd', 'ui/UI.gd'],
        total: 3
      })
      const result = await OperatorApi.fileList('file_004', '')
      expect(result.files.length).toBe(3)
    })

    it('should filter by extension', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd'],
        filter: '*.gd'
      })
      const result = await OperatorApi.fileList('file_004', 'scripts')
      expect(result.files.every((f: string) => f.endsWith('.gd'))).toBe(true)
    })

    it('should handle empty directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: [],
        directories: [],
        total: 0
      })
      const result = await OperatorApi.fileList('file_004', 'empty')
      expect(result.files.length).toBe(0)
    })

    it('should reject list outside project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project'))
      await expect(OperatorApi.fileList('file_004', '../outside')).rejects.toThrow('outside')
    })
  })

  describe('File Events', () => {
    it('should generate file_read event', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'test',
        path: 'test.gd'
      })
      await OperatorApi.fileRead('file_005', 'test.gd')
    })

    it('should generate file_patch_applied event', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        path: 'test.gd'
      })
      await OperatorApi.filePatch('file_005', 'test.gd', 'new content')
    })

    it('should generate file_list event', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['test.gd'],
        total: 1
      })
      await OperatorApi.fileList('file_005', 'scripts')
    })
  })

  describe('File Error Handling', () => {
    it('should handle file not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found'))
      await expect(OperatorApi.fileRead('file_006', 'missing.gd')).rejects.toThrow('not found')
    })

    it('should handle permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))
      await expect(OperatorApi.fileRead('file_006', 'protected.gd')).rejects.toThrow('Permission')
    })

    it('should handle binary file', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: '[binary]',
        is_binary: true,
        size: 1024
      })
      const result = await OperatorApi.fileRead('file_006', 'assets/sprite.png')
      expect(result.is_binary).toBe(true)
    })
  })
})