// Godot Project Detection Tests for Hermes Game Operator
// Tests Godot project detection, analysis, and scene parsing

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

import { GodotOperatorApi } from '@/api/operatorApi'

describe('Godot Project Detection Tests', () => {
  describe('Project Detection', () => {
    it('should detect valid Godot project', async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const result = await GodotOperatorApi.detectProject('D:/games/my-godot-project')
      expect(result).toBe(true)
    })

    it('should reject non-Godot project', async () => {
      mockInvoke.mockResolvedValueOnce(false)

      const result = await GodotOperatorApi.detectProject('D:/non-godot-project')
      expect(result).toBe(false)
    })

    it('should detect Godot 4.x project', async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const result = await GodotOperatorApi.detectProject('D:/games/godot4-project')
      expect(result).toBe(true)
    })

    it('should detect Godot 3.x project', async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const result = await GodotOperatorApi.detectProject('D:/games/godot3-project')
      expect(result).toBe(true)
    })

    it('should handle invalid path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path does not exist'))

      await expect(GodotOperatorApi.detectProject('/invalid/path'))
        .rejects.toThrow('does not exist')
    })

    it('should handle path with spaces', async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const result = await GodotOperatorApi.detectProject('D:/My Games/My Project')
      expect(result).toBe(true)
    })
  })

  describe('Project Analysis', () => {
    it('should analyze basic Godot project', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'TestProject',
        godot_version: '4.2.1',
        scripts: ['scripts/Player.gd'],
        scenes: ['main.tscn'],
        assets: [],
        main_scene: 'main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('D:/test-project')
      expect(analysis.project_name).toBe('TestProject')
      expect(analysis.godot_version).toBe('4.2.1')
      expect(analysis.scripts).toContain('scripts/Player.gd')
    })

    it('should analyze project with multiple scripts', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'MultiScriptProject',
        godot_version: '4.2',
        scripts: ['Player.gd', 'Enemy.gd', 'UI.gd', 'GameManager.gd'],
        scenes: ['Main.tscn'],
        assets: [],
        main_scene: 'Main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('/multi-script')
      expect(analysis.scripts).toHaveLength(4)
    })

    it('should analyze project with multiple scenes', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'MultiSceneProject',
        godot_version: '4.2',
        scripts: [],
        scenes: ['Main.tscn', 'Level1.tscn', 'Level2.tscn', 'UI.tscn'],
        assets: [],
        main_scene: 'Main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('/multi-scene')
      expect(analysis.scenes).toHaveLength(4)
    })

    it('should detect main scene', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'TestProject',
        godot_version: '4.2',
        scripts: [],
        scenes: ['Main.tscn', 'Level.tscn'],
        assets: [],
        main_scene: 'Main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('/test')
      expect(analysis.main_scene).toBe('Main.tscn')
    })

    it('should analyze project with assets', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'AssetProject',
        godot_version: '4.2',
        scripts: [],
        scenes: [],
        assets: ['assets/sprites/', 'assets/audio/', 'assets/fonts/'],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/asset-project')
      expect(analysis.assets).toHaveLength(3)
    })

    it('should handle corrupted project.godot', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed to parse project.godot'))

      await expect(GodotOperatorApi.analyzeProject('/corrupted'))
        .rejects.toThrow('parse')
    })

    it('should handle missing project.godot', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('project.godot not found'))

      await expect(GodotOperatorApi.analyzeProject('/missing-project-file'))
        .rejects.toThrow('not found')
    })
  })

  describe('Godot Version Detection', () => {
    it('should detect Godot 4.3', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.3',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/godot43')
      expect(analysis.godot_version).toBe('4.3')
    })

    it('should detect Godot 4.2', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.2.2',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/godot42')
      expect(analysis.godot_version).toContain('4.2')
    })

    it('should detect Godot 3.5', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '3.5.3',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/godot35')
      expect(analysis.godot_version).toContain('3.5')
    })

    it('should handle unknown version', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: 'unknown',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/unknown-version')
      expect(analysis.godot_version).toBe('unknown')
    })
  })

  describe('Script Analysis', () => {
    it('should identify GDScript files', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.2',
        scripts: ['Player.gd', 'Enemy.gd', 'Utils.gd'],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/gdscript')
      const gdScripts = (analysis.scripts as string[]).filter((s: string) => s.endsWith('.gd'))
      expect(gdScripts).toHaveLength(3)
    })

    it('should handle nested script directories', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.2',
        scripts: ['scripts/Player.gd', 'scripts/enemies/Enemy.gd', 'scripts/utils/Utils.gd'],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/nested')
      expect(analysis.scripts).toHaveLength(3)
    })

    it('should identify C# scripts (Godot 4)', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'CSharpProject',
        godot_version: '4.2',
        scripts: ['Player.cs', 'Enemy.cs'],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/csharp')
      const csScripts = (analysis.scripts as string[]).filter((s: string) => s.endsWith('.cs'))
      expect(csScripts).toHaveLength(2)
    })
  })

  describe('Scene Analysis', () => {
    it('should identify scene files', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.2',
        scripts: [],
        scenes: ['Main.tscn', 'Level.tscn', 'Player.tscn'],
        assets: [],
        main_scene: 'Main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('/scenes')
      expect(analysis.scenes).toHaveLength(3)
    })

    it('should handle nested scene directories', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Test',
        godot_version: '4.2',
        scripts: [],
        scenes: ['scenes/Main.tscn', 'scenes/levels/Level1.tscn', 'scenes/ui/UI.tscn'],
        assets: [],
        main_scene: 'scenes/Main.tscn'
      })

      const analysis = await GodotOperatorApi.analyzeProject('/nested-scenes')
      expect(analysis.scenes).toHaveLength(3)
    })
  })

  describe('Error Handling', () => {
    it('should handle permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))

      await expect(GodotOperatorApi.detectProject('/protected'))
        .rejects.toThrow('Permission')
    })

    it('should handle path too long', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Path too long'))

      await expect(GodotOperatorApi.detectProject('/very/long/path'))
        .rejects.toThrow('too long')
    })

    it('should handle unreadable directory', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot read directory'))

      await expect(GodotOperatorApi.analyzeProject('/unreadable'))
        .rejects.toThrow('read')
    })
  })

  describe('Performance', () => {
    it('should detect project quickly', async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const start = performance.now()
      await GodotOperatorApi.detectProject('/test')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(50)
    })

    it('should analyze small project quickly', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Small',
        godot_version: '4.2',
        scripts: ['Player.gd'],
        scenes: ['Main.tscn'],
        assets: [],
        main_scene: 'Main.tscn'
      })

      const start = performance.now()
      await GodotOperatorApi.analyzeProject('/small')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })

    it('should analyze large project within timeout', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Large',
        godot_version: '4.2',
        scripts: Array.from({ length: 100 }, (_, i) => `script_${i}.gd`),
        scenes: Array.from({ length: 50 }, (_, i) => `scene_${i}.tscn`),
        assets: [],
        main_scene: 'Main.tscn'
      })

      const start = performance.now()
      await GodotOperatorApi.analyzeProject('/large')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(200)
    })
  })

  describe('Edge Cases', () => {
    it('should handle empty project', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'Empty',
        godot_version: '4.2',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/empty')
      expect(analysis.scripts).toHaveLength(0)
      expect(analysis.scenes).toHaveLength(0)
    })

    it('should handle project with no main scene', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'NoMainScene',
        godot_version: '4.2',
        scripts: ['Player.gd'],
        scenes: ['Level.tscn'],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/no-main')
      expect(analysis.main_scene).toBe('')
    })

    it('should handle Windows paths', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'WindowsProject',
        godot_version: '4.2',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('D:\\Games\\MyProject')
      expect(analysis.project_name).toBe('WindowsProject')
    })

    it('should handle Unix paths', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'UnixProject',
        godot_version: '4.2',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const analysis = await GodotOperatorApi.analyzeProject('/home/user/games/project')
      expect(analysis.project_name).toBe('UnixProject')
    })
  })
})