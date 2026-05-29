import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:5173';

test.describe('ACP-UI Web Application Functional Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
  });

  // Test 1: Home page loads correctly
  test('首页正确显示 - 验证ACP UI标题', async ({ page }) => {
    const title = page.locator('h1', { hasText: 'ACP UI' });
    await expect(title).toBeVisible();
    console.log('PASS: 首页ACP UI标题正确显示');
  });

  // Test 2: Sidebar navigation buttons exist with emojis
  test('侧边栏导航按钮存在 - 验证包含emoji的按钮', async ({ page }) => {
    const navButtons = page.locator('.nav-btn');
    const count = await navButtons.count();
    expect(count).toBeGreaterThan(0);

    // Check that buttons have icons (emojis)
    const navIcons = page.locator('.nav-icon');
    const iconCount = await navIcons.count();
    expect(iconCount).toBeGreaterThan(0);
    console.log(`PASS: 侧边栏有 ${count} 个导航按钮，${iconCount} 个包含图标`);
  });

  // Test 3: View switching - click different buttons
  test('视图切换功能 - 点击不同按钮切换视图', async ({ page }) => {
    const testCases = [
      { icon: '📊', name: 'Hermes Dashboard', viewId: 'hermes' },
      { icon: '🎬', name: 'Orchestration', viewId: 'orchestration' },
      { icon: '🔗', name: 'Task Graph', viewId: 'task-graph' },
      { icon: '💡', name: 'Memory', viewId: 'memory' },
      { icon: '📚', name: 'History', viewId: 'history' },
    ];

    for (const tc of testCases) {
      // Click the navigation button by its icon
      const btn = page.locator(`.nav-btn:has-text("${tc.icon}")`).first();
      await btn.click();
      await page.waitForTimeout(300); // Allow view transition

      // Verify the button is now active
      await expect(btn).toHaveClass(/active/);
      console.log(`PASS: 成功切换到 ${tc.name} 视图`);
    }
  });

  // Test 4: Language switching functionality
  test('语言切换功能', async ({ page }) => {
    // Check if language selector exists in sidebar
    const languageSection = page.locator('.language-section');
    await expect(languageSection).toBeVisible();
    console.log('PASS: 语言选择区域存在');

    // Check for language dropdown (uses <details> element, not <select>)
    const langDropdown = page.locator('.lang-dropdown');
    await expect(langDropdown).toBeVisible();

    // Open the dropdown
    await langDropdown.click();
    await page.waitForTimeout(300);

    // Check language options in the list
    const langList = page.locator('.lang-list');
    await expect(langList).toBeVisible();

    const langItems = page.locator('.lang-item');
    const itemCount = await langItems.count();
    expect(itemCount).toBeGreaterThan(1);
    console.log(`PASS: 语言选择器有 ${itemCount} 个语言选项`);

    // Verify we can see language names
    const firstLang = langItems.first();
    await expect(firstLang).toBeVisible();
    const langText = await firstLang.textContent();
    expect(langText?.length).toBeGreaterThan(0);
    console.log(`PASS: 第一个语言选项: "${langText?.trim()}"`);
  });

  // Test 5: Agent selector component
  test('Agent选择器组件', async ({ page }) => {
    // Check agent selector exists
    const agentSelector = page.locator('.agent-selector, [class*="agent-select"]').first();
    await expect(agentSelector).toBeVisible();
    console.log('PASS: Agent选择器组件存在');

    // Check for agent list or dropdown
    const agentList = page.locator('.agent-list, [class*="agent-list"]');
    const agentListVisible = await agentList.count();
    console.log(`INFO: Agent列表组件存在: ${agentListVisible > 0}`);
  });

  // Test 6: Session list component
  test('Session列表组件', async ({ page }) => {
    // Check session list exists
    const sessionList = page.locator('.session-list, [class*="session-list"]').first();
    await expect(sessionList).toBeVisible();
    console.log('PASS: Session列表组件存在');

    // Check for session-related elements
    const sessionHeader = page.locator('[class*="session"], h3:has-text("Session"), h3:has-text("会话")').first();
    if (await sessionHeader.isVisible()) {
      console.log('PASS: Session列表标题存在');
    }
  });

  // Test 7: Hermes Dashboard view
  test('Hermes Dashboard视图', async ({ page }) => {
    // Click on Hermes navigation button (📊 icon)
    const hermesBtn = page.locator('.nav-btn:has-text("📊")').first();
    await hermesBtn.click();
    await page.waitForTimeout(500);

    // Verify Hermes Dashboard is shown by checking the dashboard class
    const hermesView = page.locator('.hermes-dashboard');
    await expect(hermesView).toBeVisible();
    console.log('PASS: Hermes Dashboard视图可访问');

    // Also verify the title is present
    const hermesTitle = page.locator('.hermes-title');
    await expect(hermesTitle).toBeVisible();
    console.log('PASS: Hermes Dashboard标题可见');
  });

  // Test 8: Sidebar toggle functionality
  test('侧边栏折叠/展开功能', async ({ page }) => {
    const toggleBtn = page.locator('.toggle-btn, button:has-text("◀")').first();
    await expect(toggleBtn).toBeVisible();
    await toggleBtn.click();
    await page.waitForTimeout(300);

    // Sidebar should be hidden, toggle should show "▶"
    const expandBtn = page.locator('.sidebar-toggle-collapsed, button:has-text("▶")');
    await expect(expandBtn).toBeVisible();
    console.log('PASS: 侧边栏折叠功能正常');

    // Expand again
    await expandBtn.click();
    await page.waitForTimeout(300);
    const sidebar = page.locator('.sidebar');
    await expect(sidebar).toBeVisible();
    console.log('PASS: 侧边栏展开功能正常');
  });

  // Test 9: Settings button functionality
  test('设置按钮功能', async ({ page }) => {
    const settingsBtn = page.locator('.settings-btn[title="Settings"]').first();
    await settingsBtn.click();
    await page.waitForTimeout(300);

    // Settings overlay should appear
    const settingsOverlay = page.locator('.settings-overlay');
    await expect(settingsOverlay).toBeVisible();
    console.log('PASS: 设置对话框可以打开');

    // Verify settings header is visible
    const settingsHeader = page.locator('.settings-header');
    await expect(settingsHeader).toBeVisible();
    console.log('PASS: 设置对话框标题可见');
  });

  // Test 10: Traffic monitor and log stream buttons
  test('流量监控和日志流按钮', async ({ page }) => {
    const headerButtons = page.locator('.header-actions button');
    const count = await headerButtons.count();
    expect(count).toBeGreaterThanOrEqual(3); // Log, Traffic, Settings
    console.log(`PASS: 头部操作按钮有 ${count} 个`);

    // Log stream button
    const logBtn = headerButtons.first();
    await logBtn.click();
    await page.waitForTimeout(300);
    const logStream = page.locator('.log-stream-view, [class*="log-stream"]');
    await expect(logStream).toBeVisible();
    console.log('PASS: 日志流面板可以打开');
  });

  // Test 11: Welcome screen elements when not connected
  test('未连接状态下的欢迎页面', async ({ page }) => {
    // Should show welcome text
    const welcomeTitle = page.locator('.welcome-screen h2');
    const isVisible = await welcomeTitle.isVisible().catch(() => false);

    if (isVisible) {
      const titleText = await welcomeTitle.textContent();
      expect(titleText?.length).toBeGreaterThan(0);
      console.log(`PASS: 欢迎页面标题: "${titleText}"`);
    } else {
      console.log('INFO: 未显示欢迎页面（可能已连接或不在chat视图）');
    }
  });

  // Test 12: Verify all feature navigation buttons have correct attributes
  test('所有导航按钮的属性验证', async ({ page }) => {
    const navButtons = page.locator('.nav-btn');
    const count = await navButtons.count();

    for (let i = 0; i < count; i++) {
      const btn = navButtons.nth(i);
      const hasIcon = await btn.locator('.nav-icon').count();
      const hasText = await btn.locator('.nav-text').count();

      expect(hasIcon).toBeGreaterThan(0);
      expect(hasText).toBeGreaterThan(0);
    }
    console.log(`PASS: 所有 ${count} 个导航按钮都包含图标和文本`);
  });

  // Test 13: Take screenshots of key pages
  test('关键页面截图', async ({ page }) => {
    // Screenshot 1: Main page with sidebar
    await page.screenshot({
      path: 'D:/dingsun/acp-ui/test-output/01-main-page.png',
      fullPage: true,
    });
    console.log('PASS: 主页截图已保存');

    // Screenshot 2: After clicking Hermes Dashboard
    const hermesBtn = page.locator('.nav-btn:has-text("📊")').first();
    await hermesBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({
      path: 'D:/dingsun/acp-ui/test-output/02-hermes-dashboard.png',
      fullPage: true,
    });
    console.log('PASS: Hermes Dashboard截图已保存');

    // Screenshot 3: Task Graph View
    const taskGraphBtn = page.locator('.nav-btn:has-text("🔗")').first();
    await taskGraphBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({
      path: 'D:/dingsun/acp-ui/test-output/03-task-graph-view.png',
      fullPage: true,
    });
    console.log('PASS: Task Graph View截图已保存');

    // Screenshot 4: Memory View
    const memoryBtn = page.locator('.nav-btn:has-text("💡")').first();
    await memoryBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({
      path: 'D:/dingsun/acp-ui/test-output/04-memory-view.png',
      fullPage: true,
    });
    console.log('PASS: Memory View截图已保存');

    // Screenshot 5: Agent Teams Dashboard
    const agentTeamsBtn = page.locator('.nav-btn:has-text("🚀")').first();
    await agentTeamsBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({
      path: 'D:/dingsun/acp-ui/test-output/05-agent-teams-dashboard.png',
      fullPage: true,
    });
    console.log('PASS: Agent Teams Dashboard截图已保存');
  });
});
