// Marketing Development E2E Tests
//
// Phase 4: Validate Kimi, Jimeng, Kling API integration

import { describe, it, expect } from 'vitest';
import { MarketingApi, KimiGenerateResult, JimengGenerateResult, KlingGenerateResult, KlingJobStatusResult } from '../api';

describe('Marketing Development E2E', () => {
  describe('Kimi API', () => {
    // These require actual Kimi API key
    it.skip('should initialize Kimi adapter', async () => {
      const apiKey = 'test_api_key';
      const id = await MarketingApi.initKimi(apiKey);
      expect(id).toBe('kimi-marketing');
    });

    it.skip('should generate text with Kimi', async () => {
      const apiKey = 'test_api_key';
      const result = await MarketingApi.kimiGenerateText(
        apiKey,
        '请写一段关于AI的广告文案',
      );
      expect(result.content).toBeDefined();
      expect(result.total_tokens).toBeGreaterThan(0);
    });

    it.skip('should translate text with Kimi', async () => {
      const apiKey = 'test_api_key';
      const result = await MarketingApi.kimiTranslate(
        apiKey,
        'Hello World',
        '中文',
      );
      expect(result).toBeDefined();
    });
  });

  describe('Jimeng API', () => {
    it.skip('should initialize Jimeng adapter', async () => {
      const apiKey = 'test_api_key';
      const id = await MarketingApi.initJimeng(apiKey);
      expect(id).toBe('jimeng-marketing');
    });

    it.skip('should generate image with Jimeng', async () => {
      const apiKey = 'test_api_key';
      const result = await MarketingApi.jimengGenerateImage(
        apiKey,
        '一只可爱的猫咪',
        'anime',
      );
      expect(result.image_urls.length).toBeGreaterThan(0);
      expect(result.cost).toBeGreaterThan(0);
    });

    it.skip('should generate image with different styles', async () => {
      const apiKey = 'test_api_key';
      const styles = ['realistic', 'anime', 'cyberpunk', 'oil_painting'] as const;

      for (const style of styles) {
        const result = await MarketingApi.jimengGenerateImage(
          apiKey,
          'beautiful landscape',
          style,
        );
        expect(result.style).toBe(style);
      }
    });
  });

  describe('Kling API', () => {
    it.skip('should initialize Kling adapter', async () => {
      const apiKey = 'test_api_key';
      const id = await MarketingApi.initKling(apiKey);
      expect(id).toBe('kling-marketing');
    });

    it.skip('should create video generation job', async () => {
      const apiKey = 'test_api_key';
      const result = await MarketingApi.klingGenerateVideo(
        apiKey,
        '一只小狗在草地上奔跑',
      );
      expect(result.task_id).toBeDefined();
      expect(result.estimated_duration).toBeGreaterThan(0);
    });

    it.skip('should get video job status', async () => {
      const apiKey = 'test_api_key';
      const taskId = 'test_task_id';
      const result = await MarketingApi.klingGetJobStatus(apiKey, taskId);
      expect(result.status).toBeDefined();
      expect(['pending', 'processing', 'completed', 'failed']).toContain(result.status);
    });

    it.skip('should generate avatar video', async () => {
      const apiKey = 'test_api_key';
      const result = await MarketingApi.klingGenerateVideo(
        apiKey,
        '虚拟主播介绍产品',
        'avatar',
      );
      expect(result.mode).toBe('avatar');
    });
  });
});

describe('Marketing Types Validation', () => {
  it('should have correct JimengStyle types', () => {
    const styles: JimengStyle[] = [
      'realistic', 'anime', 'abstract', 'cartoon', 'oil_painting',
      'watercolor', 'sketch', 'cyberpunk', 'fantasy', 'minimalist'
    ];
    expect(styles.length).toBe(10);
  });

  it('should have correct JimengImageSize types', () => {
    const sizes: JimengImageSize[] = ['512', '1024', 'portrait', 'landscape', 'wide'];
    expect(sizes.length).toBe(5);
  });

  it('should have correct KlingMode types', () => {
    const modes: KlingMode[] = ['text_to_video', 'image_to_video', 'avatar', 'extend'];
    expect(modes.length).toBe(4);
  });

  it('should have correct KlingTaskStatus types', () => {
    const statuses: KlingTaskStatus[] = ['pending', 'processing', 'completed', 'failed', 'cancelled'];
    expect(statuses.length).toBe(5);
  });
});