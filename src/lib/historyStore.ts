import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { TranscriptionEntry } from '../types';

interface HistoryStore {
  entries: TranscriptionEntry[];
  totalCount: number;
  isLoading: boolean;
  hasMore: boolean;
  error: string | null;
  loadHistory: (reset?: boolean) => Promise<void>;
  loadMore: () => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  clearAll: () => Promise<void>;
  refresh: () => Promise<void>;
}

const PAGE_SIZE = 50;

export const useHistoryStore = create<HistoryStore>((set, get) => ({
  entries: [],
  totalCount: 0,
  isLoading: false,
  hasMore: true,
  error: null,

  loadHistory: async (reset = true) => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const entries = await invoke<TranscriptionEntry[]>('get_history', {
        limit: PAGE_SIZE,
        offset: 0,
      });
      const totalCount = await invoke<number>('get_history_count');
      set({
        entries: reset ? entries : [...get().entries, ...entries],
        totalCount,
        isLoading: false,
        hasMore: entries.length >= PAGE_SIZE,
      });
    } catch (error) {
      console.error('Failed to load history:', error);
      set({ isLoading: false, error: String(error) });
    }
  },

  loadMore: async () => {
    const { entries, isLoading, hasMore } = get();
    if (isLoading || !hasMore) return;

    set({ isLoading: true });
    try {
      const newEntries = await invoke<TranscriptionEntry[]>('get_history', {
        limit: PAGE_SIZE,
        offset: entries.length,
      });
      set({
        entries: [...entries, ...newEntries],
        isLoading: false,
        hasMore: newEntries.length >= PAGE_SIZE,
      });
    } catch (error) {
      console.error('Failed to load more history:', error);
      set({ isLoading: false, error: String(error) });
    }
  },

  deleteEntry: async (id: string) => {
    try {
      await invoke('delete_history_entry', { id });
      set((state) => ({
        entries: state.entries.filter((e) => e.id !== id),
        totalCount: state.totalCount - 1,
      }));
    } catch (error) {
      console.error('Failed to delete entry:', error);
      throw error;
    }
  },

  clearAll: async () => {
    try {
      await invoke('clear_history');
      set({ entries: [], totalCount: 0, hasMore: false });
    } catch (error) {
      console.error('Failed to clear history:', error);
      throw error;
    }
  },

  refresh: async () => {
    await get().loadHistory(true);
  },
}));
