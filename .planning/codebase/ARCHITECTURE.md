# Architecture

**Analysis Date:** 2026-02-18

## Pattern Overview

**Overall:** Multi-process Tauri Desktop Application with Separated Frontend and Backend

**Key Characteristics:**
- React 18 TypeScript frontend (Vite build)
- Rust backend using Tauri v2 framework
- IPC communication via Tauri commands and events
- Multiple specialized windows (dictation overlay, settings, dashboard, history)
- Platform-specific implementations (macOS NSPanel, Windows x86 emulation, Linux X11)

## Layers

**Frontend (React/TypeScript):**
- Purpose: UI rendering, user interactions, state management, window orchestration
- Location: `/src`
- Contains: React components, Zustand stores, event listeners, styling
- Depends on: Tauri API, Zustand for state management
- Used by: User interactions, triggered by hotkeys and window events

**Backend (Rust):**
- Purpose: System integration, audio processing, transcription, file storage, native APIs
- Location: `/src-tauri/src`
- Contains: Module packages for distinct features (audio, transcription, hotkey, injection, settings)
- Depends on: Tauri framework, system libraries, native crates
- Used by: Frontend via Tauri commands

**Bridge Layer (IPC):**
- Purpose: Frontend-backend communication
- Location: Tauri commands (exported from `/src-tauri/src/lib.rs`)
- Commands: `start_recording`, `stop_recording`, `inject_text`, `update_settings`, `get_history`, etc.
- Events: `hotkey-pressed`, `hotkey-released`, `audio-level`, `transcription-complete`, `transcription-processing`

## Data Flow

**Recording Workflow:**

1. User presses hotkey (F6 by default) → Hotkey module emits `hotkey-pressed` event
2. Frontend receives `hotkey-pressed` event → calls `start_recording` command
3. Backend starts audio capture (`audio::capture::start_capture()`)
4. Backend spawns audio level emitter thread → emits `audio-level` events periodically
5. User releases hotkey → `hotkey-released` event
6. Frontend calls `stop_recording` command
7. Backend stops capture, emits `transcription-processing` event
8. Backend calls `transcription::whisper::transcribe()` on captured audio
9. Backend applies text post-processing (auto-capitalize, dictionary replacements)
10. Backend saves to history and stats (fire-and-forget)
11. Backend emits `transcription-complete` event
12. Frontend calls `inject_text` command to paste text into active application
13. Text appears in target application

**Settings Management:**

1. Frontend loads settings via `get_settings` command on app startup
2. Settings stored in Zustand store (`useStore`)
3. User modifies settings in Settings window
4. Frontend calls `update_settings` command with new settings
5. Backend persists to settings file
6. If hotkey changed, backend re-registers global hotkey via `hotkey::setup_hotkey()`

**Dashboard/History:**

1. Frontend calls `get_history` command to fetch paginated entries from backend
2. Backend reads from local history store (`src-tauri/src/history`)
3. Frontend displays in HistoryPage component with pagination
4. User can delete entries via `delete_history_entry` or clear all via `clear_history`

**State Management:**

- Frontend: Zustand stores for settings (`useStore`), history (`useHistoryStore`), stats (`useStatsStore`)
- Backend: Shared `AppState` struct containing `is_recording` Mutex, `settings` Mutex, audio level emitter flag
- Window-level state: React component state (recording, processing, audio level, error)
- Persistence: Backend writes to local files; frontend uses localStorage for quick local caches

## Key Abstractions

**Audio Module (`audio/`):**
- Purpose: Cross-platform audio capture with voice activity detection
- Examples: `audio/capture.rs` (capture), `audio/vad.rs` (voice detection)
- Pattern: Module exports public functions; internal platform-specific code

**Transcription Module (`transcription/`):**
- Purpose: Whisper STT and cloud provider support
- Examples: `transcription/whisper.rs` (local Whisper-rs), `transcription/cloud.rs` (cloud API)
- Pattern: Provider abstraction; download and cache models locally

**Injection Module (`injection/`):**
- Purpose: Platform-native text injection (keyboard/clipboard simulation)
- Implementation: Conditional compilation per OS (macOS CoreGraphics, Windows Win32, Linux X11)
- Pattern: Unified error types; platform-specific modules inside

**Settings Module (`settings/`):**
- Purpose: Settings data structures and persistence
- Pattern: Serde-serialized JSON to app config directory
- Structs: `UserSettings`, `TranscriptionSettings`, `OutputSettings`, etc.

**History/Dictionary/Stats Modules:**
- Purpose: Local data storage (JSON files in app config directory)
- Pattern: Direct file I/O; CRUD operations
- Data: `TranscriptionEntry` (text, timestamp, word count), `DictionaryEntry`, statistics

**API Module (`api/`):**
- Purpose: Cloud authentication and external API communication
- Examples: `api/client.rs` for login/auth
- Pattern: HTTP client using reqwest; token management

## Entry Points

**Main Desktop Process:**
- Location: `/src-tauri/src/main.rs`
- Triggers: Application launch
- Responsibilities: Calls `lib::run()` to initialize Tauri application

**Tauri App Initialization:**
- Location: `/src-tauri/src/lib.rs` (exported `run()` function)
- Triggers: On first load
- Responsibilities:
  - Creates AppState with initial settings loaded
  - Sets up Tauri plugins (shell, dialog, fs, http, global-shortcut)
  - Registers all Tauri commands (start_recording, stop_recording, inject_text, etc.)
  - Creates dictation window (invisible overlay)
  - Converts dictation window to NSPanel on macOS for fullscreen support
  - Sets up hotkey registration
  - Creates tray icon with context menu
  - Handles window events (minimize, close, etc.)

**Frontend Entry Point:**
- Location: `/src/main.tsx`
- Triggers: After Vite server starts
- Responsibilities: Renders React root, mounts App component

**App Component:**
- Location: `/src/App.tsx`
- Triggers: On window load
- Responsibilities:
  - Determines window type from URL hash (dictation, settings, history, dashboard)
  - Sets up event listeners for all Tauri events
  - Manages recording state, audio level, processing state
  - Routes to appropriate component based on window type
  - Handles multi-monitor tracking for dictation bar

## Error Handling

**Strategy:** Layered error propagation with context-aware fallbacks

**Patterns:**

- **Backend:** Custom error types using `thiserror` (e.g., `InjectionError`, `HotkeyError`, `SettingsError`)
- **Commands:** Return `Result<T, String>` (serialize error message to frontend)
- **Frontend:** Catch invoke errors, display transient error messages, set timeout for auto-dismiss
- **Critical Failures:** Reset state (e.g., `reset_recording_state` command) to recover from stuck states
- **Audio Level Emitter:** Runs in separate thread; uses atomic flag for clean shutdown
- **Hotkey Re-registration:** Unregisters old hotkey, registers new one; on error, falls back to default key

## Cross-Cutting Concerns

**Logging:**
- Backend: `log` crate with `env_logger`
- Frontend: `console.log()`, `console.error()`
- Pattern: Eprintln! for critical recording logs; log::info/warn/error for Rust modules

**Validation:**
- Settings: Validated on update before persistence
- Commands: Input checked in handlers (e.g., is_recording state guard)
- Frontend: Form validation in Settings/Dashboard components

**Authentication:**
- API module provides login/auth token management
- Tokens stored in keyring (platform-native secure storage)
- Optional: not required for local transcription flow

**Permissions:**
- macOS: Accessibility permission required for text injection
- Hotkey registration: Global shortcut permission required by OS
- Audio capture: Microphone permission required by OS

**Multi-Window State:**
- Dictation window: Always on top, no decorations, transparent, follows mouse monitor
- Settings window: Separate window, lazy-created, reused if already open
- Dashboard window: Lazy-created, full CRUD UI for history/dictionary
- State synced via localStorage and backend file storage

---

*Architecture analysis: 2026-02-18*
