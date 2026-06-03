import { describe, expect, it } from 'vitest'
import { OutputBuffer } from '../output-buffer'

function textUpdate(sessionUpdate: 'user_message_chunk' | 'agent_message_chunk' | 'agent_thought_chunk', text: string) {
  return {
    sessionId: 'session-1',
    update: {
      sessionUpdate,
      content: { type: 'text', text },
    },
  } as any
}

describe('OutputBuffer', () => {
  it('merges assistant chunks into one output', () => {
    const buffer = new OutputBuffer('task-1', 'session-1', 'codex')
    buffer.apply(textUpdate('agent_message_chunk', 'hello '))
    const output = buffer.apply(textUpdate('agent_message_chunk', 'world'))
    expect(output.content).toBe('hello world')
    expect(output.messages).toHaveLength(1)
    expect(output.messages[0].content).toBe('hello world')
  })

  it('tracks thought and tool call updates', () => {
    const buffer = new OutputBuffer('task-1', 'session-1', 'codex')
    buffer.apply(textUpdate('agent_message_chunk', 'answer'))
    buffer.apply(textUpdate('agent_thought_chunk', 'reasoning'))
    const output = buffer.apply({
      sessionId: 'session-1',
      update: {
        sessionUpdate: 'tool_call',
        toolCallId: 'tool-1',
        title: 'Read file',
        kind: 'read',
        status: 'pending',
      },
    } as any)
    expect(output.thought).toBe('reasoning')
    expect(output.toolCalls[0].toolCallId).toBe('tool-1')
  })

  it('completes with correct status', () => {
    const buffer = new OutputBuffer('task-1', 'session-1', 'codex')
    buffer.apply(textUpdate('agent_message_chunk', 'done'))
    const output = buffer.complete()
    expect(output.status).toBe('completed')
  })

  it('fails with error message', () => {
    const buffer = new OutputBuffer('task-1', 'session-1', 'codex')
    const output = buffer.fail('connection lost')
    expect(output.status).toBe('failed')
    expect(output.error).toBe('connection lost')
  })
})
