<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const gatewayConfig = ref({
  feishu: {
    appId: '',
    appSecret: '',
    encryptKey: '',
    verificationToken: '',
    enabled: false,
  },
  telegram: {
    botToken: '',
    enabled: false,
  },
  discord: {
    botToken: '',
    guildId: '',
    channelId: '',
    enabled: false,
  },
  app: {
    websocketPort: 1420,
    authMode: 'qrcode',
    enabled: true,
  },
  tunnel: {
    enabled: false,
    provider: 'ngrok',
    ngrokToken: '',
    ngrokRegion: 'ap',
    customUrl: '',
    status: 'stopped',
    publicUrl: '',
  },
})

const saving = ref(false)
const saved = ref(false)
const gatewayStatus = ref<'stopped' | 'running' | 'starting'>('stopped')
const qrCodeUrl = ref('')
const copied = ref(false)

// 加载配置
async function loadConfig() {
  try {
    const config = await invoke('get_gateway_config')
    if (config) {
      gatewayConfig.value = { ...gatewayConfig.value, ...(config as any) }
    }
  } catch (error) {
    console.log('No existing config, using defaults')
  }
}

// 保存配置
async function saveConfig() {
  saving.value = true
  try {
    await invoke('save_gateway_config', { config: gatewayConfig.value })
    // Reload config to confirm persistence and get masked secrets
    await loadConfig()
    saved.value = true
    setTimeout(() => { saved.value = false }, 2000)
  } catch (error) {
    console.error('Failed to save config:', error)
    alert(`保存失败: ${error}`)
  } finally {
    saving.value = false
  }
}

// 生成二维码
async function generateQRCode() {
  try {
    const url = await invoke('generate_app_qrcode')
    qrCodeUrl.value = url as string
  } catch (error) {
    console.error('Failed to generate QR code:', error)
  }
}

// 复制链接
async function copyLink() {
  if (qrCodeUrl.value) {
    await navigator.clipboard.writeText(qrCodeUrl.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

// 复制公网地址
async function copyPublicUrl() {
  if (gatewayConfig.value.tunnel.publicUrl) {
    await navigator.clipboard.writeText(gatewayConfig.value.tunnel.publicUrl)
  }
}

// 启动内网穿透
async function startTunnel() {
  try {
    gatewayConfig.value.tunnel.status = 'starting'
    const result = await invoke('start_tunnel', {
      provider: gatewayConfig.value.tunnel.provider,
      token: gatewayConfig.value.tunnel.ngrokToken,
      port: gatewayConfig.value.app.websocketPort,
    })
    gatewayConfig.value.tunnel.publicUrl = result as string
    gatewayConfig.value.tunnel.status = 'running'
  } catch (error) {
    gatewayConfig.value.tunnel.status = 'stopped'
    console.error('Failed to start tunnel:', error)
  }
}

// 停止内网穿透
async function stopTunnel() {
  try {
    await invoke('stop_tunnel')
    gatewayConfig.value.tunnel.status = 'stopped'
    gatewayConfig.value.tunnel.publicUrl = ''
  } catch (error) {
    console.error('Failed to stop tunnel:', error)
  }
}

// 启动Gateway
async function startGateway() {
  try {
    gatewayStatus.value = 'starting'
    await invoke('start_gateway', { config: gatewayConfig.value })
    gatewayStatus.value = 'running'
  } catch (error) {
    gatewayStatus.value = 'stopped'
    console.error('Failed to start gateway:', error)
  }
}

// 停止Gateway
async function stopGateway() {
  try {
    await invoke('stop_gateway')
    gatewayStatus.value = 'stopped'
  } catch (error) {
    console.error('Failed to stop gateway:', error)
  }
}

// 监听事件
onMounted(async () => {
  await loadConfig()

  listen('gateway-started', () => {
    gatewayStatus.value = 'running'
  })

  listen('gateway-stopped', () => {
    gatewayStatus.value = 'stopped'
  })
})
</script>

<template>
  <div class="gateway-page">
    <!-- Header -->
    <header class="page-header">
      <div class="header-left">
        <h1>远程控制配置</h1>
        <p class="subtitle">配置手机App、飞书、Telegram等远程控制方式</p>
      </div>
      <div class="header-right">
        <div class="status-badge" :class="gatewayStatus">
          <span class="status-dot"></span>
          <span>{{ gatewayStatus === 'running' ? '运行中' : gatewayStatus === 'starting' ? '启动中' : '已停止' }}</span>
        </div>
        <button class="btn btn-primary" @click="startGateway" :disabled="gatewayStatus === 'running'">
          启动服务
        </button>
        <button class="btn btn-secondary" @click="stopGateway" :disabled="gatewayStatus === 'stopped'">
          停止服务
        </button>
      </div>
    </header>

    <!-- Tunnel Section (内网穿透) -->
    <section class="config-section">
      <div class="section-header">
        <div class="section-icon">🌐</div>
        <div>
          <h2 class="section-title">内网穿透</h2>
          <p class="section-desc">无需公网IP，在外网也能远程控制</p>
        </div>
      </div>

      <div class="card">
        <div class="toggle-row">
          <div>
            <span class="toggle-label">启用内网穿透</span>
            <span class="toggle-desc">推荐使用ngrok或frp实现外网访问</span>
          </div>
          <div class="toggle" :class="{ active: gatewayConfig.tunnel.enabled }" @click="gatewayConfig.tunnel.enabled = !gatewayConfig.tunnel.enabled">
            <span class="toggle-knob"></span>
          </div>
        </div>

        <div v-if="gatewayConfig.tunnel.enabled" class="config-grid">
          <div class="input-group">
            <label class="input-label">穿透服务商</label>
            <select class="input" v-model="gatewayConfig.tunnel.provider">
              <option value="ngrok">ngrok (免费)</option>
              <option value="frp">frp (需要自有服务器)</option>
              <option value="cloudflare">Cloudflare Tunnel (免费)</option>
            </select>
          </div>

          <div v-if="gatewayConfig.tunnel.provider === 'ngrok'" class="input-group">
            <label class="input-label">ngrok Token</label>
            <input class="input" type="password" v-model="gatewayConfig.tunnel.ngrokToken" placeholder="从ngrok.com获取" />
            <span class="input-hint">免费注册: <a href="https://ngrok.com" target="_blank">ngrok.com</a></span>
          </div>

          <div v-if="gatewayConfig.tunnel.provider === 'ngrok'" class="input-group">
            <label class="input-label">区域</label>
            <select class="input" v-model="gatewayConfig.tunnel.ngrokRegion">
              <option value="ap">亚太 (新加坡)</option>
              <option value="us">美国</option>
              <option value="eu">欧洲</option>
            </select>
          </div>

          <div v-if="gatewayConfig.tunnel.provider === 'frp'" class="input-group full-width">
            <label class="input-label">frp服务器地址</label>
            <input class="input" type="text" v-model="gatewayConfig.tunnel.customUrl" placeholder="frp.example.com:7000" />
          </div>
        </div>

        <!-- Tunnel Status -->
        <div v-if="gatewayConfig.tunnel.enabled" class="tunnel-status">
          <div class="tunnel-info" v-if="gatewayConfig.tunnel.publicUrl">
            <span class="label">公网地址:</span>
            <code class="url">{{ gatewayConfig.tunnel.publicUrl }}</code>
            <button class="btn btn-sm btn-ghost" @click="copyPublicUrl">复制</button>
          </div>
          <div class="tunnel-actions">
            <button
              class="btn btn-success"
              @click="startTunnel"
              :disabled="gatewayConfig.tunnel.status === 'running'"
            >
              {{ gatewayConfig.tunnel.status === 'starting' ? '启动中...' : '启动穿透' }}
            </button>
            <button
              class="btn btn-danger"
              @click="stopTunnel"
              :disabled="gatewayConfig.tunnel.status === 'stopped'"
            >
              停止穿透
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- App Connection Section -->
    <section class="config-section">
      <div class="section-header">
        <div class="section-icon">📱</div>
        <div>
          <h2 class="section-title">App 远程连接</h2>
          <p class="section-desc">手机App扫描二维码直接连接</p>
        </div>
      </div>

      <div class="card">
        <div class="config-grid">
          <div class="input-group">
            <label class="input-label">WebSocket端口</label>
            <input class="input" type="number" v-model.number="gatewayConfig.app.websocketPort" />
          </div>
          <div class="input-group">
            <label class="input-label">认证方式</label>
            <select class="input" v-model="gatewayConfig.app.authMode">
              <option value="qrcode">二维码扫描</option>
              <option value="token">Token认证</option>
            </select>
          </div>
        </div>

        <div class="qrcode-section">
          <button class="btn btn-primary btn-lg" @click="generateQRCode">
            生成连接二维码
          </button>

          <div v-if="qrCodeUrl" class="qrcode-result">
            <div class="qrcode-display">
              <!-- 这里可以显示真正的二维码图片 -->
              <div class="qrcode-placeholder">
                <span class="icon">📱</span>
                <span>扫描连接</span>
              </div>
            </div>
            <div class="connection-info">
              <code class="url">{{ qrCodeUrl }}</code>
              <button class="btn btn-sm btn-ghost" @click="copyLink">
                {{ copied ? '已复制' : '复制链接' }}
              </button>
            </div>
            <p class="hint">
              提示: 手机需在同一局域网，或启用内网穿透后在外网使用
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- Feishu Section -->
    <section class="config-section">
      <div class="section-header">
        <div class="section-icon" style="background: linear-gradient(135deg, #00d6aa, #00a88a);">💬</div>
        <div>
          <h2 class="section-title">飞书机器人</h2>
          <p class="section-desc">通过飞书Bot发送指令和查看执行结果</p>
        </div>
      </div>

      <div class="card">
        <div class="toggle-row">
          <span class="toggle-label">启用飞书Bot</span>
          <div class="toggle" :class="{ active: gatewayConfig.feishu.enabled }" @click="gatewayConfig.feishu.enabled = !gatewayConfig.feishu.enabled">
            <span class="toggle-knob"></span>
          </div>
        </div>

        <div v-if="gatewayConfig.feishu.enabled" class="config-grid">
          <div class="input-group">
            <label class="input-label">App ID</label>
            <input class="input" type="text" v-model="gatewayConfig.feishu.appId" placeholder="cli_xxxxxxxxxx" />
          </div>
          <div class="input-group">
            <label class="input-label">App Secret</label>
            <input class="input" type="password" v-model="gatewayConfig.feishu.appSecret" />
          </div>
          <div class="input-group">
            <label class="input-label">Encrypt Key (可选)</label>
            <input class="input" type="password" v-model="gatewayConfig.feishu.encryptKey" />
          </div>
          <div class="input-group">
            <label class="input-label">Verification Token (可选)</label>
            <input class="input" type="text" v-model="gatewayConfig.feishu.verificationToken" />
          </div>
        </div>

        <div class="help-box">
          <h4>配置步骤</h4>
          <ol>
            <li>访问 <a href="https://open.feishu.cn" target="_blank">飞书开放平台</a> 创建应用</li>
            <li>获取 App ID 和 App Secret</li>
            <li>配置事件订阅: <code>http://IP:{{ gatewayConfig.app.websocketPort }}/feishu/webhook</code></li>
            <li>添加事件: <code>im.message.receive_v1</code></li>
          </ol>
        </div>
      </div>
    </section>

    <!-- Telegram Section -->
    <section class="config-section">
      <div class="section-header">
        <div class="section-icon" style="background: linear-gradient(135deg, #0088cc, #0066aa);">✈️</div>
        <div>
          <h2 class="section-title">Telegram Bot</h2>
          <p class="section-desc">通过Telegram远程控制</p>
        </div>
      </div>

      <div class="card">
        <div class="toggle-row">
          <span class="toggle-label">启用Telegram Bot</span>
          <div class="toggle" :class="{ active: gatewayConfig.telegram.enabled }" @click="gatewayConfig.telegram.enabled = !gatewayConfig.telegram.enabled">
            <span class="toggle-knob"></span>
          </div>
        </div>

        <div v-if="gatewayConfig.telegram.enabled" class="config-grid">
          <div class="input-group full-width">
            <label class="input-label">Bot Token</label>
            <input class="input" type="password" v-model="gatewayConfig.telegram.botToken" placeholder="123456789:ABCdef..." />
          </div>
        </div>

        <div class="help-box">
          <h4>配置步骤</h4>
          <ol>
            <li>在Telegram搜索 <code>@BotFather</code></li>
            <li>发送 <code>/newbot</code> 创建机器人</li>
            <li>获取 Bot Token</li>
          </ol>
        </div>
      </div>
    </section>

    <!-- Discord Section -->
    <section class="config-section">
      <div class="section-header">
        <div class="section-icon" style="background: linear-gradient(135deg, #5865f2, #4752c4);">🎮</div>
        <div>
          <h2 class="section-title">Discord Bot</h2>
          <p class="section-desc">通过Discord频道远程控制</p>
        </div>
      </div>

      <div class="card">
        <div class="toggle-row">
          <span class="toggle-label">启用Discord Bot</span>
          <div class="toggle" :class="{ active: gatewayConfig.discord.enabled }" @click="gatewayConfig.discord.enabled = !gatewayConfig.discord.enabled">
            <span class="toggle-knob"></span>
          </div>
        </div>

        <div v-if="gatewayConfig.discord.enabled" class="config-grid">
          <div class="input-group full-width">
            <label class="input-label">Bot Token</label>
            <input class="input" type="password" v-model="gatewayConfig.discord.botToken" />
          </div>
          <div class="input-group">
            <label class="input-label">Guild ID (可选)</label>
            <input class="input" type="text" v-model="gatewayConfig.discord.guildId" placeholder="服务器ID" />
          </div>
          <div class="input-group">
            <label class="input-label">Channel ID (可选)</label>
            <input class="input" type="text" v-model="gatewayConfig.discord.channelId" placeholder="频道ID" />
          </div>
        </div>

        <div class="help-box">
          <h4>配置步骤</h4>
          <ol>
            <li>访问 <a href="https://discord.com/developers/applications" target="_blank">开发者门户</a></li>
            <li>创建应用 → Bot → Add Bot</li>
            <li>获取 Token 并邀请Bot到服务器</li>
          </ol>
        </div>
      </div>
    </section>

    <!-- Save Button -->
    <div class="save-bar">
      <button class="btn btn-primary btn-lg" :disabled="saving" @click="saveConfig">
        {{ saving ? '保存中...' : saved ? '✓ 已保存' : '保存配置' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 基础变量回退 */
.gateway-page {
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;
  --text-xs: 12px;
  --text-sm: 14px;
  --text-base: 16px;
  --text-lg: 18px;
  --text-3xl: 30px;
  --primary: #6366f1;
  --accent: #22d3ee;
  --success: #10b981;
  --success-light: #d1fae5;
  --error: #ef4444;
  --warning: #f59e0b;
  --bg-main: #f8fafc;
  --bg-surface: #ffffff;
  --bg-subtle: #e2e8f0;
  --bg-muted: #cbd5e1;
  --text-primary: #0f172a;
  --text-secondary: #475569;
  --text-muted: #64748b;
  --text-subtle: #94a3b8;
  --border-color: #e2e8f0;
}

.gateway-page {
  padding: var(--space-6);
  max-width: 900px;
  margin: 0 auto;
  min-height: 100%;
  background: var(--bg-main);
}

/* 按钮样式 */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-sm);
  font-weight: 500;
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-secondary {
  background: var(--bg-subtle);
  color: var(--text-secondary);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.btn-success {
  background: var(--success);
  color: white;
}

.btn-danger {
  background: var(--error);
  color: white;
}

.btn-sm {
  padding: var(--space-1) var(--space-3);
  font-size: var(--text-xs);
}

.btn-lg {
  padding: var(--space-3) var(--space-6);
  font-size: var(--text-base);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 输入框样式 */
.input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.input:focus {
  border-color: var(--primary);
  outline: none;
}

.input-label {
  display: block;
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: var(--space-1);
}

.input-group {
  margin-bottom: var(--space-3);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-8);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--border-color);
}

.header-left h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--space-1);
}

.subtitle {
  color: var(--text-muted);
  font-size: var(--text-base);
}

.status-badge {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-full);
  background: var(--bg-subtle);
  font-size: var(--text-sm);
}

.status-badge.running {
  background: var(--success-light);
  color: var(--success);
}

.status-badge.running .status-dot {
  background: var(--success);
}

.status-badge.stopped .status-dot {
  background: var(--text-subtle);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.section-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.section-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
  font-size: 24px;
  background: linear-gradient(135deg, var(--primary), var(--accent));
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.section-desc {
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.toggle {
  width: 44px;
  height: 24px;
  background: var(--bg-subtle);
  border-radius: var(--radius-full);
  position: relative;
  cursor: pointer;
  transition: background 0.2s ease;
}

.toggle.active {
  background: var(--primary);
}

.toggle-knob {
  position: absolute;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: transform 0.2s ease;
}

.toggle.active .toggle-knob {
  transform: translateX(20px);
}

.status-badge {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-full);
  background: var(--bg-subtle);
  font-size: var(--text-sm);
}

.status-badge.running {
  background: var(--success-light);
  color: var(--success);
}

.status-badge.running .status-dot {
  background: var(--success);
}

.status-badge.stopped .status-dot {
  background: var(--text-subtle);
}

.config-section {
  margin-bottom: var(--space-6);
}

.card {
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  border: 1px solid var(--border-color);
}

.toggle-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3) 0;
}

.toggle-label {
  font-weight: 500;
  color: var(--text-primary);
}

.toggle-desc {
  display: block;
  font-size: var(--text-xs);
  color: var(--text-muted);
  margin-top: var(--space-1);
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-4);
  margin-top: var(--space-4);
}

.input-group.full-width {
  grid-column: 1 / -1;
}

.input-hint {
  display: block;
  font-size: var(--text-xs);
  color: var(--text-subtle);
  margin-top: var(--space-1);
}

.input-hint a {
  color: var(--primary);
}

.help-box {
  background: rgba(99, 102, 241, 0.05);
  border: 1px solid rgba(99, 102, 241, 0.1);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  margin-top: var(--space-4);
}

.help-box h4 {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--primary);
  margin-bottom: var(--space-2);
}

.help-box ol {
  margin-left: var(--space-4);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.help-box li {
  margin-bottom: var(--space-2);
}

.help-box code {
  background: var(--bg-subtle);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
}

.qrcode-section {
  margin-top: var(--space-6);
  display: flex;
  flex-direction: column;
  align-items: center;
}

.qrcode-result {
  margin-top: var(--space-4);
  text-align: center;
}

.qrcode-display {
  width: 200px;
  height: 200px;
  margin: 0 auto;
}

.qrcode-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--bg-subtle);
  border-radius: var(--radius-lg);
  gap: var(--space-2);
}

.qrcode-placeholder .icon {
  font-size: 48px;
}

.connection-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.url {
  background: var(--bg-subtle);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.hint {
  color: var(--text-muted);
  font-size: var(--text-xs);
  margin-top: var(--space-2);
}

.tunnel-status {
  margin-top: var(--space-4);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-light);
}

.tunnel-info {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.tunnel-info .label {
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.tunnel-actions {
  display: flex;
  gap: var(--space-2);
}

.save-bar {
  position: sticky;
  bottom: var(--space-4);
  display: flex;
  justify-content: center;
  padding: var(--space-4);
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}
</style>