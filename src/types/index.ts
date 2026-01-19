// Dashboard types - matches Rust backend structs

export interface DailyStats {
  date: string;
  transcriptions: number;
  words: number;
  audio_seconds: number;
}

export interface LocalStats {
  total_transcriptions: number;
  total_words: number;
  total_audio_seconds: number;
  streak_days: number;
  last_used_date: string | null;
  daily_history: DailyStats[];
}

export interface TranscriptionEntry {
  id: string;
  text: string;
  word_count: number;
  duration_ms: number;
  timestamp: string;
  synced: boolean;
}

export interface DictionaryEntry {
  id: string;
  phrase: string;
  replacement: string;
  enabled: boolean;
  synced: boolean;
}

// Dashboard navigation
export type DashboardPage = 'home' | 'history' | 'dictionary' | 'settings';
