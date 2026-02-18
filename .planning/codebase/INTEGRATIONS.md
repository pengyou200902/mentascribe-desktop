# External Integrations

**Analysis Date:** 2026-02-18

## APIs & External Services

**MentaFlux Backend API:**
- Endpoint: `https://api.voice.mentaflux.ai/v1`
- SDK/Client: Custom HTTP client via `reqwest`
- Implementation: `src-tauri/src/api/client.rs`
- Authentication: Bearer token (JWT)
- Available endpoints:
  - `POST /auth/login` - Email/password authentication
  - `POST /auth/refresh` - Token refresh
  - `POST /transcriptions` - Store transcription records
- CSP Policy: Configured in `src-tauri/tauri.conf.json` to allow `connect-src https://api.voice.mentaflux.ai`

**Cloud Speech-to-Text Providers (Framework in place, not yet implemented):**
- **OpenAI Whisper API**
  - Status: Stubbed in `src-tauri/src/transcription/cloud.rs` (line 52)
  - Requires: Audio file in WAV format + API key
  - Environment variable: `OPENAI_API_KEY` (not set up)

- **AWS Transcribe**
  - Status: Stubbed in `src-tauri/src/transcription/cloud.rs` (line 68)
  - Requires: AWS credentials
  - Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` (not set up)

- **AssemblyAI**
  - Status: Stubbed in `src-tauri/src/transcription/cloud.rs` (line 79)
  - Requires: API key
  - Environment variable: `ASSEMBLYAI_API_KEY` (not set up)

## Data Storage

**Databases:**
- Not detected - No SQL database integration (SQLite, PostgreSQL, etc.)

**File Storage:**
- Local filesystem only
  - Settings: `~/.config/mentascribe/settings.json` (JSON format)
  - History: `~/.config/mentascribe/history.json` (JSON format)
  - Implementation: `src-tauri/src/settings/mod.rs`, `src-tauri/src/history/mod.rs`
  - Managed by Tauri filesystem plugin: `tauri-plugin-fs`

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- Custom MentaFlux backend
  - Endpoint: `https://api.voice.mentaflux.ai/v1/auth/login`
  - Flow: Email/password credentials → access_token + refresh_token
  - Token storage: OS keychain via `keyring` library (macOS: Keychain, Windows: Credential Manager, Linux: encrypted file)
  - Implementation: `src-tauri/src/api/client.rs`

**Token Management:**
- Access tokens: Bearer auth in API requests
- Refresh tokens: Stored securely, used to refresh expired access tokens
- Functions in `src-tauri/src/api/client.rs`:
  - `store_tokens()` - Save to keychain
  - `get_stored_tokens()` - Retrieve from keychain
  - `clear_tokens()` - Remove on logout
  - `refresh_token()` - Exchange refresh token for new access token

## Monitoring & Observability

**Error Tracking:**
- None detected

**Logs:**
- Simple logging via `log` crate + `env_logger`
- Log initialization: Environment variable controlled
- Output: Console (development), file (if configured)
- Modules with logging: API client, settings, history, macOS panel setup

## CI/CD & Deployment

**Hosting:**
- Desktop application (not web-hosted)
- Built as native binaries for macOS, Windows, Linux

**Build Artifacts:**
- macOS: `.dmg` installer
- Windows: `.msi` + `.nsis` installer
- Linux: `.AppImage`, `.deb`, `.rpm`
- Configured in `src-tauri/tauri.conf.json` bundle section

**Build Commands:**
- Development: `pnpm dev` (via Tauri: `tauri dev`)
- Production: `pnpm build && tauri build`
- Frontend: Vite build to `dist/`
- Backend: Cargo release build with optimizations

**CI Pipeline:**
- Not detected (no GitHub Actions, GitLab CI, etc.)

## Environment Configuration

**Required Environment Variables:**
- None strictly required for basic functionality
- Optional cloud provider keys (not yet used):
  - `OPENAI_API_KEY` - For OpenAI Whisper cloud fallback
  - `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` - For AWS Transcribe
  - `ASSEMBLYAI_API_KEY` - For AssemblyAI

**Configuration Files:**
- User settings stored locally: `~/.config/mentascribe/settings.json`
  - Transcription provider selection: "whisper-local", "vosk", "cloud"
  - Language setting
  - Cleanup/text processing settings (LLM provider, model, endpoint)
  - Hotkey configuration
  - Output settings (insert method, auto-capitalize)

**Secrets Storage:**
- Tokens: OS keychain (not plaintext)
- API keys: User configurable in settings (stored in JSON - not secure for production)
  - File location: `~/.config/mentascribe/settings.json`
  - Note: Text cleanup API keys are currently stored as plaintext in JSON

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- Transcription submission to MentaFlux backend
  - Endpoint: `POST /transcriptions`
  - Triggered: After text injection or manual sync
  - Fields: raw_text, cleaned_text, duration_ms, language
  - Authentication: Bearer token (access_token)

## Tauri Plugin Integrations

**Shell Plugin** (`tauri-plugin-shell`):
- Enabled: `open: true` in `tauri.conf.json`
- Use case: May be used for system integration

**Dialog Plugin** (`tauri-plugin-dialog`):
- File/folder selection dialogs (native OS dialogs)
- Implementation: Available but not actively used in main code

**Filesystem Plugin** (`tauri-plugin-fs`):
- Used for reading/writing settings and history files
- Provides abstraction over OS filesystem

**HTTP Plugin** (`tauri-plugin-http`):
- Provides HTTP client functionality
- Replaces direct socket access on some platforms
- Note: Also uses `reqwest` for HTTP requests

**Global Shortcut Plugin** (`tauri-plugin-global-shortcut`):
- Alternative to `global-hotkey` crate
- Both available; primary implementation in Rust via `global-hotkey` crate

## System Integration

**Audio Input:**
- Cross-platform audio capture via `cpal`
- Sample rate: 16000 Hz
- Format: Mono, 16-bit signed integer
- Endpoint: Configured in `src-tauri/src/audio/capture.rs`

**Text Injection:**
- macOS: NSEvent/keyboard simulation via Cocoa
- Windows: Windows API (Win32_UI_Input_KeyboardAndMouse)
- Linux: X11 XTest protocol
- Implementation: `src-tauri/src/injection/mod.rs`

**Clipboard Access:**
- Cross-platform via `arboard` library
- Fallback for text insertion if direct injection unavailable

**Hotkey System:**
- Global hotkey binding via `global-hotkey` crate
- Monitoring: Continuous background listener
- Configuration: User-selectable keys (F5, F6, etc.) and modes (hold/toggle)

**macOS NSPanel (Fullscreen Overlay):**
- Converts Tauri webview window to Cocoa NSPanel
- Allows dictation overlay above fullscreen apps (window level 25)
- Collection behavior: Can join all spaces, stationary, fullscreen auxiliary
- Non-activating: Doesn't steal keyboard focus
- Implementation: `src-tauri/src/lib.rs` (setup_dictation_panel, refresh_panel_settings)

## Data Sync

**Transcription Sync:**
- Manual: User can trigger sync to MentaFlux backend
- Records include: raw_text, cleaned_text (if processed), duration_ms
- Sync status tracked: `synced` boolean in history entries
- Implementation: `src-tauri/src/history/mod.rs` (mark_synced function)

## Security Considerations

**Credential Storage:**
- Access/refresh tokens: OS keychain (encrypted, platform-native)
- User credentials: Never stored, only transmitted once for login
- API keys: Currently plaintext in settings JSON (security concern)

**API Communication:**
- HTTPS only for MentaFlux API
- Bearer token authentication
- No encryption for local settings/history files

**Local Data:**
- Settings file: Plaintext JSON in user config directory
- History file: Plaintext JSON with unencrypted transcription text
- Clipboard content: Accessible via arboard (standard OS-level access)

---

*Integration audit: 2026-02-18*
