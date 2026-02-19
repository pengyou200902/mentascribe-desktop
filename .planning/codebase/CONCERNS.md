# Codebase Concerns

**Analysis Date:** 2026-02-19

## Tech Debt

**Cloud Transcription Providers Not Implemented:**
- Issue: OpenAI, AWS, and AssemblyAI cloud transcription functions are placeholders
- Files: `src-tauri/src/transcription/cloud.rs` (lines 59, 72, 83)
- Impact: Cloud fallback providers are non-functional. Users cannot switch to cloud services if local Whisper fails
- Fix approach: Implement multipart form uploads for OpenAI API, AWS Transcribe SDK integration, and AssemblyAI REST API integration with proper authentication

**Mutex Lock Unwraps in Audio Capture:**
- Issue: Multiple `unwrap()` calls on lazy_static Mutex locks in hot paths
- Files: `src-tauri/src/audio/capture.rs` (lines 55, 61-64, 70, 77, 83, 89, 112-113, 196, 208, 214, 232-236, 264)
- Impact: Application panics if any lazy_static Mutex becomes poisoned (can happen if a thread panics while holding lock)
- Fix approach: Replace with safe error handling that logs and recovers gracefully, or use parking_lot::Mutex which doesn't poison

**Settings AppState Mutex Lock Unwraps:**
- Issue: Direct `.unwrap()` on settings Mutex locks in command handlers
- Files: `src-tauri/src/lib.rs` (lines 104-105, 257, 890-892, 909, 929-930, 959, 967-969)
- Impact: Commands panic if settings lock is poisoned (rare but possible)
- Fix approach: Use `.map_err()` to return proper error messages to frontend instead of unwrapping

**Tauri Application Run Expects:**
- Issue: Final `.expect()` on tauri application run
- Files: `src-tauri/src/lib.rs` (line 1201)
- Impact: Panics on startup if Tauri initialization fails
- Fix approach: Use `.map_err()` and return proper error, or write error to stderr/log

## Critical Bugs

**Mixed-DPI Coordinate Space Mismatch (macOS):**
- Issue: Tauri issue #7890 - Non-macOS fallback path uses incompatible coordinate spaces on multi-monitor setups
- Files: `src-tauri/src/lib.rs` (lines 752-757, 919-950)
- Trigger: Position dictation window on multi-monitor setup with mixed DPI (2x Retina + 1x external)
- Symptoms: Window positions incorrectly on wrong monitor or outside visible bounds
- Current mitigation: Native macOS implementation (`native_position_on_cursor_monitor`) uses direct NSEvent/NSScreen APIs to stay in consistent AppKit coordinate space
- Impact: Non-macOS platforms (Windows, Linux) may have positioning issues on multi-monitor setups
- Fix approach: Test and fix coordinate space handling in non-macOS code path; consider extracting monitor coordinate conversion into utility functions

**NSNonactivatingPanelMask Breaks Window Dragging:**
- Issue: NSNonactivatingPanelMask (128) prevents focus stealing but disables native window dragging
- Files: `src-tauri/src/lib.rs` (lines 34-35, 61)
- Impact: Users cannot drag window using standard macOS title bar - requires custom JS-level drag implementation
- Current mitigation: Manual JS-level mousedown → `setPosition(PhysicalPosition)` workaround
- Fix approach: Document limitation; consider making draggable mode disable overlay features for better UX

## Known Bugs

**Audio Capture May Produce No Samples:**
- Issue: Microphone permission denial or device unavailability silently produces empty audio buffer
- Files: `src-tauri/src/audio/capture.rs` (lines 252, 287)
- Trigger: User denies microphone permissions or no input device available
- Symptoms: Transcription returns `[BLANK_AUDIO]` marker or empty string; not clearly reported to user
- Workaround: Check system audio permissions; current code skips transcription if audio is empty
- Impact: User gets no feedback that audio capture failed

**Silence Trimming May Remove All Audio:**
- Issue: `trim_silence()` with aggressive thresholds can remove entire recording if noise floor is high
- Files: `src-tauri/src/audio/capture.rs` (lines 316-335)
- Current mitigation: Only trim if result keeps >20% of audio (lines 320); otherwise skip trimming
- Symptoms: Whisper transcribes empty audio, returns `[BLANK_AUDIO]`
- Impact: Users with loud background noise may lose recordings

**VAD Energy Threshold May Not Detect Quiet Speech:**
- Issue: Voice Activity Detection uses fixed 0.01 energy threshold; doesn't adapt to environment
- Files: `src-tauri/src/audio/vad.rs` (lines 9, 19)
- Impact: Quiet speakers or high background noise cause missed detections
- Fix approach: Implement adaptive VAD or add user-configurable noise floor

**Empty or Whitespace-Only Transcriptions Not Distinguished:**
- Issue: `[BLANK_AUDIO]` markers from Whisper and actual whitespace are stripped but not reported
- Files: `src-tauri/src/injection/mod.rs` (lines 343-352)
- Impact: User records 10 seconds but gets no output; unclear if Whisper found nothing or audio was silent
- Fix approach: Return structured result with detection status instead of silent no-op

## Security Considerations

**Clipboard Not Cleared on Error:**
- Issue: If text injection via paste fails after clipboard write, sensitive data remains on clipboard
- Files: `src-tauri/src/injection/mod.rs` (line 404)
- Risk: User text (potentially containing passwords or sensitive info) could be visible in clipboard history
- Current mitigation: Clipboard cleared in normal flow; but error path may not clear
- Recommendations: Use try-finally or RAII pattern to guarantee clipboard clearing; consider zeroing clipboard memory

**Text Injection via Keyboard Simulation Vulnerable to Foreground Window Spoofing:**
- Issue: No verification that target window has focus before injecting text
- Files: `src-tauri/src/injection/mod.rs` (line 371)
- Risk: Text could be injected into wrong application if focus changes between recording stop and injection
- Current mitigation: 50ms delay to allow app focus
- Recommendations: Add focus verification before injection; consider requiring explicit focus event

**Accessibility Permissions Not Explicitly Checked on Launch:**
- Issue: Permission check only happens when injecting text; not validated on startup
- Files: `src-tauri/src/injection/mod.rs` (line 358)
- Risk: User enables recording but gets error at injection time with no warning
- Recommendations: Check accessibility permissions at app startup and warn user

## Performance Bottlenecks

**Whisper Model Loaded Per Transcription:**
- Issue: Despite `MODEL_CACHE`, Whisper context may reload on model size change
- Files: `src-tauri/src/transcription/whisper.rs` (lines 12-25)
- Current capacity: Caches single model; loading large model (2.9GB for "large") takes 10+ seconds
- Impact: First transcription after model switch is slow; users may think app hung
- Improvement path: Pre-load model on startup if possible; show progress UI during load

**Audio Resampling Done Every Transcription:**
- Issue: No caching of resampled audio; if resample fails or takes time, transcription delays
- Files: `src-tauri/src/audio/capture.rs` (lines 346-380)
- Impact: Users with non-standard sample rates experience extra latency
- Improvement path: Profile resampling performance; consider using faster algorithm or GPU acceleration

**Lazy_static Global State in Audio Capture:**
- Issue: Multiple global Mutex-protected static variables for audio buffer, sample rate, channels, etc.
- Files: `src-tauri/src/audio/capture.rs` (lines 34-42)
- Impact: Lock contention on high-frequency operations (get_current_level called 40x/sec); performance regression possible with many audio callbacks
- Improvement path: Use Arc<Mutex<AudioState>> instead; avoid global state in future redesign

**Tray Icon Rebuilt on Every Settings Update:**
- Issue: Tray menu menu might not be efficiently updated
- Files: `src-tauri/src/lib.rs` (line 1128)
- Impact: Frequent settings changes cause brief UI stutter
- Improvement path: Cache tray menu or use efficient update pattern

## Fragile Areas

**Window Positioning Logic in Mixed-Coordinate System:**
- Files: `src-tauri/src/lib.rs` (lines 750-856, 919-950)
- Why fragile: Subtle differences between AppKit (bottom-left origin) and screen/logical coordinates; easy to introduce regression
- Safe modification: Always include both macOS native and non-macOS fallback when modifying; add tests for multi-monitor setups
- Test coverage: No unit tests for coordinate conversion; manual testing only

**Audio Capture Lifecycle with Dual State Flags:**
- Files: `src-tauri/src/audio/capture.rs` (lines 41, 76-90, 207-220)
- Why fragile: `IS_STOPPING` flag prevents races but requires careful ordering; if new code calls capture functions out of order, deadlock or state corruption possible
- Safe modification: Document preconditions clearly; consider using state machine pattern instead of flags
- Test coverage: No tests for concurrent start/stop sequences

**Settings Serialization/Deserialization:**
- Files: `src-tauri/src/settings/mod.rs`
- Why fragile: Settings stored to disk as JSON; backwards compatibility with older versions not enforced
- Safe modification: Add migration logic before deserializing; add version field to settings structure
- Test coverage: No serialization round-trip tests

**NSPanel Collection Behavior Setup:**
- Files: `src-tauri/src/lib.rs` (lines 48-57, 96-100)
- Why fragile: Bit flags for NSWindowCollectionBehavior; wrong combination may cause unexpected window behavior
- Safe modification: Document meaning of each flag; test on actual fullscreen apps (Slack, Final Cut Pro) before changing
- Test coverage: Manual testing only; no automated verification

## Scaling Limits

**Single Whisper Model Cache:**
- Current capacity: Only one model can be cached in memory at a time
- Limit: If user switches between "large" and "small" models, must reload from disk (~10GB for large)
- Scaling path: Implement LRU cache for multiple models; or lazy-load on demand

**Lazy_static Audio Buffer Unbounded:**
- Current capacity: `AUDIO_BUFFER` grows indefinitely during long recordings
- Limit: Very long recordings (>30 minutes) could exhaust available RAM
- Scaling path: Implement ringbuffer or streaming architecture; swap to disk if needed

**UI Framerate in Audio Visualization:**
- Current capacity: 40 audio level updates/sec (25ms interval)
- Limit: If frontend rendering is slow, will drop frames and miss level updates
- Scaling path: Throttle updates client-side; use requestAnimationFrame for smoother rendering

## Dependencies at Risk

**whisper-rs (0.15):**
- Risk: Model format may change in future versions of whisper.cpp; binary compatibility not guaranteed
- Impact: Frozen on specific version; updates require testing all model sizes
- Migration plan: Monitor whisper.cpp releases; consider alternative like Candle or using official Whisper binary

**Tauri v2:**
- Risk: NSPanel functionality via tauri-nspanel plugin may not be maintained if plugin abandoned
- Impact: macOS fullscreen overlay support depends on third-party plugin
- Migration plan: Contribute to plugin maintenance; consider vendoring if necessary

**Global Hotkey (0.5):**
- Risk: Global hotkey handling varies across OS versions; may break on macOS 14+
- Impact: F6 hotkey or configured hotkey may fail without notice
- Migration plan: Add UI to verify hotkey is working; fallback to menu-only if hotkey fails

## Test Coverage Gaps

**Untested Audio Capture Error Paths:**
- What's not tested: Microphone permission denial, device unavailable, capture underrun
- Files: `src-tauri/src/audio/capture.rs`
- Risk: Silent failures; users don't know why transcription didn't work
- Priority: **High** - affects core functionality

**Untested Multi-Monitor Positioning:**
- What's not tested: Cursor position detection on 3-monitor setup with mixed DPI
- Files: `src-tauri/src/lib.rs` (lines 750-856)
- Risk: Window positioning fails on certain monitor combinations
- Priority: **High** - known issue with Tauri coordinates

**Untested Cloud Transcription Fallback:**
- What's not tested: Cloud provider authentication, API errors, network failures
- Files: `src-tauri/src/transcription/cloud.rs`
- Risk: Functions exist but are non-functional stubs
- Priority: **High** - feature is advertised but doesn't work

**Untested Settings Persistence:**
- What's not tested: Corrupted settings file, missing fields, version migration
- Files: `src-tauri/src/settings/mod.rs`
- Risk: Settings load fails without recovery
- Priority: **Medium** - default fallback exists but not tested

**Untested Text Injection Edge Cases:**
- What's not tested: Emoji, combining characters, RTL text, very long strings
- Files: `src-tauri/src/injection/mod.rs`
- Risk: Text corruption or injection failure for non-ASCII input
- Priority: **Medium** - affects international users

**Untested Hot Reload of Hotkey Settings:**
- What's not tested: Changing hotkey while app is running
- Files: `src-tauri/src/lib.rs` (update_settings), `src-tauri/src/hotkey/mod.rs`
- Risk: Old hotkey still active, new hotkey doesn't register
- Priority: **Medium** - less common but impacts UX

---

*Concerns audit: 2026-02-19*
