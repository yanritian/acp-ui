# Release Process Guide

## Release Overview

This guide outlines the complete release process for Hermes Game Operator, from planning to deployment.

---

## Release Planning

### Release Types

| Type | Frequency | Changes | Example |
|------|-----------|---------|---------|
| **Major** (X.0.0) | 3-6 months | Breaking changes, major features | v2.0.0 |
| **Minor** (0.X.0) | 1-2 months | New features, enhancements | v1.1.0 |
| **Patch** (0.0.X) | As needed | Bug fixes, security patches | v1.0.1 |

### Release Planning Steps

1. **Define Scope**
   - List features/fixes to include
   - Prioritize by importance
   - Estimate effort

2. **Set Timeline**
   - Code freeze date
   - Testing period
   - Release date

3. **Assign Resources**
   - Development team
   - QA team
   - Documentation team
   - Release manager

4. **Communicate Plan**
   - Internal announcement
   - Update roadmap
   - Notify stakeholders

---

## Release Preparation

### Code Freeze

**Timeline**: 1 week before release

**Actions**:
- [ ] Announce code freeze
- [ ] Merge approved PRs
- [ ] Reject non-critical PRs
- [ ] Update branch protection
- [ ] Create release branch

**Command**:
```bash
git checkout -b release/v1.0.0
```

### Version Bump

**Actions**:
- [ ] Update package.json
  ```json
  {
    "version": "1.0.0"
  }
  ```
- [ ] Update Cargo.toml
  ```toml
  [package]
  version = "1.0.0"
  ```
- [ ] Update CHANGELOG.md
- [ ] Update documentation

**Command**:
```bash
npm version 1.0.0
```

### Changelog Update

**Format**:
```markdown
# Changelog

## [1.0.0] - 2026-07-08

### Added
- New feature description

### Changed
- Change description

### Fixed
- Bug fix description

### Removed
- Removed feature description
```

**Actions**:
- [ ] Review all commits since last release
- [ ] Categorize changes
- [ ] Write descriptions
- [ ] Get review

---

## Testing Phase

### Quality Assurance

**Timeline**: 3-5 days

**Test Types**:
1. **Unit Tests**
   ```bash
   npm run test
   cargo test
   ```

2. **Integration Tests**
   ```bash
   npm run test:integration
   ```

3. **E2E Tests**
   ```bash
   npm run test:e2e
   ```

4. **Performance Tests**
   ```bash
   npm run test:performance
   ```

5. **Security Tests**
   ```bash
   npm run test:security
   ```

6. **Manual Testing**
   - Feature verification
   - Regression testing
   - Edge case testing

### Bug Triage

**Process**:
1. **Identify** bugs
2. **Categorize** by severity
3. **Prioritize** critical bugs
4. **Fix** or defer
5. **Verify** fixes

**Severity Levels**:
- **Critical**: Must fix before release
- **High**: Should fix before release
- **Medium**: Can fix in next release
- **Low**: Can defer

### Sign-Off

**Requirements**:
- [ ] All critical bugs fixed
- [ ] All tests passing
- [ ] Performance targets met
- [ ] Security audit complete
- [ ] Documentation reviewed

**Sign-Off Meeting**:
- Review test results
- Review bug status
- Review documentation
- Final approval

---

## Build Phase

### Build Artifacts

**Frontend**:
```bash
npm run build
# Output: dist/
```

**Backend**:
```bash
cd src-tauri
cargo build --release
# Output: target/release/
```

**Desktop Application**:
```bash
npm run tauri build
# Output:
# - Windows: src-tauri/target/release/bundle/msi/*.msi
# - macOS: src-tauri/target/release/bundle/dmg/*.dmg
# - Linux: src-tauri/target/release/bundle/deb/*.deb
```

**Web Application**:
```bash
npm run build:web
# Output: dist-web/
```

### Verification

**Actions**:
- [ ] Verify all artifacts created
- [ ] Check file sizes
- [ ] Generate checksums
- [ ] Test installation
- [ ] Test functionality

**Commands**:
```bash
# Generate checksums
sha256sum *.msi *.dmg *.deb > SHA256SUMS.txt

# Verify checksums
sha256sum -c SHA256SUMS.txt
```

---

## Release Phase

### GitHub Release

**Actions**:
1. **Create Release**
   ```bash
   gh release create v1.0.0 \
     --title "v1.0.0 - Release Name" \
     --notes "Release notes" \
     --target main
   ```

2. **Upload Artifacts**
   ```bash
   gh release upload v1.0.0 *.msi *.dmg *.deb
   ```

3. **Publish Release**
   - Set as latest release
   - Add release notes
   - Add screenshots

### Web Deployment

**Actions**:
1. **Deploy to GitHub Pages**
   ```bash
   npm run deploy:web
   ```

2. **Verify Deployment**
   - Check website loads
   - Test functionality
   - Verify performance

### NPM Publication (if applicable)

**Actions**:
```bash
npm publish
```

### Cargo Publication (if applicable)

**Actions**:
```bash
cargo publish
```

---

## Announcement Phase

### Internal Announcement

**Channels**:
- Email to team
- Slack message
- Team meeting

**Template**:
```
Subject: Hermes Game Operator v1.0.0 Released

Team,

We're excited to announce the release of Hermes Game Operator v1.0.0!

Key features:
- Feature 1
- Feature 2
- Feature 3

Download: https://github.com/yourusername/acp-ui/releases/tag/v1.0.0
Documentation: https://docs.example.com

Thank you to everyone who contributed!

Best regards,
Release Team
```

### External Announcement

**Channels**:
- Blog post
- Social media
- Newsletter
- Press release

**Blog Post Template**:
```markdown
# Hermes Game Operator v1.0.0 Released

We're excited to announce the release of Hermes Game Operator v1.0.0!

## What's New

### Feature 1
Description...

### Feature 2
Description...

### Feature 3
Description...

## Getting Started

Installation instructions...

## Resources

- [Documentation](https://docs.example.com)
- [GitHub Repository](https://github.com/yourusername/acp-ui)
- [Discord Community](https://discord.gg/example)

## Thank You

Thank you to all contributors...
```

**Social Media Post**:
```
🎉 Hermes Game Operator v1.0.0 is here!

New features:
✅ Feature 1
✅ Feature 2
✅ Feature 3

Download now: https://github.com/yourusername/acp-ui/releases/tag/v1.0.0

#GameDev #AI #Godot #IndieDev
```

---

## Post-Release Phase

### Monitoring

**First 24 Hours**:
- [ ] Monitor error rates
- [ ] Monitor performance
- [ ] Monitor user feedback
- [ ] Check support tickets
- [ ] Verify deployment

**First Week**:
- [ ] Daily monitoring
- [ ] User feedback review
- [ ] Bug triage
- [ ] Documentation updates

### Bug Fixes

**Process**:
1. **Collect** bug reports
2. **Triage** by severity
3. **Fix** critical bugs immediately
4. **Plan** fixes for next release
5. **Communicate** status

**Hotfix Process**:
```bash
# Create hotfix branch
git checkout -b hotfix/v1.0.1

# Fix bug
# ...

# Test
npm run test

# Merge
git checkout main
git merge hotfix/v1.0.1

# Tag
git tag v1.0.1

# Release
gh release create v1.0.1
```

### User Feedback

**Collection**:
- GitHub Issues
- Discord
- Email
- Social media

**Analysis**:
- Categorize feedback
- Identify trends
- Prioritize improvements
- Plan next release

---

## Release Metrics

### Success Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Download count | 1000 | TBD | ⏳ |
| Active users | 500 | TBD | ⏳ |
| Bug reports | < 10 | TBD | ⏳ |
| User satisfaction | 4.5/5 | TBD | ⏳ |
| Documentation views | 5000 | TBD | ⏳ |

### Process Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Release time | < 4 weeks | TBD | ⏳ |
| Test coverage | > 80% | 85% | ✅ |
| Bug fix time | < 1 week | TBD | ⏳ |
| Documentation | 100% | 100% | ✅ |

---

## Release Checklist Summary

### Pre-Release
- [ ] Release plan approved
- [ ] Code freeze announced
- [ ] Version bumped
- [ ] Changelog updated

### Testing
- [ ] All tests passing
- [ ] Performance targets met
- [ ] Security audit complete
- [ ] Manual testing complete

### Build
- [ ] All artifacts built
- [ ] Checksums generated
- [ ] Artifacts verified
- [ ] Installation tested

### Release
- [ ] GitHub release created
- [ ] Artifacts uploaded
- [ ] Web version deployed
- [ ] NPM/Cargo published (if applicable)

### Announcement
- [ ] Internal announcement sent
- [ ] Blog post published
- [ ] Social media posted
- [ ] Newsletter sent

### Post-Release
- [ ] Monitoring active
- [ ] Bug triage ready
- [ ] Feedback collection active
- [ ] Next release planned

---

## Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- [npm Version](https://docs.npmjs.com/cli/v8/commands/npm-version)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
