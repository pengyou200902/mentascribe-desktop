# Codebase Structure

**Analysis Date:** 2026-02-20

## Directory Layout

```
mentascribe-desktop/
├── src/                           # Frontend React/TypeScript
│   ├── App.tsx                    # Root component, window detection, event setup
│   ├── main.tsx                   # React DOM render entry point
│   ├── components/                # Reusable React components
│   │   ├── DictationBar.tsx       # Fullscreen overlay UI (waveform, record button)
│   │   ├── MenuBar.tsx            # Menu bar with theme toggle
│   │   ├── History.tsx            # Transcription history (legacy, see HistoryPage)
│   │   ├── Settings.tsx           # Legacy settings UI
│   │   ├── TranscriptionOverlay.tsx # Unused overlay component
│   │   └── dashboard/             # Dashboard (settings, history, stats)
│   │       ├── Dashboard.tsx      # Dashboard router component
│   │       ├── HomePage.tsx       # Dashboard home page
│   │       ├── HistoryPage.tsx    # Browse transcription history
│   │       ├── DictionaryPage.tsx # Manage dictionary replacements
│   │       ├── SettingsPage.tsx   # All user settings
│   │       └── Sidebar.tsx        # Dashboard sidebar navigation
│   ├── hooks/                     # Custom React hooks (if any)
│   ├── icons/                     # SVG icon components
│   ├── lib/                       # Utilities and stores
│   │   ├── store.ts              # Zustand settings store (main state management)
│   │   ├── tauri.ts              # Tauri API wrappers/helpers
│   │   ├── theme.tsx             # Theme provider (light/dark)
│   │   ├── historyStore.ts       # Zustand history store
│   │   ├── dictionaryStore.ts    # Zustand dictionary store
│   │   └── statsStore.ts         # Zustand stats store
│   ├── styles/                    # Global CSS and Tailwind
│   │   └── globals.css            # Global styles, Tailwind directives
│   └── types/                     # TypeScript type definitions
│       └── index.ts               # Exported types (DashboardPage, UserSettings, etc.)
├── src-tauri/                     # Rust backend (Tauri 2)
│   ├── src/                       # Rust source code
│   │   ├── main.rs                # Binary entry point (calls lib.rs::run())
│   │   ├── lib.rs                 # Library entry point: Tauri setup, commands, window creation
│   │   ├── audio/                 # Audio capture and processing
│   │   │   ├── mod.rs             # Module definition (re-exports)
│   │   │   ├── capture.rs         # CPAL audio stream, resampling to 16kHz mono
│   │   │   └── vad.rs             # Voice Activity Detection (VAD) for streaming transcription
│   │   ├── transcription/         # Speech-to-text
│   │   │   ├── mod.rs             # Module definition, ModelInfo structs
│   │   │   ├── whisper.rs         # Local Whisper (whisper-rs crate)
│   │   │   └── cloud.rs           # Cloud providers (AWS, OpenAI, AssemblyAI)
│   │   ├── injection/             # Text injection into focused app
│   │   │   └── mod.rs             # Platform-specific text typing via CGEvent (macOS), Win32 (Windows), X11 (Linux)
│   │   ├── hotkey/                # Global hotkey registration
│   │   │   └── mod.rs             # F1–F12 hotkey parsing and setup
│   │   ├── settings/              # User settings persistence
│   │   │   └── mod.rs             # Serde structs, load/save from ~/.config/mentascribe/settings.json
│   │   ├── history/               # Transcription history persistence
│   │   │   └── mod.rs             # TranscriptionEntry, load/save from ~/.config/mentascribe/history.json
│   │   ├── dictionary/            # Dictionary replacements (e.g., "teh" → "the")
│   │   │   └── mod.rs             # Load dictionary rules, apply replacements to text
│   │   ├── stats/                 # Usage statistics
│   │   │   └── mod.rs             # Record word count, duration, frequency
│   │   ├── text/                  # Text post-processing
│   │   │   └── mod.rs             # Auto-capitalize, punctuation rules
│   │   └── api/                   # External API client (future cloud features)
│   │       ├── mod.rs             # API client setup
│   │       └── client.rs          # HTTP request helpers
│   ├── Cargo.toml                 # Rust dependencies: tauri, cpal, whisper-rs, etc.
│   ├── tauri.conf.json            # Tauri app config (window layout, resources)
│   └── build.rs                   # Build script for platform-specific features
├── dist/                          # Built frontend (vite output)
├── .planning/                     # Codebase documentation
│   └── codebase/                  # GSD analysis documents
├── docs/                          # User/developer documentation
├── vite.config.ts                 # Vite bundler config for frontend
├── tsconfig.json                  # TypeScript compiler options
├── package.json                   # Frontend dependencies (React, Zustand, Tauri API)
├── postcss.config.js              # PostCSS + Tailwind integration
├── tailwind.config.js             # Tailwind CSS config
├── index.html                     # HTML entry point
└── README.md                      # Project overview
```

## Directory Purposes

**src/**
- Purpose: All frontend React/TypeScript code
- Contains: Components, stores, types, styles
- Key files: `App.tsx` (root), `DictationBar.tsx` (overlay UI), `store.ts` (state)

**src/components/**
- Purpose: Reusable UI components
- Contains: React functional components (.tsx)
- Key files: `DictationBar.tsx` (main overlay), `dashboard/*` (settings/history/stats pages)

**src/components/dashboard/**
- Purpose: Dashboard window UI (settings, history, stats, dictionary)
- Contains: Page components for each dashboard section
- Key files: `Dashboard.tsx` (router), `SettingsPage.tsx` (all settings), `HistoryPage.tsx` (past transcriptions)

**src/lib/**
- Purpose: Utility functions and state management
- Contains: Zustand stores, Tauri API wrappers, theme provider
- Key files: `store.ts` (settings store), `historyStore.ts` (history state), `tauri.ts` (invoke wrappers)

**src/styles/**
- Purpose: Global CSS
- Contains: Tailwind directives, global styles
- Key files: `globals.css` (main stylesheet)

**src/types/**
- Purpose: Shared TypeScript types
- Contains: Interfaces for settings, components, pages
- Key files: `index.ts` (all exported types)

**src-tauri/src/**
- Purpose: All Rust backend code
- Contains: Audio, transcription, window management, settings, persistence
- Key files: `lib.rs` (app initialization), `audio/capture.rs` (CPAL integration)

**src-tauri/src/audio/**
- Purpose: Audio input capture and pre-processing
- Contains: CPAL stream setup, resampling to 16kHz mono for Whisper, VAD triggering
- Key files: `capture.rs` (main logic), `vad.rs` (voice detection)

**src-tauri/src/transcription/**
- Purpose: Speech-to-text engines
- Contains: Local Whisper integration, cloud API wrappers
- Key files: `whisper.rs` (whisper-rs calls), `cloud.rs` (external APIs)

**src-tauri/src/injection/**
- Purpose: Text injection into focused application
- Contains: Platform-specific implementations (macOS CoreGraphics, Windows Win32, Linux X11)
- Key files: `mod.rs` (public inject_text function, platform gates)

**src-tauri/src/settings/**
- Purpose: User settings persistence and access
- Contains: Serde structs for all settings, disk I/O
- Key files: `mod.rs` (load/save from ~/.config/mentascribe/settings.json)

**src-tauri/src/history/**
- Purpose: Transcription history storage
- Contains: TranscriptionEntry struct, disk persistence
- Key files: `mod.rs` (add_entry, get_all, delete_entry functions)

**src-tauri/src/dictionary/**
- Purpose: Text replacements (custom words/abbreviations)
- Contains: Dictionary loading, replacement logic
- Key files: `mod.rs` (apply_replacements function)

**src-tauri/src/stats/**
- Purpose: User statistics (words transcribed, time spent, etc.)
- Contains: Recording stats persistence
- Key files: `mod.rs` (record_transcription function)

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React app mount
- `src/App.tsx`: Root component, window type detection, event setup
- `src-tauri/src/main.rs`: Rust binary (calls lib::run)
- `src-tauri/src/lib.rs`: Tauri app initialization, all commands, window creation

**Configuration:**
- `vite.config.ts`: Frontend bundling
- `tsconfig.json`: TypeScript settings
- `src-tauri/Cargo.toml`: Rust dependencies
- `src-tauri/tauri.conf.json`: Tauri window/resource config

**Core Logic:**
- `src-tauri/src/audio/capture.rs`: CPAL audio stream, sample buffering
- `src-tauri/src/transcription/whisper.rs`: Whisper transcription
- `src-tauri/src/injection/mod.rs`: Text injection to focused app
- `src-tauri/src/hotkey/mod.rs`: Global hotkey registration

**State Management:**
- `src/lib/store.ts`: Zustand settings store (main state)
- `src-tauri/src/lib.rs`: AppState Mutex<> (recording flag, settings in-memory copy)
- `~/.config/mentascribe/settings.json`: Settings persistence

**UI Components:**
- `src/components/DictationBar.tsx`: Main overlay (waveform, record button)
- `src/components/dashboard/Dashboard.tsx`: Settings/history/stats router
- `src/components/dashboard/SettingsPage.tsx`: All user settings UI

**Data Storage:**
- `~/.config/mentascribe/settings.json`: User configuration
- `~/.config/mentascribe/history.json`: Transcription records
- `~/.config/mentascribe/dictionary.json`: Text replacements
- `~/.config/mentascribe/stats.json`: Usage statistics

## Naming Conventions

**Files:**
- React components: PascalCase (e.g., `DictationBar.tsx`, `SettingsPage.tsx`)
- Stores/utilities: camelCase (e.g., `store.ts`, `historyStore.ts`, `tauri.ts`)
- Rust modules: snake_case (e.g., `audio.rs`, `injection.rs`, `vad.rs`)
- Directories: lowercase (e.g., `components/`, `lib/`, `audio/`)

**Functions:**
- Rust: snake_case (e.g., `start_capture()`, `apply_replacements()`)
- TypeScript: camelCase (e.g., `startRecording()`, `saveToHistory()`)

**Variables:**
- Rust: snake_case for all (e.g., `is_recording`, `sample_rate`, `audio_buffer`)
- TypeScript: camelCase (e.g., `isRecording`, `audioLevel`, `waveformBars`)
- React state: camelCase (e.g., `const [isRecording, setIsRecording]`)

**Types:**
- Rust structs: PascalCase (e.g., `AudioData`, `UserSettings`, `TranscriptionEntry`)
- TypeScript interfaces: PascalCase (e.g., `UserSettings`, `DictationBarProps`)
- TypeScript types (union/enums): PascalCase (e.g., `WindowType`, `DashboardPage`)

## Where to Add New Code

**New Feature (e.g., speech enhancement):**
- Primary code: `src-tauri/src/audio/[new_module].rs`
- Frontend trigger: `src/components/dashboard/SettingsPage.tsx` (add toggle)
- Settings struct: `src-tauri/src/settings/mod.rs` (add field to AudioSettings or new sub-struct)
- Command: `src-tauri/src/lib.rs` (add new #[tauri::command])

**New Component/Module:**
- React component: `src/components/[ComponentName].tsx`
- Page in dashboard: `src/components/dashboard/[PageName].tsx`
- Export from: `src/components/index.ts` (if creating barrel export)

**Utilities:**
- Frontend helpers: `src/lib/[utilName].ts` (avoid mixing in stores)
- Rust helpers: `src-tauri/src/[domain]/[helper].rs` (within domain module)
- Shared types: `src/types/index.ts` (frontend) or `src-tauri/src/[domain]/mod.rs` (backend)

**Tests:**
- Frontend tests: Co-locate as `[ComponentName].test.tsx` or in `src/__tests__/` (not yet configured)
- Rust tests: Inline with `#[cfg(test)]` modules in same file or `tests/` directory (not yet used)

**Styling:**
- Global styles: `src/styles/globals.css`
- Component-scoped: Tailwind classes in JSX (no separate CSS files currently used)
- Theme: `src/lib/theme.tsx` (light/dark provider)

## Special Directories

**dist/**
- Purpose: Built frontend (output from vite build)
- Generated: Yes
- Committed: No (in .gitignore)

**node_modules/**
- Purpose: Frontend dependency packages
- Generated: Yes (npm install)
- Committed: No (in .gitignore)

**src-tauri/target/**
- Purpose: Rust build artifacts
- Generated: Yes (cargo build)
- Committed: No (in .gitignore)

**.planning/codebase/**
- Purpose: GSD analysis documents (ARCHITECTURE.md, STRUCTURE.md, etc.)
- Generated: No (manually maintained)
- Committed: Yes

**~/.config/mentascribe/**
- Purpose: User data (settings, history, dictionary, stats)
- Location: Outside repo (user's home directory)
- Generated: Yes (at runtime)
- Committed: No

## Cross-Platform File Handling

**Settings location:** `~/.config/mentascribe/settings.json` (XDG_CONFIG_HOME on Linux, ~/Library/Preferences on macOS)
- Via `dirs::config_dir()` in Rust (automatically platform-aware)

**Audio input:** Handled via CPAL device selection (automatic best-match device)

**Text injection:** Platform-specific implementations in `injection/mod.rs`
- macOS: CoreGraphics CGEvent with Unicode string splitting (20-char limit)
- Windows: Windows API via windows crate
- Linux: X11 xtest via x11 crate

## Tauri Window Architecture

**Two windows defined in src-tauri/tauri.conf.json:**
1. **Dictation window** (label: "dictation")
   - Role: Fullscreen overlay
   - Converted to NSPanel on macOS via `setup_dictation_panel()`
   - Position: Bottom-center of current monitor, non-activating
   - Draggable: Controlled by settings, toggled via JS in DictationBar

2. **Dashboard window** (label: "dashboard")
   - Role: Settings, history, stats, dictionary
   - Always-on-top on Windows/Linux, brought to foreground on macOS
   - URL routes via hash: `#dashboard/settings`, `#dashboard/history`, etc.

**Window communication:**
- Both windows share same app context
- Events broadcasted across windows via `app.emit()`
- Settings changes in one window propagated to other via "settings-changed" event

---

*Structure analysis: 2026-02-20*
