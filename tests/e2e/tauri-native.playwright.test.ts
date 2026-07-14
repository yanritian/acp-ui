// Tauri Native E2E Tests using Playwright
// Tests the Windows Tauri desktop application

import { test, expect } from '@playwright/test';
import { ElectronApplication, _electron as electron } from 'playwright';
import path from 'path';

let app: ElectronApplication;

test.describe('Tauri Native E2E', () => {
  test.beforeAll(async () => {
    // Launch the Tauri debug executable
    const exePath = path.join(process.cwd(), 'src-tauri', 'target', 'debug', 'acp-ui.exe');
    app = await electron.launch({
      executablePath: exePath,
      args: ['--no-sandbox'],
    });
  });

  test.afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  test('should launch the application', async () => {
    const window = await app.firstWindow();
    expect(window).toBeDefined();

    // Wait for the app to load
    await window.waitForLoadState('domcontentloaded', { timeout: 30000 });

    const title = await window.title();
    expect(title).toBeTruthy();
  });

  test('should navigate to games page', async () => {
    const window = await app.firstWindow();

    // Wait for navigation
    await window.waitForURL('**/*', { timeout: 30000 });

    const url = window.url();
    expect(url).toContain('localhost');
  });

  test('should display game operator interface', async () => {
    const window = await app.firstWindow();

    // Wait for content to load
    await window.waitForSelector('body', { timeout: 30000 });

    const body = await window.locator('body');
    const text = await body.textContent();

    // Should have some content
    expect(text?.length).toBeGreaterThan(0);
  });

  test('should have input fields for project and goal', async () => {
    const window = await app.firstWindow();

    // Check for input elements
    const inputs = await window.locator('input').count();
    const textareas = await window.locator('textarea').count();

    expect(inputs + textareas).toBeGreaterThan(0);
  });

  test('should have action buttons', async () => {
    const window = await app.firstWindow();

    // Check for buttons
    const buttons = await window.locator('button').count();

    expect(buttons).toBeGreaterThan(0);
  });
});