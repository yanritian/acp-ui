/**
 * Session Permissions Store
 *
 * Manages permission requests and authentication method selection.
 */
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { PermissionRequest } from '../lib/types';
import type { AuthMethod } from '@agentclientprotocol/sdk';

export const useSessionPermissionsStore = defineStore('sessionPermissions', () => {
  // State
  const pendingPermission = ref<PermissionRequest | null>(null);

  // Authentication state
  const pendingAuthMethods = ref<AuthMethod[]>([]);
  const pendingAuthAgentName = ref<string>('');

  // Internal resolver for auth method selection
  let authMethodResolver: ((methodId: string | null) => void) | null = null;

  // Permission resolution (requires acpClient reference)
  function resolvePermission(
    getAcpClient: () => { resolvePermission: (optionId: string) => void } | null
  ): (optionId: string) => void {
    return (optionId: string) => {
      const client = getAcpClient();
      if (client) {
        client.resolvePermission(optionId);
      }
    };
  }

  function cancelPermission(
    getAcpClient: () => { cancelPermission: () => void } | null
  ): () => void {
    return () => {
      const client = getAcpClient();
      if (client) {
        client.cancelPermission();
      }
    };
  }

  // Set pending permission (called by watch setup)
  function setPendingPermission(permission: PermissionRequest | null) {
    pendingPermission.value = permission;
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
    pendingPermission.value = null;
    pendingAuthMethods.value = [];
    pendingAuthAgentName.value = '';
    if (authMethodResolver) {
      authMethodResolver(null);
      authMethodResolver = null;
    }
  }

  return {
    // State
    pendingPermission,
    pendingAuthMethods,
    pendingAuthAgentName,

    // Actions
    setPendingPermission,
    resolvePermission,
    cancelPermission,
    promptForAuthMethod,
    selectAuthMethod,
    cancelAuthSelection,
    cancelPendingAuth,
    clearPendingState,
  };
});