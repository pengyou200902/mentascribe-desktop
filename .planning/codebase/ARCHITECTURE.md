# Architecture

**Analysis Date:** 2026-04-05

## Pattern Overview

**Overall:** Multi-window Tauri v2 desktop application with a Rust native backend and React/TypeScript frontend

**Key Characteristics:**
- Two distinct Tauri windows sharing a single React app, differentiated by URL hash (`#dashboard` vs default dictation)
- Unidirectional command-event IPC: Frontend calls `invoke()` commands, backend emits events via `app.emit()`
- Zustand stores on the frontend, `Mutex<AppState>` on the backend
- Platform-specific native code gated by `#[cfg(target_os)]` for macOS NSPanel, Windows clipboard, Linux X11
- Feature-gated Voxtral engine (`#[cfg(feature = "voxtral")]`) alongside default Whisper engine
- All persistent data stored as JSON files in `~/.config/mentascribe/`

## Layers

**Frontend UI Layer:**
- Purpose: React components rendering dictation overlay and dashboard management UI
- Location: `src/components/`, `src/App.tsx`
- Contains: React functional components, Tailwind CSS styling, SVG icons inline
- Depends on: Tauri API (`@tauri-apps/api/core`, `@tauri-apps/api/event`), Zustand stores
- Used by: `src/main.tsx` entry point

**Frontend State Layer:**
- Purpose: Centralized state management via Zustand stores
- Location: `src/lib/store.ts`, `src/lib/historyStore.ts`, `src/lib/dictionaryStore.ts`, `src/lib/statsStore.ts`
- Contains: Zustand store definitions with async `invoke()` calls
- Depends on: `@tauri-apps/api/core` for `invoke()`
- Used by: React components via `useStore()`, `useHistoryStore()`, `useDictionaryStore()`, `useStatsStore()` hooks
- Pattern: `create<StoreInterface>((set, get) => ({ ... }))`

**Tauri IPC Bridge:**
- Purpose: Typed RPC between JS frontend and Rust backend
- Location: Commands defined in `src-tauri/src/lib.rs` via `#[tauri::command]`, registered in `tauri::generate_handler![]`
- Contains: ~35 command handlers, event emissions
- Key commands: `start_recording`, `stop_recording`, `inject_text`, `get_settings`, `update_settings`, `download_model`, `get_history`, `get_dictionary`, `get_stats`, `resize_pill`, `reposition_to_mouse_monitor`, `start_native_drag`, `is_cursor_over_pill`
- Key events: `hotkey-pressed`, `hotkey-released`, `audio-level`, `transcription-processing`, `transcription-complete`, `settings-changed`, `model-preload-start`, `model-preload-complete`, `model-preload-error`, `model-needs-download`, `download-progress`, `navigate-to-page`

**Rust Backend - Application Core:**
- Purpose: Command handlers, app initialization, window management, state coordination
- Location: `src-tauri/src/lib.rs` (~1670 lines)
- Contains: `AppState` struct, `run()` function, all `#[tauri::command]` handlers, macOS NSPanel setup, native drag/positioning, tray menu
- Manages: Global state (recording flag, settings, audio level emitter), window lifecycle, model preloading

**Rust Backend - Subsystem Modules:**

| Module | Location | Purpose |
|--------|----------|---------|
| audio | `src-tauri/src/audio/` | Microphone capture (CPAL), real-time resampling (rubato), voice activity detection |
| transcription | `src-tauri/src/transcription/` | Speech-to-text via Whisper.cpp or Voxtral, model management, streaming inference |
| settings | `src-tauri/src/settings/mod.rs` | Configuration persistence (`settings.json`), typed structs with Serde |
| hotkey | `src-tauri/src/hotkey/mod.rs` | Global keyboard shortcut registration via tauri-plugin-global-shortcut |
| injection | `src-tauri/src/injection/mod.rs` | Text insertion into active application (platform-specific: CGEvent/Windows/X11) |
| history | `src-tauri/src/history/mod.rs` | Transcription log persistence (`history.json`), CRUD operations |
| dictionary | `src-tauri/src/dictionary/mod.rs` | Text replacement rules (`dictionary.json`), in-memory RwLock cache |
| stats | `src-tauri/src/stats/mod.rs` | Usage metrics persistence (`stats.json`), daily/streak tracking |
| text | `src-tauri/src/text/mod.rs` | Post-transcription processing (auto-capitalization) |
| api | `src-tauri/src/api/` | External API client for MentaFlux cloud service, keyring token storage |

**Native C Layer (Voxtral, feature-gated):**
- Purpose: Custom speech-to-text engine with Metal GPU acceleration
- Location: `src-tauri/voxtral/` (C source), `src-tauri/src/transcription/voxtral_ffi.rs` (Rust FFI bindings)
- Contains: Encoder, decoder, tokenizer, audio processing, Metal shaders, safetensors model loading
- Build: Compiled via `cc` crate in `src-tauri/build.rs`, Metal shaders precompiled to `.metallib`
- Platform: macOS Apple Silicon (Metal GPU), Linux (OpenBLAS), Windows (CPU fallback)

## Data Flow

**Recording Flow (primary data pipeline):**

1. User presses global hotkey (F6 default)
2. Rust `hotkey/mod.rs` emits `hotkey-pressed` event to all windows
3. Frontend `App.tsx` (dictation window only) receives event, calls `invoke('start_recording')`
4. Rust `start_recording()` in `lib.rs`:
   - Locks `AppState.is_recording` mutex, sets to `true`
   - Calls `audio::capture::start_capture()` which spawns CPAL audio thread
   - Determines engine from settings (`is_voxtral_engine()`)
   - Starts streaming transcription (`whisper::start_streaming()` or `voxtral::start_streaming()`)
   - Spawns audio level emitter thread (25ms interval, emits `audio-level` events)
5. Frontend receives `audio-level` events, drives waveform animation in `DictationBar.tsx`

6. User releases hotkey or presses again (toggle mode)
7. Frontend calls `invoke('stop_recording')`
8. Rust `stop_recording()`:
   - Stops audio level emitter (`AtomicBool` flag)
   - Stops streaming transcriber, collects partial results and consumed sample count
   - Stops audio capture, gets full `AudioData`
   - Trims tail audio (only un-transcribed portion for final inference)
   - Runs final transcription on tail audio (or uses streaming results if complete)
   - Applies `text::process_text()` (auto-capitalize)
   - Applies `dictionary::apply_replacements()`
   - Saves to `history::add_entry()`
   - Records `stats::record_transcription()`
   - Emits `transcription-complete` event
   - Returns final text string
9. Frontend receives text, calls `invoke('inject_text', { text })` to paste into active app
10. Frontend saves to localStorage history (duplicate, legacy)

**Settings Update Flow:**

1. User changes setting in `SettingsPage.tsx`
2. Zustand store calls `invoke('update_settings', { newSettings })`
3. Rust `update_settings()` compares old vs new, triggers side effects:
   - Hotkey changed: `hotkey::unregister_all()` then `hotkey::setup_hotkey()`
   - Draggable changed to false: `native_position_on_cursor_monitor()` to snap back
   - Opacity changed: `apply_panel_opacity()` on macOS NSPanel
   - Engine changed: Unload old engine, preload new one in background thread
   - Model size changed: Preload new model in background thread
4. Persists via `settings::save_settings()`
5. Emits `settings-changed` event to all windows
6. Both windows reload settings via Zustand store

**Multi-Monitor Positioning Flow (macOS):**

1. Frontend `App.tsx` runs 150ms polling interval calling `invoke('reposition_to_mouse_monitor')`
2. Rust uses native `NSEvent.mouseLocation` and `NSScreen.screens` to find cursor's screen
3. If widget center is on a different screen, repositions to bottom-center of cursor's screen
4. Skipped when `settings.widget.draggable` is true (user controls position)

## State Management

**Frontend State:**
- `useStore()` (`src/lib/store.ts`): Settings loaded on mount, updated on `settings-changed` event
- `useHistoryStore()` (`src/lib/historyStore.ts`): Paginated (50 per page), loaded on demand
- `useDictionaryStore()` (`src/lib/dictionaryStore.ts`): Full list, CRUD via invoke
- `useStatsStore()` (`src/lib/statsStore.ts`): Aggregated stats, loaded on demand
- `ThemeProvider` (`src/lib/theme.tsx`): React Context for light/dark/system theme with localStorage persistence
- Component-local state: `isRecording`, `isProcessing`, `audioLevel`, `error`, `isDownloadingModel`, `isPreloading` (React useState in `App.tsx`)
- Refs used to avoid stale closures in event listeners: `isRecordingRef`, `isProcessingRef`, `settingsRef`

**Backend State:**
- `AppState` struct (managed by Tauri):
  - `is_recording: Mutex<bool>` — prevents concurrent recordings
  - `settings: Mutex<UserSettings>` — in-memory settings cache
  - `audio_level_emitter_running: Arc<AtomicBool>` — thread coordination signal
- Static globals in modules:
  - `audio::capture`: `AUDIO_BUFFER`, `WHISPER_BUFFER`, `AUDIO_THREAD`, `SAMPLE_RATE`, `CHANNELS`, `CURRENT_AUDIO_LEVEL` (all `lazy_static! Mutex`)
  - `transcription::whisper`: `MODEL_CACHE` (Lazy Mutex), `STATE_CACHE` (Lazy Mutex) for pre-created WhisperState
  - `dictionary`: `DICTIONARY_CACHE` (Lazy RwLock) for read-heavy concurrent access

## Key Abstractions

**UserSettings:**
- Purpose: Complete representation of all user preferences
- Frontend: `src/lib/store.ts` (TypeScript interfaces)
- Backend: `src-tauri/src/settings/mod.rs` (Rust structs with Serde Serialize/Deserialize)
- Contains: `TranscriptionSettings`, `CleanupSettings`, `HotkeySettings`, `OutputSettings`, `WidgetSettings`
- Pattern: Defaults via `Default` trait, optional fields with `Option<T>`, `#[serde(default)]` for backward compatibility

**AudioData:**
- Purpose: Audio buffer passed through the recording pipeline
- Location: `src-tauri/src/audio/capture.rs`
- Fields: `samples: Vec<f32>`, `sample_rate: u32`, `channels: u16`, `whisper_samples: Option<Vec<f32>>` (pre-resampled 16kHz mono)
- Pattern: Created by `stop_capture()`, consumed by transcription engines

**Tauri Events:**
- Purpose: Async backend-to-frontend notifications
- Pattern: `app.emit("event-name", payload)` in Rust, `listen("event-name", callback)` in TypeScript
- Lifecycle: Event listeners registered in `useEffect` hooks, cleanup functions returned and called on unmount
- Critical pattern in `App.tsx`: Refs (`isRecordingRef`, etc.) prevent stale closures since event listeners capture values at registration time

**Window Type Detection:**
- Purpose: Single React app serves two window types
- Pattern: `window.location.hash` check at mount time — empty or `#dictation` = overlay, `#dashboard` = management UI
- Implementation: `getWindowType()` in `App.tsx`, `getInitialPage()` in `Dashboard.tsx`
- Dashboard sub-navigation: hash format `#dashboard/settings`, `#dashboard/history`

## Entry Points

**Frontend Entry (`src/main.tsx`):**
- Mounts React app at `#root` element
- Wraps in `React.StrictMode`
- No router — window type determined by hash

**App Component (`src/App.tsx`):**
- Root component, ~325 lines
- Determines window type once at mount (`useState(getWindowType)`)
- If dictation: renders `DictationBar` with all recording state props
- If dashboard: renders `Dashboard` component (which uses `ThemeProvider`)
- Sets up all event listeners in a single `useEffect` block with cleanup

**Rust Entry (`src-tauri/src/main.rs`):**
- Minimal: calls `mentascribe_desktop::run()`
- `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` hides console on Windows release builds

**Rust Application Setup (`src-tauri/src/lib.rs::run()`):**
- Initializes env_logger
- Loads settings from disk
- Builds `tauri::Builder` with plugins: shell, dialog, fs, http, global-shortcut, nspanel (macOS)
- `setup()` closure:
  - Registers global hotkey from settings
  - Auto-detects CoreML on macOS and enables if supported
  - Preloads configured speech model in background thread
  - Shows dictation window, converts to NSPanel on macOS
  - Positions dictation window at bottom-center of cursor's monitor
  - Builds system tray with menu (Settings, History, Show/Hide Widget, Quit)
  - Tray left-click opens dashboard, right-click shows menu
- Registers all ~35 command handlers via `generate_handler![]`
- Manages `AppState` via `.manage()`

## Error Handling

**Strategy:** Result-based propagation in Rust, user-facing messages in frontend

**Rust Patterns:**
- Module-level error enums with `thiserror::Error` derive: `WhisperError`, `AudioError`, `HotkeyError`, `InjectionError`, `SettingsError`, `HistoryError`, `DictionaryError`, `StatsError`, `ApiError`, `CloudError`
- `#[tauri::command]` functions return `Result<T, String>` — errors mapped via `.map_err(|e| e.to_string())`
- Critical paths use `eprintln!()` for immediate stderr output alongside `log::info/warn/error`

**Frontend Patterns:**
- Try/catch around `invoke()` calls
- Error state with timed auto-clear (configurable timeouts in `src/config/widget.ts`)
- Specific error detection: model not found triggers auto-download, mic busy shows retry message
- Graceful degradation: history/stats save failures are logged but don't fail transcription

**User Feedback:**
- Error messages displayed in DictationBar pill widget with auto-dismiss
- Model download progress shown via `download-progress` events
- Model preload status via banner in dashboard

## Cross-Cutting Concerns

**Logging:**
- Backend: `log` crate with `env_logger` (levels via `RUST_LOG` env var), plus `eprintln!()` for critical debug output
- Frontend: `console.log()` / `console.error()`, plus `invoke('frontend_log', { msg })` to forward to Rust stderr

**Validation:**
- Settings: Optional fields with defaults, `#[serde(default)]` annotations for backward compatibility
- Audio: Resampling to 16kHz mono required for Whisper (handled in `capture.rs` callback)
- Text injection: macOS CGEvent limits to 20 UTF-16 code units per event (chunked in `injection/mod.rs`)

**Authentication:**
- Cloud API client at `src-tauri/src/api/client.rs` with login/token-refresh against `https://api.voice.mentaflux.ai/v1`
- Token storage: OS keychain via `keyring` crate (`keyring::Entry::new("mentascribe", "tokens")`)
- Not deeply integrated into the recording flow — login command exposed but not required for local transcription

**Concurrency:**
- Audio capture: CPAL callback thread + separate audio processing thread
- Audio level emitter: `std::thread::spawn` with `AtomicBool` stop signal
- Model preloading: Background `std::thread::spawn` with event emissions for progress
- Streaming transcription: Background monitoring thread per engine
- Shared state protection: `Mutex` for write-heavy state, `RwLock` for read-heavy dictionary cache
- Pattern: `Arc<AtomicBool>` for thread signaling, `Mutex<Option<T>>` for optional resources

**Platform-Specific Code:**
- Gated by `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "windows")]`, `#[cfg(target_os = "linux")]`
- macOS-specific: NSPanel overlay, native drag via NSEvent monitors, CGEvent text injection, Core Graphics accessibility check, Metal GPU acceleration
- Windows-specific: `clipboard-win` + `windows` crate for text injection
- Linux-specific: `x11` crate with XTest for key simulation
- Non-macOS stubs: `setup_dictation_panel()`, `refresh_panel_settings()`, `start_native_drag()` are no-ops

## Process Model

**Multi-process (Tauri architecture):**
- Main process: Rust binary hosting Tauri runtime, WebView management, all backend logic
- Renderer processes: WebView (WKWebView on macOS, WebView2 on Windows) per window
- Two windows: `dictation` (always-on-top overlay) and `dashboard` (standard window, created on demand)
- Thread model within main process:
  - Main thread: Tauri event loop, command handlers
  - Audio capture thread: CPAL stream callback
  - Audio level emitter thread: 25ms polling loop
  - Model preload thread: One-shot background loading
  - Streaming transcription monitor thread: VAD-triggered inference

---

*Architecture analysis: 2026-04-05*
