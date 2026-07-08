# Internationalization (i18n) Guide

## Overview

This guide covers internationalization and localization for Hermes Game Operator.

---

## Supported Languages

| Language | Code | Status | Coverage |
|----------|------|--------|----------|
| English | en | ✅ Complete | 100% |
| 中文 (简体) | zh-CN | ✅ Complete | 100% |
| 日本語 | ja | ✅ Complete | 100% |
| 한국어 | ko | ✅ Complete | 100% |
| Español | es | ✅ Complete | 100% |
| Français | fr | ✅ Complete | 100% |
| Deutsch | de | ✅ Complete | 100% |
| Português | pt | ✅ Complete | 100% |
| Русский | ru | ✅ Complete | 100% |
| العربية | ar | ✅ Complete | 100% |

---

## Translation File Structure

### File Organization

```
src/locales/
├── en.json          # English (default)
├── zh-CN.json       # Chinese (Simplified)
├── ja.json          # Japanese
├── ko.json          # Korean
├── es.json          # Spanish
├── fr.json          # French
├── de.json          # German
├── pt.json          # Portuguese
├── ru.json          # Russian
└── ar.json          # Arabic
```

### Translation Keys

**en.json**:
```json
{
  "common": {
    "save": "Save",
    "cancel": "Cancel",
    "confirm": "Confirm",
    "loading": "Loading..."
  },
  "task": {
    "start": "Start Task",
    "pause": "Pause Task",
    "resume": "Resume Task",
    "stop": "Stop Task"
  },
  "status": {
    "idle": "Idle",
    "planning": "Planning",
    "running": "Running",
    "completed": "Completed"
  }
}
```

**zh-CN.json**:
```json
{
  "common": {
    "save": "保存",
    "cancel": "取消",
    "confirm": "确认",
    "loading": "加载中..."
  },
  "task": {
    "start": "启动任务",
    "pause": "暂停任务",
    "resume": "恢复任务",
    "stop": "停止任务"
  },
  "status": {
    "idle": "空闲",
    "planning": "规划中",
    "running": "运行中",
    "completed": "已完成"
  }
}
```

---

## Using Translations

### In Vue Components

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <button>{{ t('task.start') }}</button>
</template>
```

### In TypeScript

```typescript
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

function showMessage() {
  alert(t('task.complete'))
}
```

### With Parameters

```json
{
  "task": {
    "progress": "Task {id} is {status}"
  }
}
```

```typescript
t('task.progress', { 
  id: '123', 
  status: t('status.running') 
})
// Output: "Task 123 is Running"
```

---

## Language Detection

### Automatic Detection

```typescript
function detectLanguage(): string {
  // 1. Check localStorage
  const stored = localStorage.getItem('language')
  if (stored && supportedLanguages.includes(stored)) {
    return stored
  }
  
  // 2. Check browser language
  const browserLang = navigator.language
  if (supportedLanguages.includes(browserLang)) {
    return browserLang
  }
  
  // 3. Check browser languages (fallback)
  const browserLangs = navigator.languages
  for (const lang of browserLangs) {
    if (supportedLanguages.includes(lang)) {
      return lang
    }
  }
  
  // 4. Default to English
  return 'en'
}
```

### Manual Selection

```vue
<template>
  <select v-model="selectedLanguage" @change="changeLanguage">
    <option value="en">English</option>
    <option value="zh-CN">中文</option>
    <option value="ja">日本語</option>
    <!-- ... -->
  </select>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { locale } = useI18n()
const selectedLanguage = ref(locale.value)

function changeLanguage() {
  locale.value = selectedLanguage.value
  localStorage.setItem('language', selectedLanguage.value)
}
</script>
```

---

## RTL Support

### RTL Languages

- Arabic (ar)
- Hebrew (he)
- Persian (fa)
- Urdu (ur)

### CSS Implementation

```css
/* Base styles */
[dir="rtl"] {
  text-align: right;
}

/* Sidebar */
[dir="rtl"] .sidebar {
  right: 0;
  left: auto;
}

/* Icons */
[dir="rtl"] .icon-arrow-left {
  transform: scaleX(-1);
}

/* Flexbox */
[dir="rtl"] .flex-row {
  flex-direction: row-reverse;
}
```

### Template Implementation

```vue
<template>
  <div :dir="isRTL ? 'rtl' : 'ltr'">
    <Sidebar />
    <MainContent />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { locale } = useI18n()

const rtlLanguages = ['ar', 'he', 'fa', 'ur']
const isRTL = computed(() => rtlLanguages.includes(locale.value))
</script>
```

---

## Date and Time Formatting

### Localized Dates

```typescript
function formatDate(date: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  }).format(date)
}

// Examples:
// English: "July 8, 2026"
// Chinese: "2026年7月8日"
// Japanese: "2026年7月8日"
// German: "8. Juli 2026"
```

### Localized Times

```typescript
function formatTime(date: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  }).format(date)
}

// Examples:
// English: "2:30:45 PM"
// Chinese: "下午2:30:45"
// Japanese: "14:30:45"
// German: "14:30:45"
```

---

## Number Formatting

### Localized Numbers

```typescript
function formatNumber(num: number, locale: string): string {
  return new Intl.NumberFormat(locale).format(num)
}

// Examples:
// English: "1,234,567"
// German: "1.234.567"
// French: "1 234 567"
```

### Currency Formatting

```typescript
function formatCurrency(amount: number, currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: currency
  }).format(amount)
}

// Examples:
// USD in English: "$1,234.56"
// EUR in German: "1.234,56 €"
// JPY in Japanese: "￥1,235"
```

---

## Pluralization

### Simple Plurals

```json
{
  "task": {
    "count": "{count} task | {count} tasks"
  }
}
```

```typescript
t('task.count', { count: 1 })  // "1 task"
t('task.count', { count: 5 })  // "5 tasks"
```

### Complex Plurals

```json
{
  "message": {
    "items": "no items | one item | {count} items"
  }
}
```

```typescript
t('message.items', { count: 0 })  // "no items"
t('message.items', { count: 1 })  // "one item"
t('message.items', { count: 5 })  // "5 items"
```

---

## Translation Workflow

### Step 1: Add English Keys

```json
// en.json
{
  "feature": {
    "newFeature": "New Feature"
  }
}
```

---

### Step 2: Update Other Languages

```json
// zh-CN.json
{
  "feature": {
    "newFeature": "新功能"
  }
}

// ja.json
{
  "feature": {
    "newFeature": "新機能"
  }
}
```

---

### Step 3: Test

```bash
# Switch language in settings
Settings → Language → Select language

# Verify translation appears
```

---

## Translation Tools

### Translation Management

**Recommended Tools**:
- **Crowdin**: Collaborative translation platform
- **Lokalise**: Translation management system
- **Transifex**: Localization platform
- **POEditor**: Translation software

### Translation Memory

```typescript
// Cache translations
const translationCache = new Map()

function getTranslation(key: string, locale: string): string {
  const cacheKey = `${locale}:${key}`
  
  if (translationCache.has(cacheKey)) {
    return translationCache.get(cacheKey)
  }
  
  const translation = loadTranslation(key, locale)
  translationCache.set(cacheKey, translation)
  return translation
}
```

---

## Quality Assurance

### Translation Checklist

- [ ] All keys translated
- [ ] No missing translations
- [ ] Correct pluralization
- [ ] Proper date/time formatting
- [ ] RTL support (if applicable)
- [ ] Contextually appropriate
- [ ] Cultural sensitivity
- [ ] No hardcoded strings

### Automated Checks

```typescript
// Check for missing translations
function checkMissingTranslations(locale: string): string[] {
  const enKeys = Object.keys(en)
  const localeKeys = Object.keys(translations[locale])
  
  return enKeys.filter(key => !localeKeys.includes(key))
}

// Check for untranslated strings
function checkUntranslatedStrings(component: string): string[] {
  const source = fs.readFileSync(component, 'utf-8')
  const matches = source.match(/['"]([^'"]+)['"]/g)
  
  return matches?.filter(match => !isTranslationKey(match)) || []
}
```

---

## Best Practices

### 1. Use Translation Keys Everywhere

```vue
<!-- Good -->
<button>{{ t('task.start') }}</button>

<!-- Bad -->
<button>Start Task</button>
```

### 2. Avoid String Concatenation

```typescript
// Good
t('task.statusMessage', { status: t('status.running') })

// Bad
t('task.status') + ': ' + t('status.running')
```

### 3. Provide Context

```json
{
  "task": {
    "start": {
      "_comment": "Button label to start a new task",
      "text": "Start Task"
    }
  }
}
```

### 4. Test All Languages

```bash
# Test each language
for lang in en zh-CN ja ko es fr de pt ru ar; do
  npm run test -- --language=$lang
done
```

---

## Resources

- [Vue I18n Documentation](https://vue-i18n.intlify.dev/)
- [Intl API](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl)
- [Unicode CLDR](https://cldr.unicode.org/)
- [RTL Styling Guide](https://rtlstyling.com/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
