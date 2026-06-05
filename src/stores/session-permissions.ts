/**
 * Session Permissions Store
 *
 * Manages permission requests and authentication method selection.
 * Supports batch permission handling and rule-based auto-approval.
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { PermissionRequest } from '../lib/types';
import type { AuthMethod } from '@agentclientprotocol/sdk';
import { usePermissionRulesStore } from './permission-rules';

// Resolver type for permission resolution
type PermissionResolver = (optionId: string) => void;

export const useSessionPermissionsStore = defineStore('sessionPermissions', () => {
  // State - now supports multiple pending requests
  const pendingPermissions = ref<PermissionRequest[]>([]);
  const permissionResolvers = ref<Map<string, PermissionResolver>>(new Map());

  // Authentication state
  const pendingAuthMethods = ref<AuthMethod[]>([]);
  const pendingAuthAgentName = ref<string>('');

  // Internal resolver for auth method selection
  let authMethodResolver: ((methodId: string | null) => void) | null = null;

  // Current single permission (for backward compatibility)
  const pendingPermission = computed(() => pendingPermissions.value[0] || null);

  // Has any pending permissions
  const hasPendingPermissions = computed(() => pendingPermissions.value.length > 0);

  // Number of pending permissions
  const pendingCount = computed(() => pendingPermissions.value.length);

  /**
   * Add a new permission request to the queue
   * Returns a promise that resolves when the permission is granted
   */
  function addPermissionRequest(permission: PermissionRequest): Promise<string> {
    return new Promise((resolve) => {
      // Check if auto-approval is possible
      const rulesStore = usePermissionRulesStore();
      const action = rulesStore.getAction(
        permission.toolCall.kind,
        permission.toolCall.title,
        permission.toolCall.locations
      );

      if (action === 'allow') {
        // Auto-approve - find allow_once option
        const allowOption = permission.options.find(o =>
          o.kind === 'allow_once' || o.kind === 'allow_always'
        );
        if (allowOption) {
          resolve(allowOption.optionId);
          return;
        }
      }

      if (action === 'reject') {
        // Auto-reject - find reject_once option
        const rejectOption = permission.options.find(o =>
          o.kind === 'reject_once' || o.kind === 'reject_always'
        );
        if (rejectOption) {
          resolve(rejectOption.optionId);
          return;
        }
      }

      // Add to pending queue
      pendingPermissions.value = [...pendingPermissions.value, permission];
      permissionResolvers.value = new Map(permissionResolvers.value).set(
        permission.sessionId,
        resolve
      );
    });
  }

  /**
   * Resolve a specific permission request
   */
  function resolvePermission(sessionId: string, optionId: string): void {
    const resolver = permissionResolvers.value.get(sessionId);
    if (resolver) {
      resolver(optionId);
      // Remove from pending queue
      removePermission(sessionId);
    }
  }

  /**
   * Resolve multiple permissions in batch
   */
  function resolveBatchPermissions(items: { sessionId: string; optionId: string }[]): void {
    for (const item of items) {
      resolvePermission(item.sessionId, item.optionId);
    }
  }

  /**
   * Cancel a specific permission request
   */
  function cancelPermission(sessionId: string): void {
    const resolver = permissionResolvers.value.get(sessionId);
    if (resolver) {
      // Find reject_once option to resolve with rejection
      const permission = pendingPermissions.value.find(p => p.sessionId === sessionId);
      if (permission) {
        const rejectOption = permission.options.find(o => o.kind === 'reject_once');
        if (rejectOption) {
          resolver(rejectOption.optionId);
        }
      }
      removePermission(sessionId);
    }
  }

  /**
   * Cancel all pending permissions
   */
  function cancelAllPermissions(): void {
    for (const sessionId of permissionResolvers.value.keys()) {
      cancelPermission(sessionId);
    }
    pendingPermissions.value = [];
    permissionResolvers.value = new Map();
  }

  /**
   * Remove a permission from the pending queue
   */
  function removePermission(sessionId: string): void {
    pendingPermissions.value = pendingPermissions.value.filter(
      p => p.sessionId !== sessionId
    );
    const newResolvers = new Map(permissionResolvers.value);
    newResolvers.delete(sessionId);
    permissionResolvers.value = newResolvers;
  }

  /**
   * Set pending permission (legacy method for backward compatibility)
   */
  function setPendingPermission(permission: PermissionRequest | null) {
    if (permission) {
      // Add to queue if not already present
      if (!pendingPermissions.value.find(p => p.sessionId === permission.sessionId)) {
        pendingPermissions.value = [...pendingPermissions.value, permission];
      }
    } else {
      // Clear all if null
      pendingPermissions.value = [];
      permissionResolvers.value = new Map();
    }
  }

  /**
   * Legacy resolve method (requires acpClient reference)
   */
  function createResolvePermission(
    getAcpClient: () => { resolvePermission: (optionId: string) => void } | null
  ): (optionId: string) => void {
    return (optionId: string) => {
      const client = getAcpClient();
      if (client) {
        client.resolvePermission(optionId);
      }
      // Also resolve in store
      if (pendingPermissions.value.length > 0) {
        resolvePermission(pendingPermissions.value[0].sessionId, optionId);
      }
    };
  }

  /**
   * Legacy cancel method (requires acpClient reference)
   */
  function createCancelPermission(
    getAcpClient: () => { cancelPermission: () => void } | null
  ): () => void {
    return () => {
      const client = getAcpClient();
      if (client) {
        client.cancelPermission();
      }
      // Also cancel in store
      cancelAllPermissions();
    };
  }

  // Prompt user to select auth method
  async function promptForAuthMethod(authMethods: AuthMethod[], agentName: string): Promise<string | null> {
    return new Promise((resolve) => {
      pendingAuthMethods.value = authMethods;
      pendingAuthAgentName.value = agentName;
      authMethodResolver = resolve;
    });
  }

  // User selected an auth method
  function selectAuthMethod(methodId: string): void {
    if (authMethodResolver) {
      authMethodResolver(methodId);
      authMethodResolver = null;
      pendingAuthMethods.value = [];
      pendingAuthAgentName.value = '';
    }
  }

  // User cancelled auth selection
  function cancelAuthSelection(): void {
    if (authMethodResolver) {
      authMethodResolver(null);
      authMethodResolver = null;
      pendingAuthMethods.value = [];
      pendingAuthAgentName.value = '';
    }
  }

  // Cancel auth during connection
  function cancelPendingAuth(): void {
    if (authMethodResolver) {
      authMethodResolver(null);
      authMethodResolver = null;
    }
    pendingAuthMethods.value = [];
    pendingAuthAgentName.value = '';
  }

  // Clear all pending state
  function clearPendingState(): void {
    pendingPermissions.value = [];
    permissionResolvers.value = new Map();
    pendingAuthMethods.value = [];
    pendingAuthAgentName.value = '';
    if (authMethodResolver) {
      authMethodResolver(null);
      authMethodResolver = null;
    }
  }

  return {
    // State
    pendingPermission, // Legacy single permission
    pendingPermissions, // New multiple permissions
    hasPendingPermissions,
    pendingCount,
    pendingAuthMethods,
    pendingAuthAgentName,

    // Actions
    addPermissionRequest,
    resolvePermission,
    resolveBatchPermissions,
    cancelPermission,
    cancelAllPermissions,
    removePermission,
    setPendingPermission,
    createResolvePermission,
    createCancelPermission,
    promptForAuthMethod,
    selectAuthMethod,
    cancelAuthSelection,
    cancelPendingAuth,
    clearPendingState,
  };
});