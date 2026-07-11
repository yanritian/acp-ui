# Hermes Game Operator - 开发者指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 简介

本文档为开发者提供 Hermes Game Operator 的开发、构建、测试和部署指南。

---

## 项目结构

```
acp-ui/
├── src/                          # Vue 前端
│   ├── api/                      # API 客户端
│   ├── components/               # Vue 组件
│   ├── features/                 # 功能模块
│   ├── locales/                  # 国际化文件
│   ├── stores/                   # Pinia stores
│   ├── tests/                    # 测试文件
│   └── types/                    # TypeScript 类型
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── operator/             # Operator 核心模块
│   │   ├── domains/              # 领域模块
│   │   └── commands/             # Tauri 命令
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── clients/                      # 客户端
│   ├── vscode-game-operator/     # VSCode 扩展
│   └── idea-game-operator/       # IDEA 插件
├── docs/                         # 文档
└── package.json                  # Node.js 依赖
```

---

## 开发环境设置

### 前置要求

- **Node.js**: 18.x 或更高版本
- **Rust**: 1.70 或更高版本
- **Tauri CLI**: 2.x
- **Godot**: 4.x（用于测试）

### 安装依赖

```bash
# 安装 Node.js 依赖
npm install

# 安装 Rust 依赖
cd src-tauri
cargo build
```

### 环境变量

创建 `.env` 文件：

```bash
# 服务器配置
SERVER_URL=http://localhost:8080
AUTH_TOKEN=your-token-here

# 开发配置
NODE_ENV=development
DEBUG=true

# D 盘约束（Windows）
TEMP=D:\tmp
TMP=D:\tmp
```

---

## 开发工作流

### 1. 启动开发服务器

```bash
# 启动 Tauri 开发服务器
npm run tauri dev

# 或单独启动前端
npm run dev
```

### 2. 运行测试

```bash
# 运行所有测试
npm run test

# 运行特定测试
npm run test -- src/tests/unit/operator-api.test.ts

# 运行测试并生成覆盖率报告
npm run test -- --coverage
```

### 3. 类型检查

```bash
# TypeScript 类型检查
npm run type-check

# 或
npx vue-tsc --noEmit
```

### 4. 代码格式化

```bash
# 格式化代码
npm run format

# 检查代码风格
npm run lint
```

---

## 添加新功能

### 1. 创建 API 端点

在 `src-tauri/src/operator/commands.rs` 中添加：

```rust
#[tauri::command]
pub async fn operator_new_command(
    state: State<'_, OperatorState>,
    param: String,
) -> Result<ResponseType, String> {
    // 实现逻辑
    Ok(response)
}
```

在 `src-tauri/src/lib.rs` 中注册：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    operator::operator_new_command,
])
```

### 2. 创建前端 API

在 `src/api/operatorApi.ts` 中添加：

```typescript
export const OperatorApi = {
  // ... 其他方法
  async newCommand(param: string): Promise<ResponseType> {
    return invoke<ResponseType>('operator_new_command', { param })
  }
}
```

### 3. 添加类型定义

在 `src/types/operator.ts` 中添加：

```typescript
export interface NewType {
  field1: string
  field2: number
}
```

### 4. 创建测试

在 `src/tests/unit/` 或 `src/tests/integration/` 中添加测试文件。

---

## 添加新的 Locale

### 1. 创建 Locale 文件

在 `src/locales/` 中创建新文件，如 `zh-TW.ts`：

```typescript
import type { MessageSchema } from './types'

export const zhTW: MessageSchema = {
  common: {
    appName: 'Hermes Game Operator',
    // ... 其他翻译
  },
  // ... 其他命名空间
}
```

### 2. 注册 Locale

在 `src/locales/index.ts` 中注册：

```typescript
import { zhTW } from './zh-TW'

const messages = {
  'zh-CN': zhCN,
  'en-US': enUS,
  'zh-TW': zhTW, // 添加新 locale
}
```

### 3. 更新类型

如果需要，更新 `src/locales/types.ts`。

---

## 安全开发指南

### 1. 处理认证

```typescript
// 使用 Bearer Token
const headers = {
  'Authorization': `Bearer ${authToken}`
}
```

### 2. 验证输入

```typescript
// 使用 Zod 验证
import { z } from 'zod'

const schema = z.object({
  email: z.string().email(),
  age: z.number().int().min(0)
})

const validated = schema.parse(input)
```

### 3. 防止 XSS

```typescript
// 使用 DOMPurify
import DOMPurify from 'dompurify'

const clean = DOMPurify.sanitize(dirty)
```

### 4. 使用安全头

```rust
// 在 HTTP 响应中添加安全头
.headers()
    .insert("Content-Security-Policy", csp_value)
    .insert("X-Frame-Options", "DENY")
    .insert("X-Content-Type-Options", "nosniff")
```

---

## 测试指南

### 单元测试

```typescript
import { describe, it, expect } from 'vitest'

describe('MyComponent', () => {
  it('should do something', () => {
    expect(result).toBe(expected)
  })
})
```

### 集成测试

```typescript
import { describe, it, expect, vi } from 'vitest'

describe('API Integration', () => {
  it('should handle API response', async () => {
    const mock = vi.fn()
    // 测试逻辑
  })
})
```

### 性能测试

```typescript
describe('Performance', () => {
  it('should complete in under 100ms', async () => {
    const start = Date.now()
    await operation()
    const duration = Date.now() - start
    expect(duration).toBeLessThan(100)
  })
})
```

---

## 部署指南

### 1. 构建生产版本

```bash
# 构建前端
npm run build

# 构建 Tauri 应用
npm run tauri build
```

### 2. 配置生产环境

更新 `tauri.conf.json`：

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  }
}
```

### 3. 签名应用

#### Windows

```bash
# 设置签名证书
set CSC_LINK=path/to/cert.pfx
set CSC_KEY_PASSWORD=your-password

# 构建签名应用
npm run tauri build
```

#### macOS

```bash
# 设置签名身份
export APPLE_ID=your-apple-id
export APPLE_ID_PASSWORD=your-password
export APPLE_TEAM_ID=your-team-id

# 构建签名应用
npm run tauri build
```

#### Linux

```bash
# 构建 deb 包
npm run tauri build -- --bundles deb
```

### 4. 发布

```bash
# 发布到 GitHub
npm run tauri build
gh release create v0.1.0 dist/*

# 或发布到 VSCode Marketplace
cd clients/vscode-game-operator
vsce publish
```

---

## 调试指南

### 1. 前端调试

```bash
# 启动开发服务器
npm run dev

# 在浏览器中打开开发者工具
# 使用 Vue DevTools 扩展
```

### 2. 后端调试

```bash
# 启用 Rust 调试日志
export RUST_LOG=debug

# 运行 Tauri 开发服务器
npm run tauri dev

# 查看日志
# Windows: %LOCALAPPDATA%\com.your-org.acp-ui\logs
# macOS: ~/Library/Logs/com.your-org.acp-ui
# Linux: ~/.local/share/com.your-org.acp-ui/logs
```

### 3. 测试调试

```bash
# 运行单个测试
npm run test -- --run src/tests/unit/my-test.test.ts

# 使用调试器
node --inspect-brk node_modules/.bin/vitest run
```

---

## 性能优化

### 1. 前端优化

```typescript
// 使用懒加载
const LazyComponent = defineAsyncComponent(() =>
  import('./components/LazyComponent.vue')
)

// 使用计算属性
const filteredItems = computed(() =>
  items.filter(item => item.active)
)

// 使用虚拟滚动
import { useVirtualList } from '@vueuse/core'
```

### 2. 后端优化

```rust
// 使用缓存
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// 使用异步
async fn process_data() -> Result<(), Error> {
    let data = fetch_data().await?;
    // 处理数据
}

// 使用连接池
use sqlx::PgPool;
```

### 3. 数据库优化

```sql
-- 添加索引
CREATE INDEX idx_task_status ON tasks(status);

-- 使用查询优化
EXPLAIN ANALYZE SELECT * FROM tasks WHERE status = 'running';

-- 使用连接池
```

---

## 监控和日志

### 1. 结构化日志

```rust
use tracing::{info, warn, error};

info!("Task started: {}", task_id);
warn!("Task paused: {}", task_id);
error!("Task failed: {} - {}", task_id, error);
```

### 2. 指标收集

```rust
use prometheus::{Counter, Registry};

lazy_static! {
    static ref TASK_COUNTER: Counter = Counter::new(
        "tasks_total",
        "Total number of tasks"
    ).unwrap();
}

TASK_COUNTER.inc();
```

### 3. 追踪

```rust
use tracing::instrument;

#[instrument]
async fn process_task(task_id: &str) -> Result<(), Error> {
    // 自动追踪
}
```

---

## 常见问题

### Q: 如何添加新的 API 端点？

A: 参考"添加新功能"章节。

### Q: 如何调试 Rust 代码？

A: 使用 `RUST_LOG=debug` 环境变量，配合 IDE 调试器。

### Q: 如何测试国际化？

A: 在设置中切换语言，或使用伪 locale（en-XA, ar-XB）。

### Q: 如何处理大量数据？

A: 使用分页、虚拟滚动、懒加载等技术。

### Q: 如何优化性能？

A: 参考"性能优化"章节。

---

## 贡献指南

### 1. Fork 仓库

```bash
git clone https://github.com/your-org/hermes-game-operator.git
cd hermes-game-operator
git checkout -b feature/my-feature
```

### 2. 开发功能

```bash
# 编写代码
# 添加测试
# 更新文档
```

### 3. 提交代码

```bash
git add .
git commit -m "feat: add my feature"
git push origin feature/my-feature
```

### 4. 创建 Pull Request

在 GitHub 上创建 PR，填写详细描述。

---

## 代码规范

### TypeScript

- 使用 TypeScript 严格模式
- 使用 ESLint 和 Prettier
- 遵循 Vue 3 Composition API

### Rust

- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 检查
- 遵循 Rust API Guidelines

### 测试

- 测试覆盖率 > 80%
- 使用描述性测试名称
- 测试边界情况

---

## 资源

### 文档

- [Vue 3 文档](https://vuejs.org/)
- [Tauri 文档](https://tauri.app/)
- [Rust 文档](https://doc.rust-lang.org/)
- [Pinia 文档](https://pinia.vuejs.org/)

### 社区

- GitHub: https://github.com/your-org/hermes-game-operator
- Discord: [加入社区](https://discord.gg/your-invite)

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Team