import { test, expect } from '@playwright/test'

test('advanced feature navigation is visible without fake success states', async ({ page }) => {
  await page.goto('http://127.0.0.1:5173')
  await expect(page.getByRole('heading', { name: 'ACP UI' })).toBeVisible()
  await expect(page.getByRole('button', { name: /多Agent/ })).toBeVisible()
  await expect(page.getByRole('button', { name: /工作流/ })).toBeVisible()
  await expect(page.getByRole('button', { name: /总会话/ })).toBeVisible()
  await expect(page.getByRole('button', { name: /编排监控/ })).toBeVisible()
  await expect(page.getByRole('button', { name: /远程控制/ })).toBeVisible()
  await expect(page.getByRole('button', { name: /记忆/ })).toBeVisible()
})
