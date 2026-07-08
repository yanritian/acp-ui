# Release Notes Template

Use this template when creating release notes for Hermes Game Operator.

---

## 🎉 Hermes Game Operator v[VERSION] - [RELEASE_NAME]

**Release Date**: [DATE]  
**Tag**: [TAG]

---

### 🌟 What's New

#### Major Features

**[Feature Name]**
- Description of the feature
- Key capabilities
- Use cases

**[Feature Name]**
- Description of the feature
- Key capabilities
- Use cases

#### Minor Features

- Feature 1
- Feature 2
- Feature 3

---

### 🔧 Improvements

#### Performance
- Improvement 1 (XX% faster)
- Improvement 2 (reduced memory by XX%)
- Improvement 3

#### User Experience
- UI improvement 1
- UI improvement 2
- UX enhancement 1

#### Developer Experience
- API enhancement 1
- Documentation improvement 1
- Tooling improvement 1

---

### 🐛 Bug Fixes

- Fixed issue where [description] (#123)
- Fixed issue where [description] (#124)
- Fixed issue where [description] (#125)

---

### ⚠️ Breaking Changes

**[Change Name]**
- What changed
- Why it changed
- Migration steps:
  ```bash
  # Migration command
  ```

---

### 📚 Documentation

- New guide: [Guide Name](link)
- Updated: [Document Name](link)
- Added examples for [Feature]

---

### 🔒 Security

- Security improvement 1
- Security improvement 2
- Updated dependencies to fix vulnerabilities

---

### 📦 Installation

#### Desktop Application

**Windows**:
```powershell
# Download MSI installer
Invoke-WebRequest -Uri "https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/hermes-game-operator-[VERSION]-x64.msi" -OutFile "hermes-game-operator.msi"
Start-Process msiexec.exe -ArgumentList "/i hermes-game-operator.msi"
```

**macOS**:
```bash
# Download DMG
curl -L -o hermes-game-operator.dmg "https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/hermes-game-operator-[VERSION].dmg"
open hermes-game-operator.dmg
```

**Linux**:
```bash
# Debian/Ubuntu
wget https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/hermes-game-operator_[VERSION]_amd64.deb
sudo dpkg -i hermes-game-operator_[VERSION]_amd64.deb

# Fedora/RHEL
wget https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/hermes-game-operator-[VERSION].x86_64.rpm
sudo rpm -i hermes-game-operator-[VERSION].x86_64.rpm
```

#### Web Application

Visit: [https://acp-ui.github.io/](https://acp-ui.github.io/)

#### Mobile Applications

**Android**:
```bash
# Download APK
wget https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/hermes-game-operator-[VERSION].apk
```

**iOS**:
Build from source (requires Apple Developer account)

---

### 🔄 Upgrade Guide

#### From v[PREV_VERSION] to v[VERSION]

**Automatic Upgrade**:
```bash
# Desktop application
# Just install the new version over the existing one

# Web application
# No action needed, just refresh the page
```

**Manual Migration**:
1. Backup your configuration:
   ```bash
   cp ~/.config/hermes-operator ~/.config/hermes-operator.backup
   ```

2. Install new version

3. Verify upgrade:
   ```bash
   hermes-operator --version
   ```

4. Check for breaking changes (see above)

---

### 📊 Statistics

- **Commits**: [NUMBER] commits since last release
- **Contributors**: [NUMBER] contributors
- **Files Changed**: [NUMBER] files
- **Lines Added**: [NUMBER] lines
- **Lines Removed**: [NUMBER] lines

---

### 🙏 Contributors

Thanks to all the contributors who made this release possible:

- @contributor1
- @contributor2
- @contributor3

---

### 📖 Resources

- **Documentation**: [https://docs.example.com](https://docs.example.com)
- **API Reference**: [https://docs.example.com/api](https://docs.example.com/api)
- **GitHub Repository**: [https://github.com/yourusername/acp-ui](https://github.com/yourusername/acp-ui)
- **Discord**: [https://discord.gg/example](https://discord.gg/example)
- **Support**: [support@example.com](mailto:support@example.com)

---

### 🔮 What's Next

In the next release, we're planning:

- [Feature 1]
- [Feature 2]
- [Feature 3]

See our [Roadmap](https://github.com/yourusername/acp-ui/blob/main/docs/ROADMAP.md) for more details.

---

### 📝 Full Changelog

For a complete list of changes, see the [Full Changelog](https://github.com/yourusername/acp-ui/compare/v[PREV_VERSION]...v[VERSION]).

---

## Verification

To verify your download, check the SHA256 checksum:

```bash
# Download checksum file
wget https://github.com/yourusername/acp-ui/releases/download/v[VERSION]/SHA256SUMS.txt

# Verify checksum
sha256sum -c SHA256SUMS.txt
```

---

## Known Issues

See [Known Issues](https://github.com/yourusername/acp-ui/blob/main/docs/KNOWN-ISSUES.md) for a list of known issues in this release.

---

## Support

If you encounter any issues:

1. Check the [FAQ](https://github.com/yourusername/acp-ui/blob/main/docs/FAQ.md)
2. Search [GitHub Issues](https://github.com/yourusername/acp-ui/issues)
3. Ask in [Discord](https://discord.gg/example)
4. Email [support@example.com](mailto:support@example.com)

---

**Happy game development!** 🎮

---

## Template Variables

Replace these variables when using this template:

- `[VERSION]` - Release version (e.g., 1.0.0)
- `[PREV_VERSION]` - Previous version (e.g., 0.9.0)
- `[RELEASE_NAME]` - Release name (e.g., "Phoenix")
- `[DATE]` - Release date (e.g., 2026-07-08)
- `[TAG]` - Git tag (e.g., v1.0.0)
- `[NUMBER]` - Numeric values

---

**Template Version**: 1.0.0  
**Last Updated**: 2026-07-08
