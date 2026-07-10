extends Node
class_name HealthComponent

## 最大生命值（可在编辑器中配置）
@export var max_health: float = 100.0

## 当前生命值（可在编辑器中查看，运行时由脚本管理）
@export var current_health: float = 0.0

## 生命值变化信号：提供新值与旧值，供 UI 或逻辑监听
signal changed(new_value: float, old_value: float)

## 生命值归零信号：可用于触发死亡动画或游戏结束逻辑
signal died

func _ready() -> void:
	# 初始化：确保节点进入场景树时生命值处于满状态
	current_health = max_health
	changed.emit(current_health, current_health)

## 受到伤害
## 用法：health.damage(15.0)
func damage(amount: float) -> void:
	if amount <= 0.0:
		return
	var old_value: float = current_health
	current_health = max(0.0, current_health - amount)
	if current_health != old_value:
		changed.emit(current_health, old_value)
		if current_health <= 0.0:
			died.emit()

## 恢复生命
## 用法：health.heal(10.0)
func heal(amount: float) -> void:
	if amount <= 0.0:
		return
	var old_value: float = current_health
	current_health = min(max_health, current_health + amount)
	if current_health != old_value:
		changed.emit(current_health, old_value)