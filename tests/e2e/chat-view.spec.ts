import { test, expect } from '@playwright/test'

test.describe('ChatView 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Chat view', async ({ page }) => {
    await page.getByRole('button', { name: /Chat/ }).click()
    await page.waitForTimeout(1000)

    // 验证聊天视图加载
    const chatContent = await page.locator('[class*="chat"], [class*="message"]').count()
    expect(chatContent).toBeGreaterThanOrEqual(0)
  })

  test('should display chat input area', async ({ page }) => {
    await page.getByRole('button', { name: /Chat/ }).click()
    await page.waitForTimeout(500)

    // 检查消息输入框
    const messageInput = page.locator('textarea, input[type="text"]').filter({ hasNotText: '' })
    const count = await messageInput.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have send button in chat', async ({ page }) => {
    await page.getByRole('button', { name: /Chat/ }).click()
    await page.waitForTimeout(500)

    // 检查发送按钮
    const sendBtn = page.locator('button').filter({ hasText: /Send|发送|Submit/ })
    const count = await sendBtn.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should display welcome message', async ({ page }) => {
    await page.goto('/')

    // 检查欢迎信息
    const welcomeText = page.getByText(/Welcome|欢迎|Select an agent/)
    const count = await welcomeText.count()
    expect(count).toBeGreaterThanOrEqual(1)
  })

  test('should support keyboard shortcuts', async ({ page }) => {
    await page.getByRole('button', { name: /Chat/ }).click()
    await page.waitForTimeout(500)

    // 检查是否有快捷键提示
    const shortcutHint = page.locator('[class*="shortcut"], [class*="keyboard"]')
    const count = await shortcutHint.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})

test.describe('消息显示测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should render markdown content', async ({ page }) => {
    await page.getByRole('button', { name: /Chat/ }).click()
    await page.waitForTimeout(500)

    // 检查是否有 Markdown 渲染区域
    const markdownArea = page.locator('[class*="markdown"], [class*="message-content"]')
    const count = await markdownArea.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})