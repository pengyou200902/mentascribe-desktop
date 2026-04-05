# Codebase Structure

**Analysis Date:** 2026-04-05

## Directory Layout

```
mentascribe-desktop/
├── src/                              # React/TypeScript frontend (Vite + React 18)
│   ├── main.tsx                      # React DOM entry point
│   ├── App.tsx                       # Root component (window routing, event setup, recording state)
│   ├── components/                   # React UI components
│   │   ├── DictationBar.tsx          # Floating pill overlay (waveform, processing, errors)
│   │   ├── Settings.tsx              # Legacy settings editor (props-based)
│   │   ├── History.tsx               # Legacy history viewer (localStorage-based)
│   │   ├── MenuBar.tsx               # Legacy menu bar component
│   │   ├── TranscriptionOverlay.tsx  # Legacy overlay component
│   │   └── dashboard/               # Multi-page dashboard window
│   │       ├── Dashboard.tsx         # Page router + ThemeProvider wrapper
│   │       ├── Sidebar.tsx           # Navigation sidebar with icons
│   │       ├── HomePage.tsx          # Stats overview (streak, words, time)
│   │       ├── HistoryPage.tsx       # Transcription history list with CRUD
│   │       ├── DictionaryPage.tsx    # Text replacement rule management
│   │       └── SettingsPage.tsx      # Full settings UI (transcription, hotkey, output, appearance)
│   ├── lib/                          # Shared utilities and state stores
│   │   ├── store.ts                  # Main Zustand store (settings via useStore hook)
│   │   ├── historyStore.ts           # History Zustand store (paginated, useHistoryStore)
│   │   ├── dictionaryStore.ts        # Dictionary Zustand store (useDictionaryStore)
│   │   ├── statsStore.ts             # Stats Zustand store (useStatsStore)
│   │   ├── tauri.ts                  # Typed wrappers around invoke() calls
│   │   └── theme.tsx                 # ThemeProvider React context (light/dark/system)
│   ├── config/                       # Frontend constants
│   │   └── widget.ts                 # All dictation bar animation/timing constants
│   ├── types/                        # TypeScript type definitions
│   │   └── index.ts                  # Shared interfaces (TranscriptionEntry, DictionaryEntry, etc.)
│   ├── styles/                       # CSS
│   │   └── globals.css               # Tailwind directives + custom dictation pill CSS + fonts
│   └── icons/                        # Application icons (PNG, ICO, ICNS, SVG)
│
├── src-tauri/                        # Rust backend (Tauri v2)
│   ├── Cargo.toml                    # Rust dependencies and feature flags
│   ├── Cargo.lock                    # Locked dependency versions
│   ├── build.rs                      # Build script (tauri_build + cc for Voxtral C compilation)
│   ├── tauri.conf.json               # Tauri config: windows, app metadata, plugins, bundle targets
│   ├── Info.plist                     # macOS bundle info
│   ├── capabilities/                 # Tauri v2 permission system
│   │   └── default.json             # Allowed Tauri APIs per window
│   ├── gen/                          # Auto-generated Tauri schemas
│   │   └── schemas/                 # ACL manifests, capability schemas
│   ├── icons/                        # App icons for bundling
│   ├── src/                          # Rust source code
│   │   ├── main.rs                   # Binary entry (calls lib::run())
│   │   ├── lib.rs                    # Core app (~1670 lines): setup, AppState, all commands
│   │   ├── audio/                    # Audio capture subsystem
│   │   │   ├── mod.rs               # Module exports (AudioData)
│   │   │   ├── capture.rs           # CPAL device management, streaming, resampling, level metering
│   │   │   └── vad.rs               # Energy-based voice activity detection
│   │   ├── transcription/            # Speech-to-text engines
│   │   │   ├── mod.rs               # Shared types (ModelInfo, CoremlStatus, VoxtralStatus)
│   │   │   ├── whisper.rs           # Whisper.cpp: model cache, streaming VAD, batch inference
│   │   │   ├── cloud.rs             # Cloud STT stub (OpenAI, etc.)
│   │   │   ├── voxtral.rs           # Voxtral engine wrapper (feature-gated)
│   │   │   └── voxtral_ffi.rs       # C FFI bindings for Voxtral (feature-gated)
│   │   ├── settings/                 # Configuration persistence
│   │   │   └── mod.rs               # UserSettings structs, load/save to JSON
│   │   ├── hotkey/                   # Global keyboard shortcuts
│   │   │   └── mod.rs               # F1-F12 registration via tauri-plugin-global-shortcut
│   │   ├── injection/                # Text insertion into active app
│   │   │   └── mod.rs               # Platform-specific: macOS CGEvent, Windows clipboard, Linux X11
│   │   ├── history/                  # Transcription log
│   │   │   └── mod.rs               # CRUD operations, JSON persistence
│   │   ├── dictionary/               # Text replacement engine
│   │   │   └── mod.rs               # RwLock cache, phrase matching, JSON persistence
│   │   ├── stats/                    # Usage metrics
│   │   │   └── mod.rs               # Daily counters, streak tracking, JSON persistence
│   │   ├── text/                     # Post-transcription processing
│   │   │   └── mod.rs               # Auto-capitalization (with unit tests)
│   │   └── api/                      # External API client
│   │       ├── mod.rs               # AuthToken, UserInfo types
│   │       └── client.rs            # HTTP client for MentaFlux API, keyring token storage
│   └── voxtral/                      # Voxtral C source (compiled via build.rs)
│       ├── voxtral.c                # Core inference logic
│       ├── voxtral.h                # Public API header
│       ├── voxtral_audio.c/h        # Audio preprocessing
│       ├── voxtral_decoder.c        # Decoder implementation
│       ├── voxtral_encoder.c        # Encoder implementation
│       ├── voxtral_kernels.c/h      # CPU compute kernels
│       ├── voxtral_tokenizer.c/h    # Tokenizer implementation
│       ├── voxtral_safetensors.c/h  # Model weight loading
│       ├── voxtral_metal.h/m        # Metal GPU bridge (Objective-C)
│       ├── voxtral_shaders.metal    # Metal GPU shader source
│       ├── voxtral_mic.h            # Microphone abstraction
│       ├── voxtral_mic_macos.c      # macOS mic implementation
│       ├── download_model.sh        # Model download script
│       └── LICENSE                   # Voxtral license
│
├── index.html                        # HTML shell (transparent background, theme flash prevention)
├── package.json                      # Frontend dependencies and scripts
├── package-lock.json                 # Locked npm dependencies
├── tsconfig.json                     # TypeScript config (strict mode)
├── tsconfig.node.json                # Vite-specific TypeScript config
├── vite.config.ts                    # Vite bundler config (React plugin, Tauri env prefix)
├── tailwind.config.js                # Tailwind CSS config (custom amber/stone palette, dark mode)
├── postcss.config.js                 # PostCSS config (Tailwind + autoprefixer)
├── .gitignore                        # Git ignore rules
├── LICENSE                           # MIT license
├── README.md                         # Project documentation
├── images/                           # Screenshots and demo assets for README
└── .planning/                        # GSD planning system
    └── codebase/                    # Codebase analysis documents
```

## Directory Purposes

**`src/`** -- React frontend application
- Bundled by Vite, served on `localhost:1420` during development
- Built to `dist/` for production via `npm run build`
- Single React app serves both window types via URL hash detection
- Entry: `main.tsx` mounts `App` at `#root`

**`src/components/`** -- React UI components
- `DictationBar.tsx`: Core overlay widget (~260 lines). Renders a floating pill with states: collapsed, expanded-idle, recording (waveform), processing (dots+spinner), error, initializing. Uses `ResizeObserver` to dynamically resize Tauri window. Cursor hover detection via Rust invoke polling.
- `dashboard/`: Multi-page dashboard with sidebar navigation. `Dashboard.tsx` wraps content in `ThemeProvider`, routes pages via state. Pages: Home (stats), History (list), Dictionary (CRUD), Settings (all config).
- Legacy components (`Settings.tsx`, `History.tsx`, `MenuBar.tsx`, `TranscriptionOverlay.tsx`): Older implementations from pre-dashboard architecture. Not rendered in current UI but still present in the codebase.

**`src/lib/`** -- Shared state and utilities
- Four Zustand stores, each following the pattern: define interface, `create<Interface>((set, get) => ({}))`, export hook
- `tauri.ts`: Typed wrapper functions around `invoke()` for recording, text injection, login, model management
- `theme.tsx`: React Context + Provider for theme management, persists to `localStorage('mentascribe-theme')`

**`src/config/`** -- Frontend constants
- `widget.ts`: ~40 exported constants for dictation bar behavior (waveform animation, polling intervals, error timeouts, defaults). Centralized to avoid magic numbers in components.

**`src/types/`** -- Shared TypeScript interfaces
- `index.ts`: Types matching Rust backend structs: `DailyStats`, `LocalStats`, `TranscriptionEntry`, `DictionaryEntry`, `DashboardPage` type alias

**`src-tauri/src/`** -- Rust backend
- `lib.rs` is the monolithic core: all Tauri command handlers, `AppState`, `run()` setup, macOS NSPanel management, native drag, window positioning. This is the largest file at ~1670 lines.
- Each subsystem is a separate module directory with `mod.rs` (and optionally submodules).
- Subsystems are loosely coupled: `lib.rs` orchestrates calls between them.

**`src-tauri/voxtral/`** -- Voxtral C engine source
- Complete custom speech-to-text engine written in C
- Compiled by `build.rs` using the `cc` crate
- Feature-gated: only compiled when `--features voxtral` is passed
- Metal GPU shaders precompiled to `.metallib` at build time on macOS

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Frontend bootstrap (React DOM mount)
- `src/App.tsx`: Root component, window type routing, all event listener setup
- `src-tauri/src/main.rs`: Rust binary entry (delegates to `lib::run()`)
- `src-tauri/src/lib.rs`: Application initialization (`pub fn run()`), all command handlers

**Configuration:**
- `package.json`: npm scripts (`dev`, `build`, `tauri`, `lint`, `format`, `typecheck`)
- `src-tauri/Cargo.toml`: Rust deps, platform-specific deps, feature flags (`voxtral`, `custom-protocol`)
- `src-tauri/tauri.conf.json`: Window definitions (dictation: 52x10 transparent overlay; dashboard: 800x600 standard), bundle targets (dmg, msi, nsis, appimage, deb, rpm), CSP, plugins
- `src-tauri/capabilities/default.json`: Tauri v2 permission grants for both windows
- `tsconfig.json`: TypeScript strict mode, ES2021 target
- `vite.config.ts`: Vite dev server port 1420, Tauri env prefix, build targets
- `tailwind.config.js`: Custom color palette (amber brand, stone neutrals), dark mode via `class`

**Core Recording Pipeline:**
- `src-tauri/src/lib.rs` (`start_recording`, `stop_recording`): Orchestration
- `src-tauri/src/audio/capture.rs`: Audio device, sample buffering, real-time 16kHz resampling
- `src-tauri/src/transcription/whisper.rs`: Model loading, streaming inference, final batch inference
- `src-tauri/src/text/mod.rs`: Auto-capitalization
- `src-tauri/src/dictionary/mod.rs`: Text replacement application
- `src-tauri/src/injection/mod.rs`: Text insertion into active app

**Persistent Data (all at `~/.config/mentascribe/`):**
- `settings.json`: User preferences (managed by `src-tauri/src/settings/mod.rs`)
- `history.json`: Transcription log (managed by `src-tauri/src/history/mod.rs`)
- `dictionary.json`: Replacement rules (managed by `src-tauri/src/dictionary/mod.rs`)
- `stats.json`: Usage metrics (managed by `src-tauri/src/stats/mod.rs`)

**ML Models (at `~/.mentascribe/models/`):**
- Whisper GGML models: `ggml-tiny.bin`, `ggml-base.bin`, `ggml-small.bin`, `ggml-medium.bin`, `ggml-large-v3.bin`
- CoreML encoders: `ggml-{size}-encoder.mlmodelc/` directories
- VAD model: `ggml-silero-vad.bin`
- Voxtral model: downloaded via `download_model.sh` or `download_voxtral_model` command

## Module Organization

**Frontend modules communicate through:**
1. Zustand stores (global state shared across components)
2. Props (parent-to-child data flow)
3. Tauri events (backend-to-frontend via `listen()`)
4. Tauri commands (frontend-to-backend via `invoke()`)

**Backend modules communicate through:**
1. Direct function calls from `lib.rs` command handlers into subsystem modules
2. `AppState` managed state (passed via `tauri::State` parameter in commands)
3. Static globals within modules (lazy_static/once_cell for caches)
4. Tauri events emitted to frontend (progress, completion, errors)

**Module dependency graph (Rust):**
```
lib.rs (orchestrator)
├── audio::capture  (start/stop capture, get level)
├── audio::vad      (used by capture internally)
├── transcription::whisper (transcribe, preload, download, streaming)
├── transcription::voxtral (feature-gated alternative engine)
├── transcription::cloud   (stub for cloud fallback)
├── settings       (load/save UserSettings)
├── hotkey         (setup/unregister global shortcuts)
├── injection      (inject text into active app)
├── history        (add/get/delete transcription entries)
├── dictionary     (get/add/update/remove rules, apply replacements)
├── stats          (record transcription, get stats)
├── text           (process_text post-transcription)
└── api::client    (login, token management)
```

## Naming Conventions

**Files:**
- React components: `PascalCase.tsx` -- `DictationBar.tsx`, `Dashboard.tsx`, `SettingsPage.tsx`
- TypeScript utilities/stores: `camelCase.ts` -- `store.ts`, `historyStore.ts`, `tauri.ts`
- TypeScript config: `camelCase.ts` -- `widget.ts`
- Rust modules: `snake_case.rs` -- `capture.rs`, `mod.rs`, `client.rs`
- Rust feature modules: `snake_case/mod.rs` directory pattern
- Config files: standard names -- `package.json`, `Cargo.toml`, `tauri.conf.json`

**Directories:**
- React feature groups: `lowercase/` -- `components/`, `dashboard/`, `lib/`, `config/`, `types/`
- Rust subsystems: `snake_case/` -- `audio/`, `transcription/`, `settings/`, `hotkey/`, `injection/`

**Import conventions:**
- Frontend: `@tauri-apps/api/core` for `invoke`, `@tauri-apps/api/event` for `listen`
- Zustand stores imported as hooks: `import { useStore } from '../lib/store'`
- Types imported with `type` keyword: `import type { TranscriptionEntry } from '../types'`
- Rust: module paths like `crate::audio::AudioData`, `crate::settings::UserSettings`

## Where to Add New Code

**New Tauri Command:**
1. Define `#[tauri::command]` function in `src-tauri/src/lib.rs` (or in a subsystem module, called from lib.rs)
2. Register in `tauri::generate_handler![]` array in the `run()` function
3. Add TypeScript wrapper in `src/lib/tauri.ts` (optional but recommended for typing)
4. Call via `invoke('command_name', { args })` from frontend

**New Dashboard Page:**
1. Create component: `src/components/dashboard/MyPage.tsx`
2. Add page type to `DashboardPage` union in `src/types/index.ts`
3. Add navigation item in `src/components/dashboard/Sidebar.tsx` (add to `navItems` array)
4. Add case to `renderPage()` switch in `src/components/dashboard/Dashboard.tsx`

**New Rust Subsystem Module:**
1. Create directory: `src-tauri/src/my_feature/`
2. Create `src-tauri/src/my_feature/mod.rs` with public API functions
3. Add `mod my_feature;` declaration at top of `src-tauri/src/lib.rs`
4. Call functions from command handlers in `lib.rs`
5. Use `thiserror::Error` for module-specific error types
6. Use `serde::{Serialize, Deserialize}` for types crossing the IPC boundary

**New Zustand Store:**
1. Create `src/lib/myFeatureStore.ts`
2. Follow existing pattern: define interface, `create<Interface>((set, get) => ({}))`
3. Export custom hook: `export const useMyFeatureStore = create<MyFeatureStore>(...)`
4. All backend calls via `invoke()` inside store actions

**New Frontend Component:**
1. Create in `src/components/` (top-level) or `src/components/dashboard/` (dashboard page)
2. Use functional component with TypeScript props interface
3. Import Zustand stores for shared state, `invoke` for backend calls, `listen` for events
4. Follow Tailwind CSS patterns with `dark:` variants for theme support

**New Persistent Data Module (Rust):**
1. Create module in `src-tauri/src/my_data/mod.rs`
2. Define data structs with `#[derive(Serialize, Deserialize, Default)]`
3. Use `dirs::config_dir().join("mentascribe").join("my_data.json")` for storage path
4. Implement `load_from_disk()` and `save_to_disk()` functions
5. Consider `Lazy<RwLock<>>` cache if data is read frequently (see `dictionary/mod.rs` pattern)

**Tests:**
- Rust: Inline `#[cfg(test)] mod tests { ... }` at bottom of module file (see `src-tauri/src/text/mod.rs`)
- TypeScript: No test framework configured; would need Jest/Vitest setup

## Special Directories

**`src-tauri/target/`:**
- Purpose: Cargo build outputs (debug and release binaries, dependencies)
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)
- Can be very large (10+ GB with debug symbols)

**`dist/`:**
- Purpose: Vite build output (bundled frontend JS, CSS, assets)
- Generated: Yes (by `npm run build` or `pnpm build`)
- Committed: No (in `.gitignore`)
- Consumed by Tauri as `frontendDist: "../dist"` in `tauri.conf.json`

**`src-tauri/gen/`:**
- Purpose: Tauri-generated schema files for IDE support and ACL validation
- Generated: Yes (by Tauri CLI during build)
- Committed: Yes (checked into git for IDE schema resolution)

**`node_modules/`:**
- Purpose: npm/pnpm package dependencies
- Generated: Yes (by `npm install` or `pnpm install`)
- Committed: No (in `.gitignore`)

**`images/`:**
- Purpose: Screenshots and demo assets referenced in README.md
- Generated: No (manually added)
- Committed: Yes

**`.planning/codebase/`:**
- Purpose: GSD codebase analysis documents (this file and others)
- Generated: Yes (by GSD analysis agents)
- Committed: Yes

**`src/icons/` and `src-tauri/icons/`:**
- Purpose: Application icons in multiple formats/sizes for bundling
- Generated: No (designed assets)
- Committed: Yes
- Note: Duplicated between `src/icons/` (frontend) and `src-tauri/icons/` (bundle)

---

*Structure analysis: 2026-04-05*
