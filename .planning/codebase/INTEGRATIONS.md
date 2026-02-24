# External Integrations

**Analysis Date:** 2026-02-24

## APIs & External Services

**MentaFlux Voice API:**
- Service: `https://api.voice.mentaflux.ai/v1`
- What it's used for: User authentication, transcription history syncing
- SDK/Client: `reqwest` HTTP client (custom implementation in `src-tauri/src/api/client.rs`)
- Auth: Bearer token via access_token + refresh_token
- Endpoints:
  - `POST /auth/login` - Email/password authentication
  - `POST /auth/refresh` - Token refresh
  - `POST /transcriptions` - Submit transcription to dashboard

**Cloud Speech-to-Text (Planned/Stub):**
- OpenAI Whisper API - Planned, not yet implemented
- AWS Transcribe - Planned, not yet implemented
- AssemblyAI - Planned, not yet implemented
- Provider selection via settings: `transcription.cloud_provider` ("openai", "aws", "assemblyai")

**Cleanup/LLM Services (Configurable):**
- OpenAI - Optional LLM for text cleanup
- Anthropic - Optional LLM for text cleanup
- Ollama - Optional local LLM for text cleanup
- Custom endpoint - User-specified endpoint for text cleanup
- Configuration: `settings.cleanup.provider`, `settings.cleanup.model`, `settings.cleanup.custom_endpoint`, `settings.cleanup.api_key`

## Data Storage

**Local File Storage:**
- Settings: `~/.config/mentascribe/settings.json` (JSON)
- History: `~/.config/mentascribe/history.json` (JSON, max 500 entries)
- Dictionary: `~/.config/mentascribe/dictionary.json` (JSON, custom phrases/replacements)
- Transcription models: Platform-specific cache directories

**Databases:**
- None - Application uses JSON file-based local storage only
- No database server required
- Client: Standard file I/O via Rust std library + `dirs` crate for path resolution

**Authentication Token Storage:**
- OS Keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Client: `keyring` crate v2.0
- Stored as: `mentascribe` service with `tokens` entry (JSON serialized)

## Authentication & Identity

**Auth Provider:**
- Custom backend: MentaFlux Voice API
- Implementation approach: Email/password login with JWT tokens
- Token management: Access token + refresh token stored in OS keychain
- Token persistence: `keyring::Entry::new("mentascribe", "tokens")`

**Session Management:**
- Automatic token refresh via `src-tauri/src/api/client.rs::refresh_token()`
- Failed requests with 401 status trigger re-authentication flow
- Tokens cleared on logout via `keyring::Entry::delete_password()`

## Monitoring & Observability

**Error Tracking:**
- Not detected - No error tracking integration (e.g., Sentry, Rollbar)

**Logs:**
- Local logging via `log` crate with `env_logger`
- Configuration: Environment variable based via `env_logger` (e.g., `RUST_LOG=info`)
- Output: Console/stderr in debug, file logging not configured
- Frontend: Console.error/console.log for debugging

## CI/CD & Deployment

**Hosting:**
- Desktop application (self-contained .app on macOS, .exe/.msi on Windows, .AppImage on Linux)
- No centralized hosting - distributed as standalone executable

**CI Pipeline:**
- Not detected - No GitHub Actions, GitLab CI, or other CI pipeline configured
- Manual build process via `cargo tauri build`

**Distribution:**
- Tauri built-in update mechanism (not configured)
- Manual distribution required

## Environment Configuration

**Required env vars:**
- `RUST_LOG` - Logging level (optional, default not set)
- `TAURI_DEBUG` - Debug mode flag (optional, used in vite.config.ts for source maps)

**Secrets location:**
- Auth tokens: OS Keychain (encrypted by OS)
- API keys: Stored in settings.json (user-provided for cleanup providers)
- Environment files: `.env`, `.env.local`, `.env.*.local` are ignored in `.gitignore` (present detection only - contents never committed)

**Settings Configuration:**
Location: `~/.config/mentascribe/settings.json`

Schema:
```json
{
  "transcription": {
    "provider": "whisper-local|vosk|cloud",
    "language": "auto|en|es|...",
    "model_size": "tiny|base|small|medium|large",
    "cloud_provider": "aws|openai|assemblyai",
    "use_coreml": true|false|null,
    "engine": "whisper|voxtral",
    "voxtral_delay_ms": 80-2400
  },
  "cleanup": {
    "enabled": true|false,
    "provider": "openai|anthropic|ollama|custom",
    "model": "string",
    "custom_endpoint": "https://...",
    "api_key": "string",
    "remove_filler": true|false,
    "add_punctuation": true|false,
    "format_paragraphs": true|false
  },
  "hotkey": {
    "key": "F6|F5|...",
    "mode": "hold|toggle"
  },
  "output": {
    "insert_method": "type|paste",
    "auto_capitalize": true|false
  },
  "widget": {
    "draggable": true|false,
    "opacity": 0.2-1.0
  }
}
```

## Webhooks & Callbacks

**Incoming:**
- Not detected

**Outgoing:**
- Transcriptions synced to MentaFlux API via `POST /v1/transcriptions` endpoint
- Marked as synced in local history via `mark_synced()` after successful submission
- Sync status tracked per entry: `TranscriptionEntry.synced` boolean field

## Tauri IPC Bridge

**Frontend → Backend Commands:**
- `invoke('get_settings')` - Load user settings from disk
- `invoke('update_settings', { newSettings })` - Persist settings to disk
- Audio recording and transcription commands (transcription module)
- History and dictionary management commands

**Backend → Frontend Events:**
- `emit()` based events for transcription progress, status updates

## Data Sync Architecture

**Unsynced Records:**
- New transcriptions created locally with `synced: false`
- Dictionary entries created with `synced: false`
- History entries created with `synced: false`

**Sync Strategy:**
- Application sends unsynced entries to `/v1/transcriptions` endpoint
- Server responds with success/failure
- Client calls `mark_synced()` to update local records
- Entries remain locally (never deleted, only marked as synced)

---

*Integration audit: 2026-02-24*
