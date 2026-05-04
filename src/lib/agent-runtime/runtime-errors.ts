export class RuntimeError extends Error {
  constructor(
    public readonly code:
      | 'agent-not-found'
      | 'cwd-required'
      | 'cwd-not-absolute'
      | 'session-not-found'
      | 'transport-closed'
      | 'auth-cancelled'
      | 'task-failed',
    message: string,
    public readonly cause?: unknown,
  ) {
    super(message)
    this.name = 'RuntimeError'
  }
}

export function assertAbsoluteCwd(cwd: string): void {
  const value = cwd.trim()
  if (!value) {
    throw new RuntimeError('cwd-required', '请输入 Agent 所在机器上的绝对工作目录。')
  }
  const isAbsolute = value.startsWith('/') || /^[A-Za-z]:[\\/]/.test(value)
  if (!isAbsolute) {
    throw new RuntimeError('cwd-not-absolute', `工作目录必须是绝对路径，当前值: ${cwd}`)
  }
}

export function toRuntimeError(error: unknown, fallbackCode: RuntimeError['code']): RuntimeError {
  if (error instanceof RuntimeError) return error
  const message = error instanceof Error ? error.message : String(error)
  return new RuntimeError(fallbackCode, message, error)
}
