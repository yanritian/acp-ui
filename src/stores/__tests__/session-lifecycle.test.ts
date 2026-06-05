import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSessionLifecycleStore } from '../session-lifecycle';

// Mock external dependencies
vi.mock('@/lib/host/storage', () => ({
  loadKvStore: vi.fn().mockResolvedValue({
    get: vi.fn().mockResolvedValue([]),
    set: vi.fn().mockResolvedValue(undefined),
  }),
}));

vi.mock('@/lib/host', () => ({
  getAppVersion: vi.fn().mockResolvedValue('0.1.0'),
  onAgentStderr: vi.fn(),
  spawnAgent: vi.fn(),
  killAgent: vi.fn(),
}));

vi.mock('@/lib/acp-bridge', () => ({
  AcpClientBridge: vi.fn(),
  createAcpClient: vi.fn(),
}));

describe('SessionLifecycleStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('state management', () => {
    it('should have correct initial state structure', () => {
      const store = useSessionLifecycleStore();

      expect(store.savedSessions).toBeDefined();
      expect(store.currentSession).toBeDefined();
      expect(store.isConnected).toBeDefined();
      expect(store.isLoading).toBeDefined();
      expect(store.isConnecting).toBeDefined();
      expect(store.isReconnecting).toBeDefined();
      expect(store.error).toBeDefined();
      expect(store.startupPhase).toBeDefined();
      expect(store.startupLogs).toBeDefined();
      expect(store.startupElapsed).toBeDefined();
    });

    it('should allow setting connection states', () => {
      const store = useSessionLifecycleStore();

      store.isConnected = true;
      expect(store.isConnected).toBe(true);

      store.isLoading = true;
      expect(store.isLoading).toBe(true);

      store.isConnecting = true;
      expect(store.isConnecting).toBe(true);

      store.isReconnecting = true;
      expect(store.isReconnecting).toBe(true);
    });

    it('should allow setting error state', () => {
      const store = useSessionLifecycleStore();

      store.error = 'Connection failed';
      expect(store.error).toBe('Connection failed');

      store.error = null;
      expect(store.error).toBeNull();
    });

    it('should track startup progress', () => {
      const store = useSessionLifecycleStore();

      store.startupPhase = 'loading-config';
      expect(store.startupPhase).toBe('loading-config');

      store.startupLogs = ['Log 1', 'Log 2'];
      expect(store.startupLogs.length).toBe(2);

      store.startupElapsed = 5000;
      expect(store.startupElapsed).toBe(5000);
    });
  });

  describe('clearError', () => {
    it('should clear error state', () => {
      const store = useSessionLifecycleStore();
      store.error = 'Some error';

      store.clearError();

      expect(store.error).toBeNull();
    });
  });

  describe('savedSessions manipulation', () => {
    it('should allow manual session list updates', () => {
      const store = useSessionLifecycleStore();

      const session = {
        id: 'test-id',
        agentName: 'Claude',
        sessionId: 'session-123',
        title: 'Test Session',
        lastUpdated: Date.now(),
        cwd: '/home/user',
      };

      store.savedSessions = [session];

      expect(store.savedSessions.length).toBe(1);
      expect(store.savedSessions[0].title).toBe('Test Session');
    });

    it('should allow updating session properties', () => {
      const store = useSessionLifecycleStore();

      store.savedSessions = [{
        id: 'test-id',
        agentName: 'Claude',
        sessionId: 'session-1',
        title: 'Original Title',
        lastUpdated: Date.now(),
        cwd: '/test',
      }];

      store.savedSessions[0].title = 'Updated Title';
      store.savedSessions[0].lastUpdated = Date.now();

      expect(store.savedSessions[0].title).toBe('Updated Title');
    });

    it('should support pinned sessions', () => {
      const store = useSessionLifecycleStore();

      store.savedSessions = [{
        id: 'pinned-id',
        agentName: 'Claude',
        sessionId: 'session-1',
        title: 'Pinned Session',
        lastUpdated: Date.now(),
        cwd: '/test',
        pinned: true,
      }];

      expect(store.savedSessions[0].pinned).toBe(true);
    });

    it('should support resumable sessions', () => {
      const store = useSessionLifecycleStore();

      store.savedSessions = [{
        id: 'resumable-id',
        agentName: 'Claude',
        sessionId: 'session-1',
        title: 'Resumable Session',
        lastUpdated: Date.now(),
        cwd: '/test',
        supportsLoadSession: true,
      }];

      expect(store.savedSessions[0].supportsLoadSession).toBe(true);
    });
  });

  describe('currentSession management', () => {
    it('should allow setting current session', () => {
      const store = useSessionLifecycleStore();

      const session = {
        id: 'current-id',
        agentName: 'Claude',
        sessionId: 'active-session',
        title: 'Active Session',
        lastUpdated: Date.now(),
        cwd: '/home/user',
      };

      store.currentSession = session;

      expect(store.currentSession?.title).toBe('Active Session');
      expect(store.currentSession?.sessionId).toBe('active-session');
    });

    it('should allow clearing current session', () => {
      const store = useSessionLifecycleStore();

      store.currentSession = {
        id: 'temp',
        agentName: 'Claude',
        sessionId: 'temp',
        title: 'Temp',
        lastUpdated: Date.now(),
        cwd: '/test',
      };

      store.currentSession = null;

      expect(store.currentSession).toBeNull();
    });
  });
});