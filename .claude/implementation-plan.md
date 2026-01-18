# MentaScribe Desktop - Complete Voice-to-Text Implementation Plan

## Goal
Make the voice-to-text auto-paste feature fully production-ready: user speaks → text transcribed → auto-injected into focused text field.

## Current State
The app has working components but several issues prevent production use:
- ✅ F6 hotkey triggers recording
- ✅ Audio capture with CPAL works
- ✅ Whisper transcription functional
- ✅ Text injection works (partial)
- ❌ Race condition in UI state
- ❌ No error feedback to user
- ❌ Windows paste mode broken
- ❌ Hotkey hardcoded (settings ignored)
- ❌ No default model download

---

## Phase 1: Critical Fixes

### 1.1 Fix Race Condition in State Management
**File:** `src/App.tsx`

**Problem:** `setIsRecording(false)` called before transcription completes (line 67)

**Fix:** Move state updates to proper sequence:
```typescript
const stopRecording = useCallback(async () => {
  if (!isRecordingRef.current) return;
  try {
    setIsRecording(false);
    setIsProcessing(true);  // Show processing immediately
    const text = await invoke<string>('stop_recording');
    if (text) {
      await invoke('inject_text', { text });
    }
  } catch (error) {
    console.error('Failed:', error);
  } finally {
    setIsProcessing(false);  // Reset after everything completes
  }
}, []);
```

### 1.2 Add User Feedback for Failures
**Files:** `src-tauri/src/lib.rs`, `src/App.tsx`, `src/components/DictationBar.tsx`

**Changes:**
1. Backend: Emit `injection-failed` event with error message
2. Frontend: Listen for error events, display in DictationBar
3. Add `error` prop to DictationBar component

### 1.3 Fix Windows Clipboard Paste
**Files:** `src-tauri/src/injection/mod.rs`, `src-tauri/Cargo.toml`

**Changes:**
1. Add `arboard = "3"` dependency for cross-platform clipboard
2. Replace Windows fallback with actual clipboard implementation:
```rust
#[cfg(target_os = "windows")]
{
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
}
```

### 1.4 Use Settings for Hotkey
**Files:** `src-tauri/src/hotkey/mod.rs`, `src-tauri/src/lib.rs`

**Changes:**
1. Add `parse_key_code()` function to convert "F5", "F6", etc. to `Code` enum
2. Pass settings to `setup_hotkey()` function
3. Use `settings.hotkey.key` instead of hardcoded F6

---

## Phase 2: Performance Optimizations

### 2.1 Cache Whisper Model
**Files:** `src-tauri/src/transcription/whisper.rs`, `src-tauri/Cargo.toml`

**Changes:**
1. Add `once_cell = "1.19"` dependency
2. Create global `MODEL_CACHE` with `Lazy<Mutex<Option<WhisperContext>>>`
3. Load model once, reuse for subsequent transcriptions
4. Invalidate cache only when model size changes

### 2.2 Integrate VAD for Silence Trimming
**Files:** `src-tauri/src/audio/capture.rs`

**Changes:**
1. Import existing `vad::trim_silence` function
2. Apply silence trimming before returning audio in `stop_capture()`
3. Trim leading/trailing silence to improve transcription speed

---

## Phase 3: Feature Completions

### 3.1 Implement Auto-Capitalize
**Files:** Create `src-tauri/src/text/mod.rs`, update `src-tauri/src/lib.rs`

**Changes:**
1. Create text processing module with `process_text(text, auto_capitalize)` function
2. Capitalize first letter and after sentence-ending punctuation
3. Apply processing before injection based on settings

### 3.2 Add Default Model Download
**Files:** `src-tauri/src/lib.rs`, `src-tauri/src/transcription/whisper.rs`, `src/App.tsx`

**Changes:**
1. Check if any model exists during app setup
2. Emit `no-model-downloaded` event if none found
3. Frontend auto-downloads "base" model on first run
4. Add download progress events

### 3.3 Save Transcription History
**File:** `src/App.tsx`

**Changes:**
1. After successful injection, save to localStorage
2. Store: id, text, timestamp
3. Limit to 100 entries

---

## Files to Modify

| File | Priority | Changes |
|------|----------|---------|
| `src/App.tsx` | P1 | Fix race condition, error handling, history save |
| `src/components/DictationBar.tsx` | P1 | Add error display |
| `src-tauri/src/lib.rs` | P1 | Injection events, first-run check |
| `src-tauri/src/hotkey/mod.rs` | P1 | Dynamic hotkey from settings |
| `src-tauri/src/injection/mod.rs` | P1 | Windows clipboard fix |
| `src-tauri/src/transcription/whisper.rs` | P2 | Model caching |
| `src-tauri/src/audio/capture.rs` | P2 | VAD integration |
| `src-tauri/src/text/mod.rs` | P3 | NEW: Auto-capitalize |
| `src-tauri/Cargo.toml` | P1 | Add arboard, once_cell |

---

## Implementation Order

1. **P1.1** Fix race condition in App.tsx
2. **P1.3** Fix Windows paste (add arboard dependency)
3. **P1.2** Add error feedback
4. **P1.4** Use settings for hotkey
5. **P2.1** Cache Whisper model
6. **P2.2** Integrate VAD
7. **P3.1** Auto-capitalize
8. **P3.2** Default model download
9. **P3.3** Save history

---

## Verification

### Testing Each Phase:

**Phase 1:**
- Record and verify UI shows "Processing..." during transcription
- Verify "Ready" only appears after injection completes
- On Windows, verify paste mode uses clipboard (not typing)
- Change hotkey in settings, verify new key works

**Phase 2:**
- Record twice quickly, second should be faster (no model reload)
- Record with 2s silence at start/end, verify audio is trimmed

**Phase 3:**
- Say "hello. how are you" with auto-capitalize on → "Hello. How are you"
- Delete models folder, relaunch → verify download prompt appears
- Transcribe text, open History → verify it appears

### End-to-End Test:
1. Launch app (first run should download model)
2. Click into any text field (browser, notes, etc.)
3. Press F6, speak "Hello world. This is a test."
4. Release F6
5. Verify text appears in the text field with proper capitalization
6. Open History, verify transcription saved
