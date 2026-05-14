/// Permission Mode (Claw Code inspired 5-level hierarchy)
enum PermissionMode {
  /// Can only read files, no write operations
  readOnly,

  /// Can read and write within workspace directory
  workspaceWrite,

  /// Full access without permission checks (dangerous)
  dangerFullAccess,

  /// Ask user for each operation via prompt
  prompt,

  /// Auto-allow with rule-based checks (default)
  allow,
}

/// Permission Rule
class PermissionRule {
  final String pattern;
  final bool allow;
  final String? description;

  const PermissionRule({
    required this.pattern,
    required this.allow,
    this.description,
  });
}

/// Permission Result
class PermissionResult {
  final bool allowed;
  final String reason;
  final String? matchedRule;

  const PermissionResult({
    required this.allowed,
    required this.reason,
    this.matchedRule,
  });
}

/// Permission Configuration
class PermissionConfig {
  final PermissionMode mode;
  final String? cwd;
  final List<PermissionRule> allow;
  final List<PermissionRule> deny;
  final bool denyByDefault;
  final bool askOverride;

  const PermissionConfig({
    required this.mode,
    this.cwd,
    this.allow = const [],
    this.deny = const [],
    this.denyByDefault = false,
    this.askOverride = false,
  });

  /// Default configuration for Allow mode
  factory PermissionConfig.defaultAllow() => PermissionConfig(
    mode: PermissionMode.allow,
    deny: [
      PermissionRule(
        pattern: 'Bash:rm -rf.*',
        allow: false,
        description: 'Cannot run destructive commands',
      ),
    ],
  );

  /// ReadOnly configuration
  factory PermissionConfig.readOnly() => PermissionConfig(
    mode: PermissionMode.readOnly,
    denyByDefault: true,
    allow: [
      PermissionRule(
        pattern: 'tool:Read:.*',
        allow: true,
        description: 'Can read any file',
      ),
    ],
  );

  /// Configuration for code-reviewer agent
  factory PermissionConfig.codeReviewer() => PermissionConfig(
    mode: PermissionMode.readOnly,
    denyByDefault: true,
    allow: [
      PermissionRule(
        pattern: 'tool:Read:.*',
        allow: true,
        description: 'Can read any file',
      ),
      PermissionRule(
        pattern: 'tool:Bash:npx tsc.*',
        allow: true,
        description: 'Can run TypeScript compiler',
      ),
      PermissionRule(
        pattern: 'tool:Bash:npx eslint.*',
        allow: true,
        description: 'Can run ESLint',
      ),
    ],
  );
}

/// Permission Checker
class PermissionChecker {
  final PermissionConfig config;
  final List<RegExp> _allowPatterns;
  final List<RegExp> _denyPatterns;

  PermissionChecker(this.config)
      : _allowPatterns = config.allow.map((r) => RegExp(r.pattern)).toList(),
        _denyPatterns = config.deny.map((r) => RegExp(r.pattern)).toList();

  /// Check if a tool operation is allowed
  PermissionResult checkTool(String toolName, String args) {
    // Apply permission mode hierarchy
    switch (config.mode) {
      case PermissionMode.readOnly:
        final isReadTool = toolName == 'Read' ||
            (toolName.startsWith('Read:') && toolName.length > 5);
        if (!isReadTool) {
          return PermissionResult(
            allowed: false,
            reason: 'ReadOnly mode: only Read operations allowed',
            matchedRule: 'mode-ReadOnly',
          );
        }

      case PermissionMode.workspaceWrite:
        if (config.cwd != null && !args.startsWith(config.cwd!)) {
          return PermissionResult(
            allowed: false,
            reason: 'WorkspaceWrite mode: path outside workspace',
            matchedRule: 'mode-WorkspaceWrite',
          );
        }

      case PermissionMode.dangerFullAccess:
        return PermissionResult(
          allowed: true,
          reason: 'DangerFullAccess mode: all operations allowed',
          matchedRule: 'mode-DangerFullAccess',
        );

      case PermissionMode.prompt:
        return PermissionResult(
          allowed: config.askOverride,
          reason: 'Prompt mode: requires user confirmation',
          matchedRule: 'mode-Prompt',
        );

      case PermissionMode.allow:
        // Continue with rule-based checking
        break;
    }

    final operation = 'tool:$toolName:$args';

    // Check deny rules first
    for (final pattern in _denyPatterns) {
      if (pattern.hasMatch(operation) || pattern.hasMatch(toolName) || pattern.hasMatch(args)) {
        return PermissionResult(
          allowed: false,
          reason: 'Denied by rule',
          matchedRule: pattern.pattern,
        );
      }
    }

    // Check dangerous operations
    if (isDangerousOperation(toolName, args)) {
      return PermissionResult(
        allowed: false,
        reason: 'Operation deemed dangerous',
        matchedRule: 'builtin-dangerous',
      );
    }

    // Check allow rules
    for (final pattern in _allowPatterns) {
      if (pattern.hasMatch(operation) || pattern.hasMatch(toolName) || pattern.hasMatch(args)) {
        return PermissionResult(
          allowed: true,
          reason: 'Allowed by rule',
          matchedRule: pattern.pattern,
        );
      }
    }

    // Default behavior
    if (config.denyByDefault) {
      return PermissionResult(
        allowed: false,
        reason: 'Not explicitly allowed (deny by default)',
        matchedRule: null,
      );
    }

    return PermissionResult(
      allowed: true,
      reason: 'No deny rules matched',
      matchedRule: null,
    );
  }

  /// Check if operation is inherently dangerous
  bool isDangerousOperation(String toolName, String args) {
    final dangerousPatterns = [
      'Bash:rm -rf',
      'Bash:rm -r',
      'Bash:dd',
      'Bash:mkfs',
      'Bash:shutdown',
      'Bash:reboot',
      'Write:/etc/',
      'Read:/etc/shadow',
    ];

    final operation = '$toolName:$args';
    final lowerArgs = args.toLowerCase();

    for (final pattern in dangerousPatterns) {
      if (operation.startsWith(pattern) || lowerArgs.contains(pattern.toLowerCase())) {
        return true;
      }
    }

    return false;
  }

  /// Check Bash command
  PermissionResult checkBash(String command) => checkTool('Bash', command);

  /// Check Read operation
  PermissionResult checkRead(String path) => checkTool('Read', path);

  /// Check Write operation
  PermissionResult checkWrite(String path) => checkTool('Write', path);
}