// One-Shot Interface E2E Tests
//
// Phase 5: One-Shot Interface + User Role Detection

import { describe, it, expect } from 'vitest';
import { OneShotApi, OneShotResponse, UserRoleResult, AgentRecommendation, UserRoleType } from '../index';

describe('One-Shot Interface E2E', () => {
  describe('Role Detection', () => {
    it('should detect developer role', async () => {
      const result = await OneShotApi.detectRole('帮我写一个 Rust 函数');
      expect(result.role).toBe('developer');
      expect(result.confidence).toBeGreaterThan(0);
    });

    it('should detect marketer role', async () => {
      const result = await OneShotApi.detectRole('写一段抖音文案');
      expect(result.role).toBe('marketer');
    });

    it('should detect gamer role', async () => {
      const result = await OneShotApi.detectRole('帮我构建 Unity 游戏');
      expect(result.role).toBe('gamer');
    });

    it('should detect finance role', async () => {
      const result = await OneShotApi.detectRole('分析 Excel 财务数据');
      expect(result.role).toBe('finance');
    });

    it('should detect designer role', async () => {
      const result = await OneShotApi.detectRole('设计一个 UI 界面');
      expect(result.role).toBe('designer');
    });

    it('should detect general role for ambiguous input', async () => {
      const result = await OneShotApi.detectRole('随便聊聊');
      expect(result.role).toBe('general');
    });
  });

  describe('Scene Detection', () => {
    it('should detect image generation scene', async () => {
      const scene = await OneShotApi.detectScene('生成一张产品图片');
      expect(scene).toBe('image_generation');
    });

    it('should detect video generation scene', async () => {
      const scene = await OneShotApi.detectScene('制作一个宣传视频');
      expect(scene).toBe('video_generation');
    });

    it('should detect copywriting scene', async () => {
      const scene = await OneShotApi.detectScene('写一段广告文案');
      expect(scene).toBe('copywriting');
    });

    it('should detect translation scene', async () => {
      const scene = await OneShotApi.detectScene('翻译这段文字到英文');
      expect(scene).toBe('translation');
    });

    it('should detect code generation scene', async () => {
      const scene = await OneShotApi.detectScene('生成一个 API 函数');
      expect(scene).toBe('code_generation');
    });

    it('should detect bug fix scene', async () => {
      const scene = await OneShotApi.detectScene('修复这个 bug');
      expect(scene).toBe('bug_fix');
    });
  });

  describe('Agent Recommendation', () => {
    it('should recommend kimi for marketer copywriting', async () => {
      const rec = await OneShotApi.getRecommendation('marketer', 'copywriting');
      expect(rec.agent).toBe('kimi-marketing');
      expect(rec.reason).toContain('scene');
    });

    it('should recommend jimeng for image generation', async () => {
      const rec = await OneShotApi.getRecommendation('marketer', 'image_generation');
      expect(rec.agent).toBe('jimeng-marketing');
    });

    it('should recommend kling for video generation', async () => {
      const rec = await OneShotApi.getRecommendation('marketer', 'video_generation');
      expect(rec.agent).toBe('kling-marketing');
    });

    it('should recommend claude-code for developer', async () => {
      const rec = await OneShotApi.getRecommendation('developer', 'code_generation');
      expect(rec.agent).toBe('claude-code');
    });

    it('should recommend unity for gamer', async () => {
      const rec = await OneShotApi.getRecommendation('gamer', 'game_build');
      expect(rec.agent).toBe('unity-game');
    });

    it('should recommend wps for finance', async () => {
      const rec = await OneShotApi.getRecommendation('finance', 'document_generation');
      expect(rec.agent).toBe('wps-office');
    });
  });

  describe('One-Shot Execute', () => {
    it('should process marketing request', async () => {
      const response = await OneShotApi.execute('写一段抖音文案');
      expect(response.detected_role).toBe('marketer');
      expect(response.detected_scene).toBe('copywriting');
      expect(response.selected_agent).toBe('kimi-marketing');
      expect(response.selection_reason).toBeDefined();
      expect(response.transparency).toBeDefined();
    });

    it('should process developer request', async () => {
      const response = await OneShotApi.execute('帮我写一个 Rust 函数');
      expect(response.detected_role).toBe('developer');
      expect(response.selected_agent).toContain('claude');
    });

    it('should process game request', async () => {
      const response = await OneShotApi.execute('构建 Unity 游戏');
      expect(response.detected_role).toBe('gamer');
      expect(response.selected_agent).toBe('unity-game');
    });

    it('should respect preferred agent', async () => {
      const response = await OneShotApi.execute('写代码', {
        preferred_agent: 'codex',
      });
      expect(response.selected_agent).toBe('codex');
      expect(response.selection_reason).toContain('preferred');
    });

    it('should include transparency info', async () => {
      const response = await OneShotApi.execute('测试请求');
      expect(response.transparency.dag_visualization).toBeDefined();
      expect(response.transparency.estimated_cost).toBeDefined();
      expect(response.transparency.privacy_level).toBeDefined();
      expect(response.transparency.budget_status).toBeDefined();
    });
  });

  describe('Types Validation', () => {
    it('should have correct UserRoleType values', () => {
      const roles: UserRoleType[] = [
        'developer', 'marketer', 'designer', 'finance',
        'gamer', 'writer', 'analyst', 'general'
      ];
      expect(roles.length).toBe(8);
    });
  });
});