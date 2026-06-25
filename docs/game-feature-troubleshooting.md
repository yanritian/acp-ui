# Game Development Feature - Troubleshooting Guide

## 🔍 问题诊断流程

### 第一步：确认问题类型

```
问题发生
    ↓
能否检测游戏？
    ├─ 否 → 查看"检测失败"章节
    └─ 是 ↓
      能否导出游戏？
          ├─ 否 → 查看"导出失败"章节
          └─ 是 ↓
            能否启动游戏？
                ├─ 否 → 查看"启动失败"章节
                └─ 是 ↓
                  游戏运行正常？
                      ├─ 否 → 查看"运行问题"章节
                      └─ 是 → 问题已解决 ✅
```

---

## ❌ 检测失败

### 问题 1: "未找到游戏项目"

**症状：**
```
错误：未找到游戏项目
请确认目录包含 project.godot (Godot) 或 Assets/ (Unity)
```

**原因：**
- 路径不正确
- 目录不存在
- 缺少必需文件

**解决方案：**

1. **检查路径格式**
   ```bash
   # ❌ 错误：相对路径
   my-game
   
   # ✅ 正确：绝对路径
   D:/my-game
   C:/Users/Name/my-game
   ```

2. **验证目录存在**
   ```bash
   # Windows
   dir D:\my-game
   
   # 应该看到项目文件
   ```

3. **检查必需文件**
   
   **Godot 项目：**
   ```bash
   # 必须有 project.godot 文件
   dir D:\my-game\project.godot
   ```
   
   **Unity 项目：**
   ```bash
   # 必须有 Assets 和 ProjectSettings 目录
   dir D:\my-game\Assets
   dir D:\my-game\ProjectSettings
   ```

4. **重新创建项目**
   ```bash
   # Godot
   # 使用 Godot 编辑器创建新项目
   # 保存时确保生成 project.godot
   
   # Unity
   # 使用 Unity Hub 创建新项目
   # 确保生成 Assets 和 ProjectSettings
   ```

---

### 问题 2: "检测到项目但信息不完整"

**症状：**
```
警告：项目检测成功，但部分信息缺失
- 项目名称：未知
- 版本：未知
```

**原因：**
- 项目配置文件损坏
- 缺少版本信息
- 配置文件格式错误

**解决方案：**

1. **检查 project.godot 格式**
   ```ini
   # 正确的格式
   [application]
   config/name="My Game"
   config/features=PackedStringArray("4.2", "GL Compatibility")
   ```

2. **重新生成配置文件**
   ```bash
   # 使用 Godot 编辑器打开项目
   # 保存设置，重新生成 project.godot
   ```

3. **手动编辑配置**
   ```bash
   # 用文本编辑器打开 project.godot
   # 添加缺失的信息
   notepad D:\my-game\project.godot
   ```

---

## ❌ 导出失败

### 问题 3: "未找到游戏引擎"

**症状：**
```
错误：未找到 Godot 引擎
请安装 Godot 或设置 GODOT_PATH 环境变量
```

**原因：**
- 引擎未安装
- 环境变量未设置
- 引擎路径不正确

**解决方案：**

**Godot：**

1. **下载安装**
   ```
   1. 访问 https://godotengine.org/download
   2. 下载 Windows 版本
   3. 解压到 D:/tools/godot/
   4. 确认 godot.exe 存在
   ```

2. **设置环境变量**
   ```bash
   # 方法 1: 系统环境变量
   # 右键"此电脑" → 属性 → 高级系统设置 → 环境变量
   # 新建系统变量：
   #   变量名：GODOT_PATH
   #   变量值：D:\tools\godot\godot.exe
   
   # 方法 2: 临时设置（当前终端）
   set GODOT_PATH=D:\tools\godot\godot.exe
   ```

3. **验证安装**
   ```bash
   %GODOT_PATH% --version
   # 应该输出版本号，如：4.2.1.stable
   ```

**Unity：**

1. **安装 Unity Hub**
   ```
   1. 访问 https://unity.com/download
   2. 下载并安装 Unity Hub
   3. 通过 Hub 安装 Unity Editor
   ```

2. **设置环境变量**
   ```bash
   # 典型路径
   set UNITY_PATH=C:\Program Files\Unity\Hub\Editor\2021.3.0f1\Editor\Unity.exe
   ```

3. **验证安装**
   ```bash
   "%UNITY_PATH%" -version
   ```

---

### 问题 4: "导出模板缺失"

**症状：**
```
错误：缺少导出模板
请安装对应平台的导出模板
```

**原因：**
- Godot 导出模板未安装
- 模板版本不匹配

**解决方案：**

1. **安装导出模板**
   ```
   1. 打开 Godot 编辑器
   2. 编辑器 → 管理导出模板
   3. 点击"下载并安装"
   4. 等待下载完成（约 500MB）
   ```

2. **验证模板安装**
   ```bash
   # 模板位置
   dir %APPDATA%\Godot\export_templates
   
   # 应该看到版本目录，如 4.2.1.stable
   ```

3. **匹配版本**
   ```bash
   # 检查项目版本
   type D:\my-game\project.godot | findstr features
   
   # 确保导出的模板版本匹配
   ```

---

### 问题 5: "编译错误"

**症状：**
```
错误：脚本编译失败
第 42 行：Unexpected token
```

**原因：**
- 脚本语法错误
- 缺少依赖
- 资源引用错误

**解决方案：**

1. **查看完整错误日志**
   ```
   1. 在 GameManager 界面
   2. 点击"显示完整日志"
   3. 查找 ERROR 或 FAILED 行
   ```

2. **在引擎编辑器中修复**
   ```bash
   # Godot
   # 用 Godot 编辑器打开项目
   # 查看"脚本"面板的错误
   # 修复语法错误
   
   # Unity
   # 用 Unity 编辑器打开项目
   # 查看控制台的错误
   # 修复编译错误
   ```

3. **常见错误修复**
   
   **GDScript 错误：**
   ```gdscript
   # ❌ 错误：缺少冒号
   func _ready()
       pass
   
   # ✅ 正确
   func _ready():
       pass
   ```
   
   **C# 错误：**
   ```csharp
   // ❌ 错误：缺少分号
   int x = 5
   
   // ✅ 正确
   int x = 5;
   ```

---

### 问题 6: "磁盘空间不足"

**症状：**
```
错误：无法写入文件
磁盘空间不足
```

**原因：**
- 目标磁盘空间不足
- 临时文件过多

**解决方案：**

1. **检查磁盘空间**
   ```bash
   # Windows
   wmic logicaldisk get size,freespace,caption
   
   # 至少需要 2GB 可用空间
   ```

2. **清理临时文件**
   ```bash
   # 清理 Godot 缓存
   rmdir /s /q %APPDATA%\Godot\app_userdata
   
   # 清理 Unity 缓存
   rmdir /s /q %LOCALAPPDATA%\Unity\Cache
   ```

3. **更改输出目录**
   ```
   在导出时选择不同的输出目录
   选择空间充足的磁盘
   ```

---

## ❌ 启动失败

### 问题 7: "可执行文件未找到"

**症状：**
```
错误：无法找到可执行文件
路径：D:/my-game/export/game.exe
```

**原因：**
- 导出未完成
- 文件被移动或删除
- 路径不正确

**解决方案：**

1. **确认文件存在**
   ```bash
   dir D:\my-game\export\game.exe
   ```

2. **重新导出**
   ```
   1. 返回导出界面
   2. 重新选择平台
   3. 点击"导出游戏"
   4. 等待完成
   ```

3. **手动指定路径**
   ```
   在"运行游戏"区域
   点击"浏览"按钮
   手动选择可执行文件
   ```

---

### 问题 8: "权限被拒绝"

**症状：**
```
错误：无法启动进程
权限被拒绝
```

**原因：**
- 缺少运行权限
- 文件被占用
- 防病毒软件拦截

**解决方案：**

1. **以管理员身份运行**
   ```
   1. 右键 ACP-UI 快捷方式
   2. 选择"以管理员身份运行"
   ```

2. **检查文件权限**
   ```bash
   # 右键 game.exe → 属性 → 安全
   # 确保当前用户有"读取和执行"权限
   ```

3. **解除文件锁定**
   ```bash
   # 右键 game.exe → 属性
   # 如果看到"解除锁定"按钮，点击它
   ```

4. **添加防病毒白名单**
   ```
   1. 打开 Windows 安全中心
   2. 病毒和威胁防护 → 管理设置
   3. 排除项 → 添加排除项
   4. 添加游戏目录
   ```

---

## ⚠️ 运行问题

### 问题 9: "游戏启动后黑屏"

**症状：**
```
游戏窗口打开，但显示黑屏
```

**原因：**
- 显卡驱动问题
- 渲染模式不兼容
- 资源加载失败

**解决方案：**

1. **更新显卡驱动**
   ```
   1. 访问显卡制造商网站
   2. 下载最新驱动
   3. 安装并重启
   ```

2. **更改渲染模式（Godot）**
   ```ini
   # 编辑 project.godot
   [rendering]
   driver="gl_compatibility"  # 改为兼容性模式
   ```

3. **查看游戏日志**
   ```bash
   # Godot 日志位置
   type %APPDATA%\Godot\app_userdata\my-game\logs\godot.log
   
   # 查找 ERROR 行
   ```

---

### 问题 10: "游戏运行卡顿"

**症状：**
```
游戏帧率低，运行不流畅
```

**原因：**
- 性能不足
- 优化不够
- 后台进程占用

**解决方案：**

1. **检查性能监控**
   ```
   在 GameManager 的"运行状态"区域
   查看 CPU 和内存使用率
   ```

2. **关闭后台程序**
   ```bash
   # 打开任务管理器
   Ctrl + Shift + Esc
   
   # 结束不必要的进程
   ```

3. **降低游戏画质**
   ```
   在游戏设置中降低：
   - 分辨率
   - 阴影质量
   - 纹理质量
   ```

4. **优化游戏代码**
   ```
   检查是否有：
   - 过多的绘制调用
   - 未优化的循环
   - 大量实例化的对象
   ```

---

## 🛠️ 高级故障排除

### 查看完整日志

**Godot 日志：**
```bash
# 运行时日志
type %APPDATA%\Godot\app_userdata\GAME_NAME\logs\godot.log

# 导出日志
type %TEMP%\godot_export.log
```

**Unity 日志：**
```bash
# Windows
type %LOCALAPPDATA%\Unity\Editor\Editor.log

# macOS
cat ~/Library/Logs/Unity/Editor.log
```

**ACP-UI 日志：**
```
1. 在 GameManager 界面
2. 点击"显示完整日志"
3. 可以复制或导出日志
```

### 调试模式

**启用调试输出：**
```bash
# 设置环境变量
set ACP_DEBUG=1

# 启动 ACP-UI
npm run tauri dev
```

**查看网络请求：**
```
1. 按 F12 打开开发者工具
2. 切换到 Network 标签
3. 查看 API 请求和响应
```

### 重置配置

**重置游戏功能配置：**
```bash
# 删除配置文件
del %APPDATA%\acp-ui\game-config.json

# 重启应用
```

---

## 📞 获取帮助

### 自助资源

1. **文档**
   - [完整文档](./game-development-guide.md)
   - [快速开始](./game-feature-quick-start.md)
   - [测试用例](./game-feature-test-cases.md)

2. **日志分析**
   - 查看构建日志
   - 查看运行日志
   - 查看错误信息

3. **社区**
   - Godot 论坛：https://godotengine.org/community
   - Unity 论坛：https://forum.unity.com

### 报告问题

如果问题无法解决，请提供：

1. **系统信息**
   ```
   - 操作系统版本
   - ACP-UI 版本
   - 游戏引擎版本
   ```

2. **错误日志**
   ```
   - 完整的错误信息
   - 构建日志
   - 运行日志
   ```

3. **重现步骤**
   ```
   1. 打开 ACP-UI
   2. 导航到 /games
   3. 输入项目路径：...
   4. 点击...
   5. 出现错误：...
   ```

---

## ✅ 验证清单

问题解决后，验证以下内容：

- [ ] 可以检测 Godot 项目
- [ ] 可以检测 Unity 项目
- [ ] 可以导出 Windows 版本
- [ ] 可以导出 macOS 版本
- [ ] 可以导出 Linux 版本
- [ ] 可以启动游戏
- [ ] 可以停止游戏
- [ ] 性能监控正常
- [ ] 错误提示清晰
- [ ] 日志记录完整

---

**最后更新**: 2026-06-25  
**版本**: 1.0.0
