# Plugin Development Guide

## Overview

This guide explains how to develop plugins for Hermes Game Operator to extend its functionality.

---

## Plugin Architecture

### Plugin Types

| Type | Purpose | Example |
|------|---------|---------|
| **Tool Plugin** | Add new tools | Custom file analyzer |
| **Skill Plugin** | Add new skills | Godot optimization skill |
| **Hook Plugin** | Add execution hooks | Pre-commit validation |
| **UI Plugin** | Extend UI | Custom dashboard |
| **Domain Plugin** | Add engine support | Unity domain pack |

---

## Creating a Plugin

### Step 1: Plugin Structure

```
my-plugin/
├── package.json
├── README.md
├── src/
│   ├── index.ts          # Entry point
│   ├── tools/            # Tool implementations
│   ├── skills/           # Skill definitions
│   ├── hooks/            # Hook implementations
│   └── ui/               # UI components (optional)
├── dist/                 # Build output
└── tests/                # Test files
```

### Step 2: Package Configuration

**package.json**:
```json
{
  "name": "hermes-plugin-my-plugin",
  "version": "1.0.0",
  "description": "My custom plugin for Hermes Game Operator",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "lint": "eslint src/"
  },
  "dependencies": {
    "@hermes/core": "^1.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "@types/node": "^18.0.0"
  },
  "hermes": {
    "plugin": true,
    "version": "1.0.0"
  }
}
```

### Step 3: Plugin Entry Point

**src/index.ts**:
```typescript
import { Plugin, PluginContext } from '@hermes/core'

export class MyPlugin extends Plugin {
  constructor(context: PluginContext) {
    super(context)
  }

  async initialize(): Promise<void> {
    // Register tools
    this.registerTool('myTool', new MyTool())
    
    // Register skills
    this.registerSkill('mySkill', new MySkill())
    
    // Register hooks
    this.registerHook('before_file_write', new MyHook())
    
    console.log('MyPlugin initialized')
  }

  async shutdown(): Promise<void> {
    console.log('MyPlugin shutdown')
  }
}

export default MyPlugin
```

---

## Tool Plugin

### Creating a Custom Tool

**src/tools/MyTool.ts**:
```typescript
import { Tool, ToolContext, ToolResult } from '@hermes/core'

export class MyTool extends Tool {
  name = 'myTool'
  description = 'My custom tool'
  
  async execute(context: ToolContext, params: any): Promise<ToolResult> {
    try {
      // Validate input
      if (!params.input) {
        throw new Error('Missing required parameter: input')
      }
      
      // Check permissions
      if (!context.hasPermission('myTool')) {
        throw new Error('Permission denied')
      }
      
      // Execute tool logic
      const result = await this.processInput(params.input)
      
      return {
        success: true,
        output: result,
        metadata: {
          processedAt: new Date().toISOString()
        }
      }
    } catch (error) {
      return {
        success: false,
        error: error.message
      }
    }
  }
  
  private async processInput(input: string): Promise<any> {
    // Implement your logic here
    return { processed: input.toUpperCase() }
  }
}
```

### Tool Schema

```typescript
export const myToolSchema = {
  type: 'object',
  properties: {
    input: {
      type: 'string',
      description: 'Input to process'
    },
    options: {
      type: 'object',
      properties: {
        case: {
          type: 'string',
          enum: ['upper', 'lower']
        }
      }
    }
  },
  required: ['input']
}
```

---

## Skill Plugin

### Creating a Custom Skill

**src/skills/MySkill.ts**:
```typescript
import { Skill, SkillContext } from '@hermes/core'

export class MySkill extends Skill {
  name = 'mySkill'
  description = 'My custom skill'
  
  async execute(context: SkillContext, goal: string): Promise<void> {
    // Analyze goal
    const analysis = await this.analyzeGoal(goal)
    
    // Generate plan
    const plan = await this.generatePlan(analysis)
    
    // Execute plan
    for (const step of plan.steps) {
      await this.executeStep(context, step)
    }
  }
  
  private async analyzeGoal(goal: string): Promise<any> {
    // Implement goal analysis
    return { goal, type: 'custom' }
  }
  
  private async generatePlan(analysis: any): Promise<any> {
    // Implement plan generation
    return {
      steps: [
        { action: 'analyze', target: analysis.goal },
        { action: 'execute', target: analysis.goal },
        { action: 'validate', target: analysis.goal }
      ]
    }
  }
  
  private async executeStep(context: SkillContext, step: any): Promise<void> {
    // Implement step execution
    console.log(`Executing step: ${step.action}`)
  }
}
```

### Skill Documentation

**skills/my-skill/SKILL.md**:
```markdown
# My Custom Skill

## When to Use
Use this skill when you want to...

## Inputs
- `goal`: Description of what to achieve

## Workflow
1. Analyze the goal
2. Generate execution plan
3. Execute plan steps
4. Validate results

## Examples
### Example 1
**Goal**: "Process my custom data"
**Result**: Data processed successfully

## Guardrails
- Only process allowed file types
- Validate input before processing
- Log all operations
```

---

## Hook Plugin

### Creating a Custom Hook

**src/hooks/MyHook.ts**:
```typescript
import { Hook, HookContext, HookResult } from '@hermes/core'

export class MyHook extends Hook {
  name = 'before_file_write'
  description = 'My custom hook'
  
  async execute(context: HookContext, event: any): Promise<HookResult> {
    try {
      // Check if file should be modified
      if (event.path.endsWith('.custom')) {
        // Validate file content
        const validation = await this.validateContent(event.content)
        
        if (!validation.valid) {
          return {
            allowed: false,
            reason: validation.reason
          }
        }
      }
      
      return { allowed: true }
    } catch (error) {
      return {
        allowed: false,
        reason: error.message
      }
    }
  }
  
  private async validateContent(content: string): Promise<any> {
    // Implement validation logic
    return { valid: true }
  }
}
```

### Available Hooks

| Hook | Trigger | Use Case |
|------|---------|----------|
| `before_file_read` | Before reading file | Permission check |
| `after_file_read` | After reading file | Logging |
| `before_file_write` | Before writing file | Validation |
| `after_file_write` | After writing file | Backup |
| `before_command` | Before running command | Security check |
| `after_command` | After running command | Logging |
| `before_tool_call` | Before tool execution | Permission |
| `after_tool_call` | After tool execution | Metrics |

---

## UI Plugin

### Creating a Custom Component

**src/ui/MyComponent.vue**:
```vue
<template>
  <div class="my-component">
    <h3>{{ title }}</h3>
    <div class="content">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Props {
  title: string
}

const props = defineProps<Props>()

const data = ref<any>(null)

onMounted(async () => {
  // Load data
  data.value = await loadData()
})

async function loadData(): Promise<any> {
  // Implement data loading
  return { message: 'Hello from plugin' }
}
</script>

<style scoped>
.my-component {
  padding: 1rem;
  border: 1px solid #ccc;
  border-radius: 4px;
}
</style>
```

### Registering UI Components

```typescript
import MyComponent from './ui/MyComponent.vue'

export class MyPlugin extends Plugin {
  async initialize(): Promise<void> {
    // Register UI component
    this.context.ui.registerComponent('MyComponent', MyComponent)
  }
}
```

---

## Domain Plugin

### Creating a Domain Pack

**src/domains/MyDomain.ts**:
```typescript
import { DomainPack, DomainContext } from '@hermes/core'

export class MyDomain extends DomainPack {
  name = 'myDomain'
  description = 'My custom domain pack'
  version = '1.0.0'
  
  async detect(context: DomainContext, path: string): Promise<boolean> {
    // Check if path is a valid project
    const markerFile = await context.fileExists(path, 'marker.json')
    return markerFile
  }
  
  async analyze(context: DomainContext, path: string): Promise<any> {
    // Analyze project structure
    const files = await context.listDirectory(path)
    
    return {
      name: path.split('/').pop(),
      files: files.length,
      type: this.name
    }
  }
  
  async validate(context: DomainContext, path: string): Promise<any> {
    // Validate project
    const errors = []
    
    if (!(await context.fileExists(path, 'marker.json'))) {
      errors.push('Missing marker.json')
    }
    
    return {
      valid: errors.length === 0,
      errors
    }
  }
}
```

---

## Testing Plugins

### Unit Tests

**tests/MyTool.test.ts**:
```typescript
import { MyTool } from '../src/tools/MyTool'

describe('MyTool', () => {
  it('should process input correctly', async () => {
    const tool = new MyTool()
    const context = {
      hasPermission: () => true
    }
    
    const result = await tool.execute(context as any, {
      input: 'test'
    })
    
    expect(result.success).toBe(true)
    expect(result.output.processed).toBe('TEST')
  })
  
  it('should fail without permission', async () => {
    const tool = new MyTool()
    const context = {
      hasPermission: () => false
    }
    
    const result = await tool.execute(context as any, {
      input: 'test'
    })
    
    expect(result.success).toBe(false)
    expect(result.error).toContain('Permission denied')
  })
})
```

### Integration Tests

**tests/integration.test.ts**:
```typescript
import { PluginContext } from '@hermes/core'
import MyPlugin from '../src'

describe('MyPlugin Integration', () => {
  it('should initialize successfully', async () => {
    const context: PluginContext = {
      registerTool: jest.fn(),
      registerSkill: jest.fn(),
      registerHook: jest.fn()
    }
    
    const plugin = new MyPlugin(context)
    await plugin.initialize()
    
    expect(context.registerTool).toHaveBeenCalledWith('myTool', expect.anything())
    expect(context.registerSkill).toHaveBeenCalledWith('mySkill', expect.anything())
    expect(context.registerHook).toHaveBeenCalledWith('before_file_write', expect.anything())
  })
})
```

---

## Publishing Plugins

### Build Plugin

```bash
npm run build
```

### Test Plugin

```bash
npm run test
```

### Publish to NPM

```bash
npm publish
```

### Publish to Plugin Registry

```bash
hermes plugin publish
```

---

## Plugin Configuration

### User Configuration

**~/.config/hermes/plugins/my-plugin.json**:
```json
{
  "enabled": true,
  "config": {
    "option1": "value1",
    "option2": "value2"
  }
}
```

### Plugin Configuration Schema

```typescript
export const configSchema = {
  type: 'object',
  properties: {
    enabled: {
      type: 'boolean',
      default: true
    },
    config: {
      type: 'object',
      properties: {
        option1: {
          type: 'string'
        },
        option2: {
          type: 'string'
        }
      }
    }
  }
}
```

---

## Best Practices

### 1. Follow Naming Conventions

```typescript
// Good
export class MyCustomTool extends Tool {}

// Bad
export class tool extends Tool {}
```

### 2. Validate All Inputs

```typescript
// Good
if (!params.input) {
  throw new Error('Missing required parameter: input')
}

// Bad
const result = params.input.toUpperCase() // May fail
```

### 3. Handle Errors Gracefully

```typescript
// Good
try {
  const result = await riskyOperation()
  return { success: true, output: result }
} catch (error) {
  return { success: false, error: error.message }
}

// Bad
const result = await riskyOperation() // May throw
return { success: true, output: result }
```

### 4. Check Permissions

```typescript
// Good
if (!context.hasPermission('myTool')) {
  throw new Error('Permission denied')
}

// Bad
// No permission check
const result = await context.executeCommand('dangerous')
```

### 5. Log Operations

```typescript
// Good
console.log(`Executing tool: ${this.name}`)
const result = await this.execute()
console.log(`Tool completed: ${result.success}`)

// Bad
const result = await this.execute() // No logging
```

---

## Resources

- [Plugin API Reference](https://docs.hermes-game-operator.com/api/plugin)
- [Plugin Examples](https://github.com/hermes-game-operator/plugin-examples)
- [Plugin Registry](https://plugins.hermes-game-operator.com)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
