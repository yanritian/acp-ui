// File Operations Tests for Hermes Game Operator
// Tests file read, patch, preview, and list operations

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock Tauri invoke
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

describe('File Operations Tests', () => {
  describe('File Read', () => {
    it('should read file content', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'extends CharacterBody2D\nconst SPEED = 300.0\n',
        path: 'scripts/Player.gd'
      })

      const result = await OperatorApi.fileRead('task_1', 'scripts/Player.gd')
      expect(result.content).toContain('SPEED')
      expect(result.path).toBe('scripts/Player.gd')
    })

    it('should read large file', async () => {
      const largeContent = 'line\n'.repeat(10000)
      mockInvoke.mockResolvedValueOnce({
        content: largeContent,
        path: 'large_file.txt'
      })

      const result = await OperatorApi.fileRead('task_1', 'large_file.txt')
      expect(result.content).toBeDefined()
      expect(result.path).toBe('large_file.txt')
    })

    it('should read binary file', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'binary data...',
        path: 'assets/sprite.png',
        isBinary: true
      })

      const result = await OperatorApi.fileRead('task_1', 'assets/sprite.png')
      expect(result.path).toBe('assets/sprite.png')
    })

    it('should handle file not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found: missing.gd'))

      await expect(OperatorApi.fileRead('task_1', 'missing.gd'))
        .rejects.toThrow('not found')
    })

    it('should handle permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))

      await expect(OperatorApi.fileRead('task_1', 'protected.txt'))
        .rejects.toThrow('Permission')
    })

    it('should read nested file', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'nested content',
        path: 'scripts/enemies/boss/Boss.gd'
      })

      const result = await OperatorApi.fileRead('task_1', 'scripts/enemies/boss/Boss.gd')
      expect(result.path).toBe('scripts/enemies/boss/Boss.gd')
    })
  })

  describe('File Patch', () => {
    it('should apply patch successfully', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        backupPath: 'scripts/Player.gd.bak'
      })

      const result = await OperatorApi.filePatch('task_1', 'scripts/Player.gd', 'new content')
      expect(result.success).toBe(true)
      expect(result.backupPath).toBeDefined()
    })

    it('should patch without backup', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true
      })

      const result = await OperatorApi.filePatch('task_1', 'scripts/test.gd', 'content', false)
      expect(result.success).toBe(true)
    })

    it('should handle patch conflict', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Patch conflict: file was modified'))

      await expect(OperatorApi.filePatch('task_1', 'scripts/Player.gd', 'content'))
        .rejects.toThrow('conflict')
    })

    it('should handle invalid patch content', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid patch content'))

      await expect(OperatorApi.filePatch('task_1', 'test.txt', ''))
        .rejects.toThrow('Invalid')
    })

    it('should create new file with patch', async () => {
      mockInvoke.mockResolvedValueOnce({
        success: true,
        created: true
      })

      const result = await OperatorApi.filePatch('task_1', 'scripts/NewScript.gd', 'new file content')
      expect(result.success).toBe(true)
    })
  })

  describe('File Patch Preview', () => {
    it('should preview patch correctly', async () => {
      mockInvoke.mockResolvedValueOnce({
        oldContent: 'extends CharacterBody2D\nconst SPEED = 300.0\n',
        newContent: 'extends CharacterBody2D\nconst SPEED = 400.0\n',
        diff: '--- old\n+++ new\n@@ -1,2 +1,2 @@\n extends CharacterBody2D\n-const SPEED = 300.0\n+const SPEED = 400.0\n'
      })

      const preview = await OperatorApi.filePatchPreview('task_1', 'scripts/Player.gd', 'const SPEED = 400.0')
      expect(preview.diff).toContain('SPEED')
      expect(preview.oldContent).toBeDefined()
      expect(preview.newContent).toBeDefined()
    })

    it('should preview empty diff', async () => {
      mockInvoke.mockResolvedValueOnce({
        oldContent: 'same content',
        newContent: 'same content',
        diff: ''
      })

      const preview = await OperatorApi.filePatchPreview('task_1', 'test.txt', 'same content')
      expect(preview.diff).toBe('')
    })

    it('should preview large diff', async () => {
      mockInvoke.mockResolvedValueOnce({
        oldContent: 'old\n'.repeat(100),
        newContent: 'new\n'.repeat(100),
        diff: '@@ -1,100 +1,100 @@\n' + 'diff content...'
      })

      const preview = await OperatorApi.filePatchPreview('task_1', 'large.txt', 'new content')
      expect(preview.diff).toBeDefined()
    })
  })

  describe('File List', () => {
    it('should list files in directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['Player.gd', 'Enemy.gd', 'UI.gd'],
        directories: ['subdir']
      })

      const result = await OperatorApi.fileList('task_1', 'scripts')
      expect(result.files).toHaveLength(3)
      expect(result.directories).toHaveLength(1)
    })

    it('should list empty directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: [],
        directories: []
      })

      const result = await OperatorApi.fileList('task_1', 'empty_dir')
      expect(result.files).toHaveLength(0)
    })

    it('should list nested directory', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: ['Boss.gd', 'Minion.gd'],
        directories: ['boss', 'minions']
      })

      const result = await OperatorApi.fileList('task_1', 'scripts/enemies')
      expect(result.files).toHaveLength(2)
    })

    it('should handle non-existent directory', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Directory not found'))

      await expect(OperatorApi.fileList('task_1', 'nonexistent'))
        .rejects.toThrow('not found')
    })
  })

  describe('File Path Security', () => {
    it('should reject path traversal attempt', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', '../../../etc/passwd'))
        .rejects.toThrow('outside')
    })

    it('should reject absolute system path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path outside project boundary'))

      await expect(OperatorApi.fileRead('task_1', 'C:/Windows/System32'))
        .rejects.toThrow('outside')
    })

    it('should reject hidden file access', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied to hidden file'))

      await expect(OperatorApi.fileRead('task_1', '.env'))
        .rejects.toThrow('denied')
    })

    it('should allow valid relative path', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'content',
        path: 'scripts/Player.gd'
      })

      const result = await OperatorApi.fileRead('task_1', 'scripts/Player.gd')
      expect(result.content).toBeDefined()
    })
  })

  describe('Concurrent File Operations', () => {
    it('should handle concurrent reads', async () => {
      mockInvoke.mockResolvedValue({
        content: 'content',
        path: 'test.txt'
      })

      const results = await Promise.all([
        OperatorApi.fileRead('task_1', 'file1.txt'),
        OperatorApi.fileRead('task_1', 'file2.txt'),
        OperatorApi.fileRead('task_1', 'file3.txt')
      ])

      expect(results).toHaveLength(3)
    })

    it('should handle mixed operations', async () => {
      mockInvoke.mockResolvedValueOnce({ content: 'content', path: 'read.txt' })
      mockInvoke.mockResolvedValueOnce({ success: true })
      mockInvoke.mockResolvedValueOnce({ files: [], directories: [] })

      const [read, patch, list] = await Promise.all([
        OperatorApi.fileRead('task_1', 'read.txt'),
        OperatorApi.filePatch('task_1', 'patch.txt', 'content'),
        OperatorApi.fileList('task_1', 'dir')
      ])

      expect(read.content).toBeDefined()
      expect(patch.success).toBe(true)
      expect(list.files).toBeDefined()
    })
  })

  describe('File Operation Performance', () => {
    it('should read file within timeout', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'content',
        path: 'test.txt'
      })

      const start = performance.now()
      await OperatorApi.fileRead('task_1', 'test.txt')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })

    it('should apply patch within timeout', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true })

      const start = performance.now()
      await OperatorApi.filePatch('task_1', 'test.txt', 'content')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })

    it('should list files within timeout', async () => {
      mockInvoke.mockResolvedValueOnce({
        files: Array.from({ length: 100 }, (_, i) => `file_${i}.txt`),
        directories: []
      })

      const start = performance.now()
      await OperatorApi.fileList('task_1', 'large_dir')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })
  })

  describe('File Encoding', () => {
    it('should read UTF-8 file', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: '你好世界 Hello World',
        path: 'unicode.txt'
      })

      const result = await OperatorApi.fileRead('task_1', 'unicode.txt')
      expect(result.content).toContain('你好世界')
    })

    it('should read file with special characters', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: 'Special: © ® ™ € £ ¥',
        path: 'special.txt'
      })

      const result = await OperatorApi.fileRead('task_1', 'special.txt')
      expect(result.content).toContain('©')
    })

    it('should read file with emojis', async () => {
      mockInvoke.mockResolvedValueOnce({
        content: '🎮 Game Development 🕹️',
        path: 'emoji.txt'
      })

      const result = await OperatorApi.fileRead('task_1', 'emoji.txt')
      expect(result.content).toContain('🎮')
    })
  })
})