# 坦克大战项目需求分析

## 项目概述
- **名称**: Tank Battle（坦克大战）
- **目标**: 创建跨平台坦克大战游戏，支持手机端和桌面端
- **核心功能**: 两端数据同步、账号单点登录

## 技术架构

### 平台端
| 平台 | 技术栈 | 位置 |
|------|--------|------|
| 手机端 | Flutter + Dart | D:/tmp/tank-battle-game/flutter-mobile |
| 桌面端 | Tauri + Rust + Vue | D:/tmp/tank-battle-game/tauri-desktop |
| 共享核心 | Rust Core | D:/tmp/tank-battle-game/shared-core |
| 同步服务 | WebSocket Server | D:/tmp/tank-battle-game/sync-server |

### 功能模块

#### 1. 游戏核心逻辑 (shared-core)
- 坦克类 (Tank)
- 子弹类 (Bullet)
- 地图类 (Map)
- 碰撞检测 (CollisionDetector)
- 游戏状态 (GameState)

#### 2. Flutter 手机端
- 游戏画面渲染 (Canvas)
- 触控控制 (虚拟方向键 + 射击按钮)
- 账号登录界面
- 游戏进度显示

#### 3. Tauri 桌面端
- 游戏画面渲染 (Canvas/WebGL)
- 键盘控制 (WASD + Space)
- 账号登录界面
- 游戏进度显示

#### 4. 数据同步服务
- WebSocket 实时通信
- 游戏进度同步
- 账号状态管理
- 单点登录控制

## 单点登录机制

### 规则
1. 同一账号只能在一端登录
2. 手机端登录 → 桌面端被踢出
3. 桌面端登录 → 手机端被踢出
4. 登录状态实时同步

### 实现
- Session Token 存储
- 设备 ID 标识
- 登录状态广播
- 强制踢出机制

## 数据同步机制

### 同步内容
- 用户等级
- 游戏关卡进度
- 最高分数
- 成就列表
- 装备配置

### 同步时机
- 登录时全量同步
- 游戏结束时增量同步
- 关卡切换时同步

## 验收标准

1. ✅ Flutter 手机端可运行游戏
2. ✅ Tauri 桌面端可运行游戏
3. ✅ 两端游戏逻辑一致
4. ✅ 数据同步正常工作
5. ✅ 单点登录限制生效
6. ✅ 禁止 C 盘操作

## 开发流程监控

每个步骤需要记录：
- 输入内容
- 输出结果
- 执行时间
- 成功/失败状态
- 错误信息（如有）