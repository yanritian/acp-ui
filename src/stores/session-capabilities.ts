/**
 * Session Capabilities Store
 *
 * Manages available modes, commands, and models for the current session.
 */
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { SessionMode, SlashCommand, ModelInfo } from '../lib/types';
import type { SessionModeState, SessionModelState } from '@agentclientprotocol/sdk';

export const useSessionCapabilitiesStore = defineStore('sessionCapabilities', () => {
  // State
  const availableModes = ref<SessionMode[]>([]);
  const currentModeId = ref<string>('');

  // Slash commands
  const availableCommands = ref<SlashCommand[]>([]);

  // Session models
  const availableModels = ref<ModelInfo[]>([]);
  const currentModelId = ref<string>('');

  // Set capabilities from session response using SDK types
  function setModes(modes: SessionModeState | undefined) {
    if (modes) {
      availableModes.value = (modes.availableModes || []).map(m => ({
        id: m.id,
        name: m.name,
        description: m.description ?? undefined,
      }));
      currentModeId.value = modes.currentModeId || '';
    } else {
      availableModes.value = [];
      currentModeId.value = '';
    }
  }

  function setModels(models: SessionModelState | undefined) {
    if (models) {
      availableModels.value = (models.availableModels || []).map(m => ({
        modelId: m.modelId,
        name: m.name,
        description: m.description ?? undefined,
      }));
      currentModelId.value = models.currentModelId || '';
    } else {
      availableModels.value = [];
      currentModelId.value = '';
    }
  }

  function setCommands(commands: Array<{ name: string; description: string; input?: { hint?: string } }>) {
    availableCommands.value = commands.map((cmd) => ({
      name: cmd.name,
      description: cmd.description,
      hint: cmd.input?.hint ?? undefined,
    }));
  }

  // Update current mode from notification
  function updateCurrentMode(modeId: string) {
    currentModeId.value = modeId;
  }

  // Clear all capabilities
  function clearCapabilities() {
    availableModes.value = [];
    currentModeId.value = '';
    availableCommands.value = [];
    availableModels.value = [];
    currentModelId.value = '';
  }

  // Set session mode (requires acpClient reference)
  async function setMode(
    modeId: string,
    getSessionId: () => string | null,
    getAcpClient: () => { setMode: (args: { sessionId: string; modeId: string }) => Promise<void> } | null
  ): Promise<void> {
    const sessionId = getSessionId();
    const client = getAcpClient();

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.setMode({
      sessionId,
      modeId,
    });

    // Optimistically update the current mode
    currentModeId.value = modeId;
  }

  // Set session model (requires acpClient reference)
  async function setModel(
    modelId: string,
    getSessionId: () => string | null,
    getAcpClient: () => { unstable_setSessionModel: (args: { sessionId: string; modelId: string }) => Promise<void> } | null
  ): Promise<void> {
    const sessionId = getSessionId();
    const client = getAcpClient();

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.unstable_setSessionModel({
      sessionId,
      modelId,
    });

    // Optimistically update the current model
    currentModelId.value = modelId;
  }

  return {
    // State
    availableModes,
    currentModeId,
    availableCommands,
    availableModels,
    currentModelId,

    // Actions
    setModes,
    setModels,
    setCommands,
    updateCurrentMode,
    clearCapabilities,
    setMode,
    setModel,
  };
});