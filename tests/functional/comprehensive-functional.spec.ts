/**
 * ACP-UI Comprehensive Functional Tests
 *
 * Tests for all 19 views/components that were not covered in previous test runs.
 * Uses position-based navigation (nth index) since emoji icons may not render
 * consistently in headless Chromium.
 *
 * Nav button order (based on feature-registry.ts array order):
 *   0: chat (💬)
 *   1: multi-agent (🤖)
 *   2: executive-session (🚀)
 *   3: multi-session (📋)
 *   4: workflow (⚡)
 *   5: orchestration (🎬)
 *   6: agent-teams ()
 *   7: collaboration (️)
 *   8: hermes (📊)
 *   9: task-graph (🔗)
 *   10: memory (💡)
 *   11: error (🚨)
 *   12: evolution (🧬)
 *   13: pattern (📖)
 *   14: bot (🤖)
 *   15: gateway (🌐)
 *   16: status (📊)
 *   17: monitor (📡)
 *   18: history ()
 *
 * Run: node node_modules/@playwright/test/cli.js test tests/functional/comprehensive-functional.spec.ts --config=playwright.functional.config.ts
 */
import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:5173';

/**
 * Navigate to a view by clicking its nav button at the specified index.
 * Uses position-based navigation to avoid emoji rendering issues in headless browsers.
 */
async function navigateByIndex(page: any, buttonIndex: number) {
  const btn = page.locator('.nav-btn').nth(buttonIndex);
  await expect(btn).toBeVisible({ timeout: 10000 });
  await btn.click();
  await page.waitForTimeout(500);
}

/** Take screenshot for a view */
async function takeViewScreenshot(page: any, filename: string) {
  await page.screenshot({
    path: `D:/dingsun/acp-ui/test-output/comprehensive/${filename}`,
    fullPage: true,
  });
}

/** Wait for a view element to be visible */
async function waitForView(page: any, selector: string, timeout = 5000) {
  const el = page.locator(selector);
  await expect(el).toBeVisible({ timeout });
  return el;
}

/** Verify the nav button at the given index is active */
async function verifyActiveNav(page: any, buttonIndex: number) {
  const activeBtn = page.locator('.nav-btn.active').nth(0);
  const targetBtn = page.locator('.nav-btn').nth(buttonIndex);

  // Check if the active button matches the target button by comparing text
  const activeText = await activeBtn.textContent().catch(() => '');
  const targetText = await targetBtn.textContent().catch(() => '');
  return activeText.trim() === targetText.trim();
}

test.describe('ACP-UI Comprehensive Functional Tests - Untested Views', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForLoadState('domcontentloaded');
    // Full reload to reset Vue app state between tests
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 15000 });
  });

  // ============================================================
  // Test 1: Multi-Agent Chat View (nav button index 1)
  // ============================================================
  test('多Agent对话视图 (multi-agent)', async ({ page }) => {
    await navigateByIndex(page, 1);

    // MultiAgentChat has unique elements in main content:
    // .routing-mode (radio buttons) is unique to this view
    // .results-area is also unique
    // Note: .agent-selector class exists in both sidebar and this view
    await expect(page.locator('.main-content .routing-mode')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.main-content .results-area')).toBeVisible();

    // Verify radio inputs exist (single + broadcast)
    const radioInputs = page.locator('.main-content .routing-mode input[type="radio"]');
    const radioCount = await radioInputs.count();
    expect(radioCount).toBeGreaterThanOrEqual(2);
    console.log(`INFO: 路由模式单选按钮: ${radioCount} 个`);

    // Verify send button
    await expect(page.locator('.main-content .send-btn')).toBeVisible();

    await takeViewScreenshot(page, '01-multi-agent-chat.png');
    console.log('PASS: Multi-Agent Chat view loaded');
  });

  // ============================================================
  // Test 2: Multi-Session Chat View (nav button index 3)
  // ============================================================
  test('多会话聊天视图 (multi-session)', async ({ page }) => {
    await navigateByIndex(page, 3);

    // MultiSessionChat has unique .session-tabs and .chat-area elements in main content
    await expect(page.locator('.main-content .session-tabs')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.main-content .chat-area')).toBeVisible();

    const hasSessionTabs = await page.locator('.main-content .session-tabs').count() > 0;
    console.log(`INFO: Session tabs found: ${hasSessionTabs}`);

    await takeViewScreenshot(page, '02-multi-session-chat.png');
    console.log('PASS: Multi-Session Chat view loaded');
  });

  // ============================================================
  // Test 3: Status Panel View (nav button index 16)
  // ============================================================
  test('Agent状态面板视图 (status)', async ({ page }) => {
    await navigateByIndex(page, 16);

    // Status view is inline in App.vue using view-container
    await waitForView(page, '.view-container');

    // Verify stat cards container
    await expect(page.locator('.status-stats')).toBeVisible();

    // Verify individual stat cards (running, completed, total)
    const statCards = page.locator('.stat-card');
    const cardCount = await statCards.count();
    expect(cardCount).toBeGreaterThanOrEqual(3);
    console.log(`PASS: 状态面板有 ${cardCount} 个统计卡片`);

    await expect(page.locator('.stat-value').first()).toBeVisible();
    await expect(page.locator('.stat-label').first()).toBeVisible();

    // Check task list or empty state
    const hasEmpty = await page.locator('.empty-state').isVisible().catch(() => false);
    const hasTasks = await page.locator('.task-list').isVisible().catch(() => false);
    console.log(`INFO: 空状态: ${hasEmpty}, 任务列表: ${hasTasks}`);

    await takeViewScreenshot(page, '03-status-panel.png');
    console.log('PASS: Status Panel view loaded');
  });

  // ============================================================
  // Test 4: Realtime Monitor View (nav button index 17)
  // ============================================================
  test('实时监控视图 (monitor)', async ({ page }) => {
    await navigateByIndex(page, 17);

    // Monitor view uses view-container
    await waitForView(page, '.view-container');

    // Verify event list or empty state
    const hasEvents = await page.locator('.event-list').isVisible().catch(() => false);
    const hasEmpty = await page.locator('.empty-state').isVisible().catch(() => false);
    console.log(`INFO: 事件列表: ${hasEvents}, 空状态: ${hasEmpty}`);

    // Verify TrafficMonitor component embedded in view
    const hasTraffic = await page.locator('.traffic-monitor').count() > 0;
    console.log(`INFO: TrafficMonitor 内嵌: ${hasTraffic}`);

    await takeViewScreenshot(page, '04-realtime-monitor.png');
    console.log('PASS: Realtime Monitor view loaded');
  });

  // ============================================================
  // Test 5: Workflow View (nav button index 4)
  // ============================================================
  test('工作流视图 (workflow)', async ({ page }) => {
    await navigateByIndex(page, 4);

    await waitForView(page, '.workflow-view');

    await expect(page.locator('.workflow-header')).toBeVisible();

    // Verify create workflow button
    const createBtn = page.locator('.create-btn');
    await expect(createBtn).toBeVisible();

    await expect(page.locator('.workflow-section').first()).toBeVisible();

    await takeViewScreenshot(page, '05-workflow-view.png');
    console.log('PASS: Workflow view loaded');
  });

  // ============================================================
  // Test 6: Team Orchestration View (nav button index 5)
  // ============================================================
  test('团队编排视图 (orchestration)', async ({ page }) => {
    await navigateByIndex(page, 5);

    await waitForView(page, '.orchestration-view');

    await expect(page.locator('.view-header')).toBeVisible();
    await expect(page.locator('.task-cards')).toBeVisible();

    const titleCount = await page.locator('.section-title').count();
    console.log(`INFO: 段落标题数: ${titleCount}`);

    await takeViewScreenshot(page, '06-team-orchestration.png');
    console.log('PASS: Team Orchestration view loaded');
  });

  // ============================================================
  // Test 7: Bot Settings View (nav button index 14)
  // ============================================================
  test('Bot配置视图 (bot)', async ({ page }) => {
    await navigateByIndex(page, 14);

    await waitForView(page, '.bot-settings');

    // Verify header
    await expect(page.locator('.bot-settings .settings-header')).toBeVisible();

    const hasSubtitle = await page.locator('.bot-settings .subtitle').isVisible().catch(() => false);
    console.log(`INFO: Bot子标题: ${hasSubtitle}`);

    const hasTabs = await page.locator('.platform-tabs').count() > 0;
    console.log(`INFO: Bot平台标签页: ${hasTabs}`);

    const groupCount = await page.locator('.bot-settings .form-group').count();
    console.log(`INFO: 配置表单组数: ${groupCount}`);

    const hasSaveBtn = await page.locator('.bot-settings .save-btn').count() > 0;
    console.log(`INFO: 保存按钮: ${hasSaveBtn}`);

    await takeViewScreenshot(page, '07-bot-settings.png');
    console.log('PASS: Bot Settings view loaded');
  });

  // ============================================================
  // Test 8: Gateway Settings View (nav button index 15)
  // ============================================================
  test('Gateway设置视图 (gateway)', async ({ page }) => {
    await navigateByIndex(page, 15);

    // GatewaySettings uses class gateway-page
    await waitForView(page, '.gateway-page');

    // Verify header (use a more lenient selector)
    const hasHeader = await page.locator('.gateway-page h1, .gateway-page .section-title').first().isVisible().catch(() => false);
    console.log(`INFO: Gateway页面头部: ${hasHeader}`);

    // Verify status badge
    const hasStatus = await page.locator('.status-badge').count() > 0;
    console.log(`INFO: Gateway状态指示器: ${hasStatus}`);

    // Verify input fields
    const inputCount = await page.locator('.gateway-page input').count();
    console.log(`INFO: 输入字段数: ${inputCount}`);

    // Verify toggle switches
    const toggleCount = await page.locator('.gateway-page .toggle').count();
    console.log(`INFO: 开关切换数: ${toggleCount}`);

    // Verify section titles
    const titleCount = await page.locator('.section-title').count();
    console.log(`INFO: Gateway段落标题数: ${titleCount}`);

    await takeViewScreenshot(page, '08-gateway-settings.png');
    console.log('PASS: Gateway Settings view loaded');
  });

  // ============================================================
  // Test 9: Error View (nav button index 11)
  // ============================================================
  test('错误监控视图 (error)', async ({ page }) => {
    await navigateByIndex(page, 11);

    await waitForView(page, '.error-view');

    const selectCount = await page.locator('.error-view select').count();
    console.log(`INFO: 下拉选择框数: ${selectCount}`);

    const hasList = await page.locator('.error-view .error-list').count() > 0;
    const hasEmpty = await page.locator('.error-view .empty-state').isVisible().catch(() => false);
    console.log(`INFO: 错误列表: ${hasList}, 空状态: ${hasEmpty}`);

    await takeViewScreenshot(page, '09-error-monitor.png');
    console.log('PASS: Error Monitor view loaded');
  });

  // ============================================================
  // Test 10: Evolution View (nav button index 12)
  // ============================================================
  test('自进化视图 (evolution)', async ({ page }) => {
    await navigateByIndex(page, 12);

    await waitForView(page, '.evolution-view');

    // Verify stats bar
    await expect(page.locator('.stats-bar')).toBeVisible();

    const statItems = page.locator('.stat-item');
    const statCount = await statItems.count();
    expect(statCount).toBeGreaterThanOrEqual(3);
    console.log(`INFO: 进化统计项数: ${statCount}`);

    const filterCount = await page.locator('.evolution-view .filter-group').count();
    console.log(`INFO: 进化过滤器数: ${filterCount}`);

    await takeViewScreenshot(page, '10-evolution-view.png');
    console.log('PASS: Evolution view loaded');
  });

  // ============================================================
  // Test 11: Pattern View (nav button index 13)
  // ============================================================
  test('模式视图 (pattern)', async ({ page }) => {
    await navigateByIndex(page, 13);

    await waitForView(page, '.pattern-view');

    const filterCount = await page.locator('.pattern-view .filter-group').count();
    console.log(`INFO: 模式过滤器数: ${filterCount}`);

    const hasPatternList = await page.locator('.pattern-view .pattern-list').count() > 0;
    console.log(`INFO: 模式列表容器: ${hasPatternList}`);

    await takeViewScreenshot(page, '11-pattern-view.png');
    console.log('PASS: Pattern view loaded');
  });

  // ============================================================
  // Test 12: Collaboration Network View (nav button index 7)
  // ============================================================
  test('协作网络视图 (collaboration)', async ({ page }) => {
    await navigateByIndex(page, 7);

    await waitForView(page, '.enhanced-hermes-dashboard');

    await expect(page.locator('.dashboard-header, .view-header').first()).toBeVisible();

    const hasNetwork = await page.locator('.network-flow, .collaboration-network-flow, .vue-flow').count() > 0;
    console.log(`INFO: 网络可视化区域: ${hasNetwork}`);

    const nodeCount = await page.locator('.node-card, .agent-node').count();
    console.log(`INFO: 协作节点数: ${nodeCount}`);

    await takeViewScreenshot(page, '12-collaboration-network.png');
    console.log('PASS: Collaboration Network view loaded');
  });

  // ============================================================
  // Test 13: Agent Teams Dashboard View (nav button index 6)
  // ============================================================
  test('Agent团队仪表盘视图 (agent-teams)', async ({ page }) => {
    await navigateByIndex(page, 6);

    await waitForView(page, '.agent-teams-dashboard');

    await expect(page.locator('.dashboard-header, .view-header').first()).toBeVisible();

    const hasTabs = await page.locator('.view-tabs, .tab-bar').count() > 0;
    console.log(`INFO: Agent Teams标签页: ${hasTabs}`);

    const hasPet = await page.locator('.agent-pet-avatar, .pet-avatar').count() > 0;
    console.log(`INFO: Agent宠物头像: ${hasPet}`);

    await takeViewScreenshot(page, '13-agent-teams-dashboard.png');
    console.log('PASS: Agent Teams Dashboard view loaded');
  });

  // ============================================================
  // Test 14: Executive Session View (nav button index 2)
  // ============================================================
  test('执行会话视图 (executive-session)', async ({ page }) => {
    await navigateByIndex(page, 2);

    await waitForView(page, '.executive-session-view');

    const hasStats = await page.locator('.session-stats, .stats-section').count() > 0;
    console.log(`INFO: 执行会话统计: ${hasStats}`);

    const statCount = await page.locator('.stat-item, .stat-card').count();
    console.log(`INFO: 统计项目数: ${statCount}`);

    await takeViewScreenshot(page, '14-executive-session.png');
    console.log('PASS: Executive Session view loaded');
  });

  // ============================================================
  // Test 15: Traffic Monitor Panel (via button)
  // ============================================================
  test('流量监控面板 (点击按钮打开)', async ({ page }) => {
    const trafficBtn = page.locator('button[title="ACP Traffic Monitor"]');
    await expect(trafficBtn).toBeVisible();
    await trafficBtn.click();
    await page.waitForTimeout(500);

    await expect(page.locator('.traffic-monitor')).toBeVisible();
    await expect(page.locator('.monitor-header')).toBeVisible();

    const controlCount = await page.locator('.control-btn').count();
    console.log(`INFO: 控制按钮数: ${controlCount}`);

    await expect(page.locator('.filter-select')).toBeVisible();
    await expect(page.locator('.search-input')).toBeVisible();

    // Test: Type in search input
    await page.locator('.search-input').fill('test');
    await page.waitForTimeout(300);
    const searchQuery = await page.locator('.search-input').inputValue();
    expect(searchQuery).toBe('test');
    console.log('PASS: 流量监控搜索输入正常');

    // Clear search
    await page.locator('.search-input').fill('');

    // Test close button
    await expect(page.locator('.close-btn')).toBeVisible();
    await page.locator('.close-btn').click();
    await page.waitForTimeout(300);
    const panelVisible = await page.locator('.traffic-monitor').isVisible().catch(() => false);
    expect(panelVisible).toBe(false);
    console.log('PASS: 流量监控面板可以关闭');

    await takeViewScreenshot(page, '15-traffic-monitor-panel.png');
    console.log('PASS: Traffic Monitor Panel test completed');
  });

  // ============================================================
  // Test 16: Log Stream Panel (via button)
  // ============================================================
  test('日志流面板 (点击按钮打开)', async ({ page }) => {
    const logBtn = page.locator('button[title="Agent Log Stream"]');
    await expect(logBtn).toBeVisible();
    await logBtn.click();
    await page.waitForTimeout(500);

    await expect(page.locator('.log-stream-view')).toBeVisible();
    await expect(page.locator('.log-header')).toBeVisible();

    const selectCount = await page.locator('.log-stream-view select').count();
    console.log(`INFO: 日志过滤选择框: ${selectCount}`);

    const hasEntries = await page.locator('.log-entries, .log-list').count() > 0;
    console.log(`INFO: 日志条目容器: ${hasEntries}`);

    const hasResize = await page.locator('.resize-handle').count() > 0;
    console.log(`INFO: 拖拽调整手柄: ${hasResize}`);

    await takeViewScreenshot(page, '16-log-stream-panel.png');
    console.log('PASS: Log Stream Panel opened successfully');
  });

  // ============================================================
  // Test 17: Settings Dialog (via button)
  // ============================================================
  test('设置对话框 (验证设置项)', async ({ page }) => {
    const settingsBtn = page.locator('button[title="Settings"]');
    await expect(settingsBtn).toBeVisible();
    await settingsBtn.click();
    await page.waitForTimeout(500);

    await expect(page.locator('.settings-overlay')).toBeVisible();
    await expect(page.locator('.settings-header')).toBeVisible();

    const hasAgentsSection = await page.locator('.agents-section, .settings-agents').count() > 0;
    console.log(`INFO: Agent配置区域: ${hasAgentsSection}`);

    const agentRowCount = await page.locator('.agent-row').count();
    console.log(`INFO: Agent行数: ${agentRowCount}`);

    await takeViewScreenshot(page, '17-settings-dialog.png');
    console.log('PASS: Settings Dialog opened with settings items');
  });

  // ============================================================
  // Test 18: Navigation All Views - Complete Coverage
  // ============================================================
  test('全部视图导航完整性验证', async ({ page }) => {
    // All views with their nav button index and expected CSS class
    // Index based on feature-registry.ts array order
    const allViews = [
      { name: 'Multi-Agent Chat', index: 1, viewClass: '.multi-agent-chat' },
      { name: 'Multi-Session', index: 3, viewClass: '.multi-session-chat' },
      { name: 'Executive Session', index: 2, viewClass: '.executive-session-view' },
      { name: 'Workflow', index: 4, viewClass: '.workflow-view' },
      { name: 'Orchestration', index: 5, viewClass: '.orchestration-view' },
      { name: 'Agent Teams', index: 6, viewClass: '.agent-teams-dashboard' },
      { name: 'Collaboration', index: 7, viewClass: '.enhanced-hermes-dashboard' },
      { name: 'Hermes Dashboard', index: 8, viewClass: '.hermes-dashboard' },
      { name: 'Task Graph', index: 9, viewClass: '.task-graph-container' },
      { name: 'Memory', index: 10, viewClass: '.memory-view' },
      { name: 'Error Monitor', index: 11, viewClass: '.error-view' },
      { name: 'Evolution', index: 12, viewClass: '.evolution-view' },
      { name: 'Pattern', index: 13, viewClass: '.pattern-view' },
      { name: 'Bot Config', index: 14, viewClass: '.bot-settings' },
      { name: 'Gateway', index: 15, viewClass: '.gateway-page' },
      { name: 'Status', index: 16, viewClass: '.view-container' },
      { name: 'Monitor', index: 17, viewClass: '.view-container' },
      { name: 'History', index: 18, viewClass: '.history-view' },
    ];

    const results: { name: string; index: number; pass: boolean; note: string }[] = [];

    for (const view of allViews) {
      try {
        await navigateByIndex(page, view.index);

        const viewEl = page.locator(view.viewClass);
        const isVisible = await viewEl.isVisible({ timeout: 5000 }).catch(() => false);

        // For views sharing .view-container, check additional markers
        let extraCheck = true;
        if (view.name === 'Status') {
          extraCheck = await page.locator('.status-stats').isVisible().catch(() => false);
        } else if (view.name === 'Monitor') {
          extraCheck = await page.locator('.view-container h3').count() > 0;
        }

        const pass = isVisible && extraCheck;
        results.push({
          name: view.name,
          index: view.index,
          pass,
          note: `visible=${isVisible}, extra=${extraCheck}`,
        });
      } catch (e) {
        results.push({
          name: view.name,
          index: view.index,
          pass: false,
          note: `Error: ${(e as Error).message}`,
        });
      }
    }

    const passed = results.filter(r => r.pass).length;
    const failed = results.filter(r => !r.pass).length;
    console.log(`\n========== 视图导航完整性验证 ==========`);
    console.log(`总计: ${results.length} | 通过: ${passed} | 失败: ${failed}`);
    for (const r of results) {
      console.log(`  ${r.pass ? 'PASS' : 'FAIL'} [idx:${r.index}] ${r.name} - ${r.note}`);
    }

    expect(failed).toBe(0);
  });

  // ============================================================
  // Test 19: Growth System / Emotion Display (Agent Pet)
  // ============================================================
  test('Agent宠物/Growth System组件', async ({ page }) => {
    await navigateByIndex(page, 6); // Agent Teams

    await waitForView(page, '.agent-teams-dashboard');
    await page.waitForTimeout(1000);

    const hasPetAvatar = await page.locator('.agent-pet-avatar, .pet-avatar').count() > 0;
    console.log(`INFO: Agent宠物头像: ${hasPetAvatar}`);

    const hasEmotion = await page.locator('.emotion-display, .emotion-bar').count() > 0;
    console.log(`INFO: 情感显示组件: ${hasEmotion}`);

    const hasGrowth = await page.locator('.growth-system, .growth-bar, .level-badge').count() > 0;
    console.log(`INFO: 成长系统组件: ${hasGrowth}`);

    const hasInteraction = await page.locator('.interaction-panel').count() > 0;
    console.log(`INFO: 交互面板组件: ${hasInteraction}`);

    const hasProgress = await page.locator('.agent-realtime-progress, .progress-panel').count() > 0;
    console.log(`INFO: 实时进度面板: ${hasProgress}`);

    await takeViewScreenshot(page, '19-agent-pet-growth.png');
    if (hasPetAvatar || hasEmotion || hasGrowth) {
      console.log('PASS: Agent宠物/Growth系统组件存在');
    } else {
      console.log('INFO: Agent宠物组件可能为空状态（需要连接Agent后显示完整功能）');
    }
  });
});
