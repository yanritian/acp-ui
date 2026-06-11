# Contributing to ACP-Swarm

感谢你对 ACP-Swarm 的关注！本文档提供了参与项目贡献的指南。

## 开发流程

### 1. 分支策略

- `main` - 主分支，稳定版本
- `feature/*` - 新功能分支
- `fix/*` - 修复分支
- `refactor/*` - 重构分支

### 2. 提交规范

使用 Conventional Commits 格式：

```
<type>: <description>

[optional body]
```

类型说明：
- `feat` - 新功能
- `fix` - 修复问题
- `refactor` - 重构
- `docs` - 文档更新
- `test` - 测试相关
- `chore` - 杂项

### 3. Pull Request 流程

1. Fork 项目或创建分支
2. 实现功能/修复
3. 运行测试确保通过
4. 提交 PR，包含：
   - 变更说明
   - 相关 Issue 链接
   - 测试计划

## 代码规范

### Rust

- 运行 `cargo clippy` 确保无警告
- 运行 `cargo fmt` 格式化代码
- 遵循 RFC 文档中的设计原则

### TypeScript/Vue

- 运行 `npx vue-tsc --noEmit` 确无类型错误
- 遵循项目的 coding-style.md 规范

## 测试要求

- 新功能必须包含单元测试
- 修复问题必须包含回归测试
- 测试覆盖率目标：80%+

## RFC 流程

重大架构变更需要通过 RFC 流程：

1. 创建 `rfcs/RFC-XXX-title.md`
2. 描述问题、方案、权衡
3. 社区讨论
4. 批准后实施

## 问题反馈

- 使用 GitHub Issues
- 提供详细描述和复现步骤
- 标注优先级（P0-P3）

## 联系方式

- GitHub Discussions
- Issue Tracker