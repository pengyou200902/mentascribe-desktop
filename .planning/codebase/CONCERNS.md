# Codebase Concerns

**Analysis Date:** 2026-02-26

## Tech Debt

### Cloud Transcription APIs Not Implemented

**Issue:** Cloud transcription providers (OpenAI Whisper, AWS Transcribe, AssemblyAI) are stubbed but non-functional.

**Files:** `src-tauri/src/transcription/cloud.rs`

**Impact:** Users cannot use cloud-based transcription fallback. Settings accept `cloud_provider` configuration but invocation returns "not yet implemented" errors.

**Fix approach:**
- Implement multipart form upload for OpenAI Whisper API in `transcribe_openai()` (lines 52-66)
- Add AWS Transcribe SDK integration in `transcribe_aws()` (lines 68-77)
- Add AssemblyAI API client in `transcribe_assemblyai()` (lines 79-87)
- Each should handle auth via environment variables and return transcribed text

### Global Mutable State via lazy_static / once_cell

**Issue:** Extensive use of global `Mutex`-wrapped statics for transcription, audio, and UI state.

**Files:**
- `src-tauri/src/audio/capture.rs` (lines 52-65): `AUDIO_BUFFER`, `WHISPER_BUFFER`, `AUDIO_THREAD`, `SAMPLE_RATE`, `CHANNELS`, `CURRENT_AUDIO_LEVEL`, `IS_STOPPING`, `RESAMPLER_STATE`
- `src-tauri/src/transcription/whisper.rs` (lines 21, 38, 480, 597, 601, 609): `MODEL_CACHE`, `STATE_CACHE`, `VAD_CACHE`, `STREAMING_RESULTS`, `STREAMING_CONSUMED`, `VAD_MONITOR`
- `src-tauri/src/transcription/voxtral.rs` (lines 218, 341, 353): `VOXTRAL_CACHE`, `VOXTRAL_STREAMING_RESULTS`, `VOXTRAL_STREAM_HANDLE`
- `src-tauri/src/lib.rs` (line 829): `NATIVE_DRAG_STATE`

**Impact:**
- Difficult to reason about state ownership and initialization order
- Risk of deadlocks if multiple threads try to lock related statics simultaneously
- State cleanup on app exit is implicit (relies on OS cleanup)
- Cannot easily unit test code that depends on these statics

**Fix approach:**
- Consolidate audio/capture state into a single `AudioCaptureState` struct in `AppState`
- Move transcription model caches into `AppState` with explicit lifetime management
- Create a dedicated `NativeDragManager` struct to encapsulate drag state instead of static `Mutex`
- Use `.ok()` error suppression more strategically for non-critical lock failures

## Known Bugs

### macOS Mixed-DPI Coordinate Space Bug (Tauri #7890)

**Symptoms:** Dictation panel positioning fails or moves to wrong monitor on systems with mixed-DPI displays (e.g., Retina laptop + external 1x monitors).

**Files:** `src-tauri/src/lib.rs` (lines 1068-1175: `native_position_on_cursor_monitor()`)

**Trigger:** Using cursor movement to trigger repositioning on multi-monitor setups where monitors have different pixel densities.

**Current mitigation:** Fully bypassed by using NSEvent.mouseLocation + NSScreen APIs directly in AppKit coordinate space (bottom-left origin). This avoids Tauri's buggy coordinate conversion layer. Code is working but demonstrates the underlying platform incompatibility.

**Remaining risk:** If code switches back to Tauri's `cursor_position()` or `set_position()` methods, the bug will resurface. Comments at lines 1073-1075 document the specific Tauri issues.

## Security Considerations

### API Key Storage in Settings

**Risk:** Cloud provider API keys are stored in `src-tauri/src/settings/mod.rs` `CleanupSettings.api_key` field, serialized to disk as JSON.

**Files:** `src-tauri/src/settings/mod.rs` (lines 30-40)

**Current mitigation:** Settings file lives in `~/.config/mentascribe/settings.json` (user-only permissions).

**Recommendations:**
- Move sensitive API keys to system keychain (SecItem on macOS, Credential Manager on Windows)
- Encrypt API keys at rest using a per-machine key
- Never log or print API keys (add `#[serde(skip)]` attribute)
- Document security model in settings module

### Unsafe FFI Blocks

**Risk:** Multiple `unsafe` blocks calling Objective-C runtime APIs and C FFI functions without comprehensive bounds checking.

**Files:**
- `src-tauri/src/lib.rs` (lines 850-928, 956-1048): NSEvent monitor installation, panel frame queries, setFrameOrigin calls
- `src-tauri/src/injection/mod.rs` (lines 53, 92, 258, 285, 333, 471, 544): AX APIs, CGEvent, Unicode string injection, clipboard access
- `src-tauri/src/audio/capture.rs` (lines 274-320): CPAL stream initialization

**Current mitigation:** Most unsafe blocks are properly documented with preconditions. Panel pointer is validated as `usize` before use.

**Recommendations:**
- Wrap unsafe FFI in validated newtype structs (e.g., `SafeNSPanel(usize)`)
- Add runtime assertions before msg_send! calls where object pointers are involved
- Consider using higher-level bindings (e.g., `cocoa` crate) instead of raw obj-c for message sends

## Performance Bottlenecks

### Audio Resampling Fallback on Real-time Failure

**Problem:** If rubato resampler fails to process chunks in the CPAL callback, transcription falls back to post-stop resampling, which blocks stop_recording().

**Files:** `src-tauri/src/audio/capture.rs` (lines 114-133: `drain_resampler()`, 220-227: fallback logging)

**Cause:** Real-time resampling is I/O-bound and CPU-intensive; if it stalls, the callback's `try_lock()` on `RESAMPLER_STATE` will silently fail for some chunks.

**Improvement path:**
- Pre-allocate resampler output buffers to reduce allocations in hot path
- Add metrics to track resampling success rate
- Consider alternative resampling libraries (sinc vs polynomial) if cubic resampling is bottleneck
- Profile callback timing to identify jitter sources

### VAD (Voice Activity Detection) Streaming Interval

**Problem:** Streaming transcription polls whisper_samples every 500ms (line 706 in whisper.rs). On slow hardware, VAD inference itself may take 300-500ms, causing back-to-back waits.

**Files:** `src-tauri/src/transcription/whisper.rs` (lines 682-862: `vad_monitor_loop()`)

**Cause:** Hard-coded 500ms interval doesn't adapt to inference latency; no adaptive polling strategy.

**Improvement path:**
- Measure VAD inference time and adjust poll interval dynamically
- Queue multiple audio chunks ahead of time for overlapping processing
- Consider lower-overhead VAD alternatives (e.g., Silero VAD if available in rustpython)

## Fragile Areas

### Transcription Mode Switching (Voxtral ↔ Whisper)

**Files:** `src-tauri/src/lib.rs` (lines 501-538: model switching in `update_settings()`)

**Why fragile:**
- Switching engines during update_settings spawns background preload but doesn't block or await
- If user starts recording before preload completes, wrong engine may be active
- No state machine to enforce valid transitions (can't switch from recording → stopped → setting change → recording smoothly)
- Voxtral unload (line 536) is a fire-and-forget call with no error handling

**Safe modification:**
- Prevent settings updates while recording (check `is_recording` state before allowing change)
- Wait for preload to complete before returning from update_settings, or emit "preload-start" event and block recording until "preload-complete"
- Add explicit state enum for transcription engine readiness (Unloaded, Loading, Ready, Error)

### Audio Capture State Recovery

**Problem:** If audio capture crashes or leaves buffers in inconsistent state, subsequent start_recording may succeed but produce garbage.

**Files:** `src-tauri/src/audio/capture.rs` (lines 82-91: `reset_state()`)

**Why fragile:**
- reset_state() is not called automatically on error; relies on caller
- AUDIO_THREAD cleanup is not atomic with buffer clearing; race window exists
- IS_STOPPING flag can become "stuck true" if stop_capture panics

**Safe modification:**
- Add guard struct `AudioCaptureGuard` that calls reset_state() on drop
- Use RAII pattern in start_capture to guarantee cleanup on early return
- Add health check command that verifies AUDIO_THREAD state vs IS_STOPPING flag

### NSPanel Native Drag Implementation

**Files:** `src-tauri/src/lib.rs` (lines 811-1051: drag state and monitor handlers)

**Why fragile:**
- `NATIVE_DRAG_STATE` global mutex is only guarded by `Ok()` error suppression in some paths
- Panel pointer stored as `usize` is unsafe; if panel is deallocated, setFrameOrigin will crash
- Deferred monitor removal via GCD dispatch_async_f has no timeout; if main thread stalls, monitors persist
- No validation that monitors are actually removed before starting new drag

**Safe modification:**
- Wrap panel reference in reference-counted container that notifies drag handler on dealloc
- Use weak references instead of raw pointers
- Add explicit timeout (100ms) for deferred cleanup; if not removed, warn and attempt immediate removal on next drag
- Validate monitor IDs are non-zero before calling removeMonitor

### Hotkey Registration Persistence

**Files:** `src-tauri/src/hotkey/mod.rs`, referenced in `src-tauri/src/lib.rs` (lines 446-498)

**Why fragile:**
- Old hotkey is unregistered, new one registered in update_settings, with no rollback on re-registration failure
- If re-registration fails, app is left with no active hotkey but settings persisted with new (inactive) binding
- No validation that hotkey string is valid before attempting registration

**Safe modification:**
- Register new hotkey first, validate success, then unregister old
- On failure, emit error event and revert settings to previous hotkey
- Add pre-validation of hotkey syntax against OS expectations

## Scaling Limits

### Audio Buffer Pre-allocation Fixed Size

**Capacity:** 30 seconds at 48kHz stereo for AUDIO_BUFFER, 30 seconds at 16kHz mono for WHISPER_BUFFER (lines 152-162 in capture.rs).

**Limit:** Recordings longer than ~30s risk buffer reallocations during capture, causing audio dropouts.

**Scaling path:**
- Make buffer limits configurable (e.g., max recording time in settings)
- Implement circular/ring buffer to allow unbounded recordings
- Add memory usage monitoring to warn user if approaching system limits

### VAD Streaming Monitor Queue Depth

**Capacity:** STREAMING_RESULTS accumulates strings from VAD triggers; no size limit.

**Limit:** Very long silences with many false-positive VAD triggers could accumulate thousands of segments, consuming memory and delaying stop_recording().

**Scaling path:**
- Add max queue depth with overflow behavior (drop oldest, warning, or stall)
- Monitor queue size and emit metrics
- Consider alternative silence-detection heuristics to reduce false triggers

## Dependencies at Risk

### Rust: whisper-rs without OpenAI License

**Risk:** whisper-rs bundles GGML library which uses MIT license, but OpenAI Whisper models have non-commercial restrictions depending on model size.

**Impact:** Shipping with Whisper in app violates OpenAI ToS if monetized or used commercially without explicit permission.

**Migration plan:**
- Audit model usage (local GGML vs cloud APIs)
- Document licensing status in app about/legal section
- Consider switching to Silero or other open-model alternatives

### tauri-nspanel (Unmaintained)

**Risk:** Plugin used for NSPanel support may not follow Tauri major version updates.

**Impact:** Tauri v3+ release could break window management layer.

**Migration plan:**
- Monitor tauri-nspanel GitHub for Tauri v3 support
- Fallback plan: implement NSPanel conversion directly in build.rs using cocoa FFI

## Test Coverage Gaps

### Audio Capture Thread Lifecycle

**What's not tested:**
- stop_capture() behavior when capture hasn't been started
- RESAMPLER_STATE cleanup when real-time resampling fails mid-recording
- Audio callback behavior on device disconnect/reconnect

**Files:** `src-tauri/src/audio/capture.rs`

**Risk:** Edge cases in thread spawning/joining could silently fail or cause panics.

**Priority:** High — audio capture is core to all functionality

### Native Drag State Cleanup

**What's not tested:**
- Monitors are actually removed after mouseup on all screen configurations
- Panel is still valid when drag handler accesses it
- Multiple sequential drags don't leave dangling monitor IDs

**Files:** `src-tauri/src/lib.rs` (lines 811-1051)

**Risk:** Monitor handle leaks could cause memory growth or event loop saturation over time.

**Priority:** Medium — affects repeated interaction (dragging during long sessions)

### Transcription Engine Switching Under Load

**What's not tested:**
- Switching engines while recording (should block or queue)
- Switching engines while model preload is in progress
- Switching back and forth rapidly

**Files:** `src-tauri/src/lib.rs` (lines 501-538)

**Risk:** Race conditions could activate wrong engine mid-transcription.

**Priority:** High — directly impacts transcription quality

### Error Propagation in Recording Lifecycle

**What's not tested:**
- start_recording() failure rolls back all state (is_recording, audio thread, emitters)
- stop_recording() failure doesn't corrupt STREAMING_RESULTS or audio buffers
- Failure in stop_recording() before Mutex acquisition doesn't deadlock

**Files:** `src-tauri/src/lib.rs` (lines 142-400)

**Risk:** Partial state corruption could leave app in unrecoverable state.

**Priority:** High — cascading failures critical to user experience

---

*Concerns audit: 2026-02-26*
