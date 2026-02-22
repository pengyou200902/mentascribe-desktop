// Centralized widget constants — single place to tweak dictation pill behavior.

// ---- Waveform ----
export const WAVEFORM_BAR_COUNT = 9;
export const WAVEFORM_INITIAL_HEIGHT = 0.3;
export const WAVEFORM_UPDATE_INTERVAL_MS = 50;
export const WAVEFORM_SMOOTHING = 0.35;
export const WAVEFORM_BASE_MIN = 0.25;
export const WAVEFORM_CENTER_AMPLITUDE = 0.3;
export const WAVEFORM_NOISE_RANGE = 0.2;
export const WAVEFORM_RANDOM_RANGE = 0.4;
export const AUDIO_BOOST_BASE = 0.4;
export const AUDIO_BOOST_RANGE = 0.4;
export const WAVEFORM_MAX_HEIGHT = 1.0;
export const BAR_MIN_HEIGHT_PX = 4;
export const BAR_HEIGHT_SCALE = 20;

// ---- Processing ----
export const PROCESSING_DOT_COUNT = 8;
export const PROCESSING_DOT_DELAY_STEP = 0.1;

// ---- Polling & timing ----
export const CURSOR_POLL_INTERVAL_MS = 100;
export const MONITOR_POLL_INTERVAL_MS = 150;
export const MONITOR_LOG_FREQUENCY = 20;
export const PRELOAD_FLASH_DURATION_MS = 600;

// ---- Error timeouts ----
export const MIC_ERROR_TIMEOUT_MS = 2000;
export const ERROR_TIMEOUT_MS = 5000;
export const MODEL_PRELOAD_ERROR_TIMEOUT_MS = 3000;
export const MODEL_DOWNLOAD_ERROR_TIMEOUT_MS = 10000;

// ---- Defaults ----
export const DEFAULT_HOTKEY_LABEL = 'F6';
export const DEFAULT_HOTKEY_MODE = 'toggle';
export const DEFAULT_WIDGET_OPACITY = 1.0;
export const MAX_HISTORY_ENTRIES = 100;
