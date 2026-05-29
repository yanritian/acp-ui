# Skill 元工具架构 - 当前实施计划

> **创建日期**: 2026-05-29
> **当前分支**: my-agent-teams-platform
> **状态**: 执行中

---

## 一、当前状态总结

### 已完成的改动 (未提交)

#### Rust 后端 (Phase 1 核心基础设施)
- ✅ `invoke_skill.rs` - 元工具 Handler 实现
- ✅ `skill_evolution.rs` - Skill 自进化机制
- ✅ `hermes-core/types.rs` - 新增 Skill 相关类型 (254+ 行)
- ✅ `hermes-skills` - SkillStore/Hub/Guard 增强
- ✅ `register_builtins.rs` - 注册 invoke_skill 元工具
- ✅ `skill_commands.rs` - Tauri Commands 桥接层

#### Vue 前端 (Phase 3 前端集成)
- ✅ `src/lib/skill-system/types.ts` - 完整类型定义
- ✅ `src/lib/skill-system/skill-invoker.ts` - Skill 调用服务
- ✅ `src/lib/skill-system/index.ts` - 模块导出
- ✅ `src/components/skills/SkillExplorer.vue` - Skill 浏览组件
- ✅ `src/components/skills/SkillEditor.vue` - Skill 编辑组件

#### 测试
- ✅ `tests/functional/acp-ui-functional.spec.ts`
- ✅ `tests/functional/comprehensive-functional.spec.ts`
- ✅ `tests/functional/user-input-interaction.spec.ts`

### 待解决的问题

| # | 问题 | 优先级 |
|---|------|--------|
| 1 | 验证 Rust 编译是否通过 | P0 |
| 2 | 验证 Vue 前端构建是否通过 | P0 |
| 3 | 修复可能的编译/类型错误 | P0 |
| 4 | 补充 Rust 单元测试 | P1 |
| 5 | 更新 .gitignore (排除 target/) | P1 |
| 6 | 提交所有改动到本地仓库 | P0 |

---

## 二、立即执行任务

### Task 1: 验证并修复 Rust 编译
- [ ] 运行 `cargo check` 检查 Rust 编译
- [ ] 修复编译错误
- [ ] 确保所有 Hermes crates 正常编译

### Task 2: 验证并修复 Vue 前端构建
- [ ] 运行 `npm run build` 检查前端构建
- [ ] 修复 TypeScript 类型错误
- [ ] 确保所有组件正常编译

### Task 3: 补充 .gitignore
- [ ] 排除 `**/target/` 目录
- [ ] 排除测试输出目录

### Task 4: 提交改动
- [ ] 暂存所有相关改动
- [ ] 创建详细的提交信息
- [ ] 提交到本地仓库

---

## 三、后续规划 (Phase 2-5)

### Phase 2: Skill 管理与自进化增强
- [ ] Skill 版本历史与回滚
- [ ] 自然语言创建 Skill (LLM 集成)
- [ ] 文件系统 SkillProvider 增强

### Phase 4: 12-16 核心 Skill 定义
- [ ] 设计核心 Skill 清单
- [ ] 创建 SKILL.md 文件
- [ ] 收敛现有工具到核心 Skill

### Phase 5: 测试与优化
- [ ] Rust 单元测试 (覆盖率 80%+)
- [ ] 集成测试
- [ ] 前端 E2E 测试
- [ ] 性能优化

---

## 四、技术决策

根据 `docs/skill-meta-tool-architecture-plan.md` 中的待确认问题，采用以下默认方案：

1. **Skill 存储**: 方案 A - 文件系统存储 (~/.hermes/skills/)
2. **调用方式**: 方案 C - 混合模式 (LLM 路由 + 直接执行)
3. **核心 Skill**: 16 个核心 Skill (见 types.ts 中的 CORE_SKILLS)
4. **自进化反馈**: 方案 C - 混合模式 (用户反馈 + 隐式反馈)

---

## 五、验收标准

### 本次提交验收
- [ ] `cargo check` 通过
- [ ] `npm run build` 通过
- [ ] 所有新文件已暂存
- [ ] 提交信息清晰描述改动

### Phase 1 完整验收
- [ ] invoke_skill 元工具可调用
- [ ] Skill 自进化逻辑可工作
- [ ] 前端可列出和浏览 Skills
- [ ] 至少 3 个 Rust 单元测试通过
