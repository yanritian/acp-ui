# Custom Tool Development Guide

## Overview

This guide explains how to develop custom tools for Hermes Game Operator.

---

## Tool Architecture

### Tool Types

| Type | Purpose | Example |
|------|---------|---------|
| **File Tool** | File operations | Custom file analyzer |
| **Shell Tool** | Command execution | Build automation |
| **Network Tool** | HTTP/API calls | External API integration |
| **AI Tool** | AI/ML operations | Code generation |
| **Domain Tool** | Engine-specific | Godot scene parser |

---

## Creating a Custom Tool

### Step 1: Define Tool Interface

**src/tools/MyTool.ts**:
```typescript
import { Tool, ToolContext, ToolResult, ToolSchema } from '@hermes/core'

export class MyCustomTool extends Tool {
  name = 'myCustomTool'
  description = 'A custom tool for processing data'
  category = 'processing'
  
  schema: ToolSchema = {
    type: 'object',
    properties: {
      input: {
        type: 'string',
        description: 'Input data to process'
      },
      options: {
        type: 'object',
        properties: {
          format: {
            type: 'string',
            enum: ['json', 'xml', 'yaml'],
            default: 'json'
          },
          validate: {
            type: 'boolean',
            default: true
          }
        }
      }
    },
    required: ['input']
  }
  
  async execute(context: ToolContext, params: any): Promise<ToolResult> {
    // Implementation
  }
}
```

### Step 2: Implement Execute Method

```typescript
async execute(context: ToolContext, params: any): Promise<ToolResult> {
  const startTime = Date.now()
  
  try {
    // Validate input
    const validation = this.validateInput(params)
    if (!validation.valid) {
      return {
        success: false,
        error: validation.error,
        duration_ms: Date.now() - startTime
      }
    }
    
    // Check permissions
    if (!context.hasPermission(this.name)) {
      return {
        success: false,
        error: 'Permission denied',
        duration_ms: Date.now() - startTime
      }
    }
    
    // Execute tool logic
    const result = await this.processData(params.input, params.options)
    
    // Log operation
    context.log(`Tool ${this.name} executed successfully`)
    
    return {
      success: true,
      output: result,
      metadata: {
        processedAt: new Date().toISOString(),
        inputSize: params.input.length
      },
      duration_ms: Date.now() - startTime
    }
  } catch (error) {
    context.log(`Tool ${this.name} failed: ${error.message}`)
    
    return {
      success: false,
      error: error.message,
      stack: error.stack,
      duration_ms: Date.now() - startTime
    }
  }
}
```

### Step 3: Implement Helper Methods

```typescript
private validateInput(params: any): { valid: boolean; error?: string } {
  if (!params.input) {
    return { valid: false, error: 'Missing required parameter: input' }
  }
  
  if (typeof params.input !== 'string') {
    return { valid: false, error: 'Input must be a string' }
  }
  
  if (params.input.length === 0) {
    return { valid: false, error: 'Input cannot be empty' }
  }
  
  return { valid: true }
}

private async processData(input: string, options: any): Promise<any> {
  // Implement your processing logic
  let result = input
  
  // Apply transformations
  if (options.format === 'json') {
    result = JSON.stringify({ data: result })
  } else if (options.format === 'xml') {
    result = this.toJSON(result)
  }
  
  // Validate if requested
  if (options.validate) {
    const isValid = await this.validateOutput(result)
    if (!isValid) {
      throw new Error('Output validation failed')
    }
  }
  
  return result
}

private toJSON(input: string): string {
  // Convert to XML format
  return `<?xml version="1.0"?><data>${input}</data>`
}

private async validateOutput(output: string): Promise<boolean> {
  // Implement validation logic
  return output.length > 0
}
```

---

## Tool Registration

### Register in Plugin

```typescript
import { Plugin } from '@hermes/core'
import { MyCustomTool } from './tools/MyCustomTool'

export class MyPlugin extends Plugin {
  async initialize(): Promise<void> {
    // Register tool
    this.context.tools.register(new MyCustomTool())
    
    console.log('Custom tool registered')
  }
}
```

### Register in Application

```typescript
import { ToolRegistry } from '@hermes/core'
import { MyCustomTool } from './tools/MyCustomTool'

const registry = new ToolRegistry()
registry.register(new MyCustomTool())
```

---

## Tool Permissions

### Define Permissions

```typescript
export const toolPermissions = {
  myCustomTool: {
    level: 'approve', // silent, notify, approve, forbidden
    description: 'Process data with custom tool',
    risk: 'medium'
  }
}
```

### Check Permissions

```typescript
async execute(context: ToolContext, params: any): Promise<ToolResult> {
  // Check if approval is needed
  if (context.needsApproval(this.name)) {
    const approval = await context.requestApproval({
      tool: this.name,
      action: 'Process data',
      risk: 'medium',
      preview: {
        input: params.input.substring(0, 100)
      }
    })
    
    if (!approval.approved) {
      return {
        success: false,
        error: 'Approval denied'
      }
    }
  }
  
  // Continue execution
  // ...
}
```

---

## Tool Error Handling

### Error Types

```typescript
export enum ToolErrorType {
  VALIDATION = 'VALIDATION',
  PERMISSION = 'PERMISSION',
  EXECUTION = 'EXECUTION',
  TIMEOUT = 'TIMEOUT',
  NETWORK = 'NETWORK'
}
```

### Handle Errors

```typescript
async execute(context: ToolContext, params: any): Promise<ToolResult> {
  try {
    // Execute tool
    const result = await this.processData(params.input, params.options)
    
    return { success: true, output: result }
  } catch (error) {
    // Categorize error
    const errorType = this.categorizeError(error)
    
    // Log error
    context.log(`Tool error [${errorType}]: ${error.message}`)
    
    // Return structured error
    return {
      success: false,
      error: error.message,
      errorType,
      retryable: this.isRetryable(errorType),
      suggestions: this.getErrorSuggestions(errorType)
    }
  }
}

private categorizeError(error: Error): ToolErrorType {
  if (error.message.includes('validation')) {
    return ToolErrorType.VALIDATION
  }
  if (error.message.includes('permission')) {
    return ToolErrorType.PERMISSION
  }
  if (error.message.includes('timeout')) {
    return ToolErrorType.TIMEOUT
  }
  if (error.message.includes('network')) {
    return ToolErrorType.NETWORK
  }
  return ToolErrorType.EXECUTION
}

private isRetryable(errorType: ToolErrorType): boolean {
  return [ToolErrorType.TIMEOUT, ToolErrorType.NETWORK].includes(errorType)
}

private getErrorSuggestions(errorType: ToolErrorType): string[] {
  const suggestions: Record<ToolErrorType, string[]> = {
    [ToolErrorType.VALIDATION]: ['Check input format', 'Verify required fields'],
    [ToolErrorType.PERMISSION]: ['Request approval', 'Check permissions'],
    [ToolErrorType.EXECUTION]: ['Check logs', 'Contact support'],
    [ToolErrorType.TIMEOUT]: ['Increase timeout', 'Retry operation'],
    [ToolErrorType.NETWORK]: ['Check connection', 'Retry operation']
  }
  return suggestions[errorType] || []
}
```

---

## Tool Testing

### Unit Tests

```typescript
import { MyCustomTool } from './MyCustomTool'

describe('MyCustomTool', () => {
  let tool: MyCustomTool
  
  beforeEach(() => {
    tool = new MyCustomTool()
  })
  
  it('should validate input correctly', () => {
    const validation = tool.validateInput({ input: 'test' })
    expect(validation.valid).toBe(true)
  })
  
  it('should reject empty input', () => {
    const validation = tool.validateInput({ input: '' })
    expect(validation.valid).toBe(false)
    expect(validation.error).toContain('cannot be empty')
  })
  
  it('should process data correctly', async () => {
    const context = {
      hasPermission: () => true,
      log: jest.fn()
    }
    
    const result = await tool.execute(context as any, {
      input: 'test data',
      options: { format: 'json' }
    })
    
    expect(result.success).toBe(true)
    expect(result.output).toContain('test data')
  })
})
```

### Integration Tests

```typescript
import { ToolRegistry } from '@hermes/core'
import { MyCustomTool } from './MyCustomTool'

describe('Tool Integration', () => {
  it('should register and execute tool', async () => {
    const registry = new ToolRegistry()
    const tool = new MyCustomTool()
    
    registry.register(tool)
    
    const result = await registry.execute('myCustomTool', {
      input: 'test',
      options: { format: 'json' }
    })
    
    expect(result.success).toBe(true)
  })
})
```

---

## Tool Documentation

### Tool README

```markdown
# My Custom Tool

## Description
A custom tool for processing data with various formats.

## Usage

### Basic Usage
```typescript
const result = await tool.execute(context, {
  input: 'data to process',
  options: {
    format: 'json',
    validate: true
  }
})
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| input | string | Yes | Input data to process |
| options.format | string | No | Output format (json/xml/yaml) |
| options.validate | boolean | No | Validate output |

### Returns

| Field | Type | Description |
|-------|------|-------------|
| success | boolean | Operation success |
| output | any | Processed data |
| error | string | Error message (if failed) |
| metadata | object | Additional metadata |

### Examples

#### Example 1: JSON Format
```typescript
const result = await tool.execute(context, {
  input: 'test data',
  options: { format: 'json' }
})
// Output: {"data":"test data"}
```

#### Example 2: XML Format
```typescript
const result = await tool.execute(context, {
  input: 'test data',
  options: { format: 'xml' }
})
// Output: <?xml version="1.0"?><data>test data</data>
```

### Error Handling

The tool returns structured errors:

```typescript
{
  success: false,
  error: 'Error message',
  errorType: 'VALIDATION',
  retryable: false,
  suggestions: ['Check input format']
}
```

### Permissions

This tool requires approval before execution.

### Limitations

- Maximum input size: 1MB
- Supported formats: json, xml, yaml
- Validation: Basic structure validation only
```

---

## Best Practices

### 1. Validate All Inputs

```typescript
// Good
const validation = this.validateInput(params)
if (!validation.valid) {
  return { success: false, error: validation.error }
}

// Bad
const result = await this.processData(params.input) // May fail
```

### 2. Check Permissions

```typescript
// Good
if (!context.hasPermission(this.name)) {
  return { success: false, error: 'Permission denied' }
}

// Bad
const result = await context.executeCommand('dangerous') // No check
```

### 3. Handle Errors

```typescript
// Good
try {
  const result = await this.processData(params.input)
  return { success: true, output: result }
} catch (error) {
  return { success: false, error: error.message }
}

// Bad
const result = await this.processData(params.input) // May throw
return { success: true, output: result }
```

### 4. Log Operations

```typescript
// Good
context.log(`Tool ${this.name} started`)
const result = await this.processData(params.input)
context.log(`Tool ${this.name} completed`)

// Bad
const result = await this.processData(params.input) // No logging
```

### 5. Return Structured Results

```typescript
// Good
return {
  success: true,
  output: result,
  metadata: { processedAt: new Date().toISOString() },
  duration_ms: Date.now() - startTime
}

// Bad
return result // No structure
```

---

## Examples

### File Analyzer Tool

```typescript
export class FileAnalyzerTool extends Tool {
  name = 'fileAnalyzer'
  description = 'Analyze file structure and content'
  
  async execute(context: ToolContext, params: any): Promise<ToolResult> {
    const { path, options } = params
    
    try {
      // Read file
      const content = await context.readFile(path)
      
      // Analyze
      const analysis = {
        size: content.length,
        lines: content.split('\n').length,
        type: this.detectType(content),
        encoding: this.detectEncoding(content)
      }
      
      return {
        success: true,
        output: analysis,
        metadata: { path }
      }
    } catch (error) {
      return {
        success: false,
        error: error.message
      }
    }
  }
  
  private detectType(content: string): string {
    if (content.includes('function')) return 'javascript'
    if (content.includes('def ')) return 'python'
    if (content.includes('func ')) return 'go'
    return 'unknown'
  }
  
  private detectEncoding(content: string): string {
    // Detect encoding
    return 'utf-8'
  }
}
```

### Data Transformer Tool

```typescript
export class DataTransformerTool extends Tool {
  name = 'dataTransformer'
  description = 'Transform data between formats'
  
  async execute(context: ToolContext, params: any): Promise<ToolResult> {
    const { input, fromFormat, toFormat } = params
    
    try {
      // Parse input
      const data = this.parseData(input, fromFormat)
      
      // Transform
      const transformed = this.transformData(data)
      
      // Serialize output
      const output = this.serializeData(transformed, toFormat)
      
      return {
        success: true,
        output,
        metadata: {
          inputFormat: fromFormat,
          outputFormat: toFormat
        }
      }
    } catch (error) {
      return {
        success: false,
        error: error.message
      }
    }
  }
  
  private parseData(input: string, format: string): any {
    if (format === 'json') return JSON.parse(input)
    if (format === 'xml') return this.parseXML(input)
    throw new Error(`Unsupported format: ${format}`)
  }
  
  private transformData(data: any): any {
    // Implement transformation logic
    return data
  }
  
  private serializeData(data: any, format: string): string {
    if (format === 'json') return JSON.stringify(data)
    if (format === 'xml') return this.toXML(data)
    throw new Error(`Unsupported format: ${format}`)
  }
}
```

---

## Resources

- [Tool API Reference](https://docs.hermes-game-operator.com/api/tool)
- [Tool Examples](https://github.com/hermes-game-operator/tool-examples)
- [Plugin Development Guide](PLUGIN-DEVELOPMENT-GUIDE.md)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
