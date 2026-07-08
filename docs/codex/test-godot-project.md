# Godot 测试项目 - 最小可运行示例

这个项目用于测试 Hermes Game Operator 的端到端工作流。

## 项目结构

```
test-godot-project/
├── project.godot           # Godot 项目配置文件
├── scenes/
│   └── Main.tscn          # 主场景
├── scripts/
│   ├── Player.gd          # 玩家控制器脚本
│   └── Enemy.gd           # 敌人脚本
└── assets/
    └── player.png         # 玩家精灵图（占位）
```

## 测试任务

### 任务 1: 给 Player 添加二段跳

**目标**: 修改 `scripts/Player.gd`，为玩家角色添加二段跳能力

**预期操作**:
1. 分析项目结构
2. 找到 Player.gd
3. 识别跳跃逻辑
4. 生成修改计划
5. 等待用户审批
6. 应用修改
7. 输出总结

**验收标准**:
- [ ] 正确识别 `project.godot`
- [ ] 找到 `scripts/Player.gd`
- [ ] 生成合理的修改计划
- [ ] 等待用户审批
- [ ] 正确修改文件
- [ ] 输出任务总结

### 任务 2: 修改敌人移动速度

**目标**: 修改 `scripts/Enemy.gd`，将敌人移动速度从 100 改为 150

**预期操作**:
1. 分析项目结构
2. 找到 Enemy.gd
3. 识别速度变量
4. 生成修改计划
5. 等待用户审批
6. 应用修改

**验收标准**:
- [ ] 正确识别速度变量
- [ ] 生成修改计划
- [ ] 正确修改速度值

## 如何创建测试项目

### 方法 1: 使用 Godot 编辑器

1. 打开 Godot 4.x
2. 创建新项目
3. 选择此目录
4. 添加 Player 和 Enemy 场景
5. 编写脚本

### 方法 2: 手动创建（推荐用于自动化测试）

```bash
# 创建目录结构
mkdir -p test-godot-project/{scenes,scripts,assets}

# 创建 project.godot
cat > test-godot-project/project.godot << 'EOF'
; Engine configuration file.
; It's best edited using the editor UI and not directly,
; but it can also be manually edited if needed.

config_version=5

[application]

config/name="Test Godot Project"
run/main_scene="res://scenes/Main.tscn"
config/features=PackedStringArray("4.2")

[display]

window/size/viewport_width=1152
window/size/viewport_height=648

[input]

jump={
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_path":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":32,"key_label":0,"unicode":32,"location":0,"echo":false,"script":null)
]
}
EOF

# 创建 Main.tscn
cat > test-godot-project/scenes/Main.tscn << 'EOF'
[gd_scene load_steps=3 format=3 uid="uid://main"]

[ext_resource type="Script" path="res://scripts/Player.gd" id="1"]
[ext_resource type="Script" path="res://scripts/Enemy.gd" id="2"]

[node name="Main" type="Node2D"]

[node name="Player" type="CharacterBody2D" parent="."]
script = ExtResource("1")

[node name="Enemy" type="CharacterBody2D" parent="."]
script = ExtResource("2")
EOF

# 创建 Player.gd
cat > test-godot-project/scripts/Player.gd << 'EOF'
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0

func _physics_process(delta: float) -> void:
	# Add gravity
	if not is_on_floor():
		velocity += get_gravity() * delta

	# Handle jump
	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = JUMP_VELOCITY

	# Get input direction
	var direction := Input.get_axis("ui_left", "ui_right")
	if direction:
		velocity.x = direction * SPEED
	else:
		velocity.x = move_toward(velocity.x, 0, SPEED)

	move_and_slide()
EOF

# 创建 Enemy.gd
cat > test-godot-project/scripts/Enemy.gd << 'EOF'
extends CharacterBody2D

const SPEED = 100.0

func _physics_process(delta: float) -> void:
	# Simple patrol movement
	velocity.x = SPEED
	move_and_slide()
EOF

# 创建占位图片（可选）
echo "Placeholder for player.png" > test-godot-project/assets/player.png
```

## 测试执行

```bash
# 启动 Hermes Game Operator
npm run tauri dev

# 在 UI 中：
1. 选择项目路径: /path/to/test-godot-project
2. 输入目标: "给 Player 添加二段跳能力"
3. 点击 "Start Task"
4. 观察执行过程
5. 审批修改
6. 查看结果
```

## 预期输出

执行完成后，应该看到：

1. **事件流**: 完整的任务执行事件
2. **文件修改**: `scripts/Player.gd` 被修改
3. **差异展示**: 显示修改前后的对比
4. **任务总结**: 成功/失败步骤统计

## 常见问题

### Q: 如何验证项目识别？
A: 检查 `ProjectAnalysisResult` 是否包含正确的 scripts 和 scenes 列表

### Q: 如何验证修改正确？
A: 检查 `Player.gd` 是否包含二段跳逻辑（jump_count 变量）

### Q: 如何测试审批流程？
A: 在 Plan 生成后，不要立即审批，观察 WaitingApproval 状态

## 扩展测试

可以创建更多测试项目来验证：

1. **大型项目**: 100+ 脚本，测试性能
2. **复杂场景**: 嵌套节点，测试解析能力
3. **多玩家**: 多个玩家控制器，测试识别准确性
4. **资源密集型**: 大量资源文件，测试扫描速度
