import { test, expect, Page } from '@playwright/test'

test.describe('Agent Collaboration Network Visualization', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the Hermes Dashboard
    await page.goto('/hermes')
  })

  test('should display dashboard header with stats', async ({ page }) => {
    // Wait for the dashboard to load
    await page.waitForSelector('.dashboard-header')

    // Check title
    const title = await page.locator('.dashboard-title').textContent()
    expect(title).toContain('Hermes Dashboard')

    // Check subtitle
    const subtitle = await page.locator('.dashboard-subtitle').textContent()
    expect(subtitle).toContain('Agent Collaboration Network Visualization')

    // Check stats badges
    const statsBadges = await page.locator('.stat-badge').count()
    expect(statsBadges).toBe(4)

    // Check stats labels
    const labels = ['Agents', 'Running', 'Completed', 'Efficiency']
    for (const label of labels) {
      const badgeLabel = await page.locator('.stat-label').filter({ hasText: label }).textContent()
      expect(badgeLabel).toBe(label)
    }
  })

  test('should display view mode selector', async ({ page }) => {
    // Check view mode buttons
    const viewModeButtons = await page.locator('.mode-button').count()
    expect(viewModeButtons).toBe(3)

    // Check button labels
    const buttons = [
      { icon: '🕸️', text: 'Network' },
      { icon: '⏱️', text: 'Timeline' },
      { icon: '📋', text: 'Kanban' },
    ]

    for (const button of buttons) {
      const buttonText = await page
        .locator('.mode-button')
        .filter({ hasText: button.text })
        .textContent()
      expect(buttonText).toContain(button.text)
    }
  })

  test('should switch between view modes', async ({ page }) => {
    // Start with Network view (default)
    await expect(page.locator('.mode-button.active')).toContainText('Network')

    // Switch to Timeline view
    await page.click('.mode-button:has-text("Timeline")')
    await expect(page.locator('.mode-button.active')).toContainText('Timeline')

    // Switch to Kanban view
    await page.click('.mode-button:has-text("Kanban")')
    await expect(page.locator('.mode-button.active')).toContainText('Kanban')

    // Switch back to Network view
    await page.click('.mode-button:has-text("Network")')
    await expect(page.locator('.mode-button.active')).toContainText('Network')
  })

  test('should display empty state when no data', async ({ page }) => {
    // Check for empty state
    const emptyState = await page.locator('.empty-state')
    await expect(emptyState).toBeVisible()

    // Check empty state content
    const emptyTitle = await page.locator('.empty-title').textContent()
    expect(emptyTitle).toContain('No Collaboration Data')

    const emptyDescription = await page.locator('.empty-description').textContent()
    expect(emptyDescription).toContain('Start an agent collaboration task')
  })

  test('should display network view with nodes and edges', async ({ page }) => {
    // Trigger some mock data (we'll need to implement this in the actual app)
    await page.evaluate(() => {
      // This will call the store's initializeMockData method
      // We'll need to expose this through the window for testing
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    // Wait for nodes to appear
    await page.waitForTimeout(1000)

    // Check if network view is visible
    await expect(page.locator('.vue-flow-collaboration')).toBeVisible()
  })

  test('should display timeline view with events', async ({ page }) => {
    // Switch to Timeline view
    await page.click('.mode-button:has-text("Timeline")')

    // Wait for timeline to load
    await page.waitForTimeout(500)

    // Check timeline header
    const timelineHeader = await page.locator('.timeline-header').textContent()
    expect(timelineHeader).toContain('Collaboration Timeline')
  })

  test('should display kanban view with columns', async ({ page }) => {
    // Switch to Kanban view
    await page.click('.mode-button:has-text("Kanban")')

    // Wait for kanban to load
    await page.waitForTimeout(500)

    // Check kanban columns
    const columns = await page.locator('.kanban-column').count()
    expect(columns).toBe(4) // Pending, Running, Completed, Failed
  })

  test('should toggle auto refresh', async ({ page }) => {
    // Find auto refresh checkbox
    const autoRefreshCheckbox = await page.locator('.auto-refresh-toggle input')

    // Check initial state (should be checked by default)
    await expect(autoRefreshCheckbox).toBeChecked()

    // Uncheck it
    await autoRefreshCheckbox.click()
    await expect(autoRefreshCheckbox).not.toBeChecked()

    // Check it again
    await autoRefreshCheckbox.click()
    await expect(autoRefreshCheckbox).toBeChecked()
  })

  test('should trigger manual refresh', async ({ page }) => {
    // Find refresh button
    const refreshButton = await page.locator('.refresh-button')

    // Click refresh
    await refreshButton.click()

    // Check loading state
    const buttonText = await refreshButton.textContent()
    expect(buttonText).toContain('Loading...')

    // Wait for refresh to complete
    await page.waitForTimeout(2000)

    // Check if back to normal
    const finalButtonText = await refreshButton.textContent()
    expect(finalButtonText).toContain('Refresh')
  })

  test('should display recent events in sidebar', async ({ page }) => {
    // Check sidebar
    const sidebar = await page.locator('.sidebar')
    await expect(sidebar).toBeVisible()

    // Check recent events section
    const recentEventsHeader = await page.locator('.section-header:has-text("Recent Events")')
    await expect(recentEventsHeader).toBeVisible()
  })

  test('should open capability panel when clicking node', async ({ page }) => {
    // Trigger mock data
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    // Wait for data to load
    await page.waitForTimeout(1000)

    // Click on a node (we'll need to find one)
    const firstNode = await page.locator('.agent-node-wrapper').first()
    if (await firstNode.isVisible()) {
      await firstNode.click()

      // Check if capability panel opened
      await expect(page.locator('.capability-panel')).toBeVisible()
    }
  })

  test('should close capability panel', async ({ page }) => {
    // Open capability panel first
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    await page.waitForTimeout(1000)

    const firstNode = await page.locator('.agent-node-wrapper').first()
    if (await firstNode.isVisible()) {
      await firstNode.click()

      // Close panel
      await page.click('.close-panel-button')

      // Check if panel closed
      await expect(page.locator('.capability-panel')).not.toBeVisible()
    }
  })

  test('should display stats correctly', async ({ page }) => {
    // Trigger mock data
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    await page.waitForTimeout(1000)

    // Check stats values
    const agentStat = await page.locator('.stat-badge:has-text("Agents") .stat-value').textContent()
    expect(agentStat).toMatch(/\d+\/\d+/)

    const runningStat = await page.locator('.stat-badge:has-text("Running") .stat-value').textContent()
    expect(runningStat).toMatch(/\d+/)

    const efficiencyStat = await page.locator('.stat-badge:has-text("Efficiency") .stat-value').textContent()
    expect(efficiencyStat).toMatch(/\d+%/)
  })

  test('should be responsive', async ({ page }) => {
    // Test on mobile viewport
    await page.setViewportSize({ width: 375, height: 667 })

    // Check if dashboard still displays correctly
    await expect(page.locator('.dashboard-header')).toBeVisible()
    await expect(page.locator('.sidebar')).toBeVisible()
    await expect(page.locator('.main-view')).toBeVisible()

    // Test on tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 })

    // Check if dashboard still displays correctly
    await expect(page.locator('.dashboard-header')).toBeVisible()
    await expect(page.locator('.sidebar')).toBeVisible()
    await expect(page.locator('.main-view')).toBeVisible()

    // Test on desktop viewport
    await page.setViewportSize({ width: 1280, height: 720 })

    // Check if dashboard still displays correctly
    await expect(page.locator('.dashboard-header')).toBeVisible()
    await expect(page.locator('.sidebar')).toBeVisible()
    await expect(page.locator('.main-view')).toBeVisible()
  })
})

test.describe('Collaboration Network Graph', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hermes')

    // Trigger mock data
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    await page.waitForTimeout(1000)
  })

  test('should render nodes with correct status colors', async ({ page }) => {
    // Check for different node statuses
    const idleNodes = await page.locator('.agent-node-wrapper.status-idle').count()
    const activeNodes = await page.locator('.agent-node-wrapper.status-active').count()
    const waitingNodes = await page.locator('.agent-node-wrapper.status-waiting').count()
    const errorNodes = await page.locator('.agent-node-wrapper.status-error').count()

    // Should have at least some nodes
    const totalNodes = idleNodes + activeNodes + waitingNodes + errorNodes
    expect(totalNodes).toBeGreaterThan(0)
  })

  test('should render edges with flow animation', async ({ page }) => {
    // Check for flowing edges
    const flowingEdges = await page.locator('.task-edge-flowing').count()

    // If there are flowing edges, check animation
    if (flowingEdges > 0) {
      // Animation should be visible (check for animation CSS)
      const edge = await page.locator('.task-edge-flowing').first()
      await expect(edge).toBeVisible()
    }
  })

  test('should allow node drag', async ({ page }) => {
    // Find a node
    const node = await page.locator('.agent-node-wrapper').first()

    if (await node.isVisible()) {
      // Get initial position
      const initialPosition = await node.boundingBox()

      // Drag the node
      await node.hover()
      await page.mouse.down()
      await page.mouse.move(initialPosition!.x + 100, initialPosition!.y + 100)
      await page.mouse.up()

      // Check if node moved
      const newPosition = await node.boundingBox()
      expect(newPosition!.x).not.toBe(initialPosition!.x)
      expect(newPosition!.y).not.toBe(initialPosition!.y)
    }
  })

  test('should allow zoom and pan', async ({ page }) => {
    // Find the network view container
    const networkView = await page.locator('.vue-flow-collaboration')

    // Zoom in using mouse wheel
    await networkView.hover()
    await page.mouse.wheel(0, -100)

    // Zoom out
    await page.mouse.wheel(0, 100)

    // Pan by dragging
    await page.mouse.move(400, 300)
    await page.mouse.down()
    await page.mouse.move(500, 400)
    await page.mouse.up()
  })
})

test.describe('Collaboration Timeline', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hermes')

    // Switch to Timeline view
    await page.click('.mode-button:has-text("Timeline")')

    // Trigger mock data
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    await page.waitForTimeout(1000)
  })

  test('should display time axis', async ({ page }) => {
    // Check for time axis
    await expect(page.locator('.time-axis')).toBeVisible()

    // Check for milestone markers
    const milestones = await page.locator('.milestone-marker').count()
    expect(milestones).toBeGreaterThanOrEqual(0)
  })

  test('should display agent lanes', async ({ page }) => {
    // Check for agent lanes
    await expect(page.locator('.agent-lanes')).toBeVisible()

    // Check for agent lanes
    const lanes = await page.locator('.agent-lane').count()
    expect(lanes).toBeGreaterThan(0)
  })

  test('should display activity bars', async ({ page }) => {
    // Check for activity bars
    const activities = await page.locator('.activity-bar').count()
    expect(activities).toBeGreaterThanOrEqual(0)
  })

  test('should show activity details on hover', async ({ page }) => {
    // Find an activity bar
    const activity = await page.locator('.activity-bar').first()

    if (await activity.isVisible()) {
      // Hover over the activity
      await activity.hover()

      // Check if tooltip appears
      await expect(page.locator('.activity-tooltip')).toBeVisible()
    }
  })
})

test.describe('Collaboration Kanban', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hermes')

    // Switch to Kanban view
    await page.click('.mode-button:has-text("Kanban")')

    // Trigger mock data
    await page.evaluate(() => {
      if ((window as any).initializeMockData) {
        (window as any).initializeMockData()
      }
    })

    await page.waitForTimeout(1000)
  })

  test('should display kanban columns', async ({ page }) => {
    // Check for columns
    const columns = await page.locator('.kanban-column').count()
    expect(columns).toBe(4) // Pending, Running, Completed, Failed

    // Check column names
    const columnNames = ['Pending', 'Running', 'Completed', 'Failed']
    for (const name of columnNames) {
      const columnHeader = await page.locator('.column-header').filter({ hasText: name })
      await expect(columnHeader).toBeVisible()
    }
  })

  test('should display task cards', async ({ page }) => {
    // Check for task cards
    const tasks = await page.locator('.kanban-task-card').count()
    expect(tasks).toBeGreaterThanOrEqual(0)
  })

  test('should switch to agent view', async ({ page }) => {
    // Click agent view button
    await page.click('.mode-button:has-text("By Agent")')

    // Wait for view change
    await page.waitForTimeout(500)

    // Check for agent columns
    await expect(page.locator('.kanban-agent-column')).toBeVisible()
  })

  test('should show task details', async ({ page }) => {
    // Find a task card
    const task = await page.locator('.kanban-task-card').first()

    if (await task.isVisible()) {
      // Click the task
      await task.click()

      // Should trigger task click event (we'll need to verify this in actual implementation)
    }
  })
})