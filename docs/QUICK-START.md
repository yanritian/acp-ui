# 快速开始指南

> 版本: v0.1.0-alpha
> 阅读时间: 5分钟

---

## 🚀 5分钟快速体验

### 1. 安装 (30秒)

#### Web 版本 (最快)
直接访问: https://acp-ui.github.io/

#### 桌面版本
```bash
# Windows
下载 .msi 安装包 → 双击安装

# macOS
下载 .dmg → 拖入 Applications

# Linux
下载 .deb / .AppImage / .rpm → 安装
```

### 2. 创建第一个操作员 (1分钟)

1. 打开 ACP-UI
2. 点击 "New Operator"
3. 选择你的 Godot 项目路径
4. 点击 "Create"

### 3. 启动操作员 (30秒)

1. 选中刚创建的操作员
2. 点击 "Start" 按钮
3. 观察进度

### 4. 处理审批 (1分钟)

1. 点击右侧 "Approval Queue"
2. 查看待审批项
3. 点击 "Approve" 或 "Reject"

### 5. 查看结果 (30秒)

1. 查看 "Progress Timeline"
2. 检查执行结果
3. 完成！

---

## 💻 开发者快速开始

### 环境要求
- Node.js >= 18.0.0
- npm >= 9.0.0
- Rust >= 1.70.0

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://github.com/yanritian/acp-ui.git
cd acp-ui

# 2. 安装依赖
npm install

# 3. 启动开发服务器
npm run dev

# 4. 打开浏览器
# 访问 http://localhost:1420
```

### 运行测试

```bash
# 所有测试
npm test

# 监听模式
npm run test:watch

# E2E 测试
npm run test:e2e
```

### 构建生产版本

```bash
# 类型检查
npm run typecheck

# 构建
npm run build

# 桌面应用
cd src-tauri
cargo tauri build
```

---

## 📝 第一个示例

### 创建简单的 Godot 项目

```gdscript
# scripts/Player.gd
extends CharacterBody2D

var speed = 200

func _process(delta):
    var direction = Input.get_vector("ui_left", "ui_right", "ui_up", "ui_down")
    velocity = direction * speed
    move_and_slide()
```

### 在 ACP-UI 中使用

1. 创建操作员指向项目目录
2. 启动操作员
3. Agent 会自动分析项目
4. 处理审批请求
5. 查看修改结果

---

## 🎯 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` | 新建操作员 |
| `Ctrl+Enter` | 启动操作员 |
| `Ctrl+Shift+A` | 打开审批队列 |
| `Ctrl+Shift+S` | 保存设置 |
| `Ctrl+Q` | 退出应用 |

---

## 🔗 下一步

- [API 文档](API-DOCUMENTATION.md) - 学习如何使用 API
- [部署指南](DEPLOYMENT-GUIDE.md) - 部署到生产环境
- [架构文档](ARCHITECTURE.md) - 了解系统架构
- [FAQ](FAQ.md) - 常见问题解答

---

## 🆘 需要帮助？

- [GitHub Issues](https://github.com/yanritian/acp-ui/issues)
- [Discussions](https://github.com/yanritian/acp-ui/discussions)

---

<div align="center">

**🎉 恭喜！你已经完成了快速开始！**

[继续学习 →](API-DOCUMENTATION.md)

</div>
