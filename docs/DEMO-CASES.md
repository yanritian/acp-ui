# Demo Cases

This document provides practical demonstration cases for Hermes Game Operator.

## Demo 1: Add Double Jump

### Scenario
Add double jump ability to the player character in a Godot platformer game.

### Prerequisites
- Godot 4.x project with a player controller
- Player script at `scripts/Player.gd`
- Basic platformer mechanics implemented

### Steps

1. **Start Application**
   ```bash
   npm run tauri dev
   ```

2. **Navigate to Game Operator**
   - Click on Game Operator in sidebar
   - Verify you see the Game Operator interface

3. **Select Project**
   - Click "Browse"
   - Select your Godot project directory
   - Verify project is detected

4. **Enter Goal**
   ```
   Add double jump ability to the player character, 
   allowing up to 2 jumps in the air before landing.
   ```

5. **Start Task**
   - Click "Start Task"
   - Verify task status changes to "Planning"

6. **Review Plan**
   - Wait for plan generation
   - Review the execution plan:
     - Analyze Player.gd
     - Identify jump logic
     - Add jump_count variable
     - Modify jump condition
     - Add reset logic
   - Click "Approve"

7. **Monitor Progress**
   - Watch the progress timeline
   - Observe events:
     - Project analyzed
     - Plan generated
     - Plan approved
     - File modified
   - Approve file modifications when prompted

8. **Review Results**
   - Check task summary
   - Verify files modified
   - Open Player.gd
   - Review changes

### Expected Result

```gdscript
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0
var jump_count = 0
var max_jumps = 2

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity += get_gravity() * delta

	if Input.is_action_just_pressed("jump"):
		if jump_count < max_jumps:
			velocity.y = JUMP_VELOCITY
			jump_count += 1

	if is_on_floor():
		jump_count = 0

	var direction := Input.get_axis("ui_left", "ui_right")
	if direction:
		velocity.x = direction * SPEED
	else:
		velocity.x = move_toward(velocity.x, 0, SPEED)

	move_and_slide()
```

### Verification
- ✅ Player can jump once on ground
- ✅ Player can jump again in air
- ✅ Jump count resets on landing
- ✅ No more than 2 jumps allowed

---

## Demo 2: Create Menu Scene

### Scenario
Create a main menu scene with Start Game and Quit buttons.

### Prerequisites
- Godot 4.x project
- Basic project structure

### Steps

1. **Start Task**
   - Select project
   - Enter goal:
     ```
     Create a main menu scene with Start Game and Quit buttons.
     The menu should have a clean UI with a title, two buttons,
     and proper navigation.
     ```

2. **Approve Plan**
   - Review plan for creating:
     - scenes/Menu.tscn
     - scripts/Menu.gd
   - Approve plan

3. **Approve File Creation**
   - Approve creation of Menu.tscn
   - Approve creation of Menu.gd

4. **Review Results**
   - Check Menu.tscn structure
   - Review Menu.gd code

### Expected Result

**Menu.tscn**:
```
Menu (Control)
├── VBoxContainer
│   ├── Title (Label): "My Game"
│   ├── StartButton (Button): "Start Game"
│   └── QuitButton (Button): "Quit"
```

**Menu.gd**:
```gdscript
extends Control

func _on_start_button_pressed() -> void:
	get_tree().change_scene_to_file("res://scenes/Main.tscn")

func _on_quit_button_pressed() -> void:
	get_tree().quit()
```

### Verification
- ✅ Menu scene created
- ✅ Title displayed
- ✅ Start button works
- ✅ Quit button works

---

## Demo 3: Fix Enemy AI Bug

### Scenario
Fix a bug where enemies don't chase the player.

### Prerequisites
- Godot project with enemy AI
- Enemy script at `scripts/Enemy.gd`
- Known bug: enemies don't move toward player

### Steps

1. **Start Task**
   - Enter goal:
     ```
     Fix the enemy AI bug where enemies don't chase the player.
     The enemy should move toward the player when in detection range.
     ```

2. **Approve Plan**
   - Review analysis of Enemy.gd
   - Approve fix plan

3. **Approve Modification**
   - Review diff
   - Approve change

### Expected Result

**Before**:
```gdscript
func _physics_process(delta):
	velocity.x = SPEED
	move_and_slide()
```

**After**:
```gdscript
var player: Node2D

func _ready():
	player = get_tree().get_first_node_in_group("player")

func _physics_process(delta):
	if player and global_position.distance_to(player.global_position) < detection_range:
		var direction = (player.global_position - global_position).normalized()
		velocity = direction * SPEED
	else:
		velocity.x = SPEED
	move_and_slide()
```

### Verification
- ✅ Enemy detects player
- ✅ Enemy moves toward player
- ✅ Enemy returns to patrol when player is far

---

## Demo 4: Optimize Performance

### Scenario
Optimize a slow scene with too many nodes.

### Prerequisites
- Godot project with performance issues
- Scene with 1000+ nodes

### Steps

1. **Start Task**
   - Enter goal:
     ```
     Optimize the main scene performance by implementing
     object pooling for enemies and using instancing
     for repeated elements.
     ```

2. **Approve Plan**
   - Review optimization strategy
   - Approve plan

3. **Approve Modifications**
   - Approve multiple file changes

### Expected Result
- ✅ Object pooling implemented
- ✅ Instancing used for repeated elements
- ✅ Performance improved by 50%+

---

## Demo 5: Add New Feature

### Scenario
Add a sprint ability to the player.

### Prerequisites
- Godot project with player controller
- Player can move and jump

### Steps

1. **Start Task**
   - Enter goal:
     ```
     Add a sprint ability to the player. When holding shift,
     the player should move 50% faster. Add a stamina system
     that depletes while sprinting and regenerates when not sprinting.
     ```

2. **Approve Plan**
   - Review comprehensive plan
   - Approve plan

3. **Approve Modifications**
   - Approve changes to Player.gd
   - Approve UI additions

### Expected Result

**Player.gd additions**:
```gdscript
const SPRINT_MULTIPLIER = 1.5
const MAX_STAMINA = 100.0
const STAMINA_DRAIN = 20.0
const STAMINA_REGEN = 10.0

var stamina = MAX_STAMINA
var is_sprinting = false

func _physics_process(delta):
	# Sprint logic
	if Input.is_action_pressed("sprint") and stamina > 0:
		is_sprinting = true
		stamina -= STAMINA_DRAIN * delta
		speed = BASE_SPEED * SPRINT_MULTIPLIER
	else:
		is_sprinting = false
		stamina = min(MAX_STAMINA, stamina + STAMINA_REGEN * delta)
		speed = BASE_SPEED
	
	# ... rest of movement code
```

### Verification
- ✅ Sprint works when holding shift
- ✅ Stamina depletes while sprinting
- ✅ Stamina regenerates when not sprinting
- ✅ UI shows stamina bar

---

## Demo Tips

### For Presenters

1. **Prepare in Advance**
   - Have test projects ready
   - Test all demos beforehand
   - Have backup demos ready

2. **During Demo**
   - Explain each step clearly
   - Highlight key features
   - Show real-time progress
   - Answer questions

3. **Common Issues**
   - Slow analysis: Use smaller projects
   - API errors: Check internet connection
   - Approval timeout: Respond quickly

### For Viewers

1. **What to Look For**
   - Natural language understanding
   - Plan generation quality
   - Code generation accuracy
   - Safety features

2. **Key Features to Highlight**
   - Automatic project analysis
   - Intelligent planning
   - Safe execution with approvals
   - Complete audit trail
   - User control

### Demo Projects

**Recommended Test Projects**:
1. **Simple Platformer** (50 files)
   - Player controller
   - Basic enemies
   - Simple levels

2. **RPG Demo** (150 files)
   - Character system
   - Inventory
   - Dialogue system

3. **Puzzle Game** (100 files)
   - Level system
   - Puzzle mechanics
   - UI elements

### Success Criteria

**Demo is successful if**:
- ✅ Task completes without errors
- ✅ Generated code is correct
- ✅ User understands the process
- ✅ Safety features are demonstrated
- ✅ Questions are answered

---

## Demo Checklist

### Preparation
- [ ] Test projects prepared
- [ ] All demos tested
- [ ] Backup demos ready
- [ ] Internet connection stable
- [ ] API keys configured

### During Demo
- [ ] Introduction given
- [ ] Each step explained
- [ ] Key features highlighted
- [ ] Questions answered
- [ ] Next steps outlined

### After Demo
- [ ] Feedback collected
- [ ] Issues documented
- [ ] Improvements noted
- [ ] Resources shared

---

**Demo Version**: 1.0.0  
**Last Updated**: 2026-07-08
