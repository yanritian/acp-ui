# Godot Integration Examples

This document provides practical examples of using Hermes Game Operator with real Godot projects.

## Example 1: Add Double Jump

### Scenario
You have a basic platformer game and want to add double jump ability to the player.

### Initial Code (Player.gd)
```gdscript
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
```

### Task Description
```
Add double jump ability to the player. The player should be able to jump twice in the air before landing. Track the number of jumps and reset when landing.
```

### Expected Result
```gdscript
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0
var jump_count = 0
var max_jumps = 2

func _physics_process(delta: float) -> void:
    # Add gravity
    if not is_on_floor():
        velocity += get_gravity() * delta

    # Handle jump (including double jump)
    if Input.is_action_just_pressed("jump") and jump_count < max_jumps:
        velocity.y = JUMP_VELOCITY
        jump_count += 1

    # Reset jump count when landing
    if is_on_floor():
        jump_count = 0

    # Get input direction
    var direction := Input.get_axis("ui_left", "ui_right")
    if direction:
        velocity.x = direction * SPEED
    else:
        velocity.x = move_toward(velocity.x, 0, SPEED)

    move_and_slide()
```

### Verification
- [x] Player can jump once on ground
- [x] Player can jump again in air
- [x] Jump count resets on landing
- [x] Maximum 2 jumps allowed

---

## Example 2: Create Enemy AI

### Scenario
You want to add a basic enemy that patrols and chases the player.

### Task Description
```
Create an enemy that patrols between two points. When the player is within detection range, the enemy should chase the player. When the player leaves the range, return to patrol.
```

### Expected Result
```gdscript
extends CharacterBody2D

const PATROL_SPEED = 100.0
const CHASE_SPEED = 150.0
const DETECTION_RANGE = 300.0

@export var patrol_point_1: Vector2
@export var patrol_point_2: Vector2

var current_target: Vector2
var player: Node2D
var is_chasing = false

func _ready():
    current_target = patrol_point_1
    player = get_tree().get_first_node_in_group("player")

func _physics_process(delta: float) -> void:
    if player and global_position.distance_to(player.global_position) < DETECTION_RANGE:
        # Chase player
        is_chasing = true
        var direction = (player.global_position - global_position).normalized()
        velocity = direction * CHASE_SPEED
    else:
        # Patrol
        is_chasing = false
        var direction = (current_target - global_position).normalized()
        velocity = direction * PATROL_SPEED
        
        # Switch patrol points when reached
        if global_position.distance_to(current_target) < 10:
            current_target = patrol_point_2 if current_target == patrol_point_1 else patrol_point_1
    
    move_and_slide()
```

### Verification
- [x] Enemy patrols between two points
- [x] Enemy detects player within range
- [x] Enemy chases player when detected
- [x] Enemy returns to patrol when player leaves

---

## Example 3: Add Health System

### Scenario
You want to add a health system to the player with a UI indicator.

### Task Description
```
Add a health system to the player. Create a health bar UI that shows current health. The player should take damage from enemies and be able to collect health pickups.
```

### Expected Result

**Player.gd**:
```gdscript
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0

var health = 100
var max_health = 100

signal health_changed(new_health)

func _physics_process(delta: float) -> void:
    # ... movement code ...
    pass

func take_damage(amount: int):
    health = max(0, health - amount)
    emit_signal("health_changed", health)
    
    if health <= 0:
        die()

func heal(amount: int):
    health = min(max_health, health + amount)
    emit_signal("health_changed", health)

func die():
    # Handle death
    print("Player died!")
    # Reload scene or show game over
```

**HealthBar.tscn**:
```
HealthBar (Control)
├── Background (ColorRect)
│   color: Color(0.2, 0.2, 0.2, 1)
└── Fill (ColorRect)
    color: Color(0, 1, 0, 1)
```

**HealthBar.gd**:
```gdscript
extends Control

@onready var fill = $Fill
var player: Node2D

func _ready():
    player = get_tree().get_first_node_in_group("player")
    player.connect("health_changed", _on_health_changed)
    update_bar(player.health, player.max_health)

func _on_health_changed(new_health: int):
    update_bar(new_health, player.max_health)

func update_bar(current: int, maximum: int):
    var percentage = float(current) / float(maximum)
    fill.size.x = size.x * percentage
```

### Verification
- [x] Player has health (100/100)
- [x] Player can take damage
- [x] Player can heal
- [x] Health bar UI updates
- [x] Player dies at 0 health

---

## Example 4: Add Collectibles

### Scenario
You want to add coins that the player can collect.

### Task Description
```
Add collectible coins to the level. When the player touches a coin, it should be collected, play a sound, and increase the score.
```

### Expected Result

**Coin.gd**:
```gdscript
extends Area2D

signal collected

func _ready():
    connect("body_entered", _on_body_entered)

func _on_body_entered(body: Node2D):
    if body.is_in_group("player"):
        emit_signal("collected")
        queue_free()
```

**GameManager.gd**:
```gdscript
extends Node

var score = 0
signal score_changed(new_score)

func _ready():
    # Connect to all coins
    for coin in get_tree().get_nodes_in_group("coins"):
        coin.connect("collected", _on_coin_collected)

func _on_coin_collected():
    score += 10
    emit_signal("score_changed", score)
    print("Score: ", score)
```

### Verification
- [x] Coins appear in level
- [x] Player can collect coins
- [x] Coins disappear when collected
- [x] Score increases
- [x] Sound plays (optional)

---

## Example 5: Add Save/Load System

### Scenario
You want to save and load the player's progress.

### Task Description
```
Add a save/load system that saves the player's position, health, score, and level. Load the saved data when the game starts.
```

### Expected Result

**SaveSystem.gd**:
```gdscript
extends Node

const SAVE_PATH = "user://savegame.json"

func save_game():
    var player = get_tree().get_first_node_in_group("player")
    var game_manager = get_node("/root/GameManager")
    
    var save_data = {
        "player_position": {"x": player.global_position.x, "y": player.global_position.y},
        "player_health": player.health,
        "score": game_manager.score,
        "current_level": get_tree().current_scene.name
    }
    
    var file = FileAccess.open(SAVE_PATH, FileAccess.WRITE)
    file.store_string(JSON.stringify(save_data))
    file.close()
    
    print("Game saved!")

func load_game():
    if not FileAccess.file_exists(SAVE_PATH):
        print("No save file found")
        return
    
    var file = FileAccess.open(SAVE_PATH, FileAccess.READ)
    var save_data = JSON.parse_string(file.get_as_text())
    file.close()
    
    # Load player data
    var player = get_tree().get_first_node_in_group("player")
    player.global_position = Vector2(save_data.player_position.x, save_data.player_position.y)
    player.health = save_data.player_health
    
    # Load game manager data
    var game_manager = get_node("/root/GameManager")
    game_manager.score = save_data.score
    
    print("Game loaded!")
```

**MainMenu.gd**:
```gdscript
extends Control

func _on_save_button_pressed():
    get_node("/root/SaveSystem").save_game()

func _on_load_button_pressed():
    get_node("/root/SaveSystem").load_game()

func _on_new_game_button_pressed():
    get_tree().change_scene_to_file("res://scenes/Level1.tscn")
```

### Verification
- [x] Game can be saved
- [x] Save file created
- [x] Game can be loaded
- [x] Player position restored
- [x] Health restored
- [x] Score restored

---

## Example 6: Add Particle Effects

### Scenario
You want to add visual effects for jumping, collecting items, and taking damage.

### Task Description
```
Add particle effects for:
- Jumping (dust particles)
- Collecting coins (sparkle particles)
- Taking damage (blood particles)
```

### Expected Result

**JumpParticles.tscn**:
```
JumpParticles (GPUParticles2D)
├── texture: dust.png
├── amount: 10
├── lifetime: 0.5
├── one_shot: true
└── process_material:
    ├── direction: Vector3(0, -1, 0)
    ├── spread: 45
    ├── initial_velocity_min: 50
    └── initial_velocity_max: 100
```

**Player.gd** (modified):
```gdscript
@onready var jump_particles = preload("res://scenes/JumpParticles.tscn")

func _physics_process(delta: float) -> void:
    # ... movement code ...
    
    if Input.is_action_just_pressed("jump") and jump_count < max_jumps:
        velocity.y = JUMP_VELOCITY
        jump_count += 1
        
        # Spawn jump particles
        var particles = jump_particles.instantiate()
        particles.global_position = global_position
        get_parent().add_child(particles)
```

**Coin.tscn** (modified):
```
Coin (Area2D)
├── Sprite2D
└── SparkleParticles (GPUParticles2D)
    ├── texture: sparkle.png
    ├── amount: 20
    ├── lifetime: 0.3
    └── one_shot: true
```

### Verification
- [x] Jump particles appear when jumping
- [x] Sparkle particles appear when collecting coins
- [x] Particles disappear after lifetime
- [x] Visual feedback is clear

---

## Using Hermes Game Operator

### Step-by-Step Guide

1. **Launch Hermes Game Operator**
   ```bash
   npm run tauri dev
   ```

2. **Select Your Godot Project**
   - Click "Browse"
   - Navigate to your Godot project directory
   - Select the folder containing `project.godot`

3. **Enter Your Task Goal**
   ```
   Add double jump ability to the player. The player should be able to jump twice in the air before landing.
   ```

4. **Start the Task**
   - Click "Start Task"
   - Wait for analysis
   - Review the execution plan

5. **Approve the Plan**
   - Check the steps
   - Review file modifications
   - Click "Approve"

6. **Monitor Progress**
   - Watch the progress timeline
   - Approve file modifications when prompted
   - Review diffs before approving

7. **Test the Results**
   - Open Godot editor
   - Run the game
   - Verify the changes work as expected

### Tips for Best Results

✅ **Be Specific**: Use clear, detailed descriptions  
✅ **One Task at a Time**: Focus on one feature per task  
✅ **Test Incrementally**: Test after each task  
✅ **Review Diffs**: Always review code changes  
✅ **Use Version Control**: Commit before starting tasks  

### Common Task Templates

**Movement**:
```
Add [movement type] to player. Include [specific requirements].
```

**Combat**:
```
Add [weapon/attack] that [behavior]. Include [damage/effects].
```

**UI**:
```
Create [UI element] that shows [information]. Update when [event].
```

**Audio**:
```
Add [sound effect] for [action]. Play when [trigger].
```

**Save System**:
```
Add save/load system that saves [data]. Load when [trigger].
```

---

## Troubleshooting

### Issue: Generated code doesn't work
**Solution**: Review the diff, test in Godot, report issues

### Issue: Task takes too long
**Solution**: Break into smaller tasks, be more specific

### Issue: Wrong files modified
**Solution**: Review plan carefully, reject if incorrect

### Issue: Performance issues
**Solution**: Profile in Godot, optimize generated code

---

**Example Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
