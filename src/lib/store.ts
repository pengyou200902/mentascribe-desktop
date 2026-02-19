import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface TranscriptionSettings {
  provider?: string;
  language?: string;
  model_size?: string;
  cloud_provider?: string;
}

export interface CleanupSettings {
  enabled: boolean;
  provider?: string;
  model?: string;
  custom_endpoint?: string;
  api_key?: string;
  remove_filler: boolean;
  add_punctuation: boolean;
  format_paragraphs: boolean;
}

export interface HotkeySettings {
  key?: string;
  mode?: string;
}

export interface OutputSettings {
  insert_method?: string;
  auto_capitalize?: boolean;
}

export interface WidgetSettings {
  draggable: boolean;
  opacity: number; // 0.2 to 1.0
}

export interface UserSettings {
  transcription: TranscriptionSettings;
  cleanup: CleanupSettings;
  hotkey: HotkeySettings;
  output: OutputSettings;
  widget: WidgetSettings;
}

interface Store {
  settings: UserSettings | null;
  isLoading: boolean;
  loadSettings: () => Promise<void>;
  updateSettings: (settings: UserSettings) => Promise<void>;
}

export const useStore = create<Store>((set) => ({
  settings: null,
  isLoading: false,

  loadSettings: async () => {
    set({ isLoading: true });
    try {
      const settings = await invoke<UserSettings>('get_settings');
      set({ settings, isLoading: false });
    } catch (error) {
      console.error('Failed to load settings:', error);
      set({ isLoading: false });
    }
  },

  updateSettings: async (settings: UserSettings) => {
    try {
      await invoke('update_settings', { newSettings: settings });
      set({ settings });
    } catch (error) {
      console.error('Failed to update settings:', error);
    }
  },
}));
