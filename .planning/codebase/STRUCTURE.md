# Codebase Structure

**Analysis Date:** 2026-02-18

## Directory Layout

```
mentascribe-desktop/
├── src/                        # React/TypeScript frontend
│   ├── App.tsx                 # Main app router (window type detection)
│   ├── main.tsx                # React root entry point
│   ├── components/             # React components
│   │   ├── DictationBar.tsx    # Floating overlay widget with waveform
│   │   ├── Settings.tsx        # Settings form component
│   │   ├── History.tsx         # Quick history viewer
│   │   ├── dashboard/          # Full-featured dashboard (multi-page)
│   │   │   ├── Dashboard.tsx
│   │   │   ├── HomePage.tsx
│   │   │   ├── HistoryPage.tsx
│   │   │   ├── DictionaryPage.tsx
│   │   │   ├── SettingsPage.tsx
│   │   │   └── Sidebar.tsx
│   │   └── [MenuBar, TranscriptionOverlay]
│   ├── lib/                    # Zustand stores and utilities
│   │   ├── store.ts            # Main settings store
│   │   ├── historyStore.ts     # History pagination store
│   │   ├── dictionaryStore.ts  # Dictionary store
│   │   ├── statsStore.ts       # Stats store
│   │   ├── tauri.ts            # Tauri API wrappers
│   │   └── theme.tsx           # Theme context/utilities
│   ├── hooks/                  # Custom React hooks
│   ├── types/                  # TypeScript interfaces
│   │   └── index.ts
│   ├── icons/                  # Icon components (SVG)
│   └── styles/                 # CSS/Tailwind globals
│
├── src-tauri/                  # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── main.rs             # Binary entry point (minimal)
│   │   ├── lib.rs              # App initialization, commands, main loop
│   │   ├── audio/              # Audio capture and processing
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs      # Cross-platform audio capture (cpal)
│   │   │   └── vad.rs          # Voice activity detection
│   │   ├── transcription/       # Speech-to-text providers
│   │   │   ├── mod.rs
│   │   │   ├── whisper.rs      # Local Whisper (whisper-rs crate)
│   │   │   └── cloud.rs        # Cloud provider adapters
│   │   ├── injection/          # Text injection into active app
│   │   │   └── mod.rs          # Platform-specific implementations
│   │   ├── hotkey/             # Global hotkey registration
│   │   │   └── mod.rs
│   │   ├── settings/           # Settings management and persistence
│   │   │   └── mod.rs
│   │   ├── history/            # Transcription history storage
│   │   │   └── mod.rs
│   │   ├── dictionary/         # Custom word replacements
│   │   │   └── mod.rs
│   │   ├── text/               # Text post-processing
│   │   │   └── mod.rs
│   │   ├── stats/              # Usage statistics
│   │   │   └── mod.rs
│   │   └── api/                # External API integration
│   │       ├── mod.rs
│   │       └── client.rs
│   │
│   ├── Cargo.toml              # Rust dependencies and build config
│   ├── tauri.conf.json         # Tauri app configuration
│   ├── build.rs                # Tauri build script
│   ├── Info.plist              # macOS app metadata
│   ├── capabilities/           # Tauri security capabilities
│   ├── gen/                    # Generated schemas
│   ├── icons/                  # App icons (PNG, ICNS, ICO)
│   └── target/                 # Build output (excluded from repo)
│
├── dist/                       # Built frontend (generated)
├── node_modules/               # npm dependencies
├── package.json                # Frontend dependencies
├── tsconfig.json               # TypeScript config
├── tsconfig.node.json          # TypeScript config for Node tools
├── vite.config.ts              # Vite bundler config
├── tailwind.config.js          # Tailwind CSS config
├── postcss.config.js           # PostCSS config
├── .planning/                  # GSD planning documentation
└── .gitignore
```

## Directory Purposes

**`src/`:**
- Purpose: React/TypeScript frontend application
- Contains: Components (React), stores (Zustand), utilities, styles
- Key files: `App.tsx` (main router), `components/` (all UI)

**`src/components/`:**
- Purpose: Reusable and page-level React components
- Key:
  - `DictationBar.tsx`: Floating overlay widget with waveform animation
  - `Settings.tsx`: Quick settings panel
  - `dashboard/`: Full feature dashboard with tabs

**`src/lib/`:**
- Purpose: Zustand state management and Tauri API wrappers
- Key:
  - `store.ts`: Settings state (useStore hook)
  - `historyStore.ts`: Paginated history state
  - `tauri.ts`: Typed wrappers for Tauri invoke calls

**`src-tauri/src/`:**
- Purpose: Rust backend implementation
- Structure: Module per feature (audio, transcription, hotkey, injection, etc.)

**`src-tauri/src/audio/`:**
- Purpose: Cross-platform audio capture and preprocessing
- Key: `capture.rs` (manages audio stream), `vad.rs` (voice detection)

**`src-tauri/src/transcription/`:**
- Purpose: Speech-to-text model management and inference
- Key: `whisper.rs` (local Whisper model), `cloud.rs` (cloud provider fallback)

**`src-tauri/src/injection/`:**
- Purpose: Platform-specific text injection
- Key: Platform gates (macOS CoreGraphics, Windows Win32, Linux X11)

**`src-tauri/src/hotkey/`:**
- Purpose: Global hotkey registration and event emission
- Pattern: Uses `tauri-plugin-global-shortcut`, supports F1-F12 keys

**`src-tauri/src/settings/`:**
- Purpose: Settings data structures and JSON persistence
- Pattern: Serde serialization to `~/.config/mentascribe/settings.json`

## Key File Locations

**Entry Points:**
- Frontend: `/src/main.tsx` (React mount)
- Frontend App: `/src/App.tsx` (window type detection and routing)
- Backend: `/src-tauri/src/main.rs` (binary entry)
- Backend Init: `/src-tauri/src/lib.rs` (Tauri app setup and commands)

**Configuration:**
- Tauri app config: `/src-tauri/tauri.conf.json` (windows, plugins, security)
- TypeScript: `/tsconfig.json`
- Build: `/vite.config.ts`

**Core Logic:**
- Recording loop: `/src-tauri/src/lib.rs` (`start_recording`, `stop_recording` commands)
- Text injection: `/src-tauri/src/injection/mod.rs` (platform-specific)
- Settings sync: `/src-tauri/src/settings/mod.rs` (persistence)
- History storage: `/src-tauri/src/history/mod.rs` (JSON CRUD)

**Testing:**
- Tests co-located with source files (standard Rust pattern)
- Frontend tests: Component tests in `src/` (if present)
- No dedicated test directory structure yet

## Naming Conventions

**Files:**
- Component files: PascalCase (e.g., `DictationBar.tsx`, `Settings.tsx`)
- Store files: camelCase (e.g., `historyStore.ts`, `dictionaryStore.ts`)
- Module files: snake_case (e.g., `audio/capture.rs`, `src/injection/mod.rs`)
- Type files: camelCase or PascalCase based on export (e.g., `types/index.ts`)

**Directories:**
- Feature modules: lowercase plural (e.g., `components/`, `hooks/`, `lib/`)
- Dashboard subfeatures: Pascal case (e.g., `components/dashboard/HistoryPage.tsx`)
- Rust modules: lowercase (e.g., `src-tauri/src/audio/`, `src-tauri/src/transcription/`)

**Functions:**
- React components: PascalCase (e.g., `DictationBar`, `SettingsPage`)
- Zustand hooks: prefix `use` (e.g., `useStore`, `useHistoryStore`)
- Tauri commands: snake_case (e.g., `start_recording`, `inject_text`)
- Rust functions: snake_case (e.g., `transcribe()`, `inject_text()`)

**Variables:**
- Frontend: camelCase (e.g., `isRecording`, `audioLevel`)
- Rust: snake_case (e.g., `is_recording`, `audio_level`)

**Types:**
- TypeScript interfaces: PascalCase (e.g., `UserSettings`, `TranscriptionEntry`)
- Rust structs: PascalCase (e.g., `UserSettings`, `AudioData`)
- Enums: PascalCase with variants (e.g., `HotkeyError::UnknownKey`)

## Where to Add New Code

**New Feature:**
- Primary code: `/src-tauri/src/[feature]/mod.rs` for backend logic
- Frontend: `/src/components/[Feature].tsx` for UI or `/src/lib/[feature]Store.ts` for state
- Command wrapper: Add function to `/src/lib/tauri.ts`

**New Component/Module:**
- Page component: `/src/components/dashboard/[PageName].tsx`
- Utility component: `/src/components/[ComponentName].tsx`
- Shared hook: `/src/hooks/[useName].ts`

**Utilities:**
- Shared helpers: `/src/lib/[name].ts` (frontend) or `/src-tauri/src/[module]/mod.rs` (backend)
- Type definitions: `/src/types/index.ts`
- Constants/config: `/src/lib/` or `/src-tauri/src/settings/`

**Styles:**
- Global styles: `/src/styles/globals.css`
- Component styles: Inline via Tailwind classes (preferred) or component-scoped CSS

## Special Directories

**`/.planning/codebase/`:**
- Purpose: GSD codebase analysis documents
- Generated: Yes (by GSD agents)
- Committed: Yes

**`/dist/`:**
- Purpose: Built frontend output
- Generated: Yes (by `npm run build`)
- Committed: No

**`/node_modules/`:**
- Purpose: npm package cache
- Generated: Yes (by `npm install`)
- Committed: No

**`/src-tauri/target/`:**
- Purpose: Rust build output and cache
- Generated: Yes (by `cargo build`)
- Committed: No

**`/src-tauri/gen/`:**
- Purpose: Generated Tauri type definitions and schemas
- Generated: Yes (by Tauri CLI)
- Committed: Yes (for team synchronization)

## Build and Development Workflow

**Frontend Build:**
```bash
pnpm build     # Runs `tsc && vite build` → outputs to /dist
pnpm dev       # Runs vite dev server on port 1420
```

**Backend Build:**
```bash
cargo build --manifest-path src-tauri/Cargo.toml    # Dev build
cargo build --release --manifest-path src-tauri/Cargo.toml  # Optimized
```

**Desktop App:**
```bash
pnpm tauri build      # Full production build
pnpm tauri dev        # Dev mode (watches frontend and rebuilds Rust)
```

**Frontend served to Tauri:**
- Vite dev server runs on port 1420 during dev
- Built dist directory embedded during production
- Window URLs point to `index.html#[route]` for routing

---

*Structure analysis: 2026-02-18*
