# Technical Concerns

**Analysis Date:** 2026-04-05

## Security

### API Key Stored as Plaintext JSON on Disk

- **Risk:** The `CleanupSettings.api_key` field in `src-tauri/src/settings/mod.rs` (line 36) is serialized directly to `~/.config/mentascribe/settings.json` as plaintext. Any process with read access to the user's home directory can extract it.
- **Files:** `src-tauri/src/settings/mod.rs`, `src/components/dashboard/SettingsPage.tsx` (line 1748-1750), `src/lib/store.ts` (line 19)
- **Current mitigation:** File-level OS permissions only. The `keyring` crate is already a dependency and used for auth tokens in `src-tauri/src/api/client.rs` (lines 167-216).
- **Fix approach:** Move `api_key` to the OS keychain via the existing `keyring` crate. Add `#[serde(skip_serializing)]` to prevent accidental disk persistence. Frontend should call a dedicated `set_cleanup_api_key` command that writes to keychain, and a `get_cleanup_api_key` command that reads from it.

### Auth Tokens Sent Over HTTPS but Never Validated for Expiry Client-Side

- **Risk:** `src-tauri/src/api/client.rs` stores `access_token` and `refresh_token` via keychain but the app never checks `expires_in` before using the access token. Expired tokens result in 401 errors that aren't automatically refreshed.
- **Files:** `src-tauri/src/api/client.rs` (lines 39-77, 79-114), `src-tauri/src/api/mod.rs`
- **Fix approach:** Store `expires_at` timestamp alongside tokens. Before API calls, check if token is expired and auto-refresh using `refresh_token()`. Implement retry-with-refresh middleware.

### CSP Allows `unsafe-inline` for Styles

- **Risk:** The Content Security Policy in `src-tauri/tauri.conf.json` (line 47) includes `style-src 'self' 'unsafe-inline'`, which weakens XSS protections.
- **Files:** `src-tauri/tauri.conf.json`
- **Current mitigation:** Desktop app with no user-provided HTML content reduces attack surface significantly.
- **Fix approach:** Migrate inline styles to CSS classes or use style nonces if Tauri supports them. Low priority given the desktop context.

### Multiple `unsafe impl Send` Declarations

- **Risk:** Four manual `unsafe impl Send` declarations bypass Rust's thread-safety guarantees:
  - `NativeDragState` in `src-tauri/src/lib.rs` (line 826)
  - `SendableVadContext` in `src-tauri/src/transcription/whisper.rs` (line 476)
  - `VoxtralContext` in `src-tauri/src/transcription/voxtral_ffi.rs` (line 71)
  - `VoxtralStream` in `src-tauri/src/transcription/voxtral_ffi.rs` (line 160)
- **Current mitigation:** All are documented with safety rationale and accessed behind `Mutex` locks.
- **Fix approach:** Wrap raw pointers in `NonNull` or use `Arc`-based reference counting where possible. For `NativeDragState`, consider using `objc2` crate which provides sound Send/Sync guarantees for ObjC objects.

## Performance

### 150ms Monitor Polling Loop

- **Problem:** `src/App.tsx` (lines 163-191) polls `reposition_to_mouse_monitor` every 150ms via `setInterval`. This Tauri IPC round-trip runs continuously even when the widget is stationary on a single monitor.
- **Files:** `src/App.tsx` (line 184), `src-tauri/src/lib.rs` (lines 1303-1370: `reposition_to_mouse_monitor`)
- **Impact:** Unnecessary CPU wake-ups and IPC overhead. On macOS, each poll involves NSEvent.mouseLocation + NSScreen iteration + panel frame query.
- **Fix approach:** Use native mouse-move event monitoring (NSEvent addGlobalMonitorForEventsMatchingMask with NSMouseMoved mask) to trigger repositioning only when the cursor actually moves across monitor boundaries. Replace polling with event-driven notification.

### Audio Resampling Fallback Blocks Stop Recording

- **Problem:** If the rubato real-time resampler fails during capture, `prepare_for_whisper()` in `src-tauri/src/audio/capture.rs` (lines 516-573) falls back to a synchronous post-stop resampling pass over the entire recording.
- **Files:** `src-tauri/src/audio/capture.rs` (lines 575-653: `resample()` and `resample_linear()`)
- **Impact:** On a 30-second recording at 48kHz, the fallback resampler adds 50-200ms of blocking latency to `stop_recording()`, delaying text injection.
- **Fix approach:** Move fallback resampling to the dedicated transcription thread (where Whisper inference already runs) so the UI thread isn't blocked.

### Dictionary Regex Compilation on Every Transcription

- **Problem:** `apply_replacements()` in `src-tauri/src/dictionary/mod.rs` (lines 178-196) compiles a new `regex::Regex` for each dictionary entry on every transcription call.
- **Files:** `src-tauri/src/dictionary/mod.rs` (lines 189-191)
- **Impact:** With many dictionary entries, regex compilation overhead adds up. Regex compilation is O(n) where n is pattern length.
- **Fix approach:** Pre-compile regexes when dictionary entries change and cache them alongside the `DICTIONARY_CACHE`. Use `regex::RegexSet` for batch matching if applicable.

## Technical Debt

### Cloud Transcription APIs Are Stubs

- **Issue:** All three cloud providers (OpenAI, AWS, AssemblyAI) in `src-tauri/src/transcription/cloud.rs` return hardcoded error messages.
- **Files:** `src-tauri/src/transcription/cloud.rs` (lines 52-88: `transcribe_openai()`, `transcribe_aws()`, `transcribe_assemblyai()`)
- **TODOs:** Lines 59, 72, 83 contain explicit `// TODO: Implement` markers.
- **Impact:** Users see cloud provider options in settings but they are non-functional. The `cloud_provider` field in `TranscriptionSettings` accepts values that cannot be used.
- **Fix approach:** Either implement the OpenAI Whisper API call (multipart form upload, most straightforward) or remove the cloud provider options from the UI until implemented. The `audio_to_wav()` helper at line 91 is already written and functional.

### Dual History Storage: Backend JSON + Frontend localStorage

- **Issue:** Transcription history is stored in two places:
  1. Backend: `~/.config/mentascribe/history.json` via `src-tauri/src/history/mod.rs`
  2. Frontend: `localStorage` via `src/App.tsx` (lines 32-46) `saveToHistory()` callback
- **Files:** `src-tauri/src/history/mod.rs`, `src/App.tsx` (lines 30-46), `src/components/History.tsx` (lines 19, 49)
- **Impact:** Data inconsistency. The dashboard (`HistoryPage.tsx`) reads from the backend via IPC, while the old `History.tsx` component reads from localStorage. Deletions in one store don't propagate to the other.
- **Fix approach:** Remove the localStorage-based history entirely. The backend `history::add_entry()` is already called during `stop_recording()` in `src-tauri/src/lib.rs` (line 397). Delete the `saveToHistory` callback and the `History.tsx` component's localStorage usage.

### Global Mutable State via lazy_static / once_cell

- **Issue:** The codebase uses 15+ global `Mutex`-wrapped statics for audio, transcription, and UI state.
- **Files:**
  - `src-tauri/src/audio/capture.rs` (lines 52-65): `AUDIO_BUFFER`, `WHISPER_BUFFER`, `AUDIO_THREAD`, `SAMPLE_RATE`, `CHANNELS`, `CURRENT_AUDIO_LEVEL`, `IS_STOPPING`, `RESAMPLER_STATE`
  - `src-tauri/src/transcription/whisper.rs` (lines 21, 38, 480, 597, 601, 609, 976): `MODEL_CACHE`, `STATE_CACHE`, `VAD_CACHE`, `STREAMING_RESULTS`, `STREAMING_CONSUMED`, `VAD_MONITOR`, `TRANSCRIPTION_TX`
  - `src-tauri/src/transcription/voxtral.rs`: `VOXTRAL_CACHE`, `VOXTRAL_STREAMING_RESULTS`, `VOXTRAL_STREAM_HANDLE`
  - `src-tauri/src/lib.rs` (line 829): `NATIVE_DRAG_STATE`
- **Impact:** Difficult to test in isolation, risk of deadlocks if lock ordering isn't consistent, state cleanup relies on OS process termination.
- **Fix approach:** Consolidate related state into struct-based managers (e.g., `AudioCaptureManager`, `TranscriptionManager`) and inject them through Tauri's `manage()` system. This enables per-test isolation and explicit lifetime management.

### `lib.rs` Is 1669 Lines with Mixed Concerns

- **Issue:** `src-tauri/src/lib.rs` contains Tauri command handlers, macOS NSPanel management, native drag implementation, window positioning, tray menu setup, and app initialization all in one file.
- **Files:** `src-tauri/src/lib.rs`
- **Impact:** Hard to navigate, high merge conflict risk, difficult to understand responsibility boundaries.
- **Fix approach:** Extract into modules:
  - `src-tauri/src/commands/` for Tauri command handlers
  - `src-tauri/src/window/` for panel, positioning, and drag logic
  - `src-tauri/src/tray/` for system tray setup
  - Keep `lib.rs` as the thin `run()` entry point

## Scalability

### Audio Buffers Fixed at 30 Seconds

- **Current capacity:** `AUDIO_BUFFER` pre-allocates for 30s at 48kHz stereo (2.88M samples), `WHISPER_BUFFER` for 30s at 16kHz mono (480K samples), in `src-tauri/src/audio/capture.rs` (lines 152-162).
- **Limit:** Recordings beyond ~30s trigger Vec reallocation during the real-time CPAL callback, risking audio dropouts and latency spikes.
- **Fix approach:** Either enforce a max recording duration (with UI warning) or switch to a ring buffer that overwrites oldest data. Alternatively, increase pre-allocation to match a configurable max duration.

### History JSON File Grows Unbounded (Up to 500 Entries)

- **Current capacity:** `src-tauri/src/history/mod.rs` (line 77) truncates at 500 entries, but each entry stores the full transcription text.
- **Limit:** Long transcriptions (thousands of words each) can push `history.json` to several MB. Every `add_entry()` call reads the entire file from disk, deserializes, prepends, re-serializes, and writes back (lines 62-83).
- **Fix approach:** Use SQLite (via `rusqlite`) for history storage. This eliminates full-file reads/writes and enables efficient pagination.

### Stats Daily History Kept at 30 Days

- **Current capacity:** `src-tauri/src/stats/mod.rs` (line 119) truncates `daily_history` to 30 entries.
- **Limit:** Users lose stats data older than 30 days with no export mechanism.
- **Fix approach:** Increase retention or store aggregated monthly summaries. Add data export functionality.

## Error Handling Gaps

### Pervasive `.unwrap()` on Mutex Locks

- **Problem:** Approximately 60 instances of `.lock().unwrap()` across the Rust codebase. If any lock is poisoned (e.g., a thread panicked while holding it), the application will panic.
- **Files:** Concentrated in `src-tauri/src/audio/capture.rs` (22+ instances), `src-tauri/src/transcription/whisper.rs` (15+ instances), `src-tauri/src/transcription/voxtral.rs` (8+ instances).
- **Impact:** A single panic in the audio thread poisons `AUDIO_BUFFER`'s Mutex. The next call to `start_capture()` or `stop_capture()` will propagate the panic, crashing the entire application.
- **Fix approach:** Replace `.lock().unwrap()` with `.lock().unwrap_or_else(|e| e.into_inner())` (clear poisoned state) or `.lock().map_err()` with proper error propagation. The CPAL audio callback already uses `.try_lock()` correctly.

### `stop_recording()` Can Leave State Inconsistent on Error

- **Problem:** In `src-tauri/src/lib.rs` (lines 226-408), `stop_recording()` sets `is_recording = false` early (line 242), then proceeds through streaming stop, audio capture stop, transcription, and text injection. If any step fails, the recording state is already cleared but audio resources may not be properly released.
- **Files:** `src-tauri/src/lib.rs` (lines 226-408)
- **Impact:** After a transcription failure, the audio level emitter is stopped (line 233), streaming monitor is stopped, but if `stop_capture()` fails the audio thread may still be running while `is_recording` is false.
- **Fix approach:** Implement a state machine with explicit transitions: `Recording -> Stopping -> Processing -> Idle`. Use an RAII guard that ensures cleanup runs regardless of which step fails.

### Model Download Has No Resume/Retry

- **Problem:** `download_model()` in `src-tauri/src/transcription/whisper.rs` (lines 252-301) writes to a file directly as chunks arrive. If the download is interrupted (network failure, app closed), a partial file remains on disk.
- **Files:** `src-tauri/src/transcription/whisper.rs` (lines 252-301, 334-431)
- **Impact:** The partial file passes the `model_path.exists()` check, so the app thinks the model is downloaded. Loading a truncated model file causes a cryptic "Failed to load model" error.
- **Fix approach:** Download to a `.tmp` file and rename atomically on completion. Before loading, validate file size against expected `ggml_size_bytes()`. Implement HTTP Range-based resume for partial downloads.

### Frontend Error Handling Swallows Details

- **Problem:** In `src/App.tsx`, error messages shown to users are generic (e.g., "Mic busy -- try again" on line 92, "Failed: {errorMessage}" on line 143). The actual error from the Rust backend is only logged to console.
- **Files:** `src/App.tsx` (lines 82-94, 127-149)
- **Impact:** Users cannot diagnose issues. Microphone permission errors, model corruption, and accessibility permission failures all show similar vague messages.
- **Fix approach:** Categorize backend errors into user-actionable types (permission, model, hardware, network) and display targeted guidance (e.g., "Open System Settings > Privacy > Microphone to grant access").

### File Writes Are Not Atomic

- **Problem:** All JSON data files (`settings.json`, `history.json`, `stats.json`, `dictionary.json`) are written via `std::fs::write()` which is not atomic. A crash during write produces a truncated or empty file.
- **Files:** `src-tauri/src/settings/mod.rs` (line 112), `src-tauri/src/history/mod.rs` (line 56), `src-tauri/src/stats/mod.rs` (line 71), `src-tauri/src/dictionary/mod.rs` (line 65)
- **Impact:** A crash or power loss during settings save could corrupt the settings file, causing the app to fail to start or lose all configuration.
- **Fix approach:** Write to a `.tmp` file in the same directory, then rename (which is atomic on most filesystems). Keep one backup of the previous file as `.bak`.

## Missing Features

### No Wayland Support on Linux

- **Problem:** The Linux text injection implementation in `src-tauri/src/injection/mod.rs` (lines 621-666) uses X11 directly (XTestFakeKeyEvent). Wayland sessions return an explicit error.
- **Files:** `src-tauri/src/injection/mod.rs` (lines 634-638: `is_wayland()`, line 643)
- **Impact:** Users on modern Linux distributions defaulting to Wayland (Ubuntu 22.04+, Fedora 34+) cannot use text injection at all.
- **Fix approach:** Implement Wayland text injection via `wtype` tool or `wl-clipboard` + `ydotool` combination. The `enigo` crate used for typing (line 937) may support Wayland in newer versions.

### Hotkey Only Supports F1-F12

- **Problem:** `parse_key_code()` in `src-tauri/src/hotkey/mod.rs` (lines 14-29) only maps F1-F12. No modifier key combinations, no letter keys, no special keys.
- **Files:** `src-tauri/src/hotkey/mod.rs`
- **Impact:** Users cannot bind dictation to commonly preferred shortcuts like Ctrl+Shift+D or media keys.
- **Fix approach:** Parse modifier prefixes (Ctrl+, Alt+, Shift+, Super+) and support a broader key code mapping. The `tauri-plugin-global-shortcut` already supports `Modifiers` (line 36).

### No Cleanup LLM Integration (Despite Settings Existing)

- **Problem:** `CleanupSettings` in `src-tauri/src/settings/mod.rs` (lines 30-40) has fields for LLM-based text cleanup (provider, model, custom_endpoint, api_key, remove_filler, add_punctuation, format_paragraphs) but no transcription pipeline step uses them.
- **Files:** `src-tauri/src/settings/mod.rs` (lines 30-40), `src/components/dashboard/SettingsPage.tsx` (exposes full cleanup UI)
- **Impact:** Users can configure cleanup settings but enabling them has no effect on transcription output.
- **Fix approach:** Add a `cleanup::process_text()` function that sends transcribed text to the configured LLM endpoint and applies the selected transformations. Call it in `stop_recording()` between transcription and text injection.

## Dependency Risks

### `tauri-nspanel` Pinned to Git Branch

- **Risk:** `src-tauri/Cargo.toml` depends on `tauri-nspanel` via a git branch reference: `git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2"`. This is a third-party plugin without guaranteed compatibility with Tauri updates.
- **Impact:** Tauri v3 release will likely break this dependency. Build reproducibility depends on the git branch HEAD not changing.
- **Fix approach:** Monitor for a published crate version. Pin to a specific commit hash instead of a branch. Prepare a fallback plan to implement NSPanel conversion directly using `cocoa` crate FFI.

### `whisper-rs` Model Licensing

- **Risk:** The `whisper-rs` crate bundles whisper.cpp which is MIT-licensed, but OpenAI Whisper models have varying license restrictions. The app downloads models directly from HuggingFace.
- **Impact:** Commercial distribution may require audit of model-specific licenses (some are Apache 2.0, some have additional restrictions).
- **Fix approach:** Document the license for each model in the model selection UI. Add a license acceptance step before first model download.

### `reqwest` 0.11 Is Two Major Versions Behind

- **Risk:** `src-tauri/Cargo.toml` pins `reqwest = "0.11"`. The current version is 0.12+. Version 0.11 uses `hyper` 0.14 which has known issues.
- **Impact:** Missing security patches, performance improvements, and API features in newer reqwest versions.
- **Fix approach:** Upgrade to `reqwest` 0.12. This may require updating the async runtime interface but is straightforward.

### Broad `tokio` Feature Set

- **Risk:** `Cargo.toml` enables `tokio` with `features = ["full"]`, pulling in every tokio feature (fs, signal, process, net, etc.) when the app only needs async runtime basics and oneshot channels.
- **Impact:** Increased binary size and compile time.
- **Fix approach:** Replace `"full"` with specific features needed: `["rt-multi-thread", "macros", "sync"]`.

## Code Quality Issues

### `SettingsPage.tsx` Is 1790 Lines

- **Problem:** `src/components/dashboard/SettingsPage.tsx` is the largest frontend file at 1790 lines. It contains all settings sections (transcription, hotkey, output, widget, cleanup, model management, CoreML, Voxtral) in a single component.
- **Impact:** Difficult to maintain, slow to navigate, challenging to add new settings sections.
- **Fix approach:** Extract each settings section into its own component (e.g., `TranscriptionSettings.tsx`, `ModelManager.tsx`, `HotkeySettings.tsx`). Share state via props or a settings context.

### Inconsistent Error Propagation Pattern

- **Problem:** Tauri commands use `Result<T, String>` with `.map_err(|e| e.to_string())` throughout `src-tauri/src/lib.rs`. This discards error type information and stack context.
- **Files:** `src-tauri/src/lib.rs` (nearly every `#[tauri::command]` function)
- **Impact:** Frontend receives opaque error strings with no structured error codes. Cannot programmatically distinguish between "model not found" and "disk full" without string matching.
- **Fix approach:** Define a `CommandError` enum with `serde::Serialize` that maps to structured JSON responses. Use `impl From<XError> for CommandError` for each module error type.

### Excessive `eprintln!` Debug Logging

- **Problem:** The codebase uses `eprintln!()` extensively for debug logging (~100+ instances) alongside the `log` crate. This means debug output always goes to stderr regardless of log level configuration.
- **Files:** Throughout `src-tauri/src/lib.rs`, `src-tauri/src/audio/capture.rs`, `src-tauri/src/injection/mod.rs`
- **Impact:** Release builds produce verbose stderr output that cannot be silenced via `env_logger` configuration. Performance impact from string formatting in hot paths (e.g., audio callback at line 260).
- **Fix approach:** Replace `eprintln!()` with `log::debug!()` or `log::trace!()`. Keep `log::info!()` for important state transitions. Remove logging from the CPAL audio callback hot path.

## Accessibility

### No Keyboard Navigation in Dashboard

- **Problem:** The dashboard components (`src/components/dashboard/`) use click-based interaction without explicit keyboard navigation support (no `tabIndex`, `onKeyDown`, or ARIA attributes observed).
- **Files:** `src/components/dashboard/Sidebar.tsx`, `src/components/dashboard/SettingsPage.tsx`, `src/components/dashboard/HistoryPage.tsx`, `src/components/dashboard/DictionaryPage.tsx`
- **Impact:** Users who rely on keyboard navigation cannot effectively use the dashboard settings interface.
- **Fix approach:** Add `tabIndex`, `role`, and `aria-label` attributes to interactive elements. Implement keyboard event handlers for custom components (sidebar navigation, dictionary entry management).

## Data Safety

### No Backup or Export for User Data

- **Problem:** Settings, history (500 entries), stats (30 days), and dictionary data are stored as individual JSON files in `~/.config/mentascribe/` with no backup, export, or import functionality.
- **Files:** `src-tauri/src/settings/mod.rs` (line 88), `src-tauri/src/history/mod.rs` (line 33), `src-tauri/src/stats/mod.rs` (line 48), `src-tauri/src/dictionary/mod.rs` (line 41)
- **Impact:** OS reinstallation, home directory wipe, or file corruption results in total data loss. No way to transfer configuration between machines.
- **Fix approach:** Add export/import commands that bundle all JSON files into a single archive. Implement auto-backup on app startup (copy current files to `~/.config/mentascribe/backup/`).

### Concurrent File Access Has No Locking

- **Problem:** History, stats, and dictionary files are read and written without file-level locks. If the dashboard window and dictation window both trigger writes simultaneously (e.g., dictation saves history while dashboard deletes an entry), data can be lost.
- **Files:** `src-tauri/src/history/mod.rs` (lines 36-59: `load_history_data()` and `save_history_data()`), similar pattern in `src-tauri/src/stats/mod.rs` and `src-tauri/src/dictionary/mod.rs`
- **Impact:** Race condition: Window A reads file, Window B reads same file, Window A writes, Window B writes (overwriting A's changes).
- **Fix approach:** Use file-level advisory locks (`fs2` crate) or switch to SQLite which handles concurrent access natively. Alternatively, route all data mutations through the Tauri backend where a Mutex can serialize access.

## Test Coverage Gaps

### No Automated Tests for Frontend

- **What's not tested:** Zero test files exist in `src/`. No jest, vitest, or any test framework configuration present.
- **Files:** All of `src/components/`, `src/lib/`, `src/App.tsx`
- **Risk:** UI regressions, state management bugs, and event handler issues are caught only by manual testing.
- **Priority:** Medium -- the frontend is primarily declarative UI with state management via zustand; high-risk logic lives in the Rust backend.

### Minimal Rust Tests (Only `text/mod.rs`)

- **What's tested:** Only `src-tauri/src/text/mod.rs` has tests (3 unit tests for `capitalize_sentences` and `process_text`).
- **What's not tested:**
  - Audio capture start/stop lifecycle (`src-tauri/src/audio/capture.rs`)
  - Dictionary regex replacement with edge cases (`src-tauri/src/dictionary/mod.rs`)
  - History truncation and pagination (`src-tauri/src/history/mod.rs`)
  - Stats streak calculation (`src-tauri/src/stats/mod.rs`)
  - Settings serialization round-trip (`src-tauri/src/settings/mod.rs`)
  - Hotkey key code parsing (`src-tauri/src/hotkey/mod.rs`)
- **Risk:** Core business logic (text processing, data persistence, configuration) has no regression safety net.
- **Priority:** High -- dictionary replacement, stats streak logic, and history pagination are pure functions that are trivial to test but critical to correctness.

### No Integration Tests for Tauri IPC

- **What's not tested:** The 30+ Tauri commands in `src-tauri/src/lib.rs` (lines 1625-1666) have no integration tests verifying that frontend invocations produce correct backend responses.
- **Risk:** Serialization mismatches between Rust structs and TypeScript interfaces (e.g., `src/types/index.ts` vs `src-tauri/src/history/mod.rs`) would only be caught at runtime.
- **Priority:** Medium -- TypeScript types mirror Rust structs manually, so drift is possible.

---

*Concerns audit: 2026-04-05*
