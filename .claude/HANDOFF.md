# MentaScribe Desktop - AI Agent Handoff Document

**Last Updated:** 2026-01-18 14:01:05 EST (Accessibility permissions check + frontend timing fix)
**Status:** Implementation Complete - Bug Fixes Applied

---

## Project Overview

MentaScribe Desktop is a voice-to-text dictation app built with:
- **Frontend:** React + TypeScript + Vite + Tailwind CSS
- **Backend:** Tauri 2.x + Rust
- **STT Engine:** Whisper.cpp via whisper-rs

**Core Flow:** User presses F6 → Records audio → Whisper transcribes → Text auto-injected into focused app

---

## Implementation Status

### ✅ All 9 Tasks Completed

| # | Task | Status | Files Changed |
|---|------|--------|---------------|
| 1 | Fix race condition in App.tsx | ✅ Done | `src/App.tsx` |
| 2 | Fix Windows clipboard paste | ✅ Done | `src-tauri/src/injection/mod.rs`, `Cargo.toml` |
| 3 | Add error feedback for failures | ✅ Done | `src/App.tsx`, `src/components/DictationBar.tsx` |
| 4 | Use settings for hotkey config | ✅ Done | `src-tauri/src/hotkey/mod.rs`, `src-tauri/src/lib.rs` |
| 5 | Cache Whisper model | ✅ Done | `src-tauri/src/transcription/whisper.rs` |
| 6 | Integrate VAD silence trimming | ✅ Done | `src-tauri/src/audio/capture.rs` |
| 7 | Implement auto-capitalize | ✅ Done | `src-tauri/src/text/mod.rs` (NEW), `src-tauri/src/lib.rs` |
| 8 | Add default model download | ✅ Done | `src-tauri/src/lib.rs`, `src/App.tsx` |
| 9 | Save transcription history | ✅ Done | `src/App.tsx` |

---

## Key Files Modified

### Frontend (React/TypeScript)
```
src/
├── App.tsx                    # Main app - state, events, history saving
└── components/
    └── DictationBar.tsx       # UI - added error, statusOverride props
```

### Backend (Rust/Tauri)
```
src-tauri/src/
├── lib.rs                     # Commands, auto-capitalize, model check, hotkey setup
├── hotkey/mod.rs              # Configurable hotkey (F1-F12 support)
├── injection/mod.rs           # Cross-platform clipboard with arboard
├── transcription/whisper.rs   # Model caching with once_cell
├── audio/capture.rs           # VAD silence trimming integration
└── text/mod.rs                # NEW: Auto-capitalize text processing
```

### Dependencies Added (Cargo.toml)
```toml
arboard = "3"      # Cross-platform clipboard
once_cell = "1.19" # Lazy static for model cache
```

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│ User presses F6                                             │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ hotkey/mod.rs: Emits "hotkey-pressed" event                 │
│ (Reads key from settings, supports F1-F12)                  │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ App.tsx: startRecording() → invoke('start_recording')       │
│ Sets isRecording=true, shows "Listening..."                 │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ audio/capture.rs: CPAL audio stream captures to buffer      │
│ Emits audio-level events (25ms intervals) for waveform      │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ User releases F6 → "hotkey-released" event                  │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ App.tsx: stopRecording() → invoke('stop_recording')         │
│ Sets isRecording=false, isProcessing=true                   │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ lib.rs: stop_recording command                              │
│ 1. Stop audio capture                                       │
│ 2. Emit "transcription-processing"                          │
│ 3. Call whisper::transcribe()                               │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ audio/capture.rs: prepare_for_whisper()                     │
│ - Convert to mono, resample to 16kHz                        │
│ - Trim silence using vad::trim_silence()                    │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ transcription/whisper.rs: transcribe()                      │
│ - Check MODEL_CACHE, load if needed                         │
│ - Run Whisper inference                                     │
│ - Return raw text                                           │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ text/mod.rs: process_text()                                 │
│ - Apply auto-capitalize if enabled                          │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ lib.rs: Emit "transcription-complete", return text          │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ App.tsx: Receive text, invoke('inject_text', { text })      │
│ Save to history on success                                  │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ injection/mod.rs: inject_text()                             │
│ - "type" mode: enigo keyboard simulation                    │
│ - "paste" mode: arboard clipboard + Cmd/Ctrl+V              │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ Text appears in focused application!                        │
│ App.tsx: setIsProcessing(false), show "Ready"               │
└─────────────────────────────────────────────────────────────┘
```

---

## Pending Testing / Known Issues

### Needs Testing:
1. **First run model download** - Delete `~/.mentascribe/models/` and verify auto-download
2. **Hotkey configuration** - Change hotkey in settings, verify it works
3. **Windows clipboard** - Test paste mode on Windows
4. **VAD trimming** - Record with long silence, verify it's trimmed
5. **Auto-capitalize** - Verify "hello. world" becomes "Hello. World"
6. **History** - Transcribe and check History window

### Bug Fixes Applied (2026-01-18):

**1. Waveform Animation Not Visible**
- **Issue:** Waveform bars were static/too small during recording
- **Cause:** Base height values too low, bars started at 0.15 and slowly interpolated
- **Fix:**
  - Immediately set bars to visible random heights (0.4-0.7) when recording starts
  - Widened base height range to 0.3-0.7 for more visible movement
  - Faster update interval (40ms/25fps) and smoothing (0.4)
  - Removed CSS transition that conflicted with JS animation
- **Files:** `src/components/DictationBar.tsx`, `src/styles/globals.css`

**2. Stale Closure Reference**
- **Issue:** `isProcessing` used in animation closure was stale
- **Cause:** Direct prop reference in requestAnimationFrame callback
- **Fix:** Added `isProcessingRef` to keep value in sync
- **File:** `src/components/DictationBar.tsx`

**3. Error State After Failed Transcription**
- **Issue:** UI showed "Error!" then got stuck in bad state on subsequent F6 presses
- **Cause:** Missing model caused transcription to fail, poor error handling
- **Fix:** Added specific handling for "Model not found" error, auto-triggers model download
- **File:** `src/App.tsx`

**4. Lifetime Error in Hotkey Setup**
- **Issue:** Compilation error - borrowed `&str` escaping into closure
- **Cause:** `key_name` parameter used directly in `'static` closure
- **Fix:** Convert to owned `String` with `.to_string()` and `.clone()`
- **File:** `src-tauri/src/hotkey/mod.rs`

**5. Audio Capture Race Condition (Buffer Cleared During Stop)**
- **Issue:** Transcription failed with "Input sample buffer was empty" even though audio was recorded
- **Cause:** Race condition in `stop_capture()` / `start_capture()`:
  1. `stop_capture()` takes the `AUDIO_THREAD` handle (releasing lock)
  2. While waiting for audio thread to join, another `start_recording` comes in
  3. `start_capture()` checks `AUDIO_THREAD.is_some()` → false (handle was taken)
  4. `start_capture()` proceeds and clears `AUDIO_BUFFER`
  5. Original `stop_capture()` finishes join and reads empty buffer
- **Fix:** Added `IS_STOPPING` flag:
  - Set to `true` at start of `stop_capture()`
  - Checked in `start_capture()` - returns error if stopping in progress
  - Cleared at end of `stop_capture()` after buffer is read
- **File:** `src-tauri/src/audio/capture.rs`

**6. Frontend Timing Bug (Duplicate Hotkey Events)**
- **Issue:** Multiple `start_recording` calls happening before any `stop_recording`
- **Cause:** `isRecordingRef` was only set to `true` AFTER `await invoke('start_recording')` completed. If another hotkey event fired during the await, the guard check would pass.
- **Fix:** Set `isRecordingRef.current = true` IMMEDIATELY before the await, reset on error. Same pattern applied to `stopRecording()` with `isProcessingRef`.
- **File:** `src/App.tsx`

**7. Text Injection Silent Failure (macOS Accessibility)**
- **Issue:** Text injection reported success but nothing was typed into editor
- **Cause:** enigo library silently fails when app lacks macOS Accessibility permissions - doesn't throw an error
- **Fix:** Added `check_accessibility_permissions()` function that uses AppleScript to check if we can interact with System Events. Returns proper error if denied.
- **File:** `src-tauri/src/injection/mod.rs`

### Critical Requirements:

**1. Whisper Model Required:**
- A Whisper model MUST be downloaded before transcription works
- On first run, app emits `no-model-downloaded` event
- Frontend should auto-download "base" model
- If auto-download fails, user must download manually via Settings
- Models stored at `~/.mentascribe/models/`

**2. macOS Accessibility Permissions:**
- Text injection requires **Accessibility permissions** on macOS
- Go to: System Preferences → Privacy & Security → Accessibility
- Add and enable the MentaScribe app
- Without this permission, text injection will silently fail

---

## How to Run

```bash
cd mentascribe-desktop
pnpm install
pnpm tauri dev
```

### Build for Production
```bash
pnpm tauri build
```

---

## Settings Structure

Settings are stored at:
- macOS: `~/Library/Application Support/mentascribe/settings.json`
- Windows: `%APPDATA%/mentascribe/settings.json`
- Linux: `~/.config/mentascribe/settings.json`

```json
{
  "transcription": {
    "model_size": "base",
    "language": "auto"
  },
  "hotkey": {
    "key": "F6",
    "mode": "hold"  // or "toggle"
  },
  "output": {
    "insert_method": "type",  // or "paste"
    "auto_capitalize": true
  }
}
```

---

## Whisper Models

Stored at `~/.mentascribe/models/`:
- `ggml-tiny.bin` (75MB)
- `ggml-base.bin` (142MB) ← Default, auto-downloaded
- `ggml-small.bin` (466MB)
- `ggml-medium.bin` (1.5GB)
- `ggml-large-v3.bin` (2.9GB)

---

## Future Enhancements (Not Implemented)

1. **Cloud STT fallback** - `transcription/cloud.rs` has stubs
2. **AI cleanup** - Settings UI exists but backend not connected
3. **Download progress** - Events exist but UI doesn't show progress bar
4. **Audio input selection** - Currently uses system default

---

## Files Reference

See `implementation-plan.md` in this directory for the full implementation plan with code snippets.

---

## Contact / Context

This implementation was done to make the voice-to-text app fully functional:
- Press F6 (or configured hotkey)
- Speak into microphone
- Release F6
- Text appears in whatever text field is focused

All core functionality is implemented. Main remaining work is testing and potential bug fixes.
