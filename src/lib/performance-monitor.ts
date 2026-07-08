// Performance Monitor for Hermes Game Operator
// Tracks metrics for task execution and UI performance

interface PerformanceMetric {
  name: string
  startTime: number
  endTime?: number
  duration?: number
  metadata?: Record<string, unknown>
}

class PerformanceMonitor {
  private metrics: Map<string, PerformanceMetric> = new Map()
  private enabled: boolean = true

  enable(enabled: boolean): void {
    this.enabled = enabled
  }

  startMetric(name: string, metadata?: Record<string, unknown>): void {
    if (!this.enabled) return
    
    this.metrics.set(name, {
      name,
      startTime: performance.now(),
      metadata
    })
  }

  endMetric(name: string): number | null {
    if (!this.enabled) return null
    
    const metric = this.metrics.get(name)
    if (!metric) return null

    metric.endTime = performance.now()
    metric.duration = metric.endTime - metric.startTime

    return metric.duration
  }

  getMetric(name: string): PerformanceMetric | undefined {
    return this.metrics.get(name)
  }

  getAllMetrics(): PerformanceMetric[] {
    return Array.from(this.metrics.values())
  }

  clearMetrics(): void {
    this.metrics.clear()
  }

  getSummary(): {
    totalMetrics: number
    averageDuration: number
    slowestMetric: PerformanceMetric | null
  } {
    const metrics = this.getAllMetrics().filter(m => m.duration !== undefined)
    
    if (metrics.length === 0) {
      return { totalMetrics: 0, averageDuration: 0, slowestMetric: null }
    }

    const totalDuration = metrics.reduce((sum, m) => sum + (m.duration || 0), 0)
    const averageDuration = totalDuration / metrics.length
    const slowestMetric = metrics.reduce((prev, curr) => 
      (curr.duration || 0) > (prev.duration || 0) ? curr : prev
    )

    return { totalMetrics: metrics.length, averageDuration, slowestMetric }
  }

  // Task-specific metrics
  startTask(taskId: string): void {
    this.startMetric(`task_${taskId}`, { taskId })
  }

  endTask(taskId: string): number | null {
    return this.endMetric(`task_${taskId}`)
  }

  // API call metrics
  startApiCall(endpoint: string): void {
    this.startMetric(`api_${endpoint}`, { endpoint })
  }

  endApiCall(endpoint: string): number | null {
    return this.endMetric(`api_${endpoint}`)
  }

  // UI render metrics
  startRender(componentName: string): void {
    this.startMetric(`render_${componentName}`, { componentName })
  }

  endRender(componentName: string): number | null {
    return this.endMetric(`render_${componentName}`)
  }
}

// Export singleton instance
export const perfMonitor = new PerformanceMonitor()

// Export type
export type { PerformanceMetric }
