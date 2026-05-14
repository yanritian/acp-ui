# ACP-UI 多端协同Agent系统 - 系统架构文档

## 系统概述

本系统是一个**本地优先**的多端协同Agent系统，支持桌面端（Tauri）、移动端和CLI协同工作，用于控制开发工具（HBuilderX、微信小程序、Android Studio、浏览器）。

## 架构分层

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Client Layer                               │
├────────────────────┬────────────────────┬────────────────────────────┤
│   Desktop (Tauri)  │   Mobile (Tauri)   │      Web (H5 Preview)      │
│   WebView + Rust   │   WebView + Rust   │      WebSocket Client      │
└────────────────────┴────────────────────┴────────────────────────────┘
                              │
                              ▼ WebSocket / ACP Protocol
┌─────────────────────────────────────────────────────────────────────┐
│                        Agent Orchestration Layer                     │
├────────────────────┬────────────────────┬────────────────────────────┤
│      Hermes        │   QA Agents        │      Task Parser           │
│   (Orchestrator)   │ (CodeReview/Test)  │      (DAG Builder)         │
│   Agent Matcher    │                    │                            │
└────────────────────┴────────────────────┴────────────────────────────┘
                              │
                              ▼ Agent Sandbox System
┌─────────────────────────────────────────────────────────────────────┐
│                          Agent Sandbox Layer                         │
├────────────────────┬────────────────────┬────────────────────────────┤
│   MCP Isolation    │   Skills Isolation │      Hooks Isolation       │
│   Per-Agent MCP    │   Per-Agent Skills │      Per-Agent Hooks       │
│   Process Manager  │   Loader           │      Executor              │
└────────────────────┴────────────────────┴────────────────────────────┘
                              │
                              ▼ Tool Adapters
┌─────────────────────────────────────────────────────────────────────┐
│                        Adapter Layer (Phase 4)                       │
├────────────────────┬────────────────────┬────────────────────────────┤
│   BrowserAdapter   │  HBuilderXAdapter  │  WeChatDevToolsAdapter     │
│   CDP WebSocket    │   CLI Wrapper      │      CLI + API Monitor     │
│                    │                    │                            │
│   AndroidStudioAdapter                                              │
│   Gradle + ADB Wrapper                                              │
└────────────────────┴────────────────────┴────────────────────────────┘
                              │
                              ▼ Local Infrastructure
┌─────────────────────────────────────────────────────────────────────┐
│                        Infrastructure Layer                          │
├────────────────────┬────────────────────┬────────────────────────────┤
│   LogStream        │   Permission       │      Config Parser         │
│   SQLite Storage   │   Checker          │      YAML Parser           │
│   WebSocket Push   │   Regex Rules      │                            │
│                    │                    │                            │
│   Ngrok Tunnel     │   Offline Cache    │      Command Queue         │
│   Public URL       │   Mobile Storage   │      Reconnect             │
└────────────────────┴────────────────────┴────────────────────────────┘
```

## 核心组件详解

### Phase 1: 最小可用产品 ✅ 完成

#### 1.1 LogStream系统
- **文件**: `src-tauri/src/log_stream.rs`, `src/components/LogStreamView.vue`
- **功能**: 实时日志捕获、分类、存储、推送
- **特性**:
  - CircularBuffer环形缓冲区
  - 7种日志类型分类
  - SQLite持久化存储
  - WebSocket批量推送（100ms间隔）
  - 虚拟滚动渲染（10000+条不卡顿）

#### 1.2 PermissionChecker
- **文件**: `src-tauri/src/permission_checker.rs`
- **功能**: 命令权限检查、工作目录限制
- **特性**:
  - 正则匹配allow/deny规则
  - 危险命令检测（rm -rf, /etc/*）
  - cwd路径限制
  - 内置测试用例

#### 1.3 AgentConfigParser
- **文件**: `src-tauri/src/agent_config_parser.rs`
- **功能**: YAML配置解析、验证、存储
- **特性**:
  - 简化版YAML解析器
  - 必填字段验证
  - SQLite存储

#### 1.4 Ngrok隧道
- **文件**: `src-tauri/src/tunnel.rs`
- **功能**: ngrok进程管理、公网URL获取
- **特性**:
  - 真实API调用（ureq HTTP客户端）
  - 进程启动/停止
  - 状态监控

### Phase 2: Agent沙箱基础 ✅ 完成

#### 2.1 MCP隔离
- **文件**: `src-tauri/src/mcp_manager.rs`
- **功能**: MCP进程管理、隔离验证
- **特性**:
  - stdio/WebSocket/HTTP三种传输
  - instance_key隔离格式
  - validate_access方法

#### 2.2 Skills隔离
- **文件**: `src/lib/skills-loader.ts`
- **功能**: Skills动态加载、调用路由
- **特性**:
  - 默认Skills定义
  - validateAccess方法
  - Agent-Skills绑定

#### 2.3 Hooks隔离
- **文件**: `src-tauri/src/hooks_executor.rs`
- **功能**: PreToolUse/PostToolUse执行
- **特性**:
  - 脚本执行（带timeout）
  - BLOCK/DENY检测
  - 输出捕获

### Phase 3: Hermes基础 ✅ 完成

#### 3.1 TaskParser
- **文件**: `src/lib/task-parser.ts`
- **功能**: 任务模板解析、DAG构建
- **特性**:
  - 3个内置模板（miniprogram/android/browser）
  - 模板匹配（关键词触发）
  - DAG构建（依赖关系）
  - 并行任务识别

#### 3.2 AgentMatcher
- **文件**: `src/lib/agent-matcher.ts`
- **功能**: Agent能力匹配、负载均衡
- **特性**:
  - Jaccard相似度计算
  - 能力向量匹配
  - 负载均衡（考虑currentLoad）
  - 系统负载分析

#### 3.3 Orchestrator
- **文件**: `src/lib/orchestrator.ts`
- **功能**: 任务编排、事件处理
- **特性**:
  - 任务分配（findBestAgent）
  - EventBus（subscribe/emit）
  - TaskComplete/TaskFailed处理
  - 依赖阻塞（blockDependents）
  - 重试机制

#### 3.4 QA Agents
- **文件**: `src/lib/code-review-agent.ts`, `src/lib/test-validator-agent.ts`
- **功能**: 代码审查、测试验证
- **特性**:
  - CodeReviewAgent: tsc/eslint/安全扫描
  - TestValidatorAgent: npm test/覆盖率检查
  - Pass/Fail决策
  - 报告生成

#### 3.5 Hermes UI
- **文件**: `src/components/HermesDashboard.vue`, `src/components/TaskGraphView.vue`
- **功能**: 状态可视化、任务图展示
- **特性**:
  - 任务列表、Agent状态
  - QA验证状态
  - SVG DAG可视化
  - 任务详情面板

### Phase 4: 多客户端适配器 ✅ 完成

#### 4.1 BrowserAdapter
- **文件**: `src/lib/browser-adapter.ts`
- **功能**: Chrome浏览器控制
- **特性**:
  - CDP WebSocket连接
  - Page.navigate/Runtime.evaluate
  - Input.dispatchMouseEvent（点击）
  - Page.captureScreenshot（截图）
  - Console监听、Network监控

#### 4.2 HBuilderXAdapter
- **文件**: `src/lib/hbuilderx-adapter.ts`
- **功能**: HBuilderX小程序控制
- **特性**:
  - CLI命令封装
  - compile/run/build/package
  - 设备列表获取
  - 日志解析

#### 4.3 WeChatDevToolsAdapter
- **文件**: `src/lib/wechat-devtools-adapter.ts`
- **功能**: 微信小程序调试
- **特性**:
  - CLI命令封装
  - preview/upload/build
  - API调用监控
  - 性能分析

#### 4.4 AndroidStudioAdapter
- **文件**: `src/lib/android-studio-adapter.ts`
- **功能**: Android构建和部署
- **特性**:
  - Gradle命令封装
  - buildDebug/buildRelease
  - ADB设备管理
  - APK安装
  - logcat日志

### Phase 5: 完善与优化 ✅ 完成

#### 5.1 离线缓存
- **文件**: `src/lib/offline-cache.ts`
- **功能**: 移动端离线数据存储
- **特性**:
  - LogCache（100条）
  - ApiCallCache（50条）
  - StateCache
  - LRU淘汰策略

#### 5.2 命令队列
- **文件**: `src/lib/command-queue.ts`
- **功能**: 断线命令暂存
- **特性**:
  - 优先级队列
  - 重试机制
  - 批量发送（flush）
  - 防重复执行

## QA验证机制

```
QA验证流程：
├── CodeReviewAgent审查
│   ├── npx tsc --noEmit
│   ├── npx eslint
│   ├── 安全扫描（grep模式）
│   └── Pass/Fail决策
│
├── TestValidatorAgent验证
│   ├── npm test
│   ├── 覆盖率检查（≥80%）
│   └── Pass/Fail决策
│
└── Hermes决策
    ├── QA Pass → 进入下一Phase
    ├── QA Fail → 阻断，通知修复
```

## 数据流

```
用户请求 → TaskParser → DAG
         ↓
    AgentMatcher → Agent分配
         ↓
    Orchestrator → 任务执行
         ↓
    QA Agents → 验证
         ↓
    Hermes → Pass/Fail决策
         ↓
    WebSocket → 手机端显示
```

## 技术栈

| 层次 | 技术 |
|------|------|
| 桌面端 | Tauri (Rust + WebView) |
| 移动端 | Tauri Mobile |
| 前端 | Vue 3 + TypeScript |
| 后端 | Rust |
| 数据库 | SQLite |
| 协议 | WebSocket + ACP |
| 隧道 | ngrok |

## 完成状态

| Phase | 状态 | 完成日期 |
|-------|------|----------|
| Phase 0: MVP验证 | ✅ 完成 | 2026-05-10 |
| Phase 1: 最小可用产品 | ✅ 完成 | 2026-05-10 |
| Phase 2: Agent沙箱基础 | ✅ 完成 | 2026-05-10 |
| Phase 3: Hermes基础 | ✅ 完成 | 2026-05-11 |
| Phase 4: 多客户端适配器 | ✅ 完成 | 2026-05-11 |
| Phase 5: 完善与优化 | ✅ 完成 | 2026-05-11 |

**系统状态**: ✅ 全部Phase完成，系统可用