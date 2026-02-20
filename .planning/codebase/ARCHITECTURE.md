# Architecture

**Analysis Date:** 2026-02-20

## Pattern Overview

**Overall:** Tauri v2 hybrid desktop application with cross-platform backend (Rust) and fullscreen overlay UI (React/TypeScript)

**Key Characteristics:**
- **Two-window system**: Dictation overlay (NSPanel on macOS for fullscreen support) + Dashboard (settings/history/stats)
- **Backend-driven recording**: Rust backend handles audio capture, transcription, text injection; frontend provides UI controls only
- **Event-based communication**: Tauri IPC (invoke/emit) between React frontend and Rust backend
- **State synchronization**: Settings stored in disk JSON, loaded into Rust Mutex<AppState> at startup, synced bidirectionally via events
- **Streaming transcription**: VAD-triggered utterance detection during recording reduces inference time on stop

## Layers

**Presentation Layer (React/TypeScript):**
- Purpose: Fullscreen overlay UI, settings dashboard, history/dictionary browsing
- Location: `src/components/`, `src/App.tsx`
- Contains: React components, Zustand stores, event listeners
- Depends on: Tauri API (invoke, listen, emit), localStorage for temporary history
- Used by: User interactions trigger Rust commands via `invoke()`

**IPC/Command Layer (Tauri Bridge):**
- Purpose: Expose Rust functionality as async commands callable from frontend
- Location: `src-tauri/src/lib.rs` (all `#[tauri::command]` functions)
- Contains: Recording control, settings I/O, model downloads, window positioning
- Depends on: Tauri runtime for app context and state
- Used by: Frontend via `invoke('command_name', args)`

**Business Logic Layer (Rust):**
- Purpose: Core transcription workflow, audio processing, text injection
- Location: `src-tauri/src/` (subdirectories: audio, transcription, injection, hotkey, etc.)
- Contains: Audio capture/resampling, Whisper transcription (local + cloud), keyboard injection, hotkey handling
- Depends on: External Rust crates (cpal, whisper-rs, enigo, global-hotkey, etc.)
- Used by: IPC command layer invokes these modules

**Data Layer:**
- Purpose: Persistent storage for settings, history, statistics, dictionary
- Location: JSON files in `~/.config/mentascribe/` (settings.json, history.json, dictionary.json, stats.json)
- Contains: User configuration, transcription records with timestamps/word counts, dictionary rules
- Depends on: Serde JSON serialization
- Used by: Settings and history modules load/save on disk

**Platform Layer:**
- Purpose: OS-specific integrations for audio, window management, text injection
- Location: `src-tauri/src/` (platform gates via `#[cfg(target_os = "...")]`)
- Contains: macOS (CoreGraphics, NSPanel via tauri-nspanel), Windows (Win32 API), Linux (X11)
- Depends on: Platform-specific crates (cocoa, objc, windows, x11)
- Used by: Injection, audio capture, window positioning modules

## Data Flow

**Recording Lifecycle:**

1. User presses hotkey (F6 default) → hotkey module detects press → emit "hotkey-pressed" event
2. Frontend receives event → calls `invoke('start_recording')`
3. Backend starts CPAL audio stream + spawns VAD streaming transcription in background
4. Audio frames flow: CPAL callback → mono conversion → resampler (16kHz mono) → WHISPER_BUFFER
5. VAD detects silence → triggers streaming Whisper transcription → stores completed utterances
6. Audio level emitted every 25ms → frontend updates waveform visualization
7. User releases hotkey → frontend calls `invoke('stop_recording')`
8. Backend stops CPAL, trims whisper_samples buffer to only untranscribed tail, transcribes tail
9. Text processing: raw transcript → auto-capitalize → dictionary replacements → inject into focused app
10. History/stats recorded locally, completion event emitted to frontend
11. Settings changed in Dashboard → `updateSettings()` → `invoke('update_settings')` → save to disk → emit "settings-changed" → all windows reload

**Multi-Monitor Tracking (Dictation Window):**

1. Frontend polls `invoke('reposition_to_mouse_monitor')` every 150ms
2. Rust detects mouse on different monitor → repositions dictation window to match monitor's coordinate space
3. Handles macOS mixed-DPI coordinate conversion (monitor.position divided by scale_factor)
4. Returns true if moved, false if no change

**Settings Synchronization:**

- Settings → Rust Mutex<UserSettings> (in-memory) + settings.json (disk)
- Dashboard SettingsPage calls `updateSettings()` → triggers file write + "settings-changed" event
- Dictation window listens for "settings-changed" → reloads settings from state

## Key Abstractions

**AudioData (struct):**
- Purpose: Bundle raw samples, sample rate, channels, and pre-processed 16kHz Whisper buffer
- Examples: `src-tauri/src/audio/capture.rs` (AudioData struct), `src-tauri/src/audio/mod.rs` (pub use)
- Pattern: Passed from capture → transcription modules; includes optional `whisper_samples` for streaming pre-processing

**UserSettings (struct):**
- Purpose: Represent all configuration: transcription, hotkey, output, widget, cleanup options
- Examples: `src-tauri/src/settings/mod.rs` (Rust struct), `src/lib/store.ts` (TypeScript interface)
- Pattern: Serde-serialized to JSON, cloned into Mutex for thread-safe access in command handlers

**Transcription Modules (whisper, cloud):**
- Purpose: Abstract local vs. cloud transcription backends
- Examples: `src-tauri/src/transcription/whisper.rs` (local Whisper via whisper-rs), `src-tauri/src/transcription/cloud.rs` (API endpoints)
- Pattern: Both expose `transcribe()` async function; streaming_config controls model size/language

**Event System:**
- Purpose: Decouple frontend UI updates from backend state changes
- Examples: "hotkey-pressed", "audio-level", "transcription-complete", "model-needs-download"
- Pattern: Backend emits via `app.emit()`, frontend listens via `listen()` and sets React state

## Entry Points

**Rust Entry Point:**
- Location: `src-tauri/src/main.rs` (empty, delegates to lib)
- Triggers: App launch via Tauri runtime
- Responsibilities: Minimal — calls `run()` from lib.rs

**Rust Library Entry Point:**
- Location: `src-tauri/src/lib.rs` (run() function, ~600+ lines)
- Triggers: Called from main.rs at startup
- Responsibilities:
  - Initialize AppState (is_recording, settings, audio_level_emitter_running)
  - Create two windows: "dictation" (NSPanel on macOS) and "dashboard"
  - Register all Tauri commands (start_recording, stop_recording, inject_text, etc.)
  - Set up tray icon with context menu
  - Call `setup_dictation_panel()` to configure NSPanel collection behavior on macOS
  - Load settings from disk and populate app state
  - Register global hotkey listener

**Frontend Entry Point:**
- Location: `src/main.tsx`
- Triggers: HTML page loads
- Responsibilities: Mount React app into #root element

**App Component Entry Point:**
- Location: `src/App.tsx`
- Triggers: React renders top-level component
- Responsibilities:
  - Determine window type (dictation vs. dashboard) via URL hash
  - Set up event listeners for hotkey, transcription events, audio levels
  - Load settings on mount
  - Render either DictationBar (overlay) or Dashboard (settings UI)
  - Manage recording state refs to avoid race conditions
  - Poll `reposition_to_mouse_monitor` for dictation window position tracking

**Dashboard Component Entry Point:**
- Location: `src/components/dashboard/Dashboard.tsx`
- Triggers: App renders when window hash is "dashboard"
- Responsibilities: Route between pages (home/history/dictionary/settings), listen for page navigation events

## Error Handling

**Strategy:** Domain-specific error types using `thiserror` crate; frontend displays user-friendly messages; critical errors logged to console.

**Patterns:**

- **Audio Errors:** AudioError enum (`src-tauri/src/audio/capture.rs`) — "No input device", "Already running", "Not running"
  - Frontend catches and shows: "Mic busy — try again" or "No audio input"

- **Transcription Errors:** Generic string errors from whisper.rs; checked for "Model not found" to trigger auto-download
  - Backend logs full error, frontend displays "Failed: {error_message}"

- **Settings Errors:** SettingsError enum with IO and Serde variants (`src-tauri/src/settings/mod.rs`)
  - Non-fatal if load fails; app starts with defaults

- **Injection Errors:** InjectionError enum (`src-tauri/src/injection/mod.rs`) — accessibility permission, platform-specific issues
  - Frontend shows: "Failed to paste: {error}"

- **Hotkey Errors:** HotkeyError for registration failures (`src-tauri/src/hotkey/mod.rs`)
  - Prevents app from starting if primary hotkey registration fails

- **Recording State Recovery:** `reset_recording_state()` command resets Mutex flags and audio buffers if stuck
  - Called manually from frontend if recording hangs

## Cross-Cutting Concerns

**Logging:**
- Rust: log crate (env_logger backend, initialized in lib.rs)
  - Emits to stderr; includes context tags like "[recording]", "[nspanel]", "[capture]"
- Frontend: console.log/console.error for development; stored in localStorage for history

**Validation:**
- Settings: Serde defaults + manual validation (opacity clamped 0.2–1.0)
- Audio input: Device availability check in capture.rs
- Hotkey keys: Restricted to F1–F12 via parse_key_code()

**Authentication:**
- Not currently implemented; future cleanup/cloud features will use API keys
- Keyring crate integrated for secure credential storage (API keys stored here when added)

**Coordinate System Handling (macOS Mixed-DPI Bug):**
- `cursor_position()` returns points in CG coordinate space (not physical pixels)
- `monitor.position()` must be divided by monitor.scale_factor to get points
- `monitor.size()` returns true physical pixels (no division needed)
- `set_position()` called with LogicalPosition to avoid Tauri's scale multiplication
- See `src-tauri/src/lib.rs`: `native_position_on_cursor_monitor()` implements this fix

**Window State:**
- Dictation: NSPanel (macOS only) configured for fullscreen overlay + non-activating + can-join-all-spaces
- Dashboard: Regular window, alwaysOnTop on Windows/Linux, brought to foreground on macOS
- Both windows can emit/listen to same events (settings-changed, hotkey-pressed, etc.)

---

*Architecture analysis: 2026-02-20*
