import { afterEach, describe, expect, it } from 'vitest'
import { unref } from 'vue'
import { i18n, loadInitialLocale } from '../index'

describe('locale initialization', () => {
  afterEach(() => {
    localStorage.removeItem('acp-ui:locale')
  })

  it('loads a persisted lazy locale before the application renders', async () => {
    localStorage.setItem('acp-ui:locale', 'pt-BR')

    await loadInitialLocale()

    expect(i18n.global.availableLocales).toContain('pt-BR')
    expect(unref(i18n.global.locale)).toBe('pt-BR')
  })
})
