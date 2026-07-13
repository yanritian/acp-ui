# Hermes Game Operator IDEA Plugin

IntelliJ IDEA client for Hermes Game Operator - Control Godot game development tasks directly from your IDE.

## Features

- **Task Management**: Create, pause, resume, and stop tasks
- **Timeline View**: Monitor task events in real-time
- **Approval Queue**: Review and approve/reject file changes
- **Multi-Task Support**: Manage multiple concurrent tasks

## Installation

### From JetBrains Marketplace (when published)

1. Open IntelliJ IDEA
2. Go to Settings → Plugins
3. Search for "Hermes Game Operator"
4. Click Install

### Manual Installation

1. Download the plugin ZIP from releases
2. Go to Settings → Plugins → ⚙️ → Install Plugin from Disk
3. Select the downloaded ZIP file

## Configuration

1. Go to Settings → Tools → Hermes Game Operator
2. Enter your Game Operator server URL (e.g., `http://127.0.0.1:1422`)
3. Enter your authentication token (if required)

## Usage

### Open Game Operator Tool Window

1. Click "Game Operator" in the right sidebar
2. Or go to View → Tool Windows → Game Operator

### Connect to Server

1. Click "Connect" button
2. Or go to Tools → Game Operator → Connect to Game Operator

### Start a New Task

1. Click "Start Task" button
2. Or go to Tools → Game Operator → Start New Task
3. Enter task goal

### Review Approvals

1. Open "Approvals" tab in the tool window
2. Review pending approvals
3. Click approve or reject

## Development

### Prerequisites

- IntelliJ IDEA 2023.3 or later
- JDK 17 or later
- Kotlin 1.9.21 or later

### Build

```bash
./gradlew build
```

### Run Plugin in Sandbox

```bash
./gradlew runIde
```

### Package Plugin

```bash
./gradlew buildPlugin
```

The plugin ZIP will be in `build/distributions/`.

## Requirements

- IntelliJ IDEA 2023.3 or later
- Hermes Game Operator server running

## License

MIT

## Author

yan_fan_tian
