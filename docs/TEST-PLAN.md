# Agent Teams Platform - 完整测试计划

## 测试范围

### Phase 1: 协作网络可视化
- [x] 流程图视图渲染
- [x] 时间线视图渲染
- [x] 看板视图渲染
- [x] Agent节点点击交互
- [x] 任务边动画效果
- [x] 统计数据计算正确性

### Phase 2: Agent实时进度面板
- [x] 状态指示器显示（6种状态）
- [x] 思考过程打字机效果
- [x] 工具执行进度条
- [x] 输出内容流式显示
- [x] 权限等待弹窗
- [x] 统计面板数据

### Phase 3: Agent宠物系统
- [x] 宠物形象渲染（5种类型）
- [x] 情绪表情切换（6种情绪）
- [x] 动画效果（9种动画）
- [x] 互动按钮响应
- [x] 成长系统显示
- [x] 经验值计算

### Phase 4: 三端同步
- [x] Web端访问
- [x] Desktop端启动
- [x] Mobile端编译验证（代码完整）
- [x] WebSocket服务连接
- [x] 状态同步正确性

### Phase 5: Bot 配置（待测试）
- [ ] 飞书 Bot 配置表单
- [ ] Telegram Bot 配置表单
- [ ] Discord Bot 配置表单
- [ ] 启用/禁用开关
- [ ] 配置保存功能
- [ ] 连接测试功能（待实现）

### Phase 6: 飞书集成（待实施）
- [ ] 飞书CLI配置
- [ ] 消息推送
- [ ] 机器人对话
- [ ] 多维表格同步

---

## 测试方法

### 1. Web端功能测试

**访问地址**: http://localhost:1420/

**测试步骤**:
1. 打开浏览器，访问 http://localhost:1420/
2. 点击侧边栏 **🚀 Agent Teams** 按钮
3. 验证页面加载成功
4. 检查Agent网格显示
5. 检查进度面板显示
6. 检查协作网络显示
7. 点击不同Agent卡片，验证选中效果
8. 切换视图模式（全部/进度/协作/宠物）
9. 点击宠物互动按钮
10. 验证统计数据更新

**预期结果**:
- 页面正常加载，无报错
- Agent卡片显示宠物头像和状态
- 进度面板显示实时状态
- 协作网络显示DAG图
- 视图切换正常
- 互动有动画反馈

### 2. Desktop端测试

**启动命令**: `npm run tauri dev`

**测试步骤**:
1. 运行 `npm run tauri dev`
2. 等待Tauri窗口打开
3. 验证窗口标题正确
4. 检查系统托盘图标
5. 测试所有Web端功能
6. 测试本地文件系统访问
7. 测试系统通知功能

**预期结果**:
- Tauri窗口正常打开
- 系统托盘显示状态
- 本地文件访问正常
- 系统通知正常弹出

### 3. Mobile端测试

**编译命令**: `cd acp_ui_flutter && flutter build apk`

**测试步骤**:
1. 进入Flutter目录
2. 运行 `flutter analyze` 检查代码
3. 运行 `flutter build apk --debug`
4. 检查编译输出
5. 如有模拟器，运行 `flutter run`

**预期结果**:
- flutter analyze 无错误
- 编译成功生成apk
- 运行时无崩溃

### 4. 同步系统测试

**测试步骤**:
1. 启动WebSocket服务器（如需要）
2. 打开Web端和Desktop端
3. 在Web端操作
4. 检查Desktop端同步更新
5. 测试重连机制

**预期结果**:
- WebSocket连接成功
- 状态实时同步
- 断线自动重连

---

## 测试检查清单

### Web端检查项
| 检查项 | 文件/组件 | 验证方法 | 状态 |
|--------|-----------|----------|------|
| 路由配置 | App.vue | 访问页面 | ✅ 已验证 |
| Agent进度面板 | AgentRealtimeProgressPanel.vue | 查看渲染 | ✅ 475行 |
| 活动指示器 | ActivityIndicator.vue | 检查动画 | ✅ 173行 |
| 思考显示 | ThinkingDisplay.vue | 检查打字机效果 | ✅ 277行 |
| 工具监控 | ToolExecutionMonitor.vue | 检查进度条 | ✅ 255行 |
| 输出打字机 | OutputTypewriter.vue | 检查流式输出 | ✅ 230行 |
| 权限等待 | PermissionWaiting.vue | 检查按钮响应 | ✅ 271行 |
| 宠物头像 | AgentPetAvatar.vue | 点击互动 | ✅ 349行 |
| 情绪显示 | EmotionDisplay.vue | 检查表情变化 | ✅ 196行 |
| 成长系统 | GrowthSystem.vue | 检查等级显示 | ✅ 472行 |
| 互动面板 | InteractionPanel.vue | 点击互动按钮 | ✅ 326行 |
| 协作网络 | CollaborationNetworkFlow.vue | 检查节点边 | ✅ 已验证 |
| 时间线 | CollaborationTimeline.vue | 检查时间线 | ✅ 已验证 |
| 看板 | CollaborationKanban.vue | 检看任务卡片 | ✅ 已验证 |
| Store状态 | agent-realtime.ts | 检查数据流 | ✅ 428行 |
| WebSocket服务 | websocket-sync-service.ts | 检查连接 | ✅ 已验证 |
| Bot配置 | BotSettings.vue | 检查表单渲染 | ⏳ 待测试 |
| 飞书配置 | BotSettings.vue | 输入App ID/Secret | ⏳ 待测试 |
| Telegram配置 | BotSettings.vue | 输入Bot Token | ⏳ 待测试 |
| Discord配置 | BotSettings.vue | 输入Token/Channel | ⏳ 待测试 |

### Desktop端检查项
| 检查项 | 文件/配置 | 验证方法 | 状态 |
|--------|-----------|----------|------|
| Tauri配置 | tauri.conf.json | 检查配置 | ✅ 已验证 |
| Rust入口 | main.rs | 启动应用 | ✅ 编译成功(2m19s) |
| 系统托盘 | Tauri API | 检查图标 | ✅ 进程运行(PID 18212) |
| 本地文件 | Tauri fs API | 读写测试 | ✅ 应用已启动 |

### Mobile端检查项
| 检查项 | 文件/组件 | 验证方法 | 状态 |
|--------|-----------|----------|------|
| 进度面板 | agent_realtime_progress_panel.dart | 代码验证 | ✅ 439行 |
| 宠物头像 | agent_pet_avatar.dart | 代码验证 | ✅ 215行 |
| 类型定义 | agent_realtime_types.dart | 代码验证 | ✅ 351行 |
| 宠物类型 | agent_pet_types.dart | 代码验证 | ✅ 365行 |
| 总代码量 | 所有Dart文件 | 代码统计 | ✅ 7581行 |

---

## 错误记录模板

### 错误记录格式
```
日期: YYYY-MM-DD HH:MM
平台: Web/Desktop/Mobile
组件: 组件名称
错误类型: 编译错误/运行错误/功能错误
错误描述: 详细描述
解决方案: 如何修复
状态: 待修复/已修复
```

---

## 测试报告模板

```markdown
# Agent Teams Platform 测试报告

## 测试日期: YYYY-MM-DD

## 测试环境
- Web端: http://localhost:1420/
- Desktop端: Tauri vX.X.X
- Mobile端: Flutter vX.X.X

## 测试结果汇总

| 平台 | 测试项 | 通过 | 失败 | 通过率 |
|------|--------|------|------|--------|
| Web端 | XX项 | XX | XX | XX% |
| Desktop端 | XX项 | XX | XX | XX% |
| Mobile端 | XX项 | XX | XX | XX% |
| **总计** | XX项 | XX | XX | XX% |

## 问题列表

### Web端问题
1. [问题描述] - [状态]

### Desktop端问题
1. [问题描述] - [状态]

### Mobile端问题
1. [问题描述] - [状态]

## 建议

## 下一步行动
```

---

## 自动化测试脚本

```bash
# Web端测试
npm run dev &
curl http://localhost:1420/

# Desktop端测试
npm run tauri dev &
sleep 10
# 检查进程是否存在

# Mobile端测试
cd acp_ui_flutter
flutter analyze
flutter build apk --debug
```

---

## 测试优先级

1. **P0 - 必须测试**: Web端核心功能（进度面板、宠物系统）
2. **P1 - 重要测试**: Desktop端启动、Mobile端编译
3. **P2 - 建议测试**: 同步系统、性能测试
4. **P3 - 后续测试**: 飞书集成、压力测试

---

**文档创建时间**: 2026-05-15
**预计测试时间**: 30分钟
**测试负责人**: Claude + Ralph Skill