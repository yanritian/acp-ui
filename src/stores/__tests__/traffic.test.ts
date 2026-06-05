import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useTrafficStore } from '../traffic';

describe('TrafficStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('initial state', () => {
    it('should start with empty entries', () => {
      const store = useTrafficStore();
      expect(store.entries).toEqual([]);
      expect(store.isPaused).toBe(false);
      expect(store.filter).toBe('all');
      expect(store.searchQuery).toBe('');
    });
  });

  describe('addEntry', () => {
    it('should add traffic entry with generated id and timestamp', () => {
      const store = useTrafficStore();
      const before = Date.now();

      store.addEntry({
        direction: 'out',
        type: 'request',
        method: 'prompt',
        payload: { text: 'Hello' },
      });

      const entry = store.entries[0];
      expect(entry.id).toBeDefined();
      expect(entry.timestamp).toBeGreaterThanOrEqual(before);
      expect(entry.direction).toBe('out');
      expect(entry.method).toBe('prompt');
    });

    it('should not add entry when paused', () => {
      const store = useTrafficStore();
      store.togglePause();

      store.addEntry({
        direction: 'in',
        type: 'response',
        method: 'prompt',
        payload: {},
      });

      expect(store.entries.length).toBe(0);
    });

    it('should extract error message from JSON-RPC error response', () => {
      const store = useTrafficStore();

      store.addEntry({
        direction: 'in',
        type: 'response',
        method: 'prompt',
        payload: {
          error: {
            message: 'Something went wrong',
            code: -1,
          },
        },
      });

      const entry = store.entries[0];
      expect(entry.error).toBe(true);
      expect(entry.errorMessage).toBe('Something went wrong');
    });

    it('should limit entries to MAX_ENTRIES', () => {
      const store = useTrafficStore();

      // Add more than MAX_ENTRIES (500)
      for (let i = 0; i < 550; i++) {
        store.addEntry({
          direction: 'out',
          type: 'request',
          method: 'test',
          payload: { index: i },
        });
      }

      expect(store.entries.length).toBeLessThanOrEqual(500);
      // Should keep most recent entries
      expect(store.entries[store.entries.length - 1].payload.index).toBe(549);
    });

    it('should add performance metrics', () => {
      const store = useTrafficStore();

      store.addEntry({
        direction: 'in',
        type: 'response',
        method: 'prompt',
        payload: {},
        ttft: 150,
        totalTokens: 500,
        responseTime: 2000,
      });

      const entry = store.entries[0];
      expect(entry.ttft).toBe(150);
      expect(entry.totalTokens).toBe(500);
      expect(entry.responseTime).toBe(2000);
    });
  });

  describe('filtering', () => {
    beforeEach(() => {
      const store = useTrafficStore();
      store.addEntry({ direction: 'out', type: 'request', method: 'prompt', payload: {} });
      store.addEntry({ direction: 'in', type: 'response', method: 'prompt', payload: {} });
      store.addEntry({ direction: 'in', type: 'notification', method: 'session/update', payload: {} });
      store.addEntry({ direction: 'out', type: 'request', method: 'cancel', payload: {} });
    });

    it('should filter by type: requests', () => {
      const store = useTrafficStore();
      store.setFilter('requests');

      expect(store.filteredEntries.length).toBe(2);
      expect(store.filteredEntries.every(e => e.type === 'request')).toBe(true);
    });

    it('should filter by type: responses', () => {
      const store = useTrafficStore();
      store.setFilter('responses');

      expect(store.filteredEntries.length).toBe(1);
      expect(store.filteredEntries[0].type).toBe('response');
    });

    it('should filter by type: notifications', () => {
      const store = useTrafficStore();
      store.setFilter('notifications');

      expect(store.filteredEntries.length).toBe(1);
      expect(store.filteredEntries[0].type).toBe('notification');
    });

    it('should filter by method', () => {
      const store = useTrafficStore();
      store.setMethodFilter('prompt');

      expect(store.filteredEntries.length).toBe(2);
      expect(store.filteredEntries.every(e => e.method === 'prompt')).toBe(true);
    });

    it('should filter by search query', () => {
      const store = useTrafficStore();
      store.addEntry({
        direction: 'out',
        type: 'request',
        method: 'specialMethod',
        payload: { uniqueKey: 'searchTarget' },
      });

      store.setSearch('searchTarget');

      expect(store.filteredEntries.length).toBe(1);
      expect(store.filteredEntries[0].method).toBe('specialMethod');
    });

    it('should combine multiple filters', () => {
      const store = useTrafficStore();
      store.setFilter('requests');
      store.setMethodFilter('prompt');

      expect(store.filteredEntries.length).toBe(1);
      expect(store.filteredEntries[0].type).toBe('request');
      expect(store.filteredEntries[0].method).toBe('prompt');
    });
  });

  describe('uniqueMethods', () => {
    it('should extract unique methods from entries', () => {
      const store = useTrafficStore();
      store.addEntry({ direction: 'out', type: 'request', method: 'prompt', payload: {} });
      store.addEntry({ direction: 'out', type: 'request', method: 'cancel', payload: {} });
      store.addEntry({ direction: 'in', type: 'response', method: 'prompt', payload: {} });

      expect(store.uniqueMethods).toContain('prompt');
      expect(store.uniqueMethods).toContain('cancel');
      expect(store.uniqueMethods.length).toBe(2);
    });

    it('should sort methods alphabetically', () => {
      const store = useTrafficStore();
      store.addEntry({ direction: 'out', type: 'request', method: 'zebra', payload: {} });
      store.addEntry({ direction: 'out', type: 'request', method: 'apple', payload: {} });

      expect(store.uniqueMethods).toEqual(['apple', 'zebra']);
    });
  });

  describe('controls', () => {
    it('should toggle pause state', () => {
      const store = useTrafficStore();
      expect(store.isPaused).toBe(false);

      store.togglePause();
      expect(store.isPaused).toBe(true);

      store.togglePause();
      expect(store.isPaused).toBe(false);
    });

    it('should clear all entries', () => {
      const store = useTrafficStore();
      store.addEntry({ direction: 'out', type: 'request', method: 'test', payload: {} });
      store.addEntry({ direction: 'in', type: 'response', method: 'test', payload: {} });

      store.clear();

      expect(store.entries.length).toBe(0);
    });

    it('should clear search query', () => {
      const store = useTrafficStore();
      store.setSearch('test query');

      store.clearSearch();

      expect(store.searchQuery).toBe('');
    });
  });
});