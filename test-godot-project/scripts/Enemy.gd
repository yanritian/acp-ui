extends CharacterBody2D

const SPEED = 100.0

func _physics_process(delta: float) -> void:
	# Simple patrol movement
	velocity.x = SPEED
	move_and_slide()
