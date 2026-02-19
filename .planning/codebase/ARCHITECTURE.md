# Architecture

**Analysis Date:** 2026-02-19

## Pattern Overview

**Overall:** Tauri v2 desktop application with fullscreen overlay UI pattern

**Key Characteristics:**
- Dual-window architecture: lightweight dictation overlay (NSPanel on macOS) + dashboard management window
- Rust backend for all system interaction (audio, text injection, hotkeys) and data persistence
- React/TypeScript frontend for UI with Zustand for state management
- Event-driven communication via Tauri invoke/listen system
- Mixed-DPI multi-monitor aware positioning with native AppKit coordination on macOS

## Layers

**UI Presentation Layer:**
- Purpose: Render user interfaces and handle user interactions
- Location: `src/components/`, `src/App.tsx`
- Contains: React components (DictationBar, Dashboard, Settings, History)
- Depends on: Tauri API, store (Zustand), frontend utilities
- Used by: Main window rendering pipeline

**State Management Layer:**
- Purpose: Manage frontend state for settings, history, dictionary, stats
- Location: `src/lib/store.ts`, `src/lib/historyStore.ts`, `src/lib/dictionaryStore.ts`, `src/lib/statsStore.ts`
- Contains: Zustand stores with async Tauri invocation
- Depends on: Tauri `invoke()` for backend communication
- Used by: React components via custom hooks

**Tauri Command Interface:**
- Purpose: Serialize/validate frontend requests to Rust backend
- Location: `src-tauri/src/lib.rs` (command definitions via `#[tauri::command]`)
- Contains: ~30 commands for recording, settings, history, dictionary, model management
- Depends on: Backend modules, AppState management
- Used by: Frontend invoke() calls

**Rust Backend Layers:**

**System Integration Layer:**
- Purpose: Low-level OS interaction
- Location: `src-tauri/src/audio/capture.rs`, `src-tauri/src/injection/`, `src-tauri/src/hotkey/`
- Contains: Audio capture (CPAL), text injection (enigo), keyboard shortcuts (global-hotkey)
- Depends on: External crates (cpal, enigo, global-hotkey)
- Used by: Recording pipeline, hotkey dispatcher

**Transcription Engine:**
- Purpose: Speech-to-text conversion with model management
- Location: `src-tauri/src/transcription/whisper.rs`, `src-tauri/src/transcription/cloud.rs`
- Contains: Whisper model loading/caching, GGML + CoreML support, model download/deletion
- Depends on: whisper-rs, reqwest (HTTP downloads), model file system
- Used by: stop_recording command

**Data Layer:**
- Purpose: Persistent storage of settings, history, dictionary, stats
- Location: `src-tauri/src/settings/`, `src-tauri/src/history/`, `src-tauri/src/dictionary/`, `src-tauri/src/stats/`
- Contains: JSON/file-based persistence with serde serialization
- Depends on: dirs crate for path resolution, standard file I/O
- Used by: Commands, startup initialization

**Text Processing:**
- Purpose: Post-transcription text cleanup and customization
- Location: `src-tauri/src/text/`, `src-tauri/src/dictionary/`
- Contains: Auto-capitalization, dictionary replacements
- Depends on: regex crate
- Used by: stop_recording command

**Application State:**
- Purpose: Runtime state management for recording/processing status
- Location: `src-tauri/src/lib.rs` (AppState struct)
- Contains: Recording flag, settings Mutex, audio level emitter flag
- Depends on: Mutex/Arc for thread-safe access
- Used by: All Tauri commands

## Data Flow

**Dictation Recording Flow:**

1. User presses hotkey → `hotkey-pressed` event fired by Tauri hotkey plugin
2. Frontend App.tsx listens → calls `startRecording()` invoke
3. Rust `start_recording()` → initializes audio capture via CPAL, spawns audio level emitter thread
4. Audio level emitter thread emits `audio-level` event every 25ms → DictationBar animates waveform
5. User releases hotkey → `hotkey-released` event → calls `stopRecording()` invoke
6. Rust `stop_recording()`:
   - Stops audio capture, collects AudioData
   - Loads Whisper model (with caching to avoid reloads)
   - Transcribes via whisper-rs (optionally using CoreML acceleration)
   - Applies dictionary replacements and auto-capitalization
   - Records stats and history entries
   - Emits `transcription-complete` event with text
7. Frontend receives event → calls `inject_text()` invoke to type/paste result
8. Text injected via enigo (platform-specific input simulation)
9. History updated via localStorage (also persisted to Rust backend)

**Settings Update Flow:**

1. User modifies setting in Settings page component
2. Calls `updateSettings()` from Zustand store → `invoke('update_settings')`
3. Rust `update_settings()`:
   - Acquires AppState.settings Mutex
   - Persists to `~/.config/mentascribe/settings.json`
   - Re-registers hotkey if key changed
   - Applies opacity to NSPanel if changed
   - Emits `settings-changed` event to all windows
4. Frontend receives event → `loadSettings()` reloads state
5. Dictation window monitors `draggable` setting and adjusts behavior

**Window Positioning Flow (macOS):**

1. Frontend detects mouse moved to different monitor via 150ms poll
2. Calls `reposition_to_mouse_monitor()` invoke
3. Rust uses native AppKit APIs (bypassing Tauri's buggy tao layer):
   - Calls `NSEvent.mouseLocation` for cursor position (AppKit coordinates)
   - Iterates `NSScreen.screens` to find screen containing cursor
   - Calculates bottom-center position within visible frame
   - Calls `setFrameOrigin` to move NSPanel
4. Returns bool indicating if repositioning occurred

**Dashboard Navigation Flow:**

1. URL hash determines initial page (e.g., `#dashboard/settings`)
2. Dashboard.tsx parses hash, renders appropriate page component
3. Tray menu can emit `navigate-to-page` event → triggers page change
4. Each page (HomePage, HistoryPage, DictionaryPage, SettingsPage) manages own data loading

## Key Abstractions

**Recording Pipeline:**
- Purpose: Abstract the full recording → transcription → injection flow
- Examples: `startRecording()`, `stopRecording()` commands encapsulate entire process
- Pattern: Imperative commands with event notifications for progress

**Settings Configuration:**
- Purpose: Provide single source of truth for user preferences
- Examples: `UserSettings` struct in Rust, `useStore()` hook in frontend
- Pattern: Mutex-protected settings on backend, Zustand store on frontend, bi-directional sync via events

**Model Management:**
- Purpose: Lazy-load, cache, and manage Whisper model lifecycle
- Examples: `MODEL_CACHE` in whisper.rs, model info queries, download progress
- Pattern: Lazy initialization, in-memory cache for current model, async downloads with progress callbacks

**Audio Level Streaming:**
- Purpose: Provide real-time audio visualization feedback during recording
- Examples: Spawned thread in `start_recording()` emits level every 25ms
- Pattern: Separate thread pushing data via event emissions, frontend renders waveform

**Window Coordination:**
- Purpose: Keep dictation panel positioned correctly across multi-monitor moves
- Examples: 150ms poll in App.tsx → `reposition_to_mouse_monitor()`, native drag handling
- Pattern: Polling from frontend, native implementation on macOS, fallback on other platforms

## Entry Points

**Frontend Entry Point:**
- Location: `src/main.tsx`
- Triggers: Application start
- Responsibilities: React root initialization, renders App.tsx

**Main Application Component:**
- Location: `src/App.tsx`
- Triggers: Frontend initialization
- Responsibilities: Determines window type (dictation/dashboard), sets up event listeners, manages recording/processing state, handles hotkey events, polls for monitor changes

**Rust Application Entry:**
- Location: `src-tauri/src/main.rs`
- Triggers: Desktop application launch
- Responsibilities: Minimal — just calls `run()` from lib.rs

**Rust Application Builder:**
- Location: `src-tauri/src/lib.rs` (`run()` function)
- Triggers: Application initialization
- Responsibilities: Loads settings, initializes plugins, sets up state, creates windows, configures hotkeys, preloads model, sets up tray menu, registers command handlers

**Dictionary Window:**
- Location: `src/components/DictationBar.tsx`
- Triggers: App.tsx renders when windowType === 'dictation'
- Responsibilities: Renders overlay UI, handles click to record/stop, displays audio waveform, shows status/errors, manages draggable state

**Dashboard Window:**
- Location: `src/components/dashboard/Dashboard.tsx`
- Triggers: Tray click or menu selection opens dashboard
- Responsibilities: Navigation between pages, page rendering, event listener setup

## Error Handling

**Strategy:** Two-tier error handling (frontend and Rust)

**Patterns:**

- **Rust Command Errors:** Commands return `Result<T, String>` via Tauri, serialized to frontend
  - Examples: `start_recording()` returns `Err("Already recording")`, `stop_recording()` returns transcribed text or error

- **Frontend Error Display:** Errors stored in App.tsx state, displayed in DictationBar
  - Example: `setError()` shows error for 5 seconds, auto-clears
  - Special handling for "Model not found" → triggers auto-download

- **Recording State Recovery:** `reset_recording_state()` command unlocks stuck states
  - Example: If transcription hangs, user can invoke reset to recover

- **Async Error Handling:** Await points wrapped in try/catch with user feedback
  - Audio capture, transcription, text injection failures all caught and reported

## Cross-Cutting Concerns

**Logging:**
- Rust: env_logger with eprintln! for debugging (prefixed with [module] tags)
- Frontend: console logging, plus `frontend_log()` command to echo frontend events to Rust terminal
- Used to track: audio capture state, transcription pipeline, settings changes, window positioning

**Validation:**
- Frontend: Basic input validation in Settings page (e.g., opacity 0.2–1.0)
- Rust: Serde deserialization validates JSON structure, type checking at compile time

**Authentication:**
- Placeholder: `login()` command in lib.rs (currently stub)
- Infrastructure: API client module exists (`src-tauri/src/api/`) but not integrated

**Multi-Monitor Awareness:**
- Tauri coordinate bug workaround: Native AppKit APIs on macOS for all positioning
- Polling mechanism: 150ms poll in App.tsx checks for monitor changes
- Automatic repositioning when cursor moves to different monitor (configurable via `draggable` setting)

---

*Architecture analysis: 2026-02-19*
