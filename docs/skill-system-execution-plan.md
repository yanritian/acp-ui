# Skill 元工具架构 — 执行计划与进度

> **创建日期**: 2026-05-29
> **分支**: my-agent-teams-platform
> **状态**: Phase 1 完成，待提交

---

## 一、项目背景

基于 OpenClacky 的 Skill 元工具模式，将 ACP-UI 的工具数量收敛到 12-16 个核心工具，所有扩展能力通过 `invoke_skill` 元工具委托，Skill 支持自进化。

### 核心架构

```
用户请求 → LLM 判断 → invoke_skill(skill_name, params)
                              ↓
                    SkillProvider 查找 SKILL.md
                              ↓
                    执行引擎 (LLM-guided / direct)
                              ↓
                    收集反馈 → 自进化判定 → 更新 SKILL.md
```

---

## 二、当前已完成的工作（本次会话验证）

### Rust 后端 (Phase 1)

| 文件 | 状态 | 说明 |
|------|------|------|
| `hermes-core/src/types.rs` | ✅ 新增 254+ 行 | Skill/SkillVersion/ExecutionStats/EvolutionTriggers 类型 |
| `hermes-tools/src/tools/invoke_skill.rs` | ✅ 新建 | 元工具 Handler，支持 Skill 调用和自进化触发 |
| `hermes-tools/src/tools/skill_evolution.rs` | ✅ 新建 | 自进化引擎：失败率/错误模式/评分/用户请求 四种触发 |
| `hermes-tools/src/tools/mod.rs` | ✅ 修改 | 导出 invoke_skill + skill_evolution |
| `hermes-tools/src/register_builtins.rs` | ✅ 修改 | 注册 invoke_skill 元工具 |
| `hermes-skills/src/` | ✅ 增强 | SkillStore/Hub/Guard 扩展 |
| `src/skill_commands.rs` | ✅ 新建 | Tauri Commands 桥接（skills_list/skill_invoke/skill_create） |
| `src/lib.rs` | ✅ 修改 | 注册 skill_commands |

**编译验证**：
- hermes-core ✅
- hermes-skills ✅
- hermes-tools ✅

### Vue 前端 (Phase 3)

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/lib/skill-system/types.ts` | ✅ 新建 | 完整类型定义 + CORE_SKILLS 16 个核心 Skill |
| `src/lib/skill-system/skill-invoker.ts` | ✅ 新建 | Skill 调用服务（Tauri invoke 桥接） |
| `src/lib/skill-system/index.ts` | ✅ 新建 | 模块导出 |
| `src/components/skills/SkillExplorer.vue` | ✅ 新建 | Skill 浏览/搜索/分类组件 |
| `src/components/skills/SkillEditor.vue` | ✅ 新建 | Skill 创建/编辑组件 |

**类型检查**：vue-tsc --noEmit ✅ 通过

### 功能测试

| 文件 | 说明 |
|------|------|
| `tests/functional/acp-ui-functional.spec.ts` | 核心功能测试 |
| `tests/functional/comprehensive-functional.spec.ts` | 综合功能测试 |
| `tests/functional/user-input-interaction.spec.ts` | 用户交互测试 |

---

## 三、后续执行计划

### Phase 2: Skill 管理增强（预计 2-3 天）

- [ ] 自然语言创建 Skill（LLM 生成 SKILL.md）
- [ ] Skill 版本历史与回滚（skill_versioning.rs）
- [ ] 文件系统 SkillProvider 版本管理
- [ ] 前端 SkillVersionHistory.vue 组件

### Phase 4: 核心 Skill 定义（预计 3-4 天）

- [ ] 创建 16 个核心 SKILL.md 文件
- [ ] 收敛现有 50+ 工具到核心 Skill
- [ ] 工具分组映射和兼容层

### Phase 5: 测试与优化（预计 2-3 天）

- [ ] Rust 单元测试（覆盖率 80%+）
- [ ] 集成测试（Skill 创建→执行→进化完整流程）
- [ ] Playwright E2E（Skill 管理 UI）
- [ ] 性能优化（Skill 加载缓存）

---

## 四、技术决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| Skill 存储 | 文件系统 (~/.hermes/skills/) | 与现有 skill_orchestrator 兼容 |
| 调用方式 | 混合模式（LLM 路由 + 直接执行） | 兼顾灵活性和性能 |
| 核心 Skill 数 | 16 个 | 覆盖所有核心能力 |
| 自进化反馈 | 混合模式（用户反馈 + 隐式反馈） | 最全面准确 |

---

## 五、验收标准

### Phase 1（本次提交）
- [x] invoke_skill 元工具编译通过
- [x] skill_evolution 自进化逻辑编译通过
- [x] 前端 skill-system 模块类型检查通过
- [x] SkillExplorer/SkillEditor 组件编译通过

### Phase 2-5（后续）
- [ ] 完整 Skill CRUD + 版本管理
- [ ] 16 个核心 Skill 定义并可用
- [ ] Rust 测试覆盖率 80%+
- [ ] E2E 测试关键路径通过
