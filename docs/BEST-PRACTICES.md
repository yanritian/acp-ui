# 最佳实践指南

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [代码规范](#代码规范)
2. [测试实践](#测试实践)
3. [安全实践](#安全实践)
4. [性能优化](#性能优化)
5. [文档规范](#文档规范)
6. [Git 工作流](#git-工作流)

---

## 代码规范

### TypeScript/Vue

#### ✅ 推荐

```typescript
// 使用 Composition API
<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value++
}
</script>

// 使用 TypeScript 类型
interface User {
  id: string
  name: string
  email: string
}

const user: User = {
  id: '1',
  name: 'John',
  email: 'john@example.com'
}

// 使用不可变数据
const newUser = { ...user, name: 'Jane' }
```

#### ❌ 避免

```typescript
// 不要使用 Options API (除非必要)
export default {
  data() {
    return { count: 0 }
  }
}

// 不要使用 any
const user: any = getUser()

// 不要直接修改 props
props.count++ // ❌
```

### Rust

#### ✅ 推荐

```rust
// 使用 Result 处理错误
fn read_file(path: &Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// 使用枚举表示状态
enum State {
    Idle,
    Running { progress: f32 },
    Completed,
}

// 使用不可变绑定
let config = Config::load()?;
let new_config = Config { ...config, debug: true };
```

#### ❌ 避免

```rust
// 不要使用 unwrap()
let value = map.get("key").unwrap(); // ❌

// 不要使用 panic!
panic!("Something went wrong"); // ❌

// 不要使用可变全局状态
static mut COUNTER: i32 = 0; // ❌
```

---

## 测试实践

### 单元测试

#### ✅ 推荐

```typescript
describe('GameOperator', () => {
  it('should create operator with valid config', () => {
    const config = { projectPath: '/path/to/project' }
    const operator = createOperator(config)
    expect(operator).toBeDefined()
    expect(operator.status).toBe('idle')
  })

  it('should throw error for invalid path', () => {
    expect(() => createOperator({ projectPath: '' }))
      .toThrow('Invalid project path')
  })
})
```

#### 测试覆盖率目标

- 语句覆盖率: >= 80%
- 分支覆盖率: >= 75%
- 函数覆盖率: >= 85%
- 行覆盖率: >= 80%

### 集成测试

```typescript
describe('Approval System', () => {
  it('should handle approval workflow', async () => {
    // 创建操作员
    const operator = await createOperator(config)
    
    // 启动操作员
    await startOperator(operator.id)
    
    // 等待审批请求
    const approval = await waitForApproval(operator.id)
    
    // 批准请求
    await resolveApproval(approval.id, 'approve')
    
    // 验证结果
    const status = await getOperatorStatus(operator.id)
    expect(status.state).toBe('running')
  })
})
```

### E2E 测试

```typescript
test('complete user journey', async ({ page }) => {
  // 打开应用
  await page.goto('/')
  
  // 创建操作员
  await page.click('[data-testid="new-operator"]')
  await page.fill('[data-testid="project-path"]', '/path/to/project')
  await page.click('[data-testid="create-button"]')
  
  // 启动操作员
  await page.click('[data-testid="start-button"]')
  
  // 处理审批
  await page.click('[data-testid="approval-queue"]')
  await page.click('[data-testid="approve-button"]')
  
  // 验证结果
  await expect(page.locator('[data-testid="status"]'))
    .toHaveText('Running')
})
```

---

## 安全实践

### 输入验证

#### ✅ 推荐

```typescript
import { z } from 'zod'

const configSchema = z.object({
  projectPath: z.string().min(1).max(500),
  agentConfig: z.object({
    model: z.string(),
    temperature: z.number().min(0).max(2),
  }).optional(),
})

function createOperator(input: unknown) {
  const config = configSchema.parse(input)
  // 使用验证后的配置
}
```

#### ❌ 避免

```typescript
// 不要信任用户输入
function createOperator(config: any) {
  const path = config.projectPath // ❌ 未验证
  // 直接使用...
}
```

### 路径安全

```rust
// 使用 PathGuard
let guard = PathGuard::new()
    .allow_path("/projects")
    .block_path("/etc");

// 验证路径
if let Err(e) = guard.validate(&user_path) {
    return Err(Error::PathViolation(e));
}
```

### 命令安全

```rust
// 使用 CommandGuard
let guard = CommandGuard::new()
    .allow_command("cargo build")
    .allow_command("npm test")
    .block_command("rm -rf");

// 验证命令
if let Err(e) = guard.validate(&command) {
    return Err(Error::CommandBlocked(e));
}
```

---

## 性能优化

### 前端优化

#### ✅ 推荐

```vue
<!-- 使用虚拟滚动 -->
<template>
  <VirtualList :items="largeList" :item-height="50">
    <template #item="{ item }">
      <div>{{ item.name }}</div>
    </template>
  </VirtualList>
</template>

<!-- 使用计算属性缓存 -->
<script setup>
const filteredItems = computed(() => {
  return items.value.filter(item => item.active)
})
</script>

<!-- 使用防抖 -->
<script setup>
import { useDebounce } from '@vueuse/core'

const searchQuery = ref('')
const debouncedQuery = useDebounce(searchQuery, 300)

watch(debouncedQuery, (query) => {
  // 执行搜索
})
</script>
```

### 后端优化

```rust
// 使用异步
async fn process_tasks(tasks: Vec<Task>) -> Vec<Result> {
    let futures = tasks.into_iter().map(|task| {
        tokio::spawn(async move {
            process_task(task).await
        })
    });
    
    join_all(futures).await
}

// 使用连接池
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;
```

---

## 文档规范

### 代码注释

#### ✅ 推荐

```typescript
/**
 * 创建新的游戏操作员实例
 * 
 * @param config - 操作员配置
 * @returns 操作员实例
 * @throws 如果配置无效则抛出错误
 * 
 * @example
 * ```typescript
 * const operator = createOperator({
 *   projectPath: '/path/to/project'
 * })
 * ```
 */
function createOperator(config: OperatorConfig): Operator {
  // 实现...
}
```

#### ❌ 避免

```typescript
// 不要写无意义的注释
const count = 0 // 设置 count 为 0 ❌

// 不要注释掉代码
// const oldLogic = calculate() // ❌
```

### 文档结构

```markdown
# 文档标题

> 简短描述

---

## 目录

1. [章节1](#章节1)
2. [章节2](#章节2)

---

## 章节1

内容...

## 章节2

内容...

---

## 相关链接

- [链接1](url)
- [链接2](url)
```

---

## Git 工作流

### 提交消息

#### ✅ 推荐

```bash
feat: add approval queue system

Implement 4-level approval system:
- Silent: auto-approve
- Notify: notify without blocking
- Approve: require manual approval
- Forbidden: block action

Includes:
- Approval queue component
- Backend approval service
- Integration tests
- Documentation

Closes #123
```

#### ❌ 避免

```bash
# 不要使用模糊的消息
fix: fix bug ❌
update: update code ❌
wip ❌
```

### 分支命名

```bash
# 功能分支
feature/approval-system
feature/user-authentication

# 修复分支
fix/approval-button-alignment
fix/memory-leak

# 文档分支
docs/api-documentation
docs/quick-start-guide

# 重构分支
refactor/state-machine
refactor/approval-system
```

### PR 流程

1. **创建分支**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **开发并提交**
   ```bash
   git add .
   git commit -m "feat: add my feature"
   git push origin feature/my-feature
   ```

3. **创建 PR**
   - 标题清晰
   - 描述详细
   - 关联 Issue
   - 添加审查者

4. **代码审查**
   - 至少 1 个审查者批准
   - 所有检查通过
   - 解决所有评论

5. **合并**
   - 使用 Squash 合并
   - 删除分支

---

## 审查清单

### 代码审查

- [ ] 代码符合规范
- [ ] 有适当的注释
- [ ] 有完整的测试
- [ ] 没有安全隐患
- [ ] 性能可接受
- [ ] 文档已更新

### PR 审查

- [ ] 标题清晰描述变更
- [ ] 描述详细说明变更
- [ ] 关联了相关 Issue
- [ ] 所有检查通过
- [ ] 至少 1 个审查者批准
- [ ] 没有未解决的评论

---

## 工具推荐

### 开发工具

- **IDE**: VSCode, IntelliJ IDEA
- **版本控制**: Git, GitHub Desktop
- **API 测试**: Postman, Insomnia
- **性能分析**: Chrome DevTools, Rust Profiler

### 代码质量

- **Linting**: ESLint, Clippy
- **格式化**: Prettier, rustfmt
- **类型检查**: TypeScript, rust-analyzer
- **测试**: Vitest, cargo test

### 文档工具

- **文档生成**: TypeDoc, rustdoc
- **图表**: Mermaid, Draw.io
- **截图**: CleanShot, Snagit

---

## 更多信息

- [贡献指南](../CONTRIBUTING.md)
- [代码规范](../CONTRIBUTING.md#code-style)
- [GitHub 仓库](https://github.com/yanritian/acp-ui)

---

<div align="center">

**遵循最佳实践，写出更好的代码！**

[开始贡献 →](../CONTRIBUTING.md)

</div>
