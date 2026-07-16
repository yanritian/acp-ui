extends SceneTree

func _init():
    print("GODOT_HEADLESS_VALIDATION_START")
    print("Engine version: ", Engine.get_version_info().string)
    print("Platform: ", OS.get_name())
    print("Headless mode: ", OS.has_feature("headless"))

    # Test project loading
    var project_path = ProjectSettings.globalize_path("res://")
    print("Project path: ", project_path)

    # Test script loading
    var player_script = load("res://scripts/Player.gd")
    if player_script:
        print("✅ Player.gd loaded successfully")
    else:
        print("❌ Failed to load Player.gd")
        quit(1)

    var enemy_script = load("res://scripts/Enemy.gd")
    if enemy_script:
        print("✅ Enemy.gd loaded successfully")
    else:
        print("❌ Failed to load Enemy.gd")
        quit(1)

    # Test scene loading
    var main_scene = load("res://scenes/Main.tscn")
    if main_scene:
        print("✅ Main.tscn loaded successfully")
    else:
        print("⚠️ Main.tscn not found (optional)")

    # Test node instantiation
    var player = player_script.new()
    if player:
        print("✅ Player node instantiated")
        print("Player health: ", player.health if "health" in player else "N/A")
    else:
        print("❌ Failed to instantiate Player")
        quit(1)

    print("GODOT_HEADLESS_VALIDATION_SUCCESS")
    quit(0)
