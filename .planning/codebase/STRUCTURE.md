# Codebase Structure

**Analysis Date:** 2026-02-24

## Directory Layout

```
mentascribe-desktop/
├── index.html              # Tauri WebviewWindow entry point
├── package.json            # Frontend dependencies (React, Tauri, Tailwind, etc)
├── tsconfig.json           # TypeScript config
├── vite.config.ts          # Vite build config
├── tailwind.config.js      # Tailwind CSS config
├── postcss.config.js       # PostCSS config
│
├── src/                    # Frontend (React/TypeScript)
│   ├── main.tsx            # React root mount
│   ├── App.tsx             # Main router (dictation vs dashboard)
│   ├── types/              # Shared types (DashboardPage, TranscriptionEntry, etc)
│   │   └── index.ts
│   ├── config/             # Centralized constants
│   │   └── widget.ts       # Dictation pill animation/timing constants
│   ├── lib/                # Shared utilities and state
│   │   ├── store.ts        # Zustand settings store
│   │   ├── tauri.ts        # Tauri command wrappers (deprecated, use invoke directly)
│   │   ├── theme.tsx       # Theme provider (light/dark)
│   │   ├── historyStore.ts # Zustand history state
│   │   ├── dictionaryStore.ts # Zustand dictionary state
│   │   └── statsStore.ts   # Zustand stats state
│   ├── styles/             # Global CSS
│   │   └── globals.css     # Tailwind imports, shared styles
│   ├── components/         # Reusable UI components
│   │   ├── DictationBar.tsx      # Fullscreen overlay (pill + waveform + error)
│   │   ├── History.tsx            # Transcription history list
│   │   ├── Settings.tsx           # Settings form (not used in dashboard, duplicated)
│   │   ├── MenuBar.tsx            # Menu bar (not used)
│   │   ├── TranscriptionOverlay.tsx # Overlay layer (not used)
│   │   └── dashboard/             # Dashboard window pages
│   │       ├── Dashboard.tsx      # Main layout + router (Sidebar + page content)
│   │       ├── HomePage.tsx       # Statistics display
│   │       ├── HistoryPage.tsx    # Transcription history with search/delete
│   │       ├── DictionaryPage.tsx # Word replacement dictionary
│   │       ├── SettingsPage.tsx   # Configuration (transcription engine, hotkey, etc)
│   │       └── Sidebar.tsx        # Navigation tabs
│   ├── hooks/              # React hooks (likely empty or unused)
│   ├── icons/              # SVG icon components
│   └── assets/             # Static assets (if any)
│
├── src-tauri/              # Rust backend (Tauri app)
│   ├── Cargo.toml          # Rust dependencies (whisper-rs, cpal, voxtral FFI, etc)
│   ├── Cargo.lock
│   ├── tauri.conf.json     # Tauri app config (windows, permissions, tray menu)
│   │
│   ├── src/                # Rust source
│   │   ├── main.rs         # Binary entry (calls lib::run())
│   │   ├── lib.rs          # Command handlers, AppState, window setup
│   │   │                   #   - setup_dictation_panel() for macOS NSPanel conversion
│   │   │                   #   - start_recording(), stop_recording() commands
│   │   │                   #   - Event emission setup
│   │   │
│   │   ├── audio/          # Audio capture and VAD (Voice Activity Detection)
│   │   │   ├── mod.rs      # Module exports
│   │   │   ├── capture.rs  # cpal-based audio stream, frame chunking, PCM preparation
│   │   │   └── vad.rs      # Silero VAD model (silence detection)
│   │   │
│   │   ├── transcription/  # Transcription engine selection and models
│   │   │   ├── mod.rs      # ModelInfo, CoremlStatus, MetalStatus, VoxtralStatus structs
│   │   │   ├── whisper.rs  # Whisper.cpp streaming + state cache (ggml models)
│   │   │   ├── cloud.rs    # Cloud transcription provider support (stub)
│   │   │   ├── voxtral.rs  # Voxtral wrapper (feature-gated, faster streaming)
│   │   │   └── voxtral_ffi.rs # FFI bindings to Voxtral C library
│   │   │
│   │   ├── settings/       # User settings persistence
│   │   │   └── mod.rs      # UserSettings struct, load/save to ~/.config/mentascribe/settings.json
│   │   │
│   │   ├── hotkey/         # Global hotkey system
│   │   │   └── mod.rs      # tauri-plugin-global-shortcut, F1-F12 key parsing, hotkey-pressed/released events
│   │   │
│   │   ├── injection/      # Text injection into active app
│   │   │   └── mod.rs      # Platform-specific (macOS: CGEvent paste, Linux: X11)
│   │   │
│   │   ├── history/        # Transcription history persistence
│   │   │   └── mod.rs      # TranscriptionEntry struct, load/save to ~/.config/mentascribe/history.json
│   │   │
│   │   ├── dictionary/     # Word replacement dictionary
│   │   │   └── mod.rs      # DictionaryEntry struct, persistence (stub implementation)
│   │   │
│   │   ├── stats/          # Usage statistics
│   │   │   └── mod.rs      # LocalStats struct, track daily transcriptions/words/audio_seconds
│   │   │
│   │   ├── text/           # Text utilities
│   │   │   └── mod.rs      # cleanup_text() (remove filler words, add punctuation)
│   │   │
│   │   └── api/            # API client layer (unused, for future cloud integrations)
│   │       ├── mod.rs      # AuthToken, UserInfo structs
│   │       └── client.rs   # HTTP client skeleton
│   │
│   ├── voxtral/            # Voxtral model/library directory (feature-gated)
│   │   └── [voxtral build artifacts]
│   │
│   ├── icons/              # App icon files (macOS, Windows, Linux)
│   ├── capabilities/       # Tauri capabilities (permissions)
│   └── gen/                # Generated schemas, types

└── .planning/              # GSD planning docs
    └── codebase/           # This directory
        ├── ARCHITECTURE.md # (you are here)
        ├── STRUCTURE.md
        ├── CONVENTIONS.md
        ├── TESTING.md
        └── CONCERNS.md
```

## Directory Purposes

**Frontend (src/):**
- Purpose: React user interface, state management, event listeners
- Contains: TSX/TS components, Zustand stores, Tauri API wrappers
- Key files: `App.tsx` (router), `components/DictationBar.tsx` (overlay), `components/dashboard/Dashboard.tsx` (settings window)

**Backend (src-tauri/src/):**
- Purpose: Core logic (audio, transcription, settings, hotkey, injection)
- Contains: Rust modules for each domain concern
- Key files: `lib.rs` (command handlers, AppState), `audio/capture.rs`, `transcription/whisper.rs`

**Configuration (src-tauri/):**
- Purpose: Build and runtime config
- Contains: Cargo.toml, tauri.conf.json
- Key files: `Cargo.toml` (dependencies), `tauri.conf.json` (app config, window definition, tray menu)

**Assets (src-tauri/icons/):**
- Purpose: App icons for macOS, Windows, Linux
- Contains: PNG, ICNS, ICO files
- Generated by Tauri

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React app mount point
- `src-tauri/src/main.rs`: Rust binary entry (minimal, calls `lib::run()`)
- `src-tauri/src/lib.rs`: Tauri app setup, command registration

**Frontend Components:**
- `src/components/DictationBar.tsx`: Fullscreen overlay pill (waveform, recording state, error display)
- `src/components/dashboard/Dashboard.tsx`: Settings window main layout
- `src/components/dashboard/SettingsPage.tsx`: Hotkey, engine, opacity configuration UI

**Backend Modules:**
- `src-tauri/src/audio/capture.rs`: Audio stream + frame handling
- `src-tauri/src/transcription/whisper.rs`: Whisper streaming engine + model caching
- `src-tauri/src/injection/mod.rs`: Text injection (platform-specific)
- `src-tauri/src/hotkey/mod.rs`: Global hotkey registration

**State & Config:**
- `src/lib/store.ts`: Zustand settings store (frontend)
- `src-tauri/src/settings/mod.rs`: Settings persistence (backend, JSON file)
- `src/config/widget.ts`: Animation/timing constants (centralized)

**Types & Interfaces:**
- `src/types/index.ts`: Frontend TypeScript interfaces (DashboardPage, TranscriptionEntry, DictionaryEntry, LocalStats)
- `src-tauri/src/transcription/mod.rs`: ModelInfo, CoremlStatus, MetalStatus, VoxtralStatus (backend structs)

## Naming Conventions

**Files:**
- Components: `PascalCase.tsx` (e.g., `DictationBar.tsx`, `Dashboard.tsx`)
- Utilities/modules: `camelCase.ts` (e.g., `store.ts`, `widget.ts`)
- Rust modules: `snake_case.rs` (e.g., `capture.rs`, `whisper.rs`)
- Test files: `*.test.ts` or `*.spec.ts` (see TESTING.md)

**Directories:**
- Component groups: `camelCase/` (e.g., `components/dashboard/`)
- Rust modules: `snake_case/` (e.g., `src-tauri/src/audio/`)
- Config: `config/` (top-level, centralized)
- Utilities: `lib/` (shared helpers)

**TypeScript:**
- Interfaces: `PascalCase` (e.g., `UserSettings`, `TranscriptionEntry`)
- Functions: `camelCase` (e.g., `startRecording()`, `loadSettings()`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `WAVEFORM_BAR_COUNT`, `ERROR_TIMEOUT_MS`)

**Rust:**
- Structs: `PascalCase` (e.g., `UserSettings`, `ModelCache`)
- Functions: `snake_case` (e.g., `start_recording()`, `get_models_dir()`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `MODEL_BASE_URL`, `MAX_UTF16_UNITS_PER_EVENT`)
- Enums: `PascalCase` variants (e.g., `HotkeyError::RegisterError`)

## Where to Add New Code

**New Feature (End-to-End):**
- Create command in `src-tauri/src/lib.rs` decorated with `#[tauri::command]`
- Create wrapper in `src/lib/tauri.ts` or call `invoke()` directly in component
- Create UI component in `src/components/` or add to existing page
- Create Zustand store in `src/lib/` if stateful
- Add tests to `src-tauri/src/[module].rs` (#[cfg(test)])

**New Component/Module:**
- Frontend: Create file in `src/components/` with `.tsx` extension
- Backend: Create module in `src-tauri/src/[domain]/mod.rs` and declare in `src-tauri/src/lib.rs` with `mod [domain]`
- Export public items via `pub` keyword
- Maintain separation of concerns: UI in frontend, logic in backend

**Utilities:**
- Shared constants: `src/config/[domain].ts` (e.g., widget.ts for UI timing)
- Shared types: `src/types/index.ts`
- Tauri command wrappers: `src/lib/tauri.ts` (optional, can invoke directly)
- Rust helpers: `src-tauri/src/[module]/mod.rs` as free functions

**Stores/State:**
- Frontend reactive state: Zustand store in `src/lib/[domain]Store.ts`
- Backend persistent state: JSON file in `~/.config/mentascribe/` with load/save functions in `src-tauri/src/[domain]/mod.rs`
- Transient state: React component state via `useState()`

**Tests:**
- Unit tests: Inline in `src-tauri/src/[module].rs` in `#[cfg(test)]` blocks
- Component tests: Create `[Component].test.tsx` co-located with component
- Integration tests: Create `tests/` directory with end-to-end scenarios

## Special Directories

**Frontend:**
- `src/styles/`: Global CSS and Tailwind directives (not component-scoped)
- `src/icons/`: SVG icon components (reusable across pages)
- `src/hooks/`: Custom React hooks (currently sparse)

**Backend:**
- `src-tauri/voxtral/`: Voxtral C library and FFI bindings (feature-gated, large binary)
- `src-tauri/icons/`: App icons (ICNS, ICO, PNG) — generated by `cargo tauri icon` command
- `src-tauri/gen/`: Tauri-generated schemas and TypeScript bindings (auto-generated, do not edit)
- `src-tauri/capabilities/`: Tauri v2 capability definitions (permissions)

**Data Storage (User Home):**
- `~/.mentascribe/models/`: Whisper GGML models (ggml-tiny.bin, ggml-small.bin, etc) + optional CoreML encoders
- `~/.config/mentascribe/`: Settings, history, and dictionary JSON files
  - `settings.json`: User preferences (engine, hotkey, opacity, etc)
  - `history.json`: Transcription entries (max 500)
  - `dictionary.json`: Word replacement entries

**Build Artifacts:**
- `dist/`: Frontend build output (Vite)
- `src-tauri/target/`: Rust build output (Cargo)
- `.planning/codebase/`: GSD codebase mapping docs (not committed to dist)

## Patterns by Module

**Audio Capture:**
- `src-tauri/src/audio/capture.rs`: Starts cpal stream, feeds frames to VAD filter, emits audio-level events
- Pattern: `start_capture()` → `AUDIO_STREAM.set()` → callback emits events

**Transcription Selection:**
- `src-tauri/src/lib.rs::start_recording()` checks `settings.transcription.engine`
- Pattern: if engine == "voxtral" → call `voxtral::start_streaming()`, else → call `whisper::start_streaming()`

**Settings Sync:**
- Frontend: User changes setting → `updateSettings()` calls `invoke('update_settings')`
- Backend: `update_settings` command locks Mutex, writes JSON, re-registers hotkey if needed, emits `settings-changed` event
- Pattern: Dual source of truth, strong consistency via Mutex + JSON file

**Window Management:**
- Dictation window: Created in `lib.rs::run()`, converted to NSPanel in `setup_dictation_panel()`
- Dashboard window: Created with tray menu "Settings" action
- Pattern: Two independent Tauri WebviewWindows, separate HTML load paths (`#dictation` vs `#dashboard`)

---

*Structure analysis: 2026-02-24*
