# External Integrations

**Analysis Date:** 2026-02-19

## APIs & External Services

**MentaFlux Voice API:**
- Service: `https://api.voice.mentaflux.ai/v1` (base URL in `src-tauri/src/api/client.rs`)
- What it's used for: User authentication, transcription history tracking, user dashboard sync
- SDK/Client: Custom HTTP client using `reqwest` 0.11
- Auth: Bearer token (JWT access_token + refresh_token)
- Endpoints:
  - `POST /auth/login` - Email/password authentication (expects `accessToken`, `refreshToken`, `expiresIn`, `user` in response)
  - `POST /auth/refresh` - Token refresh using `refreshToken`
  - `POST /transcriptions` - Submit transcription records with `rawText`, `cleanedText`, `durationMs`, `language`

**Cloud Speech-to-Text Providers (not yet fully implemented):**
- OpenAI Whisper API - Planned integration for cloud transcription fallback
- AWS Transcribe - Planned integration for cloud transcription
- AssemblyAI - Planned integration for cloud transcription
- Implementation location: `src-tauri/src/transcription/cloud.rs`
- Status: Framework exists but APIs not yet implemented (returns `ApiError` with "not yet implemented")

**Hugging Face Model Repository:**
- Service: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main`
- What it's used for: Downloading OpenAI Whisper GGML model binaries for local speech-to-text
- Files: GGML model binaries (`ggml-tiny.bin`, `ggml-base.bin`, `ggml-small.bin`, `ggml-medium.bin`, `ggml-large-v3.bin`)
- CoreML encoder models: `ggml-{size}-encoder.mlmodelc` packages (macOS acceleration)
- Client: `reqwest` HTTP client with progress tracking
- Models stored: `~/.mentascribe/models/`
- Implementation: `src-tauri/src/transcription/whisper.rs`

## Data Storage

**Local Settings:**
- Storage: JSON file at `~/.config/mentascribe/settings.json` (platform-dependent via `dirs` crate)
- Format: JSON (deserialized to `UserSettings` struct)
- Client: Native filesystem access via Tauri `fs` plugin
- Contents:
  - Transcription settings (provider, language, model size, CoreML preference)
  - Cleanup settings (LLM provider: OpenAI/Anthropic/Ollama/custom, model, API key)
  - Hotkey configuration
  - Output settings (insertion method, auto-capitalization)
  - Widget settings (draggable, opacity)
- Implementation: `src-tauri/src/settings/mod.rs`, loaded/saved on startup and after updates

**Local Transcription Models:**
- Storage: `~/.mentascribe/models/` directory
- Models: Whisper GGML and CoreML encoder files
- Management: Downloaded on-demand, stored locally for offline use

## Authentication & Identity

**Auth Provider:**
- Service: MentaFlux custom backend at `https://api.voice.mentaflux.ai/v1`
- Implementation: Email/password authentication via `POST /auth/login`
- Token types:
  - `access_token` - JWT for API requests (short-lived)
  - `refresh_token` - Long-lived token for obtaining new access tokens
  - `expires_in` - Expiration time in seconds
- Secure storage: OS keychain via `keyring` crate (2)
  - Entry name: "mentascribe"
  - Key: "tokens"
  - Stored as JSON: `{"access_token": "...", "refresh_token": "..."}`
- Token refresh: `POST /auth/refresh` endpoint automatically called on expiration
- Implementation: `src-tauri/src/api/client.rs` (login, refresh_token, store_tokens, get_stored_tokens, clear_tokens functions)

**User Profile:**
- Returned on login: `UserInfo` struct with `id`, `email`, `name`, `avatar_url`
- Used for: Dashboard display, history/stats tracking, user identification

## Monitoring & Observability

**Error Tracking:**
- Not detected - No Sentry, Bugsnag, or similar integration found

**Logs:**
- Framework: Native Rust `log` crate 0.4 with `env_logger` 0.11 implementation
- Console output: Debug logs printed to stderr and stdout (e.g., `[nspanel]`, `[recording]` prefixes)
- File logging: Not configured
- Log levels: debug, info, warn, error used throughout codebase
- Examples:
  - `log::info!("Dictation window successfully converted to NSPanel...")`
  - `log::warn!("OpenAI cloud transcription not yet implemented")`
  - `log::error!("Failed to convert dictation window to NSPanel...")`

## CI/CD & Deployment

**Hosting:**
- Distribution: Native desktop app (not cloud-hosted)
- Deployment targets: macOS (.dmg), Windows (.msi, .nsis), Linux (.appimage, .deb, .rpm)
- Build managed by: Tauri CLI v2

**CI Pipeline:**
- Not detected - No GitHub Actions, GitLab CI, or similar configuration found

**Build Scripts:**
- Dev: `pnpm dev` (runs Vite dev server at localhost:1420)
- Build: `pnpm build` (runs `tsc && vite build` for frontend)
- Tauri commands via: `tauri dev`, `tauri build` CLI

## Environment Configuration

**Required env vars:**
- No runtime environment variables required
- Tauri respects `VITE_*` and `TAURI_*` prefixes for build-time configuration
- Settings managed through JSON file, not environment variables

**Secrets location:**
- API tokens: OS keychain (secure platform storage)
- LLM API keys: Stored in settings JSON at `~/.config/mentascribe/settings.json` under `cleanup.api_key`
  - Note: API keys stored locally in plaintext JSON - should use keychain for production
- Cloud provider credentials: Configured via settings but not yet integrated

**CSP (Content Security Policy):**
- Defined in `src-tauri/tauri.conf.json`
- `default-src 'self'` - Only self scripts/resources
- `connect-src https://api.voice.mentaflux.ai` - Only allow API calls to MentaFlux API
- `img-src 'self' data: https:` - Local, data URLs, and HTTPS images
- `style-src 'self' 'unsafe-inline'` - Local and inline styles (for Tailwind)

## Webhooks & Callbacks

**Incoming:**
- Not detected - No webhook endpoints implemented

**Outgoing:**
- Transcription submissions: `POST https://api.voice.mentaflux.ai/v1/transcriptions` - Sends transcription data to dashboard/history
- No other external callbacks found

---

*Integration audit: 2026-02-19*
