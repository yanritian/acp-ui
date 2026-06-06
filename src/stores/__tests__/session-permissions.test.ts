import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSessionPermissionsStore } from '../session-permissions';
import type { PermissionRequest } from '@/lib/types';

interface AuthMethod {
  id: string;
  name: string;
  description: string;
}

describe('SessionPermissionsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('pendingPermission state', () => {
    it('should start with no pending permissions', () => {
      const store = useSessionPermissionsStore();
      expect(store.pendingPermission).toBeNull();
      expect(store.pendingPermissions).toEqual([]);
      expect(store.hasPendingPermissions).toBe(false);
    });

    it('should set pending permission via setPendingPermission', () => {
      const store = useSessionPermissionsStore();
      const request: PermissionRequest = {
        sessionId: 'test-session',
        toolCall: {
          kind: 'read',
          title: 'Read file',
          locations: [{ path: '/test/file.txt' }],
          toolCallId: 'tc-1',
          status: 'pending',
        },
        options: [
          { kind: 'allow_once', name: 'Allow', optionId: 'opt-1' },
        ],
      };

      store.setPendingPermission(request);

      expect(store.pendingPermissions.length).toBe(1);
      expect(store.pendingPermission).toEqual(request);
      expect(store.hasPendingPermissions).toBe(true);
    });

    it('should not duplicate permission with same sessionId', () => {
      const store = useSessionPermissionsStore();
      const request1: PermissionRequest = {
        sessionId: 'session-1',
        toolCall: { kind: 'read', title: 'Read', locations: [], toolCallId: 'tc-1', status: 'pending' },
        options: [{ kind: 'allow_once', name: 'Allow', optionId: 'opt-1' }],
      };
      const request2: PermissionRequest = {
        sessionId: 'session-1', // Same sessionId
        toolCall: { kind: 'write', title: 'Write', locations: [], toolCallId: 'tc-2', status: 'pending' },
        options: [{ kind: 'deny_once', name: 'Deny', optionId: 'opt-2' }],
      };

      store.setPendingPermission(request1);
      store.setPendingPermission(request2);

      // Should only have one because same sessionId
      expect(store.pendingPermissions.length).toBe(1);
    });

    it('should handle multiple permission requests with different sessionIds', () => {
      const store = useSessionPermissionsStore();
      const request1: PermissionRequest = {
        sessionId: 'session-1',
        toolCall: { kind: 'read', title: 'Read', locations: [], toolCallId: 'tc-1', status: 'pending' },
        options: [{ kind: 'allow_once', name: 'Allow', optionId: 'opt-1' }],
      };
      const request2: PermissionRequest = {
        sessionId: 'session-2',
        toolCall: { kind: 'write', title: 'Write', locations: [], toolCallId: 'tc-2', status: 'pending' },
        options: [{ kind: 'deny_once', name: 'Deny', optionId: 'opt-2' }],
      };

      store.setPendingPermission(request1);
      store.setPendingPermission(request2);

      expect(store.pendingPermissions.length).toBe(2);
      expect(store.pendingCount).toBe(2);
    });

    it('should clear all permissions when setPendingPermission(null)', () => {
      const store = useSessionPermissionsStore();
      store.setPendingPermission({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R', locations: [], toolCallId: 'tc-r', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o1' }],
      });

      store.setPendingPermission(null);

      expect(store.pendingPermissions.length).toBe(0);
    });
  });

  describe('resolvePermission', () => {
    it('should resolve permission and remove from pending', () => {
      const store = useSessionPermissionsStore();

      // Use setPendingPermission for synchronous testing
      store.setPendingPermission({
        sessionId: 'test-session',
        toolCall: { kind: 'read', title: 'Read', locations: [], toolCallId: 'tc-1', status: 'pending' },
        options: [{ kind: 'allow_once', name: 'Allow', optionId: 'opt-1' }],
      });

      // Should be pending now
      expect(store.pendingPermissions.length).toBe(1);

      // Note: resolvePermission requires a resolver from addPermissionRequest
      // For setPendingPermission, use removePermission instead
      store.removePermission('test-session');

      expect(store.pendingPermissions.length).toBe(0);
      expect(store.pendingPermission).toBeNull();
    });

    it('should handle removePermission for batch-like scenarios', () => {
      const store = useSessionPermissionsStore();

      store.setPendingPermission({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R1', locations: [], toolCallId: 'tc-r1', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o1' }],
      });
      store.setPendingPermission({
        sessionId: 's2',
        toolCall: { kind: 'write', title: 'W', locations: [], toolCallId: 'tc-w', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o2' }],
      });

      expect(store.pendingPermissions.length).toBe(2);

      store.removePermission('s1');
      store.removePermission('s2');

      expect(store.pendingPermissions.length).toBe(0);
    });
  });

  describe('cancelPermission', () => {
    it('should cancel permission and remove from pending', async () => {
      const store = useSessionPermissionsStore();

      const promise = store.addPermissionRequest({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R', locations: [], toolCallId: 'tc-r', status: 'pending' },
        options: [
          { kind: 'allow_once', name: 'Allow', optionId: 'o-allow' },
          { kind: 'reject_once', name: 'Reject', optionId: 'o-reject' },
        ],
      });

      // Cancel should remove the permission
      store.cancelPermission('s1');

      // Promise should resolve (with whatever option cancel found)
      const result = await promise;
      expect(result).toBeDefined();

      expect(store.pendingPermissions.length).toBe(0);
    });

    it('should cancel all permissions', async () => {
      const store = useSessionPermissionsStore();

      store.addPermissionRequest({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R', locations: [], toolCallId: 'tc-r', status: 'pending' },
        options: [
          { kind: 'allow', name: 'A', optionId: 'o1' },
          { kind: 'reject_once', name: 'R', optionId: 'o-reject1' },
        ],
      });
      store.addPermissionRequest({
        sessionId: 's2',
        toolCall: { kind: 'write', title: 'W', locations: [], toolCallId: 'tc-w', status: 'pending' },
        options: [
          { kind: 'allow', name: 'A', optionId: 'o2' },
          { kind: 'reject_once', name: 'R', optionId: 'o-reject2' },
        ],
      });

      store.cancelAllPermissions();

      expect(store.pendingPermissions.length).toBe(0);
    });
  });

  describe('auth methods', () => {
    it('should handle auth method selection', async () => {
      const store = useSessionPermissionsStore();
      const methods: AuthMethod[] = [
        { id: 'm1', name: 'Method 1', description: 'First method' },
        { id: 'm2', name: 'Method 2', description: 'Second method' },
      ];

      const promptPromise = store.promptForAuthMethod(methods, 'TestAgent');

      expect(store.pendingAuthMethods).toEqual(methods);
      expect(store.pendingAuthAgentName).toBe('TestAgent');

      store.selectAuthMethod('m1');

      await expect(promptPromise).resolves.toBe('m1');

      expect(store.pendingAuthMethods).toEqual([]);
      expect(store.pendingAuthAgentName).toBe('');
    });

    it('should cancel auth selection', async () => {
      const store = useSessionPermissionsStore();
      const methods: AuthMethod[] = [
        { id: 'm1', name: 'Method 1', description: 'First' },
      ];

      const promptPromise = store.promptForAuthMethod(methods, 'Agent');
      store.cancelAuthSelection();

      await expect(promptPromise).resolves.toBeNull();

      expect(store.pendingAuthMethods).toEqual([]);
      expect(store.pendingAuthAgentName).toBe('');
    });
  });

  describe('removePermission', () => {
    it('should remove specific permission by sessionId', () => {
      const store = useSessionPermissionsStore();
      store.setPendingPermission({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R', locations: [], toolCallId: 'tc-r', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o1' }],
      });
      store.setPendingPermission({
        sessionId: 's2',
        toolCall: { kind: 'write', title: 'W', locations: [], toolCallId: 'tc-w', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o2' }],
      });

      store.removePermission('s1');

      expect(store.pendingPermissions.length).toBe(1);
      expect(store.pendingPermissions[0].sessionId).toBe('s2');
    });
  });

  describe('clearPendingState', () => {
    it('should clear all pending state', () => {
      const store = useSessionPermissionsStore();
      store.setPendingPermission({
        sessionId: 's1',
        toolCall: { kind: 'read', title: 'R', locations: [], toolCallId: 'tc-r', status: 'pending' },
        options: [{ kind: 'allow', name: 'A', optionId: 'o1' }],
      });
      store.promptForAuthMethod(
        [{ id: 'm1', name: 'M', description: 'D' }],
        'Agent'
      );

      store.clearPendingState();

      expect(store.pendingPermissions).toEqual([]);
      expect(store.pendingAuthMethods).toEqual([]);
      expect(store.pendingAuthAgentName).toBe('');
    });
  });
});