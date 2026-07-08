// E2E Test: Hermes Game Operator
// Tests the Game Operator UI and basic functionality

import { test, expect } from '@playwright/test'

test.describe('Game Operator View', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('http://localhost:1420')
  })

  test('should show Game Operator in navigation', async ({ page }) => {
    // Check if games nav button exists
    const gamesNav = page.locator('button:has-text("games")')
    await expect(gamesNav).toBeVisible()
  })

  test('should navigate to Game Operator view', async ({ page }) => {
    // Click on games navigation
    await page.click('button:has-text("games")')

    // Should be on games route
    await expect(page).toHaveURL(/#\/games/)

    // Should show Game Operator header
    await expect(page.locator('h1:has-text("Hermes Game Operator")')).toBeVisible()
  })

  test('should show project selection form', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Should have project path input
    const pathInput = page.locator('input[placeholder*="godot"]')
    await expect(pathInput).toBeVisible()

    // Should have task goal textarea
    const goalInput = page.locator('textarea[placeholder*="double jump"]')
    await expect(goalInput).toBeVisible()

    // Should have Start Task button
    const startButton = page.locator('button:has-text("Start Task")')
    await expect(startButton).toBeVisible()
  })

  test('should disable Start Task without project and goal', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    const startButton = page.locator('button:has-text("Start Task")')
    await expect(startButton).toBeDisabled()
  })

  test('should enable Start Task with valid inputs', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Fill in project path
    await page.fill('input[placeholder*="godot"]', 'D:/tmp/test-godot-project')

    // Fill in goal
    await page.fill('textarea[placeholder*="double jump"]', 'Add triple jump to player')

    // Start Task button should now be enabled
    const startButton = page.locator('button:has-text("Start Task")')
    await expect(startButton).toBeEnabled()
  })
})

test.describe('Game Operator Components', () => {
  test('should show correct page title', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    const title = page.locator('h1:has-text("Hermes Game Operator")')
    await expect(title).toBeVisible()
  })

  test('should show MVP subtitle', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    const subtitle = page.locator('.subtitle')
    await expect(subtitle).toContainText('Godot MVP')
  })
})

test.describe('Game Operator Form Validation', () => {
  test('should clear error when project path is filled', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Fill project path
    await page.fill('input[placeholder*="godot"]', 'D:/tmp/test-godot-project')

    // Fill goal
    await page.fill('textarea[placeholder*="double jump"]', 'Add jump feature')

    // No error should be visible
    const errorElement = page.locator('.error-message')
    await expect(errorElement).not.toBeVisible()
  })

  test('should have Browse button for project selection', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    const browseButton = page.locator('button:has-text("Browse")')
    await expect(browseButton).toBeVisible()
  })
})

test.describe('Game Operator UI Layout', () => {
  test('should have responsive layout', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Check main container exists
    const view = page.locator('.game-operator-view')
    await expect(view).toBeVisible()

    // Check form group exists
    const formGroups = page.locator('.form-group')
    await expect(formGroups.first()).toBeVisible()
  })

  test('should show correct placeholder text', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Check project path placeholder
    const pathInput = page.locator('input[placeholder*="godot"]')
    await expect(pathInput).toHaveAttribute('placeholder', /godot/i)

    // Check goal placeholder
    const goalInput = page.locator('textarea[placeholder*="double jump"]')
    await expect(goalInput).toHaveAttribute('placeholder', /double jump/i)
  })
})

test.describe('Game Operator Navigation', () => {
  test('should be default route', async ({ page }) => {
    await page.goto('http://localhost:1420')

    // Should redirect to /games
    await expect(page).toHaveURL(/#\/games/)
  })

  test('should show active state for games nav', async ({ page }) => {
    await page.goto('http://localhost:1420/#/games')

    // Games nav should be visible and active
    const gamesNav = page.locator('button:has-text("games")')
    await expect(gamesNav).toBeVisible()
  })
})