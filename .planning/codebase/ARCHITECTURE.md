# Architecture

**Analysis Date:** 2026-02-24

## Pattern Overview

**Overall:** Tauri-based desktop application with dual-window architecture (dictation overlay + settings dashboard), featuring command-based IPC between React frontend and Rust backend.

**Key Characteristics:**
- Two independent windows: fullscreen overlay (dictation) and modal settings (dashboard)
- Unidirectional command flow: frontend → Rust backend via `invoke()`, backend → frontend via events (`emit()`)
- Settings synchronized via Zustand store (frontend) and Mutex<AppState> (backend)
- macOS NSPanel for fullscreen overlay capability (window layer 25, non-activating mask 128)
- Monolithic Whisper/Voxtral transcription engine selection via feature flags
- Local-first architecture: models cached at `~/.mentascribe/models/`, settings at `~/.config/mentascribe/`

## Layers

**Frontend (React/TypeScript):**
- Purpose: User-facing dictation overlay and settings dashboard
- Location: `src/`
- Contains: React components, Zustand stores, Tauri API wrappers
- Depends on: Tauri core (`@tauri-apps/api`), Tailwind CSS
- Used by: End user through native window

**Tauri Bridge:**
- Purpose: IPC bridge between React and Rust, event emission system
- Location: Built-in via Tauri v2
- Contains: Command registration, event listeners, window management
- Depends on: Tauri core, Rust backend
- Used by: Both frontend and backend

**Rust Backend (Tauri Commands):**
- Purpose: Core logic: audio capture, transcription, settings management, hotkey handling, text injection
- Location: `src-tauri/src/`
- Contains: Modules for audio, transcription, injection, settings, history, dictionary, API
- Depends on: whisper-rs (GGML), Voxtral FFI, CoreGraphics (macOS), global-shortcut plugin
- Used by: Frontend via `invoke()` commands

**Audio Capture Layer:**
- Purpose: Stream raw audio PCM from system microphone
- Location: `src-tauri/src/audio/capture.rs`, `src-tauri/src/audio/vad.rs`
- Contains: Audio stream setup, frame chunking, silence detection (VAD)
- Depends on: cpal (cross-platform audio), silero-vad model
- Used by: Transcription engines (Whisper/Voxtral)

**Transcription Layer:**
- Purpose: Convert audio PCM → text (Whisper or Voxtral)
- Location: `src-tauri/src/transcription/`
- Contains: Whisper engine (streaming + VAD), Voxtral FFI wrapper, cloud providers
- Depends on: whisper-rs (Whisper), voxtral FFI bindings, optional CoreML acceleration
- Used by: Main recording flow

**Settings/State Management:**
- Purpose: Persist user preferences (transcription engine, hotkey, widget opacity, etc.)
- Location: Frontend: `src/lib/store.ts` (Zustand), Backend: `src-tauri/src/settings/mod.rs` (JSON file)
- Contains: UserSettings struct with transcription/hotkey/output/widget/cleanup settings
- Depends on: dirs crate for config directory resolution
- Used by: All modules requiring user preferences

## Data Flow

**Recording Flow (User Presses F6):**

1. Global hotkey listener (`src-tauri/src/hotkey/mod.rs`) detects key press
2. Emits `hotkey-pressed` event to frontend
3. Frontend (`src/App.tsx`) calls `startRecording()` → `invoke('start_recording')`
4. Rust backend:
   - Sets `is_recording = true` in AppState
   - Starts audio capture (`audio::capture::start_capture()`)
   - Spawns transcription engine (Whisper or Voxtral based on settings)
5. Transcription streams:
   - Audio frames → VAD filter → Whisper/Voxtral model
   - Backend emits `audio-level` events (for waveform visualization)
   - Backend emits `transcription-complete` when done
6. Frontend receives events, updates UI state (isRecording, audioLevel)

**Text Injection Flow:**

1. Transcription complete → backend calls `invoke('stop_recording')`
2. Rust backend returns transcribed text
3. Frontend calls `invoke('inject_text', { text })`
4. Rust injection layer (`src-tauri/src/injection/mod.rs`):
   - macOS: Simulates Cmd+V (paste) via CGEvent + clipboard
   - Linux: Uses X11 clipboard + key simulation
5. Frontend saves transcription to localStorage history
6. Backend saves to persistent history (`~/.config/mentascribe/history.json`)

**Settings Synchronization:**

1. Frontend: User changes hotkey in SettingsPage
2. Frontend calls `invoke('update_settings', { newSettings })`
3. Backend:
   - Locks AppState mutex
   - Writes settings to `~/.config/mentascribe/settings.json`
   - Updates in-memory AppState.settings
   - Re-registers hotkey if key changed (`hotkey::setup_hotkey()`)
4. Backend emits `settings-changed` event
5. Frontend listener calls `loadSettings()` to sync Zustand store

**Monitor Tracking Flow (Mixed-DPI Handling):**

1. Frontend periodically calls `invoke('reposition_to_mouse_monitor')` (~150ms)
2. Rust backend (see MEMORY.md for mixed-DPI coordinate fix):
   - Gets cursor position in CG point space (not physical pixels)
   - Compares against each monitor's origin (converted to CG points)
   - If cursor moved to different monitor, adjusts window position
   - Returns boolean if repositioned
3. Frontend updates window position state if needed

**State Management:**

- Frontend: Zustand store (`useStore()`) holds `settings: UserSettings | null`
- Backend: AppState Mutex holds `UserSettings`, plus `is_recording` and audio level emitter flag
- Settings are read-through on startup (`loadSettings()`) and updated incrementally
- Window state (recording, processing) lives in frontend React state, synced with backend via events

## Key Abstractions

**DictationBar Component:**
- Purpose: Renders fullscreen overlay pill with waveform, recording indicator, error display
- Examples: `src/components/DictationBar.tsx`
- Pattern: Functional component with refs for audio animation state, useInterval for waveform updates, cursor proximity polling via Rust

**Dashboard Component:**
- Purpose: Settings window with multi-page navigation (Home, History, Dictionary, Settings)
- Examples: `src/components/dashboard/Dashboard.tsx`, `src/components/dashboard/SettingsPage.tsx`
- Pattern: Layout with Sidebar navigation, theme provider, page routing based on URL hash

**Tauri Command Handler:**
- Purpose: Rust function decorated with `#[tauri::command]` macro, callable from frontend via `invoke()`
- Examples: `start_recording()`, `stop_recording()`, `inject_text()`, `update_settings()`
- Pattern: Receive AppHandle for state access and events, return Result<T, String> for error propagation

**Settings Store (Dual):**
- Purpose: Single source of truth for user preferences
- Examples: Frontend `src/lib/store.ts` (Zustand), Backend `src-tauri/src/settings/mod.rs` (persistent JSON)
- Pattern: Read on app startup, write on user action, emit events on change

**Transcription Engine Abstraction:**
- Purpose: Pluggable transcription backend (Whisper vs Voxtral)
- Examples: `src-tauri/src/transcription/whisper.rs`, `src-tauri/src/transcription/voxtral.rs` (feature-gated)
- Pattern: Feature flags determine compilation, settings.transcription.engine selects runtime, both implement same event emission interface

## Entry Points

**Frontend Entry:**
- Location: `src/main.tsx`
- Triggers: Browser (Tauri WebviewWindow loads HTML)
- Responsibilities: React root mount, loads App.tsx component tree

**App.tsx Main Router:**
- Location: `src/App.tsx`
- Triggers: App component initialization
- Responsibilities: Routes to DictationBar or Dashboard based on URL hash, sets up global event listeners (hotkey, transcription complete, audio level, model preload), manages recording state

**Rust Backend Entry:**
- Location: `src-tauri/src/lib.rs::run()`
- Triggers: Desktop app launch (via main.rs)
- Responsibilities: Creates AppState, registers Tauri commands, sets up two windows (dictation + dashboard), converts dictation to NSPanel on macOS, spawns hotkey system

**Dashboard Entry:**
- Location: `src/components/dashboard/Dashboard.tsx`
- Triggers: App.tsx if URL hash contains "dashboard"
- Responsibilities: Renders multi-page settings UI, listens for navigate-to-page events from tray menu, loads initial page from URL hash

**Dictation Window Initialization:**
- Location: `src-tauri/src/lib.rs::setup_dictation_panel()`
- Triggers: After dictation window created
- Responsibilities: Converts NSWindow to NSPanel (macOS only), sets window level 25, applies collection behaviors for fullscreen overlay, applies non-activating mask

## Error Handling

**Strategy:** Try-catch in frontend, Result<T, String> in backend, propagate errors to user via status messages and error toasts.

**Patterns:**

- **Transcription Errors:** If model not found, emit event that triggers download dialog in Settings
- **Mic Errors:** Catch at command boundary, return descriptive error string to frontend, display 2s timeout error banner
- **Accessibility Errors (macOS):** Check AXIsProcessTrusted() at injection time, prompt user to enable in System Settings
- **Settings Persistence:** Use serde_json for JSON errors, fall back to defaults if malformed
- **Audio Capture Errors:** Propagate cpal error messages, frontend shows "Mic busy — try again"
- **Hotkey Registration:** Parse key string safely, return UnknownKey error if unsupported (only F1-F12 supported)

## Cross-Cutting Concerns

**Logging:**
- Backend: `log` crate + `env_logger`, stderr via `eprintln!()` for recording events
- Frontend: `console.log/error`, browser DevTools
- Tauri logs accessible via tray menu → Settings

**Validation:**
- Settings: Optional fields with defaults (engine defaults to "whisper", opacity defaults to 1.0)
- Transcription: Model size validated against list of known sizes (tiny, base, small, medium, large)
- Text injection: Text chunked at UTF-16 boundaries (max 20 units per CGEvent on macOS)

**Authentication:**
- API module defined (`src-tauri/src/api/mod.rs`, `src-tauri/src/api/client.rs`) but not active in current build
- CloudProvider enum exists but Whisper is default transcription engine
- Could be extended for future cloud-based transcription

**Concurrency:**
- AppState uses Mutex for thread-safe access to settings and recording flag
- Audio capture runs in separate thread (cpal callback)
- Whisper inference runs async (no blocking on main thread)
- Model cache (MODEL_CACHE, STATE_CACHE) protected by Lazy + Mutex for multi-use access

---

*Architecture analysis: 2026-02-24*
