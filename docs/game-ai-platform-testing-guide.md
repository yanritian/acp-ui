# 游戏开发 AI 平台 - 测试和部署指南

**日期**: 2026-06-26
**状态**: ⚠️ 代码完成，需要手动测试

---

## 一、当前状态

### ✅ 已完成

1. **Ren'Py Adapter**
   - 后端：680 行 Rust 代码
   - 前端：500 行 Vue 组件
   - 命令：7 个 Tauri 命令
   - 测试：Python 集成测试脚本

2. **GameDesignerAgent**
   - 后端：750 行 Rust 代码
   - 前端：600 行 Vue 组件
   - 命令：3 个 Tauri 命令

3. **GameDeveloperAgent**
   - 后端：800 行 Rust 代码
   - 前端：550 行 Vue 组件
   - 命令：3 个 Tauri 命令

### ⚠️ 需要手动处理

1. **Tauri 构建权限问题**
   - 错误："拒绝访问 (os error 5)"
   - 原因：系统安全策略或防病毒软件阻止
   - 解决：需要管理员权限或调整安全设置

2. **依赖项**
   - ✅ tempfile 已添加到 dev-dependencies
   - ✅ recommend_engine 方法已改为 public

### ❌ 未完成

1. **完整应用构建** - 需要解决 Tauri 权限问题
2. **前端实际测试** - 需要启动开发服务器
3. **端到端测试** - 需要完整应用运行

---

## 二、解决 Tauri 构建问题

### 方案 1: 以管理员身份运行

1. 右键点击命令提示符或 PowerShell
2. 选择"以管理员身份运行"
3. 导航到项目目录
4. 运行 `cargo build`

### 方案 2: 禁用防病毒软件实时保护

1. 打开 Windows 安全中心
2. 进入"病毒和威胁防护"
3. 临时禁用实时保护
4. 运行 `cargo build`
5. 完成后重新启用保护

### 方案 3: 添加排除项

1. 打开 Windows 安全中心
2. 进入"病毒和威胁防护" → "管理设置"
3. 在"排除项"中添加：
   - `D:\dingsun\acp-ui\src-tauri\target\`
   - `C:\Users\Administrator\.cargo\`

### 方案 4: 清理并重建

```bash
cd D:\dingsun\acp-ui\src-tauri
cargo clean
cargo build --release
```

---

## 三、测试步骤

### 3.1 后端测试

#### 编译检查
```bash
cd D:\dingsun\acp-ui\src-tauri
cargo check
```
**期望**: 无错误（可能有警告）

#### 单元测试
```bash
cargo test --lib
```
**期望**: 所有测试通过

#### Ren'Py 集成测试
```bash
python D:\tmp\test_renpy_adapter.py
```
**期望**: 7/7 测试通过

### 3.2 前端测试

#### 启动开发服务器
```bash
cd D:\dingsun\acp-ui
npm run dev
```
**期望**: 开发服务器启动在 http://localhost:1420

#### 访问界面
1. 打开浏览器访问 http://localhost:1420
2. 导航到游戏开发页面
3. 测试以下功能：
   - Ren'Py Manager（项目创建、脚本生成）
   - Game Designer（GDD 生成）
   - Game Developer（代码生成）

### 3.3 完整应用测试

#### 构建应用
```bash
cd D:\dingsun\acp-ui\src-tauri
cargo tauri build
```
**期望**: 生成可执行文件

#### 运行应用
```bash
# Windows
.\target\release\acp-ui.exe

# macOS
open ./target/release/AcpUi.app

# Linux
./target/release/acp-ui
```

---

## 四、功能测试清单

### 4.1 Ren'Py Adapter

- [ ] 获取 Ren'Py 版本
- [ ] 创建新项目
- [ ] 检测现有项目
- [ ] 生成游戏脚本
- [ ] 运行游戏
- [ ] 编译游戏
- [ ] 代码检查

### 4.2 GameDesignerAgent

- [ ] 输入游戏创意
- [ ] 生成完整 GDD
- [ ] 查看游戏概念
- [ ] 查看技术架构
- [ ] 查看核心机制
- [ ] 查看系统设计
- [ ] 查看资源需求
- [ ] 查看开发计划
- [ ] 推荐游戏引擎

### 4.3 GameDeveloperAgent

- [ ] 输入 GDD JSON
- [ ] 生成完整代码
- [ ] 查看生成的文件列表
- [ ] 查看单个文件内容
- [ ] 复制代码
- [ ] 查看支持的引擎
- [ ] 快速生成特定文件

---

## 五、已知问题

### 5.1 Tauri 构建权限

**问题**: `cargo build` 失败，提示"拒绝访问"

**可能原因**:
- 防病毒软件阻止构建脚本执行
- Windows 安全策略限制
- 文件系统权限不足

**解决方案**:
- 以管理员身份运行
- 临时禁用防病毒软件
- 添加排除项

### 5.2 前端路由

**问题**: 新创建的游戏页面可能未注册到路由

**解决方案**:
检查 `src/router/index.ts`，确保以下路由已添加：
```typescript
{
  path: '/games/renpy',
  component: () => import('../features/games/RenPyManager.vue')
},
{
  path: '/games/designer',
  component: () => import('../features/games/GameDesigner.vue')
},
{
  path: '/games/developer',
  component: () => import('../features/games/GameDeveloper.vue')
}
```

### 5.3 Tauri 命令注册

**问题**: 命令可能未正确注册

**验证方法**:
检查 `src-tauri/src/lib.rs` 中的 `invoke_handler`，确保以下命令已注册：
- `renpy_*` (7 个命令)
- `game_designer_*` (3 个命令)
- `game_developer_*` (3 个命令)

---

## 六、部署检查清单

### 6.1 构建前检查

- [ ] 所有代码编译通过 (`cargo check`)
- [ ] 所有测试通过 (`cargo test`)
- [ ] 前端无错误 (`npm run build`)
- [ ] Tauri 配置正确 (`tauri.conf.json`)

### 6.2 构建检查

- [ ] 后端构建成功
- [ ] 前端构建成功
- [ ] 资源文件打包正确
- [ ] 图标和元数据正确

### 6.3 发布前检查

- [ ] 所有功能测试通过
- [ ] 性能测试通过
- [ ] 安全审查完成
- [ ] 文档完整

---

## 七、后续开发计划

### 7.1 短期（1-2 周）

1. **资源 Agent**
   - 集成 DALL-E 生成角色立绘
   - 集成 Suno 生成背景音乐
   - 自动导入到项目

2. **反思 Agent**
   - 自动测试游戏
   - 评估游戏质量
   - 提供改进建议

### 7.2 中期（1 个月）

1. **Unreal Engine Adapter**
   - 安装 Unreal Engine
   - 实现 Adapter
   - 支持大型 3D 游戏

2. **高级编辑器**
   - 可视化场景编辑
   - 拖拽式对话编辑
   - 实时预览

### 7.3 长期（3 个月）

1. **协作功能**
   - 多人编辑
   - 版本控制
   - 评论系统

2. **发布平台**
   - itch.io 集成
   - Steam 集成
   - 移动端发布

---

## 八、联系和支持

### 8.1 问题反馈

如果遇到问题，请提供：
1. 错误信息截图
2. 操作步骤
3. 系统信息（OS、版本）
4. 日志文件

### 8.2 贡献指南

欢迎贡献！请：
1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

---

## 九、总结

### 当前完成度

- ✅ **代码编写**: 100%
- ✅ **编译检查**: 100%
- ⚠️ **构建测试**: 0% (Tauri 权限问题)
- ⚠️ **功能测试**: 0% (需要构建)
- ⚠️ **前端验证**: 0% (需要运行)

### 下一步行动

1. **立即**: 解决 Tauri 构建权限问题
2. **本周**: 完成所有功能测试
3. **下周**: 修复发现的问题
4. **本月**: 发布第一版

---

**文档版本**: 1.0
**最后更新**: 2026-06-26
**维护者**: AI Agent
