# 故障排除指南

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 常见问题

### 1. 构建失败

**问题**: npm run build 失败

**解决方案**:
```bash
# 清理缓存
rm -rf node_modules dist
npm install

# 检查 Node.js 版本
node -v  # 需要 >= 18.0.0

# 更新依赖
npm update

# 重新构建
npm run build
```

### 2. 测试失败

**问题**: npm test 失败

**解决方案**:
```bash
# 运行单个测试定位问题
npm test -- path/to/test.test.ts

# 清理测试缓存
rm -rf node_modules/.vitest

# 检查环境变量
cat .env.test

# 重新运行测试
npm test
```

### 3. 应用无法启动

**问题**: npm run dev 失败

**解决方案**:
```bash
# 检查端口占用
lsof -i :1420  # macOS/Linux
netstat -ano | findstr :1420  # Windows

# 杀死占用端口的进程
kill -9 <PID>  # macOS/Linux
taskkill /F /PID <PID>  # Windows

# 重新启动
npm run dev
```

### 4. 语言切换无效

**问题**: 切换语言后没有效果

**解决方案**:
```bash
# 清除浏览器缓存
# Chrome: Ctrl+Shift+Delete
# Firefox: Ctrl+Shift+Delete

# 清除 localStorage
localStorage.clear()

# 重新加载页面
location.reload()
```

### 5. 审批按钮显示不全

**问题**: 1024x720 分辨率下按钮显示不全

**解决方案**:
这个问题已在 v0.1.0-alpha 中修复。请更新到最新版本。

---

## 错误代码

### OPERATOR_NOT_FOUND

**含义**: 操作员不存在

**解决方案**:
```typescript
// 检查操作员 ID
const operators = await invoke('game_operator_list')
if (!operators.find(op => op.id === operatorId)) {
  console.error('Operator not found')
}
```

### TASK_NOT_FOUND

**含义**: 任务不存在

**解决方案**:
```typescript
// 检查任务 ID
const tasks = await invoke('task_list', { operatorId })
if (!tasks.find(task => task.id === taskId)) {
  console.error('Task not found')
}
```

### PATH_VIOLATION

**含义**: 路径访问违规

**解决方案**:
```rust
// 检查 PathGuard 配置
let guard = PathGuard::new()
    .allow_path("/projects")
    .block_path("/etc");

// 验证路径
if let Err(e) = guard.validate(&path) {
    eprintln!("Path violation: {}", e);
}
```

---

## 日志查看

### 桌面应用

**Windows**:
```
%APPDATA%\acp-ui\logs\app.log
```

**macOS**:
```
~/Library/Application Support/acp-ui/logs/app.log
```

**Linux**:
```
~/.config/acp-ui/logs/app.log
```

### Web 应用

打开浏览器开发者工具 (F12)，查看 Console 标签。

---

## 获取帮助

- **GitHub Issues**: https://github.com/yanritian/acp-ui/issues
- **Discussions**: https://github.com/yanritian/acp-ui/discussions
- **FAQ**: [FAQ.md](FAQ.md)

---

<div align="center">

**遇到问题？查看这里！**

[查看 FAQ →](FAQ.md)

</div>
