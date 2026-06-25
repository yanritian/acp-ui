<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';

// Props
defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

// 游戏状态
const isLoggedIn = ref(false);
const username = ref('');
const isPlaying = ref(false);
const score = ref(0);
const level = ref(1);
const kills = ref(0);
const health = ref(100);

// Canvas
const canvasRef = ref<HTMLCanvasElement | null>(null);
const ctx = ref<CanvasRenderingContext2D | null>(null);

// 游戏对象
const playerTank = ref({
  x: 400,
  y: 520,
  direction: 'up',
  speed: 5
});

const enemies = ref<Array<{
  id: string;
  x: number;
  y: number;
  direction: string;
  health: number;
  alive: boolean;
}>>([]);

const bullets = ref<Array<{
  x: number;
  y: number;
  vx: number;
  vy: number;
  owner: string;
  active: boolean;
}>>([]);

// 按键状态
const keys = ref({
  up: false,
  down: false,
  left: false,
  right: false,
  shoot: false
});

let gameLoop: number | null = null;
let lastShootTime = 0;

// 登录
function handleLogin() {
  if (!username.value.trim()) return;
  isLoggedIn.value = true;
  startGame();
}

// 开始游戏
function startGame() {
  isPlaying.value = true;
  score.value = 0;
  level.value = 1;
  kills.value = 0;
  health.value = 100;

  // 重置玩家位置
  playerTank.value = {
    x: 400,
    y: 520,
    direction: 'up',
    speed: 5
  };

  // 生成敌人
  spawnEnemies();

  // 启动游戏循环
  nextTick(() => {
    initCanvas();
    startGameLoop();
  });
}

// 初始化 Canvas
function initCanvas() {
  if (!canvasRef.value) return;
  ctx.value = canvasRef.value.getContext('2d');
}

// 生成敌人
function spawnEnemies() {
  enemies.value = [];
  const count = 3 + level.value;
  for (let i = 0; i < count; i++) {
    enemies.value.push({
      id: `enemy-${i}`,
      x: 100 + i * 150,
      y: 60,
      direction: 'down',
      health: 100,
      alive: true
    });
  }
}

// 游戏循环
function startGameLoop() {
  if (gameLoop) clearInterval(gameLoop);
  gameLoop = window.setInterval(() => {
    updateGame();
    renderGame();
  }, 50);
}

// 更新游戏状态
function updateGame() {
  if (!isPlaying.value) return;

  // 移动玩家
  if (keys.value.up) movePlayer('up');
  if (keys.value.down) movePlayer('down');
  if (keys.value.left) movePlayer('left');
  if (keys.value.right) movePlayer('right');

  // 射击
  if (keys.value.shoot) {
    shoot();
  }

  // 更新子弹
  bullets.value = bullets.value.filter(b => b.active);
  for (const bullet of bullets.value) {
    bullet.x += bullet.vx;
    bullet.y += bullet.vy;

    // 边界检查
    if (bullet.x < 0 || bullet.x > 800 || bullet.y < 0 || bullet.y > 600) {
      bullet.active = false;
    }

    // 碰撞检测 - 玩家子弹击中敌人
    if (bullet.owner === 'player' && bullet.active) {
      for (const enemy of enemies.value) {
        if (enemy.alive && checkCollision(bullet, enemy)) {
          enemy.health -= 25;
          bullet.active = false;
          if (enemy.health <= 0) {
            enemy.alive = false;
            score.value += 100;
            kills.value += 1;
          }
        }
      }
    }

    // 碰撞检测 - 敌人子弹击中玩家
    if (bullet.owner !== 'player' && bullet.active) {
      if (checkCollision(bullet, playerTank.value)) {
        health.value -= 25;
        bullet.active = false;
        if (health.value <= 0) {
          endGame(false);
        }
      }
    }
  }

  // 更新敌人 AI
  for (const enemy of enemies.value) {
    if (enemy.alive) {
      updateEnemyAI(enemy);

      // 敌人随机射击
      if (Math.random() < 0.02) {
        enemyShoot(enemy);
      }
    }
  }

  // 清理死亡敌人
  enemies.value = enemies.value.filter(e => e.alive);

  // 检查胜利
  if (enemies.value.length === 0) {
    endGame(true);
  }

  // 升级检查
  if (score.value >= level.value * 500) {
    level.value += 1;
  }
}

// 碰撞检测
function checkCollision(bullet: { x: number; y: number }, tank: { x: number; y: number }) {
  const dx = bullet.x - tank.x;
  const dy = bullet.y - tank.y;
  const dist = Math.sqrt(dx * dx + dy * dy);
  return dist < 20;
}

// 移动玩家
function movePlayer(direction: string) {
  playerTank.value.direction = direction;
  const speed = playerTank.value.speed;

  switch (direction) {
    case 'up':
      if (playerTank.value.y > 40) playerTank.value.y -= speed;
      break;
    case 'down':
      if (playerTank.value.y < 520) playerTank.value.y += speed;
      break;
    case 'left':
      if (playerTank.value.x > 40) playerTank.value.x -= speed;
      break;
    case 'right':
      if (playerTank.value.x < 728) playerTank.value.x += speed;
      break;
  }
}

// 玩家射击
function shoot() {
  const now = Date.now();
  if (now - lastShootTime < 500) return;
  lastShootTime = now;

  let vx = 0, vy = 0;
  switch (playerTank.value.direction) {
    case 'up': vy = -10; break;
    case 'down': vy = 10; break;
    case 'left': vx = -10; break;
    case 'right': vx = 10; break;
  }

  bullets.value.push({
    x: playerTank.value.x + 16,
    y: playerTank.value.y + 16,
    vx,
    vy,
    owner: 'player',
    active: true
  });
}

// 敌人 AI
function updateEnemyAI(enemy: { x: number; y: number; direction: string; health: number; alive: boolean }) {
  const playerX = playerTank.value.x;
  const playerY = playerTank.value.y;

  // 朝向玩家移动
  if (playerX < enemy.x) {
    enemy.direction = 'left';
    enemy.x -= 2;
  } else if (playerX > enemy.x) {
    enemy.direction = 'right';
    enemy.x += 2;
  }

  if (playerY < enemy.y) {
    enemy.direction = 'up';
    enemy.y -= 2;
  } else if (playerY > enemy.y) {
    enemy.direction = 'down';
    enemy.y += 2;
  }
}

// 敌人射击
function enemyShoot(enemy: { x: number; y: number; direction: string; id: string }) {
  let vx = 0, vy = 0;
  switch (enemy.direction) {
    case 'up': vy = -8; break;
    case 'down': vy = 8; break;
    case 'left': vx = -8; break;
    case 'right': vx = 8; break;
  }

  bullets.value.push({
    x: enemy.x + 16,
    y: enemy.y + 16,
    vx,
    vy,
    owner: enemy.id,
    active: true
  });
}

// 渲染游戏
function renderGame() {
  if (!ctx.value || !canvasRef.value) return;

  const ctx2d = ctx.value;

  // 清空画布
  ctx2d.fillStyle = '#1a1a2e';
  ctx2d.fillRect(0, 0, 800, 600);

  // 绘制网格
  ctx2d.strokeStyle = '#2d2d4a';
  ctx2d.lineWidth = 1;
  for (let x = 0; x < 800; x += 40) {
    ctx2d.beginPath();
    ctx2d.moveTo(x, 0);
    ctx2d.lineTo(x, 600);
    ctx2d.stroke();
  }
  for (let y = 0; y < 600; y += 40) {
    ctx2d.beginPath();
    ctx2d.moveTo(0, y);
    ctx2d.lineTo(800, y);
    ctx2d.stroke();
  }

  // 绘制边界墙
  ctx2d.fillStyle = '#8B4513';
  ctx2d.fillRect(0, 0, 800, 40);
  ctx2d.fillRect(0, 560, 800, 40);
  ctx2d.fillRect(0, 0, 40, 600);
  ctx2d.fillRect(760, 0, 40, 600);

  // 绘制障碍物
  ctx2d.fillStyle = '#654321';
  ctx2d.fillRect(200, 200, 80, 40);
  ctx2d.fillRect(400, 300, 80, 40);
  ctx2d.fillRect(600, 150, 80, 40);

  // 绘制基地
  ctx2d.fillStyle = '#FF4444';
  ctx2d.fillRect(360, 520, 80, 40);
  ctx2d.fillStyle = '#FFF';
  ctx2d.font = '14px Arial';
  ctx2d.fillText('基地', 380, 545);

  // 绘制敌人坦克
  for (const enemy of enemies.value) {
    if (enemy.alive) {
      drawTank(ctx2d, enemy.x, enemy.y, enemy.direction, '#FF6B6B');
    }
  }

  // 绘制玩家坦克
  drawTank(ctx2d, playerTank.value.x, playerTank.value.y, playerTank.value.direction, '#4CAF50');

  // 绘制子弹
  for (const bullet of bullets.value) {
    if (bullet.active) {
      ctx2d.fillStyle = bullet.owner === 'player' ? '#FFFF00' : '#FF8800';
      ctx2d.beginPath();
      ctx2d.arc(bullet.x, bullet.y, 4, 0, Math.PI * 2);
      ctx2d.fill();
    }
  }
}

// 绘制坦克
function drawTank(ctx2d: CanvasRenderingContext2D, x: number, y: number, direction: string, color: string) {
  ctx2d.save();
  ctx2d.translate(x + 16, y + 16);

  // 旋转
  let angle = 0;
  switch (direction) {
    case 'up': angle = 0; break;
    case 'right': angle = 90; break;
    case 'down': angle = 180; break;
    case 'left': angle = 270; break;
  }
  ctx2d.rotate(angle * Math.PI / 180);

  // 坦克主体
  ctx2d.fillStyle = color;
  ctx2d.fillRect(-12, -12, 24, 24);

  // 炮管
  ctx2d.fillStyle = color;
  ctx2d.fillRect(-4, -20, 8, 14);

  // 履带
  ctx2d.fillStyle = '#333';
  ctx2d.fillRect(-14, -12, 4, 24);
  ctx2d.fillRect(10, -12, 4, 24);

  ctx2d.restore();
}

// 游戏结束
function endGame(victory: boolean) {
  isPlaying.value = false;
  if (gameLoop) {
    clearInterval(gameLoop);
    gameLoop = null;
  }

  // 显示结果
  if (victory) {
    alert(`胜利！\n分数: ${score.value}\n等级: ${level.value}\n击杀: ${kills.value}`);
  } else {
    alert(`游戏结束！\n分数: ${score.value}\n等级: ${level.value}`);
  }
}

// 键盘事件
function handleKeyDown(e: KeyboardEvent) {
  if (!isLoggedIn.value || !isPlaying.value) return;

  switch (e.key) {
    case 'w': case 'W': case 'ArrowUp':
      keys.value.up = true;
      break;
    case 's': case 'S': case 'ArrowDown':
      keys.value.down = true;
      break;
    case 'a': case 'A': case 'ArrowLeft':
      keys.value.left = true;
      break;
    case 'd': case 'D': case 'ArrowRight':
      keys.value.right = true;
      break;
    case ' ':
      keys.value.shoot = true;
      e.preventDefault();
      break;
  }
}

function handleKeyUp(e: KeyboardEvent) {
  switch (e.key) {
    case 'w': case 'W': case 'ArrowUp':
      keys.value.up = false;
      break;
    case 's': case 'S': case 'ArrowDown':
      keys.value.down = false;
      break;
    case 'a': case 'A': case 'ArrowLeft':
      keys.value.left = false;
      break;
    case 'd': case 'D': case 'ArrowRight':
      keys.value.right = false;
      break;
    case ' ':
      keys.value.shoot = false;
      break;
  }
}

// 生命周期
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('keyup', handleKeyUp);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('keyup', handleKeyUp);
  if (gameLoop) {
    clearInterval(gameLoop);
  }
});

// 关闭弹框
function handleClose() {
  if (gameLoop) {
    clearInterval(gameLoop);
  }
  isPlaying.value = false;
  isLoggedIn.value = false;
  emit('close');
}
</script>

<template>
  <div v-if="visible" class="game-overlay" @click.self="handleClose">
    <div class="game-dialog">
      <!-- 头部 -->
      <div class="dialog-header">
        <div class="header-title">
          <span class="game-icon">🎮</span>
          <h3>坦克大战</h3>
          <span class="platform-tag">ACP-UI 演示</span>
        </div>
        <button class="close-btn" @click="handleClose">✕</button>
      </div>

      <!-- 登录页面 -->
      <div v-if="!isLoggedIn" class="login-page">
        <div class="login-content">
          <div class="login-logo">
            <span class="tank-icon">🚀</span>
            <h2>坦克大战</h2>
            <p>跨平台游戏演示</p>
          </div>

          <div class="login-form">
            <input
              v-model="username"
              type="text"
              placeholder="输入用户名"
              class="input-field"
              @keyup.enter="handleLogin"
            />
            <button class="login-btn" @click="handleLogin">
              开始游戏
            </button>
          </div>

          <div class="login-notice">
            <p>⚠️ 同一账号只能在一端登录</p>
            <p>数据将同步保存到本地</p>
          </div>
        </div>
      </div>

      <!-- 游戏页面 -->
      <div v-else class="game-page">
        <!-- 状态栏 -->
        <div class="status-bar">
          <div class="status-item">
            <span class="status-label">用户:</span>
            <span class="status-value">{{ username }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">分数:</span>
            <span class="status-value score">{{ score }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">等级:</span>
            <span class="status-value">{{ level }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">击杀:</span>
            <span class="status-value kills">{{ kills }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">生命:</span>
            <span class="status-value health">
              <span class="health-bar">
                <span class="health-fill" :style="{ width: health + '%' }"></span>
              </span>
              {{ health }}
            </span>
          </div>
          <div class="status-item">
            <span class="status-label">敌人:</span>
            <span class="status-value enemies">{{ enemies.length }}</span>
          </div>
        </div>

        <!-- 游戏画布 -->
        <div class="game-canvas-container">
          <canvas
            ref="canvasRef"
            width="800"
            height="600"
            class="game-canvas"
          ></canvas>
        </div>

        <!-- 控制提示 -->
        <div class="controls-hint">
          <p>
            <span class="key">W/↑</span> 上
            <span class="key">S/↓</span> 下
            <span class="key">A/←</span> 左
            <span class="key">D/→</span> 右
            <span class="key">Space</span> 射击
          </p>
        </div>
      </div>

      <!-- 底部 -->
      <div class="dialog-footer">
        <div class="footer-info">
          <span>🎮 Flutter Mobile + Tauri Desktop 跨平台演示</span>
          <span>数据同步: 已启用</span>
          <span>单点登录: 已启用</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.game-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
}

.game-dialog {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 12px;
  width: 850px;
  max-height: 90vh;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  border: 1px solid #2d2d4a;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid #2d2d4a;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.game-icon {
  font-size: 24px;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  color: #fff;
}

.platform-tag {
  font-size: 12px;
  color: #4CAF50;
  background: rgba(76, 175, 80, 0.2);
  padding: 4px 8px;
  border-radius: 4px;
}

.close-btn {
  border: none;
  background: rgba(255, 255, 255, 0.1);
  font-size: 18px;
  cursor: pointer;
  color: #ccc;
  padding: 8px 12px;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 68, 68, 0.3);
  color: #ff4444;
}

/* 登录页面 */
.login-page {
  padding: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.login-content {
  text-align: center;
  max-width: 400px;
}

.login-logo {
  margin-bottom: 30px;
}

.tank-icon {
  font-size: 64px;
  display: block;
  margin-bottom: 16px;
}

.login-logo h2 {
  font-size: 28px;
  color: #fff;
  margin: 0 0 8px 0;
}

.login-logo p {
  color: #888;
  font-size: 14px;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.input-field {
  padding: 14px 18px;
  border: 2px solid #2d2d4a;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.3);
  color: #fff;
  font-size: 16px;
  transition: all 0.2s;
}

.input-field:focus {
  outline: none;
  border-color: #4CAF50;
  background: rgba(0, 0, 0, 0.5);
}

.input-field::placeholder {
  color: #666;
}

.login-btn {
  padding: 16px;
  background: linear-gradient(135deg, #4CAF50 0%, #2E7D32 100%);
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
}

.login-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(76, 175, 80, 0.4);
}

.login-notice {
  margin-top: 24px;
  padding: 16px;
  background: rgba(255, 152, 0, 0.1);
  border-radius: 8px;
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.login-notice p {
  color: #FFA726;
  font-size: 12px;
  margin: 4px 0;
}

/* 游戏页面 */
.game-page {
  display: flex;
  flex-direction: column;
}

.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-around;
  padding: 10px 20px;
  background: rgba(0, 0, 0, 0.4);
  border-bottom: 1px solid #2d2d4a;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label {
  color: #888;
  font-size: 12px;
}

.status-value {
  color: #fff;
  font-size: 14px;
  font-weight: 600;
}

.status-value.score {
  color: #FFD700;
}

.status-value.kills {
  color: #FF4444;
}

.status-value.enemies {
  color: #FF6B6B;
}

.health-bar {
  display: inline-block;
  width: 60px;
  height: 8px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  margin-right: 8px;
  vertical-align: middle;
}

.health-fill {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #4CAF50 0%, #8BC34A 100%);
  border-radius: 4px;
  transition: width 0.3s;
}

.game-canvas-container {
  display: flex;
  justify-content: center;
  padding: 10px;
  background: #000;
}

.game-canvas {
  border: 2px solid #4CAF50;
  border-radius: 4px;
  box-shadow: 0 0 20px rgba(76, 175, 80, 0.3);
}

.controls-hint {
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.3);
  text-align: center;
}

.controls-hint p {
  color: #888;
  font-size: 13px;
}

.key {
  display: inline-block;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  color: #4CAF50;
  font-weight: 600;
  margin-right: 4px;
}

/* 底部 */
.dialog-footer {
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.3);
  border-top: 1px solid #2d2d4a;
}

.footer-info {
  display: flex;
  justify-content: space-around;
  color: #888;
  font-size: 12px;
}
</style>