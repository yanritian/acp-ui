<script setup lang="ts">
import { ref, onMounted } from 'vue'

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
  // Try to load from Tauri if available
  try {
    const saved = localStorage.getItem('bot-config')
    if (saved) {
      config.value = JSON.parse(saved)
    }
  } catch (e) {
    console.error('Failed to load bot config:', e)
  }
}

async function saveConfig() {
  saving.value = true
  savedMessage.value = ''

  try {
    // Save to localStorage for now
    localStorage.setItem('bot-config', JSON.stringify(config.value))

    // In Tauri, would call backend to save
    // await invoke('save_bot_config', { config: config.value })

    savedMessage.value = '配置已保存'
    setTimeout(() => { savedMessage.value = '' }, 3000)
  } catch (e) {
    savedMessage.value = '保存失败: ' + (e as Error).message
  } finally {
    saving.value = false
  }
}

function testConnection(platform: string) {
  // Placeholder for connection test
  savedMessage.value = `${platform} 连接测试功能正在开发中`
  setTimeout(() => { savedMessage.value = '' }, 3000)
}

onMounted(loadConfig)
</script>

<template>
  <div class="bot-settings">
    <header class="settings-header">
      <h1>🤖 Bot 配置</h1>
      <p class="subtitle">配置远程指令入口（飞书/Telegram/Discord）</p>
    </header>

    <!-- Platform tabs -->
    <div class="platform-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'feishu' }]"
        @click="activeTab = 'feishu'"
      >
        <span class="tab-icon">📱</span>
        <span>飞书</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'telegram' }]"
        @click="activeTab = 'telegram'"
      >
        <span class="tab-icon">✈️</span>
        <span>Telegram</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'discord' }]"
        @click="activeTab = 'discord'"
      >
        <span class="tab-icon">🎮</span>
        <span>Discord</span>
      </button>
    </div>

    <!-- Feishu config -->
    <div v-if="activeTab === 'feishu'" class="config-panel">
      <div class="config-section">
        <h3>飞书机器人配置</h3>
        <p class="help-text">
          在飞书开放平台创建应用后，获取 App ID 和 App Secret。
          <a href="https://open.feishu.cn/app" target="_blank">前往飞书开放平台</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.feishu.enabled" />
            <span>启用飞书 Bot</span>
          </label>
        </div>

        <div class="form-group">
          <label>App ID</label>
          <input
            type="text"
            v-model="config.feishu.appId"
            placeholder="cli_xxxxxxxxxxxx"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>App Secret</label>
          <input
            type="password"
            v-model="config.feishu.appSecret"
            placeholder="应用密钥"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>Encrypt Key（可选）</label>
          <input
            type="text"
            v-model="config.feishu.encryptKey"
            placeholder="消息加密密钥"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="form-group">
          <label>Verification Token（可选）</label>
          <input
            type="text"
            v-model="config.feishu.verificationToken"
            placeholder="事件验证令牌"
            :disabled="!config.feishu.enabled"
          />
        </div>

        <div class="action-row">
          <button
            class="test-btn"
            @click="testConnection('飞书')"
            :disabled="!config.feishu.enabled || !config.feishu.appId"
          >
            测试连接
          </button>
        </div>
      </div>
    </div>

    <!-- Telegram config -->
    <div v-if="activeTab === 'telegram'" class="config-panel">
      <div class="config-section">
        <h3>Telegram Bot 配置</h3>
        <p class="help-text">
          在 Telegram 中与 @BotFather 对话创建 Bot，获取 Bot Token。
          <a href="https://t.me/BotFather" target="_blank">前往 BotFather</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.telegram.enabled" />
            <span>启用 Telegram Bot</span>
          </label>
        </div>

        <div class="form-group">
          <label>Bot Token</label>
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
            @click="testConnection('Telegram')"
            :disabled="!config.telegram.enabled || !config.telegram.botToken"
          >
            测试连接
          </button>
        </div>
      </div>
    </div>

    <!-- Discord config -->
    <div v-if="activeTab === 'discord'" class="config-panel">
      <div class="config-section">
        <h3>Discord Bot 配置</h3>
        <p class="help-text">
          在 Discord Developer Portal 创建应用并获取 Bot Token。
          <a href="https://discord.com/developers/applications" target="_blank">前往 Discord Developer Portal</a>
        </p>

        <div class="form-group">
          <label>
            <input type="checkbox" v-model="config.discord.enabled" />
            <span>启用 Discord Bot</span>
          </label>
        </div>

        <div class="form-group">
          <label>Bot Token</label>
          <input
            type="password"
            v-model="config.discord.botToken"
            placeholder="Bot Token"
            :disabled="!config.discord.enabled"
          />
        </div>

        <div class="form-group">
          <label>Channel ID（可选）</label>
          <input
            type="text"
            v-model="config.discord.channelId"
            placeholder="频道 ID"
            :disabled="!config.discord.enabled"
          />
        </div>

        <div class="action-row">
          <button
            class="test-btn"
            @click="testConnection('Discord')"
            :disabled="!config.discord.enabled || !config.discord.botToken"
          >
            测试连接
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
        {{ saving ? '保存中...' : '保存配置' }}
      </button>
      <span v-if="savedMessage" :class="['save-message', { error: savedMessage.includes('失败') }]">
        {{ savedMessage }}
      </span>
    </div>

    <!-- Info panel -->
    <div class="info-panel">
      <h4>💡 使用说明</h4>
      <ul>
        <li>配置 Bot 后，用户可以通过飞书/Telegram/Discord 发送指令</li>
        <li>Bot 会将指令转发给 Agent 执行，并返回结果</li>
        <li>支持文本消息、图片、文件等富媒体消息</li>
        <li>敏感信息（密钥等）建议使用环境变量配置</li>
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