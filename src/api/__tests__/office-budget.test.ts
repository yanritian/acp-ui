// Office & Budget E2E Tests
//
// Phase 4 Week 3-4: Validate WPS Office + Cost Tracker

import { describe, it, expect } from 'vitest';
import { OfficeApi, BudgetApi, WPSGenerateResult, ExcelAnalysisResult, CostUsage, CostForecast, DocumentFormat, WpsDocumentType, TokenTotals } from '../index';

describe('Office Development E2E', () => {
  describe('WPS API', () => {
    it.skip('should initialize WPS adapter', async () => {
      const apiKey = 'test_api_key';
      const id = await OfficeApi.initWps(apiKey);
      expect(id).toBe('wps-office');
    });

    it.skip('should generate document with WPS', async () => {
      const apiKey = 'test_api_key';
      const result = await OfficeApi.generateDoc(
        apiKey,
        '请生成一份项目报告',
        'report',
      );
      expect(result.document_url).toBeDefined();
      expect(result.document_type).toBe('report');
      expect(result.cost).toBeGreaterThan(0);
    });

    it.skip('should generate different document types', async () => {
      const apiKey = 'test_api_key';
      const types = ['report', 'contract', 'proposal', 'summary'] as const;

      for (const type of types) {
        const result = await OfficeApi.generateDoc(
          apiKey,
          `Generate ${type}`,
          type,
        );
        expect(result.document_type).toBe(type);
      }
    });

    it.skip('should analyze Excel file', async () => {
      const apiKey = 'test_api_key';
      const fileUrl = 'https://example.com/data.xlsx';
      const result = await OfficeApi.analyzeExcel(apiKey, fileUrl);
      expect(result.summary).toBeDefined();
      expect(result.insights.length).toBeGreaterThan(0);
    });
  });

  describe('Document Format Types', () => {
    it('should have correct DocumentFormat types', () => {
      const formats: DocumentFormat[] = ['docx', 'pdf', 'pptx', 'xlsx', 'txt'];
      expect(formats.length).toBe(5);
    });

    it('should have correct WpsDocumentType types', () => {
      const types: WpsDocumentType[] = ['report', 'contract', 'proposal', 'resume', 'summary', 'plan', 'article'];
      expect(types.length).toBe(7);
    });
  });
});

describe('Budget Management E2E', () => {
  describe('Budget API', () => {
    it('should set budget limits', async () => {
      await BudgetApi.setLimit(10.0, 100.0);
      // No error means success
    });

    it('should set budget with hard stop', async () => {
      await BudgetApi.setLimit(5.0, 50.0, true);
    });

    it('should get current usage', async () => {
      const usage = await BudgetApi.getUsage();
      expect(usage.daily_total).toBeDefined();
      expect(usage.monthly_total).toBeDefined();
      expect(usage.adapter_totals).toBeDefined();
      expect(usage.token_totals).toBeDefined();
    });

    it('should get cost forecast', async () => {
      const forecast = await BudgetApi.getForecast();
      expect(forecast.predicted_daily).toBeDefined();
      expect(forecast.predicted_monthly).toBeDefined();
      expect(forecast.days_remaining).toBeGreaterThan(0);
      expect(forecast.budget_remaining_percent).toBeDefined();
      expect(typeof forecast.will_exceed).toBe('boolean');
    });

    it('should check if budget allows execution', async () => {
      const allowed = await BudgetApi.isAllowed();
      expect(typeof allowed).toBe('boolean');
    });
  });

  describe('Budget Types', () => {
    it('should have correct CostUsage structure', () => {
      const usage: CostUsage = {
        daily_total: 5.0,
        monthly_total: 50.0,
        total_all_time: 500.0,
        adapter_totals: { kimi: 20.0, wps: 30.0 },
        scene_totals: { marketing: 40.0 },
        token_totals: { input_tokens: 10000, output_tokens: 5000, total_tokens: 15000 },
      };
      expect(usage.daily_total).toBe(5.0);
      expect(usage.monthly_total).toBe(50.0);
    });

    it('should have correct CostForecast structure', () => {
      const forecast: CostForecast = {
        predicted_daily: 2.0,
        predicted_monthly: 60.0,
        days_remaining: 15,
        budget_remaining_percent: 40.0,
        will_exceed: true,
      };
      expect(forecast.days_remaining).toBe(15);
      expect(forecast.will_exceed).toBe(true);
    });

    it('should have correct TokenTotals structure', () => {
      const tokens: TokenTotals = {
        input_tokens: 1000,
        output_tokens: 500,
        total_tokens: 1500,
      };
      expect(tokens.total_tokens).toBe(1500);
    });
  });
});