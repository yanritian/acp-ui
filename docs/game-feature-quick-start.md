# Game Development Feature - Quick Start Guide

## 🚀 5分钟快速开始

### 1. 访问游戏管理界面

**方式一：侧边栏图标**
- 点击左侧边栏的 🎮 图标
- 自动跳转到游戏管理界面

**方式二：直接路由**
- 在浏览器地址栏输入：`http://localhost:1420/#/games`
- 或在应用内导航到 `/games`

### 2. 检测游戏项目

1. 在"项目目录"输入框中输入游戏项目路径
   - 例如：`D:/my-godot-game`
   - 或：`D:/my-unity-game`

2. 点击 **"检测游戏"** 按钮

3. 查看检测结果：
   - ✅ 游戏引擎（Godot/Unity）
   - ✅ 项目名称
   - ✅ 版本信息
   - ✅ 场景数量

### 3. 导出游戏

1. 在"导出平台"区域选择目标平台：
   - 🪟 Windows
   - 🍎 macOS
   - 🐧 Linux
   - 🌐 Web

2. 点击 **"导出游戏"** 按钮

3. 观察导出进度：
   - 进度条实时显示
   - 当前阶段提示
   - 预估剩余时间

4. 导出完成后：
   - 查看输出文件路径
   - 查看文件大小
   - 查看构建时间

### 4. 启动游戏

1. 在"运行游戏"区域：
   - 选择可执行文件（自动检测或手动选择）
   - 点击 **"启动游戏"**

2. 游戏将在新窗口中打开

3. 在"运行状态"区域可以：
   - 查看运行时间
   - 查看内存使用
   - 点击 **"停止游戏"** 结束进程

## 📋 前置要求

### Godot 项目

**必需文件：**
- `project.godot` - 项目配置文件

**推荐结构：**
```
my-godot-game/
├── project.godot
├── scenes/
│   ├── main.tscn
│   └── level1.tscn
├── scripts/
│   └── player.gd
└── assets/
    ├── textures/
    └── audio/
```

### Unity 项目

**必需目录：**
- `Assets/` - 资源目录
- `ProjectSettings/` - 项目设置目录

**推荐结构：**
```
my-unity-game/
├── Assets/
│   ├── Scenes/
│   │   └── Main.unity
│   ├── Scripts/
│   │   └── Player.cs
│   └── Materials/
├── ProjectSettings/
│   └── ProjectVersion.txt
└── Packages/
    └── manifest.json
```

## 🔧 常见问题

### Q: 检测失败，提示"未找到游戏项目"

**A:** 检查以下几点：
1. 路径是否正确（使用绝对路径）
2. 目录是否存在
3. 是否包含必需文件：
   - Godot: `project.godot`
   - Unity: `Assets/` 和 `ProjectSettings/`

### Q: 导出失败，提示"未找到引擎"

**A:** 需要安装游戏引擎：
- **Godot**: 
  - 下载：https://godotengine.org/download
  - 安装到：`D:/tools/godot/`
  - 设置环境变量：`GODOT_PATH=D:/tools/godot/godot.exe`

- **Unity**:
  - 下载：https://unity.com/download
  - 安装 Unity Hub
  - 设置环境变量：`UNITY_PATH=C:/Program Files/Unity/Hub/Editor/2021.3.0f1/Editor/Unity.exe`

### Q: 导出很慢

**A:** 导出时间取决于：
- 项目大小（资源数量）
- 目标平台
- 电脑性能

**预期时间：**
- 小型项目（< 50MB）：10-30秒
- 中型项目（50-500MB）：1-5分钟
- 大型项目（> 500MB）：5-30分钟

### Q: 游戏启动后没有反应

**A:** 检查：
1. 可执行文件是否正确
2. 是否有运行权限
3. 查看控制台日志
4. 尝试手动运行可执行文件

### Q: 如何查看详细的构建日志？

**A:** 
1. 在 GameManager 界面底部有日志区域
2. 点击 **"显示完整日志"** 按钮
3. 可以复制日志内容用于调试

## 💡 使用技巧

### 技巧 1: 使用环境变量

创建 `D:/dingsun/acp-ui/.env` 文件：
```env
GODOT_PATH=D:/tools/godot/godot.exe
UNITY_PATH=C:/Program Files/Unity/Hub/Editor/2021.3.0f1/Editor/Unity.exe
```

### 技巧 2: 批量导出

可以连续导出多个平台：
1. 导出 Windows 版本
2. 等待完成
3. 切换到 macOS
4. 再次导出

### 技巧 3: 监控性能

在"运行状态"区域可以实时查看：
- 📊 内存使用
- ⏱️ 运行时间
- 🔄 帧率（如果游戏支持）

### 技巧 4: 使用快捷键

- `Ctrl + G` - 快速跳转到游戏管理
- `Ctrl + E` - 快速导出
- `Ctrl + R` - 快速运行

## 🎯 完整工作流示例

### 场景：开发一个 Godot 小游戏

**步骤 1: 创建项目**
```bash
# 使用 Godot 编辑器创建新项目
# 保存至：D:/my-first-game
```

**步骤 2: 开发游戏**
```bash
# 在 Godot 编辑器中开发
# 添加场景、脚本、资源
```

**步骤 3: 在 ACP-UI 中检测**
```
1. 打开 ACP-UI
2. 导航到 /games
3. 输入：D:/my-first-game
4. 点击"检测游戏"
5. 确认检测成功
```

**步骤 4: 导出为 Windows**
```
1. 选择平台：Windows
2. 点击"导出游戏"
3. 等待导出完成
4. 查看输出：D:/my-first-game/export/game.exe
```

**步骤 5: 测试游戏**
```
1. 点击"启动游戏"
2. 游戏窗口打开
3. 测试游戏功能
4. 点击"停止游戏"结束
```

**步骤 6: 发布游戏**
```
1. 导出 Release 版本
2. 压缩 export 目录
3. 上传到 itch.io 或其他平台
```

## 📚 更多信息

- **完整文档**: [game-development-guide.md](./game-development-guide.md)
- **测试用例**: [game-feature-test-cases.md](./game-feature-test-cases.md)
- **API 参考**: 见完整文档的 API 部分
- **故障排除**: 见本文的"常见问题"部分

## 🆘 获取帮助

如果遇到问题：
1. 查看本文的"常见问题"
2. 查看完整文档
3. 查看构建日志
4. 检查游戏引擎是否正确安装
5. 确认项目结构是否正确

---

**最后更新**: 2026-06-25  
**版本**: 1.0.0
