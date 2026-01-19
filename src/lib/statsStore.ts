import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { LocalStats } from '../types';

interface StatsStore {
  stats: LocalStats | null;
  isLoading: boolean;
  error: string | null;
  loadStats: () => Promise<void>;
  refresh: () => Promise<void>;
}

const defaultStats: LocalStats = {
  total_transcriptions: 0,
  total_words: 0,
  total_audio_seconds: 0,
  streak_days: 0,
  last_used_date: null,
  daily_history: [],
};

export const useStatsStore = create<StatsStore>((set, get) => ({
  stats: null,
  isLoading: false,
  error: null,

  loadStats: async () => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const stats = await invoke<LocalStats>('get_stats');
      set({ stats, isLoading: false });
    } catch (error) {
      console.error('Failed to load stats:', error);
      set({
        stats: defaultStats,
        isLoading: false,
        error: String(error)
      });
    }
  },

  refresh: async () => {
    try {
      const stats = await invoke<LocalStats>('get_stats');
      set({ stats });
    } catch (error) {
      console.error('Failed to refresh stats:', error);
    }
  },
}));
