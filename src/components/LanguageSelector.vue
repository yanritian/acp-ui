<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { SUPPORTED_LANGS, loadLanguageAsync, type SupportedLang } from '../locales'

const { locale, t } = useI18n()

const currentLang = computed(() =>
  SUPPORTED_LANGS.find(l => l.code === locale.value)
)

async function changeLanguage(code: SupportedLang) {
  await loadLanguageAsync(code)
}
</script>

<template>
  <div class="language-selector">
    <details class="lang-dropdown">
      <summary class="lang-current" :title="t('language.selectLanguage')">
        <span class="lang-flag">{{ currentLang?.flag }}</span>
        <span class="lang-name">{{ currentLang?.name }}</span>
        <span class="lang-arrow">▾</span>
      </summary>
      <ul class="lang-list">
        <li
          v-for="lang in SUPPORTED_LANGS"
          :key="lang.code"
          :class="['lang-item', { active: lang.code === locale }]"
          @click="changeLanguage(lang.code)"
        >
          <span class="lang-flag">{{ lang.flag }}</span>
          <span class="lang-name">{{ lang.name }}</span>
          <span v-if="lang.code === locale" class="lang-check">✓</span>
        </li>
      </ul>
    </details>
  </div>
</template>

<style scoped>
.language-selector {
  position: relative;
}

.lang-dropdown {
  list-style: none;
}

.lang-dropdown summary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: #64748B;
  transition: all 0.2s;
  list-style: none;
}

.lang-dropdown summary::-webkit-details-marker {
  display: none;
}

.lang-dropdown[open] summary {
  background: #F1F5F9;
}

.lang-current:hover {
  background: #F1F5F9;
  color: #1E293B;
}

.lang-flag {
  font-size: 16px;
}

.lang-name {
  font-weight: 500;
}

.lang-arrow {
  font-size: 10px;
  opacity: 0.6;
}

.lang-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin: 4px 0 0 0;
  padding: 4px;
  list-style: none;
  background: white;
  border: 1px solid #E2E8F0;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  max-height: 320px;
  overflow-y: auto;
  z-index: 1000;
}

.lang-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.15s;
}

.lang-item:hover {
  background: #F1F5F9;
}

.lang-item.active {
  background: #EFF6FF;
  color: #1D4ED8;
}

.lang-check {
  margin-left: auto;
  font-weight: bold;
  color: #3B82F6;
}
</style>