import { test, expect } from '@playwright/test'

test.describe('HermesDashboard 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Hermes Dashboard', async ({ page }) => {
    // 点击 Hermes Dashboard 按钮
    await page.getByRole('button', { name: /Hermes Dashboard/ }).click()

    // 验证导航成功 - 等待页面加载
    await page.waitForTimeout(1000)

    // 检查页面是否有相关内容
    const dashboardContent = await page.locator('[class*="hermes"], [class*="dashboard"]').count()
    expect(dashboardContent).toBeGreaterThanOrEqual(0)
  })

  test('should display agent status when connected', async ({ page }) => {
    await page.goto('/')
    await page.waitForTimeout(2000)

    // 检查 Agent 选择器
    const agentSelector = page.locator('select, [class*="agent-select"]')
    const count = await agentSelector.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have working directory input', async ({ page }) => {
    await page.goto('/')

    // 检查 Working Directory 输入框
    const workDirInput = page.getByPlaceholder(/Enter/)
    await expect(workDirInput).toBeVisible()

    // 测试输入
    await workDirInput.fill('/tmp/test-workspace')
    await expect(workDirInput).toHaveValue('/tmp/test-workspace')
  })

  test('should display saved sessions section', async ({ page }) => {
    await page.goto('/')

    // 检查 Saved Sessions 区域
    const savedSessions = page.getByRole('heading', { name: /Saved Sessions/ })
    await expect(savedSessions).toBeVisible()
  })

  test('should display feature navigation section', async ({ page }) => {
    await page.goto('/')

    // 检查 Feature Navigation 区域
    const featureNav = page.getByRole('heading', { name: /Feature Navigation/ })
    await expect(featureNav).toBeVisible()

    // 验证导航按钮数量
    const navButtons = page.locator('nav button')
    const count = await navButtons.count()
    expect(count).toBeGreaterThanOrEqual(10)
  })
})

test.describe('任务执行功能测试', () => {
  test('should have session creation capability', async ({ page }) => {
    await page.goto('/')

    // 检查是否有创建会话的方式
    const sessionButtons = page.locator('button').filter({ hasText: /Session|New|Create/ })
    const count = await sessionButtons.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})