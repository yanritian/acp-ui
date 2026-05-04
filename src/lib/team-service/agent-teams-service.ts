import { useConfigStore } from '../../stores/config'
import { AcpSessionRunner } from '../agent-runtime/acp-session-runner'
import { getAppVersion } from '../../lib/host'
import type { RuntimeOutput, RuntimeTask } from '../agent-runtime/types'
import type { TeamAgentSelection, TeamRunRequest, TeamRunResult, TeamRoutingMode } from './types'

export class AgentTeamsService {
  private runners: Map<string, AcpSessionRunner> = new Map()

  async runTeamTask(request: TeamRunRequest): Promise<TeamRunResult> {
    const appVersion = await getAppVersion()
    const configStore = useConfigStore()

    const task: RuntimeTask = {
      id: crypto.randomUUID(),
      title: request.title,
      prompt: request.prompt,
      source: request.source,
      status: 'running',
      targetSessionIds: [],
      createdAt: Date.now(),
      startedAt: Date.now(),
    }

    const outputs: RuntimeOutput[] = []
    const errors: string[] = []

    // Validate all agents exist in config
    const validAgents: TeamAgentSelection[] = []
    for (const sel of request.agents) {
      if (!configStore.config.agents[sel.agentName]) {
        errors.push(`Agent '${sel.agentName}' not found in config`)
        continue
      }
      validAgents.push(sel)
    }

    if (validAgents.length === 0) {
      task.status = 'failed'
      task.completedAt = Date.now()
      task.error = errors.join('; ') || 'No valid agents selected'
      return { task, outputs, status: 'failed', error: task.error }
    }

    // Determine target agents based on routing mode
    const targets = this.selectTargets(validAgents, request.routing)

    // Create runners for each target
    for (const sel of targets) {
      const agentConfig = configStore.config.agents[sel.agentName]
      if (!agentConfig) continue

      const runner = new AcpSessionRunner({
        agentName: sel.agentName,
        agentConfig,
        cwd: sel.cwd,
        appVersion,
        onOutput: (output: RuntimeOutput) => {
          const idx = outputs.findIndex(o => o.sessionId === output.sessionId)
          if (idx >= 0) {
            outputs[idx] = output
          }
        },
      })

      this.runners.set(sel.agentName, runner)

      try {
        const runtimeSession = await runner.create()
        task.targetSessionIds.push(runtimeSession.id)
      } catch (e) {
        errors.push(`${sel.agentName}: ${e instanceof Error ? e.message : String(e)}`)
        this.runners.delete(sel.agentName)
      }
    }

    if (task.targetSessionIds.length === 0) {
      task.status = 'failed'
      task.completedAt = Date.now()
      task.error = errors.join('; ') || 'All agents failed to connect'
      return { task, outputs, status: 'failed', error: task.error }
    }

    // Execute tasks based on routing mode
    if (request.routing === 'broadcast') {
      await this.broadcastPrompt(targets, request.prompt, outputs, errors)
    } else {
      await this.singlePrompt(targets[0], request.prompt, outputs, errors)
    }

    // Determine final status
    const failedCount = outputs.filter(o => o.status === 'failed').length
    const completedCount = outputs.filter(o => o.status === 'completed').length

    if (failedCount > 0 && completedCount === 0) {
      task.status = 'failed'
      task.error = errors.join('; ')
    } else if (failedCount > 0) {
      task.status = 'completed'
      task.error = errors.join('; ')
    } else {
      task.status = 'completed'
    }

    task.completedAt = Date.now()
    return { task, outputs, status: task.status, error: task.error }
  }

  async cancelTask(_taskId: string): Promise<void> {
    for (const [, runner] of this.runners) {
      try {
        await runner.cancel()
      } catch {
        // ignore cancel errors
      }
    }
  }

  async disconnectAll(): Promise<void> {
    for (const [, runner] of this.runners) {
      try {
        await runner.disconnect()
      } catch {
        // ignore disconnect errors
      }
    }
    this.runners.clear()
  }

  private selectTargets(agents: TeamAgentSelection[], mode: TeamRoutingMode): TeamAgentSelection[] {
    switch (mode) {
      case 'broadcast':
        return agents
      case 'single':
        return agents.length > 0 ? [agents[0]] : []
      case 'round-robin':
      case 'load-balanced':
      default:
        return agents
    }
  }

  private async broadcastPrompt(
    targets: TeamAgentSelection[],
    prompt: string,
    outputs: RuntimeOutput[],
    errors: string[],
  ): Promise<void> {
    const promises = targets.map(async (sel) => {
      const runner = this.runners.get(sel.agentName)
      if (!runner) {
        errors.push(`${sel.agentName}: runner not found`)
        return
      }
      try {
        const output = await runner.prompt({
          taskId: crypto.randomUUID(),
          prompt,
          source: 'multi-agent',
        })
        outputs.push(output)
      } catch (e) {
        errors.push(`${sel.agentName}: ${e instanceof Error ? e.message : String(e)}`)
      }
    })
    await Promise.allSettled(promises)
  }

  private async singlePrompt(
    target: TeamAgentSelection,
    prompt: string,
    outputs: RuntimeOutput[],
    errors: string[],
  ): Promise<void> {
    const runner = this.runners.get(target.agentName)
    if (!runner) {
      errors.push(`${target.agentName}: runner not found`)
      return
    }
    try {
      const output = await runner.prompt({
        taskId: crypto.randomUUID(),
        prompt,
        source: 'multi-agent',
      })
      outputs.push(output)
    } catch (e) {
      errors.push(`${target.agentName}: ${e instanceof Error ? e.message : String(e)}`)
    }
  }
}
