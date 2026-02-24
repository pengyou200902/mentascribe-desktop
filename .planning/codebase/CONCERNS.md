# Codebase Concerns

**Analysis Date:** 2026-02-24

## Critical Bugs

**Tauri macOS Mixed-DPI Coordinate System Bug (Issue #7890):**
- Problem: `cursor_position()` returns CG display **points** but is labeled as `PhysicalPosition`. For mixed-DPI monitors (e.g., 2x Retina + 1x external), coordinate spaces overlap, causing widget positioning to be wildly incorrect when cursor moves between monitors.
- Files: `src-tauri/src/lib.rs` (lines 1050-1150, coordinate conversion functions), `src-tauri/src/hotkey/mod.rs` (monitor detection)
- Cause: Tauri's abstraction leaks platform details. `monitor.position()` converts CGDisplayBounds points × each monitor's own scale_factor, but these values are incompatible with each other when mixed-DPI.
- Impact: Widget snaps to wrong position, jumps between monitors incorrectly, drag operations fail.
- Fix approach:
  1. Create helper functions `monitor_origin_points()` and `monitor_size_points()` that divide by scale_factor to normalize everything to CG point space
  2. Always use `LogicalPosition` for `set_position()` (passes through without scale conversion)
  3. Convert all coordinate comparisons to CG point space before testing cursor position
  4. Detailed workaround documented in `./memory/tauri-coordinates.md`

**NSPanel Focus-Stealing Workaround (NSNonactivatingPanelMask = 128):**
- Problem: While NSNonactivatingPanelMask prevents focus stealing, it also breaks native macOS window dragging. Manual drag workaround implemented via JS → `start_native_drag` → NSEvent monitors.
- Files: `src-tauri/src/lib.rs` (line 35, constant definition), `src/components/DictationBar.tsx` (lines 162-172, drag handler)
- Impact: Dragging widget requires constant Rust polling of NSEvent coordinates, adding 150ms latency per position update. Window feels sluggish when dragged.
- Fix approach: Investigate if a lower window level (not 25) or different collection behavior would allow native dragging while still appearing above fullscreen apps. Currently no known alternative.

**Audio Capture Thread Race Condition (IS_STOPPING Flag):**
- Problem: `IS_STOPPING` flag set in `stop_capture()` prevents `start_capture()` from running, but if `start_capture()` fails, the flag is never cleared, permanently blocking future recordings.
- Files: `src-tauri/src/audio/capture.rs` (lines 60-61 flag definition, 138-142 check, 370-371 error handling)
- Symptoms: After a failed `start_capture()`, all subsequent recordings fail with "AlreadyRunning" error.
- Trigger: Call `start_capture()` while `stop_capture()` is already running (rare but possible during rapid hotkey taps).
- Workaround: `reset_recording_state()` command clears the flag, but users must manually invoke it.
- Fix approach: Ensure `IS_STOPPING` flag is **always** cleared in `stop_capture()` finally block, or restructure as a state machine with explicit states (Idle, Recording, Stopping).

## Tech Debt

**JSON File-Based Persistence (No Database):**
- Issue: All persistent data (history, settings, stats, dictionary) stored as plain JSON files read/written atomically.
- Files: `src-tauri/src/history/mod.rs` (lines 36-60, load/save), `src-tauri/src/settings/mod.rs`, `src-tauri/src/dictionary/mod.rs`, `src-tauri/src/stats/mod.rs`
- Impact:
  - No concurrency safety — multiple simultaneous writes corrupt files
  - History operations O(n) — full load/save even for single-entry adds
  - No query capability — all data loaded into memory
  - No schema versioning — breaking changes require manual migration
- Improvement path: Migrate to SQLite with proper locking (rusqlite with WAL mode). Phased migration: history first (~500 entries), then settings, then dictionary/stats.

**Monolithic lib.rs with Many Responsibilities:**
- Issue: `src-tauri/src/lib.rs` is 1,662 lines containing panel setup, window management, recording lifecycle, settings, hotkey handling, menu events, drag handling.
- Files: `src-tauri/src/lib.rs` (entire file)
- Impact: Hard to test individual concerns, changes in one area break others, difficult to understand flow.
- Improvement path: Extract into modules: `window_manager.rs` (panel + window setup), `recording_lifecycle.rs` (start/stop logic), `event_handlers.rs` (menu/tray/hotkey). Keep main only for Tauri builder setup.

**Lazy Static Global Mutable State (Audio Capture):**
- Issue: Seven `lazy_static!` mutexes manage audio state: `AUDIO_BUFFER`, `WHISPER_BUFFER`, `AUDIO_THREAD`, `SAMPLE_RATE`, `CHANNELS`, `CURRENT_AUDIO_LEVEL`, `IS_STOPPING`, `RESAMPLER_STATE`.
- Files: `src-tauri/src/audio/capture.rs` (lines 52-65)
- Impact: Hard to reason about state, impossible to unit test without global side effects, no clear ownership.
- Improvement path: Wrap in a `AudioCaptureState` struct managed by a command handler state. Single Mutex<AudioCaptureState> instead of seven separate statics.

**Repeated Settings Lock/Clone Cycles:**
- Issue: Throughout `lib.rs`, pattern: `state.settings.lock()` → read/clone → release → later `lock()` again for update.
- Files: `src-tauri/src/lib.rs` (lines 447-456, 472-473, etc.)
- Impact: Risk of TOCTOU bugs, hard to reason about consistency, verbose.
- Improvement path: Provide helper functions like `read_settings()` and `update_settings()` that handle lock/clone internally. Or use RwLock for read-heavy workloads.

**Unsafe FFI Bindings Without Validation:**
- Issue: `CGEventKeyboardSetUnicodeString` called with raw pointers; `AXUIElementCopyAttributeValue` uses raw C pointers; objc `msg_send!` uses unsafe blocks.
- Files: `src-tauri/src/injection/mod.rs` (lines 29-33 FFI, 92-168 unsafe block in `try_ax_insert`, 256-275 unsafe in `type_unicode_chunk`)
- Impact: If invalid pointers passed or API contracts violated, memory corruption/crashes (rare but possible under extreme load).
- Fix approach: Add defensive checks for null pointers before dereferencing. Document preconditions for each unsafe block. Consider using higher-level macOS bindings where available.

**String Trimming of [BLANK_AUDIO] Markers:**
- Issue: `src-tauri/src/injection/mod.rs` (lines 700-703) manually strips `[BLANK_AUDIO]` markers output by Whisper. This assumes Whisper always uses exact marker strings.
- Files: `src-tauri/src/injection/mod.rs` (lines 700-703)
- Impact: If Whisper output changes format (e.g., `[BLANK AUDIO]` with space), markers leak into injected text. Silent data corruption.
- Fix approach: Add unit test with known Whisper outputs. Document expected marker formats. Consider regex matching instead of exact string replacement.

## Fragile Areas

**Window Dragging Implementation (NSEvent Polling):**
- Files: `src-tauri/src/lib.rs` (lines 1080-1150, `start_native_drag` function), `src/components/DictationBar.tsx` (lines 165-172, drag initiation)
- Why fragile: Relies on non-public Objective-C APIs (`NSEventMonitor`, undocumented behavior). Coordinate system conversion is error-prone (mixed-DPI). If NSEvent behavior changes in future macOS, dragging breaks.
- Safe modification: Any changes to drag logic must account for mixed-DPI monitors (see Critical Bugs section). Add integration test with cursor at different monitor positions. Use Tauri coordinate helpers consistently.
- Test coverage: No tests for drag functionality. Manual testing only.

**Injection Method Fallback Chain (macOS):**
- Files: `src-tauri/src/injection/mod.rs` (lines 750-825, `inject_auto_macos` function)
- Why fragile: Three-tier fallback (AX API → CGEvent typing → clipboard paste). Each tier can silently fail and fall through to next. If clipboard tier fails but returns Ok, user doesn't know text wasn't injected. Terminal app detection uses hardcoded bundle ID list (lines 311-324).
- Safe modification: Add explicit error reporting for each tier. Add bundle ID detection via app's actual capabilities rather than hardcoded list. Log which tier succeeded.
- Test coverage: No unit tests for tier selection or fallback logic. Only tested manually per app.

**Real-Time Audio Resampling (rubato Integration):**
- Files: `src-tauri/src/audio/capture.rs` (lines 200-235 resampler creation, 286-304 callback processing, 384-446 flush)
- Why fragile: Resampler created in audio thread, accessed via Arc<Mutex> from CPAL callback. If callback mutex times out (via try_lock), samples silently drop. Resampler failure marked as failed; if flush fails, pre-processed buffer discarded.
- Safe modification: Ensure resampler state transitions are logged. Add buffer underrun detection. Test with various sample rates (48kHz, 44.1kHz, 16kHz).
- Test coverage: No unit tests for resampling edge cases (partial chunks, rate changes mid-recording).

**Multi-Window State Synchronization (Dictation + Dashboard):**
- Files: `src/App.tsx` (lines 155-159 settings-changed listener), `src-tauri/src/lib.rs` (line 493, emit settings-changed), `src/components/DictationBar.tsx` (lines 60-62 log draggable changes)
- Why fragile: Both windows listen to settings-changed event, but there's no guarantee of order or delivery. If dashboard updates settings faster than dictation reads them, stale values used. Draggable prop changes cause entire DictationBar re-mount.
- Safe modification: Use Zustand store for single source of truth (already done). Ensure all settings changes go through store, not direct Rust commands. Add test asserting both windows see consistent settings.
- Test coverage: No tests for multi-window consistency.

## Performance Bottlenecks

**History Loading (Full File Read on Every Operation):**
- Problem: `get_history()`, `add_entry()`, `delete_entry()` all call `load_history_data()` which reads entire JSON file, parses, searches, re-serializes, re-writes.
- Files: `src-tauri/src/history/mod.rs` (lines 62-83 add_entry, 85-98 get_history)
- Cause: File-based persistence, no indexing.
- Improvement: For 500 entries, each operation is ~5ms (JSON parse) + disk latency. Add 10+ operations per session = 50ms+ wasted.
- Scaling: With 10,000 entries, parse time grows to 50ms+. Beyond 100,000 entries, unusable.
- Improvement path: Migrate to SQLite with indexed queries. Single `INSERT` ~1ms, `SELECT LIMIT 50` ~0.5ms.

**Settings Lock Contention on Recording:**
- Problem: Audio level emitter spawned in `start_recording()` periodically locks `app_state.settings` (indirectly via Zustand). Every 25ms the main recording thread might contend.
- Files: `src-tauri/src/lib.rs` (lines 205-220, audio level emitter), `src/lib/store.ts` (settings state)
- Cause: Global mutable state accessed from multiple threads.
- Impact: Negligible on modern machines, but under extreme load (very fast recordings) could cause audio thread stalls.
- Improvement path: Pass immutable settings snapshot to audio thread at start_recording time rather than reading live.

**Cursor Proximity Polling (Every 150ms):**
- Problem: `DictationBar.tsx` polls `is_cursor_over_pill` every 150ms (CURSOR_POLL_INTERVAL_MS) even when window not visible.
- Files: `src/components/DictationBar.tsx` (lines 78-86), `src/config/widget.ts` (CURSOR_POLL_INTERVAL_MS setting)
- Cause: JS mouse events don't fire when NSPanel doesn't have focus, so Rust polling required.
- Impact: 6-7 Rust invocations per second from every window instance (dictation + dashboard if both open). Not heavy but unnecessary overhead.
- Improvement path: Only poll when window is visible. Poll interval could increase to 200-300ms without UX impact (hover detection latency would still be < 400ms).

**Model Preloading on Separate Thread (Blocks Startup):**
- Problem: `preload_model()` called in background thread, but user can't record until it completes (error if model not loaded).
- Files: `src-tauri/src/lib.rs` (lines 1500-1547, preload logic)
- Impact: Startup latency visible to user — app appears ready but recording fails if attempted before preload completes. Preload for large models (base/medium) takes 3-5 seconds.
- Improvement path: Start preload earlier (maybe async during model download). Show loading indicator in UI. Allow recording with smaller fallback model while large model preloads.

## Security Considerations

**Accessibility API Permissions Not Enforced:**
- Risk: On macOS, if user denies Accessibility permission, injection defaults to clipboard method without warning user it falls back (security downgrade).
- Files: `src-tauri/src/injection/mod.rs` (lines 717-727, permission check), `src-tauri/src/lib.rs` (lines 1180-1220, inject_text command)
- Current mitigation: Permission check happens, but fallback is silent. User sees error only if clipboard method also fails.
- Recommendations:
  1. Log and display warning if AX permission denied
  2. Add UI setting "Prefer AX API for security" with user education
  3. On first injection without AX, show one-time prompt explaining why AX is safer

**Clipboard Data Temporarily Exposed:**
- Risk: When using clipboard injection, original clipboard contents are saved to local variables. If process crashes between paste and restore, original clipboard lost.
- Files: `src-tauri/src/injection/mod.rs` (lines 329-445 clipboard_save_paste_restore)
- Current mitigation: Change count monitoring detects if user copies during paste window (150ms). Saved data held in Rust structs, not persisted.
- Recommendations:
  1. Reduce paste window from 150ms to <100ms if possible
  2. Document this behavior in UX (briefly display "clipboard saved" message)
  3. Consider persistent clipboard backup in case of crash

**No Input Validation on Text Injection:**
- Risk: `inject_text` accepts any string without validation. Extremely long text (10MB+) could cause memory issues or injection failures.
- Files: `src-tauri/src/injection/mod.rs` (lines 686-747, inject_text entry point)
- Current mitigation: Windows has 10,000 event limit (line 542), chunking prevents overflow.
- Recommendations:
  1. Add max length check (suggest 500KB limit for safety)
  2. Add timeout for injection operations
  3. Log if text exceeds expected sizes

**Voxtral Model Files Not Verified:**
- Risk: Model files downloaded from HuggingFace but no checksum validation. MITM attack could inject malicious model.
- Files: `src-tauri/src/transcription/voxtral.rs` (lines 114-150, download_model)
- Current mitigation: Size check (lines 131-138) ensures roughly correct file size.
- Recommendations:
  1. Add SHA256 checksum verification from HuggingFace metadata
  2. Sign checksum with Hugging Face public key if available
  3. Log model file paths for user inspection

**Settings File Contains No Schema Validation:**
- Risk: Manually edited settings JSON with invalid values (negative opacity, invalid enum values) not validated on load.
- Files: `src-tauri/src/settings/mod.rs` (settings load), `src/lib/store.ts` (frontend settings)
- Current mitigation: TypeScript types on frontend, but Rust side accepts any JSON.
- Recommendations:
  1. Add validation layer on settings load
  2. Return error if settings invalid instead of silently using defaults
  3. Provide migration function for breaking schema changes

## Scaling Limits

**History File Grows Without Bound (500 entry cap, not enforced):**
- Current capacity: 500 entries = ~100KB JSON (assuming ~200 bytes per entry)
- Limit: At 10,000 entries (~2MB file), JSON parse time becomes noticeable (50ms+). File I/O stalls recording.
- Limit reached: ~100,000 entries (20MB+ file), parse time > 200ms.
- Scaling path: Migrate to SQLite immediately if planning to support users with years of history. Add pagination UI so users don't load all history at once.

**Audio Buffers Pre-Allocated Per Recording:**
- Current capacity: 30 seconds max recording at 48kHz stereo = ~11.5MB allocated, freed on stop.
- Limit: With 10+ concurrent recordings or very high sample rates, could hit memory limit on resource-constrained machines.
- Limit reached: Rare on modern machines, but possible on older MacBook Air models.
- Scaling path: Make buffer size configurable. Consider ring buffer to avoid peak allocation. Stream to disk instead of buffering entire recording.

**Dashboard History View Loads All Entries Into DOM:**
- Current capacity: 50 entries shown per page (configured in code), but entire history loaded from backend on initial render.
- Limit: With 10,000 entries, render can pause for 1-2 seconds during list virtualization.
- Limit reached: ~50,000 entries, React reconciliation becomes noticeably slow.
- Scaling path: Implement pagination backend-side (already partially done with limit/offset). Use React virtual scrolling to render only visible items.

**Whisper Model Sizes:**
- Current capacity: Small model fits in VRAM on most systems (~1.5GB VRAM required). Medium/large models may OOM on 4GB systems.
- Limit: Large model requires 6GB+ VRAM. Tiny model fast but poor accuracy.
- Scaling path: Add model benchmarking in Settings (show VRAM available). Auto-select appropriate model size.

## Missing Critical Features

**No Network Sync of Transcriptions:**
- Problem: History marked with `synced: bool` field but no backend sync implemented. User switches machines, loses all history.
- Files: `src-tauri/src/history/mod.rs` (line 22, synced field), `src-tauri/src/api/client.rs` (partially implemented)
- Impact: Users cannot use app on multiple machines effectively. Data loss risk if machine dies.

**No User Preferences for Model Selection Per-Machine:**
- Problem: Settings global — user forced to use same model size on MacBook Air and Mac Studio (vastly different performance characteristics).
- Files: `src-tauri/src/settings/mod.rs` (UserSettings structure)
- Impact: Poor performance on resource-constrained machines, wasted VRAM on powerful machines.

**No Keyboard Shortcut Configuration for Text Injection Method:**
- Problem: Users cannot quickly switch between injection methods (AX API vs typing vs clipboard) without going to Settings.
- Files: `src-tauri/src/injection/mod.rs`, `src/components/dashboard/SettingsPage.tsx`
- Impact: For advanced users, friction when switching between apps with different injection method needs.

**No Undo/Redo for Injected Text:**
- Problem: Once text injected, user cannot undo without using app's native undo. MentaScribe has no undo.
- Files: `src-tauri/src/injection/mod.rs` (no undo tracking)
- Impact: Users lose work if app injects incorrectly (rare but possible with injection method fallback failures).

## Test Coverage Gaps

**No Tests for Mixed-DPI Monitor Handling:**
- What's not tested: Cursor position detection across monitors with different scale factors. Widget repositioning when cursor moves monitors.
- Files: `src-tauri/src/lib.rs` (lines 1050-1150, coordinate functions)
- Risk: This is the critical bug — no regression tests prevent it from being reintroduced. Simulator can't replicate mixed-DPI.
- Recommendation: Add integration test with mocked monitor layout data. Or document manual testing procedure for users with mixed-DPI setups.

**No Unit Tests for Injection Method Fallback:**
- What's not tested: AX API failure → CGEvent fallback → clipboard fallback. Terminal app detection.
- Files: `src-tauri/src/injection/mod.rs` (lines 750-825, inject_auto_macos)
- Risk: Changes to fallback logic could silently break text injection for specific app types.
- Recommendation: Create mock AX API to test each tier independently. Add test for terminal app detection against real app bundle IDs.

**No Tests for Audio Resampling Edge Cases:**
- What's not tested: Partial chunks, resampler failures, buffer underruns, various sample rates.
- Files: `src-tauri/src/audio/capture.rs` (lines 200-235, 286-304)
- Risk: Audio quality degradation or transcription failure under edge conditions (high sample rate, network lag causing VAD delays).
- Recommendation: Create synthetic audio test data at various rates. Mock CPAL callback failures. Test resampler flush with incomplete final chunk.

**No Integration Tests for Recording Lifecycle:**
- What's not tested: start → audio input → streaming transcription → stop → transcribe tail → inject → save to history. End-to-end flow.
- Files: `src-tauri/src/lib.rs` (start_recording, stop_recording), entire transcription pipeline
- Risk: Refactoring any part breaks the full flow. Regressions in one module cascade to UI.
- Recommendation: Create mock audio device that plays known audio. Assert transcription output and history side effects.

**No Tests for Settings Persistence:**
- What's not tested: Settings saved → load → verify consistency across app restart. Partial corruption recovery.
- Files: `src-tauri/src/settings/mod.rs`, `src/lib/store.ts`
- Risk: Settings accidentally lost on app update or file system corruption.
- Recommendation: Test load/save round-trip. Test with manually corrupted JSON. Test migration paths.

**No Tests for Multi-Window Synchronization:**
- What's not tested: Settings change in dashboard → emitted to dictation window → DictationBar re-renders with new values.
- Files: `src/App.tsx`, `src-tauri/src/lib.rs` (emit settings-changed)
- Risk: Windows get out of sync, user sees stale state in dictation bar while settings show new values.
- Recommendation: Add React testing library test for App.tsx listening to settings-changed event. Mock Tauri events.

---

*Concerns audit: 2026-02-24*
