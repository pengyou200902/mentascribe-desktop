# Codebase Structure

**Analysis Date:** 2026-02-26

## Directory Layout

```
mentascribe-desktop/
├── src/                          # React/TypeScript frontend (Vite)
│   ├── App.tsx                   # Root component (routes dictation vs dashboard)
│   ├── main.tsx                  # React DOM entry point
│   ├── components/               # React UI components
│   │   ├── DictationBar.tsx      # Overlay widget with waveform
│   │   ├── Settings.tsx          # Settings editor
│   │   ├── History.tsx           # Local history viewer
│   │   ├── MenuBar.tsx           # Tray menu
│   │   ├── TranscriptionOverlay.tsx
│   │   └── dashboard/            # Multi-page dashboard
│   │       ├── Dashboard.tsx     # Route controller
│   │       ├── HomePage.tsx      # Stats/quick actions
│   │       ├── HistoryPage.tsx   # Transcription history
│   │       ├── DictionaryPage.tsx # Text replacement rules
│   │       ├── SettingsPage.tsx  # Settings UI
│   │       └── Sidebar.tsx       # Navigation
│   ├── lib/                      # Zustand stores & utilities
│   │   ├── store.ts             # Main settings store (useStore hook)
│   │   ├── historyStore.ts      # History pagination (useHistoryStore)
│   │   ├── dictionaryStore.ts   # Dictionary CRUD (useDictionaryStore)
│   │   ├── statsStore.ts        # Statistics (useStatsStore)
│   │   ├── tauri.ts             # Tauri API wrapper functions
│   │   └── theme.tsx            # Theme provider (React context)
│   ├── config/                   # Frontend constants
│   │   └── widget.ts             # Dictation bar animation constants
│   ├── types/                    # TypeScript definitions
│   │   └── index.ts             # Shared interfaces (TranscriptionEntry, etc)
│   ├── icons/                    # SVG icon components
│   ├── styles/                   # Global CSS
│   │   └── globals.css          # Tailwind + custom styles
│   └── hooks/                    # Custom React hooks (currently empty)
│
├── src-tauri/                    # Rust backend (Tauri v2)
│   ├── Cargo.toml               # Rust dependencies
│   ├── src/
│   │   ├── lib.rs              # Main application (~1662 lines)
│   │   │                        # Contains: Tauri setup, AppState, all #[tauri::command]
│   │   │                        # Functions: start_recording, stop_recording, settings, etc
│   │   ├── main.rs             # Binary entry (minimal)
│   │   │
│   │   ├── audio/               # Microphone input & VAD
│   │   │   ├── mod.rs          # Module exports
│   │   │   ├── capture.rs      # CPAL audio device, streaming, level metering
│   │   │   └── vad.rs          # Voice Activity Detection (Silero via whisper-rs)
│   │   │
│   │   ├── transcription/       # Speech-to-text engines
│   │   │   ├── mod.rs          # Common types (ModelInfo, CoremlStatus)
│   │   │   ├── whisper.rs      # Whisper.cpp via whisper-rs
│   │   │   │                    # Model management, caching, streaming/batch
│   │   │   ├── cloud.rs        # Cloud API adapters (OpenAI, etc)
│   │   │   ├── voxtral.rs      # Voxtral engine (proprietary)
│   │   │   └── voxtral_ffi.rs  # C bindings for Voxtral
│   │   │
│   │   ├── settings/            # Configuration persistence
│   │   │   └── mod.rs          # Load/save settings.json, typed structs
│   │   │
│   │   ├── hotkey/              # Global keyboard shortcuts
│   │   │   └── mod.rs          # Setup/unregister via tauri-plugin-global-shortcut
│   │   │
│   │   ├── injection/           # Text insertion into apps
│   │   │   └── mod.rs          # Platform-specific implementations (macOS/Windows/Linux)
│   │   │
│   │   ├── history/             # Transcription log
│   │   │   └── mod.rs          # Add/get/delete history entries, persist to JSON
│   │   │
│   │   ├── dictionary/          # Text replacement engine
│   │   │   └── mod.rs          # Load rules, apply replacements
│   │   │
│   │   ├── stats/               # Usage analytics
│   │   │   └── mod.rs          # Track daily stats, persist to JSON
│   │   │
│   │   ├── text/                # Post-transcription processing
│   │   │   └── mod.rs          # Auto-capitalization with tests
│   │   │
│   │   └── api/                 # External API clients
│   │       ├── mod.rs          # Common types (AuthToken, UserInfo)
│   │       └── client.rs       # HTTP client, login function
│   │
│   ├── tauri.conf.json          # Tauri build & window config
│   ├── build.rs                 # Build script (CC for native code)
│   ├── capabilities/            # Tauri plugin permissions
│   ├── gen/                     # Generated Tauri schemas (auto)
│   └── voxtral/                 # Voxtral model/library files (if included)
│
├── package.json                 # Frontend npm dependencies
├── tsconfig.json                # TypeScript config
├── tsconfig.node.json           # Vite-specific TS config
├── dist/                        # Built frontend (vite build output)
├── docs/                        # Project documentation
├── .planning/                   # GSD planning directory
│   └── codebase/               # Codebase analysis documents
└── node_modules/               # npm packages
```

## Directory Purposes

**`src/`** — React frontend application
- Vite dev server, built to `dist/` via `npm run build`
- Entry: `main.tsx` → `App.tsx`
- Window detection: `window.location.hash` (empty/dictation, `#dashboard` for management)

**`src/components/`** — Reusable React components
- `DictationBar.tsx`: Animated overlay widget (recording indicator, waveform, error messages)
- `dashboard/`: Multi-page UI (home, history, dictionary, settings)
- `Settings.tsx`: Settings form editor
- Components use props + Zustand stores for state

**`src/lib/`** — Shared utilities, stores, and hooks
- Zustand stores: Call `useStore()` in components to access settings
- `tauri.ts`: Thin wrapper around `invoke()` calls
- `theme.tsx`: React context for dark/light mode
- No custom hooks yet (directory prepared for future)

**`src/config/`** — Frontend constants
- `widget.ts`: All magic numbers for DictationBar animations (waveform, timing, etc.)
- Centralized for easy tweaking without code changes

**`src/types/`** — TypeScript interfaces
- `index.ts`: Shared types matching Rust backend (TranscriptionEntry, DailyStats, etc.)
- No enums (use string literals instead for Serde compatibility)

**`src-tauri/src/`** — Rust backend application
- **`lib.rs`**: Core app logic, all Tauri commands, AppState management (~1662 lines)
  - No separate main command module; commands defined inline with #[tauri::command]
  - Contains: setup_dictation_panel, start_recording, stop_recording, settings handlers
- Modular subsystems (audio, transcription, hotkey, injection, etc.) each in own mod.rs

**`src-tauri/src/audio/`** — Microphone input
- `capture.rs`: CPAL stream management, sample buffering, resampling to 16kHz
- `vad.rs`: Voice Activity Detection threshold logic
- Exports: `AudioData` struct, `start_capture()`, `stop_capture()`, `get_current_level()`

**`src-tauri/src/transcription/`** — Speech-to-text
- `whisper.rs`: Local Whisper.cpp model (CPU or Apple Neural Engine via CoreML)
  - Streaming mode: VAD-triggered partial results
  - Batch mode: Final inference on tail audio
  - Model cache (static) to avoid reload on every transcription
- `voxtral.rs`: Proprietary streaming engine (optional feature, Metal GPU on macOS)
- `cloud.rs`: Placeholder for cloud API adapters

**`src-tauri/src/settings/`** — User preferences
- Persistent to `~/.config/mentascribe/settings.json`
- Struct mirrors frontend `UserSettings` (typed, Serde)
- Functions: `load_settings()`, `save_settings()`

**`src-tauri/src/hotkey/`** — Global keyboard shortcuts
- Registers F1-F12 keys via tauri-plugin-global-shortcut
- Emits `hotkey-pressed`, `hotkey-released` events
- Re-registers on settings change if key binding changes

**`src-tauri/src/injection/`** — Text insertion into active app
- macOS: CGEvent keyboard simulation (raw key events + Unicode characters)
- Windows: Clipboard + paste simulation
- Linux: X11 key events
- Falls back to paste via clipboard (more compatible)
- Requires accessibility permission on macOS

**`src-tauri/src/history/`** — Transcription log
- Persistent to `~/.config/mentascribe/history.json`
- 500 entry limit (older pruned)
- Commands: get, delete, clear, count

**`src-tauri/src/dictionary/`** — Text replacement rules
- User-defined phrase → replacement mappings (e.g., "u" → "you")
- Applied post-transcription before injection
- Persistent to `~/.config/mentascribe/dictionary.json`

**`src-tauri/src/stats/`** — Usage metrics
- Daily counters: transcriptions, words, audio seconds
- Streak tracking (consecutive days used)
- Persistent to `~/.config/mentascribe/stats.json`
- Updated automatically on transcription completion

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React bootstrap
- `src/App.tsx`: Root component (window routing, event setup)
- `src-tauri/src/lib.rs`: Tauri app initialization, all backend commands

**Configuration:**
- `package.json`: Frontend scripts, dependencies
- `src-tauri/Cargo.toml`: Rust dependencies, feature flags
- `src-tauri/tauri.conf.json`: Window geometry, app info, plugins
- `tsconfig.json`: TypeScript compilation

**Core Logic:**
- `src/App.tsx`: Recording state machine, event listeners, window type detection
- `src-tauri/src/lib.rs`: Command handlers (start/stop recording), AppState, settings dispatch
- `src-tauri/src/audio/capture.rs`: Audio streaming, device management, level metering
- `src-tauri/src/transcription/whisper.rs`: Model loading, VAD streaming, batch inference

**Testing:**
- `src-tauri/src/text/mod.rs`: Unit tests for text processing (auto-capitalize)
- Pattern: Inline `#[cfg(test)]` modules with `#[test]` functions

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (e.g., `capture.rs`, `mod.rs`)
- React components: `PascalCase.tsx` (e.g., `DictationBar.tsx`, `Dashboard.tsx`)
- Utilities: `camelCase.ts` (e.g., `store.ts`, `tauri.ts`)
- Config/constants: `camelCase.ts` (e.g., `widget.ts`)

**Directories:**
- Rust modules: `snake_case/` (e.g., `audio/`, `transcription/`)
- React features: `camelCase/` (e.g., `components/`, `dashboard/`)
- Functional grouping: descriptive (e.g., `src/lib/`, `src/config/`)

**Rust Identifiers:**
- Functions: `snake_case` (e.g., `start_recording`, `apply_replacements`)
- Structs/Enums: `PascalCase` (e.g., `UserSettings`, `AudioData`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `MAX_UTF16_UNITS_PER_EVENT`)
- Module names: `snake_case` (e.g., `mod audio;`)

**TypeScript Identifiers:**
- Interfaces/Types: `PascalCase` (e.g., `TranscriptionEntry`, `UserSettings`)
- Functions: `camelCase` (e.g., `loadSettings`, `startRecording`)
- Constants: `UPPER_SNAKE_CASE` for magic numbers (e.g., `MAX_HISTORY_ENTRIES`)
- React components: `PascalCase` (e.g., `DictationBar`, `Dashboard`)
- Zustand stores: `useNoun` pattern (e.g., `useStore`, `useHistoryStore`)

## Where to Add New Code

**New Feature (e.g., noise reduction):**
- Core logic: `src-tauri/src/audio/noise_reduction.rs` (new module)
  - Create `mod.rs` or `noise_reduction.rs`
  - Import in `src-tauri/src/lib.rs` via `mod noise_reduction;`
  - Call from recording flow (e.g., in `stop_recording` after capture)
- Frontend UI: `src/components/SettingsPage.tsx` (extend settings form)
- Types: `src/types/index.ts` (add to UserSettings if configurable)
- Tests: Inline in Rust module under `#[cfg(test)]`

**New Component/Module:**
- React component: `src/components/MyFeature.tsx`
  - Use existing patterns (e.g., `DictationBar.tsx` for overlay, `HistoryPage.tsx` for dashboard)
  - Props should be minimal; use Zustand stores for state
  - Import Tauri API as needed: `import { invoke } from '@tauri-apps/api/core';`
- Rust subsystem: `src-tauri/src/feature_name/mod.rs`
  - Expose public API function(s) in mod.rs
  - Implement in submodules if needed
  - Use Result<T, E> for error handling with custom error type

**Utilities:**
- Shared Rust helpers: `src-tauri/src/lib.rs` (inline) or new module if >100 lines
- Shared TypeScript: `src/lib/tauri.ts` (wrapper functions) or new module
- Constants: `src/config/widget.ts` (frontend), inline in Rust (no centralized config file yet)

**Tests:**
- Rust: Inline `#[cfg(test)] mod tests { #[test] fn test_name() {} }`
- TypeScript: Jest not configured; tests would go in `*.test.ts` (setup needed)

## Special Directories

**`src-tauri/target/`:**
- Purpose: Cargo build outputs (debug, release binaries)
- Generated: Yes (cargo build, cargo build --release)
- Committed: No (.gitignore)

**`dist/`:**
- Purpose: Built frontend (bundled JS, CSS, assets)
- Generated: Yes (npm run build)
- Committed: No (.gitignore)

**`src-tauri/gen/`:**
- Purpose: Tauri-generated schemas and bindings
- Generated: Yes (tauri CLI on build)
- Committed: Partially (schemas may be committed for IDE support)

**`node_modules/`:**
- Purpose: npm dependencies
- Generated: Yes (npm install)
- Committed: No (.gitignore)

**`.planning/codebase/`:**
- Purpose: GSD analysis documents
- Generated: Yes (by GSD analyzer)
- Committed: Yes (future reference)

---

*Structure analysis: 2026-02-26*
