# Project Structure

**Analysis Date:** 2026-04-06

## Directory Layout

```
mentascribe-desktop/
├── src/                        # Frontend (React/TypeScript)
│   ├── components/             # React components
│   │   ├── dashboard/          # Dashboard window components
│   │   │   ├── Dashboard.tsx   # Dashboard shell with sidebar + page routing
│   │   │   ├── DictionaryPage.tsx  # Custom word/phrase management
│   │   │   ├── HistoryPage.tsx     # Transcription history viewer
│   │   │   ├── HomePage.tsx        # Stats overview / landing page
│   │   │   ├── SettingsPage.tsx    # All user settings
│   │   │   └── Sidebar.tsx         # Dashboard navigation sidebar
│   │   ├── DictationBar.tsx    # Overlay pill widget (waveform, status, drag)
│   │   ├── History.tsx         # Legacy history component (dictation window)
│   │   ├── MenuBar.tsx         # Legacy menu bar component
│   │   ├── Settings.tsx        # Legacy settings component (dictation window)
│   │   └── TranscriptionOverlay.tsx  # Legacy transcription display
│   ├── config/
│   │   └── widget.ts           # Centralized UI constants (timing, sizing, defaults)
│   ├── icons/                  # App icons (png, icns, ico, svg)
│   ├── lib/
│   │   ├── dictionaryStore.ts  # Zustand store for dictionary CRUD
│   │   ├── historyStore.ts     # Zustand store for history (pagination, CRUD)
│   │   ├── statsStore.ts       # Zustand store for usage statistics
│   │   ├── store.ts            # Zustand store for settings (load/update)
│   │   ├── tauri.ts            # Typed wrappers around Tauri invoke() calls
│   │   └── theme.tsx           # Theme context provider (light/dark/system)
│   ├── styles/
│   │   └── globals.css         # Global CSS (Tailwind directives, custom styles)
│   ├── types/
│   │   └── index.ts            # Shared TypeScript interfaces (matches Rust structs)
│   ├── App.tsx                 # Root component (window routing, recording logic)
│   └── main.tsx                # React entry point (renders App into #root)
├── src-tauri/                  # Backend (Rust/Tauri)
│   ├── .cargo/
│   │   └── config.toml         # Cargo build config (CUDA host compiler workaround)
│   ├── capabilities/
│   │   └── default.json        # Tauri v2 capability permissions
│   ├── gen/                    # Auto-generated Tauri schemas (do not edit)
│   │   └── schemas/            # JSON schemas for capabilities validation
│   ├── icons/                  # Platform app icons for bundling
│   ├── src/
│   │   ├── api/
│   │   │   ├── mod.rs          # API types (AuthToken, UserInfo)
│   │   │   └── client.rs       # HTTP client (reqwest), auth, keychain storage
│   │   ├── audio/
│   │   │   ├── mod.rs          # Re-exports AudioData
│   │   │   ├── capture.rs      # CPAL mic capture, real-time resampling, levels
│   │   │   └── vad.rs          # Energy-based voice activity detector
│   │   ├── dictionary/
│   │   │   └── mod.rs          # Dictionary entries, RwLock cache, regex replacements
│   │   ├── history/
│   │   │   └── mod.rs          # Transcription history (JSON persistence, pagination)
│   │   ├── hotkey/
│   │   │   └── mod.rs          # Global shortcut registration (F1-F12)
│   │   ├── injection/
│   │   │   └── mod.rs          # Platform text injection (AX/CGEvent/clipboard/SendInput/X11)
│   │   ├── settings/
│   │   │   └── mod.rs          # UserSettings struct, JSON load/save
│   │   ├── stats/
│   │   │   └── mod.rs          # Usage stats (totals, daily, streaks)
│   │   ├── text/
│   │   │   └── mod.rs          # Post-transcription text processing (capitalization)
│   │   ├── transcription/
│   │   │   ├── mod.rs          # Shared types (ModelInfo, CoremlStatus, etc.)
│   │   │   ├── whisper.rs      # Whisper engine (model cache, streaming, download)
│   │   │   ├── cloud.rs        # Cloud STT stubs (OpenAI, AWS, AssemblyAI)
│   │   │   ├── voxtral.rs      # Voxtral engine (feature-gated)
│   │   │   └── voxtral_ffi.rs  # C FFI bindings for voxtral (feature-gated)
│   │   ├── lib.rs              # Tauri app setup, all #[tauri::command] definitions, AppState
│   │   └── main.rs             # Binary entry point (calls lib::run())
│   ├── voxtral/                # Custom C voxtral engine source
│   │   ├── voxtral.c/h         # Core engine
│   │   ├── voxtral_audio.c/h   # Audio processing
│   │   ├── voxtral_decoder.c   # Decoder
│   │   ├── voxtral_encoder.c   # Encoder
│   │   ├── voxtral_kernels.c/h # Compute kernels
│   │   ├── voxtral_metal.h/m   # Metal GPU bridge (macOS)
│   │   ├── voxtral_shaders.metal  # Metal compute shaders
│   │   ├── voxtral_safetensors.c/h  # Model format loader
│   │   ├── voxtral_tokenizer.c/h   # Tokenizer
│   │   ├── voxtral_mic*.c/h    # Mic capture (macOS-specific)
│   │   ├── download_model.sh   # Model download script
│   │   └── LICENSE
│   ├── build.rs                # Build script (tauri_build + voxtral C compilation)
│   ├── Cargo.toml              # Rust dependencies and feature flags
│   └── tauri.conf.json         # Tauri window config, CSP, bundle settings
├── images/                     # Static images (README, marketing)
├── index.html                  # HTML entry point (shared by both windows)
├── package.json                # Frontend dependencies and scripts
├── vite.config.ts              # Vite dev server and build config
├── tailwind.config.js          # Tailwind CSS configuration
├── postcss.config.js           # PostCSS configuration
├── tsconfig.json               # TypeScript config (frontend)
├── tsconfig.node.json          # TypeScript config (Vite/Node tooling)
└── .planning/                  # GSD planning documents
    └── codebase/               # Codebase analysis documents
```

## Directory Purposes

**`src/`:**
- Purpose: Frontend React/TypeScript application
- Contains: Components, state management stores, type definitions, CSS, icons
- Key files: `App.tsx` (root), `main.tsx` (entry), `lib/store.ts` (settings store)

**`src/components/`:**
- Purpose: All React UI components
- Contains: Dictation overlay widget and dashboard page components
- Key file: `DictationBar.tsx` is the primary user-facing overlay

**`src/components/dashboard/`:**
- Purpose: Dashboard window UI (opened from tray icon)
- Contains: Full settings UI, history viewer, dictionary editor, stats home page
- Key file: `Dashboard.tsx` is the shell with sidebar routing

**`src/lib/`:**
- Purpose: Shared utilities, state stores, and Tauri IPC wrappers
- Contains: Zustand stores for each data domain, theme provider, typed invoke wrappers
- Key file: `store.ts` (settings store used by both windows)

**`src/config/`:**
- Purpose: Centralized UI configuration constants
- Contains: `widget.ts` with all timing, sizing, and behavior defaults
- Pattern: Import constants from here rather than using magic numbers in components

**`src/types/`:**
- Purpose: Shared TypeScript interfaces that mirror Rust backend structs
- Contains: `index.ts` with `LocalStats`, `TranscriptionEntry`, `DictionaryEntry`, `DashboardPage`
- Pattern: Keep in sync with Rust serde types in `src-tauri/src/`

**`src-tauri/src/`:**
- Purpose: Rust backend -- all native system operations
- Contains: Module directories for each subsystem, `lib.rs` as the central hub
- Key file: `lib.rs` (1744 lines) contains Tauri setup, all command definitions, platform window management

**`src-tauri/src/api/`:**
- Purpose: MentaFlux cloud API client
- Contains: HTTP calls, auth token management, OS keychain storage

**`src-tauri/src/audio/`:**
- Purpose: Microphone capture and audio processing
- Contains: CPAL stream management, real-time resampling, energy-based VAD

**`src-tauri/src/transcription/`:**
- Purpose: Speech-to-text engines
- Contains: Whisper (primary), cloud stubs, Voxtral (feature-gated)

**`src-tauri/src/injection/`:**
- Purpose: Cross-platform text injection into focused applications
- Contains: macOS (AX/CGEvent/clipboard), Windows (SendInput/clipboard), Linux (X11)

**`src-tauri/voxtral/`:**
- Purpose: Custom C-based speech recognition engine source code
- Contains: Encoder, decoder, tokenizer, Metal GPU shaders, model loader
- Generated: No (hand-written C code)
- Committed: Yes
- Build: Compiled by `build.rs` when `voxtral` feature is enabled

**`src-tauri/gen/`:**
- Purpose: Auto-generated Tauri v2 schema files
- Generated: Yes (by Tauri CLI)
- Committed: Yes
- Do not edit: Regenerated on `tauri build`

**`src-tauri/capabilities/`:**
- Purpose: Tauri v2 permission declarations
- Contains: `default.json` listing allowed window operations, plugin permissions
- Key detail: Both `dictation` and `dashboard` windows share the same capability set

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Frontend React mount point
- `src-tauri/src/main.rs`: Rust binary entry (calls `lib::run()`)
- `src-tauri/src/lib.rs`: Tauri app builder, plugin registration, command handler setup
- `index.html`: Shared HTML entry (both windows load this, routing via hash)

**Configuration:**
- `src-tauri/tauri.conf.json`: Window definitions, CSP, bundle targets, plugin config
- `src-tauri/Cargo.toml`: Rust dependencies, platform-specific deps, feature flags
- `src-tauri/capabilities/default.json`: Tauri v2 permission grants
- `src-tauri/.cargo/config.toml`: CUDA build workaround (Windows)
- `package.json`: Frontend deps, npm scripts
- `vite.config.ts`: Vite dev server (port 1420), build targets
- `tailwind.config.js`: Tailwind CSS setup
- `tsconfig.json` / `tsconfig.node.json`: TypeScript configuration

**Core Logic:**
- `src-tauri/src/lib.rs`: All Tauri commands, AppState, platform overlay setup, tray menu, window management (~1744 lines)
- `src-tauri/src/audio/capture.rs`: Audio capture with real-time resampling (~654 lines)
- `src-tauri/src/transcription/whisper.rs`: Whisper engine, model management, streaming (~1100+ lines)
- `src-tauri/src/injection/mod.rs`: Platform text injection (~300+ lines)
- `src/App.tsx`: Frontend recording state machine, event listeners (~326 lines)
- `src/components/DictationBar.tsx`: Overlay UI with waveform visualization

**Testing:**
- `src-tauri/src/text/mod.rs`: Contains `#[cfg(test)] mod tests` with unit tests for text processing

**User Data (runtime, not in repo):**
- `~/.mentascribe/models/`: Downloaded Whisper/Voxtral model files
- `{config_dir}/mentascribe/settings.json`: User preferences
- `{config_dir}/mentascribe/history.json`: Transcription history
- `{config_dir}/mentascribe/dictionary.json`: Custom dictionary entries
- `{config_dir}/mentascribe/stats.json`: Usage statistics

## Naming Conventions

**Files:**
- React components: PascalCase (e.g., `DictationBar.tsx`, `SettingsPage.tsx`)
- Stores/utilities: camelCase (e.g., `historyStore.ts`, `store.ts`)
- Rust modules: snake_case directories with `mod.rs` (e.g., `audio/mod.rs`, `hotkey/mod.rs`)
- Rust multi-file modules: snake_case (e.g., `capture.rs`, `client.rs`, `whisper.rs`)
- C source: snake_case with `voxtral_` prefix (e.g., `voxtral_encoder.c`)

**Directories:**
- Frontend: camelCase for feature dirs (`dashboard/`), lowercase for category dirs (`lib/`, `types/`, `config/`)
- Backend: snake_case matching Rust module names (`audio/`, `transcription/`, `injection/`)

## Where to Add New Code

**New Tauri Command:**
1. Implement logic in the appropriate module under `src-tauri/src/` (or create new module)
2. Add `#[tauri::command]` function in `src-tauri/src/lib.rs`
3. Register in `tauri::generate_handler![]` array at the bottom of `lib.rs`
4. Add TypeScript wrapper in `src/lib/tauri.ts`
5. Call via `invoke()` from React components

**New Backend Module:**
1. Create `src-tauri/src/{module_name}/mod.rs`
2. Add `mod {module_name};` at top of `src-tauri/src/lib.rs`
3. Expose commands through `lib.rs`

**New Dashboard Page:**
1. Create component in `src/components/dashboard/{PageName}Page.tsx`
2. Add to page enum in `src/types/index.ts` (`DashboardPage` type)
3. Add route case in `src/components/dashboard/Dashboard.tsx` `renderPage()`
4. Add navigation item in `src/components/dashboard/Sidebar.tsx`

**New Frontend Component:**
- Dictation overlay component: `src/components/{ComponentName}.tsx`
- Dashboard component: `src/components/dashboard/{ComponentName}.tsx`

**New Zustand Store:**
- Create `src/lib/{domain}Store.ts` following the pattern in `historyStore.ts`
- Import from components that need it

**New Frontend Type:**
- Add to `src/types/index.ts` (keep in sync with Rust serde structs)

**New Frontend Constants:**
- Add to `src/config/widget.ts` for UI/timing constants

**Platform-Specific Rust Code:**
- Use `#[cfg(target_os = "macos")]` / `#[cfg(target_os = "windows")]` / `#[cfg(target_os = "linux")]` blocks
- For injection: add platform module inside `src-tauri/src/injection/mod.rs`
- For window management: add platform-specific functions in `src-tauri/src/lib.rs`

**Feature-Gated Code:**
- Use `#[cfg(feature = "voxtral")]` for Voxtral-specific code
- Define feature in `src-tauri/Cargo.toml` under `[features]`

## Special Directories

**`src-tauri/target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No (in .gitignore)

**`src-tauri/gen/`:**
- Purpose: Tauri auto-generated schema files
- Generated: Yes (by `tauri build` / `tauri dev`)
- Committed: Yes

**`node_modules/`:**
- Purpose: NPM dependency tree
- Generated: Yes
- Committed: No (in .gitignore)

**`dist/`:**
- Purpose: Vite build output (frontend assets)
- Generated: Yes (by `vite build`)
- Committed: No
- Referenced: `tauri.conf.json` sets `frontendDist: "../dist"`

**`.planning/`:**
- Purpose: GSD codebase analysis and planning documents
- Generated: By analysis tools
- Committed: Yes

**`.claude/`:**
- Purpose: Claude agent configuration, skills, workflows
- Contains: Agent definitions, GSD tooling, hooks
- Committed: Yes

## Build Outputs

**Development:**
- Frontend: Vite dev server at `http://localhost:1420`
- Backend: Cargo incremental build in `src-tauri/target/debug/`
- Command: `npm run tauri dev` (or `cargo tauri dev`)

**Production:**
- Frontend: `dist/` (static assets from `vite build`)
- Backend: `src-tauri/target/release/` (optimized binary with LTO)
- Bundle targets: DMG (macOS), MSI/NSIS (Windows), AppImage/DEB/RPM (Linux)
- Command: `npm run tauri build`

**Build Profile (release):**
- `panic = "abort"`, `codegen-units = 1`, `lto = true`, `opt-level = 3`, `strip = true`

---

*Structure analysis: 2026-04-06*
