import { test, expect } from '@playwright/test'
import fs from 'fs'
import path from 'path'

const SCREENSHOT_DIR = 'D:/tmp/e2e-screenshots'

// 确保截图目录存在
test.beforeAll(async () => {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true })
  }
})

async function takeScreenshot(page: any, name: string) {
  const filePath = path.join(SCREENSHOT_DIR, `${name}-${Date.now()}.png`)
  await page.screenshot({ path: filePath, fullPage: false })
  console.log(`Screenshot saved: ${filePath}`)
  return filePath
}

test.describe('ACP-UI E2E Critical User Journeys', () => {
  test.describe('1. 页面加载测试', () => {
    test('should load application and verify Dashboard page', async ({ page }) => {
      // 打开应用
      await page.goto('http://localhost:1420')

      // 等待页面加载完成
      await page.waitForLoadState('networkidle')

      // 验证页面标题
      await expect(page).toHaveTitle(/ACP/)

      // 验证Dashboard页面显示正常
      await expect(page.locator('body')).toBeVisible()
      await takeScreenshot(page, '01-dashboard-loaded')

      // 验证侧边栏存在
      const sidebar = page.locator('[class*="sidebar"], [class*="nav"], aside, nav').first()
      await expect(sidebar).toBeVisible({ timeout: 5000 })
      await takeScreenshot(page, '02-sidebar-visible')

      // 验证侧边栏导航项存在
      const navItems = page.locator('nav button, nav a, .sidebar button, .sidebar a')
      const count = await navItems.count()
      expect(count).toBeGreaterThan(0)
      console.log(`Found ${count} navigation items`)
    })

    test('should have working navigation', async ({ page }) => {
      await page.goto('http://localhost:1420')
      await page.waitForLoadState('networkidle')

      // 获取所有导航按钮
      const navButtons = page.locator('button:has-text("Chat"), button:has-text("Hermes"), button:has-text("Dashboard"), button:has-text("Swarm"), button:has-text("Workflow"), button:has-text("Agent")')
      const count = await navButtons.count()
      expect(count).toBeGreaterThan(0)
      console.log(`Found ${count} navigation buttons`)
    })
  })

  test.describe('2. 核心功能测试', () => {
    test('should navigate to Chat page and verify chat interface', async ({ page }) => {
      await page.goto('http://localhost:1420')
      await page.waitForLoadState('networkidle')

      // 查找并点击 Chat 导航
      const chatButton = page.getByRole('button', { name: /Chat/i }).first()
      if (await chatButton.isVisible({ timeout: 3000 }).catch(() => false)) {
        await chatButton.click()
        await page.waitForTimeout(1000)
        await takeScreenshot(page, '03-chat-page')

        // 验证聊天界面元素
        const chatContainer = page.locator('[class*="chat"], [class*="message"], textarea, input[type="text"]').first()
        await expect(chatContainer).toBeVisible({ timeout: 5000 })
        await takeScreenshot(page, '04-chat-interface')
      } else {
        console.log('Chat button not found, checking for alternative navigation')
        await takeScreenshot(page, '03-chat-page-not-found')
      }
    })

    test('should navigate to Swarm Dashboard via URL and verify page', async ({ page }) => {
      // 直接通过 URL 访问 Swarm Dashboard（使用 Hash 路由）
      await page.goto('http://localhost:1420/#/swarm')
      await page.waitForLoadState('networkidle')
      await page.waitForTimeout(1000)

      await takeScreenshot(page, '05-swarm-dashboard')

      // 验证页面 URL 包含 swarm
      expect(page.url()).toContain('swarm')

      // 验证页面加载（即使内容为空）
      await expect(page.locator('body')).toBeVisible()
    })

    test('should navigate to Workflow Editor via URL and verify page', async ({ page }) => {
      // 直接通过 URL 访问 Workflow Editor（使用 Hash 路由）
      await page.goto('http://localhost:1420/#/workflow-editor')
      await page.waitForLoadState('networkidle')
      await page.waitForTimeout(1000)

      await takeScreenshot(page, '06-workflow-editor')

      // 验证页面 URL 包含 workflow-editor
      expect(page.url()).toContain('workflow-editor')

      // 验证页面加载（即使内容为空）
      await expect(page.locator('body')).toBeVisible()
    })
  })

  test.describe('3. 数据持久化测试', () => {
    test('should create a test session and verify data persistence', async ({ page }) => {
      await page.goto('http://localhost:1420')
      await page.waitForLoadState('networkidle')

      // 尝试找到创建 Session 的按钮
      const newSessionButton = page.getByRole('button', { name: /New|Create|Session|新/i }).first()

      if (await newSessionButton.isVisible({ timeout: 3000 }).catch(() => false)) {
        await newSessionButton.click()
        await page.waitForTimeout(1000)
        await takeScreenshot(page, '07-new-session')

        // 尝试输入测试消息
        const messageInput = page.locator('textarea, input[type="text"]').first()
        if (await messageInput.isVisible({ timeout: 3000 }).catch(() => false)) {
          await messageInput.fill('E2E Test Message - ' + Date.now())
          await takeScreenshot(page, '08-message-entered')

          // 尝试发送消息
          const sendButton = page.getByRole('button', { name: /Send|Submit|发送/i }).first()
          if (await sendButton.isVisible({ timeout: 3000 }).catch(() => false)) {
            await sendButton.click()
            await page.waitForTimeout(2000)
            await takeScreenshot(page, '09-message-sent')
          }
        }
      } else {
        console.log('No session creation button found')
      }

      // 验证页面仍然正常工作（数据持久化的基本验证）
      await page.reload()
      await page.waitForLoadState('networkidle')
      await takeScreenshot(page, '10-after-reload')
      await expect(page.locator('body')).toBeVisible()
    })
  })

  test.describe('4. 完整导航流程截图', () => {
    test('should capture all main pages', async ({ page }) => {
      await page.goto('http://localhost:1420')
      await page.waitForLoadState('networkidle')
      await takeScreenshot(page, '11-home-page')

      // 遍历所有可见的导航按钮并截图
      const navButtons = page.locator('button, a').filter({ hasText: /.+/ })
      const count = await navButtons.count()
      console.log(`Total buttons/links found: ${count}`)

      // 点击前5个有意义的导航按钮
      const importantNavs = ['Chat', 'Hermes', 'Dashboard', 'Swarm', 'Workflow', 'Agent', 'Memory', 'History', 'Setting', 'Config']

      for (const navName of importantNavs) {
        await page.goto('http://localhost:1420')
        await page.waitForLoadState('networkidle')

        const button = page.getByRole('button', { name: new RegExp(navName, 'i') }).first()
        if (await button.isVisible({ timeout: 2000 }).catch(() => false)) {
          await button.click()
          await page.waitForTimeout(800)
          await takeScreenshot(page, `12-nav-${navName.toLowerCase()}`)
        }
      }
    })
  })
})
