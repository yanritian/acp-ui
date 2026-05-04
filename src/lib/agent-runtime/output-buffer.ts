import type { SessionNotification } from '@agentclientprotocol/sdk'
import type { ChatMessage, ToolCallInfo } from '../types'
import type { RuntimeOutput, RuntimeTaskStatus } from './types'

export class OutputBuffer {
  private messages: ChatMessage[] = []
  private toolCalls = new Map<string, ToolCallInfo>()
  private content = ''
  private thought = ''
  private status: RuntimeTaskStatus = 'running'
  private error: string | undefined

  constructor(
    private readonly taskId: string,
    private readonly sessionId: string,
    private readonly agentName: string,
  ) {}

  apply(notification: SessionNotification): RuntimeOutput {
    const update = notification.update

    if (update.sessionUpdate === 'user_message_chunk' && update.content.type === 'text') {
      this.appendMessage('user', update.content.text)
    }

    if (update.sessionUpdate === 'agent_message_chunk' && update.content.type === 'text') {
      this.content += update.content.text
      this.appendMessage('assistant', update.content.text)
    }

    if (update.sessionUpdate === 'agent_thought_chunk' && update.content.type === 'text') {
      this.thought += update.content.text
      const last = this.messages[this.messages.length - 1]
      if (last && last.role === 'assistant') {
        last.thought = (last.thought ?? '') + update.content.text
      } else {
        this.messages.push({
          id: crypto.randomUUID(),
          role: 'assistant',
          content: '',
          thought: update.content.text,
          timestamp: Date.now(),
          toolCalls: [],
        })
      }
    }

    if (update.sessionUpdate === 'tool_call') {
      const toolCall: ToolCallInfo = {
        toolCallId: update.toolCallId,
        title: update.title,
        kind: update.kind || 'other',
        status: update.status || 'pending',
        locations: update.locations,
      }
      this.toolCalls.set(update.toolCallId, toolCall)
      const lastAssistant = [...this.messages].reverse().find((m) => m.role === 'assistant')
      if (lastAssistant) {
        lastAssistant.toolCalls = lastAssistant.toolCalls ?? []
        lastAssistant.toolCalls.push(toolCall)
      }
    }

    if (update.sessionUpdate === 'tool_call_update') {
      const existing = this.toolCalls.get(update.toolCallId)
      if (existing) {
        if (update.title) existing.title = update.title
        if (update.status) existing.status = update.status
      }
    }

    return this.snapshot()
  }

  complete(): RuntimeOutput {
    this.status = 'completed'
    return this.snapshot()
  }

  fail(error: string): RuntimeOutput {
    this.status = 'failed'
    this.error = error
    return this.snapshot()
  }

  snapshot(): RuntimeOutput {
    return {
      taskId: this.taskId,
      sessionId: this.sessionId,
      agentName: this.agentName,
      content: this.content,
      thought: this.thought,
      messages: this.messages,
      toolCalls: Array.from(this.toolCalls.values()),
      status: this.status,
      error: this.error,
    }
  }

  private appendMessage(role: 'user' | 'assistant', text: string): void {
    const last = this.messages[this.messages.length - 1]
    if (last && last.role === role) {
      last.content += text
      return
    }

    this.messages.push({
      id: crypto.randomUUID(),
      role,
      content: text,
      timestamp: Date.now(),
      toolCalls: role === 'assistant' ? [] : undefined,
    })
  }
}
