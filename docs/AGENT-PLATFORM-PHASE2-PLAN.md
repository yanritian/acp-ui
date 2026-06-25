# Phase 2: Desktop Development Agents

## Overview

Phase 2 extends Agent Platform to support desktop application development:
- Tauri + Electron adapters
- Cross-platform build support (Windows/Mac/Linux)
- Desktop-specific scene routing
- UI component generation

## Timeline: 4 Weeks

### Week 1-2: Tauri Adapter Enhancement

**Goal**: Full Tauri desktop app development support

#### Tasks

1. **Tauri Adapter Enhancement** (`src-tauri/src/agent_adapter/tauri_adapter.rs`)
   - Tauri-specific capabilities: `tauri-build`, `tauri-dev`, `tauri-release`
   - Build target detection (Windows/Mac/Linux)
   - Tauri.conf.json manipulation
   - Cargo.toml dependency management

2. **Desktop Scene Detection** (`src-tauri/src/project_context.rs` enhancement)
   - Detect Tauri projects from `tauri.conf.json`
   - Detect Electron projects from `electron-builder.yml`
   - Platform-specific routing (Windows vs Mac vs Linux)

3. **Desktop Build Commands**
   - `tauri_build_dev` - Development build with hot reload
   - `tauri_build_release` - Production build with optimization
   - `tauri_cross_compile` - Cross-platform build

4. **Frontend API** (`src/api/desktop.ts`)
   - Desktop-specific TypeScript API
   - Build progress tracking
   - Platform selection UI

### Week 3: Electron Adapter

**Goal**: Electron app development support

#### Tasks

1. **Electron Adapter** (`src-tauri/src/agent_adapter/electron_adapter.rs`)
   - Electron-specific capabilities: `electron-build`, `electron-dev`
   - package.json manipulation
   - Electron-builder configuration

2. **Electron/Tauri Hybrid Projects**
   - Support projects with both Tauri and Electron
   - Smart adapter selection based on project config

### Week 4: Integration & Testing

#### Tasks

1. **E2E Tests for Desktop**
   - Test Tauri build flow
   - Test Electron build flow
   - Cross-platform validation

2. **Desktop UI Components**
   - Vue components for desktop development
   - Build progress visualization
   - Platform selector component

## New Types

```rust
// Desktop-specific scene types
pub enum DesktopScene {
    TauriApp,
    ElectronApp,
    FlutterDesktop,
    QtApp,
    GtkApp,
}

// Desktop platform
pub enum DesktopPlatform {
    Windows,
    MacOS,
    Linux,
    CrossPlatform,
}

// Desktop build config
pub struct DesktopBuildConfig {
    pub target: DesktopPlatform,
    pub dev_mode: bool,
    pub hot_reload: bool,
    pub release_mode: bool,
    pub optimization_level: OptimizationLevel,
}
```

## New Tauri Commands

| Command | Description |
|---------|-------------|
| `desktop_detect_project` | Detect desktop framework |
| `desktop_build_tauri` | Build Tauri app |
| `desktop_build_electron` | Build Electron app |
| `desktop_get_platform` | Get current platform |
| `desktop_cross_compile` | Cross-compile for other platforms |
| `desktop_hot_reload` | Start hot reload dev server |

## Dependencies

- No new Rust dependencies needed
- Frontend: `@tauri-apps/api` (already installed)

## Success Criteria

1. ✅ Tauri adapter can build dev/release
2. ✅ Electron adapter can build dev/release
3. ✅ Scene detection correctly identifies desktop projects
4. ✅ Cross-platform builds work (at least Windows + Mac)
5. ✅ E2E tests pass for desktop flows

## Risks

1. **Cross-compilation complexity**: Mac builds on Windows require special setup
2. **Electron dependency**: Requires Node.js environment
3. **Build time**: Desktop builds can be slow (5-10 min)

## Mitigation

1. Start with single-platform builds first
2. Use Docker for cross-compilation
3. Cache build artifacts aggressively