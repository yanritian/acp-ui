import { describe, expect, it } from 'vitest'
import { assertAbsoluteCwd } from '../../lib/agent-runtime/runtime-errors'

describe('multi-session cwd validation', () => {
  it('rejects relative cwd', () => {
    expect(() => assertAbsoluteCwd('.')).toThrow(/绝对路径/)
  })

  it('rejects empty cwd', () => {
    expect(() => assertAbsoluteCwd('')).toThrow(/绝对工作目录/)
  })

  it('accepts Windows absolute cwd', () => {
    expect(() => assertAbsoluteCwd('D:\\work\\repo')).not.toThrow()
  })

  it('accepts Windows absolute cwd with forward slashes', () => {
    expect(() => assertAbsoluteCwd('D:/work/repo')).not.toThrow()
  })

  it('accepts Unix absolute cwd', () => {
    expect(() => assertAbsoluteCwd('/work/repo')).not.toThrow()
  })
})
