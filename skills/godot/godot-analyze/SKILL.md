# Godot Project Analysis Skill

> 分析 Godot 项目结构，提取关键信息

## When to Use

当用户选择一个 Godot 项目目录时，自动触发此 Skill 进行项目分析。

## Inputs

- `project_path`: Godot 项目根目录路径

## Outputs

返回 `GodotProjectInfo` 结构：

```json
{
  "project_path": "/path/to/project",
  "project_file": "/path/to/project/project.godot",
  "project_name": "My Game",
  "godot_version": "4.2",
  "main_scene": "res://scenes/Main.tscn",
  "scripts": ["res://scripts/Player.gd", ...],
  "scenes": ["res://scenes/Main.tscn", ...],
  "assets": ["res://assets/player.png", ...],
  "entry_scene": "res://scenes/Main.tscn"
}
```

## Available Tools

- `godot_detect_project`: 检测是否为 Godot 项目
- `godot_analyze_project`: 分析项目结构
- `godot_parse_scene`: 解析 .tscn 文件
- `godot_find_player_controllers`: 查找玩家控制器脚本

## Workflow

1. **检测项目** - 验证 `project.godot` 存在
2. **读取配置** - 解析 `project.godot` 获取项目名、版本、主场景
3. **扫描脚本** - 递归查找 `.gd` 文件
4. **扫描场景** - 递归查找 `.tscn` 文件
5. **扫描资源** - 查找图片、音频等资源文件
6. **识别入口** - 确定主场景和玩家控制器
7. **生成报告** - 返回完整项目信息

## Guardrails

- **禁止** 修改项目文件
- **禁止** 访问项目目录外的文件
- **禁止** 执行任何命令
- 只读操作，纯分析

## Acceptance Criteria

- [ ] 能正确识别 `project.godot`
- [ ] 能提取项目名和 Godot 版本
- [ ] 能列出所有 `.gd` 脚本文件
- [ ] 能列出所有 `.tscn` 场景文件
- [ ] 能识别主场景
- [ ] 能找到玩家控制器候选脚本

## Examples

### 输入
```
project_path: "/home/user/my-godot-game"
```

### 输出
```json
{
  "project_name": "My Godot Game",
  "godot_version": "4.2",
  "main_scene": "res://scenes/Main.tscn",
  "scripts": [
    "res://scripts/Player.gd",
    "res://scripts/Enemy.gd"
  ],
  "scenes": [
    "res://scenes/Main.tscn",
    "res://scenes/Game.tscn"
  ]
}
```
