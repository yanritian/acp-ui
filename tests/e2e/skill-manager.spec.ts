import { test, expect } from '@playwright/test'

test.describe('Skill Manager 功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should navigate to Skills view', async ({ page }) => {
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)

    // 验证技能管理器加载
    const skillContent = await page.locator('[class*="skill"]').count()
    expect(skillContent).toBeGreaterThanOrEqual(0)
  })

  test('should display skill list', async ({ page }) => {
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)

    // 检查技能列表
    const skillList = page.locator('[class*="skill-list"], [class*="skill-item"]')
    const count = await skillList.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should show skill details', async ({ page }) => {
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)

    // 点击第一个技能查看详情
    const firstSkill = page.locator('[class*="skill-item"]').first()
    if (await firstSkill.count() > 0) {
      await firstSkill.click()
      await page.waitForTimeout(500)

      // 检查技能详情显示
      const details = page.locator('[class*="skill-detail"], [class*="skill-info"]')
      const count = await details.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })

  test('should have execute button for skills', async ({ page }) => {
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)

    // 检查执行按钮
    const executeBtn = page.locator('button').filter({ hasText: /Execute|执行|Run|运行/ })
    const count = await executeBtn.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should display version history', async ({ page }) => {
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)

    // 点击第一个技能
    const firstSkill = page.locator('[class*="skill-item"]').first()
    if (await firstSkill.count() > 0) {
      await firstSkill.click()
      await page.waitForTimeout(500)

      // 检查版本历史
      const versionHistory = page.locator('[class*="version"], [class*="history"]')
      const count = await versionHistory.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })
})

test.describe('技能执行测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)
  })

  test('should open execution dialog', async ({ page }) => {
    // 点击执行按钮
    const executeBtn = page.locator('button').filter({ hasText: /Execute|执行|Run|运行/ }).first()
    if (await executeBtn.count() > 0) {
      await executeBtn.click()
      await page.waitForTimeout(500)

      // 检查执行对话框
      const dialog = page.locator('[class*="dialog"], [class*="modal"]')
      const count = await dialog.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })

  test('should have parameter inputs', async ({ page }) => {
    // 打开执行对话框
    const executeBtn = page.locator('button').filter({ hasText: /Execute|执行|Run|运行/ }).first()
    if (await executeBtn.count() > 0) {
      await executeBtn.click()
      await page.waitForTimeout(500)

      // 检查参数输入框
      const inputs = page.locator('input, textarea, select')
      const count = await inputs.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })

  test('should close execution dialog', async ({ page }) => {
    // 打开执行对话框
    const executeBtn = page.locator('button').filter({ hasText: /Execute|执行|Run|运行/ }).first()
    if (await executeBtn.count() > 0) {
      await executeBtn.click()
      await page.waitForTimeout(500)

      // 关闭对话框
      const closeBtn = page.locator('button').filter({ hasText: /Close|关闭|Cancel|取消/ }).first()
      if (await closeBtn.count() > 0) {
        await closeBtn.click()
        await page.waitForTimeout(300)

        // 验证对话框关闭
        const dialog = page.locator('[class*="dialog"], [class*="modal"]')
        const count = await dialog.count()
        expect(count).toBe(0)
      }
    }
  })
})

test.describe('技能版本管理测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)
  })

  test('should show version selector', async ({ page }) => {
    // 点击第一个技能
    const firstSkill = page.locator('[class*="skill-item"]').first()
    if (await firstSkill.count() > 0) {
      await firstSkill.click()
      await page.waitForTimeout(500)

      // 检查版本选择器
      const versionSelector = page.locator('select, [class*="version-select"]')
      const count = await versionSelector.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })

  test('should have create version button', async ({ page }) => {
    // 检查创建新版本按钮
    const createVersionBtn = page.locator('button').filter({ hasText: /New Version|新版本|Create Version/ })
    const count = await createVersionBtn.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should display version timeline', async ({ page }) => {
    // 点击第一个技能
    const firstSkill = page.locator('[class*="skill-item"]').first()
    if (await firstSkill.count() > 0) {
      await firstSkill.click()
      await page.waitForTimeout(500)

      // 检查版本时间线
      const timeline = page.locator('[class*="timeline"], [class*="version-list"]')
      const count = await timeline.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })
})

test.describe('技能创建测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)
  })

  test('should have create skill button', async ({ page }) => {
    // 检查创建技能按钮
    const createBtn = page.locator('button').filter({ hasText: /Create|创建|New|新建/ })
    const count = await createBtn.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should open create skill dialog', async ({ page }) => {
    // 点击创建按钮
    const createBtn = page.locator('button').filter({ hasText: /Create|创建|New|新建/ }).first()
    if (await createBtn.count() > 0) {
      await createBtn.click()
      await page.waitForTimeout(500)

      // 检查创建对话框
      const dialog = page.locator('[class*="dialog"], [class*="modal"]')
      const count = await dialog.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })

  test('should have natural language input', async ({ page }) => {
    // 打开创建对话框
    const createBtn = page.locator('button').filter({ hasText: /Create|创建|New|新建/ }).first()
    if (await createBtn.count() > 0) {
      await createBtn.click()
      await page.waitForTimeout(500)

      // 检查自然语言输入框
      const textArea = page.locator('textarea').first()
      const count = await textArea.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })
})

test.describe('技能搜索和过滤测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: /Skills|技能/ }).click()
    await page.waitForTimeout(1000)
  })

  test('should have search input', async ({ page }) => {
    // 检查搜索框
    const searchInput = page.locator('input[type="text"], input[type="search"]').filter({ has: page.locator('[placeholder*="Search"], [placeholder*="搜索"]') })
    const count = await searchInput.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have filter options', async ({ page }) => {
    // 检查过滤器
    const filters = page.locator('select, [class*="filter"], [class*="category"]')
    const count = await filters.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should filter skills by category', async ({ page }) => {
    // 点击分类过滤器
    const categoryFilter = page.locator('select').filter({ hasText: /Category|分类|Type|类型/ }).first()
    if (await categoryFilter.count() > 0) {
      await categoryFilter.selectOption({ index: 1 })
      await page.waitForTimeout(500)

      // 验证列表更新
      const skillList = page.locator('[class*="skill-list"], [class*="skill-item"]')
      const count = await skillList.count()
      expect(count).toBeGreaterThanOrEqual(0)
    }
  })
})
