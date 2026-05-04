import { vi } from 'vitest'

Object.defineProperty(globalThis, 'crypto', {
  value: {
    randomUUID: () => 'test-uuid',
  },
  configurable: true,
})

vi.stubGlobal('__TAURI_INTERNALS__', undefined)
