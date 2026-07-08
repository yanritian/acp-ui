# Hermes Game Operator 故障排除指南

## 常见问题

### 1. cargo check 失败

**解决方案:**
- 安装 Visual Studio Build Tools
- 勾选 "Desktop development with C++"
- 安装 Windows 10 SDK

### 2. Hermes CLI 未找到

**解决方案:**
```bash
hermes --version  # 检查安装
export HERMES_CLI_PATH=/path/to/hermes  # 设置路径
```

### 3. API Key 无效

**解决方案:**
```bash
export ANTHROPIC_API_KEY=sk-xxx
```

### 4. 路径越权错误

**解决方案:**
- 确保路径在项目目录内
- 不使用 `..` 父目录引用

### 5. 状态转换失败

**解决方案:**
- 检查任务当前状态
- 确认转换是否允许

## 诊断命令

```bash
./scripts/check-env.sh
npm run test
npm run typecheck
```

## 重置操作

```bash
./scripts/clean.sh
npm install
```
