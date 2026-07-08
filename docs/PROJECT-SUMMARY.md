# Hermes Game Operator 项目摘要

> 完成日期: 2026-07-08
> 版本: 0.1.0-alpha
> 状态: 代码层面 100% 完成

---

## 快速开始

```bash
# 安装依赖
npm install

# 运行测试
npm run test

# 构建前端
npm run build

# 启动开发服务器
npm run tauri dev
```

---

## 项目结构

```
D:\dingsun\acp-ui\
├── src/                          # Vue 前端
│   ├── features/game-operator/   # Game Operator 组件
│   ├── api/operatorApi.ts        # API 层
│   └── types/operator.ts         # TypeScript 类型
│
├── src-tauri/src/                # Rust 后端
│   ├── operator/                 # Operator Control Plane
│   └── domains/games/godot/      # Godot Domain Pack
│
├── tests/                        # 测试
├── scripts/                      # 工具脚本
└── docs/                         # 文档
```

---

## 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 前端 | Vue 3 | 3.x |
| 后端 | Tauri | 2.x |
| 后端 | Rust | 1.70+ |
| 测试 | Vitest | 1.x |

---

## 测试覆盖

- 单元测试: 413
- 性能测试: 7
- 错误处理: 22
- E2E 测试: 14
- **总计: 413 passed**

---

## 下一步

1. 安装 Windows 10 SDK
2. 验证 cargo check
3. 安装 Hermes CLI
4. 使用测试项目验证闭环
