import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSessionCapabilitiesStore } from '../session-capabilities';

describe('SessionCapabilitiesStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('initial state', () => {
    it('should start with empty capabilities', () => {
      const store = useSessionCapabilitiesStore();
      expect(store.availableModes).toEqual([]);
      expect(store.currentModeId).toBe('');
      expect(store.availableCommands).toEqual([]);
      expect(store.availableModels).toEqual([]);
      expect(store.currentModelId).toBe('');
    });
  });

  describe('modes management', () => {
    it('should set available modes', () => {
      const store = useSessionCapabilitiesStore();
      store.setModes({
        availableModes: [
          { id: 'mode-1', name: 'Code Mode' },
          { id: 'mode-2', name: 'Chat Mode' },
        ],
        currentModeId: 'mode-1',
      });

      expect(store.availableModes.length).toBe(2);
      expect(store.currentModeId).toBe('mode-1');
    });

    it('should update current mode', () => {
      const store = useSessionCapabilitiesStore();
      store.setModes({
        availableModes: [
          { id: 'mode-1', name: 'Code' },
          { id: 'mode-2', name: 'Chat' },
        ],
        currentModeId: 'mode-1',
      });

      store.updateCurrentMode('mode-2');

      expect(store.currentModeId).toBe('mode-2');
    });

    it('should clear modes when undefined', () => {
      const store = useSessionCapabilitiesStore();
      store.setModes({
        availableModes: [{ id: 'mode-1', name: 'Test' }],
        currentModeId: 'mode-1',
      });

      store.setModes(undefined);

      expect(store.availableModes).toEqual([]);
      expect(store.currentModeId).toBe('');
    });
  });

  describe('commands management', () => {
    it('should set available commands', () => {
      const store = useSessionCapabilitiesStore();

      store.setCommands([
        { name: '/help', description: 'Show help', input: { hint: 'Optional topic' } },
        { name: '/clear', description: 'Clear chat' },
      ]);

      expect(store.availableCommands.length).toBe(2);
    });

    it('should handle empty commands', () => {
      const store = useSessionCapabilitiesStore();
      store.setCommands([
        { name: '/test', description: 'Test command' },
      ]);

      store.setCommands([]);

      expect(store.availableCommands).toEqual([]);
    });
  });

  describe('models management', () => {
    it('should set available models', () => {
      const store = useSessionCapabilitiesStore();
      store.setModels({
        availableModels: [
          { modelId: 'claude-3', name: 'Claude 3' },
          { modelId: 'claude-2', name: 'Claude 2' },
        ],
        currentModelId: 'claude-3',
      });

      expect(store.availableModels.length).toBe(2);
      expect(store.currentModelId).toBe('claude-3');
    });

    it('should clear models when undefined', () => {
      const store = useSessionCapabilitiesStore();
      store.setModels({
        availableModels: [{ modelId: 'model-1', name: 'Model 1' }],
        currentModelId: 'model-1',
      });

      store.setModels(undefined);

      expect(store.availableModels).toEqual([]);
      expect(store.currentModelId).toBe('');
    });
  });

  describe('clearCapabilities', () => {
    it('should clear all capabilities', () => {
      const store = useSessionCapabilitiesStore();
      store.setModes({
        availableModes: [{ id: 'm1', name: 'Mode' }],
        currentModeId: 'm1',
      });
      store.setCommands([{ name: '/cmd', description: 'Command' }]);
      store.setModels({
        availableModels: [{ modelId: 'model', name: 'Model' }],
        currentModelId: 'model',
      });

      store.clearCapabilities();

      expect(store.availableModes).toEqual([]);
      expect(store.currentModeId).toBe('');
      expect(store.availableCommands).toEqual([]);
      expect(store.availableModels).toEqual([]);
      expect(store.currentModelId).toBe('');
    });
  });

  describe('edge cases', () => {
    it('should handle modes without currentModeId', () => {
      const store = useSessionCapabilitiesStore();

      store.setModes({
        availableModes: [{ id: 'mode-1', name: 'Test' }],
      });

      expect(store.availableModes.length).toBe(1);
      expect(store.currentModeId).toBe('');
    });

    it('should handle commands without input hint', () => {
      const store = useSessionCapabilitiesStore();

      store.setCommands([
        { name: '/simple', description: 'Simple command' },
      ]);

      expect(store.availableCommands[0].name).toBe('/simple');
    });
  });
});