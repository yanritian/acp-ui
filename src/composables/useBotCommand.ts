import { onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useConfigStore } from '../stores/config'
import { useTeamRuntimeStore } from '../stores/team-runtime'
import { trackBehavior } from '../lib/self-improvement'
import { isDesktop } from '../lib/platform'

// Extend Window interface for bot command cleanup
declare global {
  interface Window {
    _unlistenBotCommand?: () => void;
  }
}

/**
 * Composable for handling bot commands from Telegram/Feishu/App WebSocket.
 * Only active in Tauri desktop environment.
 */
export function useBotCommand() {
  const configStore = useConfigStore()
  const teamRuntime = useTeamRuntimeStore()

  // Handle bot commands from Telegram/Feishu/App WebSocket
  async function handleBotCommand(payload: Record<string, unknown>) {
    const type = payload.type as string
    const prompt = payload.prompt as string

    console.log('[Bot Command Received]', payload)

    const cwd = configStore.getDefaultCwd()

    if (type === 'agent') {
      // Single agent task
      const agentName = (payload.agent_name as string) || configStore.agentNames[0] || 'Claude Code'
      await teamRuntime.runTeamTask({
        title: prompt.substring(0, 50),
        prompt,
        source: 'bot',
        routing: 'single',
        agents: [{ agentName, cwd }],
      })
      trackBehavior('bot-task-created', { type: 'agent', agentName })
    } else if (type === 'team') {
      // Multi-agent task
      const agentsPayload = payload.agents as string[] | undefined
      const routing = (payload.routing as string) || 'single'

      const agents = agentsPayload?.length
        ? agentsPayload.map(name => ({ agentName: name, cwd }))
        : [{ agentName: configStore.agentNames[0] || 'Claude Code', cwd }]

      await teamRuntime.runTeamTask({
        title: prompt.substring(0, 50),
        prompt,
        source: 'bot',
        routing: routing as 'single' | 'broadcast' | 'round-robin' | 'load-balanced',
        agents,
      })
      trackBehavior('bot-task-created', { type: 'team', agentCount: agents.length })
    }
  }

  onMounted(async () => {
    // Only listen for bot commands in Tauri desktop environment
    if (!isDesktop()) {
      return
    }

    try {
      // Listen for bot commands (from Telegram/Feishu/App WebSocket)
      const unlistenBot = await listen('bot-command', (event) => {
        handleBotCommand(event.payload as Record<string, unknown>)
      })

      // Store unlisten function for cleanup
      window._unlistenBotCommand = unlistenBot
    } catch (e) {
      console.warn('[BotCommand] Failed to setup listener:', e)
    }
  })

  onBeforeUnmount(() => {
    // Cleanup bot command listener
    if (window._unlistenBotCommand) {
      window._unlistenBotCommand()
      window._unlistenBotCommand = undefined
    }
  })
}
