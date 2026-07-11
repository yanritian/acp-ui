import { createI18n } from 'vue-i18n'
import type { MessageSchema } from './types'
import { zhCN } from './zh-CN'
import { zhTW } from './zh-TW'
import { enUS } from './en-US'

// 语言配置
export const SUPPORTED_LANGS = [
  { code: 'zh-CN', name: '中文', flag: '🇨🇳' },
  { code: 'zh-TW', name: '繁體中文', flag: '🇹🇼' },
  { code: 'en-US', name: 'English', flag: '🇺🇸' },
  { code: 'de-DE', name: 'Deutsch', flag: '🇩🇪' },
  { code: 'es-ES', name: 'Español', flag: '🇪🇸' },
  { code: 'ru-RU', name: 'Русский', flag: '🇷🇺' },
  { code: 'ja-JP', name: '日本語', flag: '🇯🇵' },
  { code: 'ko-KR', name: '한국어', flag: '🇰🇷' },
  { code: 'vi-VN', name: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'th-TH', name: 'ไทย', flag: '🇹🇭' },
  { code: 'ms-MY', name: 'Bahasa Melayu', flag: '🇲🇾' },
  { code: 'fr-FR', name: 'Français', flag: '🇫🇷' },
] as const

export type SupportedLang = (typeof SUPPORTED_LANGS)[number]['code']

// 按需加载其他语言（code splitting）
const messageLoaders: Record<string, () => Promise<MessageSchema>> = {
  'zh-TW': () => import('./zh-TW').then(m => m.zhTW),
  'de-DE': () => import('./de-DE').then(m => m.deDE),
  'es-ES': () => import('./es-ES').then(m => m.esES),
  'ru-RU': () => import('./ru-RU').then(m => m.ruRU),
  'ja-JP': () => import('./ja-JP').then(m => m.jaJP),
  'ko-KR': () => import('./ko-KR').then(m => m.koKR),
  'vi-VN': () => import('./vi-VN').then(m => m.viVN),
  'th-TH': () => import('./th-TH').then(m => m.thTH),
  'ms-MY': () => import('./ms-MY').then(m => m.msMY),
  'fr-FR': () => import('./fr-FR').then(m => m.frFR),
}

// 从 localStorage 或浏览器语言偏好获取初始语言
function getInitialLocale(): string {
  const stored = localStorage.getItem('acp-ui:locale')
  if (stored && SUPPORTED_LANGS.some(l => l.code === stored)) {
    return stored
  }
  // 回退到浏览器语言
  const browserLang = navigator.language
  const match = SUPPORTED_LANGS.find(l => browserLang.startsWith(l.code.split('-')[0]))
  return match?.code ?? 'en-US'
}

export const i18n = createI18n<[MessageSchema], string>({
  legacy: false, // Composition API 模式
  locale: getInitialLocale(),
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  } as Record<string, MessageSchema>,
})

// 动态加载语言包
export async function loadLanguageAsync(locale: SupportedLang): Promise<void> {
  const currentLocale = i18n.global.locale as any
  const currentLocaleValue = currentLocale.value ?? currentLocale
  if (currentLocaleValue === locale) return

  // 如果还没有加载过该语言，动态导入
  const availableLocales = i18n.global.availableLocales
  if (!availableLocales.includes(locale) && messageLoaders[locale]) {
    const messages = await messageLoaders[locale]()
    i18n.global.setLocaleMessage(locale, messages)
  }

  // 设置当前语言
  if (currentLocale.value !== undefined) {
    currentLocale.value = locale
  } else {
    i18n.global.locale = locale
  }

  localStorage.setItem('acp-ui:locale', locale)
  document.documentElement.lang = locale
}

// 导出 useI18n 的便捷封装
export { useI18n } from 'vue-i18n'