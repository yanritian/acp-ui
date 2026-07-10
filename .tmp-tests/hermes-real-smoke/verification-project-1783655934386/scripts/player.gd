extends Node2D

@export var speed: float = 180.0

func _process(delta: float) -> void:
    position.x += speed * delta
