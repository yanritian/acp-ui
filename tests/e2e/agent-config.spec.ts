import { test, expect } from '@playwright/test';

test.describe('Agent Config View', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 1200 });
    await page.goto('http://localhost:1420');
    await page.waitForLoadState('networkidle');
  });

  test('should show Agent Config in navigation', async ({ page }) => {
    const agentConfigNav = page.locator('button:has-text("Agent Config")');
    await expect(agentConfigNav).toBeVisible();
  });

  test('should navigate to Agent Config view', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);

    const header = page.locator('h2:has-text("Agent Config")');
    await expect(header).toBeVisible();
  });

  test('should show empty state when no agents configured', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);

    // Use more specific selector to avoid matching the agent selector dropdown
    const emptyState = page.locator('.empty-state h3:has-text("No Agents Configured")');
    await expect(emptyState).toBeVisible();
  });

  test('should open add agent form', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);

    await page.click('button:has-text("Add Agent")');
    await page.waitForTimeout(300);

    const modal = page.locator('.modal-overlay');
    await expect(modal).toBeVisible();
  });

  test('should validate form fields', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Add Agent")');
    await page.waitForTimeout(300);

    const saveButton = page.locator('.modal-content button:has-text("Save")');
    await expect(saveButton).toBeDisabled();

    await page.fill('input[placeholder="e.g. My Agent"]', 'Test Agent');

    await expect(saveButton).toBeEnabled();
  });

  test('should switch between connection types', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Add Agent")');
    await page.waitForTimeout(300);

    // Target the select inside the modal, not the agent selector in sidebar
    const connectionTypeSelect = page.locator('.modal-content select');
    await expect(connectionTypeSelect).toBeVisible();

    // Default should be WebSocket
    const urlInput = page.locator('.modal-content input[placeholder="ws://localhost:8080/ws"]');
    await expect(urlInput).toBeVisible();

    // Switch to stdio type
    await connectionTypeSelect.selectOption('stdio');
    await page.waitForTimeout(200);

    // Verify command input appears
    const commandInput = page.locator('.modal-content input[placeholder="acp-server --port 8080"]');
    await expect(commandInput).toBeVisible();

    // URL input should disappear
    await expect(urlInput).not.toBeVisible();
  });

  test('should close form on cancel', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Add Agent")');
    await page.waitForTimeout(300);

    await page.click('.modal-content button:has-text("Cancel")');
    await page.waitForTimeout(300);

    const modal = page.locator('.modal-overlay');
    await expect(modal).not.toBeVisible();
  });

  test('should close form on overlay click', async ({ page }) => {
    await page.click('button:has-text("Agent Config")');
    await page.waitForTimeout(500);
    await page.click('button:has-text("Add Agent")');
    await page.waitForTimeout(300);

    await page.click('.modal-overlay', { position: { x: 10, y: 10 } });
    await page.waitForTimeout(300);

    const modal = page.locator('.modal-overlay');
    await expect(modal).not.toBeVisible();
  });
});
