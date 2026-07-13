# Hermes Game Operator VSCode Extension

VSCode client for Hermes Game Operator - Control Godot game development tasks directly from your IDE.

## Features

- **Task Management**: Create, pause, resume, and stop tasks
- **Timeline View**: Monitor task events in real-time
- **Approval Queue**: Review and approve/reject file changes
- **Multi-Task Support**: Manage multiple concurrent tasks

## Installation

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Hermes Game Operator"
4. Click Install

## Configuration

Add to your VSCode settings:

```json
{
  "gameOperator.serverUrl": "http://127.0.0.1:1422",
  "gameOperator.authToken": "your-token-here"
}
```

## Usage

### Connect to Game Operator

1. Open Command Palette (Ctrl+Shift+P)
2. Type "Connect to Game Operator"
3. Press Enter

### Start a New Task

1. Click the "+" icon in the Game Operator view
2. Enter task goal (e.g., "Add double jump to the player character")
3. Press Enter

### Review Approvals

1. Expand the "Approvals" view
2. Click on an approval to see details
3. Use the approve/reject buttons

## Development

```bash
# Install dependencies
npm install

# Compile
npm run compile

# Watch mode
npm run watch

# Run tests
npm test
```

## Requirements

- VSCode 1.85.0 or later
- Hermes Game Operator server running

## License

MIT

## Author

yan_fan_tian
