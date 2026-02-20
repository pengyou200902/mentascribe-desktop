# External Integrations

**Analysis Date:** 2026-02-20

## APIs & External Services

**MentaScribe Backend:**
- Service: MentaScribe Account API
  - Base URL: `https://api.voice.mentaflux.ai/v1`
  - SDK/Client: Custom async client using `reqwest` (`src-tauri/src/api/client.rs`)
  - Auth: Bearer token (JWT `access_token`)
  - Endpoints:
    - `POST /auth/login` - Request: email, password; Response: access_token, refresh_token, expiresIn, user
    - `POST /auth/refresh` - Request: refreshToken; Response: new access/refresh tokens
    - `POST /transcriptions` - Create transcription record with metadata (raw_text, cleaned_text, durationMs, language)

**Speech-to-Text Providers (Optional Cloud Fallback):**
- Settings location: `src-tauri/src/settings/mod.rs` (`TranscriptionSettings.cloud_provider`)
- Supported: "aws", "openai", "assemblyai" (defined as options, not yet fully implemented)
- Status: Cloud transcription stub exists (`src-tauri/src/transcription/cloud.rs`) - OpenAI Whisper API marked TODO

**Text Cleanup/Polish (Optional):**
- Settings location: `src-tauri/src/settings/mod.rs` (`CleanupSettings.provider`)
- Supported providers: "openai", "anthropic", "ollama", "custom"
- Usage: LLM-based text post-processing (remove filler words, add punctuation, format paragraphs)
- Implementation: Settings define API key storage, model, and custom endpoint support

## Data Storage

**Local Storage (Filesystem):**

**Configuration Files:**
- Settings: `~/.config/mentascribe/settings.json` (or OS-specific equivalent)
  - Persisted via `dirs::config_dir()` + `mentascribe/settings.json`
  - Format: JSON serialized from `UserSettings` struct
  - Updated via Tauri invoke: `update_settings(settings)`

**Transcription History:**
- Location: `~/.config/mentascribe/history.json` (same config dir pattern)
- Format: JSON array of `TranscriptionEntry` objects with id, text, word_count, duration_ms, timestamp, synced flag
- Loaded/saved via `src-tauri/src/history/mod.rs` functions
- Synced flag indicates whether entry was sent to backend API

**Audio Models:**
- Whisper models: `~/.mentascribe/models/` directory
  - Files: `ggml-{size}.bin` (e.g., `ggml-tiny.bin`, `ggml-base.bin`, `ggml-small.bin`, `ggml-medium.bin`, `ggml-large-v3.bin`)
  - CoreML encoder variants: `ggml-{size}-encoder.mlmodelc/` (directory structure on macOS)
  - VAD model: `ggml-silero-vad.bin` (Voice Activity Detection)
  - Downloaded on-demand from HuggingFace if missing

**Statistics:**
- Location: `~/.config/mentascribe/stats.json` (config dir)
- Tracks: daily/weekly/monthly dictation metrics
- Updated after each transcription

**Cache:**
- Whisper model context: in-memory `Arc<WhisperContext>` via `once_cell::sync::Lazy` static (reused across transcriptions)
- WhisperState pre-cache: background thread creates next state while current transcription runs (50-200ms speedup)

**Secure Storage:**
- Credentials/Tokens: OS keychain via `keyring` crate
  - Service name: "mentascribe"
  - Key name: "tokens"
  - Stores: JSON with access_token, refresh_token
  - Accessed via `src-tauri/src/api/client.rs` functions: `store_tokens()`, `get_stored_tokens()`, `clear_tokens()`
  - Secure for: macOS Keychain, Windows Credential Manager, Linux Secret Service

## Authentication & Identity

**Auth Provider:**
- Service: Custom MentaScribe backend (managed by MentaFlux)
- Implementation: OAuth-like (email/password login → JWT tokens)
- Token storage: OS keychain (via `keyring` crate)
- Flow:
  1. User logs in via frontend with email + password
  2. Backend `login(email, password)` calls `/auth/login`, receives access_token, refresh_token, expires_in, user info
  3. Tokens stored securely in keychain
  4. Access token used as Bearer auth for API requests
  5. Refresh token used to obtain new access token before expiry

**User Info:**
- Fields: id, email, name (optional), avatar_url (optional)
- Returned from login/refresh endpoints
- Cached in frontend Zustand store

## Monitoring & Observability

**Error Tracking:**
- None detected (no Sentry, Bugsnag, or similar integration)

**Logging:**
- Framework: Rust `log` crate with `env_logger` backend
- Log level: Controlled via environment variable (e.g., `RUST_LOG=debug`)
- Key log statements: NSPanel setup/teardown, audio capture state, transcription progress, API errors
- Frontend: JavaScript console logging (Tauri dev console available in debug builds)

**Statistics:**
- Manual tracking via `src-tauri/src/stats/mod.rs`
- Metrics: word count, duration, date/time of transcriptions
- Local-first; synced to backend on demand

## CI/CD & Deployment

**Hosting:**
- Desktop application (not web-hosted)
- Distributed via: .dmg (macOS), .msi (Windows), .AppImage/.deb/.rpm (Linux)
- Build artifacts in: `src-tauri/target/release/bundle/{dmg,msi,appimage,deb,rpm}/`

**Build Pipeline:**
- No automatic CI/CD configured (git repo present, no GitHub Actions, GitLab CI, etc.)
- Manual builds via: `pnpm tauri build` command
- Build lifecycle:
  1. Frontend: TypeScript → esbuild minification → bundled assets
  2. Backend: Rust compilation with release optimizations (LTO, codegen-units=1)
  3. Bundle: Tauri packages both into installer for target platform
  4. Code signing: Not configured (Windows certificateThumbprint is null)

## Environment Configuration

**Required env vars:**
- `VITE_` prefix: Frontend variables (injected at build time)
- `TAURI_` prefix: Tauri framework configuration
- `RUST_LOG`: Optional log level for env_logger (e.g., `debug`, `info`, `warn`)
- API credentials: NOT in env vars (stored in OS keychain at runtime after login)

**Secrets location:**
- OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Service: "mentascribe"
- Not in `.env` files (no `.env` detected in codebase)
- API key for cleanup providers: stored in settings as `CleanupSettings.api_key` (settings.json file-based)

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- `/transcriptions` POST endpoint: Called after local transcription to sync result to backend
- Includes: raw_text, cleaned_text (optional), durationMs, language (optional)
- Bearer auth required

## Settings Synchronization

**Flow:**
- Frontend (Zustand store) → `invoke('update_settings', { settings })` (Tauri IPC)
- Rust handler updates `AppState` Mutex with new settings
- Persists to `~/.config/mentascribe/settings.json`
- On app restart, settings reloaded from disk into Zustand store

**Providers configured at settings level:**
- Transcription: local whisper-rs vs. cloud (AWS, OpenAI, AssemblyAI)
- Cleanup: OpenAI, Anthropic, Ollama, or custom HTTP endpoint
- Hotkey: F5, F6, etc. with mode (hold vs. toggle)
- Output: type vs. paste insertion method
- Widget: draggable flag, opacity (0.2-1.0)

## Third-Party Integrations Summary

| Service | Type | Purpose | Status |
|---------|------|---------|--------|
| MentaScribe API | Authentication + Sync | User login, transcription sync | Active |
| OS Keychain | Credential Storage | Token/credential storage | Active |
| HuggingFace | Model Distribution | Whisper GGML model downloads | Active (on-demand) |
| OpenAI/Anthropic/Ollama | Text Cleanup (LLM) | Optional text post-processing | Configurable (not default) |
| AWS/AssemblyAI | Cloud STT Fallback | Optional cloud transcription | Defined but not implemented |
| OS Audio System | Audio Capture | cpal/ALSA/CoreAudio integration | Active |
| OS Clipboard | Text Output | Clipboard paste via arboard | Active |
| OS Input System | Text Injection | Keyboard simulation via enigo | Active |

---

*Integration audit: 2026-02-20*
