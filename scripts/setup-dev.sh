#!/bin/bash
# Setup Development Environment for Hermes Game Operator

set -e

echo "Setting up Hermes Game Operator development environment..."

# Install npm dependencies
echo "Installing npm dependencies..."
npm install

# Create .env file if not exists
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env
    echo "Please edit .env and add your API keys"
fi

# Create test Godot project
TEST_PROJECT="D:/tmp/test-godot-project"
if [ ! -d "$TEST_PROJECT" ]; then
    echo "Creating test Godot project..."
    mkdir -p "$TEST_PROJECT/scripts"

    # Create project.godot
    cat > "$TEST_PROJECT/project.godot" << 'GODOT'
; Engine configuration file.
config_features=PackedStringArray("4.2")
config/name="Test Godot Project"
run/main_scene="res://main.tscn"
GODOT

    # Create Player.gd
    cat > "$TEST_PROJECT/scripts/Player.gd" << 'GDSCRIPT'
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0

func _physics_process(delta):
    var direction = Input.get_axis("ui_left", "ui_right")
    velocity.x = direction * SPEED
    move_and_slide()
GDSCRIPT

    echo "✓ Test project created at $TEST_PROJECT"
else
    echo "✓ Test project already exists at $TEST_PROJECT"
fi

# Run tests
echo "Running tests..."
npm run test -- --run

echo ""
echo "Setup complete!"
echo ""
echo "To start development:"
echo "  npm run tauri dev"
echo ""
echo "To run tests:"
echo "  npm run test"
echo ""