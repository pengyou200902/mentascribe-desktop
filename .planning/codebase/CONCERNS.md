# Codebase Concerns

**Analysis Date:** 2026-02-20

## Critical Known Issue: Tauri macOS Mixed-DPI Coordinate Bug

**Issue:** Tauri/tao has a fundamental bug with coordinate spaces on macOS with mixed-DPI monitors
- Files: `src-tauri/src/lib.rs` (lines 875-1065), `src/App.tsx` (lines 150-180)
- Impact: Window positioning fails on multi-monitor setups with different scale factors (e.g., Retina laptop + external monitors)
- Symptoms: Dictation window does not follow mouse cursor correctly, snaps to wrong monitor positions
- Current mitigation: Native AppKit APIs used directly instead of tao for positioning:
  - `native_position_on_cursor_monitor()` (lines 875-967): Uses NSScreen and NSEvent.mouseLocation in AppKit coordinate space
  - `start_native_drag()` (lines 738-850): Direct NSEvent monitors for window dragging
  - All coordinates kept in AppKit space (bottom-left origin, Y increases upward)
- Why it persists: Tauri's coordinate conversion layer mixes logical (points) and physical (pixels) coordinates inconsistently
  - `cursor_position()` returns CG display points labeled as PhysicalPosition
  - `monitor.position()` converts CGDisplayBounds by monitor scale_factor
  - `monitor.size()` returns physical pixels
  - These overlap for mixed-DPI setups, making comparison unreliable
- Safe modification: Use `LogicalPosition` for `set_position()` (passes through without scale conversion); stay in AppKit coordinate space for all internal calculations
- Test coverage: Requires 3-monitor setup (Retina 2x + external 1x) to reproduce

## Architecture Fragility: NSPanel-NSWindow Mismatch

**Issue:** NSPanel created from NSWindow at runtime for fullscreen overlay support
- Files: `src-tauri/src/lib.rs` (lines 20-78, 738-850)
- Why fragile:
  - Panel must be created AFTER window is shown and rendered (line 1228)
  - Collection behavior, window level, and style mask must be re-applied after window visibility changes (lines 1083-1084)
  - NSPanel non-activating mask (128) prevents native macOS window dragging—requires manual event monitor workaround
- Safe modification:
  - Always call `refresh_panel_settings()` after `window.show()`
  - Verify panel pointer validity before sending events (store as usize, not direct reference)
  - Monitor for macOS Sonoma+ API changes in NSPanel behavior

## Large Component Risk: SettingsPage (1573 lines)

**Issue:** Single component handles all settings UI
- Files: `src/components/dashboard/SettingsPage.tsx` (1573 lines)
- Impact: Hard to maintain, test, and modify; many inline icon definitions and state management
- Fragile areas:
  - Model management (download, delete, detect CoreML/Metal support)—no error recovery if partial download fails
  - Theme toggle affecting entire app state—no error boundary
  - Multiple event listeners that may stack if component remounts
  - Hotkey input validation happens client-side only; no validation in Rust backend
- Fix approach: Extract into sub-components (ModelManager, ThemeSettings, HotkeySettings) with isolated state

## State Management: Non-resilient Recording State

**Issue:** Recording state split between frontend (React) and backend (Rust Mutex)
- Files: `src-tauri/src/lib.rs` (lines 143-365), `src/App.tsx` (lines 64-139)
- Current pattern:
  - Frontend uses refs (isRecordingRef, isProcessingRef) to prevent duplicate invoke calls
  - Backend uses Mutex<bool> (is_recording)
  - Frontend event listeners don't validate backend state before starting
  - If backend hangs or crashes, frontend state becomes stale
- Risk: User presses hotkey while backend is stuck; frontend thinks it's recording but backend isn't
- Workaround in place: `reset_recording_state()` command (lines 349-365) allows frontend to force reset
- Fix approach:
  - Add heartbeat channel from backend to frontend (emit `recording-state-sync` every 500ms)
  - Frontend validates its state against backend on hotkey press
  - Implement timeout-based auto-reset in backend if stop_recording hangs >10s

## Audio Processing: Silent Failure in Real-time Resampling

**Issue:** Real-time audio resampling can fail silently and fall back to post-stop processing
- Files: `src-tauri/src/audio/capture.rs` (lines 198-235, 286-314)
- Impact: VAD/streaming transcription may miss data if resampler creation fails
- Current behavior:
  - If `FastFixedIn` creation fails (line 220), falls back gracefully
  - If resampler process fails during callback (line 123-128), marks state as failed
  - Backend logs warning but transcription continues (line 222-224)
  - No frontend notification that real-time processing is degraded
- Safe modification:
  - Emit event to frontend if real-time resampling unavailable (skip streaming, process entire recording on stop)
  - Add `prepare_for_whisper()` validation before transcription attempt

## History/Stats Data Races

**Issue:** History and stats use load-then-save pattern without locking
- Files: `src-tauri/src/history/mod.rs` (lines 36-60), `src-tauri/src/stats/mod.rs`
- Impact: Two simultaneous transcriptions could overwrite each other's history entries
- Example race:
  1. Transcription 1: `load_history_data()` → reads 100 entries
  2. Transcription 2: `load_history_data()` → reads same 100 entries
  3. Transcription 1: `add_entry()`, `save_history_data()` → writes 101 entries
  4. Transcription 2: `add_entry()`, `save_history_data()` → writes 101 entries (overwrites T1)
- Current guards: Implicitly assumes single-threaded transcription (enforced by AppState.is_recording Mutex)
- Safe modification: Use file-level atomic writes (write to temp file, atomically rename) or explicit file lock (flock on Unix, LockFile crate)
- Test coverage: Unit tests don't cover concurrent add_entry calls

## Transcription: Streaming State Not Cleaned Up on Error

**Issue:** VAD streaming transcription state may persist after transcription error
- Files: `src-tauri/src/transcription/whisper.rs` (lines 174-177, 233-246)
- Impact: If transcription fails mid-stream, streaming state remains in memory; next recording may process stale audio
- Example: Network error during cloud transcription → streaming thread still running → next recording has extra latency
- Fix approach:
  - Add `ensure_streaming_stopped()` called in `stop_recording()` error handler
  - Return error from `stop_streaming()` if unexpected state detected
  - Add timeout (10s) for streaming monitor if VAD stuck

## Injection: Accessibility Check Not Called Before Typing

**Issue:** Text injection doesn't verify accessibility permissions before attempting type/paste
- Files: `src-tauri/src/injection/mod.rs` (lines 58-64, 97+)
- Impact: Silent failure if app loses accessibility permission between checks
- Current flow: Settings page checks once at load, then assumes permission persists
- Safe modification:
  - Check accessibility in `inject_text()` command before attempting injection
  - Return specific `InjectionError::AccessibilityPermissionRequired` on failure
  - Frontend shows user-friendly message with link to System Settings

## Frontend: Event Listener Cleanup Issues

**Issue:** Multiple event listeners in components may accumulate without proper cleanup
- Files: `src/components/DictationBar.tsx` (lines 62-103), `src/App.tsx` (lines 182-223)
- Pattern: `listen()` used instead of `onEvent()` with proper unlisten tracking
- Risk: Component remounts (e.g., during settings changes) could create duplicate listeners
- Example from DictationBar:
  ```typescript
  const unlistenFocus = window.onFocusChanged(...)
  return () => { unlistenFocus.then(fn => fn()); }
  ```
  If this effect runs twice, two listeners exist until first cleanup
- Safe modification: Use cleanup callbacks properly, or switch to window event subscription model

## Hotkey Management: No Hotkey Conflict Detection

**Issue:** Users can configure hotkeys that conflict with OS-level hotkeys
- Files: `src-tauri/src/hotkey/mod.rs`, `src/components/dashboard/SettingsPage.tsx` (hotkey input section)
- Impact: User sets hotkey to Cmd+C (copy), system copy breaks; no feedback given
- Current validation: Frontend accepts any key string; backend tries to register
- Safe modification:
  - Maintain list of reserved macOS hotkeys (Cmd+Q, Cmd+W, Cmd+Tab, etc.)
  - Validate in SettingsPage before allowing save
  - Add platform-specific warning message to user

## Zustand Store: No Error Recovery

**Issue:** Settings store doesn't persist loading state or error messages
- Files: `src/lib/store.ts` (lines 53-76)
- Impact: If settings load fails, store remains in isLoading=true state forever (until manual refresh)
- Current behavior: Error logged to console, but user sees no visual feedback
- Fix approach:
  - Add `error: string | null` to store state
  - Emit error event on load/save failure
  - Auto-retry load after 5s with exponential backoff
  - Show error banner in UI with retry button

## Missing Model Error Handling

**Issue:** Model download can fail partially, leaving corrupted model files
- Files: `src-tauri/src/transcription/whisper.rs` (model download functions)
- Impact: Next transcription attempt fails; user must manually delete corrupted model
- Current validation: Frontend shows "Model not found" error
- Safe modification:
  - Validate model file integrity (checksum/size) before using
  - Clean up partial downloads on failure (add `.tmp` extension during download, rename on completion)
  - Return specific error "Model corrupted" vs "Model missing" to frontend

## Performance: 150ms Monitor Poll on Every Frame

**Issue:** Dictation window polls monitor position every 150ms (App.tsx line 173)
- Files: `src/App.tsx` (lines 150-180)
- Impact: Unnecessary CPU/battery usage on macOS (especially on battery)
- Current check: Only moves if window is on different monitor
- Safe modification:
  - Increase poll interval to 500-1000ms (user unlikely to move between monitors faster)
  - Or use NSWorkspace notifications instead of polling (requires native code)
  - Log skipped repositions to detect if this is ever actually needed

## Settings Serialization: Missing Default Value Handling

**Issue:** Optional settings fields may deserialize as null instead of using Rust defaults
- Files: `src-tauri/src/settings/mod.rs` (multiple Option<String> fields)
- Example: `model_size: Option<String>` deserializes as None even if default exists
- Impact: Users upgrading from old config versions may lose settings
- Current workaround: Frontend provides fallback values (e.g., line 122 in App.tsx)
- Safe modification:
  - Use `#[serde(default = "...")]` for all Option fields
  - Add migration function in `load_settings()` to fill in missing fields
  - Log when defaults are applied for first run detection

## Missing Test Coverage

**Key untested areas:**
- Concurrent transcription attempts (handled by Mutex but not tested)
- History file corruption recovery
- Settings persistence across app restarts
- Network errors in cloud transcription
- VAD streaming with very short utterances (<100ms)
- Multi-monitor repositioning with mixed DPI ratios
- Hotkey registration conflicts on different languages/keyboard layouts

**Test infrastructure:** No test files found in codebase
- Files: Would live in `src/__tests__` and `src-tauri/tests/`
- Priority: Add integration tests for recording pipeline (start → audio capture → transcription → injection)

## Unsafe Code: NSPanel Pointer Arithmetic

**Issue:** NSPanel pointer stored as usize for Send compatibility
- Files: `src-tauri/src/lib.rs` (lines 618, 763, 769-776)
- Safety: Relies on Apple's guarantee that NSPanel* remains valid for lifetime of drag operation
- Risk: If panel is deallocated during drag (shouldn't happen, but no guarantee), dereferencing is UB
- Safe modification:
  - Add guard to check if panel still exists before setFrameOrigin
  - Use try_lock before accessing panel to detect if another thread deallocated it
  - Add timeout (5s) for drag operations to prevent indefinite holding of stale pointer

## Audio Buffering: Hard-coded 30-second Limit

**Issue:** Audio buffers pre-allocate for max 30 seconds
- Files: `src-tauri/src/audio/capture.rs` (lines 152-162)
- Impact: Recordings longer than 30s will reallocate buffers mid-stream (performance penalty)
- Safe modification: Estimate buffer size based on available memory, or stream audio to disk after 30s

---

*Concerns audit: 2026-02-20*
