// Hermes CLI Integration Tests for Hermes Game Operator
// Tests Hermes CLI connection and plan generation

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

import { HermesCliApi } from '@/api/operatorApi'

describe('Hermes CLI Integration Tests', () => {
  describe('Connection Check', () => {
    it('should detect Hermes CLI available', async () => {
      mockInvoke.mockResolvedValueOnce({
        available: true,
        version: '1.0.0',
        path: '/usr/local/bin/hermes'
      })

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(true)
      expect(status.version).toBe('1.0.0')
    })

    it('should detect Hermes CLI not available', async () => {
      mockInvoke.mockResolvedValueOnce({
        available: false,
        error: 'Hermes CLI not found in PATH'
      })

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(false)
      expect(status.error).toContain('not found')
    })

    it('should handle connection check timeout', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection timeout'))

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(false)
      expect(status.error).toContain('timeout')
    })

    it('should handle Hermes CLI version check', async () => {
      mockInvoke.mockResolvedValueOnce({
        available: true,
        version: '2.1.3',
        path: 'C:/Program Files/hermes/bin/hermes.exe'
      })

      const status = await HermesCliApi.checkConnection()
      expect(status.version).toBe('2.1.3')
      expect(status.path).toContain('hermes.exe')
    })
  })

  describe('Project Analysis', () => {
    it('should analyze project successfully', async () => {
      mockInvoke.mockResolvedValueOnce({
        projectName: 'MyGodotGame',
        godotVersion: '4.2.1',
        scripts: ['scripts/Player.gd', 'scripts/Enemy.gd'],
        scenes: ['scenes/Main.tscn', 'scenes/Level1.tscn'],
        playerControllers: [],
        analysisTimeMs: 150
      })

      const analysis = await HermesCliApi.analyzeProject('D:/games/my-game')
      expect(analysis.projectName).toBe('MyGodotGame')
      expect(analysis.scripts).toHaveLength(2)
      expect(analysis.scenes).toHaveLength(2)
    })

    it('should handle invalid project path', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Project path does not exist'))

      await expect(HermesCliApi.analyzeProject('/invalid/path'))
        .rejects.toThrow('does not exist')
    })

    it('should handle non-Godot project', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Not a Godot project'))

      await expect(HermesCliApi.analyzeProject('/non-godot-project'))
        .rejects.toThrow('Not a Godot project')
    })

    it('should analyze large project', async () => {
      mockInvoke.mockResolvedValueOnce({
        projectName: 'LargeGame',
        godotVersion: '4.3',
        scripts: Array.from({ length: 50 }, (_, i) => `scripts/script_${i}.gd`),
        scenes: Array.from({ length: 20 }, (_, i) => `scenes/scene_${i}.tscn`),
        playerControllers: [],
        analysisTimeMs: 200
      })

      const analysis = await HermesCliApi.analyzeProject('/large-project')
      expect(analysis.scripts).toHaveLength(50)
      expect(analysis.scenes).toHaveLength(20)
    })
  })

  describe('Plan Generation', () => {
    it('should generate simple plan', async () => {
      mockInvoke.mockResolvedValueOnce({
        taskId: 'task_plan_1',
        goal: 'Add player movement',
        steps: [
          { id: 1, description: 'Analyze Player.gd', files: ['scripts/Player.gd'], estimatedTimeSeconds: 30, requiresApproval: false },
          { id: 2, description: 'Add movement logic', files: ['scripts/Player.gd'], estimatedTimeSeconds: 60, requiresApproval: true }
        ],
        totalEstimatedTimeSeconds: 90
      })

      const plan = await HermesCliApi.generatePlan('task_plan_1', 'Add player movement')
      expect(plan.steps).toHaveLength(2)
      expect(plan.totalEstimatedTimeSeconds).toBe(90)
    })

    it('should generate complex multi-file plan', async () => {
      mockInvoke.mockResolvedValueOnce({
        taskId: 'task_complex',
        goal: 'Implement combat system',
        steps: [
          { id: 1, description: 'Create combat manager', files: ['scripts/CombatManager.gd'], estimatedTimeSeconds: 120, requiresApproval: false },
          { id: 2, description: 'Update player for combat', files: ['scripts/Player.gd'], estimatedTimeSeconds: 60, requiresApproval: true },
          { id: 3, description: 'Update enemy for combat', files: ['scripts/Enemy.gd'], estimatedTimeSeconds: 60, requiresApproval: true },
          { id: 4, description: 'Add UI for health', files: ['scripts/UI.gd', 'scenes/UI.tscn'], estimatedTimeSeconds: 90, requiresApproval: true },
          { id: 5, description: 'Test combat system', files: ['tests/CombatTest.gd'], estimatedTimeSeconds: 30, requiresApproval: false }
        ],
        totalEstimatedTimeSeconds: 360
      })

      const plan = await HermesCliApi.generatePlan('task_complex', 'Implement combat system')
      expect(plan.steps).toHaveLength(5)
      expect(plan.totalEstimatedTimeSeconds).toBe(360)
    })

    it('should handle plan generation failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Plan generation failed: unclear goal'))

      await expect(HermesCliApi.generatePlan('task_fail', 'unclear goal'))
        .rejects.toThrow('Plan generation failed')
    })

    it('should generate plan with approval requirements', async () => {
      mockInvoke.mockResolvedValueOnce({
        taskId: 'task_approval',
        goal: 'Delete unused files',
        steps: [
          { id: 1, description: 'Identify unused files', files: [], estimatedTimeSeconds: 30, requiresApproval: false },
          { id: 2, description: 'Delete OldScript.gd', files: ['scripts/OldScript.gd'], estimatedTimeSeconds: 10, requiresApproval: true },
          { id: 3, description: 'Delete OldScene.tscn', files: ['scenes/OldScene.tscn'], estimatedTimeSeconds: 10, requiresApproval: true }
        ],
        totalEstimatedTimeSeconds: 50
      })

      const plan = await HermesCliApi.generatePlan('task_approval', 'Delete unused files')
      const approvalSteps = plan.steps.filter(s => s.requiresApproval)
      expect(approvalSteps).toHaveLength(2)
    })
  })

  describe('Error Handling', () => {
    it('should handle Hermes CLI crash', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Hermes CLI crashed'))

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(false)
      expect(status.error).toContain('crashed')
    })

    it('should handle API rate limiting', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Rate limit exceeded'))

      await expect(HermesCliApi.generatePlan('task_rate', 'test'))
        .rejects.toThrow('Rate limit')
    })

    it('should handle invalid Hermes response', async () => {
      mockInvoke.mockResolvedValueOnce({ available: false, error: 'Invalid response format' })

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(false)
      expect(status.error).toBeDefined()
    })

    it('should handle network error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const status = await HermesCliApi.checkConnection()
      expect(status.available).toBe(false)
    })
  })

  describe('Performance', () => {
    it('should check connection quickly', async () => {
      mockInvoke.mockResolvedValueOnce({ available: true, version: '1.0.0' })

      const start = performance.now()
      await HermesCliApi.checkConnection()
      const duration = performance.now() - start

      expect(duration).toBeLessThan(50)
    })

    it('should analyze project within timeout', async () => {
      mockInvoke.mockResolvedValueOnce({
        project_name: 'TestProject',
        godot_version: '4.2',
        scripts: [],
        scenes: [],
        assets: [],
        main_scene: ''
      })

      const start = performance.now()
      await HermesCliApi.analyzeProject('/test')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(100)
    })

    it('should generate plan efficiently', async () => {
      mockInvoke.mockResolvedValueOnce({
        taskId: 'task_perf',
        goal: 'test',
        steps: [],
        totalEstimatedTimeSeconds: 0
      })

      const start = performance.now()
      await HermesCliApi.generatePlan('task_perf', 'test')
      const duration = performance.now() - start

      expect(duration).toBeLessThan(50)
    })
  })
})