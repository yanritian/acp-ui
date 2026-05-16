import { ref, onMounted, onUnmounted } from 'vue'
import { useCollaborationStore } from '@/stores/collaboration'

/**
 * Collaboration Keyboard Shortcuts
 *
 * Provides keyboard shortcuts for collaboration network visualization
 */
export class CollaborationKeyboardShortcuts {
  private enabled = ref(true)
  private shortcuts = ref([
    {
      key: 'n',
      ctrl: false,
      action: 'toggle-network',
      description: 'Switch to Network View',
    },
    {
      key: 't',
      ctrl: false,
      action: 'toggle-timeline',
      description: 'Switch to Timeline View',
    },
    {
      key: 'k',
      ctrl: false,
      action: 'toggle-kanban',
      description: 'Switch to Kanban View',
    },
    {
      key: 'r',
      ctrl: false,
      action: 'refresh',
      description: 'Refresh Data',
    },
    {
      key: 'a',
      ctrl: false,
      action: 'toggle-auto-refresh',
      description: 'Toggle Auto Refresh',
    },
    {
      key: 'c',
      ctrl: false,
      action: 'toggle-capabilities',
      description: 'Toggle Capabilities Panel',
    },
    {
      key: 'f',
      ctrl: false,
      action: 'toggle-fullscreen',
      description: 'Toggle Fullscreen',
    },
    {
      key: 'h',
      ctrl: false,
      action: 'toggle-help',
      description: 'Toggle Help',
    },
    {
      key: '?',
      ctrl: false,
      action: 'toggle-help',
      description: 'Toggle Help',
    },
    {
      key: 'Escape',
      ctrl: false,
      action: 'close-panel',
      description: 'Close Current Panel',
    },
    {
      key: '+',
      ctrl: true,
      action: 'zoom-in',
      description: 'Zoom In',
    },
    {
      key: '-',
      ctrl: true,
      action: 'zoom-out',
      description: 'Zoom Out',
    },
    {
      key: '0',
      ctrl: true,
      action: 'reset-zoom',
      description: 'Reset Zoom',
    },
    {
      key: 's',
      ctrl: true,
      action: 'save-config',
      description: 'Save Configuration',
    },
    {
      key: 'l',
      ctrl: true,
      action: 'load-config',
      description: 'Load Configuration',
    },
  ])

  private store = useCollaborationStore()
  private handlers: Map<string, () => void> = new Map()
  private helpVisible = ref(false)

  /**
   * Enable shortcuts
   */
  enable(): void {
    this.enabled.value = true
  }

  /**
   * Disable shortcuts
   */
  disable(): void {
    this.enabled.value = false
  }

  /**
   * Toggle shortcuts
   */
  toggle(): void {
    this.enabled.value = !this.enabled.value
  }

  /**
   * Get shortcuts list
   */
  getShortcuts() {
    return this.shortcuts.value
  }

  /**
   * Register handler for action
   */
  registerHandler(action: string, handler: () => void): void {
    this.handlers.set(action, handler)
  }

  /**
   * Unregister handler
   */
  unregisterHandler(action: string): void {
    this.handlers.delete(action)
  }

  /**
   * Execute action
   */
  private executeAction(action: string): void {
    const handler = this.handlers.get(action)
    if (handler) {
      handler()
      return
    }

    // Default handlers
    switch (action) {
      case 'toggle-network':
        this.store.setViewMode('network')
        break

      case 'toggle-timeline':
        this.store.setViewMode('timeline')
        break

      case 'toggle-kanban':
        this.store.setViewMode('kanban')
        break

      case 'refresh':
        this.refresh()
        break

      case 'toggle-auto-refresh':
        this.toggleAutoRefresh()
        break

      case 'toggle-capabilities':
        this.toggleCapabilities()
        break

      case 'toggle-fullscreen':
        this.toggleFullscreen()
        break

      case 'toggle-help':
        this.toggleHelp()
        break

      case 'close-panel':
        this.closePanel()
        break

      case 'zoom-in':
        this.zoomIn()
        break

      case 'zoom-out':
        this.zoomOut()
        break

      case 'reset-zoom':
        this.resetZoom()
        break

      case 'save-config':
        this.saveConfig()
        break

      case 'load-config':
        this.loadConfig()
        break
    }
  }

  /**
   * Handle keyboard event
   */
  private handleKeyDown(event: KeyboardEvent): void {
    if (!this.enabled.value) return

    // Ignore when typing in input fields
    const target = event.target as HTMLElement
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return

    // Find matching shortcut
    const matchingShortcut = this.shortcuts.value.find(shortcut => {
      const keyMatch = shortcut.key.toLowerCase() === event.key.toLowerCase()
      const ctrlMatch = shortcut.ctrl === (event.ctrlKey || event.metaKey)
      return keyMatch && ctrlMatch
    })

    if (matchingShortcut) {
      event.preventDefault()
      this.executeAction(matchingShortcut.action)
    }
  }

  /**
   * Start listening
   */
  startListening(): void {
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', this.handleKeyDown.bind(this))
    }
  }

  /**
   * Stop listening
   */
  stopListening(): void {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', this.handleKeyDown.bind(this))
    }
  }

  /**
   * Default actions
   */
  private refresh(): void {
    this.store.setLoading(true)
    setTimeout(() => this.store.setLoading(false), 1000)
  }

  private toggleAutoRefresh(): void {
    const config = this.store.config
    this.store.updateConfig({
      autoRefresh: !config.autoRefresh,
    })
  }

  private toggleCapabilities(): void {
    const config = this.store.config
    this.store.updateConfig({
      showCapabilities: !config.showCapabilities,
    })
  }

  private toggleFullscreen(): void {
    if (typeof document !== 'undefined') {
      if (document.fullscreenElement) {
        document.exitFullscreen()
      } else {
        document.documentElement.requestFullscreen()
      }
    }
  }

  private toggleHelp(): void {
    this.helpVisible.value = !this.helpVisible.value
  }

  private closePanel(): void {
    this.store.selectNode(null)
    this.store.selectEdge(null)
    this.helpVisible.value = false
  }

  private zoomIn(): void {
    // Zoom in by 10%
    console.log('Zoom in')
  }

  private zoomOut(): void {
    // Zoom out by 10%
    console.log('Zoom out')
  }

  private resetZoom(): void {
    // Reset zoom to 100%
    console.log('Reset zoom')
  }

  private saveConfig(): void {
    localStorage.setItem('collaboration-config', JSON.stringify(this.store.config))
  }

  private loadConfig(): void {
    const savedConfig = localStorage.getItem('collaboration-config')
    if (savedConfig) {
      this.store.updateConfig(JSON.parse(savedConfig))
    }
  }

  /**
   * Get help visibility
   */
  isHelpVisible(): boolean {
    return this.helpVisible.value
  }

  /**
   * Get help content
   */
  getHelpContent(): string {
    const shortcuts = this.shortcuts.value
    const helpLines = shortcuts.map(s => {
      const keyDisplay = s.ctrl ? `Ctrl+${s.key}` : s.key
      return `${keyDisplay}: ${s.description}`
    })

    return `
Keyboard Shortcuts for Collaboration Network:

${helpLines.join('\n')}

Press Escape to close this help panel.
    `.trim()
  }

  /**
   * Lifecycle hooks
   */
  mount(): void {
    this.startListening()
  }

  unmount(): void {
    this.stopListening()
  }
}

/**
 * Create shortcuts instance
 */
export function useCollaborationShortcuts() {
  const shortcuts = new CollaborationKeyboardShortcuts()

  onMounted(() => {
    shortcuts.mount()
  })

  onUnmounted(() => {
    shortcuts.unmount()
  })

  return shortcuts
}