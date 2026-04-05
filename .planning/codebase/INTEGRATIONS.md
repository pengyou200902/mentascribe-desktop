# External Integrations

**Analysis Date:** 2026-04-05

## APIs & External Services

**MentaFlux Backend API:**
- Service: MentaFlux voice-to-text cloud platform
- Base URL: `https://api.voice.mentaflux.ai/v1`
- Client: Custom HTTP client via `reqwest` crate
- Implementation: `src-tauri/src/api/client.rs`
- Auth: Bearer token (access_token obtained via login)
- Endpoints:
  - `POST /auth/login` — Email/password authentication, returns access_token + refresh_token + user info
  - `POST /auth/refresh` — Refresh expired access_token using refresh_token
  - `POST /transcriptions` — Store transcription record (raw_text, cleaned_text, duration_ms, language)
- CSP allowlisted in `src-tauri/tauri.conf.json` under `app.security.csp`

**Hugging Face CDN (Whisper Models):**
- Service: Model file downloads for local speech-to-text
- Base URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main`
- Implementation: `src-tauri/src/transcription/whisper.rs`
- Auth: None (public downloads)
- Files downloaded:
  - `ggml-tiny.bin`, `ggml-base.bin`, `ggml-small.bin`, `ggml-medium.bin`, `ggml-large.bin`
  - `ggml-silero-vad.bin` (Voice Activity Detection model)
  - CoreML model archives (macOS only)
- Download destination: `~/.mentascribe/models/`
- Progress events emitted to frontend via `download-progress` Tauri event

**Hugging Face CDN (Voxtral Model):**
- Service: Model file downloads for Voxtral STT engine (feature-gated)
- Base URL: `https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602/resolve/main`
- Implementation: `src-tauri/src/transcription/voxtral.rs`
- Auth: None (public downloads)
- Files: `consolidated.safetensors` (~8.9GB), `tekken.json` (~15MB), `params.json` (~500B)
- Download destination: `~/.mentascribe/models/voxtral-mini-4b/`

**Cloud STT Providers (Stubbed - Not Implemented):**
- Infrastructure exists in `src-tauri/src/transcription/cloud.rs`
- All three return errors with "not yet implemented" messages:
  - OpenAI Whisper API — `transcribe_openai()` (TODO)
  - AWS Transcribe — `transcribe_aws()` (TODO)
  - AssemblyAI — `transcribe_assemblyai()` (TODO)
- Configuration field: `settings.transcription.cloud_provider` ("openai" | "aws" | "assemblyai")

**Google Fonts CDN:**
- Service: Web font loading
- Implementation: CSS `@import` in `src/styles/globals.css`
- Fonts: DM Sans (variable weight), JetBrains Mono (400/500/600)
- URL: `https://fonts.googleapis.com/css2?family=DM+Sans:...`

## Data Storage

**Databases:**
- None. All persistence uses local JSON files.

**Local JSON File Storage:**

| Data | File Path | Managed By |
|------|-----------|-----------|
| User settings | `~/.config/mentascribe/settings.json` | `src-tauri/src/settings/mod.rs` |
| Transcription history | `~/.config/mentascribe/history.json` | `src-tauri/src/history/mod.rs` |
| Usage statistics | `~/.config/mentascribe/stats.json` | `src-tauri/src/stats/mod.rs` |
| Custom dictionary | `~/.config/mentascribe/dictionary.json` | `src-tauri/src/dictionary/mod.rs` |

All paths resolved via `dirs::config_dir()` with fallback to `~/.config/`.

**Model File Storage:**

| Data | File Path | Managed By |
|------|-----------|-----------|
| Whisper GGML models | `~/.mentascribe/models/ggml-*.bin` | `src-tauri/src/transcription/whisper.rs` |
| Silero VAD model | `~/.mentascribe/models/ggml-silero-vad.bin` | `src-tauri/src/transcription/whisper.rs` |
| CoreML model archives | `~/.mentascribe/models/ggml-*-encoder.mlmodelc/` | `src-tauri/src/transcription/whisper.rs` |
| Voxtral model files | `~/.mentascribe/models/voxtral-mini-4b/` | `src-tauri/src/transcription/voxtral.rs` |

Model directory resolved via `dirs::home_dir()` -> `~/.mentascribe/models/`.

**Frontend Local Storage:**
- `transcription-history` key in `localStorage` — lightweight recent history (max 100 entries) in dictation window (`src/App.tsx`)
- `mentascribe-theme` key in `localStorage` — theme preference ("light" | "dark" | "system") (`src/lib/theme.tsx`)

**In-Memory Caches (Rust Backend):**
- `MODEL_CACHE` — Whisper model context, persists across transcriptions (`src-tauri/src/transcription/whisper.rs`)
- `STATE_CACHE` — Pre-allocated WhisperState for ~50-200ms speedup (`src-tauri/src/transcription/whisper.rs`)
- `DICTIONARY_CACHE` — RwLock-protected dictionary entries, loaded from disk on first access (`src-tauri/src/dictionary/mod.rs`)
- Voxtral model context — global Mutex-guarded context (`src-tauri/src/transcription/voxtral.rs`)

## Authentication & Identity

**Auth Provider:** Custom MentaFlux API (email/password login)

**Implementation:** `src-tauri/src/api/client.rs`

**Flow:**
1. User submits email + password via frontend
2. Frontend calls `invoke('login', { email, password })`
3. Rust backend POSTs to `https://api.voice.mentaflux.ai/v1/auth/login`
4. Response contains: `access_token`, `refresh_token`, `expires_in`, `user` (id, email, name, avatar_url)
5. Tokens stored in OS keychain via `keyring` crate (entry: `mentascribe`/`tokens`)
6. Token refresh via `POST /auth/refresh` with refresh_token

**Secure Token Storage:**
- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (libsecret)
- Functions: `store_tokens()`, `get_stored_tokens()`, `clear_tokens()` in `src-tauri/src/api/client.rs`

**Frontend Types:** `src/lib/tauri.ts` defines `AuthToken` interface

## File System

**Config Directory (`~/.config/mentascribe/`):**
- `settings.json` — User preferences (transcription, cleanup, hotkey, output, widget)
- `history.json` — Transcription entries (last 500, with id, text, word_count, duration_ms, timestamp)
- `stats.json` — Usage statistics (totals, streaks, daily history)
- `dictionary.json` — Custom word replacements (phrase -> replacement mappings)

**Model Directory (`~/.mentascribe/models/`):**
- Whisper GGML model binaries (50MB - 3GB depending on model size)
- Silero VAD model (~2MB)
- CoreML model archives (macOS only)
- Voxtral safetensors model (~8.9GB, feature-gated)

**Tauri Capabilities:** `src-tauri/capabilities/default.json` grants:
- `fs:allow-app-read-recursive`, `fs:allow-app-write-recursive`
- `fs:allow-appdata-read-recursive`, `fs:allow-appdata-write-recursive`

## IPC / Inter-Process Communication

**Tauri Command Bridge (Frontend -> Backend):**

All IPC uses `invoke()` from `@tauri-apps/api/core`. Commands defined as `#[tauri::command]` functions in `src-tauri/src/lib.rs`.

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `start_recording` | none | `()` | Start mic capture + streaming transcription |
| `stop_recording` | none | `String` | Stop capture, transcribe tail, return full text |
| `inject_text` | `text: String` | `()` | Type/paste text into active app |
| `reset_recording_state` | none | `()` | Recovery from stuck recording state |
| `get_settings` | none | `UserSettings` | Load current settings |
| `update_settings` | `new_settings: UserSettings` | `()` | Save settings, re-register hotkey if changed |
| `login` | `email, password` | `AuthToken` | Authenticate with MentaFlux API |
| `download_model` | `size: String` | `()` | Download Whisper model from Hugging Face |
| `get_available_models` | none | `Vec<ModelInfo>` | List Whisper models with download status |
| `get_coreml_status` | none | `CoremlStatus` | Check CoreML availability |
| `get_metal_status` | none | `MetalStatus` | Check Metal GPU availability |
| `download_coreml_model` | `size: String` | `()` | Download CoreML model variant |
| `delete_model` | `size: String` | `()` | Delete downloaded Whisper model |
| `delete_coreml_model` | `size: String` | `()` | Delete CoreML model |
| `get_stats` | none | `LocalStats` | Load usage statistics |
| `record_transcription_stats` | `word_count, duration_ms` | `LocalStats` | Record a transcription event |
| `get_history` | `limit?, offset?` | `Vec<TranscriptionEntry>` | Paginated history query |
| `get_history_entry` | `id: String` | `Option<TranscriptionEntry>` | Single history entry |
| `delete_history_entry` | `id: String` | `bool` | Delete one history entry |
| `clear_history` | none | `()` | Delete all history |
| `get_history_count` | none | `usize` | Total history entry count |
| `get_dictionary` | none | `Vec<DictionaryEntry>` | Load all dictionary entries |
| `add_dictionary_entry` | `phrase, replacement` | `DictionaryEntry` | Create dictionary entry |
| `update_dictionary_entry` | `id, phrase, replacement, enabled` | `DictionaryEntry` | Update dictionary entry |
| `remove_dictionary_entry` | `id: String` | `bool` | Delete dictionary entry |
| `get_voxtral_status` | none | `VoxtralStatus` | Check Voxtral engine status |
| `get_voxtral_models` | none | `Vec<ModelInfo>` | List Voxtral models |
| `download_voxtral_model` | none | `()` | Download Voxtral model files |
| `delete_voxtral_model` | none | `()` | Delete Voxtral model |
| `frontend_log` | `msg: String` | `()` | Forward frontend log to terminal |

**Tauri Event Bridge (Backend -> Frontend):**

Events emitted via `app.emit()` and listened to via `listen()` from `@tauri-apps/api/event`.

| Event | Payload | Purpose |
|-------|---------|---------|
| `hotkey-pressed` | `()` | Global hotkey pressed (start recording) |
| `hotkey-released` | `()` | Global hotkey released (stop recording) |
| `audio-level` | `f32` | Real-time audio level for waveform visualization |
| `transcription-processing` | `()` | Transcription started processing |
| `transcription-complete` | `String` | Transcription finished with result text |
| `streaming-partial` | `String` | Partial streaming transcription result |
| `download-progress` | `{ model_type, model_id, percent }` | Model download progress |
| `model-preload-start` | `String` | Model preloading started |
| `model-preload-complete` | `{ model, elapsed_secs }` | Model preloaded successfully |
| `model-preload-error` | `{ model, error }` | Model preload failed |
| `settings-changed` | `UserSettings` | Settings updated (broadcast to all windows) |

**Multi-Window Architecture:**

Two Tauri windows defined in `src-tauri/tauri.conf.json`:

1. **`dictation`** — Transparent overlay pill (52x10px, no decorations, always-on-top, skip-taskbar)
   - Converted to NSPanel on macOS for fullscreen overlay via `tauri-nspanel`
   - URL: `index.html` (default)
2. **`dashboard`** — Settings/history window (800x600, resizable, decorations)
   - URL: `index.html#dashboard`

Both windows share the same React app; routing determined by URL hash in `src/App.tsx`.

## OS-Level System Integrations

**Audio Capture (Microphone):**
- Library: `cpal` 0.15 (cross-platform audio)
- Implementation: `src-tauri/src/audio/capture.rs`
- Real-time resampling to 16kHz mono via `rubato` during recording
- Voice Activity Detection: `src-tauri/src/audio/vad.rs` (energy-based)
- Silero VAD model also used via whisper-rs for streaming detection

**Text Injection (Keyboard Simulation):**
- Implementation: `src-tauri/src/injection/mod.rs`
- macOS: CGEvent-based Unicode keyboard events via `core-graphics` (20 UTF-16 units per event, 2ms chunk delay)
- macOS: Clipboard paste via `Cmd+V` simulation as fallback
- Windows: Win32 `SendInput` via `windows` crate + `clipboard-win`
- Linux: X11 XTest key simulation via `x11` crate
- Requires Accessibility permission on macOS (checked via `AXIsProcessTrusted()`)

**Global Hotkey:**
- Library: `tauri-plugin-global-shortcut` 2
- Implementation: `src-tauri/src/hotkey/mod.rs`
- Default: F6 key (configurable F1-F12)
- Modes: toggle (press to start/stop) or hold (hold to record)
- Events: `hotkey-pressed` / `hotkey-released` emitted to frontend

**Clipboard:**
- Cross-platform: `arboard` 3
- Windows-specific: `clipboard-win` 5
- Used for paste-based text injection method

**Credential Storage:**
- Library: `keyring` 2
- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (libsecret)
- Entry: service=`mentascribe`, user=`tokens`

**System Tray:**
- Tauri tray-icon feature enabled
- Menu items: Show Dashboard, Quit
- Implementation: `src-tauri/src/lib.rs` in `run()` function

**macOS NSPanel (Fullscreen Overlay):**
- Library: `tauri-nspanel` (git, v2 branch)
- Converts dictation window to NSPanel with:
  - Window level 25 (above main menu)
  - Non-activating panel mask (does not steal focus)
  - Collection behavior: CanJoinAllSpaces, Stationary, FullScreenAuxiliary, IgnoresCycle
- Opacity control via `[NSWindow setAlphaValue:]`
- Native drag support via NSEvent monitors (local + global)

## Environment Configuration

**No `.env` file required.** Configuration is stored via:

1. **OS Keychain** — API tokens (secure, via `keyring` crate)
2. **JSON config files** — User settings, history, stats, dictionary (at `~/.config/mentascribe/`)
3. **localStorage** — Theme preference, lightweight dictation history
4. **Zustand stores** — In-memory frontend state (loaded from backend on init)

**Build-Time Environment Variables:**
- `TAURI_DEBUG` — Enables debug builds, sourcemaps, disables minification
- `RUST_LOG` — Controls `env_logger` log level (e.g., `RUST_LOG=info`)
- `CARGO_CFG_TARGET_OS` / `CARGO_CFG_TARGET_ARCH` — Used in `src-tauri/build.rs` for platform-conditional Voxtral compilation

**Content Security Policy (`src-tauri/tauri.conf.json`):**
```
default-src 'self';
connect-src https://api.voice.mentaflux.ai;
img-src 'self' data: https:;
style-src 'self' 'unsafe-inline'
```

## Monitoring & Observability

**Error Tracking:** None (no Sentry, Rollbar, or similar)

**Logging:**
- Framework: `log` crate + `env_logger` backend
- Level: Configurable via `RUST_LOG` env var
- Output: stderr (local only, no remote logging)
- Frontend: `console.log`/`console.error` + `frontend_log` command for terminal forwarding

**Analytics:** None

**CI/CD Pipeline:** None detected (no `.github/workflows/`, no CI config files)

---

*Integration audit: 2026-04-05*
