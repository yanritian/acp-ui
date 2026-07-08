# Version Upgrade Guide

This guide helps you upgrade Hermes Game Operator to newer versions.

## Before Upgrading

### Check Current Version

```bash
# Desktop application
acp-ui --version

# Or check in application
Settings → About → Version
```

### Backup Your Data

```bash
# Backup configuration
cp -r ~/.config/acp-ui ~/.config/acp-ui.backup

# Windows
xcopy %APPDATA%\acp-ui %APPDATA%\acp-ui.backup /E /I

# Backup projects (optional)
git add .
git commit -m "Backup before upgrade"
```

### Review Release Notes

Check what's new in the latest version:
- GitHub Releases page
- CHANGELOG.md
- Migration guide (this document)

## Upgrade Methods

### Desktop Application

#### Windows

1. **Download new version**
   - Go to GitHub Releases
   - Download `.msi` or `.exe` installer

2. **Run installer**
   - Double-click installer
   - Follow installation wizard
   - Existing installation will be upgraded

3. **Verify upgrade**
   ```bash
   acp-ui --version
   ```

#### macOS

1. **Download new version**
   - Go to GitHub Releases
   - Download `.dmg` file

2. **Install**
   - Open `.dmg` file
   - Drag to Applications folder
   - Replace existing version

3. **Verify upgrade**
   ```bash
   acp-ui --version
   ```

#### Linux

**Debian/Ubuntu**:
```bash
# Download .deb package
wget https://github.com/.../acp-ui_X.Y.Z_amd64.deb

# Install
sudo dpkg -i acp-ui_X.Y.Z_amd64.deb

# Or upgrade
sudo apt upgrade acp-ui
```

**RPM-based**:
```bash
# Download .rpm package
wget https://github.com/.../acp-ui-X.Y.Z.x86_64.rpm

# Install
sudo rpm -U acp-ui-X.Y.Z.x86_64.rpm
```

**AppImage**:
```bash
# Download new AppImage
wget https://github.com/.../acp-ui-X.Y.Z.AppImage

# Make executable
chmod +x acp-ui-X.Y.Z.AppImage

# Replace old version
mv acp-ui-X.Y.Z.AppImage ~/Applications/acp-ui.AppImage
```

### Web Application

The web version is automatically updated when you refresh the page.

```bash
# Clear cache if needed
Ctrl + Shift + R  # Hard refresh
```

### Mobile Application

#### Android

1. **Download new APK**
   - Go to GitHub Releases
   - Download `.apk` file

2. **Install**
   - Enable "Install unknown apps" in Settings
   - Open APK file
   - Follow installation prompts

3. **Verify upgrade**
   - Open application
   - Check Settings → About

#### iOS

iOS builds require manual compilation:

```bash
# Pull latest code
git pull origin main

# Build
npm run tauri ios build -- --release

# Install via Xcode
```

## Version Compatibility

### Configuration Files

Configuration files are backward compatible:
- Old configs work with new versions
- New versions may add optional fields
- Deprecated fields are ignored

### API Compatibility

API changes follow semantic versioning:
- **Major version (X.0.0)**: Breaking changes
- **Minor version (0.X.0)**: New features, backward compatible
- **Patch version (0.0.X)**: Bug fixes, fully compatible

### Data Migration

Automatic migration for:
- Task history
- Event logs
- User preferences
- Agent configurations

Manual migration for:
- Custom skills
- API keys (re-enter recommended)

## Breaking Changes

### Version 1.0.0 → 2.0.0

**Breaking Changes**:

1. **Operator Protocol Changes**
   ```typescript
   // Old
   const task = await OperatorApi.startTask({
     domain: 'godot',
     goal: 'Add feature'
   })
   
   // New
   const task = await OperatorApi.startTask({
     domain: 'game.godot',  // Changed format
     goal: 'Add feature',
     mode: 'propose_then_apply',  // New required field
     approval_policy: 'safe_default'  // New required field
   })
   ```

2. **Event Stream Format**
   ```typescript
   // Old
   {
     "type": "file_changed",
     "data": { "path": "..." }
   }
   
   // New
   {
     "type": "file_modified",
     "payload": { "path": "..." }
   }
   ```

3. **Approval System**
   ```typescript
   // Old
   await OperatorApi.approve({
     task_id: "...",
     approval_id: "...",
     approved: true
   })
   
   // New
   await OperatorApi.approve({
     task_id: "...",
     approval_id: "...",
     decision: "approve",  // Changed from boolean
     comment: "..."  // New optional field
   })
   ```

**Migration Steps**:

1. Update API calls to use new format
2. Update event handlers for new event types
3. Test approval workflow
4. Review logs for deprecation warnings

### Version 0.9.0 → 1.0.0

**Breaking Changes**:

1. **State Machine Changes**
   - Added new states: `redirecting`, `cancelling`
   - State transitions changed
   - Update state handling logic

2. **Security Enhancements**
   - PathGuard now requires explicit allowed roots
   - CommandGuard whitelist updated
   - Review security configuration

3. **File Backup**
   - Automatic backups now enabled by default
   - Backup location changed
   - Update restore procedures

## Upgrade Checklist

### Pre-Upgrade

- [ ] Backup configuration
- [ ] Backup projects (Git commit)
- [ ] Review release notes
- [ ] Check compatibility
- [ ] Test in staging environment

### During Upgrade

- [ ] Download new version
- [ ] Install/upgrade application
- [ ] Verify installation
- [ ] Check version number

### Post-Upgrade

- [ ] Test core functionality
- [ ] Verify API compatibility
- [ ] Check event stream
- [ ] Test approval workflow
- [ ] Review logs for errors
- [ ] Update documentation
- [ ] Notify team members

## Rollback Procedure

If upgrade causes issues:

### Desktop Application

1. **Uninstall current version**
   ```bash
   # Windows
   Control Panel → Programs → Uninstall
   
   # macOS
   Drag to Trash
   
   # Linux
   sudo apt remove acp-ui
   ```

2. **Restore from backup**
   ```bash
   # Restore configuration
   rm -rf ~/.config/acp-ui
   cp -r ~/.config/acp-ui.backup ~/.config/acp-ui
   ```

3. **Install previous version**
   - Download previous version from GitHub Releases
   - Install normally

### Web Application

Web version automatically rolls back when you access the previous deployment.

### Mobile Application

1. **Uninstall current version**
2. **Download previous APK/IPA**
3. **Install previous version**
4. **Restore from backup**

## Troubleshooting

### Upgrade Fails

**Symptoms**: Installation error or crash

**Solutions**:
- Check system requirements
- Free up disk space
- Disable antivirus temporarily
- Run installer as administrator

### Configuration Not Loading

**Symptoms**: Settings lost after upgrade

**Solutions**:
- Check configuration file location
- Restore from backup
- Re-enter API keys
- Reset to defaults

### API Incompatibility

**Symptoms**: API calls fail

**Solutions**:
- Check API version in documentation
- Update API calls to new format
- Review migration guide
- Contact support

### Performance Issues

**Symptoms**: Slower after upgrade

**Solutions**:
- Clear cache
- Disable unnecessary features
- Check system requirements
- Review performance guide

## Version History

### 1.0.0 (2026-07-08)

**Features**:
- Complete Hermes Game Operator implementation
- 10-state task lifecycle
- 17 Tauri commands
- Approval queue system
- Unified error handling
- E2E test suite

**Documentation**:
- API reference
- User manual
- Deployment guide
- Security guide

### 0.9.0 (2026-06-22)

**Features**:
- Loop Engine implementation
- Goal-driven architecture
- Swarm engine
- Tool sandbox

**Bug Fixes**:
- 17 critical bugs fixed
- Performance improvements
- Security enhancements

### 0.1.14 (2026-05-05)

**Features**:
- Initial ACP-UI release
- Multi-agent support
- WebSocket remote connection
- Session management

## Best Practices

### 1. Test Before Production

Always test upgrades in staging environment first.

### 2. Backup Regularly

Keep regular backups of configuration and projects.

### 3. Read Release Notes

Always review release notes before upgrading.

### 4. Upgrade Gradually

For teams, upgrade one user at a time.

### 5. Monitor After Upgrade

Watch logs and metrics after upgrade.

## Resources

- [GitHub Releases](https://github.com/yourusername/acp-ui/releases)
- [CHANGELOG.md](../CHANGELOG.md)
- [User Manual](USER-MANUAL.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Support](mailto:support@example.com)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
