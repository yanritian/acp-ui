import { test, expect } from '@playwright/test';

test.describe('Gateway Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);
  });

  test('should show Gateway/Remote Control in advanced navigation', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);

    const gatewayNav = page.locator('button').filter({ hasText: /Remote Control|远程控制/ });
    await expect(gatewayNav).toBeVisible();
  });

  test('should navigate to Gateway view', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Verify Gateway settings page header
    const gatewayHeader = page.locator('h1').filter({ hasText: /Remote Control|远程控制/ });
    await expect(gatewayHeader).toBeVisible();
  });

  test('should display all Bot platform cards', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Check for Feishu Bot card
    const feishuCard = page.locator('h2').filter({ hasText: /Feishu|飞书/ });
    await expect(feishuCard).toBeVisible();

    // Check for Telegram Bot card
    const telegramCard = page.locator('h2').filter({ hasText: 'Telegram' });
    await expect(telegramCard).toBeVisible();

    // Check for Discord Bot card
    const discordCard = page.locator('h2').filter({ hasText: 'Discord' });
    await expect(discordCard).toBeVisible();
  });

  test('should display Bot enable toggles', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Check for enable text in each Bot section
    const feishuEnable = page.locator('text=Enable Feishu Bot');
    await expect(feishuEnable).toBeVisible();

    const telegramEnable = page.locator('text=Enable Telegram Bot');
    await expect(telegramEnable).toBeVisible();

    const discordEnable = page.locator('text=Enable Discord Bot');
    await expect(discordEnable).toBeVisible();
  });

  test('should display Gateway service status', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Check for status badge (Stopped/Running)
    const statusBadge = page.locator('text=Stopped');
    await expect(statusBadge).toBeVisible();
  });

  test('should have Start/Stop service buttons', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Check for Start Service button
    const startBtn = page.locator('button').filter({ hasText: 'Start Service' });
    await expect(startBtn).toBeVisible();

    // Check for Stop Service button
    const stopBtn = page.locator('button').filter({ hasText: 'Stop Service' });
    await expect(stopBtn).toBeVisible();
  });

  test('should save Gateway configuration', async ({ page }) => {
    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    const saveButton = page.locator('button').filter({ hasText: /Save Configuration|保存配置/ });
    await expect(saveButton).toBeVisible();
  });
});

test.describe('Bot Adapter Integration', () => {
  test('should show configuration steps for each platform', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const moreBtn = page.locator('button').filter({ hasText: /More Features|更多/ });
    await moreBtn.click();
    await page.waitForTimeout(500);
    await page.locator('button').filter({ hasText: /Remote Control|远程控制/ }).click();
    await page.waitForTimeout(1000);

    // Check Feishu configuration steps
    const feishuSteps = page.locator('h4').filter({ hasText: 'Configuration Steps' }).first();
    await expect(feishuSteps).toBeVisible();

    // Check Telegram configuration steps
    const telegramSteps = page.locator('h4').filter({ hasText: 'Configuration Steps' });
    await expect(telegramSteps.first()).toBeVisible();
  });
});