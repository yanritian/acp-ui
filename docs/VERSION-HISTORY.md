# Hermes Game Operator 版本历史

## v0.1.0-alpha (2026-07-09)

### 新增功能

#### 核心功能
- 任务生命周期管理（创建、暂停、继续、停止、重定向）
- 10 状态状态机
- 事件流跟踪
- 任务总结生成

#### 安全功能
- PathGuard 路径边界验证
- CommandGuard 命令白名单
- 注入防护（防止命令注入、路径遍历）
- API Key 验证

#### Godot 集成
- 项目检测（project.godot）
- 项目分析（scripts、scenes、assets）
- 场景解析
- 玩家控制器识别

#### 审批流程
- 审批队列管理
- 多级审批
- 审批历史记录

#### Hermes CLI 集成
- CLI 检测和连接状态
- 项目分析
- 计划生成
- 步骤执行

### API

- 20+ Tauri 命令
- TypeScript 类型定义
- Rust 类型定义

### 测试

- 413 个测试通过
- 22 个测试文件
- 覆盖率: API、安全、性能、错误处理、边缘情况

### 文档

- API 参考
- 配置指南
- 架构文档
- 快速参考
- 故障排除指南
- 测试覆盖率报告

### 工具

- 构建脚本
- 发布脚本
- 环境检查脚本
- Docker 配置
- CI/CD 配置
- Makefile

---

## 已知限制

1. **Windows SDK 缺失** - cargo check 无法运行
2. **Hermes CLI 未安装** - Agent 执行使用 mock 实现
3. **仅支持 Godot** - Unity/Ren'Py 待扩展

---

## 下一步计划

### v0.2.0

- 真实 Hermes Agent API 集成
- 完整闭环验证
- 性能优化

### v0.3.0

- Unity Domain Pack
- Ren'Py Domain Pack
- VSCode 插件集成

### v1.0.0

- 生产级稳定性
- 完整文档
- 社区支持