# External Integrations

**Analysis Date:** 2026-02-26

## APIs & External Services

**MentaFlux Backend API:**
- **Service:** MentaFlux voice-to-text cloud API
- **Endpoint:** https://api.voice.mentaflux.ai/v1
- **SDK/Client:** Custom HTTP client via `reqwest`
- **Implementation file:** `src-tauri/src/api/client.rs`
- **Endpoints:**
  - `POST /auth/login` — User login (email/password)
  - `POST /auth/refresh` — Token refresh
  - `POST /transcriptions` — Store transcription metadata

**Speech-to-Text Providers (Cloud Fallback):**
These are not fully implemented yet, but infrastructure exists in `src-tauri/src/transcription/cloud.rs`:
- **OpenAI Whisper API** — Status: TODO
- **AWS Transcribe** — Status: TODO
- **AssemblyAI** — Status: TODO
- Configuration via settings: `transcription.cloud_provider` field

**Model Downloads:**
- **Hugging Face CDN** — Whisper models
  - Base URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main`
  - Models: ggml-tiny.bin, ggml-base.bin, ggml-small.bin, ggml-medium.bin, ggml-large.bin
  - VAD model: `ggml-silero-vad.bin`
  - Usage: `src-tauri/src/transcription/whisper.rs` (model loading/caching)

- **Mistral HuggingFace** — Voxtral model (alternative engine)
  - Base URL: `https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602/resolve/main`
  - Model size: ~8.9 GB
  - Files: consolidated.safetensors, tekken.json, params.json
  - Usage: `src-tauri/src/transcription/voxtral.rs` (optional voxtral feature)

## Data Storage

**Databases:**
- **Not applicable** — No database backend
- Settings and history stored locally as JSON files

**File Storage (Local Filesystem):**
- **Settings:** `~/.config/mentascribe/settings.json` (user settings via `dirs` crate)
  - Managed by: `src-tauri/src/settings/mod.rs`
  - Content: transcription, cleanup, hotkey, output, widget settings

- **Models Cache:** `~/.mentascribe/models/`
  - Whisper models: `ggml-tiny.bin`, `ggml-base.bin`, etc.
  - VAD model: `ggml-silero-vad.bin`
  - Voxtral models: `consolidated.safetensors`, `tekken.json`, `params.json`
  - Managed by: `src-tauri/src/transcription/whisper.rs`, `voxtral.rs`

- **History:** Stored in-memory via Zustand, persisted via Tauri file plugin
  - Store: `src/lib/historyStore.ts`
  - Location: TBD (likely `~/.config/mentascribe/history.json`)

- **Dictionary:** Stored in-memory via Zustand
  - Store: `src/lib/dictionaryStore.ts`

**Caching:**
- **In-memory Model Cache:** Whisper context cached in `src-tauri/src/transcription/whisper.rs`
  - Static: `MODEL_CACHE` (WhisperContext)
  - Static: `STATE_CACHE` (Pre-allocated WhisperState for 50-200ms speedup)
- **No external caching service** — All caching is local/in-memory

## Authentication & Identity

**Auth Provider:**
- **Custom MentaFlux API** — Custom authentication
- **Implementation:** `src-tauri/src/api/client.rs`
- **Token Types:**
  - Access Token (Bearer token for API requests)
  - Refresh Token (for refreshing expired access tokens)
  - User Info (id, email, name, avatar_url)

- **Token Storage:**
  - **Secure:** OS Keychain via `keyring` crate
    - macOS: Keychain
    - Windows: Credential Manager
    - Linux: Secret Service
  - Entry name: `mentascribe`/`tokens`
  - Content: JSON-encoded access_token and refresh_token
  - Functions: `store_tokens()`, `get_stored_tokens()`, `clear_tokens()`

**Frontend State:**
- Zustand store: `src/lib/store.ts` (user settings only, not auth state visible)
- Auth state managed in Rust backend via AppState Mutex

## Monitoring & Observability

**Error Tracking:**
- **Not detected** — No Sentry, Rollbar, or similar integration

**Logging:**
- **Framework:** `log` crate with `env_logger` backend
- **Level:** Configurable via RUST_LOG environment variable
- **Log statements:** Throughout codebase (src-tauri/src/)
  - Example: `log::info!()`, `log::warn!()`
- **No remote logging** — Logs are local only

**Analytics:**
- **Not detected** — No analytics service integrated

## CI/CD & Deployment

**Hosting:**
- **Target Platforms:** Desktop (macOS, Windows, Linux)
- **No cloud hosting** — This is a desktop application distributed via installers

**Build & Distribution:**
- **Tauri Bundler:** Creates platform-specific installers
  - macOS: `.dmg` (disk image)
  - Windows: `.msi`, `.nsis`
  - Linux: `.appimage`, `.deb`, `.rpm`
- **Build config:** `src-tauri/tauri.conf.json`
  - Bundle category: Productivity
  - Icon sets: 32x32, 128x128, 128x128@2x (Retina), .icns (macOS), .ico (Windows)

**CI Pipeline:**
- **Not detected** — No GitHub Actions, GitLab CI, or similar configured in visible files

**Signing & Notarization:**
- **macOS:** Entitlements support (not currently configured, `entitlements: null`)
- **Windows:** Code signing support (certificate thumbprint, digest algorithm available)

## Webhooks & Callbacks

**Incoming:**
- **Not applicable** — Desktop app is event-driven only

**Outgoing:**
- **API Callbacks:** Creates transcription records via `POST /transcriptions`
  - Triggered after successful speech-to-text conversion
  - Payload: raw text, cleaned text (if processed), duration, language
  - Implementation: `src-tauri/src/api/client.rs::create_transcription()`

## System Integration Points

**OS-Level Integrations:**

1. **Audio Capture (System Microphone):**
   - Library: `cpal` (cross-platform)
   - VAD (Voice Activity Detection): `src-tauri/src/audio/vad.rs`
   - Captures and processes microphone input for transcription

2. **Text Injection (System Keyboard):**
   - Library: `enigo` (cross-platform keyboard simulation)
   - Clipboard: `arboard` (copy-to-clipboard alternative)
   - Injects transcribed text into active application

3. **Hotkey System:**
   - Library: `global-hotkey` (cross-platform system hotkeys)
   - Configurable: F5, F6, other keys
   - Modes: toggle, hold
   - Implementation: `src-tauri/src/hotkey/mod.rs`

4. **Credential Storage:**
   - macOS: Keychain (via `keyring`)
   - Windows: Credential Manager
   - Linux: Secret Service
   - Stores: API tokens, refresh tokens

5. **Display & Monitor Detection (macOS specific):**
   - Libraries: `core-graphics`, `core-foundation`
   - Used in: Monitor positioning for NSPanel overlay
   - Critical for mixed-DPI coordinate conversion (see MEMORY.md)

6. **Accessibility (macOS private API):**
   - Library: `accessibility-sys`
   - Feature flag: `tauri/macos-private-api`
   - Enables NSPanel fullscreen overlay capability

7. **Clipboard (Platform-Specific):**
   - macOS/Linux: `arboard` (cross-platform)
   - Windows: `clipboard-win` (Windows-specific)

## Security & Sensitive Data

**API Authentication:**
- **Token Storage:** OS Keychain (never in plain text on disk)
- **HTTPS Enforcement:** All API calls to https://api.voice.mentaflux.ai
- **Bearer Token:** Used for authenticated requests to `/transcriptions`
- **Token Refresh:** Automatic refresh via refresh_token

**Content Security Policy (CSP):**
- `src-tauri/tauri.conf.json` security section:
  - Default source: `'self'`
  - Connect source: https://api.voice.mentaflux.ai (API domain only)
  - Image source: `'self'`, `data:`, `https:` (remote images allowed)
  - Style source: `'self'`, `'unsafe-inline'` (Tailwind requires inline)

**Audio Data:**
- Local models: Whisper processes audio entirely offline
- Cloud transcription: Audio would be sent to configured cloud provider (currently disabled)

## Environment Variables

**No .env file used** — Configuration is stored via:
1. **Zustand store** (in-memory): User settings
2. **OS Keychain** (secure): API tokens
3. **Disk JSON** (`~/.config/mentascribe/settings.json`): Persistent settings

**Optional Build Variables:**
- `RUST_LOG` — Logger level control (when running with env_logger)
- `TAURI_DEBUG` — Enables debug builds and sourcemaps

---

*Integration audit: 2026-02-26*
