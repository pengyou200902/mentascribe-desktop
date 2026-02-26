# Architecture

**Analysis Date:** 2026-02-26

## Pattern Overview

**Overall:** Multi-window Tauri desktop application with Rust backend + React/TypeScript frontend

**Key Characteristics:**
- Two distinct window types: dictation overlay (NSPanel on macOS) and dashboard management UI
- Unidirectional command flow: Frontend invokes Tauri commands → Rust backend → Frontend listens for events
- Zustand for lightweight frontend state management
- Rust Mutex-based AppState for backend synchronization
- Platform-specific implementations (macOS NSPanel, Windows clipboard, Linux X11)

## Layers

**Frontend UI Layer:**
- Purpose: React components for user interaction (dictation bar + dashboard)
- Location: `src/components/`, `src/App.tsx`
- Contains: React components, hooks, styling with Tailwind CSS
- Depends on: Tauri API (`@tauri-apps/api/core`), Zustand stores
- Used by: App entry point
- Key files: `DictationBar.tsx` (overlay), `Dashboard.tsx` (settings/history/stats)

**Frontend State Layer:**
- Purpose: Centralized state management via Zustand stores
- Location: `src/lib/store.ts`, `src/lib/historyStore.ts`, `src/lib/dictionaryStore.ts`, `src/lib/statsStore.ts`
- Contains: Store definitions with async invoke patterns
- Depends on: Tauri core API for invoke/listen
- Used by: React components via hooks
- Pattern: Create stores with `create()`, expose via `useStore()` hook

**Tauri IPC Bridge:**
- Purpose: RPC between frontend and Rust backend
- Location: Commands in `src-tauri/src/lib.rs`, events via `app.emit()`
- Contains: Decorated functions with `#[tauri::command]`, event emitters
- Key commands: `start_recording`, `stop_recording`, `inject_text`, `get_settings`, `update_settings`
- Key events: `hotkey-pressed`, `hotkey-released`, `audio-level`, `transcription-complete`, `settings-changed`

**Rust Backend Application Layer:**
- Purpose: Core application logic and business rules
- Location: `src-tauri/src/lib.rs` (~1662 lines)
- Contains: Command handlers, app initialization, AppState management, event dispatch
- Entry points: `tauri::Builder` setup, `setup_dictation_panel()`, `start_recording()`, `stop_recording()`
- Manages: Global AppState with settings, recording state, audio level emitter

**Rust Backend - Subsystems:**

**Audio Capture Subsystem:**
- Purpose: Microphone input acquisition and voice activity detection
- Location: `src-tauri/src/audio/capture.rs`, `src-tauri/src/audio/vad.rs`
- Contains: CPAL audio device management, resampling, VAD via whisper-rs
- Used by: Recording flow (start_recording → audio::capture::start_capture)
- Key functions: `start_capture()`, `stop_capture()`, `get_current_level()`
- Data: AudioData struct with samples and sample_rate

**Transcription Subsystem:**
- Purpose: Convert audio to text via Whisper or Voxtral engines
- Location: `src-tauri/src/transcription/`
- Contains: Model management, streaming transcription, fallback logic
- Two engines: Whisper (local, CPU/GPU) and Voxtral (proprietary, Metal GPU on macOS)
- Whisper pipeline: `whisper::start_streaming()` (VAD trigger) + `whisper::stop_streaming()` (tail inference)
- Voxtral pipeline: `voxtral::start_streaming()` (continuous) + `voxtral::finish()` (final)
- Key data structures: ModelInfo, CoremlStatus, MetalStatus, VoxtralStatus

**Settings Management:**
- Purpose: Persistent user configuration
- Location: `src-tauri/src/settings/mod.rs`
- Persists to: `~/.config/mentascribe/settings.json`
- Structure: TranscriptionSettings, HotkeySettings, OutputSettings, WidgetSettings, CleanupSettings
- Flow: Frontend loads via `get_settings()` → Zustand store → UI → update via `update_settings()`
- Side effects: Hotkey re-registration, model preloading, panel reposition when draggable changes

**Hotkey System:**
- Purpose: Global keyboard shortcut listening
- Location: `src-tauri/src/hotkey/mod.rs`
- Uses: tauri-plugin-global-shortcut
- Modes: "toggle" (press = record/stop) or "hold" (press = start, release = stop)
- Events emitted: `hotkey-pressed`, `hotkey-released`
- Lifecycle: `setup_hotkey()` on init, re-register on settings change, `unregister_all()` before change

**Text Injection:**
- Purpose: Insert transcribed text into active application
- Location: `src-tauri/src/injection/mod.rs`
- Platform implementations:
  - macOS: CGEventKeyboardSetUnicodeString (raw keyboard events)
  - Windows: Windows clipboard API + paste simulation
  - Linux: X11 key simulation
- Method: Default is paste via clipboard (faster, more compatible)
- Error handling: Requires accessibility permission on macOS

**History Module:**
- Purpose: Local persistent record of transcriptions
- Location: `src-tauri/src/history/mod.rs`
- Persists to: `~/.config/mentascribe/history.json`
- Structure: TranscriptionEntry with id, text, word_count, duration_ms, timestamp, synced flag
- Limit: 500 entries max (older entries pruned)
- Commands: `get_history()`, `delete_entry()`, `clear_history()`, `get_total_count()`

**Dictionary Module:**
- Purpose: Text replacement rules (e.g., "u" → "you")
- Location: `src-tauri/src/dictionary/mod.rs`
- Persists to: `~/.config/mentascribe/dictionary.json`
- Applied: After transcription, before injection via `dictionary::apply_replacements()`
- Structure: DictionaryEntry with id, phrase, replacement, enabled flag

**Statistics Module:**
- Purpose: Aggregate usage metrics
- Location: `src-tauri/src/stats/mod.rs`
- Persists to: `~/.config/mentascribe/stats.json`
- Tracks: Daily transcription count, words, audio seconds, usage streak
- Updated: Automatically when transcription completes via `record_transcription()`

**Text Processing:**
- Purpose: Post-transcription text transformations
- Location: `src-tauri/src/text/mod.rs`
- Transforms: Auto-capitalization of sentence starts
- Applied: After transcription, before dictionary/injection

**macOS NSPanel Integration:**
- Purpose: Fullscreen overlay capability (dictation bar stays on top of fullscreen apps)
- Location: `src-tauri/src/lib.rs` functions `setup_dictation_panel()`, `refresh_panel_settings()`, `apply_panel_opacity()`
- Uses: tauri-nspanel plugin (GitHub: ahkohd/tauri-nspanel)
- Panel configuration:
  - Window level: 25 (NSMainMenuWindowLevel + 1)
  - Collection behavior: CanJoinAllSpaces, Stationary, FullScreenAuxiliary, IgnoresCycle
  - Style: NSNonactivatingPanelMask (128) to avoid focus stealing
  - Opacity: Configurable via NSPanel setAlphaValue
- Limitations: Focus stealing behavior, manual JS-level dragging workaround needed

## Data Flow

**Recording Flow:**

1. User presses hotkey (or dashboard dictation button)
2. Frontend calls `invoke('start_recording')`
3. Rust backend:
   - Sets `is_recording = true` in AppState
   - Starts audio capture via `audio::capture::start_capture()`
   - Determines engine (Whisper vs Voxtral) from settings
   - If Whisper: Starts streaming with VAD monitoring (emits partial results)
   - If Voxtral: Starts streaming mode with configured delay
   - Spawns audio level emitter thread (emits 25fps `audio-level` events)
4. Frontend updates state, shows waveform animation, processing spinner

5. User releases hotkey (or stops recording)
6. Frontend calls `invoke('stop_recording')`
7. Rust backend:
   - Stops audio level emitter
   - Stops streaming transcriber (collects remaining results)
   - Stops audio capture
   - Trims tail audio (only untranscribed portion)
   - Runs final transcription on tail (or returns streaming result if complete)
   - Applies text processing (auto-capitalize)
   - Applies dictionary replacements
   - Records history entry
   - Records stats
   - Emits `transcription-complete` event with final text
8. Frontend injects text, saves to local history

**Settings Update Flow:**

1. User changes setting in dashboard
2. Frontend calls `updateSettings()` which invokes `update_settings` command
3. Rust backend:
   - Locks AppState
   - Compares old vs new values for changes:
     - Hotkey changed: Unregister old, register new
     - Draggable changed: Re-position panel if disabled
     - Model size changed: Preload new model in background
     - Engine changed: Unload old, preload new
     - Opacity changed: Apply to NSPanel
   - Persists to disk via `settings::save_settings()`
   - Emits `settings-changed` event
4. Frontend receives event, reloads settings via store

**State Management:**

- **Frontend State**: Zustand stores (simple, centralized)
  - `useStore()`: Settings loaded once on mount, updated via events
  - `useHistoryStore()`: Paginated loading (50 items per page)
  - `useDictionaryStore()`: Dictionary entries
  - `useStatsStore()`: Usage statistics
  - Local state: Recording, processing, error messages (React useState)

- **Backend State**: AppState (Mutex-protected)
  - `is_recording`: Bool flag
  - `settings`: UserSettings clone (updated on settings command)
  - `audio_level_emitter_running`: Arc<AtomicBool> for thread coordination
  - Thread-safe via Mutex locks

## Key Abstractions

**UserSettings Struct:**
- Purpose: Typed representation of all user configuration
- Examples: `src/lib/store.ts`, `src-tauri/src/settings/mod.rs`
- Pattern: Serde-serializable, with defaults and optional fields
- Includes: TranscriptionSettings, HotkeySettings, OutputSettings, WidgetSettings

**AudioData Struct:**
- Purpose: Capsule for audio samples during recording
- Contains: samples (f32 vector), sample_rate (u32), whisper_samples (optional tail)
- Pattern: Passed between capture, processing, and transcription

**TranscriptionEntry:**
- Purpose: Single completed transcription record
- Used by: History storage and API
- Pattern: ID (UUID), text, word count, duration, timestamp, synced flag

**Events (Tauri):**
- Purpose: Async notifications from backend to frontend
- Pattern: One-way pub/sub via `app.emit()` and `listen()`
- Lifecycle: Registered in `useEffect` with cleanup in return
- Risk: Stale closures if not using refs (handled in App.tsx with Refs)

## Entry Points

**Frontend Entry Point:**
- Location: `src/main.tsx`
- Triggers: Browser load
- Responsibilities: Mount React DOM at #root

**App Component:**
- Location: `src/App.tsx`
- Triggers: React render
- Responsibilities:
  - Determine window type (dictation vs dashboard) via URL hash
  - Initialize event listeners (hotkey, audio level, transcription events)
  - Manage recording/processing state
  - Render DictationBar (overlay) or Dashboard (management UI)
  - Handle settings sync across windows

**Tauri Setup:**
- Location: `src-tauri/src/lib.rs` in `tauri::Builder`
- Triggered: Application startup
- Responsibilities:
  - Create AppState with initial settings
  - Create dictation and dashboard windows
  - Setup hotkey system
  - Register all Tauri commands
  - Setup macOS NSPanel if applicable
  - Initialize event listeners for inter-window communication
  - Setup tray menu (if configured)

## Error Handling

**Strategy:** Result-based error propagation with user-facing messages

**Patterns:**

- **Command Errors**: `#[tauri::command]` functions return `Result<T, String>` where error is serialized as string
  - Example: `stop_recording()` returns `Result<String, String>` (text on success, error message on failure)
  - Frontend catches and displays via error state

- **Module Errors**: Custom error enums with thiserror
  - Examples: WhisperError, HotkeyError, InjectionError, SettingsError
  - Pattern: `#[error("description")]` on variants, map to String in commands

- **Logging**: Rust logs via `log` crate (info/warn/error levels)
  - Stderr output captured in console
  - Frontend also logs to console via `console.error()`, `console.log()`

- **User Feedback**:
  - Recording errors: Show notification in UI, clear after timeout
  - Model not found: Trigger auto-download, show progress
  - Accessibility permission: Show clear message with system settings hint

## Cross-Cutting Concerns

**Logging:**
- Backend: `log` crate with env_logger (stderr)
- Frontend: browser console.log()
- Pattern: Eprintln for critical debug output during development, log:: for production

**Validation:**
- Settings: Validated on update, defaults applied if missing
- Audio: Sample rate validation (16000 Hz for Whisper)
- Text: UTF-16 chunk size limits for injection (20 units max per event on macOS)

**Authentication:**
- API login function exists (`src-tauri/src/api/client.rs`)
- Not heavily integrated (basic structure present)
- Pattern: Keyring for secure token storage

**Concurrency:**
- Tauri commands run on background thread pool (non-blocking)
- Audio capture: Separate thread via CPAL stream
- Audio level emitter: Spawned thread with AtomicBool coordination
- Model preloading: Background thread via std::thread::spawn
- Pattern: Arc/Mutex for shared state, AtomicBool for signaling

**Platform-specific Code:**
- Guarded by `#[cfg(target_os = "macos")]` and platform! macro
- Separate module sections for Windows, Linux implementations
- Example: injection/mod.rs has macOS (CGEvent), Windows (clipboard), Linux (X11) implementations

---

*Architecture analysis: 2026-02-26*
