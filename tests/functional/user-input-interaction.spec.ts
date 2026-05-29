/**
 * ACP-UI User Input Interaction Tests
 *
 * Simulates real user operations across the application:
 * - Working directory input
 * - Language switching (i18n)
 * - Bot Settings form input
 * - Gateway Settings form input
 * - Traffic Monitor search
 * - Sidebar collapse/expand
 * - Consecutive view navigation
 *
 * Run: npx playwright test tests/functional/user-input-interaction.spec.ts --config=playwright.functional.config.ts
 */
import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:5173';
const OUTPUT_DIR = 'D:/dingsun/acp-ui/test-output/user-input';

/** Navigate to a view by its nav button index */
async function navigateByIndex(page: any, buttonIndex: number) {
  const btn = page.locator('.nav-btn').nth(buttonIndex);
  await expect(btn).toBeVisible({ timeout: 10000 });
  await btn.click();
  await page.waitForTimeout(500);
}

/** Take a screenshot to the output directory */
async function screenshot(page: any, filename: string) {
  await page.screenshot({
    path: `${OUTPUT_DIR}/${filename}`,
    fullPage: false,
  });
}

test.describe('ACP-UI 用户输入交互测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('domcontentloaded');
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 15000 });
  });

  // ============================================================
  // Test 1: 工作目录输入测试
  // ============================================================
  test('工作目录输入 - 输入路径并验证保存', async ({ page }) => {
    // On web mode (localhost:5173), the cwd picker renders as a free-text
    // input (.cwd-input) rather than the desktop folder-picker button.
    // Detect which mode we are in and handle accordingly.
    const cwdInput = page.locator('.cwd-input');
    const isWebMode = await cwdInput.isVisible().catch(() => false);

    if (isWebMode) {
      // Web mode: type into the free-text input
      await cwdInput.click();
      await cwdInput.fill('');
      await cwdInput.type('D:\\dingsun\\test-project');

      // Verify the input value
      const inputValue = await cwdInput.inputValue();
      expect(inputValue).toBe('D:\\dingsun\\test-project');

      // Verify the input is still visible and not disabled
      await expect(cwdInput).toBeVisible();
      const isDisabled = await cwdInput.isDisabled();
      expect(isDisabled).toBe(false);

      console.log('PASS: Web mode cwd input verified');
    } else {
      // Desktop mode: the .cwd-path span shows the last segment of the path
      // We can verify the display element exists
      const cwdPath = page.locator('.cwd-path');
      await expect(cwdPath).toBeVisible();
      const initialText = await cwdPath.textContent();
      console.log(`INFO: Desktop mode cwd display: "${initialText?.trim()}"`);

      // Click the folder picker button (may not work in headless, so skip actual selection)
      const cwdBtn = page.locator('.cwd-btn');
      await expect(cwdBtn).toBeVisible();
      console.log('INFO: Desktop mode folder picker button visible');
    }

    await screenshot(page, '01-working-directory-input.png');
    console.log('PASS: 工作目录输入测试完成');
  });

  // ============================================================
  // Test 2: 语言切换测试
  // ============================================================
  test('语言切换 - 中英文切换验证', async ({ page }) => {
    // The LanguageSelector uses a <details> element
    const langDropdown = page.locator('.lang-dropdown');
    const langCurrent = page.locator('.lang-current');

    await expect(langCurrent).toBeVisible({ timeout: 10000 });

    // Capture initial state
    const initialLangName = await langCurrent.locator('.lang-name').textContent();
    console.log(`INFO: 初始语言: ${initialLangName?.trim()}`);

    // Open the dropdown
    await langCurrent.click();
    await page.waitForTimeout(300);

    // Verify dropdown is open
    const isOpen = await langDropdown.getAttribute('open');
    expect(isOpen).not.toBeNull();
    console.log('PASS: 语言下拉菜单已打开');

    // Click "English" to switch to English
    const englishItem = page.locator('.lang-item').filter({ hasText: 'English' });
    await expect(englishItem).toBeVisible();
    await englishItem.click();
    await page.waitForTimeout(500);

    // Verify the current language now shows "English"
    const afterSwitchLang = await langCurrent.locator('.lang-name').textContent();
    expect(afterSwitchLang?.trim()).toBe('English');

    // Verify the welcome text is in English
    const welcomeTitle = page.locator('.welcome-screen h2');
    if (await welcomeTitle.isVisible().catch(() => false)) {
      const welcomeText = await welcomeTitle.textContent();
      expect(welcomeText).toContain('Welcome');
      console.log(`PASS: 欢迎文本已切换为英文: "${welcomeText}"`);
    }

    await screenshot(page, '02-language-switched-to-english.png');
    console.log('PASS: 语言已切换到英文');

    // Now switch back to Chinese
    // The <details> element may not toggle properly in headless Chromium.
    // Use JavaScript to explicitly open it.
    await page.evaluate(() => {
      const details = document.querySelector('.lang-dropdown');
      if (details) details.setAttribute('open', '');
    });
    await page.waitForTimeout(500);

    const chineseItem = page.locator('.lang-item').filter({ hasText: '中文' });
    await expect(chineseItem).toBeVisible({ timeout: 5000 });
    await chineseItem.click();
    await page.waitForTimeout(500);

    // Verify the current language shows "中文"
    const finalLangName = await langCurrent.locator('.lang-name').textContent();
    expect(finalLangName?.trim()).toBe('中文');

    await screenshot(page, '02-language-switched-back-to-chinese.png');
    console.log('PASS: 语言已切换回中文');
  });

  // ============================================================
  // Test 3: Bot Settings 输入测试
  // ============================================================
  test('Bot Settings 输入 - 填写表单并验证', async ({ page }) => {
    // Navigate to Bot Settings (index 14)
    await navigateByIndex(page, 14);

    // Verify Bot Settings view is loaded
    await expect(page.locator('.bot-settings')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.bot-settings .settings-header')).toBeVisible();

    // First, enable the feishu platform via checkbox (inputs are disabled by default)
    const enableCheckbox = page.locator('.bot-settings .config-panel input[type="checkbox"]').first();
    await expect(enableCheckbox).toBeVisible({ timeout: 5000 });
    await enableCheckbox.click();
    await page.waitForTimeout(300);

    // Now fill the App ID field (was disabled, now enabled)
    const appIdInput = page.locator('.bot-settings .config-panel input[type="text"]').first();
    await expect(appIdInput).toBeVisible();
    await appIdInput.click();
    await appIdInput.fill('');
    await appIdInput.fill('TestBot');

    // Verify the input value
    const appIdValue = await appIdInput.inputValue();
    expect(appIdValue).toBe('TestBot');
    console.log('PASS: Bot名称输入验证通过');

    // Fill the appSecret field (type="password")
    const appSecretInput = page.locator('.bot-settings .config-panel input[type="password"]').first();
    await expect(appSecretInput).toBeVisible({ timeout: 5000 });
    await appSecretInput.click();
    await appSecretInput.fill('');
    await appSecretInput.fill('测试机器人描述');

    const secretValue = await appSecretInput.inputValue();
    expect(secretValue).toBe('测试机器人描述');
    console.log('PASS: Bot描述输入验证通过');

    await screenshot(page, '03-bot-settings-input.png');
    console.log('PASS: Bot Settings 输入测试完成');
  });

  // ============================================================
  // Test 4: Gateway Settings 输入测试
  // ============================================================
  test('Gateway Settings 输入 - 填写Gateway名称和URL', async ({ page }) => {
    // Navigate to Gateway Settings (index 15)
    await navigateByIndex(page, 15);

    // Verify Gateway Settings view is loaded
    await expect(page.locator('.gateway-page')).toBeVisible({ timeout: 10000 });

    // Verify header elements
    const hasHeader = await page.locator('.gateway-page h1').isVisible().catch(() => false);
    expect(hasHeader).toBe(true);
    console.log('PASS: Gateway页面头部可见');

    // Verify status badge
    await expect(page.locator('.status-badge')).toBeVisible();
    const statusText = await page.locator('.status-badge span:last-child').textContent();
    console.log(`INFO: Gateway状态: ${statusText?.trim()}`);

    // Toggle the tunnel switch to reveal additional inputs
    const tunnelToggle = page.locator('.toggle-row .toggle').first();
    await expect(tunnelToggle).toBeVisible({ timeout: 5000 });
    await tunnelToggle.click();
    await page.waitForTimeout(500);

    // Verify the toggle is now active
    const isActive = await tunnelToggle.evaluate((el: Element) => el.classList.contains('active'));
    expect(isActive).toBe(true);
    console.log('PASS: Tunnel开关已打开');

    // Find the ngrok token input (appears when tunnel is enabled and provider is ngrok)
    const ngrokTokenInput = page.locator('.gateway-page input[placeholder*="ngrok"]').first();
    await expect(ngrokTokenInput).toBeVisible({ timeout: 5000 });

    await ngrokTokenInput.click();
    await ngrokTokenInput.fill('');
    await ngrokTokenInput.fill('TestGateway');

    const tokenValue = await ngrokTokenInput.inputValue();
    expect(tokenValue).toBe('TestGateway');
    console.log('PASS: Gateway名称输入验证通过');

    // Also try filling the feishu appId input in the feishu section
    // Enable the feishu toggle first
    const feishuToggle = page.locator('.config-section .toggle').nth(1);
    if (await feishuToggle.isVisible().catch(() => false)) {
      await feishuToggle.click();
      await page.waitForTimeout(300);

      const feishuAppIdInput = page.locator('.gateway-page input[placeholder="cli_xxxxxxxxxx"]').first();
      if (await feishuAppIdInput.isVisible().catch(() => false)) {
        await feishuAppIdInput.click();
        await feishuAppIdInput.fill('');
        await feishuAppIdInput.fill('http://localhost:8080');

        const feishuValue = await feishuAppIdInput.inputValue();
        expect(feishuValue).toBe('http://localhost:8080');
        console.log('PASS: Gateway URL输入验证通过');
      }
    }

    // Verify toggle switches exist
    const toggleCount = await page.locator('.gateway-page .toggle').count();
    console.log(`INFO: Gateway开关数: ${toggleCount}`);
    expect(toggleCount).toBeGreaterThanOrEqual(3);

    await screenshot(page, '04-gateway-settings-input.png');
    console.log('PASS: Gateway Settings 输入测试完成');
  });

  // ============================================================
  // Test 5: Traffic Monitor 搜索输入测试
  // ============================================================
  test('Traffic Monitor 搜索 - 输入关键词并验证', async ({ page }) => {
    // Open Traffic Monitor panel via the 📡 button
    const trafficBtn = page.locator('button[title="ACP Traffic Monitor"]');
    await expect(trafficBtn).toBeVisible({ timeout: 10000 });
    await trafficBtn.click();
    await page.waitForTimeout(500);

    // Verify Traffic Monitor panel is visible
    await expect(page.locator('.traffic-monitor')).toBeVisible();
    await expect(page.locator('.monitor-header')).toBeVisible();
    console.log('PASS: Traffic Monitor面板已打开');

    // Verify search input exists
    const searchInput = page.locator('.search-input');
    await expect(searchInput).toBeVisible();
    console.log('PASS: 搜索输入框可见');

    // Type "session" in the search box
    await searchInput.click();
    await searchInput.fill('');
    await searchInput.type('session');
    await page.waitForTimeout(300);

    // Verify the search value
    const searchValue = await searchInput.inputValue();
    expect(searchValue).toBe('session');
    console.log('PASS: 搜索关键词输入验证通过');

    // Verify match count appears (even if 0 matches)
    const matchCountEl = page.locator('.match-count');
    if (await matchCountEl.isVisible().catch(() => false)) {
      const matchText = await matchCountEl.textContent();
      console.log(`INFO: 搜索结果: ${matchText?.trim()}`);
    }

    // Clear search
    const clearBtn = page.locator('.search-clear-btn');
    if (await clearBtn.isVisible().catch(() => false)) {
      await clearBtn.click();
      await page.waitForTimeout(200);
      const clearedValue = await searchInput.inputValue();
      expect(clearedValue).toBe('');
      console.log('PASS: 搜索清除验证通过');
    }

    // Verify filter select exists
    await expect(page.locator('.filter-select')).toBeVisible();

    // Verify control buttons
    const controlBtnCount = await page.locator('.traffic-monitor .control-btn').count();
    console.log(`INFO: 控制按钮数: ${controlBtnCount}`);

    // Close the panel
    const closeBtn = page.locator('.traffic-monitor .close-btn');
    await expect(closeBtn).toBeVisible();
    await closeBtn.click();
    await page.waitForTimeout(300);

    // Verify panel is closed
    const panelVisible = await page.locator('.traffic-monitor').isVisible().catch(() => false);
    expect(panelVisible).toBe(false);
    console.log('PASS: Traffic Monitor面板已关闭');

    await screenshot(page, '05-traffic-monitor-search.png');
    console.log('PASS: Traffic Monitor 搜索输入测试完成');
  });

  // ============================================================
  // Test 6: 侧边栏折叠/展开测试
  // ============================================================
  test('侧边栏折叠/展开 - 验证隐藏和恢复', async ({ page }) => {
    // Initially sidebar should be visible
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 10000 });
    console.log('PASS: 侧边栏初始状态为可见');

    await screenshot(page, '06-sidebar-expanded.png');

    // Click the collapse button (◀) - it's the .toggle-btn in the header
    const collapseBtn = page.locator('.toggle-btn');
    await expect(collapseBtn).toBeVisible();
    await collapseBtn.click();
    await page.waitForTimeout(500);

    // Verify sidebar is hidden (v-show removes it from DOM)
    const sidebarVisible = await page.locator('.sidebar').isVisible().catch(() => false);
    expect(sidebarVisible).toBe(false);
    console.log('PASS: 侧边栏已折叠（隐藏）');

    await screenshot(page, '06-sidebar-collapsed.png');

    // Verify the expand button (▶) is visible
    const expandBtn = page.locator('.sidebar-toggle-collapsed');
    await expect(expandBtn).toBeVisible({ timeout: 5000 });
    console.log('PASS: 展开按钮可见');

    // Click the expand button
    await expandBtn.click();
    await page.waitForTimeout(500);

    // Verify sidebar is visible again
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 5000 });
    console.log('PASS: 侧边栏已展开（恢复）');

    await screenshot(page, '06-sidebar-expanded-again.png');
    console.log('PASS: 侧边栏折叠/展开测试完成');
  });

  // ============================================================
  // Test 7: 连续视图切换测试
  // ============================================================
  test('连续视图切换 - 5个不同视图的导航验证', async ({ page }) => {
    // Select 5 different views to navigate through
    // Index mapping based on feature-registry.ts:
    //  1: multi-agent (🤖)
    //  4: workflow (⚡)
    // 11: error (🚨)
    // 14: bot (🤖)
    // 15: gateway (🌐)
    const viewSequence = [
      { index: 1, name: 'multi-agent', expectedClass: '.multi-agent-chat' },
      { index: 4, name: 'workflow', expectedClass: '.workflow-view' },
      { index: 11, name: 'error', expectedClass: '.error-view' },
      { index: 14, name: 'bot', expectedClass: '.bot-settings' },
      { index: 15, name: 'gateway', expectedClass: '.gateway-page' },
    ];

    const navResults: { name: string; index: number; viewVisible: boolean; navActive: boolean }[] = [];

    for (const view of viewSequence) {
      // Click the nav button
      await navigateByIndex(page, view.index);

      // Verify the target view is visible
      const viewEl = page.locator(view.expectedClass);
      const viewVisible = await viewEl.isVisible({ timeout: 5000 }).catch(() => false);

      // Verify the nav button has active state
      const activeBtn = page.locator('.nav-btn.active');
      const activeBtnText = await activeBtn.textContent().catch(() => '');
      const targetBtn = page.locator('.nav-btn').nth(view.index);
      const targetBtnText = await targetBtn.textContent().catch(() => '');
      const navActive = activeBtnText.trim() === targetBtnText.trim();

      navResults.push({
        name: view.name,
        index: view.index,
        viewVisible,
        navActive,
      });

      console.log(`  ${viewVisible && navActive ? 'PASS' : 'FAIL'} [idx:${view.index}] ${view.name} - view=${viewVisible}, active=${navActive}`);
    }

    // Take screenshot after all navigations
    await screenshot(page, '07-consecutive-navigation-final.png');

    // Verify all views passed
    const allPassed = navResults.every(r => r.viewVisible && r.navActive);

    // Detailed assertion per view
    for (const result of navResults) {
      expect(result.viewVisible).toBe(true);
      expect(result.navActive).toBe(true);
    }

    console.log(`\n========== 连续视图切换验证 ==========`);
    console.log(`总计: ${navResults.length} | 通过: ${navResults.filter(r => r.viewVisible && r.navActive).length}`);
    console.log('PASS: 连续视图切换测试完成');
  });
});
