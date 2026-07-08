# Multi-Language Support

This guide covers internationalization (i18n) and localization (l10n) for Hermes Game Operator.

## Supported Languages

### Current Support

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

## Translation Files

### File Structure

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

---

### Translation Keys

**Structure**:
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
    "stop": "Stop Task",
    "complete": "Complete Task"
  },
  "status": {
    "idle": "Idle",
    "planning": "Planning",
    "waitingApproval": "Waiting for Approval",
    "running": "Running",
    "paused": "Paused",
    "completed": "Completed",
    "failed": "Failed",
    "cancelled": "Cancelled"
  },
  "approval": {
    "request": "Approval Request",
    "approve": "Approve",
    "reject": "Reject",
    "comment": "Add Comment"
  }
}
```

---

## Usage

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

---

### In TypeScript

```typescript
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

function showMessage() {
  alert(t('task.complete'))
}
```

---

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

## Adding New Translations

### Step 1: Add Keys to English

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

## Language Switching

### User Interface

**Settings Panel**:
```vue
<select v-model="selectedLanguage">
  <option value="en">English</option>
  <option value="zh-CN">中文</option>
  <option value="ja">日本語</option>
  <!-- ... -->
</select>
```

---

### Programmatic

```typescript
import { useI18n } from 'vue-i18n'

const { locale } = useI18n()

function switchLanguage(lang: string) {
  locale.value = lang
  localStorage.setItem('language', lang)
}
```

---

### Auto-Detection

```typescript
function detectLanguage(): string {
  // Check localStorage
  const stored = localStorage.getItem('language')
  if (stored) return stored
  
  // Check browser language
  const browserLang = navigator.language
  if (supportedLanguages.includes(browserLang)) {
    return browserLang
  }
  
  // Default to English
  return 'en'
}
```

---

## RTL Support

### Arabic (ar) and Hebrew (he)

**CSS**:
```css
[dir="rtl"] {
  text-align: right;
}

[dir="rtl"] .sidebar {
  right: 0;
  left: auto;
}

[dir="rtl"] .icon {
  transform: scaleX(-1);
}
```

---

### Template

```vue
<div :dir="isRTL ? 'rtl' : 'ltr'">
  <Sidebar />
  <MainContent />
</div>
```

---

### Detection

```typescript
const rtlLanguages = ['ar', 'he', 'fa', 'ur']

function isRTL(lang: string): boolean {
  return rtlLanguages.includes(lang)
}
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

// English: "July 8, 2026"
// Chinese: "2026年7月8日"
// Japanese: "2026年7月8日"
```

---

### Localized Times

```typescript
function formatTime(date: Date, locale: string): string {
  return new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  }).format(date)
}

// English: "2:30:45 PM"
// Chinese: "下午2:30:45"
// Japanese: "14:30:45"
```

---

## Number Formatting

### Localized Numbers

```typescript
function formatNumber(num: number, locale: string): string {
  return new Intl.NumberFormat(locale).format(num)
}

// English: "1,234,567"
// German: "1.234.567"
// French: "1 234 567"
```

---

### Currency

```typescript
function formatCurrency(amount: number, currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: currency
  }).format(amount)
}

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

---

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

## Translation Management

### Translation Workflow

1. **Add English Keys**
   - Update `en.json`
   - Document new keys

2. **Request Translations**
   - Create translation request
   - Assign to translators

3. **Review Translations**
   - Check accuracy
   - Verify context

4. **Merge Translations**
   - Update language files
   - Test in application

5. **Release**
   - Include in release notes
   - Update documentation

---

### Translation Tools

**Recommended**:
- **Crowdin**: Collaborative translation
- **Lokalise**: Translation management
- **Transifex**: Translation platform
- **POEditor**: Translation software

---

### Quality Assurance

**Checklist**:
- [ ] All keys translated
- [ ] No missing translations
- [ ] Correct pluralization
- [ ] Proper date/time formatting
- [ ] RTL support (if applicable)
- [ ] Contextually appropriate
- [ ] Cultural sensitivity

---

## Common Translation Issues

### Issue 1: Context Missing

**Problem**: Translators don't understand context

**Solution**:
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

---

### Issue 2: String Concatenation

**Problem**: Breaking sentences across keys

**Bad**:
```typescript
t('task.status') + ': ' + t('status.running')
```

**Good**:
```json
{
  "task": {
    "statusMessage": "Task status: {status}"
  }
}
```

```typescript
t('task.statusMessage', { status: t('status.running') })
```

---

### Issue 3: Hardcoded Strings

**Problem**: Text not in translation files

**Bad**:
```vue
<button>Start Task</button>
```

**Good**:
```vue
<button>{{ t('task.start') }}</button>
```

---

### Issue 4: Inconsistent Terminology

**Problem**: Same concept, different translations

**Solution**:
- Create glossary
- Use consistent terms
- Review regularly

---

## Testing

### Manual Testing

1. **Switch Languages**
   - Test all supported languages
   - Check all screens

2. **Verify Text**
   - No untranslated strings
   - Correct formatting
   - Proper alignment

3. **Test RTL**
   - Arabic/Hebrew layouts
   - Icon mirroring
   - Text direction

---

### Automated Testing

```typescript
import { describe, it, expect } from 'vitest'
import { useI18n } from 'vue-i18n'

describe('i18n', () => {
  it('should have all keys in all languages', () => {
    const enKeys = Object.keys(en)
    const zhKeys = Object.keys(zhCN)
    
    expect(zhKeys).toEqual(enKeys)
  })
})
```

---

## Best Practices

### Do's

✅ Use translation keys everywhere  
✅ Provide context for translators  
✅ Test with all supported languages  
✅ Use ICU message format  
✅ Handle pluralization properly  
✅ Format dates/numbers locally  
✅ Support RTL languages  

### Don'ts

❌ Hardcode strings in templates  
❌ Concatenate translated strings  
❌ Assume text length  
❌ Ignore cultural differences  
❌ Skip testing  
❌ Use machine translation only  
❌ Forget to update documentation  

---

## Resources

- [Vue I18n Documentation](https://vue-i18n.intlify.dev/)
- [ICU Message Format](https://formatjs.io/docs/core-concepts/icu-syntax/)
- [Unicode CLDR](https://cldr.unicode.org/)
- [RTL Styling Guide](https://rtlstyling.com/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08
