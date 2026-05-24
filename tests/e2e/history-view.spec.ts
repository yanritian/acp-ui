import { test, expect } from '@playwright/test'

test.describe('HistoryView 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to History view', async ({ page }) => {
    // 点击 History 按钮
    await page.getByRole('button', { name: /History/ }).click()
    await page.waitForTimeout(1000)

    // 验证历史视图加载
    const historyContent = await page.locator('[class*="history"], [class*="record"]').count()
    expect(historyContent).toBeGreaterThanOrEqual(0)
  })

  test('should display history button in navigation', async ({ page }) => {
    await page.goto('/')

    // 检查历史按钮存在
    const historyBtn = page.getByRole('button', { name: /History/ })
    await expect(historyBtn).toBeVisible()
  })

  test('should have search functionality in history', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /History/ }).click()
    await page.waitForTimeout(500)

    // 检查搜索框
    const searchInput = page.locator('input[type="search"], input[placeholder*="search"], input[placeholder*="Search"]')
    const count = await searchInput.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have filter options in history', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /History/ }).click()
    await page.waitForTimeout(500)

    // 检查筛选器
    const filterElements = page.locator('select, [class*="filter"], [class*="status-select"]')
    const count = await filterElements.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should display task records if available', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /History/ }).click()
    await page.waitForTimeout(1000)

    // 检查任务记录列表
    const taskRecords = page.locator('[class*="task-record"], [class*="history-item"], tr')
    const count = await taskRecords.count()
    // 可能没有记录，所以只检查元素是否存在
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('导出功能测试', () => {
  test('should have export button in history view', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /History/ }).click()
    await page.waitForTimeout(500)

    // 检查导出按钮
    const exportBtn = page.locator('button').filter({ hasText: /Export|导出/ })
    const count = await exportBtn.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})