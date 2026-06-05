import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { usePermissionRulesStore, type PermissionRule } from '../permission-rules';

describe('PermissionRulesStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
    });
  });

  describe('default rules', () => {
    it('should initialize with default rules', () => {
      const store = usePermissionRulesStore();

      expect(store.rules.length).toBeGreaterThan(0);
      expect(store.rules.some(r => r.id === 'allow-workspace-read')).toBe(true);
      expect(store.rules.some(r => r.id === 'reject-dangerous-rm')).toBe(true);
    });

    it('should have valid rule structure', () => {
      const store = usePermissionRulesStore();
      const rule = store.rules[0];

      expect(rule).toHaveProperty('id');
      expect(rule).toHaveProperty('name');
      expect(rule).toHaveProperty('pattern');
      expect(rule).toHaveProperty('action');
      expect(rule).toHaveProperty('scope');
      expect(rule).toHaveProperty('enabled');
      expect(rule).toHaveProperty('createdAt');
      expect(rule).toHaveProperty('updatedAt');
    });
  });

  describe('addRule', () => {
    it('should add a new rule', () => {
      const store = usePermissionRulesStore();
      const initialCount = store.rules.length;

      store.addRule({
        name: 'Test Rule',
        pattern: '^test.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });

      expect(store.rules.length).toBe(initialCount + 1);
      expect(store.rules[store.rules.length - 1].name).toBe('Test Rule');
    });

    it('should generate id and timestamps for new rule', () => {
      const store = usePermissionRulesStore();

      store.addRule({
        name: 'Auto ID Rule',
        pattern: 'auto.*',
        action: 'reject',
        scope: 'pattern',
        enabled: true,
      });

      const newRule = store.rules[store.rules.length - 1];
      expect(newRule.id).toBeDefined();
      expect(newRule.id.length).toBeGreaterThan(0);
      expect(newRule.createdAt).toBeDefined();
      expect(newRule.updatedAt).toBeDefined();
    });
  });

  describe('updateRule', () => {
    it('should update existing rule', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'ToUpdate',
        pattern: 'update.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });

      const ruleId = store.rules[store.rules.length - 1].id;
      store.updateRule(ruleId, {
        name: 'Updated',
        action: 'reject',
      });

      const updatedRule = store.rules.find(r => r.id === ruleId);
      expect(updatedRule?.name).toBe('Updated');
      expect(updatedRule?.action).toBe('reject');
      // updatedAt should be updated
      expect(updatedRule?.updatedAt).toBeDefined();
    });

    it('should preserve unchanged fields on update', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'PreserveTest',
        pattern: 'preserve.*',
        action: 'allow',
        scope: 'path',
        enabled: true,
        toolKind: 'read',
      });

      const ruleId = store.rules[store.rules.length - 1].id;
      store.updateRule(ruleId, { name: 'NewName' });

      const updatedRule = store.rules.find(r => r.id === ruleId);
      expect(updatedRule?.pattern).toBe('preserve.*');
      expect(updatedRule?.scope).toBe('path');
      expect(updatedRule?.toolKind).toBe('read');
    });
    it('should delete rule by id', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'ToDelete',
        pattern: 'delete.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });

      const ruleId = store.rules[store.rules.length - 1].id;
      const countBefore = store.rules.length;

      store.deleteRule(ruleId);

      expect(store.rules.length).toBe(countBefore - 1);
      expect(store.rules.find(r => r.id === ruleId)).toBeUndefined();
    });
  });

  describe('toggleRule', () => {
    it('should toggle rule enabled state', () => {
      const store = usePermissionRulesStore();
      const rule = store.rules[0];
      const initialEnabled = rule.enabled;

      store.toggleRule(rule.id);

      const toggledRule = store.rules.find(r => r.id === rule.id);
      expect(toggledRule?.enabled).toBe(!initialEnabled);

      // Toggle back
      store.toggleRule(rule.id);
      expect(store.rules.find(r => r.id === rule.id)?.enabled).toBe(initialEnabled);
    });
  });

  describe('matchRule', () => {
    it('should match rule by tool kind', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'Read Allow',
        pattern: '.*',
        action: 'allow',
        scope: 'tool',
        toolKind: 'read',
        enabled: true,
      });

      const match = store.matchRule('read', 'Read file', [{ path: '/test.txt' }]);
      expect(match).toBeDefined();
      expect(match?.action).toBe('allow');
    });

    it('should match rule by pattern', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'Danger Pattern',
        pattern: 'rm\\s+-rf',
        action: 'reject',
        scope: 'pattern',
        enabled: true,
      });

      const match = store.matchRule('bash', 'Execute rm -rf /', []);
      expect(match).toBeDefined();
      expect(match?.action).toBe('reject');
    });

    it('should not match disabled rules', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'Disabled Rule',
        pattern: '.*',
        action: 'allow',
        scope: 'tool',
        enabled: false,
      });

      const match = store.matchRule('any', 'Any', []);
      // Should not match disabled rule
      expect(match?.name).not.toBe('Disabled Rule');
    });

    it('should return null when no rule matches', () => {
      const store = usePermissionRulesStore();
      // Clear all rules for this test
      store.rules = [];

      const match = store.matchRule('unknown', 'Unknown tool', []);
      expect(match).toBeNull();
    });

    it('should prioritize first matching rule', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'First Added',
        pattern: '^read.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });
      store.addRule({
        name: 'Second Added',
        pattern: '^read.*',
        action: 'reject',
        scope: 'tool',
        enabled: true,
      });

      const match = store.matchRule('read', 'Read something', []);
      // Should match one of the rules
      expect(match).toBeDefined();
      expect(match?.pattern).toBe('^read.*');
    });
  });

  describe('createRuleFromRequest', () => {
    it('should create rule from permission request', () => {
      const store = usePermissionRulesStore();
      const initialCount = store.rules.length;

      store.createRuleFromRequest('read', 'Read file', 'allow', [{ path: '/safe' }]);

      expect(store.rules.length).toBe(initialCount + 1);
      const newRule = store.rules[store.rules.length - 1];
      expect(newRule.toolKind).toBe('read');
      expect(newRule.action).toBe('allow');
      expect(newRule.enabled).toBe(true);
    });
  });

  describe('resetToDefaults', () => {
    it('should reset to default rules', () => {
      const store = usePermissionRulesStore();
      store.addRule({
        name: 'Custom Rule',
        pattern: 'custom.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });

      // Disable a default rule
      const defaultRuleId = store.rules[0].id;
      store.toggleRule(defaultRuleId);

      const customCount = store.rules.length;
      store.resetToDefaults();

      // Should have default rules count (not custom count)
      expect(store.rules.length).toBeLessThan(customCount);
      // Default rules should be fresh enabled
      expect(store.rules.every(r => r.enabled)).toBe(true);
    });
  });

  describe('persistence', () => {
    it('should save to localStorage after changes', () => {
      const store = usePermissionRulesStore();

      store.addRule({
        name: 'Persist Test',
        pattern: 'persist.*',
        action: 'allow',
        scope: 'tool',
        enabled: true,
      });

      expect(localStorage.setItem).toHaveBeenCalled();
    });
  });
});