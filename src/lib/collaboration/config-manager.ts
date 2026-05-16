import { ref } from 'vue'
import type { CollaborationNetworkConfig } from './types'

/**
 * Collaboration Config Manager
 *
 * Manages configuration for collaboration network visualization
 */
export class CollaborationConfigManager {
  private config = ref<CollaborationNetworkConfig>({
    layoutAlgorithm: 'force-directed',
    animationEnabled: true,
    animationSpeed: 0.5,
    nodeSize: 'medium',
    edgeStyle: 'curved',
    showCapabilities: true,
    showProtocols: true,
    showMetrics: true,
    autoRefresh: true,
    refreshInterval: 2000,
  })

  private storageKey = 'collaboration-config'

  /**
   * Get current config
   */
  getConfig(): CollaborationNetworkConfig {
    return this.config.value
  }

  /**
   * Update config
   */
  updateConfig(newConfig: Partial<CollaborationNetworkConfig>): void {
    Object.assign(this.config.value, newConfig)
    this.saveConfig()
  }

  /**
   * Reset config to default
   */
  resetConfig(): void {
    this.config.value = {
      layoutAlgorithm: 'force-directed',
      animationEnabled: true,
      animationSpeed: 0.5,
      nodeSize: 'medium',
      edgeStyle: 'curved',
      showCapabilities: true,
      showProtocols: true,
      showMetrics: true,
      autoRefresh: true,
      refreshInterval: 2000,
    }
    this.saveConfig()
  }

  /**
   * Load config from storage
   */
  loadConfig(): void {
    try {
      const savedConfig = localStorage.getItem(this.storageKey)
      if (savedConfig) {
        const parsed = JSON.parse(savedConfig)
        Object.assign(this.config.value, parsed)
      }
    } catch (error) {
      console.error('Failed to load collaboration config:', error)
    }
  }

  /**
   * Save config to storage
   */
  saveConfig(): void {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(this.config.value))
    } catch (error) {
      console.error('Failed to save collaboration config:', error)
    }
  }

  /**
   * Layout algorithms
   */
  getLayoutAlgorithms() {
    return [
      { id: 'force-directed', name: 'Force Directed', description: 'Nodes push each other away' },
      { id: 'hierarchical', name: 'Hierarchical', description: 'Top-down tree layout' },
      { id: 'circular', name: 'Circular', description: 'Nodes in a circle' },
      { id: 'grid', name: 'Grid', description: 'Nodes in a grid pattern' },
    ]
  }

  /**
   * Node sizes
   */
  getNodeSizes() {
    return [
      { id: 'small', name: 'Small', width: 120, height: 80 },
      { id: 'medium', name: 'Medium', width: 180, height: 120 },
      { id: 'large', name: 'Large', width: 250, height: 160 },
    ]
  }

  /**
   * Edge styles
   */
  getEdgeStyles() {
    return [
      { id: 'straight', name: 'Straight', description: 'Direct line between nodes' },
      { id: 'curved', name: 'Curved', description: 'Smooth curve with bezier' },
      { id: 'orthogonal', name: 'Orthogonal', description: 'Right-angle path' },
    ]
  }

  /**
   * Animation speeds
   */
  getAnimationSpeeds() {
    return [
      { id: 'slow', name: 'Slow', value: 0.25 },
      { id: 'normal', name: 'Normal', value: 0.5 },
      { id: 'fast', name: 'Fast', value: 1.0 },
      { id: 'instant', name: 'Instant', value: 2.0 },
    ]
  }

  /**
   * Refresh intervals
   */
  getRefreshIntervals() {
    return [
      { id: 'fast', name: 'Fast (1s)', value: 1000 },
      { id: 'normal', name: 'Normal (2s)', value: 2000 },
      { id: 'slow', name: 'Slow (5s)', value: 5000 },
      { id: 'manual', name: 'Manual', value: 0 },
    ]
  }
}

/**
 * Singleton instance
 */
export const collaborationConfigManager = new CollaborationConfigManager()