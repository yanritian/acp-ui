# Hermes Game Operator - 用户指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 简介

Hermes Game Operator 是一个由操作员掌控的游戏开发 Agent 控制系统。它让开发者能够使用 AI Agent 参与真实游戏项目开发，同时保留最终决策权和项目安全边界。

### 核心特性

- ✅ **任务管理**：创建、暂停、恢复、停止和重定向任务
- ✅ **两级审批**：计划审批和文件修改审批
- ✅ **实时事件流**：监控任务执行进度
- ✅ **多平台支持**：Desktop (Tauri)、VSCode、IDEA、Web
- ✅ **完整国际化**：支持 11 种语言
- ✅ **安全增强**：OIDC/OAuth2、RBAC、TLS、审计日志

---

## 快速开始

### 1. 安装

#### Desktop 版本 (Tauri)

```bash
# 下载最新 release
# 或从源码构建
cd D:/dingsun/acp-ui
npm install
npm run build
```

#### VSCode 扩展

```bash
# 在 VSCode 中搜索 "Hermes Game Operator"
# 或手动安装
cd clients/vscode-game-operator
npm install
npm run compile
```

#### IDEA 插件

```bash
# 构建插件
cd clients/idea-game-operator
./gradlew buildPlugin

# 在 IDEA 中安装
# Settings → Plugins → ⚙️ → Install Plugin from Disk
```

### 2. 配置

#### Desktop 配置

编辑 `config.json`：

```json
{
  "serverUrl": "http://localhost:8080",
  "authToken": "your-token-here",
  "defaultProjectPath": "/path/to/your/godot/project"
}
```

#### VSCode 配置

在 VSCode 设置中添加：

```json
{
  "gameOperator.serverUrl": "http://localhost:8080",
  "gameOperator.authToken": "your-token-here"
}
```

#### IDEA 配置

在 IDEA 设置中：
- Settings → Tools → Hermes Game Operator
- 输入服务器 URL 和认证令牌

### 3. 连接服务器

1. 启动 Game Operator 服务器
2. 在客户端中点击"Connect"
3. 确认连接状态显示为"Online"

---

## 使用指南

### 创建任务

#### 步骤 1：选择项目

1. 点击"Select Project"或"Browse"
2. 选择 Godot 项目目录
3. 系统会自动检测项目类型

#### 步骤 2：输入目标

1. 在"Task Goal"字段输入任务目标
2. 描述你希望 Agent 完成的工作
3. 示例：
   - "Add double jump to the player character"
   - "Fix collision detection issues"
   - "Optimize rendering performance"

#### 步骤 3：启动任务

1. 点击"Start Task"
2. 等待系统生成执行计划
3. 审批计划后开始执行

### 审批任务

#### 计划审批

1. 查看 Agent 生成的执行计划
2. 检查步骤和预期文件修改
3. 点击"Approve"或"Reject"

#### 文件修改审批

1. 查看文件差异（diff）
2. 检查修改内容
3. 点击"Approve"、"Reject"或"Request Changes"

### 监控任务

#### 查看事件流

1. 打开"Timeline"标签页
2. 实时查看任务执行事件
3. 了解当前步骤和进度

#### 查看任务状态

1. 在"Tasks"列表中查看任务
2. 状态说明：
   - **Idle**：空闲
   - **Planning**：规划中
   - **Waiting Approval**：等待审批
   - **Running**：运行中
   - **Paused**：已暂停
   - **Completed**：已完成
   - **Failed**：失败

### 控制任务

#### 暂停任务

1. 选择运行中的任务
2. 点击"Pause"
3. 任务会暂停在当前步骤

#### 恢复任务

1. 选择已暂停的任务
2. 点击"Resume"
3. 任务会从暂停处继续

#### 停止任务

1. 选择任务
2. 点击"Stop"
3. 确认停止操作

#### 重定向任务

1. 选择运行中的任务
2. 输入新的目标
3. 点击"Redirect"
4. 任务会调整方向

---

## 安全特性

### 认证

- **OIDC/OAuth2**：支持标准认证协议
- **JWT Token**：安全的令牌机制
- **Bearer Token**：API 访问认证

### 授权

- **RBAC**：基于角色的访问控制
- **Project Scope**：项目级别的权限控制
- **Permission Management**：细粒度权限管理

### 审计

- **Audit Log**：完整的操作审计日志
- **Replay Protection**：重放攻击防护
- **Rate Limiting**：请求速率限制

### 安全头

- **CSP**：内容安全策略
- **HSTS**：HTTP 严格传输安全
- **X-Frame-Options**：防止点击劫持
- **X-Content-Type-Options**：防止 MIME 类型嗅探

---

## 国际化

### 支持的语言

1. 🇨🇳 中文 (zh-CN)
2. 🇺🇸 英文 (en-US)
3. 🇩🇪 德语 (de-DE)
4. 🇯🇵 日语 (ja-JP)
5. 🇰🇷 韩语 (ko-KR)
6. 🇫🇷 法语 (fr-FR)
7. 🇪🇸 西班牙语 (es-ES)
8. 🇷🇺 俄语 (ru-RU)
9. 🇹🇭 泰语 (th-TH)
10. 🇻🇳 越南语 (vi-VN)
11. 🇲🇾 马来语 (ms-MY)

### 切换语言

#### Desktop

在设置中选择语言

#### VSCode

在设置中设置 `locale`

#### IDEA

在 IDEA 语言设置中切换

---

## 故障排除

### 连接失败

**问题**：无法连接到服务器

**解决方案**：
1. 检查服务器是否运行
2. 确认服务器 URL 正确
3. 检查网络连接
4. 查看防火墙设置

### 任务失败

**问题**：任务执行失败

**解决方案**：
1. 查看任务事件日志
2. 检查错误信息
3. 确认项目路径正确
4. 检查文件权限

### 审批问题

**问题**：无法审批任务

**解决方案**：
1. 确认你有审批权限
2. 检查认证令牌是否有效
3. 查看审计日志

---

## 最佳实践

### 任务目标

1. **明确具体**：描述清晰的任务目标
2. **可测试**：目标应该是可验证的
3. **范围适当**：避免过大的任务范围

### 审批流程

1. **仔细审查**：审批前仔细查看计划
2. **逐步审批**：复杂任务分阶段审批
3. **记录决策**：记录审批原因

### 监控

1. **定期检查**：定期查看任务进度
2. **关注异常**：注意异常事件
3. **及时干预**：发现问题及时暂停或停止

---

## 高级用法

### 自定义 Agent 配置

编辑 `agent-config.json`：

```json
{
  "model": "claude-3-opus",
  "maxIterations": 100,
  "timeoutSeconds": 3600,
  "allowedTools": ["file.read", "file.patch", "godot.analyze"]
}
```

### 批量操作

使用 API 进行批量操作：

```bash
# 批量创建任务
curl -X POST http://localhost:8080/api/tasks/batch \
  -H "Authorization: Bearer your-token" \
  -d '[{"goal": "Task 1"}, {"goal": "Task 2"}]'
```

### 自动化工作流

结合 CI/CD 实现自动化：

```yaml
# .github/workflows/game-operator.yml
name: Game Operator
on: [push]
jobs:
  operator:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Game Operator
        uses: hermes-game-operator/action@v1
        with:
          goal: "Review code changes"
```

---

## 获取帮助

### 文档

- [API 文档](./API.md)
- [开发者指南](./DEVELOPER.md)
- [架构文档](./ARCHITECTURE.md)

### 社区

- GitHub: https://github.com/your-org/hermes-game-operator
- Discord: [加入社区](https://discord.gg/your-invite)
- 论坛: https://forum.example.com

### 支持

- 邮箱: support@example.com
- 问题反馈: https://github.com/your-org/hermes-game-operator/issues

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Team