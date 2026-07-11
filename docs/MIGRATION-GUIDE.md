# 迁移指南

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [从 v0.0.x 迁移到 v0.1.0](#从-v00x-迁移到-v010)
2. [API 变更](#api-变更)
3. [配置变更](#配置变更)
4. [数据结构变更](#数据结构变更)
5. [废弃功能](#废弃功能)
6. [迁移工具](#迁移工具)

---

## 从 v0.0.x 迁移到 v0.1.0

### 概述

v0.1.0 引入了多项重大改进：
- 完整的国际化支持（13种语言）
- 改进的审批系统（4级审批）
- 新增文件工具
- 增强的安全守卫
- 多客户端支持

### 迁移步骤

#### 1. 备份数据

```bash
# 备份数据库
pg_dump acp_ui > backup_before_migration.sql

# 备份配置文件
cp -r config/ config_backup/

# 备份用户数据
tar -czf user_data_backup.tar.gz ~/.config/acp-ui/
```

#### 2. 更新依赖

```bash
# 更新 Node.js 依赖
npm install

# 更新 Rust 依赖
cd src-tauri
cargo update
```

#### 3. 更新配置

```typescript
// 旧配置 (v0.0.x)
{
  "locale": "zh",
  "theme": "dark"
}

// 新配置 (v0.1.0)
{
  "locale": "zh-CN",
  "theme": "dark",
  "approval": {
    "defaultLevel": "approve",
    "autoApproveSafe": true
  },
  "security": {
    "pathGuard": {
      "enabled": true,
      "allowedPaths": ["/projects"]
    },
    "commandGuard": {
      "enabled": true,
      "allowedCommands": ["cargo build", "npm test"]
    }
  }
}
```

#### 4. 更新 API 调用

```typescript
// 旧 API (v0.0.x)
const result = await invoke('create_operator', {
  path: '/path/to/project'
});

// 新 API (v0.1.0)
const result = await invoke('game_operator_create', {
  project_path: '/path/to/project',
  agent_config: {
    model: 'gpt-4',
    temperature: 0.7
  }
});
```

---

## API 变更

### 命令名称变更

| 旧名称 (v0.0.x) | 新名称 (v0.1.0) | 说明 |
|----------------|----------------|------|
| `create_operator` | `game_operator_create` | 更清晰的命名 |
| `start_operator` | `game_operator_start` | 更清晰的命名 |
| `stop_operator` | `game_operator_stop` | 更清晰的命名 |
| `get_status` | `game_operator_status` | 更清晰的命名 |

### 参数变更

#### game_operator_create

```typescript
// 旧参数
{
  path: string;
}

// 新参数
{
  project_path: string;
  agent_config?: {
    model: string;
    temperature?: number;
    max_tokens?: number;
  };
}
```

#### approval_resolve

```typescript
// 旧参数
{
  id: string;
  approve: boolean;
}

// 新参数
{
  approval_id: string;
  decision: 'approve' | 'reject';
  reason?: string;
  resolved_by: string;
}
```

### 返回值变更

```typescript
// 旧返回值
{
  success: boolean;
  data: any;
}

// 新返回值
{
  operator_id: string;
  status: OperatorStatus;
  created_at: string;
  // 更多详细信息
}
```

---

## 配置变更

### 国际化配置

```typescript
// 旧配置
{
  "language": "zh"
}

// 新配置
{
  "locale": "zh-CN"  // 使用完整的语言代码
}
```

支持的语言代码：
- `zh-CN` (简体中文)
- `zh-TW` (繁体中文)
- `en-US` (英语)
- `pt-BR` (葡萄牙语-巴西)
- `de-DE` (德语)
- `es-ES` (西班牙语)
- `ru-RU` (俄语)
- `ja-JP` (日语)
- `ko-KR` (韩语)
- `vi-VN` (越南语)
- `th-TH` (泰语)
- `ms-MY` (马来语)
- `fr-FR` (法语)

### 审批配置

```typescript
// 新增配置
{
  "approval": {
    "defaultLevel": "approve",  // silent | notify | approve | forbidden
    "autoApproveSafe": true,     // 自动批准安全操作
    "requireReason": false,      // 拒绝时是否需要原因
    "batchOperations": true      // 启用批量操作
  }
}
```

### 安全配置

```typescript
// 新增配置
{
  "security": {
    "pathGuard": {
      "enabled": true,
      "allowedPaths": ["/projects", "/workspace"],
      "blockedPaths": ["/etc", "/usr"],
      "allowRelativePaths": false
    },
    "commandGuard": {
      "enabled": true,
      "allowedCommands": ["cargo build", "npm test", "git status"],
      "blockedCommands": ["rm -rf", "format"],
      "requireApproval": ["sudo", "apt-get"]
    }
  }
}
```

---

## 数据结构变更

### Task 结构

```typescript
// 旧结构
interface Task {
  id: string;
  title: string;
  status: string;
}

// 新结构
interface Task {
  task_id: string;
  operator_id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: 'low' | 'medium' | 'high';
  created_at: string;
  updated_at: string;
  metadata?: Record<string, any>;
}
```

### Approval 结构

```typescript
// 旧结构
interface Approval {
  id: string;
  task_id: string;
  approved: boolean;
}

// 新结构
interface Approval {
  approval_id: string;
  task_id: string;
  level: ApprovalLevel;
  action: string;
  title: string;
  reason?: string;
  status: 'pending' | 'approved' | 'rejected';
  created_at: string;
  resolved_at?: string;
  resolved_by?: string;
  decision?: 'approve' | 'reject';
}
```

---

## 废弃功能

### 已废弃

| 功能 | 替代方案 | 移除版本 |
|------|---------|---------|
| `create_operator` | `game_operator_create` | v0.2.0 |
| 单语言支持 | 13种语言支持 | v0.2.0 |
| 简单审批 | 4级审批系统 | v0.2.0 |
| 无安全守卫 | PathGuard + CommandGuard | v0.2.0 |

### 迁移建议

```typescript
// ❌ 废弃
invoke('create_operator', { path: '/project' })

// ✅ 推荐
invoke('game_operator_create', {
  project_path: '/project',
  agent_config: { model: 'gpt-4' }
})
```

---

## 迁移工具

### 自动迁移脚本

```bash
# 运行迁移工具
npm run migrate

# 或手动运行
node scripts/migrate.js --from 0.0.x --to 0.1.0
```

### 迁移检查清单

- [ ] 备份所有数据
- [ ] 更新依赖
- [ ] 更新配置文件
- [ ] 更新 API 调用
- [ ] 运行测试
- [ ] 验证功能
- [ ] 检查日志
- [ ] 验证性能

### 回滚方案

```bash
# 如果迁移失败，回滚到旧版本
git checkout v0.0.x

# 恢复备份
pg_restore -d acp_ui backup_before_migration.sql
cp -r config_backup/ config/

# 重新启动应用
npm run dev
```

---

## 常见问题

### Q: 迁移会丢失数据吗？

**A**: 不会。迁移工具会保留所有数据，只是更新数据结构。

### Q: 迁移需要多长时间？

**A**: 通常需要 5-10 分钟，取决于数据量。

### Q: 可以跳过多个版本迁移吗？

**A**: 可以，但建议逐版本迁移以确保稳定性。

### Q: 迁移失败怎么办？

**A**: 使用回滚方案恢复到旧版本，然后联系支持。

---

## 更多信息

- [API 文档](API-DOCUMENTATION.md)
- [部署指南](DEPLOYMENT-GUIDE.md)
- [维护指南](MAINTENANCE-GUIDE.md)
- [GitHub 仓库](https://github.com/yanritian/acp-ui)

---

<div align="center">

**平滑迁移，无缝升级！**

[查看 API 文档 →](API-DOCUMENTATION.md)

</div>
