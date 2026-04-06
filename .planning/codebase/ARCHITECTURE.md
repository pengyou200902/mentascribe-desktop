# Architecture

**Analysis Date:** 2026-04-06

## High-Level Architecture

**Pattern:** Tauri v2 desktop application with a Rust backend and React/TypeScript frontend. Two-process model: a native Rust process handles all system-level operations (audio capture, speech-to-text, text injection, hotkeys) while a web-based frontend renders the UI in platform webviews.

**Key Characteristics:**
- Multi-window architecture: a tiny transparent overlay ("dictation pill") and a full dashboard window, both served from the same `index.html` with hash-based routing
- IPC via Tauri's `invoke()` / `#[tauri::command]` bridge -- frontend calls Rust commands, Rust emits events back
- Platform-native overlay behavior: macOS uses NSPanel (via tauri-nspanel plugin) for fullscreen overlay support; Windows uses WS_EX_NOACTIVATE Win32 styles; both ensure the overlay never steals focus from the target application
- Dual transcription engines: local Whisper (whisper-rs with CoreML/Metal/CUDA acceleration) and feature-gated Voxtral (custom C engine via FFI)

## Core Components

### 1. Audio Capture (`src-tauri/src/audio/`)

**Purpose:** Record microphone input, compute real-time audio levels, and produce 16kHz mono samples for speech-to-text engines.

**Key Files:**
- `src-tauri/src/audio/capture.rs` -- Core capture module using CPAL. Manages a dedicated audio thread that owns the CPAL stream. Uses `lazy_static!` globals (`AUDIO_BUFFER`, `WHISPER_BUFFER`, `AUDIO_THREAD`, `RESAMPLER_STATE`) for cross-thread state. Real-time resampling happens in the CPAL callback via rubato's `FastFixedIn` resampler with cubic interpolation. The callback uses `try_lock()` on all mutexes to avoid blocking the audio thread.
- `src-tauri/src/audio/vad.rs` -- Simple energy-based voice activity detector (RMS threshold). Used as a fallback; the primary VAD for Whisper streaming is Silero VAD (loaded from `ggml-silero-vad.bin`).
- `src-tauri/src/audio/mod.rs` -- Re-exports `AudioData` struct.

**Data Structure:**
```rust
pub struct AudioData {
    pub samples: Vec<f32>,           // Raw interleaved samples
    pub sample_rate: u32,            // Device sample rate (e.g., 48000)
    pub channels: u16,               // Device channels (e.g., 2)
    pub whisper_samples: Option<Vec<f32>>,  // Pre-processed 16kHz mono (zero post-stop latency)
}
```

**Flow:** `start_capture()` spawns an audio thread -> CPAL callback appends raw samples to `AUDIO_BUFFER` and incrementally resampled 16kHz mono to `WHISPER_BUFFER` -> `stop_capture()` sends stop signal, joins thread, flushes remaining resampler accumulator, returns `AudioData`.

### 2. Transcription (`src-tauri/src/transcription/`)

**Purpose:** Convert audio to text using local or cloud speech-to-text engines.

**Key Files:**
- `src-tauri/src/transcription/mod.rs` -- Module declarations and shared types (`ModelInfo`, `CoremlStatus`, `MetalStatus`, `TranscriptionResult`, `VoxtralStatus`).
- `src-tauri/src/transcription/whisper.rs` -- Primary transcription engine. Uses whisper-rs (C++ whisper.cpp bindings). Features: model cache (`MODEL_CACHE` static), pre-created WhisperState cache (`STATE_CACHE` for 50-200ms savings), VAD-triggered streaming transcription (`start_streaming`/`stop_streaming`), model download from HuggingFace, CoreML encoder download (macOS). Streaming uses a background thread that polls `snapshot_whisper_buffer()` and runs Silero VAD to detect speech segments, transcribing each independently.
- `src-tauri/src/transcription/cloud.rs` -- Cloud STT fallback (OpenAI, AWS, AssemblyAI). Currently stub implementations that return errors. Audio-to-WAV conversion is implemented.
- `src-tauri/src/transcription/voxtral.rs` -- Feature-gated (`#[cfg(feature = "voxtral")]`) alternative engine. Custom C-based Voxtral model with native streaming support.
- `src-tauri/src/transcription/voxtral_ffi.rs` -- FFI bindings to the C voxtral library compiled from `src-tauri/voxtral/`.

**Whisper Streaming Architecture:**
1. `start_streaming()` spawns a VAD monitor thread
2. Monitor thread polls `snapshot_whisper_buffer()` at intervals for new 16kHz mono samples
3. Silero VAD detects speech/silence boundaries
4. When silence is detected after speech, the accumulated speech segment is transcribed
5. Results are stored in `STREAMING_RESULTS` (a `Vec<String>`)
6. On `stop_streaming()`, accumulated results and consumed sample count are returned
7. `stop_recording()` in `lib.rs` combines streaming results with tail transcription (remaining un-transcribed audio)

**Model Management:**
- Models stored in `~/.mentascribe/models/`
- Downloaded from HuggingFace (`https://huggingface.co/ggerganov/whisper.cpp/resolve/main`)
- Supported sizes: tiny (75MB), base (142MB), small (466MB), medium (1.5GB), large-v3 (2.9GB), plus quantized/turbo variants
- CoreML encoder models (macOS only) downloaded as zip archives and extracted alongside GGML models
- Silero VAD model (`ggml-silero-vad.bin`) auto-downloaded on startup

### 3. Text Injection (`src-tauri/src/injection/mod.rs`)

**Purpose:** Insert transcribed text into the currently focused application without stealing focus.

**Platform Implementations (via `#[cfg(target_os)]` modules):**

**macOS (`platform` module):**
- Tier 1: Accessibility API (`AXUIElementSetAttributeValue` on `kAXSelectedTextAttribute`) -- instant, no clipboard pollution, works on any AX-compatible text field
- Tier 2: CGEvent Unicode typing -- sends keyboard events with `CGEventKeyboardSetUnicodeString`, chunks text into 20 UTF-16 code units per event (macOS hard limit), 2ms inter-chunk delay
- Tier 3: Clipboard paste -- copies to clipboard via `arboard`, simulates Cmd+V via CGEvent

**Windows (`platform` module):**
- Detects terminal applications (Windows Terminal, ConEmu, etc.) and uses clipboard paste for them (SendInput doesn't work in terminals)
- Non-terminals: clipboard paste with Ctrl+V via `SendInput` (clipboard-win crate for clipboard, windows crate for input simulation)
- WS_EX_NOACTIVATE on the dictation window ensures SendInput reaches the target app

**Linux (`platform` module):**
- X11 + XTest: uses `x11` crate with xtest feature for key simulation

**Fallback Strategy:** The `inject_text()` function checks settings for `insert_method` ("type" or "paste") and dispatches accordingly. On macOS, "type" attempts AX insert first, falls back to CGEvent typing, then clipboard paste.

### 4. Hotkey System (`src-tauri/src/hotkey/mod.rs`)

**Purpose:** Register system-wide global hotkeys for dictation control.

**Implementation:** Uses `tauri-plugin-global-shortcut` for cross-platform hotkey registration. Supports F1-F12 keys. On press/release, emits `hotkey-pressed`/`hotkey-released` events to the frontend. Supports two modes: "toggle" (press to start/stop) and "hold" (hold to record, release to stop).

### 5. Settings (`src-tauri/src/settings/mod.rs`)

**Purpose:** Persist user preferences as JSON.

**Storage:** `{config_dir}/mentascribe/settings.json` (platform config directory via `dirs` crate).

**Structure:**
```rust
pub struct UserSettings {
    pub transcription: TranscriptionSettings,  // provider, language, model_size, engine, voxtral_delay_ms
    pub cleanup: CleanupSettings,              // LLM-based text cleanup (provider, model, options)
    pub hotkey: HotkeySettings,                // key (F1-F12), mode (toggle/hold)
    pub output: OutputSettings,                // insert_method (type/paste), auto_capitalize
    pub widget: WidgetSettings,                // draggable, opacity
}
```

### 6. History & Stats (`src-tauri/src/history/mod.rs`, `src-tauri/src/stats/mod.rs`)

**Purpose:** Track transcription history and usage statistics locally.

**History:** JSON file at `{config_dir}/mentascribe/history.json`. Stores last 500 `TranscriptionEntry` records (id, text, word_count, duration_ms, timestamp, synced). Supports pagination, deletion, and sync marking.

**Stats:** JSON file at `{config_dir}/mentascribe/stats.json`. Tracks totals (transcriptions, words, audio seconds), daily breakdowns (last 30 days), and usage streak.

### 7. Dictionary (`src-tauri/src/dictionary/mod.rs`)

**Purpose:** User-defined word corrections and vocabulary biasing.

**Storage:** `{config_dir}/mentascribe/dictionary.json`. In-memory cache via `RwLock<Option<Vec<DictionaryEntry>>>` for concurrent read access during transcription.

**Two entry types:**
- **Auto-correct:** phrase != replacement -- applied as case-insensitive regex word-boundary replacements post-transcription
- **Vocabulary:** phrase == replacement -- injected into Whisper's `initial_prompt` to bias the decoder toward recognizing specific names/terms

### 8. API Client (`src-tauri/src/api/`)

**Purpose:** Communication with the MentaFlux cloud backend.

**Key Files:**
- `src-tauri/src/api/mod.rs` -- Type definitions (`AuthToken`, `UserInfo`)
- `src-tauri/src/api/client.rs` -- HTTP client using reqwest. Endpoints: login (`/v1/auth/login`), token refresh (`/v1/auth/refresh`), create transcription (`/v1/transcriptions`). Token storage via OS keychain (`keyring` crate).

**Base URL:** `https://api.voice.mentaflux.ai/v1`

### 9. Text Processing (`src-tauri/src/text/mod.rs`)

**Purpose:** Post-transcription text transformations. Currently implements sentence capitalization (capitalize after `.`, `!`, `?`).

### 10. Frontend UI (`src/`)

**Purpose:** React-based UI for dictation overlay and dashboard.

**Key Files:**
- `src/main.tsx` -- React entry point, renders `<App />` into `#root`
- `src/App.tsx` -- Root component. Determines window type from URL hash (`#dashboard` vs default dictation). Manages recording state, event listeners, hotkey handling, and multi-monitor tracking.
- `src/components/DictationBar.tsx` -- Transparent overlay pill widget. Shows waveform during recording, processing dots during transcription, error messages, and hotkey label. Supports dragging (native drag on macOS, Tauri drag on other platforms) and cursor proximity detection.
- `src/components/dashboard/Dashboard.tsx` -- Dashboard shell with sidebar navigation and theme provider. Pages: Home, History, Dictionary, Settings.
- `src/lib/store.ts` -- Zustand store for settings (load/update via IPC)
- `src/lib/historyStore.ts` -- Zustand store for history (pagination, CRUD via IPC)
- `src/lib/dictionaryStore.ts` -- Zustand store for dictionary (CRUD via IPC)
- `src/lib/statsStore.ts` -- Zustand store for stats
- `src/lib/theme.tsx` -- Theme context (light/dark/system) with localStorage persistence
- `src/lib/tauri.ts` -- Typed wrappers around `invoke()` calls
- `src/config/widget.ts` -- Centralized constants for waveform, timing, and UI behavior

## Data Flow

### Recording Flow (Happy Path)

```
User presses F6
  -> Tauri emits "hotkey-pressed" event
  -> App.tsx startRecording()
  -> invoke("start_recording")
  -> lib.rs: start_recording()
      1. Set is_recording = true
      2. audio::capture::start_capture() -- spawns CPAL audio thread
      3. Start streaming transcription (VAD monitor thread)
      4. Start audio level emitter thread (25ms polling)
  -> Frontend shows recording state (waveform visualization)

User presses F6 again (toggle mode)
  -> Tauri emits "hotkey-pressed" event
  -> App.tsx stopRecording()
  -> invoke("stop_recording")
  -> lib.rs: stop_recording()
      1. Stop audio level emitter
      2. Set is_recording = false
      3. Stop streaming monitor (joins thread, collects results)
      4. Stop audio capture (joins thread, flushes resampler)
      5. Trim whisper buffer to tail (un-transcribed audio only)
      6. Transcribe tail audio with Whisper/Voxtral
      7. Combine streaming prefix + tail transcription
      8. Apply auto-capitalize (text::process_text)
      9. Apply dictionary replacements (dictionary::apply_replacements)
     10. Record to history and stats
     11. Emit "transcription-complete" event
  -> Returns transcribed text to frontend
  -> App.tsx: invoke("inject_text", { text })
  -> lib.rs: inject_text() -> injection::inject_text()
      macOS: try AX insert -> CGEvent typing -> clipboard paste
      Windows: clipboard paste with SendInput Ctrl+V
  -> Text appears in the user's focused application
```

### Settings Change Flow

```
User changes setting in Dashboard
  -> SettingsPage invoke("update_settings", { newSettings })
  -> lib.rs: update_settings()
      1. Compare old vs new settings
      2. Persist to disk (settings::save_settings)
      3. Re-register hotkey if key changed
      4. Apply panel opacity if changed (macOS)
      5. Emit "settings-changed" to all windows
      6. Switch engines if engine changed (unload old, preload new)
      7. Preload new Whisper model if model_size changed
```

### State Management

**Backend (Rust):**
- `AppState` managed by Tauri: `is_recording` (Mutex<bool>), `settings` (Mutex<UserSettings>), `audio_level_emitter_running` (Arc<AtomicBool>)
- Audio capture state: global `lazy_static!` mutexes in `capture.rs`
- Whisper model cache: global `Lazy<Mutex<ModelCache>>` and `Lazy<Mutex<Option<CachedWhisperState>>>` in `whisper.rs`
- Dictionary cache: global `Lazy<RwLock<Option<Vec<DictionaryEntry>>>>` in `dictionary/mod.rs`

**Frontend (TypeScript):**
- Zustand stores: `useStore` (settings), `useHistoryStore` (history), `useDictionaryStore` (dictionary), `useStatsStore` (stats)
- React state in `App.tsx`: `isRecording`, `isProcessing`, `audioLevel`, `error`, `isDownloadingModel`, `isPreloading`
- Refs for stale-closure prevention: `isRecordingRef`, `isProcessingRef`, `settingsRef`

## Key Design Decisions

### 1. NSPanel for macOS Overlay

Regular NSWindow cannot appear above fullscreen applications on macOS (Apple-enforced since Big Sur). The dictation window is converted to an NSPanel (via `tauri-nspanel` plugin) with `NSWindowCollectionBehaviorFullScreenAuxiliary` and `NSNonactivatingPanelMask`. This is the only way to provide dictation overlay in fullscreen apps.

### 2. Real-Time Audio Resampling

Rather than resampling the entire recording after stop, the CPAL callback incrementally resamples to 16kHz mono during recording. This eliminates post-stop latency (which was noticeable for longer recordings). The resampler state is shared via `Arc<Mutex<>>` with `try_lock()` in the callback to avoid blocking the audio thread.

### 3. Streaming Transcription with VAD

A background thread monitors the whisper buffer during recording, using Silero VAD to detect speech segments. Completed utterances are transcribed immediately, so by the time the user stops recording, most of the text is already transcribed. Only the tail (partial final utterance) needs processing on stop.

### 4. Dual Transcription Engines

Whisper is the default engine (well-tested, wide model support). Voxtral is a feature-gated alternative compiled from custom C code (in `src-tauri/voxtral/`), offering native streaming and potentially different performance characteristics. The engine switch happens at the settings level and involves model unloading/preloading.

### 5. Multi-Tier Text Injection (macOS)

Three injection strategies with automatic fallback: AX API (instant, no clipboard), CGEvent typing (works everywhere but slower), clipboard paste (most compatible but pollutes clipboard). This handles the wide variety of macOS text input contexts.

### 6. Multi-Window with Hash Routing

Both the dictation overlay and dashboard share the same `index.html` entry point. Window type is determined by URL hash: `#dashboard` routes to `<Dashboard />`, default routes to `<DictationBar />`. This avoids duplicate build artifacts while allowing different window configurations in `tauri.conf.json`.

### 7. Model Preloading on Startup

On app launch, the configured speech model is loaded in a background thread. This ensures the first dictation is fast (otherwise model loading adds 2-10 seconds of latency). The frontend shows a preloading indicator.

### 8. Focus Preservation on Windows

The dictation window on Windows uses `WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW` extended styles and `ShowWindow(SW_SHOWNOACTIVATE)` to prevent focus theft. This ensures `SendInput` keystrokes always reach the user's target application, not the MentaScribe overlay.

## Component Interactions

### IPC Contract (Frontend -> Backend)

All frontend-to-backend calls use `invoke()`. The full command list registered in `lib.rs`:

**Recording:**
- `start_recording` -- Start audio capture + streaming transcription
- `stop_recording` -> `String` -- Stop capture, transcribe, return text
- `inject_text(text)` -- Inject text into focused app
- `reset_recording_state` -- Emergency state reset

**Settings:**
- `get_settings` -> `UserSettings`
- `update_settings(newSettings)` -- Persist and apply

**Auth:**
- `login(email, password)` -> `AuthToken`

**Models:**
- `download_model(size)` -- Download Whisper GGML model
- `get_available_models` -> `Vec<ModelInfo>`
- `get_coreml_status` -> `CoremlStatus`
- `get_metal_status` -> `MetalStatus`
- `download_coreml_model(size)` -- Download CoreML encoder
- `delete_model(size)` / `delete_coreml_model(size)`

**Stats/History/Dictionary:**
- `get_stats` / `record_transcription_stats`
- `get_history(limit, offset)` / `get_history_entry(id)` / `delete_history_entry(id)` / `clear_history` / `get_history_count`
- `get_dictionary` / `add_dictionary_entry(phrase, replacement)` / `update_dictionary_entry(id, phrase, replacement, enabled)` / `remove_dictionary_entry(id)`

**Window Management:**
- `reposition_to_mouse_monitor` -> `bool` -- Move to cursor's monitor
- `start_native_drag` -- Begin NSPanel/window drag
- `resize_pill(width, height)` -- Resize dictation window
- `is_cursor_over_pill` -> `bool` -- Native cursor proximity check

**Voxtral (feature-gated):**
- `get_voxtral_status` / `get_voxtral_models` / `download_voxtral_model` / `delete_voxtral_model`

**Debug:**
- `frontend_log(msg)` -- Forward frontend logs to Rust stderr

### Event Contract (Backend -> Frontend)

Backend emits events via `app.emit()`:

- `hotkey-pressed` / `hotkey-released` -- Global hotkey state changes
- `audio-level` (`f32`) -- Real-time mic level (25ms intervals)
- `transcription-processing` / `transcription-complete` (`String`) -- Transcription lifecycle
- `model-preload-start` (`String`) / `model-preload-complete` / `model-preload-error` -- Model loading status
- `model-needs-download` (`String`) -- Model not found on startup
- `download-progress` (`{ model_type, model_id, percent }`) -- Download progress
- `settings-changed` (`UserSettings`) -- Settings updated
- `navigate-to-page` (`String`) -- Tray menu navigation

### Dependency Graph (Rust Modules)

```
lib.rs (Tauri setup, commands, AppState)
  |-- audio::capture       (CPAL, rubato resampling)
  |-- audio::vad           (energy-based VAD)
  |-- transcription::whisper  (whisper-rs, model mgmt, VAD streaming)
  |-- transcription::cloud    (reqwest, stub implementations)
  |-- transcription::voxtral  (FFI to C voxtral, feature-gated)
  |-- injection            (platform-specific text injection)
  |-- hotkey               (tauri-plugin-global-shortcut)
  |-- settings             (serde_json, dirs)
  |-- text                 (post-processing)
  |-- history              (chrono, uuid, JSON persistence)
  |-- dictionary           (regex, RwLock cache, JSON persistence)
  |-- stats                (chrono, JSON persistence)
  |-- api::client          (reqwest, keyring)
```

### Platform-Specific Code Map

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Overlay | NSPanel (tauri-nspanel) | WS_EX_NOACTIVATE (windows crate) | None (standard window) |
| Focus preservation | NSNonactivatingPanelMask | SW_SHOWNOACTIVATE + SWP_NOACTIVATE | N/A |
| Text injection | AX API / CGEvent / clipboard | Clipboard + SendInput | X11 XTest |
| GPU acceleration | CoreML + Metal (whisper-rs) | CUDA (whisper-rs) | CPU only |
| Dragging | Native NSEvent monitors | Tauri start_dragging() | Tauri start_dragging() |
| Positioning | Native AppKit APIs | Tauri monitor APIs | Tauri monitor APIs |
| Voxtral GPU | Metal (Accelerate + Metal shaders) | CPU only | OpenBLAS |

## Error Handling

**Strategy:** Errors propagate through `thiserror`-derived error types per module, then convert to `String` at the Tauri command boundary via `.map_err(|e| e.to_string())`. The frontend displays errors in the dictation pill for a configurable timeout.

**Patterns:**
- Each module defines its own error enum with `#[derive(Error)]` from `thiserror`
- `anyhow` is in dependencies but most code uses module-specific errors
- Audio capture errors are non-fatal if recording was started (capture state is reset)
- History/stats recording failures are logged but don't fail the transcription
- Model not found triggers auto-download flow via frontend event handling

**Recovery:**
- `reset_recording_state` command provides emergency state reset for stuck recording
- `audio::capture::reset_state()` clears all globals (buffers, flags, thread handles)
- Resampler failures fall back to post-stop processing or linear interpolation

---

*Architecture analysis: 2026-04-06*
