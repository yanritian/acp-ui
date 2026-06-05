/**
 * Permission Rules Store
 *
 * Manages permission rule templates for automatic approval/rejection.
 * Rules are persisted to localStorage for web builds.
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

// Rule action types
export type PermissionAction = 'allow' | 'reject';

// Rule scope types
export type PermissionScope = 'tool' | 'path' | 'pattern';

// Permission rule interface
export interface PermissionRule {
  id: string;
  name: string;
  description?: string;
  pattern: string; // Regex pattern or glob pattern
  action: PermissionAction;
  scope: PermissionScope;
  toolKind?: string; // Specific tool kind (e.g., 'read', 'write', 'bash')
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

// Storage key for web builds
const WEB_PERMISSION_RULES_KEY = 'acp-ui-permission-rules';

// Default preset rules
const DEFAULT_RULES: PermissionRule[] = [
  {
    id: 'allow-workspace-read',
    name: '允许读取工作区文件',
    description: '始终允许读取工作区内的文件',
    pattern: '^read.*',
    action: 'allow',
    scope: 'tool',
    toolKind: 'read',
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  {
    id: 'reject-dangerous-rm',
    name: '拒绝危险删除命令',
    description: '始终拒绝 rm -rf 等危险删除命令',
    pattern: 'rm\\s+-rf',
    action: 'reject',
    scope: 'pattern',
    toolKind: 'bash',
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  {
    id: 'reject-force-push',
    name: '拒绝强制推送',
    description: '始终拒绝 git push --force 等强制操作',
    pattern: '(push\\s+--force|push\\s+-f)',
    action: 'reject',
    scope: 'pattern',
    toolKind: 'bash',
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
];

// Load rules from localStorage
function loadRulesFromStorage(): PermissionRule[] {
  if (typeof localStorage === 'undefined') return DEFAULT_RULES;
  const raw = localStorage.getItem(WEB_PERMISSION_RULES_KEY);
  if (!raw) return DEFAULT_RULES;
  try {
    const parsed = JSON.parse(raw) as PermissionRule[];
    // Merge with default rules (keep user modifications)
    const defaultIds = DEFAULT_RULES.map(r => r.id);
    const userRules = parsed.filter(r => !defaultIds.includes(r.id));
    const mergedDefaults = DEFAULT_RULES.map(defaultRule => {
      const existing = parsed.find(r => r.id === defaultRule.id);
      return existing ? { ...defaultRule, enabled: existing.enabled } : defaultRule;
    });
    return [...mergedDefaults, ...userRules];
  } catch {
    return DEFAULT_RULES;
  }
}

// Save rules to localStorage
function saveRulesToStorage(rules: PermissionRule[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(WEB_PERMISSION_RULES_KEY, JSON.stringify(rules));
  } catch {
    // Storage might be full or disabled
  }
}

export const usePermissionRulesStore = defineStore('permissionRules', () => {
  // State
  const rules = ref<PermissionRule[]>(loadRulesFromStorage());

  // Computed: enabled rules only
  const enabledRules = computed(() =>
    rules.value.filter(r => r.enabled)
  );

  // Computed: rules by scope
  const toolRules = computed(() =>
    enabledRules.value.filter(r => r.scope === 'tool')
  );

  const pathRules = computed(() =>
    enabledRules.value.filter(r => r.scope === 'path')
  );

  const patternRules = computed(() =>
    enabledRules.value.filter(r => r.scope === 'pattern')
  );

  /**
   * Check if a permission request matches any rule
   * Returns the matching rule or null
   */
  function matchRule(
    toolKind: string,
    toolTitle: string,
    locations?: { path: string }[]
  ): PermissionRule | null {
    // Check tool-kind rules first
    for (const rule of toolRules.value) {
      if (rule.toolKind && rule.toolKind === toolKind) {
        return rule;
      }
      // Also match by pattern on tool kind
      try {
        const regex = new RegExp(rule.pattern, 'i');
        if (regex.test(toolKind)) {
          return rule;
        }
      } catch {
        // Invalid regex pattern, skip
      }
    }

    // Check pattern rules (match against tool title or command)
    for (const rule of patternRules.value) {
      if (rule.toolKind && rule.toolKind !== toolKind) {
        continue; // Skip if rule is for different tool kind
      }
      try {
        const regex = new RegExp(rule.pattern, 'i');
        if (regex.test(toolTitle)) {
          return rule;
        }
      } catch {
        // Invalid regex pattern, skip
      }
    }

    // Check path rules (match against locations)
    if (locations && locations.length > 0) {
      for (const rule of pathRules.value) {
        if (rule.toolKind && rule.toolKind !== toolKind) {
          continue;
        }
        try {
          const regex = new RegExp(rule.pattern, 'i');
          for (const loc of locations) {
            if (regex.test(loc.path)) {
              return rule;
            }
          }
        } catch {
          // Invalid regex pattern, skip
        }
      }
    }

    return null;
  }

  /**
   * Get action for a permission request based on rules
   * Returns 'allow', 'reject', or null if no rule matches
   */
  function getAction(
    toolKind: string,
    toolTitle: string,
    locations?: { path: string }[]
  ): PermissionAction | null {
    const rule = matchRule(toolKind, toolTitle, locations);
    return rule ? rule.action : null;
  }

  /**
   * Add a new rule
   */
  function addRule(rule: Omit<PermissionRule, 'id' | 'createdAt' | 'updatedAt'>): PermissionRule {
    const newRule: PermissionRule = {
      ...rule,
      id: `rule-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    rules.value = [...rules.value, newRule];
    saveRulesToStorage(rules.value);
    return newRule;
  }

  /**
   * Update an existing rule
   */
  function updateRule(id: string, updates: Partial<PermissionRule>): PermissionRule | null {
    const index = rules.value.findIndex(r => r.id === id);
    if (index === -1) return null;

    const updatedRule: PermissionRule = {
      ...rules.value[index],
      ...updates,
      updatedAt: Date.now(),
    };

    // Immutable update
    rules.value = [
      ...rules.value.slice(0, index),
      updatedRule,
      ...rules.value.slice(index + 1),
    ];

    saveRulesToStorage(rules.value);
    return updatedRule;
  }

  /**
   * Delete a rule
   */
  function deleteRule(id: string): boolean {
    const index = rules.value.findIndex(r => r.id === id);
    if (index === -1) return false;

    // Immutable delete
    rules.value = [
      ...rules.value.slice(0, index),
      ...rules.value.slice(index + 1),
    ];

    saveRulesToStorage(rules.value);
    return true;
  }

  /**
   * Toggle rule enabled state
   */
  function toggleRule(id: string): boolean {
    const rule = rules.value.find(r => r.id === id);
    if (!rule) return false;
    return updateRule(id, { enabled: !rule.enabled }) !== null;
  }

  /**
   * Reset to default rules
   */
  function resetToDefaults(): void {
    rules.value = DEFAULT_RULES.map(r => ({
      ...r,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }));
    saveRulesToStorage(rules.value);
  }

  /**
   * Create rule from permission request (for "always allow/reject" actions)
   */
  function createRuleFromRequest(
    toolKind: string,
    toolTitle: string,
    action: PermissionAction,
    locations?: { path: string }[]
  ): PermissionRule | null {
    // Determine the best scope and pattern
    let scope: PermissionScope = 'tool';
    let pattern: string = '';
    let ruleName: string = '';

    if (locations && locations.length > 0) {
      // Create path-based rule
      scope = 'path';
      // Use the first location's directory as pattern
      const firstPath = locations[0].path;
      const dirPath = firstPath.split('/').slice(0, -1).join('/') || firstPath;
      pattern = dirPath.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); // Escape regex
      ruleName = `${action === 'allow' ? '允许' : '拒绝'} ${toolKind} (${dirPath})`;
    } else {
      // Create tool-based rule
      scope = 'tool';
      pattern = `^${toolKind}$`;
      ruleName = `${action === 'allow' ? '始终允许' : '始终拒绝'} ${toolKind}`;
    }

    return addRule({
      name: ruleName,
      description: `从权限对话框创建的规则`,
      pattern,
      action,
      scope,
      toolKind,
      enabled: true,
    });
  }

  return {
    // State
    rules,
    enabledRules,
    toolRules,
    pathRules,
    patternRules,

    // Actions
    matchRule,
    getAction,
    addRule,
    updateRule,
    deleteRule,
    toggleRule,
    resetToDefaults,
    createRuleFromRequest,
  };
});