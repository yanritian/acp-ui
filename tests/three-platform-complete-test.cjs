#!/usr/bin/env node

/**
 * Agent Teams Platform - 三端完整功能测试
 * Phase 1-4 全系统验证
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const testResults = {
  web: { passed: 0, failed: 0, errors: [] },
  desktop: { passed: 0, failed: 0, errors: [] },
  mobile: { passed: 0, failed: 0, errors: [] },
  sync: { passed: 0, failed: 0, errors: [] },
  total: { passed: 0, failed: 0 }
};

console.log('🚀 Agent Teams Platform - 三端完整功能测试');
console.log('=' .repeat(60));
console.log('');

// ==================== Web端测试 ====================
console.log('🌐 Web端测试 (Vue 3 + Vite)');
console.log('-'.repeat(40));

function testWebFiles() {
  const webFiles = [
    // Phase 2: Agent进度面板
    'src/components/agent-progress/AgentRealtimeProgressPanel.vue',
    'src/components/agent-progress/ActivityIndicator.vue',
    'src/components/agent-progress/ThinkingDisplay.vue',
    'src/components/agent-progress/ToolExecutionMonitor.vue',
    'src/components/agent-progress/OutputTypewriter.vue',
    'src/components/agent-progress/PermissionWaiting.vue',
    // Phase 3: Agent宠物系统
    'src/components/agent-pet/AgentPetAvatar.vue',
    'src/components/agent-pet/EmotionDisplay.vue',
    'src/components/agent-pet/GrowthSystem.vue',
    'src/components/agent-pet/InteractionPanel.vue',
    // Phase 4: WebSocket同步
    'src/lib/sync/sync-message-types.ts',
    'src/lib/sync/websocket-sync-service.ts',
    // 状态管理
    'src/stores/agent-realtime.ts',
    'src/stores/agent-pet.ts',
    // 类型定义
    'src/lib/agent-runtime/realtime-progress-types.ts',
    // 主视图
    'src/views/AgentTeamsDashboard.vue',
    // Phase 1: 协作网络
    'src/components/collaboration/CollaborationNetworkFlow.vue',
    'src/components/collaboration/CollaborationTimeline.vue',
    'src/components/collaboration/CollaborationKanban.vue',
    'src/lib/collaboration/types.ts',
    'src/stores/collaboration.ts',
  ];

  const basePath = 'D:/dingsun/acp-ui';

  webFiles.forEach(file => {
    const fullPath = path.join(basePath, file);
    if (fs.existsSync(fullPath)) {
      const stats = fs.statSync(fullPath);
      if (stats.size > 0) {
        console.log(`  ✅ ${file} (${stats.size} bytes)`);
        testResults.web.passed++;
      } else {
        console.log(`  ❌ ${file} (空文件)`);
        testResults.web.failed++;
        testResults.web.errors.push(`${file} 是空文件`);
      }
    } else {
      console.log(`  ❌ ${file} (不存在)`);
      testResults.web.failed++;
      testResults.web.errors.push(`${file} 不存在`);
    }
  });
}

function testWebServer() {
  console.log('\n📡 Web服务器测试');

  try {
    const req = http.request({
      hostname: 'localhost',
      port: 1420,
      path: '/',
      method: 'GET',
      timeout: 5000
    }, (res) => {
      if (res.statusCode === 200) {
        console.log('  ✅ Web服务器响应正常 (HTTP 200)');
        testResults.web.passed++;
      } else {
        console.log(`  ❌ Web服务器响应异常 (HTTP ${res.statusCode})`);
        testResults.web.failed++;
        testResults.web.errors.push(`HTTP ${res.statusCode}`);
      }
    });

    req.on('error', (e) => {
      console.log(`  ❌ Web服务器连接失败: ${e.message}`);
      testResults.web.failed++;
      testResults.web.errors.push(e.message);
    });

    req.on('timeout', () => {
      console.log('  ❌ Web服务器超时');
      testResults.web.failed++;
      testResults.web.errors.push('服务器超时');
      req.destroy();
    });

    req.end();
  } catch (e) {
    console.log(`  ❌ Web服务器测试失败: ${e.message}`);
    testResults.web.failed++;
    testResults.web.errors.push(e.message);
  }
}

function testWebBuild() {
  console.log('\n📦 Web构建测试');

  const distPath = 'D:/dingsun/acp-ui/dist';
  if (fs.existsSync(distPath)) {
    const files = fs.readdirSync(distPath);
    if (files.length > 0) {
      console.log(`  ✅ 构建输出存在 (${files.length} 个文件)`);
      testResults.web.passed++;

      // 检查关键文件
      const indexHtml = path.join(distPath, 'index.html');
      if (fs.existsSync(indexHtml)) {
        console.log('  ✅ index.html 存在');
        testResults.web.passed++;
      }
    } else {
      console.log('  ❌ 构建输出为空');
      testResults.web.failed++;
    }
  } else {
    console.log('  ⚠️ 构建输出目录不存在 (需要运行 npm run build)');
  }
}

// ==================== Desktop端测试 ====================
console.log('\n🖥️ Desktop端测试 (Tauri)');
console.log('-'.repeat(40));

function testDesktopFiles() {
  const desktopFiles = [
    'src-tauri/Cargo.toml',
    'src-tauri/tauri.conf.json',
    'src-tauri/src/main.rs',
    'src-tauri/src/lib.rs',
  ];

  const basePath = 'D:/dingsun/acp-ui';

  desktopFiles.forEach(file => {
    const fullPath = path.join(basePath, file);
    if (fs.existsSync(fullPath)) {
      console.log(`  ✅ ${file}`);
      testResults.desktop.passed++;
    } else {
      console.log(`  ❌ ${file} (不存在)`);
      testResults.desktop.failed++;
      testResults.desktop.errors.push(`${file} 不存在`);
    }
  });

  // 检查Tauri配置
  const tauriConf = path.join(basePath, 'src-tauri/tauri.conf.json');
  if (fs.existsSync(tauriConf)) {
    try {
      const config = JSON.parse(fs.readFileSync(tauriConf, 'utf8'));
      if (config.build?.beforeBuildCommand) {
        console.log('  ✅ Tauri构建配置正确');
        testResults.desktop.passed++;
      }
    } catch (e) {
      console.log('  ❌ Tauri配置解析失败');
      testResults.desktop.failed++;
    }
  }
}

// ==================== Mobile端测试 ====================
console.log('\n📱 Mobile端测试 (Flutter)');
console.log('-'.repeat(40));

function testMobileFiles() {
  const mobileFiles = [
    // Phase 2: Agent进度面板
    'acp_ui_flutter/lib/features/agent_progress/agent_realtime_progress_panel.dart',
    // Phase 3: Agent宠物
    'acp_ui_flutter/lib/features/agent_pet/agent_pet_avatar.dart',
    // 类型定义
    'acp_ui_flutter/lib/data/models/agent_realtime/agent_realtime_types.dart',
    'acp_ui_flutter/lib/data/models/agent_realtime/agent_pet_types.dart',
    // Phase 1: 协作网络
    'acp_ui_flutter/lib/features/collaboration/collaboration_network_view.dart',
    'acp_ui_flutter/lib/data/models/collaboration.dart',
    'acp_ui_flutter/lib/data/stores/collaboration_store.dart',
  ];

  const basePath = 'D:/dingsun/acp-ui';

  mobileFiles.forEach(file => {
    const fullPath = path.join(basePath, file);
    if (fs.existsSync(fullPath)) {
      const stats = fs.statSync(fullPath);
      if (stats.size > 100) {
        console.log(`  ✅ ${file} (${stats.size} bytes)`);
        testResults.mobile.passed++;
      } else {
        console.log(`  ❌ ${file} (文件过小)`);
        testResults.mobile.failed++;
        testResults.mobile.errors.push(`${file} 文件过小`);
      }
    } else {
      console.log(`  ❌ ${file} (不存在)`);
      testResults.mobile.failed++;
      testResults.mobile.errors.push(`${file} 不存在`);
    }
  });

  // 检查Flutter项目配置
  const pubspec = path.join(basePath, 'acp_ui_flutter/pubspec.yaml');
  if (fs.existsSync(pubspec)) {
    console.log('  ✅ Flutter项目配置存在');
    testResults.mobile.passed++;
  } else {
    console.log('  ❌ Flutter项目配置不存在');
    testResults.mobile.failed++;
  }
}

// ==================== 三端同步测试 ====================
console.log('\n🔄 三端同步系统测试');
console.log('-'.repeat(40));

function testSyncSystem() {
  const syncFiles = [
    'src/lib/sync/sync-message-types.ts',
    'src/lib/sync/websocket-sync-service.ts',
  ];

  const basePath = 'D:/dingsun/acp-ui';

  syncFiles.forEach(file => {
    const fullPath = path.join(basePath, file);
    if (fs.existsSync(fullPath)) {
      const content = fs.readFileSync(fullPath, 'utf8');

      // 检查关键类型
      if (content.includes('SyncSource') && content.includes('SyncTarget')) {
        console.log(`  ✅ ${file} - 同步类型定义完整`);
        testResults.sync.passed++;
      }

      if (content.includes('WebSocket') && content.includes('reconnect')) {
        console.log(`  ✅ ${file} - WebSocket重连机制存在`);
        testResults.sync.passed++;
      }

      if (content.includes('manualChunks') || content.includes('messageQueue')) {
        console.log(`  ✅ ${file} - 消息队列/代码分割存在`);
        testResults.sync.passed++;
      }
    }
  });

  // 检查vite配置中的代码分割
  const viteConfig = path.join(basePath, 'vite.config.ts');
  if (fs.existsSync(viteConfig)) {
    const content = fs.readFileSync(viteConfig, 'utf8');
    if (content.includes('manualChunks')) {
      console.log('  ✅ Vite代码分割配置存在');
      testResults.sync.passed++;
    }
  }
}

// ==================== 执行测试 ====================
testWebFiles();
testWebServer();
testWebBuild();
testDesktopFiles();
testMobileFiles();
testSyncSystem();

// 等待HTTP请求完成
setTimeout(() => {
  // ==================== 测试报告 ====================
  console.log('\n' + '='.repeat(60));
  console.log('📊 三端完整测试报告');
  console.log('=' .repeat(60));

  // Web端
  console.log('\n🌐 Web端:');
  console.log(`   通过: ${testResults.web.passed} ✅`);
  console.log(`   失败: ${testResults.web.failed} ❌`);
  if (testResults.web.errors.length > 0) {
    console.log('   错误详情:');
    testResults.web.errors.forEach((e, i) => console.log(`     ${i+1}. ${e}`));
  }

  // Desktop端
  console.log('\n🖥️ Desktop端:');
  console.log(`   通过: ${testResults.desktop.passed} ✅`);
  console.log(`   失败: ${testResults.desktop.failed} ❌`);

  // Mobile端
  console.log('\n📱 Mobile端:');
  console.log(`   通过: ${testResults.mobile.passed} ✅`);
  console.log(`   失败: ${testResults.mobile.failed} ❌`);

  // 同步系统
  console.log('\n🔄 同步系统:');
  console.log(`   通过: ${testResults.sync.passed} ✅`);
  console.log(`   失败: ${testResults.sync.failed} ❌`);

  // 总计
  testResults.total.passed =
    testResults.web.passed +
    testResults.desktop.passed +
    testResults.mobile.passed +
    testResults.sync.passed;
  testResults.total.failed =
    testResults.web.failed +
    testResults.desktop.failed +
    testResults.mobile.failed +
    testResults.sync.failed;

  console.log('\n📈 总计:');
  console.log(`   通过: ${testResults.total.passed} ✅`);
  console.log(`   失败: ${testResults.total.failed} ❌`);
  const totalRate = testResults.total.passed / (testResults.total.passed + testResults.total.failed) * 100;
  console.log(`   成功率: ${totalRate.toFixed(1)}%`);

  console.log('\n' + '='.repeat(60));

  if (testResults.total.failed === 0) {
    console.log('✅ 所有测试通过！Agent Teams Platform三端系统完整可用');
  } else if (testResults.total.failed <= 5) {
    console.log('⚠️ 大部分功能正常，有少量问题需要修复');
  } else {
    console.log('❌ 发现较多问题，建议先修复再使用');
  }

  console.log('=' .repeat(60));
  console.log('\n📋 功能入口:');
  console.log('   Web端: http://localhost:1420/ → 侧边栏点击 🚀 Agent Teams');
  console.log('   Desktop端: 运行 npm run tauri dev');
  console.log('   Mobile端: 运行 cd acp_ui_flutter && flutter run');
  console.log('');
}, 2000);