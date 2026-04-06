# External Integrations

**Analysis Date:** 2026-04-06

## APIs & External Services

### MentaFlux Cloud API
- **Base URL:** `https://api.voice.mentaflux.ai/v1`
- **Defined in:** `src-tauri/src/api/client.rs` (line 5)
- **CSP allowlisted in:** `src-tauri/tauri.conf.json` (`connect-src`)
- **Auth:** Email/password login -> JWT access token + refresh token
- **Endpoints used:**
  - `POST /auth/login` - Email/password authentication, returns `accessToken`, `refreshToken`, `expiresIn`, `user`
  - `POST /auth/refresh` - Refresh expired access token using refresh token
  - `POST /transcriptions` - Upload transcription (raw text, cleaned text, duration, language), requires Bearer auth
- **Client:** `reqwest` 0.11 (Rust async HTTP client)
- **Token storage:** OS keychain via `keyring` crate, stored under service `"mentascribe"` key `"tokens"` as JSON (`src-tauri/src/api/client.rs` lines 167-216)
- **Frontend wrapper:** `src/lib/tauri.ts` exposes `login()` via Tauri `invoke`

### Hugging Face Model Downloads
- **Whisper models:** `https://huggingface.co/ggerganov/whisper.cpp/resolve/main`
  - Defined in: `src-tauri/src/transcription/whisper.rs` (line 52)
  - Downloads GGML model files (`ggml-tiny.bin`, `ggml-base.bin`, `ggml-small.bin`, `ggml-medium.bin`, `ggml-large-v3.bin`)
  - Also downloads Silero VAD model: `ggml-silero-vad.bin`
  - CoreML encoder models downloaded separately as `.mlmodelc` archives
- **Voxtral model:** `https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602/resolve/main`
  - Defined in: `src-tauri/src/transcription/voxtral.rs` (lines 42-43)
  - Downloads: `consolidated.safetensors` (~8.9 GB), `tekken.json` (~15 MB), `params.json` (~500 B)
- **Auth:** None (public model downloads)
- **Progress:** Download progress emitted to frontend via Tauri events (`download-progress`)

### Cloud STT Providers (Stubbed)
- **Defined in:** `src-tauri/src/transcription/cloud.rs`
- **Providers declared but NOT implemented:**
  - `openai` - OpenAI Whisper API (TODO at line 59)
  - `aws` - AWS Transcribe (TODO at line 73)
  - `assemblyai` - AssemblyAI (TODO at line 84)
- All three return `CloudError::ApiError("...not yet implemented")` when called
- Audio conversion to WAV format IS implemented (`audio_to_wav` function, lines 91-118)

### LLM Text Cleanup Providers (Configured, Not Invoked)
- **Settings defined in:** `src-tauri/src/settings/mod.rs` (`CleanupSettings` struct, lines 30-40)
- **Provider options:** `"openai"`, `"anthropic"`, `"ollama"`, `"custom"`
- **Configurable fields:** `model`, `custom_endpoint`, `api_key`
- **Features:** `remove_filler`, `add_punctuation`, `format_paragraphs`
- **Status:** Settings UI exists but no actual LLM API calls are implemented in the backend

### Google Fonts CDN
- **URL:** `https://fonts.googleapis.com` / `https://fonts.gstatic.com`
- **Used in:** `src/styles/globals.css` (line 2) and `index.html` (lines 9-10)
- **Fonts loaded:** DM Sans (variable weight), JetBrains Mono (400, 500, 600)
- **Preconnect hints** in `index.html` for faster font loading

## System Integrations

### Microphone / Audio Input
- **Library:** `cpal` 0.15 (`src-tauri/src/audio/capture.rs`)
- **Access:** Default input device via `cpal::default_host().default_input_device()`
- **Processing pipeline:** Device sample rate -> mono downmix -> `rubato` resampling to 16kHz -> f32 sample buffer
- **Real-time resampling:** Performed incrementally in CPAL audio callback to minimize latency on recording stop
- **macOS permission:** `NSMicrophoneUsageDescription` in `src-tauri/Info.plist`

### Keyboard / Text Injection
Multi-tier injection system with platform-specific implementations in `src-tauri/src/injection/mod.rs`:

**macOS (`#[cfg(target_os = "macos")]`):**
1. **Tier 1 - Accessibility API:** `AXUIElementSetAttributeValue(kAXSelectedText)` - Instant insertion if target supports it
2. **Tier 2 - CGEvent Unicode:** `CGEventKeyboardSetUnicodeString` - Chunked typing (20 UTF-16 units/event, 2ms delay)
3. **Tier 3 - Clipboard paste:** Save clipboard -> set text + transient marker -> `Cmd+V` -> restore clipboard
- Terminal app detection via `NSWorkspace.frontmostApplication.bundleIdentifier` (falls back to clipboard for terminals)

**Windows (`#[cfg(target_os = "windows")]`):**
1. **Tier 1 - SendInput Unicode:** `KEYEVENTF_UNICODE` batched events (up to 10,000 per call), with mouse-move flush for Win11 Notepad bug
2. **Tier 2 - Clipboard paste:** `clipboard-win` to set text -> `SendInput(Ctrl+V)` -> restore
- Terminal app detection via `QueryFullProcessImageNameW` on foreground window process
- Modifier key clearing (`VK_SHIFT`, `VK_CONTROL`, etc.) before injection to prevent hotkey leakage

**Linux (`#[cfg(target_os = "linux")]`):**
- X11 XTest extension for keyboard simulation
- Wayland explicitly NOT supported (returns `WaylandNotSupported` error)

### Window Management

**macOS NSPanel System:**
- Dictation window converted to NSPanel via `tauri-nspanel` for fullscreen overlay support
- `NSNonactivatingPanelMask` prevents focus stealing
- `NSWindowCollectionBehaviorFullScreenAuxiliary` allows appearing over fullscreen apps
- Window level 25 (`NSMainMenuWindowLevel + 1`)
- Native dragging via NSEvent local/global monitors (`start_native_drag` command in `src-tauri/src/lib.rs`)
- `resize_pill` command uses `setFrame:display:` for bottom-anchored resize
- Native cursor proximity detection via `NSEvent.mouseLocation`
- Opacity control via `[NSWindow setAlphaValue:]`

**Windows Window Management:**
- `WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW` applied via `SetWindowLongPtrW`
- Focus-safe show via `ShowWindow(SW_SHOWNOACTIVATE)` + `SetWindowPos(SWP_NOACTIVATE)`
- Window found by title via `FindWindowW(w!("MentaScribe"))`

### Global Hotkey
- **Library:** `tauri-plugin-global-shortcut` 2 + `global-hotkey` 0.5
- **Defined in:** `src-tauri/src/hotkey/mod.rs`
- **Supported keys:** F1-F12 (default: F6)
- **Modes:** Toggle (press to start/stop) or Hold (hold to record, release to stop)
- **Events emitted:** `hotkey-pressed`, `hotkey-released` via Tauri event system

### System Tray
- Built with Tauri's `TrayIconBuilder` in `src-tauri/src/lib.rs` (line 1661)
- Left-click: Opens dashboard window
- Right-click: Menu with Settings, History, Show/Hide Widget, Quit
- Icon: App default icon

## Data Storage

### Local File Storage
All data stored as JSON files in platform-appropriate config directories via the `dirs` crate:

| Data | Path | File |
|------|------|------|
| Settings | `dirs::config_dir()/mentascribe/` | `settings.json` |
| Stats | `dirs::config_dir()/mentascribe/` | `stats.json` |
| History | `dirs::config_dir()/mentascribe/` | `history.json` |
| Dictionary | `dirs::config_dir()/mentascribe/` | `dictionary.json` |
| Whisper models | `dirs::home_dir()/.mentascribe/models/` | `ggml-*.bin` |
| Voxtral model | `dirs::home_dir()/.mentascribe/models/voxtral-mini-4b/` | `consolidated.safetensors`, `tekken.json`, `params.json` |
| CoreML models | `dirs::home_dir()/.mentascribe/models/` | `ggml-*-encoder.mlmodelc/` |

**Platform paths:**
- macOS: `~/Library/Application Support/mentascribe/`
- Windows: `%APPDATA%\mentascribe\`
- Linux: `~/.config/mentascribe/`

### Frontend localStorage
- `transcription-history` - Last 100 transcription entries (legacy, duplicated by backend history)
- `mentascribe-theme` - Theme preference (`light`, `dark`, `system`)

### OS Keychain
- **Library:** `keyring` 2
- **Service:** `"mentascribe"`, key `"tokens"`
- **Stored data:** JSON with `access_token` and `refresh_token`
- **Backend:** macOS Keychain, Windows Credential Manager, Linux Secret Service

### Databases
- **None.** All persistence is file-based JSON.

### Caching
- **Whisper model cache:** `MODEL_CACHE` static in `src-tauri/src/transcription/whisper.rs` - Holds loaded `WhisperContext` (Arc-wrapped) to avoid reloading between transcriptions
- **Whisper state cache:** `STATE_CACHE` static - Pre-created `WhisperState` for next transcription (~50-200ms savings, 200-400MB allocation)
- **Dictionary cache:** `DICTIONARY_CACHE` RwLock in `src-tauri/src/dictionary/mod.rs` - In-memory cache of dictionary entries, loaded once, updated on mutations

## Communication Protocols

### Tauri IPC (Frontend <-> Backend)
- **Mechanism:** Tauri `invoke` (frontend -> backend) and `emit`/`listen` (bidirectional events)
- **Serialization:** JSON via serde
- **Commands registered:** 30+ commands in `src-tauri/src/lib.rs` (lines 1699-1740)

**Key command groups:**
- Recording: `start_recording`, `stop_recording`, `inject_text`, `reset_recording_state`
- Settings: `get_settings`, `update_settings`
- Auth: `login`
- Models: `download_model`, `get_available_models`, `get_coreml_status`, `get_metal_status`, `download_coreml_model`, `delete_model`, `delete_coreml_model`
- Stats: `get_stats`, `record_transcription_stats`
- History: `get_history`, `get_history_entry`, `delete_history_entry`, `clear_history`, `get_history_count`
- Dictionary: `get_dictionary`, `add_dictionary_entry`, `update_dictionary_entry`, `remove_dictionary_entry`
- Window: `reposition_to_mouse_monitor`, `start_native_drag`, `resize_pill`, `is_cursor_over_pill`
- Voxtral: `get_voxtral_status`, `get_voxtral_models`, `download_voxtral_model`, `delete_voxtral_model`
- Debug: `frontend_log`

**Key events (backend -> frontend):**
- `hotkey-pressed`, `hotkey-released` - Global shortcut activation
- `audio-level` (f32) - Real-time microphone level for waveform visualization
- `transcription-processing`, `transcription-complete` - Transcription lifecycle
- `model-preload-start`, `model-preload-complete`, `model-preload-error` - Background model loading
- `model-needs-download` - Triggers auto-download of missing speech model
- `download-progress` - Model download percentage with model type/ID
- `settings-changed` - Cross-window settings synchronization
- `navigate-to-page` - Dashboard page navigation from tray menu

### HTTP (Backend -> External)
- **Library:** `reqwest` 0.11
- **Protocols:** HTTPS only
- **Endpoints:** MentaFlux API (`api.voice.mentaflux.ai`), Hugging Face (`huggingface.co`)
- **Auth methods:** Bearer token (MentaFlux API), none (Hugging Face public models)

### Voxtral FFI (Rust <-> C)
- **Defined in:** `src-tauri/src/transcription/voxtral_ffi.rs`
- **Mechanism:** `extern "C"` function bindings to `libvoxtral` (compiled in-tree)
- **Key functions:** `vox_load`, `vox_free`, `vox_stream_init`, `vox_stream_feed`, `vox_stream_finish`, `vox_stream_get`, `vox_stream_flush`
- **Thread safety:** `VoxtralContext` wrapper with `Arc<Mutex<>>`, marked `Send + Sync`
- **macOS GPU:** `vox_metal_init()`, `vox_metal_available()` for Metal initialization

### Multi-Window Communication
- Two Tauri windows: `dictation` (overlay pill) and `dashboard` (settings/history/stats)
- Communication via Tauri event system (`emit` broadcasts to all windows)
- Hotkey events only processed by dictation window (dashboard ignores via `windowType` check in `src/App.tsx`)
- Settings changes broadcast via `settings-changed` event for cross-window sync
- Window type determined by URL hash (`#dashboard` vs default)

## Webhooks & Callbacks

**Incoming:** None

**Outgoing:** None

---

*Integration audit: 2026-04-06*
