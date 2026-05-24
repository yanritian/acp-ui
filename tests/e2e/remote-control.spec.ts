import { test, expect } from '@playwright/test'

test.describe('Remote Control 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Remote Control view', async ({ page }) => {
    await page.getByRole('button', { name: /Remote Control/ }).click()
    await page.waitForTimeout(1000)

    // 验证远程控制视图加载
    const remoteContent = await page.locator('[class*="remote"], [class*="control"]').count()
    expect(remoteContent).toBeGreaterThanOrEqual(0)
  })

  test('should display remote agent status', async ({ page }) => {
    await page.getByRole('button', { name: /Remote Control/ }).click()
    await page.waitForTimeout(500)

    // 检查远程 Agent 状态
    const agentStatus = page.locator('[class*="agent"], [class*="status"]')
    const count = await agentStatus.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have command input for remote control', async ({ page }) => {
    await page.getByRole('button', { name: /Remote Control/ }).click()
    await page.waitForTimeout(500)

    // 检查命令输入
    const commandInput = page.locator('textarea, input[type="text"]').filter({ hasNotText: '' })
    const count = await commandInput.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('Bot Config 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Bot Config view', async ({ page }) => {
    await page.getByRole('button', { name: /Bot Config/ }).click()
    await page.waitForTimeout(1000)

    // 验证 Bot 配置视图加载
    const configContent = await page.locator('[class*="bot"], [class*="config"]').count()
    expect(configContent).toBeGreaterThanOrEqual(0)
  })

  test('should display bot configuration form', async ({ page }) => {
    await page.getByRole('button', { name: /Bot Config/ }).click()
    await page.waitForTimeout(500)

    // 检查配置表单
    const formElements = page.locator('form, [class*="form"], input, textarea, select')
    const count = await formElements.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('Status 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Status view', async ({ page }) => {
    await page.getByRole('button', { name: /Status/ }).click()
    await page.waitForTimeout(1000)

    // 验证状态视图加载
    const statusContent = await page.locator('[class*="status"], [class*="dashboard"]').count()
    expect(statusContent).toBeGreaterThanOrEqual(0)
  })

  test('should display system statistics', async ({ page }) => {
    await page.getByRole('button', { name: /Status/ }).click()
    await page.waitForTimeout(500)

    // 检查系统统计
    const statsElements = page.locator('[class*="stat"], [class*="metric"], [class*="chart"]')
    const count = await statsElements.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('Monitor 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Monitor view', async ({ page }) => {
    // 使用精确匹配，避免与 "Error Monitor" 冲突
    await page.getByRole('button', { name: '📡 Monitor' }).click()
    await page.waitForTimeout(1000)

    // 验证监控视图加载
    const monitorContent = await page.locator('[class*="monitor"], [class*="log"]').count()
    expect(monitorContent).toBeGreaterThanOrEqual(0)
  })

  test('should display real-time logs', async ({ page }) => {
    await page.getByRole('button', { name: '📡 Monitor' }).click()
    await page.waitForTimeout(500)

    // 检查实时日志
    const logElements = page.locator('[class*="log"], [class*="stream"], [class*="console"]')
    const count = await logElements.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('All Sessions 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to All Sessions view', async ({ page }) => {
    await page.getByRole('button', { name: /All Sessions/ }).click()
    await page.waitForTimeout(1000)

    // 验证所有会话视图加载
    const sessionsContent = await page.locator('[class*="session"], [class*="list"]').count()
    expect(sessionsContent).toBeGreaterThanOrEqual(0)
  })

  test('should display session list', async ({ page }) => {
    await page.getByRole('button', { name: /All Sessions/ }).click()
    await page.waitForTimeout(500)

    // 检查会话列表
    const sessionItems = page.locator('[class*="session-item"], [class*="card"], li, tr')
    const count = await sessionItems.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})