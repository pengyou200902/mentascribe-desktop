# Codebase Concerns

**Analysis Date:** 2026-02-18

## Tech Debt

**Audio Capture State Management via Lazy Statics:**
- Issue: Global mutable state using `lazy_static!` mutexes for audio buffer, thread handles, and sample rate
- Files: `src-tauri/src/audio/capture.rs` (lines 34-42)
- Impact: Thread-safety guarantees rely on mutex correctness; state not properly isolated; difficult to test; can lead to race conditions if `.unwrap()` is called on poisoned mutexes
- Fix approach: Migrate to a dedicated audio capture struct managed via Tauri's state system (AppState); replace `lazy_static!` with Arc<Mutex<>> in managed state

**Panicking Unwraps in Critical Paths:**
- Issue: Multiple `.unwrap()` calls throughout audio capture and transcription paths that will panic if mutexes are poisoned
- Files: `src-tauri/src/audio/capture.rs` (lines 55, 61, 64, 70, 83, 90, 208, 234-236)
- Impact: Audio stream crashes if any thread panics; application goes unresponsive; users cannot recover without restart
- Fix approach: Replace all `.unwrap()` on mutex locks with proper error handling; return `Result<>` from functions that acquire locks; log poisoned mutexes explicitly

**Whisper Model Context Caching with Unwrap:**
- Issue: Model context cached in `lazy_static!` Mutex; `.unwrap()` on cache access (line 193 in whisper.rs)
- Files: `src-tauri/src/transcription/whisper.rs` (lines 18-24, 166, 193)
- Impact: If cache lock is poisoned (rare but possible), transcription panics; will crash entire application during transcription
- Fix approach: Replace lazy_static cache with proper managed state; return error instead of panicking on poisoned locks

**Fire-and-Forget History and Stats Recording:**
- Issue: History and stats recording failures are silently ignored in stop_recording (lines 244-249 in lib.rs)
- Files: `src-tauri/src/lib.rs` (lines 243-249)
- Impact: User loses transcription history without knowing; stats are incomplete; no visibility into data persistence failures
- Fix approach: Emit events on failure; log warnings as errors; consider retrying or queuing failed entries

**Hardcoded Whisper Model Defaults:**
- Issue: Model defaults hardcoded as "small" in multiple places; no validation that model exists before transcription
- Files: `src-tauri/src/lib.rs` (lines 130, 614), `src-tauri/src/transcription/whisper.rs` (line 130)
- Impact: If model missing, transcription fails with confusing error; no pre-flight checks; users blame app not themselves
- Fix approach: Validate model exists before transcription attempt; emit "model-needs-download" event early

---

## Known Bugs

**Audio Level Meter Emitter Thread Not Properly Terminated:**
- Symptoms: Audio level events continue after stop_recording or on app crash
- Files: `src-tauri/src/lib.rs` (lines 141-160)
- Trigger: Start recording, then force-quit app without stopping; thread doesn't clean up
- Current mitigation: `audio_level_emitter_running` atomic flag stops thread, but if main app crashes, flag is never cleared
- Workaround: App restart resets the flag

**Silence Trimming Too Aggressive on Empty Audio:**
- Symptoms: Short recordings or very quiet audio trimmed down to nothing before Whisper
- Files: `src-tauri/src/audio/capture.rs` (lines 314-332)
- Trigger: Record less than 100ms of quiet speech
- Current mitigation: Guard at line 318 skips trimming if result < 20% of original, but threshold may still be too high for real speech
- Recommendation: Lower aggressive trim threshold or use ML-based VAD (Silero VAD)

**Window Position Calculation Ignores Scale Factor on Some Configs:**
- Symptoms: Dictation window positions incorrectly on high-DPI displays with multiple monitors of different scales
- Files: `src-tauri/src/lib.rs` (lines 413-429, 494-501)
- Trigger: 27-inch 4K monitor + 13-inch 2x Retina external display (common MacBook setup)
- Current mitigation: Logical coordinates used for positioning, but initial position selection logic (lines 494-501) uses physical coords
- Workaround: Manual window drag to correct position on first launch

**Clipboard Not Cleared on Paste Failure:**
- Symptoms: Sensitive text (user's dictation) left on clipboard if paste injection fails
- Files: `src-tauri/src/injection/mod.rs` (lines 391-408)
- Trigger: CGEvent paste fails but function returns error before reaching clipboard.clear()
- Current mitigation: None - clipboard persists
- Fix approach: Use RAII guard to ensure clipboard is always cleared, even on error

---

## Security Considerations

**API Tokens Stored in System Keyring Without Validation:**
- Risk: Malicious app with accessibility permissions could read tokens; no token rotation enforcement
- Files: `src-tauri/src/api/client.rs` (lines 167-180, 184-203)
- Current mitigation: Uses OS keyring (better than plaintext), but no encryption layer, no expiry monitoring
- Recommendations:
  1. Implement token refresh before expiry rather than on-demand
  2. Add token versioning to detect compromised tokens
  3. Monitor for unusual API usage patterns

**Accessibility Permission Overreach:**
- Risk: Once accessibility granted, app can inject text into any app including password fields, browser devtools, etc.
- Files: `src-tauri/src/injection/mod.rs` (entire module)
- Current mitigation: User consent required at OS level, but no in-app warning about what this enables
- Recommendations:
  1. Display in-app warning: "Accessibility access allows typing in any application"
  2. Add confirmation dialog before first injection
  3. Consider per-app whitelist or typing-into-sensitive-fields detection

**No Input Validation on Text Injection:**
- Risk: If user injects very long text (megabytes), could cause app unresponsiveness
- Files: `src-tauri/src/injection/mod.rs` (lines 333-385)
- Current mitigation: None
- Recommendations: Add max length check (e.g., 10,000 chars); rate-limit injection requests

**Settings File Readable by Any User:**
- Risk: If user runs app with sudo or in shared account, settings.json includes API keys in plaintext on disk
- Files: `src-tauri/src/settings/mod.rs` (lines 71-84)
- Current mitigation: File written with default permissions (mode 0644)
- Recommendations:
  1. Migrate all sensitive settings (API keys, tokens) to OS keyring
  2. Settings file should be mode 0600 (user-only)
  3. Document that settings file is not encrypted

**Model Download URL Hardcoded, No Checksum Verification:**
- Risk: If Hugging Face account compromised or DNS hijacked, malicious models could be downloaded
- Files: `src-tauri/src/transcription/whisper.rs` (lines 38, 98)
- Current mitigation: None
- Recommendations:
  1. Add SHA256 checksum verification after download
  2. Use HTTPS only (already done)
  3. Pin certificate if possible
  4. Consider signing models with app developer key

---

## Performance Bottlenecks

**Full History JSON Rewritten On Every New Entry:**
- Problem: Every transcription causes full history.json to be re-read, modified, and re-written
- Files: `src-tauri/src/history/mod.rs` (lines 62-83)
- Cause: No indexing; full serialize/deserialize on each add
- Current capacity: Up to 500 entries before truncation (line 78)
- Improvement path:
  1. At 500 entries, expect ~50KB JSON file; reads become 10-50ms on slow systems
  2. Switch to append-only log format for writes (immediate), read full file on startup only
  3. Or use SQLite for history instead of JSON files

**Audio Resampling Uses Nearest-Neighbor Interpolation:**
- Problem: Simple frame dropping for resampling (lines 344-357 in capture.rs) produces audio artifacts
- Files: `src-tauri/src/audio/capture.rs` (lines 344-357)
- Cause: Linear interpolation not implemented
- Impact: Audio quality degrades slightly; may affect Whisper accuracy on non-44.1kHz inputs
- Improvement path: Implement linear or cubic interpolation, or use a proper resampling library

**Mutex Lock Held During Audio Stream Setup:**
- Problem: Thread spawned (line 145-160 in lib.rs) holds `is_recording` lock while initializing audio
- Files: `src-tauri/src/lib.rs` (lines 120-163)
- Impact: If audio init takes 100ms, main thread blocked; UI freezes
- Improvement path: Set flag, spawn thread, let thread initialize and emit event on failure

**Settings Changes Require Full Hotkey Re-registration:**
- Problem: Any settings change triggers hotkey unregister/register cycle
- Files: `src-tauri/src/lib.rs` (lines 305-311)
- Impact: 100-200ms UI lag when user changes any setting
- Improvement path: Only unregister/register if hotkey key actually changed

---

## Fragile Areas

**macOS NSPanel Conversion Timing Dependent:**
- Files: `src-tauri/src/lib.rs` (lines 20-101, 666-668)
- Why fragile: NSPanel conversion happens in setup callback; if window not rendered yet, conversion may fail silently (line 70-72 logs error but doesn't retry)
- Risk: Window appears as regular NSWindow instead of NSPanel; can't appear over fullscreen apps
- Safe modification:
  1. Add retry logic in setup callback
  2. Test with fullscreen app (e.g., Final Cut Pro) at launch time
  3. Add diagnostic logging to detect when NSPanel conversion fails

**Dictionary Entry Updates Race Condition:**
- Files: `src-tauri/src/dictionary/mod.rs`
- Why fragile: Reading, modifying, and writing dictionary file without atomic transactions; concurrent updates via multiple windows cause data loss
- Risk: If user opens two settings windows and modifies dictionary in both, one change is lost
- Safe modification:
  1. Lock file before reading in dictionary functions
  2. Or migrate to SQLite
  3. Test scenario: Open Dashboard on two monitors, add entry in each simultaneously

**Audio Buffer Reused Between Recordings:**
- Files: `src-tauri/src/audio/capture.rs` (lines 64, 89)
- Why fragile: AUDIO_BUFFER not cleared if stop_capture called without matching start_capture; next recording includes previous data
- Risk: If user spams hotkey during startup, audio from previous session mixed into current recording
- Safe modification:
  1. Test rapid hotkey pressing (10 times in 1 second)
  2. Add assertions that AUDIO_THREAD is None before accepting new capture
  3. Add explicit state validation in start_capture

**Frontend State Not Synced on Backend Error:**
- Files: `src/lib/store.ts` (lines 46-69)
- Why fragile: Frontend optimistically updates state on updateSettings, but if backend fails, frontend and backend diverge
- Risk: Settings appear changed in UI but are reverted on app restart
- Safe modification:
  1. Add rollback in catch handler
  2. Or validate settings on load and emit warning if mismatch detected
  3. Test scenario: Corrupt settings file mid-update

---

## Scaling Limits

**Local History Limited to 500 Entries:**
- Current capacity: 500 transcriptions
- Limit: At 50KB per 500 entries, file I/O becomes bottleneck
- Scaling path:
  1. Migrate to SQLite with indexed queries
  2. Implement pagination with cursor-based loading
  3. Archive old entries (> 1 year) to separate storage

**Whisper Model Cache Holds Single Context:**
- Current capacity: One model in memory at a time
- Limit: 2.9GB for large model; switching models causes full reload
- Scaling path:
  1. If supporting multiple languages, keep 2-3 models cached
  2. Implement LRU eviction when memory threshold exceeded
  3. Or support streaming transcription to reduce peak memory usage

**Audio Buffer Unbounded if VAD Fails:**
- Current capacity: No limit; fills available RAM
- Limit: Can consume gigabytes on continuous background noise
- Scaling path:
  1. Implement 60-second max recording time with auto-stop
  2. Add memory usage monitoring and warn user if > 500MB
  3. Improve VAD to detect false speech triggers

---

## Dependencies at Risk

**whisper-rs (0.11) Depends on Old whisper.cpp:**
- Risk: whisper-rs tracks whisper.cpp infrequently; latest features/bug fixes may lag
- Impact: May miss important accuracy improvements or security patches
- Migration plan: Monitor whisper-rs releases; consider switching to whisper-rs if upstream becomes unmaintained; evaluate Rust-native alternatives like Candle's Whisper implementation

**tauri-nspanel (Git Dependency, Unstable):**
- Risk: Points to GitHub branch "v2"; not versioned; may break with Tauri updates
- Impact: macOS fullscreen overlay functionality fragile; breaks on Tauri 2.x minor updates
- Migration plan:
  1. Monitor tauri-nspanel for official releases
  2. Consider fallback to regular NSWindow if nspanel unavailable
  3. Test macOS compatibility on each Tauri update before releasing

**enigo (0.2) Has Limited Wayland Support:**
- Risk: Wayland adoption growing; Linux users on Wayland will fail to use text injection
- Impact: Linux Wayland users cannot use app at all
- Migration plan:
  1. Implement custom Wayland clipboard manager
  2. Or switch to wayland-client directly for text input
  3. Gracefully disable injection on Wayland with user-facing message

---

## Test Coverage Gaps

**No Tests for Audio Capture State Recovery:**
- What's not tested: Race conditions between start/stop capture; behavior when thread panics; state cleanup on app crash
- Files: `src-tauri/src/audio/capture.rs`
- Risk: Silent audio capture failures; corrupted audio buffers; memory leaks
- Priority: High - audio is core functionality

**No Tests for History Concurrent Updates:**
- What's not tested: Multiple threads adding entries simultaneously; file corruption on concurrent writes
- Files: `src-tauri/src/history/mod.rs`
- Risk: Lost transcriptions; corrupted history file
- Priority: High - user data loss

**No Tests for Settings Hotkey Re-registration:**
- What's not tested: Rapid hotkey changes; invalid hotkey codes; unregister failure handling
- Files: `src-tauri/src/lib.rs` (lines 305-311), `src-tauri/src/hotkey/mod.rs`
- Risk: Hotkey doesn't respond; app becomes unusable without restart
- Priority: Medium - breaks core functionality but fixable via settings change

**No Integration Tests for macOS NSPanel Conversion:**
- What's not tested: NSPanel created correctly; can appear above fullscreen apps; window positioning correct on multi-monitor setups
- Files: `src-tauri/src/lib.rs` (lines 20-101)
- Risk: Window doesn't appear over fullscreen apps (main feature on macOS); users blame app as broken
- Priority: High - breaks main feature on macOS

**No End-to-End Tests for Text Injection:**
- What's not tested: Text injection into real applications (TextEdit, VS Code, etc.); unicode handling; special characters
- Files: `src-tauri/src/injection/mod.rs`
- Risk: Silent failures; text doesn't appear in target app; unicode corruption
- Priority: High - user-facing feature, hard to debug

**No Crash Recovery Tests:**
- What's not tested: State after app crash during recording; recovery from corrupted history/settings files
- Files: Multiple
- Risk: Unrecoverable app state requiring manual file deletion
- Priority: Medium - rare but bad UX

---

## Missing Critical Features

**No Undo/Delete for Injected Text:**
- Problem: Once text injected into app, no way to undo within MentaScribe
- Blocks: Users cannot correct mistaken recordings without manual deletion in target app

**No Mic Level Monitoring Before Recording:**
- Problem: User doesn't know if mic is working until after recording finishes
- Blocks: Discovering mic issues requires full record-transcribe cycle (3-5 seconds)

**No Language Detection:**
- Problem: Must manually select language; no auto-detect for multilingual users
- Blocks: Seamless multilingual dictation experience

**No Support for Custom Whisper Models:**
- Problem: Limited to official whisper.cpp models (5 sizes)
- Blocks: Users with specialized models (medical, legal terms) cannot use app

**No Batch Transcription:**
- Problem: Can only transcribe one recording at a time
- Blocks: Processing multiple audio files or long recordings

---

*Concerns audit: 2026-02-18*
