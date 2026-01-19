import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { DictionaryEntry } from '../types';

interface DictionaryStore {
  entries: DictionaryEntry[];
  isLoading: boolean;
  error: string | null;
  loadDictionary: () => Promise<void>;
  addEntry: (phrase: string, replacement: string) => Promise<DictionaryEntry>;
  updateEntry: (id: string, phrase: string, replacement: string, enabled: boolean) => Promise<void>;
  removeEntry: (id: string) => Promise<void>;
  toggleEntry: (id: string) => Promise<void>;
  refresh: () => Promise<void>;
}

export const useDictionaryStore = create<DictionaryStore>((set, get) => ({
  entries: [],
  isLoading: false,
  error: null,

  loadDictionary: async () => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const entries = await invoke<DictionaryEntry[]>('get_dictionary');
      set({ entries, isLoading: false });
    } catch (error) {
      console.error('Failed to load dictionary:', error);
      set({ isLoading: false, error: String(error) });
    }
  },

  addEntry: async (phrase: string, replacement: string) => {
    try {
      const entry = await invoke<DictionaryEntry>('add_dictionary_entry', {
        phrase,
        replacement,
      });
      set((state) => ({
        entries: [...state.entries, entry],
      }));
      return entry;
    } catch (error) {
      console.error('Failed to add dictionary entry:', error);
      throw error;
    }
  },

  updateEntry: async (id: string, phrase: string, replacement: string, enabled: boolean) => {
    try {
      await invoke('update_dictionary_entry', {
        id,
        phrase,
        replacement,
        enabled,
      });
      set((state) => ({
        entries: state.entries.map((e) =>
          e.id === id ? { ...e, phrase, replacement, enabled, synced: false } : e
        ),
      }));
    } catch (error) {
      console.error('Failed to update dictionary entry:', error);
      throw error;
    }
  },

  removeEntry: async (id: string) => {
    try {
      await invoke('remove_dictionary_entry', { id });
      set((state) => ({
        entries: state.entries.filter((e) => e.id !== id),
      }));
    } catch (error) {
      console.error('Failed to remove dictionary entry:', error);
      throw error;
    }
  },

  toggleEntry: async (id: string) => {
    const entry = get().entries.find((e) => e.id === id);
    if (!entry) return;
    await get().updateEntry(id, entry.phrase, entry.replacement, !entry.enabled);
  },

  refresh: async () => {
    await get().loadDictionary();
  },
}));
