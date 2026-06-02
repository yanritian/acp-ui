<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isTauriHost } from '../lib/platform'
import { useI18n } from '@/locales'

const { t } = useI18n()

interface BotConfig {
  feishu: {
    appId: string
    appSecret: string
    encryptKey: string
    verificationToken: string
    enabled: boolean
  }
  telegram: {
    botToken: string
    enabled: boolean
  }
  discord: {
    botToken: string
    channelId: string
    enabled: boolean
  }
}

const config = ref<BotConfig>({
  feishu: {
    appId: '',
    appSecret: '',
    encryptKey: '',
    verificationToken: '',
    enabled: false
  },
  telegram: {
    botToken: '',
    enabled: false
  },
  discord: {
    botToken: '',
    channelId: '',
    enabled: false
  }
})

const saving = ref(false)
const savedMessage = ref('')
const activeTab = ref<'feishu' | 'telegram' | 'discord'>('feishu')

async function loadConfig() {
  if (isTauriHost()) {
    try {
      const gatewayConfig = await invoke<Record<string, unknown>>('get_gateway_config')
      if (gatewayConfig) {
        // Extract bot-related config from gateway config
        if (gatewayConfig.feishu) {
          const incoming = gatewayConfig.feishu as Record<string, unknown>
          config.value.feishu = {
            appId: (incoming.appId as string) ?? '',
            appSecret: (incoming.appSecret as string) ?? '',
            encryptKey: (incoming.encryptKey as string) ?? '',
            verificationToken: (incoming.verificationToken as string) ?? '',
            enabled: (incoming.enabled as boolean) ?? false,
          }
        }
        if (gatewayConfig.telegram) {
          const incoming = gatewayConfig.telegram as Record<string, unknown>
          config.value.telegram = {
            botToken: (incoming.botToken as string) ?? '',
            enabled: (incoming.enabled as boolean) ?? false,
          }
        }
        if (gatewayConfig.discord) {
          const incoming = gatewayConfig.discord as Record<string, unknown>
          config.value.discord = {
            botToken: (incoming.botToken as string) ?? '',
            channelId: (incoming.channelId as string) ?? '',
            enabled: (incoming.enabled as boolean) ?? false,
          }
        }
      }
    } catch (e) {
      console.log('No existing gateway config, using defaults')
    }
  } else {
    // Web fallback: localStorage
    try {
      const saved = localStorage.getItem('bot-config')
      if (saved) {
        config.value = JSON.parse(saved)
      }
    } catch (e) {
      console.error('Failed to load bot config:', e)
    }
  }
}

async function saveConfig() {
  saving.value = true
  savedMessage.value = ''

  try {
    if (isTauriHost()) {
      // Save via gateway config command
      await invoke('save_gateway_config', {
        config: {
          feishu: config.value.feishu,
          telegram: config.value.telegram,
          discord: config.value.discord,
        }
      })
    } else {
      localStorage.setItem('bot-config', JSON.stringify(config.value))
    }
    savedMessage.value = t('botSettings.configSaved')
    setTimeout(() => { savedMessage.value = '' }, 3000)
  } catch (e) {
    savedMessage.value = t('botSettings.saveFailed') + ': ' + (e as Error).message
  } finally {
    saving.value = false
  }
}

function testConnection(platform: string) {
  savedMessage.value = t('botSettings.testConnectionDev', { platform })
  setTimeout(() => { savedMessage.value = '' }, 3000)
}

onMounted(loadConfig)
</script>

<template>
  <div class="bot-settings">
    <header class="settings-header">
      <h1>🤖 {{ t('botSettings.title') }}</h1>
      <p class="subtitle">{{ t('botSettings.subtitle') }}</p>
    </header>

    <!-- Platform tabs -->
    <div class="platform-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'feishu' }]"
        @click="activeTab = 'feishu'"
      >
        <span class="tab-icon">📱</span>
        <span>{{ t('botSettings.feishu') }}</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'telegram' }]"
        @click="activeTab = 'telegram'"
      >
        <span class="tab-icon">✈️</span>
        <span>{{ t('botSettings.telegram') }}</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'discord' }]"
        @click="activeTab = 'discord'"
      >
        <span class="tab-icon">🎮</span>
        <span>{{ t('botSettings.discord') }}</span>
      </button>
    </div>

    <!-- Feishu config -->
    <div v-if="activeTab === 'feishu'" class="config-panel">
      <div class="config-section">
        <h3>{{ t('botSettings.feishuConfig') }}</h3>
        <p class="help-text">
          {{ t('botSettings.feishuHelp') }}
          <a href="https://open.feishu.cn/app" target="_blank">{{ t('botSettings.feishuPlatform') }}</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.feishu.enabled" />
            <span>{{ t('botSettings.enableFeishu') }}</span>
          </label>
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.appId') }}</label>
          <input
            type="text"
            v-model="config.feishu.appId"
            placeholder="cli_xxxxxxxxxxxx"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.appSecret') }}</label>
          <input
            type="password"
            v-model="config.feishu.appSecret"
            :placeholder="t('botSettings.appSecret')"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.encryptKeyOptional') }}</label>
          <input
            type="text"
            v-model="config.feishu.encryptKey"
            :placeholder="t('botSettings.encryptKey')"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.verificationTokenOptional') }}</label>
          <input
            type="text"
            v-model="config.feishu.verificationToken"
            :placeholder="t('botSettings.verificationToken')"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="action-row">
          <button
            class="test-btn"
            @click="testConnection(t('botSettings.feishu'))"
            :disabled="!config.feishu.enabled || !config.feishu.appId"
          >
            {{ t('botSettings.testConnection') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Telegram config -->
    <div v-if="activeTab === 'telegram'" class="config-panel">
      <div class="config-section">
        <h3>{{ t('botSettings.telegramConfig') }}</h3>
        <p class="help-text">
          {{ t('botSettings.telegramHelp') }}
          <a href="https://t.me/BotFather" target="_blank">{{ t('botSettings.telegramBotFather') }}</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.telegram.enabled" />
            <span>{{ t('botSettings.enableTelegram') }}</span>
          </label>
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.botToken') }}</label>
          <input
            type="password"
            v-model="config.telegram.botToken"
            placeholder="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
            :disabled="!config.telegram.enabled"
          />
        </div>

        <div class="action-row">
          <button
            class="test-btn"
            @click="testConnection(t('botSettings.telegram'))"
            :disabled="!config.telegram.enabled || !config.telegram.botToken"
          >
            {{ t('botSettings.testConnection') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Discord config -->
    <div v-if="activeTab === 'discord'" class="config-panel">
      <div class="config-section">
        <h3>{{ t('botSettings.discordConfig') }}</h3>
        <p class="help-text">
          {{ t('botSettings.discordHelp') }}
          <a href="https://discord.com/developers/applications" target="_blank">{{ t('botSettings.discordPortal') }}</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.discord.enabled" />
            <span>{{ t('botSettings.enableDiscord') }}</span>
          </label>
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.botToken') }}</label>
          <input
            type="password"
            v-model="config.discord.botToken"
            :placeholder="t('botSettings.botToken')"
            :disabled="!config.discord.enabled"
          />
        </div>

        <div class="form-group">
          <label>{{ t('botSettings.channelIdOptional') }}</label>
          <input
            type="text"
            v-model="config.discord.channelId"
            :placeholder="t('botSettings.channelId')"
            :disabled="!config.discord.enabled"
          />
        </div>

        <div class="action-row">
          <button
            class="test-btn"
            @click="testConnection(t('botSettings.discord'))"
            :disabled="!config.discord.enabled || !config.discord.botToken"
          >
            {{ t('botSettings.testConnection') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Save button -->
    <div class="save-section">
      <button
        class="save-btn"
        @click="saveConfig"
        :disabled="saving"
      >
        {{ saving ? t('botSettings.saving') : t('botSettings.saveConfig') }}
      </button>
      <span v-if="savedMessage" :class="['save-message', { error: savedMessage.includes(t('botSettings.saveFailed')) }]">
        {{ savedMessage }}
      </span>
    </div>

    <!-- Info panel -->
    <div class="info-panel">
      <h4>💡 {{ t('botSettings.usageNotes') }}</h4>
      <ul>
        <li>{{ t('botSettings.usageNote1') }}</li>
        <li>{{ t('botSettings.usageNote2') }}</li>
        <li>{{ t('botSettings.usageNote3') }}</li>
        <li>{{ t('botSettings.usageNote4') }}</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.bot-settings {
  padding: 20px;
  background: linear-gradient(135deg, #F8FAFC 0%, #EEF2FF 50%, #F8FAFC 100%);
  min-height: 100%;
  font-family: system-ui, -apple-system, sans-serif;
}

.settings-header {
  margin-bottom: 20px;
}

.settings-header h1 {
  font-size: 20px;
  font-weight: 600;
  color: #1E293B;
  margin: 0;
}

.subtitle {
  font-size: 13px;
  color: #64748B;
  margin-top: 4px;
}

.platform-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.tab-btn {
  padding: 10px 16px;
  background: white;
  border: 1px solid #E2E8F0;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.15s;
  font-size: 14px;
}

.tab-btn:hover {
  background: #F8FAFC;
}

.tab-btn.active {
  background: #3B82F6;
  border-color: #3B82F6;
  color: white;
}

.tab-icon {
  font-size: 18px;
}

.config-panel {
  background: white;
  border-radius: 12px;
  border: 1px solid #E2E8F0;
  padding: 20px;
  margin-bottom: 16px;
}

.config-section h3 {
  font-size: 16px;
  font-weight: 600;
  color: #1E293B;
  margin: 0 0 8px 0;
}

.help-text {
  font-size: 13px;
  color: #64748B;
  margin-bottom: 16px;
}

.help-text a {
  color: #3B82F6;
  text-decoration: none;
}

.help-text a:hover {
  text-decoration: underline;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  color: #475569;
  margin-bottom: 6px;
}

.form-group input[type="text"],
.form-group input[type="password"] {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #E2E8F0;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.15s;
}

.form-group input:focus {
  outline: none;
  border-color: #3B82F6;
}

.form-group input:disabled {
  background: #F1F5F9;
  color: #94A3B8;
}

.form-group input[type="checkbox"] {
  accent-color: #3B82F6;
  margin-right: 8px;
}

.action-row {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.test-btn {
  padding: 8px 16px;
  background: #F1F5F9;
  border: 1px solid #E2E8F0;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.test-btn:hover:not(:disabled) {
  background: #E2E8F0;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.save-btn {
  padding: 10px 20px;
  background: #3B82F6;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.save-btn:hover:not(:disabled) {
  background: #2563EB;
}

.save-btn:disabled {
  background: #94A3B8;
  cursor: not-allowed;
}

.save-message {
  font-size: 13px;
  color: #22C55E;
}

.save-message.error {
  color: #EF4444;
}

.info-panel {
  background: #FEF3C7;
  border: 1px solid #FCD34D;
  border-radius: 8px;
  padding: 16px;
}

.info-panel h4 {
  font-size: 14px;
  font-weight: 600;
  color: #92400E;
  margin: 0 0 8px 0;
}

.info-panel ul {
  margin: 0;
  padding-left: 20px;
  font-size: 13px;
  color: #78350F;
}

.info-panel li {
  margin-bottom: 4px;
}
</style>