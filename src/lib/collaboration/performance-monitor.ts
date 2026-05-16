import { ref, computed, watch } from 'vue'

/**
 * Collaboration Performance Monitor
 *
 * Monitors performance metrics for collaboration network
 */
export class CollaborationPerformanceMonitor {
  private metrics = ref({
    // Render metrics
    renderTime: 0,
    nodeCount: 0,
    edgeCount: 0,
    eventCount: 0,

    // Animation metrics
    animationFPS: 60,
    activeAnimations: 0,

    // Network metrics
    networkLatency: 0,
    dataTransferRate: 0,

    // Interaction metrics
    interactionCount: 0,
    avgInteractionTime: 0,

    // Error metrics
    errorCount: 0,
    errorRate: 0,

    // Timestamps
    startTime: Date.now(),
    lastUpdateTime: Date.now(),
  })

  private performanceHistory: Array<{
    timestamp: number
    metrics: any
  }> = []

  private maxHistorySize = 100

  /**
   * Get current metrics
   */
  getMetrics() {
    return this.metrics.value
  }

  /**
   * Update metrics
   */
  updateMetrics(newMetrics: Partial<typeof this.metrics.value>): void {
    Object.assign(this.metrics.value, newMetrics)
    this.metrics.value.lastUpdateTime = Date.now()

    // Add to history
    this.addToHistory()
  }

  /**
   * Add to history
   */
  private addToHistory(): void {
    this.performanceHistory.push({
      timestamp: Date.now(),
      metrics: { ...this.metrics.value },
    })

    // Limit history size
    if (this.performanceHistory.length > this.maxHistorySize) {
      this.performanceHistory.shift()
    }
  }

  /**
   * Get performance history
   */
  getHistory() {
    return this.performanceHistory
  }

  /**
   * Calculate performance score
   */
  calculatePerformanceScore(): number {
    const metrics = this.metrics.value

    // Base score (100)
    let score = 100

    // Deduct for render time (> 16ms = less than 60fps)
    if (metrics.renderTime > 16) {
      score -= Math.min((metrics.renderTime - 16) * 2, 20)
    }

    // Deduct for large node count (> 50)
    if (metrics.nodeCount > 50) {
      score -= Math.min((metrics.nodeCount - 50) * 0.5, 15)
    }

    // Deduct for large edge count (> 100)
    if (metrics.edgeCount > 100) {
      score -= Math.min((metrics.edgeCount - 100) * 0.3, 10)
    }

    // Deduct for low FPS (< 60)
    if (metrics.animationFPS < 60) {
      score -= Math.min((60 - metrics.animationFPS) * 1, 25)
    }

    // Deduct for high error rate
    score -= metrics.errorRate * 5

    // Bonus for high interaction count (active usage)
    if (metrics.interactionCount > 10) {
      score += Math.min(metrics.interactionCount * 0.1, 5)
    }

    return Math.max(0, Math.min(100, score))
  }

  /**
   * Get performance status
   */
  getPerformanceStatus(): 'excellent' | 'good' | 'fair' | 'poor' {
    const score = this.calculatePerformanceScore()

    if (score >= 90) return 'excellent'
    if (score >= 70) return 'good'
    if (score >= 50) return 'fair'
    return 'poor'
  }

  /**
   * Get performance recommendations
   */
  getRecommendations(): string[] {
    const metrics = this.metrics.value
    const recommendations: string[] = []

    if (metrics.renderTime > 16) {
      recommendations.push('Consider reducing node count or disabling animations')
    }

    if (metrics.nodeCount > 50) {
      recommendations.push('Large number of nodes may impact performance. Consider filtering or grouping')
    }

    if (metrics.edgeCount > 100) {
      recommendations.push('Too many edges may clutter the view. Consider simplifying the network')
    }

    if (metrics.animationFPS < 60) {
      recommendations.push('Animation performance is below optimal. Consider disabling animations')
    }

    if (metrics.errorRate > 0.1) {
      recommendations.push('High error rate detected. Check network connectivity and data quality')
    }

    if (recommendations.length === 0) {
      recommendations.push('Performance is optimal. No recommendations needed')
    }

    return recommendations
  }

  /**
   * Start monitoring render time
   */
  startRenderTimer(): number {
    return performance.now()
  }

  /**
   * End monitoring render time
   */
  endRenderTimer(startTime: number): void {
    const renderTime = performance.now() - startTime
    this.updateMetrics({ renderTime })
  }

  /**
   * Monitor animation FPS
   */
  monitorFPS(): void {
    let lastTime = performance.now()
    let frames = 0

    const measureFPS = () => {
      frames++
      const currentTime = performance.now()

      if (currentTime - lastTime >= 1000) {
        const fps = Math.round((frames * 1000) / (currentTime - lastTime))
        this.updateMetrics({ animationFPS: fps })

        frames = 0
        lastTime = currentTime
      }

      requestAnimationFrame(measureFPS)
    }

    requestAnimationFrame(measureFPS)
  }

  /**
   * Track interaction
   */
  trackInteraction(duration: number): void {
    const metrics = this.metrics.value
    const totalInteractions = metrics.interactionCount + 1
    const totalTime = metrics.avgInteractionTime * metrics.interactionCount + duration

    this.updateMetrics({
      interactionCount: totalInteractions,
      avgInteractionTime: totalTime / totalInteractions,
    })
  }

  /**
   * Track error
   */
  trackError(): void {
    const metrics = this.metrics.value
    const errorCount = metrics.errorCount + 1
    const timeSinceStart = Date.now() - metrics.startTime
    const errorRate = errorCount / (timeSinceStart / 1000)

    this.updateMetrics({
      errorCount,
      errorRate,
    })
  }

  /**
   * Reset metrics
   */
  resetMetrics(): void {
    this.metrics.value = {
      renderTime: 0,
      nodeCount: 0,
      edgeCount: 0,
      eventCount: 0,
      animationFPS: 60,
      activeAnimations: 0,
      networkLatency: 0,
      dataTransferRate: 0,
      interactionCount: 0,
      avgInteractionTime: 0,
      errorCount: 0,
      errorRate: 0,
      startTime: Date.now(),
      lastUpdateTime: Date.now(),
    }
    this.performanceHistory = []
  }

  /**
   * Export metrics as JSON
   */
  exportMetrics(): string {
    return JSON.stringify({
      current: this.metrics.value,
      history: this.performanceHistory,
      score: this.calculatePerformanceScore(),
      status: this.getPerformanceStatus(),
      recommendations: this.getRecommendations(),
    }, null, 2)
  }

  /**
   * Get performance summary
   */
  getSummary(): string {
    const metrics = this.metrics.value
    const score = this.calculatePerformanceScore()
    const status = this.getPerformanceStatus()

    return `
Performance Summary:
- Score: ${score}/100 (${status})
- Nodes: ${metrics.nodeCount}
- Edges: ${metrics.edgeCount}
- Events: ${metrics.eventCount}
- Render Time: ${metrics.renderTime.toFixed(2)}ms
- FPS: ${metrics.animationFPS}
- Errors: ${metrics.errorCount} (${metrics.errorRate.toFixed(2)} rate)
- Interactions: ${metrics.interactionCount}
    `.trim()
  }
}

/**
 * Singleton instance
 */
export const collaborationPerformanceMonitor = new CollaborationPerformanceMonitor()