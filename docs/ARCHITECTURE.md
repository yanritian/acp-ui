# Architecture Documentation

This document describes the architecture of Hermes Game Operator.

## System Overview

```
┌─────────────────────────────────────────────────────────┐
│                      Client Layer                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │           Desktop UI / VSCode / Web               │  │
│  │  (Vue 3 + TypeScript + Pinia)                     │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  Operator Control Plane                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Task State Machine    Event Stream               │  │
│  │  Approval Queue        Error Handler              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                     Agent Runtime                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Hermes Agent Bridge    Task Executor              │  │
│  │  Tool Registry          Skill Runtime              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                      Domain Packs                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Godot Analyzer    Unity Analyzer    Ren'Py ...    │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   Tools & Environment                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │  File Tools    Shell Tools    MCP Tools            │  │
│  │  PathGuard     CommandGuard   Security Layer       │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. Client Layer

**Technology**: Vue 3 + TypeScript + Pinia

**Components**:
- GameOperatorView - Main operator interface
- OperatorControlBar - Task controls (pause/resume/stop)
- ProgressTimeline - Real-time event stream
- PlanPanel - Execution plan display
- ApprovalDrawer - Approval requests

**Responsibilities**:
- User interface
- State management
- API communication
- Event rendering

### 2. Operator Control Plane

**Technology**: Rust + Tauri

**Components**:
- TaskStateMachine - 10-state task lifecycle
- EventStore - Append-only event storage
- ApprovalQueue - Approval request management
- ErrorHandler - Unified error handling

**Responsibilities**:
- Task lifecycle management
- Event tracking
- Approval workflow
- Error management

### 3. Agent Runtime

**Technology**: Rust + Hermes Agent

**Components**:
- HermesAgentBridge - Hermes API integration
- TaskExecutor - Task execution orchestration
- ToolRegistry - Tool registration and dispatch
- SkillRuntime - Skill loading and execution

**Responsibilities**:
- Agent execution
- Tool dispatch
- Skill management
- Code generation

### 4. Domain Packs

**Technology**: Rust

**Components**:
- GodotAnalyzer - Godot project analysis
- GodotSceneParser - Scene file parsing
- GodotScriptAnalyzer - Script analysis

**Responsibilities**:
- Project structure analysis
- File type detection
- Domain-specific logic
- Validation

### 5. Tools & Environment

**Technology**: Rust

**Components**:
- FileTools - File read/write/patch operations
- ShellTools - Shell command execution
- PathGuard - Path validation
- CommandGuard - Command filtering

**Responsibilities**:
- File operations
- Command execution
- Security validation
- Backup management

## Data Flow

### Task Execution Flow

```
1. User starts task
   ↓
2. TaskStateMachine creates task (Idle → Planning)
   ↓
3. GodotAnalyzer analyzes project
   ↓
4. HermesAgent generates plan
   ↓
5. TaskStateMachine transitions (Planning → WaitingApproval)
   ↓
6. User approves plan
   ↓
7. TaskStateMachine transitions (WaitingApproval → Running)
   ↓
8. TaskExecutor executes steps
   ↓
9. FileTools apply modifications
   ↓
10. TaskStateMachine transitions (Running → Completed)
    ↓
11. Summary generated
```

### Event Flow

```
1. Component emits event
   ↓
2. EventStore appends event
   ↓
3. EventStream broadcasts to UI
   ↓
4. ProgressTimeline renders event
   ↓
5. User observes progress
```

### Approval Flow

```
1. Dangerous operation requested
   ↓
2. ApprovalQueue creates request
   ↓
3. UI displays approval drawer
   ↓
4. User reviews diff
   ↓
5. User approves/rejects
   ↓
6. ApprovalQueue updates request
   ↓
7. Operation proceeds/cancels
```

## State Machine

### Task States

```
Idle
  ↓ (start_task)
Planning
  ↓ (plan_ready)
WaitingApproval
  ↓ (approve)         ↓ (reject)
Running              Cancelled
  ↓ (pause)           ↓ (stop)
Paused               Cancelled
  ↓ (resume)
Running
  ↓ (complete)
Completed
```

### State Transitions

| From | To | Trigger | Condition |
|------|-----|---------|-----------|
| Idle | Planning | start_task | Valid project |
| Planning | WaitingApproval | plan_ready | Plan generated |
| WaitingApproval | Running | approve | User approved |
| WaitingApproval | Cancelled | reject | User rejected |
| Running | Paused | pause | User paused |
| Paused | Running | resume | User resumed |
| Running | Completed | complete | All steps done |
| Running | Failed | error | Error occurred |
| Running | Cancelling | stop | User stopped |
| Cancelling | Cancelled | cleanup | Cleanup done |

## Security Model

### Defense in Depth

```
Layer 1: PathGuard
  - Validates file paths
  - Prevents traversal attacks
  - Enforces allowed roots

Layer 2: CommandGuard
  - Whitelist-based filtering
  - Blocks dangerous commands
  - Requires approval

Layer 3: Approval System
  - User approval for modifications
  - Review diffs before approval
  - Audit trail

Layer 4: Event Tracking
  - Complete audit log
  - Immutable event store
  - Forensic capability
```

### Security Checks

**Path Validation**:
```rust
// Canonicalize path
let canonical = path.canonicalize()?;

// Check against allowed roots
for root in &self.allowed_roots {
    if canonical.starts_with(root) {
        return Ok(());
    }
}

Err(PathGuardError::PathOutsideBoundary)
```

**Command Validation**:
```rust
// Check whitelist
if !self.allowed_commands.contains(&command) {
    return Err(CommandGuardError::UnknownCommand);
}

// Check blacklist
if self.forbidden_commands.contains(&command) {
    return Err(CommandGuardError::ForbiddenCommand);
}

Ok(())
```

## Performance Considerations

### Caching Strategy

**Analysis Cache**:
- Cache project analysis results
- Invalidate on file changes
- TTL: 5 minutes

**Response Cache**:
- Cache Hermes API responses
- Cache identical requests
- TTL: 10 minutes

### Optimization Techniques

**Lazy Loading**:
- Load components on demand
- Defer non-critical operations
- Progressive rendering

**Batch Operations**:
- Batch file reads
- Batch API calls
- Reduce network overhead

**Memory Management**:
- Limit concurrent tasks
- Clear old events
- Compress backups

## Extension Points

### Adding New Tools

1. Implement tool interface
2. Register in ToolRegistry
3. Define permissions
4. Add documentation

### Adding New Domain Packs

1. Implement analyzer interface
2. Define project markers
3. Add file parsers
4. Create validation rules

### Adding New UI Components

1. Create Vue component
2. Define props/emits
3. Add to GameOperatorView
4. Style with CSS

## Scalability

### Horizontal Scaling

**Current**: Single-user desktop application

**Future**:
- Multi-user support
- Shared task queue
- Distributed execution

### Vertical Scaling

**Current**: Limited by local resources

**Optimization**:
- Efficient algorithms
- Memory optimization
- Async processing

## Monitoring & Observability

### Metrics

**Task Metrics**:
- Task duration
- Success rate
- Error rate
- Approval time

**Performance Metrics**:
- Analysis time
- Generation time
- Memory usage
- CPU usage

### Logging

**Log Levels**:
- Debug - Detailed debugging
- Info - General information
- Warning - Potential issues
- Error - Errors occurred

**Log Destinations**:
- Console
- File
- Remote logging service

### Alerting

**Alert Conditions**:
- High error rate
- Slow response time
- Memory threshold
- Disk space low

## Testing Strategy

### Unit Tests

**Coverage Target**: 80%

**Test Types**:
- State machine tests
- Security tests
- Tool tests
- Analyzer tests

### Integration Tests

**Coverage Target**: 70%

**Test Types**:
- API tests
- Workflow tests
- Security tests
- Performance tests

### E2E Tests

**Coverage Target**: 60%

**Test Types**:
- User workflow tests
- Integration tests
- Regression tests
- Performance tests

## Deployment

### Desktop Deployment

**Platforms**:
- Windows (MSI, EXE)
- macOS (DMG)
- Linux (DEB, RPM, AppImage)

**Distribution**:
- GitHub Releases
- Package managers
- Direct download

### Web Deployment

**Hosting**:
- GitHub Pages
- Netlify
- Vercel

**CDN**:
- Cloudflare
- AWS CloudFront
- Google Cloud CDN

### Mobile Deployment

**Platforms**:
- Android (APK, AAB)
- iOS (IPA)

**Distribution**:
- Google Play Store
- Apple App Store
- Direct download

## Future Enhancements

### Phase 2 Features

- Unity Domain Pack
- Ren'Py Domain Pack
- VSCode extension
- Mobile remote control

### Phase 3 Features

- Multi-agent collaboration
- Cloud-based operator
- Team collaboration
- Advanced analytics

### Phase 4 Features

- Plugin marketplace
- Custom domain packs
- Enterprise features
- Advanced security

## Resources

- [API Reference](api.md)
- [User Manual](USER-MANUAL.md)
- [Security Guide](SECURITY.md)
- [Performance Guide](PERFORMANCE.md)
- [Best Practices](BEST-PRACTICES.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
