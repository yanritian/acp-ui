# Godot Code Generation Skill

> 为 Godot 项目生成 GDScript 代码

## When to Use

当用户要求为 Godot 项目添加功能、修改玩法、修复 bug 时触发。

## Inputs

- `task_goal`: 用户需求描述
- `project_info`: GodotProjectInfo 结构
- `target_file`: 目标脚本路径（可选）

## Outputs

返回代码修改提案：

```json
{
  "file_changes": [
    {
      "path": "res://scripts/Player.gd",
      "action": "modify",
      "diff": "--- original\n+++ modified\n...",
      "approval_level": "approve"
    }
  ],
  "validation_steps": [
    "在 Godot 编辑器中打开项目",
    "运行游戏测试新功能"
  ]
}
```

## Available Tools

- `file.read`: 读取脚本文件
- `file.patch`: 生成文件修改 diff
- `godot.parse_scene`: 解析场景结构
- `memory.read`: 读取项目记忆
- `memory.write`: 写入项目记忆

## Workflow

1. **理解需求** - 分析用户目标
2. **定位文件** - 找到相关脚本/场景
3. **读取上下文** - 加载现有代码
4. **生成方案** - 制定修改计划
5. **生成 diff** - 创建代码变更
6. **请求审批** - 提交给用户审批
7. **应用变更** - 审批通过后写入文件
8. **验证结果** - 运行检查或提供验证步骤

## Guardrails

- **必须** 在修改前展示 diff
- **必须** 等待用户审批才能写入文件
- **禁止** 删除文件（需要 Level 3 审批）
- **禁止** 修改项目外的文件
- **禁止** 执行 shell 命令（除非显式允许）
- 优先使用 Godot 4.x 语法

## Memory Schema

写入项目记忆：

```json
{
  "scope": "project",
  "domain": "godot",
  "key": "player_controller_path",
  "value": "res://scripts/Player.gd",
  "confidence": 0.95
}
```

## Acceptance Criteria

- [ ] 生成的代码符合 GDScript 语法
- [ ] 生成的代码符合 Godot 4.x 最佳实践
- [ ] 修改前展示 diff
- [ ] 等待用户审批
- [ ] 审批后才写入文件
- [ ] 提供验证步骤

## Examples

### 输入
```
task_goal: "给 Player 添加二段跳能力"
project_info: { ... }
```

### 输出
```json
{
  "file_changes": [
    {
      "path": "res://scripts/Player.gd",
      "action": "modify",
      "diff": "@@ -10,6 +10,12 @@\n var jump_count = 0\n+var max_jumps = 2\n+\n func _physics_process(delta):\n   if Input.is_action_just_pressed(\"jump\"):\n-    if is_on_floor():\n+    if jump_count < max_jumps:\n       velocity.y = jump_velocity\n+      jump_count += 1",
      "approval_level": "approve"
    }
  ]
}
```

## Common Patterns

### 添加跳跃能力

```gdscript
# 变量声明
var jump_velocity = -400.0
var jump_count = 0
var max_jumps = 2

# 在 _physics_process 中
if Input.is_action_just_pressed("jump"):
    if jump_count < max_jumps:
        velocity.y = jump_velocity
        jump_count += 1

# 落地时重置
if is_on_floor():
    jump_count = 0
```

### 添加冲刺能力

```gdscript
var dash_speed = 500.0
var dash_duration = 0.2
var dash_cooldown = 1.0
var can_dash = true

func dash():
    if can_dash:
        velocity.x = dash_speed * sign(input_direction)
        can_dash = false
        await get_tree().create_timer(dash_cooldown).timeout
        can_dash = true
```
