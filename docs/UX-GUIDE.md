# User Experience Guide

This guide provides UX design principles and best practices for Hermes Game Operator.

## Design Principles

### 1. Clarity Over Cleverness

**Principle**: Users should understand what's happening without guessing

**Examples**:
- ✅ Clear status indicators (Planning, Running, Completed)
- ✅ Descriptive event messages
- ✅ Explicit approval requests

**Anti-patterns**:
- ❌ Ambiguous status messages
- ❌ Hidden state changes
- ❌ Unclear consequences

---

### 2. User Control

**Principle**: Users should always feel in control

**Features**:
- Pause/Resume/Stop controls
- Approval system for dangerous operations
- Redirect capability
- Undo support (via backups)

**Implementation**:
```typescript
// Always show control options
const controls = [
  { action: 'pause', label: 'Pause', icon: '⏸️' },
  { action: 'resume', label: 'Resume', icon: '▶️' },
  { action: 'stop', label: 'Stop', icon: '⏹️' }
]
```

---

### 3. Transparency

**Principle**: Show users what the operator is doing

**Features**:
- Real-time event stream
- Progress timeline
- Plan visibility
- File change preview

**Implementation**:
```typescript
// Show every step
events.forEach(event => {
  console.log(`[${event.timestamp}] ${event.title}`)
})
```

---

### 4. Safety First

**Principle**: Prevent mistakes before they happen

**Features**:
- PathGuard validation
- CommandGuard filtering
- Approval requirements
- Automatic backups

**Implementation**:
```typescript
// Validate before execution
if (!pathGuard.validate(path)) {
  showError('Path outside allowed boundaries')
  return
}
```

---

### 5. Progressive Disclosure

**Principle**: Show complexity only when needed

**Levels**:
1. **Basic**: Start task, view progress
2. **Intermediate**: Review plan, approve changes
3. **Advanced**: Redirect, custom settings

**Implementation**:
```vue
<!-- Basic view -->
<div v-if="!showAdvanced">
  <StartTaskButton />
  <ProgressBar />
</div>

<!-- Advanced view -->
<div v-else>
  <AdvancedControls />
  <DebugPanel />
</div>
```

---

## UI Components

### Task Control Bar

**Purpose**: Primary task controls

**Design**:
```
┌─────────────────────────────────────────┐
│ [⏸️ Pause] [▶️ Resume] [⏹️ Stop]       │
│ Status: Running | Duration: 2:34        │
└─────────────────────────────────────────┘
```

**Best Practices**:
- Always visible during task execution
- Clear visual hierarchy
- Disabled states when not applicable
- Keyboard shortcuts

---

### Progress Timeline

**Purpose**: Real-time event display

**Design**:
```
┌─────────────────────────────────────────┐
│ 10:30:15 | ✅ Project analyzed          │
│ 10:30:18 | ⏳ Generating plan...        │
│ 10:30:25 | ✅ Plan ready                │
│ 10:30:30 | ⚠️ Approval requested       │
└─────────────────────────────────────────┘
```

**Best Practices**:
- Chronological order (newest first)
- Color-coded by severity
- Expandable for details
- Searchable/filterable

---

### Plan Panel

**Purpose**: Execution plan display

**Design**:
```
┌─────────────────────────────────────────┐
│ Execution Plan                          │
│ ─────────────────────────────────────── │
│ ✓ Step 1: Analyze project              │
│ ✓ Step 2: Identify player controller   │
│ ⏳ Step 3: Generate implementation      │
│ ○ Step 4: Apply changes                │
│ ○ Step 5: Validate results             │
└─────────────────────────────────────────┘
```

**Best Practices**:
- Clear step numbering
- Visual status indicators
- Estimated time per step
- Expandable details

---

### Approval Drawer

**Purpose**: Approval request display

**Design**:
```
┌─────────────────────────────────────────┐
│ ⚠️ Approval Required                    │
│ ─────────────────────────────────────── │
│ Action: Modify file                     │
│ File: scripts/Player.gd                │
│ Risk: Medium                            │
│                                         │
│ [View Diff]                             │
│                                         │
│ [✓ Approve] [✗ Reject]                 │
└─────────────────────────────────────────┘
```

**Best Practices**:
- Clear risk indication
- Easy access to diff
- Prominent approve/reject buttons
- Timeout indicator

---

## Interaction Patterns

### Task Creation

**Flow**:
1. Select project
2. Enter goal
3. Start task
4. Review plan
5. Approve execution

**Best Practices**:
- Autocomplete for project paths
- Goal suggestions based on project
- Preview before starting
- Clear success/failure feedback

---

### Plan Review

**Flow**:
1. View plan steps
2. Expand details
3. Check file list
4. Approve or modify

**Best Practices**:
- Side-by-side comparison
- File impact summary
- Time estimates
- Risk assessment

---

### File Modification

**Flow**:
1. View diff
2. Check changes
3. Approve/reject
4. Confirm action

**Best Practices**:
- Syntax-highlighted diff
- Line-by-line comparison
- Change summary
- Undo option

---

## Accessibility

### Keyboard Navigation

**Shortcuts**:
```
Ctrl + Enter  → Start task
Ctrl + P      → Pause task
Ctrl + R      → Resume task
Ctrl + S      → Stop task
Ctrl + A      → Approve
Ctrl + D      → View diff
```

**Implementation**:
```typescript
document.addEventListener('keydown', (e) => {
  if (e.ctrlKey && e.key === 'Enter') {
    startTask()
  }
})
```

---

### Screen Reader Support

**ARIA Labels**:
```vue
<button 
  aria-label="Pause current task"
  role="button"
>
  Pause
</button>
```

**Focus Management**:
```typescript
// Move focus to important elements
importantElement.focus()
```

---

### Color Contrast

**Standards**:
- WCAG 2.1 AA compliance
- Minimum 4.5:1 contrast ratio
- Color-blind friendly palette

**Implementation**:
```css
/* Good contrast */
.text-primary {
  color: #1a1a1a; /* Dark text on light background */
}

.text-secondary {
  color: #666666; /* Medium contrast */
}
```

---

## Responsive Design

### Desktop (> 1200px)

**Layout**:
```
┌─────────────────────────────────────────┐
│ Sidebar (300px) │ Main Content          │
│                 │                       │
│ - Project       │ - Control Bar         │
│ - Tasks         │ - Timeline            │
│ - Settings      │ - Plan Panel          │
│                 │                       │
└─────────────────────────────────────────┘
```

---

### Tablet (768px - 1200px)

**Layout**:
```
┌─────────────────────────────────────────┐
│ Collapsed Sidebar                       │
│ ─────────────────────────────────────── │
│ Main Content                            │
│                                         │
│ - Control Bar                           │
│ - Timeline                              │
│ - Plan Panel (collapsible)              │
│                                         │
└─────────────────────────────────────────┘
```

---

### Mobile (< 768px)

**Layout**:
```
┌─────────────────────────────────────────┐
│ Hamburger Menu                          │
│ ─────────────────────────────────────── │
│ Main Content                            │
│                                         │
│ - Control Bar (compact)                 │
│ - Timeline (vertical)                   │
│ - Plan Panel (modal)                    │
│                                         │
└─────────────────────────────────────────┘
```

---

## Error Handling

### User-Friendly Errors

**Bad**:
```
Error: PathGuardError::PathOutsideBoundary
```

**Good**:
```
⚠️ The file path is outside your project directory.
Please select a file within your project.
```

---

### Error Categories

**1. Validation Errors**
- Invalid project path
- Missing required files
- Invalid goal

**2. Permission Errors**
- Path outside boundary
- Forbidden operation
- Insufficient permissions

**3. Network Errors**
- API unavailable
- Connection timeout
- Rate limit exceeded

**4. Execution Errors**
- Plan generation failed
- Code generation failed
- File modification failed

---

### Error Recovery

**Strategies**:
1. **Retry**: Automatic retry with backoff
2. **Fallback**: Alternative approach
3. **Manual**: User intervention required
4. **Abort**: Stop and notify

**Implementation**:
```typescript
try {
  await riskyOperation()
} catch (error) {
  if (error.retryable) {
    await retry(operation)
  } else if (error.fallback) {
    await fallback(operation)
  } else {
    showError(error.message)
  }
}
```

---

## Performance UX

### Loading States

**Skeleton Screens**:
```vue
<div class="skeleton">
  <div class="skeleton-header"></div>
  <div class="skeleton-content"></div>
</div>
```

**Progress Indicators**:
```vue
<ProgressBar :progress="taskProgress" />
```

---

### Optimistic Updates

**Pattern**:
```typescript
// Update UI immediately
task.status = 'running'

// Then make API call
await api.startTask(task)
```

---

### Caching

**Strategy**:
- Cache project analysis
- Cache API responses
- Cache user preferences

**Implementation**:
```typescript
const cache = new Map()

function getCached(key, fetcher) {
  if (cache.has(key)) {
    return cache.get(key)
  }
  const value = fetcher()
  cache.set(key, value)
  return value
}
```

---

## Internationalization

### Supported Languages

- English (en)
- 中文 (zh)
- 日本語 (ja)
- 한국어 (ko)
- Español (es)
- Français (fr)
- Deutsch (de)
- Português (pt)
- Русский (ru)
- العربية (ar)

---

### Translation Keys

**Structure**:
```json
{
  "task": {
    "start": "Start Task",
    "pause": "Pause Task",
    "resume": "Resume Task",
    "stop": "Stop Task"
  },
  "status": {
    "idle": "Idle",
    "planning": "Planning",
    "running": "Running",
    "completed": "Completed"
  }
}
```

---

### RTL Support

**Implementation**:
```css
[dir="rtl"] .sidebar {
  right: 0;
  left: auto;
}

[dir="rtl"] .icon {
  transform: scaleX(-1);
}
```

---

## Best Practices Summary

### Do's

✅ Use clear, descriptive labels  
✅ Provide visual feedback for all actions  
✅ Show progress for long operations  
✅ Allow users to cancel operations  
✅ Use consistent terminology  
✅ Provide keyboard shortcuts  
✅ Support screen readers  
✅ Use color + icon + text for status  

### Don'ts

❌ Use ambiguous labels  
❌ Hide important information  
❌ Block the UI during operations  
❌ Force users to wait without feedback  
❌ Use inconsistent terminology  
❌ Rely only on keyboard shortcuts  
❌ Use color alone to convey information  
❌ Show technical error messages  

---

## Resources

- [Material Design](https://material.io/design)
- [Apple HIG](https://developer.apple.com/design/human-interface-guidelines)
- [WCAG 2.1](https://www.w3.org/WAI/WCAG21/quickref/)
- [Vue.js Best Practices](https://vuejs.org/guide/best-practices.html)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08
