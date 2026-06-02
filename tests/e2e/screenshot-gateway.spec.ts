import { test, expect } from '@playwright/test';

test('capture gateway screenshots', async ({ page }) => {
  await page.goto('/');
  await page.waitForTimeout(2000);

  // Expand advanced menu
  await page.locator('button').filter({ hasText: /More Features/ }).click();
  await page.waitForTimeout(500);

  // Navigate to Gateway
  await page.locator('button').filter({ hasText: /Remote Control/ }).click();
  await page.waitForTimeout(1500);

  // Take full page screenshot
  await page.screenshot({ path: 'docs/test-reports/screenshots/gateway-full-page.png', fullPage: true });

  // Take viewport screenshot
  await page.screenshot({ path: 'docs/test-reports/screenshots/gateway-view.png' });

  // Screenshot individual Bot cards
  const feishuCard = page.locator('h2').filter({ hasText: 'Feishu' }).locator('..').locator('..');
  await feishuCard.screenshot({ path: 'docs/test-reports/screenshots/feishu-card.png' });

  const telegramCard = page.locator('h2').filter({ hasText: 'Telegram' }).locator('..').locator('..');
  await telegramCard.screenshot({ path: 'docs/test-reports/screenshots/telegram-card.png' });

  const discordCard = page.locator('h2').filter({ hasText: 'Discord' }).locator('..').locator('..');
  await discordCard.screenshot({ path: 'docs/test-reports/screenshots/discord-card.png' });

  // Screenshot service control area
  const serviceArea = page.locator('text=Stopped').locator('..').locator('..');
  await serviceArea.screenshot({ path: 'docs/test-reports/screenshots/service-control.png' });
});