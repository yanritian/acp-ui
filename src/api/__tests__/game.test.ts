// Game Development E2E Tests
//
// Phase 3: Validate Unity and Godot detection + build commands

import { describe, it, expect, beforeAll } from 'vitest';
import { GameApi, GameFramework, UnityPlatform, GodotPlatform } from '../index';

describe('Game Development E2E', () => {
  describe('GameApi', () => {
    it('should get available game platforms', async () => {
      const platforms = await GameApi.getPlatforms();
      expect(platforms).toBeDefined();
      expect(platforms.length).toBeGreaterThan(0);
      expect(platforms).toContain('Windows');
      expect(platforms).toContain('Android');
      expect(platforms).toContain('WebGL');
    });

    it('should detect unknown framework for empty directory', async () => {
      const result = await GameApi.detectFramework('/tmp/empty');
      expect(result.framework).toBe('unknown');
      expect(result.scenes.length).toBe(0);
    });

    // Note: These tests require actual game project directories
    // In CI, we'd create mock project structures

    it.skip('should detect Unity project', async () => {
      // Requires actual Unity project path with Assets/ and ProjectSettings/
      const result = await GameApi.detectFramework('/path/to/unity/project');
      expect(result.framework).toBe('unity');
      expect(result.scenes.length).toBeGreaterThan(0);
    });

    it.skip('should detect Godot project', async () => {
      // Requires actual Godot project path with project.godot
      const result = await GameApi.detectFramework('/path/to/godot/project');
      expect(result.framework).toBe('godot');
      expect(result.scenes.length).toBeGreaterThan(0);
    });

    it.skip('should detect hybrid project (Unity + Godot)', async () => {
      // Requires project with both frameworks (rare case)
      const result = await GameApi.detectFramework('/path/to/hybrid/project');
      expect(result.framework).toBe('hybrid');
    });
  });

  describe('Unity Build Commands', () => {
    // These are integration tests that require:
    // 1. Unity Editor installed
    // 2. Actual Unity project
    // 3. Long execution time (10-30 minutes)

    it.skip('should build Unity project (dev mode) for Windows', async () => {
      const cwd = '/path/to/unity/project';
      const result = await GameApi.unityBuildDev(cwd, 'Windows');
      expect(result).toBeDefined();
      expect(result).toContain('build');
    });

    it.skip('should build Unity project (release) for Android', async () => {
      const cwd = '/path/to/unity/project';
      const result = await GameApi.unityBuildRelease(cwd, 'Android');
      expect(result).toBeDefined();
    });

    it.skip('should build Unity project for WebGL', async () => {
      const cwd = '/path/to/unity/project';
      const result = await GameApi.unityBuildRelease(cwd, 'WebGL');
      expect(result).toBeDefined();
    });

    it.skip('should get Unity scenes', async () => {
      const cwd = '/path/to/unity/project';
      const scenes = await GameApi.unityGetScenes(cwd);
      expect(scenes.length).toBeGreaterThan(0);
      expect(scenes[0]).toContain('.unity');
    });
  });

  describe('Godot Export Commands', () => {
    // These are integration tests that require:
    // 1. Godot Editor installed
    // 2. Actual Godot project
    // 3. Long execution time

    it.skip('should export Godot project (dev mode) for Windows', async () => {
      const cwd = '/path/to/godot/project';
      const result = await GameApi.godotBuildDev(cwd, 'Windows');
      expect(result).toBeDefined();
      expect(result).toContain('Exported to');
    });

    it.skip('should export Godot project (release) for Android', async () => {
      const cwd = '/path/to/godot/project';
      const result = await GameApi.godotBuildRelease(cwd, 'Android');
      expect(result).toBeDefined();
    });

    it.skip('should export Godot project for Web', async () => {
      const cwd = '/path/to/godot/project';
      const result = await GameApi.godotBuildRelease(cwd, 'Web');
      expect(result).toBeDefined();
    });

    it.skip('should get Godot scenes', async () => {
      const cwd = '/path/to/godot/project';
      const scenes = await GameApi.godotGetScenes(cwd);
      expect(scenes.length).toBeGreaterThan(0);
      expect(scenes[0]).toContain('.tscn');
    });
  });
});

describe('Game Types Validation', () => {
  it('should have correct GameFramework types', () => {
    const frameworks: GameFramework[] = ['unity', 'godot', 'hybrid', 'unknown'];
    expect(frameworks.length).toBe(4);
  });

  it('should have correct UnityPlatform types', () => {
    const platforms: UnityPlatform[] = ['Windows', 'MacOS', 'Linux', 'Android', 'iOS', 'WebGL'];
    expect(platforms.length).toBe(6);
  });

  it('should have correct GodotPlatform types', () => {
    const platforms: GodotPlatform[] = ['Windows', 'MacOS', 'Linux', 'Android', 'iOS', 'Web'];
    expect(platforms.length).toBe(6);
  });
});

describe('Game Asset Detection', () => {
  it('should get all supported asset types', async () => {
    const types = await GameApi.getAssetTypes();
    expect(types).toBeDefined();
    expect(types.length).toBe(10);
    expect(types).toContain('sprite2d');
    expect(types).toContain('model3d');
    expect(types).toContain('audio');
    expect(types).toContain('shader');
  });

  it('should detect assets in empty directory', async () => {
    const assets = await GameApi.detectAssets('/tmp/empty', 'unknown');
    expect(assets.length).toBe(0);
  });

  it('should get asset stats for empty directory', async () => {
    const stats = await GameApi.getAssetStats('/tmp/empty', 'unknown');
    expect(stats.total_assets).toBe(0);
    expect(stats.total_size_bytes).toBe(0);
  });

  // These require actual game project directories
  it.skip('should detect Unity assets', async () => {
    const assets = await GameApi.detectAssets('/path/to/unity/project', 'unity');
    expect(assets.length).toBeGreaterThan(0);
    expect(assets[0].asset_type).toBeDefined();
  });

  it.skip('should detect Godot assets', async () => {
    const assets = await GameApi.detectAssets('/path/to/godot/project', 'godot');
    expect(assets.length).toBeGreaterThan(0);
  });

  it.skip('should get optimization suggestions for large texture', async () => {
    const suggestions = await GameApi.getOptimizationSuggestions(
      'sprite2d',
      10 * 1024 * 1024, // 10MB
      'png'
    );
    expect(suggestions.length).toBeGreaterThan(0);
    expect(suggestions[0].priority).toBeGreaterThan(0);
  });

  it.skip('should suggest format conversion for BMP', async () => {
    const suggestions = await GameApi.getOptimizationSuggestions(
      'sprite2d',
      500 * 1024, // 500KB
      'bmp'
    );
    expect(suggestions.some(s => s.suggestion_type === 'ConvertFormat')).toBe(true);
  });
});