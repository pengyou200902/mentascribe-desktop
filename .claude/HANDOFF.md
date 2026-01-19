# MentaScribe Desktop - AI Agent Handoff Document

**Last Updated:** 2026-01-19 01:51:37 EST
**Status:** Functional - All core features implemented + Dashboard with theme support

---

## Project Overview

Voice-to-text dictation app: **Press F6 → Speak → Release → Text appears in focused app**

| Layer | Technology |
|-------|------------|
| Frontend | React + TypeScript + Vite + Tailwind CSS |
| Backend | Tauri 2.x + Rust |
| STT | Whisper.cpp via whisper-rs |

---

## Architecture Flow

```
F6 Press → hotkey/mod.rs emits event
    → App.tsx: startRecording() → invoke('start_recording')
    → audio/capture.rs: CPAL captures to buffer

F6 Release → hotkey/mod.rs emits event
    → App.tsx: stopRecording() → invoke('stop_recording')
    → audio/capture.rs: prepare_for_whisper() (mono, 16kHz, trim silence)
    → transcription/whisper.rs: Whisper inference
    → text/mod.rs: auto-capitalize
    → injection/mod.rs: clipboard + native paste (CGEvent/SendInput/XTest)
    → Text appears in focused app
```

---

## Dashboard Window

Accessible via system tray menu. Features:

| Page | Description |
|------|-------------|
| **Home** | Usage stats (streak, total words, avg WPM), recent transcriptions |
| **Dictionary** | Add/edit/delete custom phrases for transcription |
| **History** | Full transcription history with copy/delete, pagination |
| **Settings** | Theme selector, transcription options, hotkey config |

### Theme System

- **Modes:** Light / Dark / System (auto-detect)
- **Persistence:** localStorage key `mentascribe-theme`
- **Design:** "Sonic Warmth" - amber accents (#f0972a), DM Sans font, warm stone neutrals
- **Flash prevention:** Inline script in `index.html` sets theme class before React loads

---

## Key Files

| File | Purpose |
|------|---------|
| `src/App.tsx` | Main state, events, history |
| `src/components/DictationBar.tsx` | UI with waveform |
| `src-tauri/src/lib.rs` | Tauri commands |
| `src-tauri/src/hotkey/mod.rs` | F1-F12 hotkey support |
| `src-tauri/src/audio/capture.rs` | CPAL audio capture + VAD |
| `src-tauri/src/transcription/whisper.rs` | Whisper with model caching |
| `src-tauri/src/injection/mod.rs` | Native text injection (CGEvent/SendInput/XTest) |
| `src-tauri/src/text/mod.rs` | Auto-capitalize |

### Dashboard Files

| File | Purpose |
|------|---------|
| `src/components/dashboard/Dashboard.tsx` | Dashboard layout with ThemeProvider |
| `src/components/dashboard/Sidebar.tsx` | Navigation with logo, waveform decoration |
| `src/components/dashboard/HomePage.tsx` | Stats cards (streak, words, WPM), recent activity |
| `src/components/dashboard/HistoryPage.tsx` | Full history timeline with copy/delete |
| `src/components/dashboard/DictionaryPage.tsx` | Custom phrase management |
| `src/components/dashboard/SettingsPage.tsx` | Theme selector + all app settings |
| `src/lib/theme.tsx` | ThemeProvider (light/dark/system) |
| `src/lib/historyStore.ts` | Zustand store for history |
| `src/lib/dictionaryStore.ts` | Zustand store for dictionary |
| `src/lib/statsStore.ts` | Zustand store for usage stats |
| `src-tauri/src/history/mod.rs` | History CRUD backend |
| `src-tauri/src/dictionary/mod.rs` | Dictionary CRUD backend |
| `src-tauri/src/stats/mod.rs` | Stats tracking backend |

---

## Critical Requirements

1. **Whisper Model:** Must exist at `~/.mentascribe/models/ggml-base.bin` (auto-downloads on first run)
2. **macOS Accessibility:** System Settings → Privacy & Security → Accessibility → Enable MentaScribe
3. **Linux:** X11 only (Wayland not supported)

---

## Bug Fixes Applied

| # | Issue | Fix | Files |
|---|-------|-----|-------|
| 1 | Waveform static | Immediate random heights (0.3-0.7), 25fps | `DictationBar.tsx` |
| 2 | Stale closure in animation | Added `isProcessingRef` | `DictationBar.tsx` |
| 3 | Error state stuck after failed transcription | Handle "Model not found", auto-download | `App.tsx` |
| 4 | Lifetime error in hotkey | Convert `&str` to owned `String` | `hotkey/mod.rs` |
| 5 | Audio buffer race condition | Added `IS_STOPPING` flag | `capture.rs` |
| 6 | Duplicate hotkey events | Set ref BEFORE await | `App.tsx` |
| 7 | Silent injection failure (macOS) | Check accessibility before inject | `injection/mod.rs` |
| 8 | Type mode unreliable | Default to paste mode | `injection/mod.rs` |
| 9 | Paste failed in Apple apps | Use AppleScript for Cmd+V | `injection/mod.rs` |
| 10 | Recording state stuck | Reset on capture failure | `lib.rs`, `capture.rs` |
| 11 | AppleScript slow, enigo unreliable | Native APIs: CGEvent (macOS), SendInput (Win), XTest (Linux). 900ms→100ms | `injection/mod.rs`, `Cargo.toml` |
| 12 | BLANK_AUDIO skipped entire text | Strip marker instead of skip | `injection/mod.rs` |
| 13 | UI too large | Minimal redesign: mic icon only (color=state) + waveform when active. Window 120x40 | `DictationBar.tsx`, `globals.css`, `tauri.conf.json` |

---

## Settings

Stored at `~/Library/Application Support/mentascribe/settings.json` (macOS)

```json
{
  "transcription": { "model_size": "base", "language": "auto" },
  "hotkey": { "key": "F6", "mode": "hold" },
  "output": { "insert_method": "paste", "auto_capitalize": true }
}
```

---

## Run Commands

```bash
pnpm install && pnpm tauri dev    # Development
pnpm tauri build                   # Production
```

---

## Future Work

- Cloud STT fallback (stubs in `transcription/cloud.rs`)
- Download progress UI
- Audio input device selection
- Wayland support for Linux
