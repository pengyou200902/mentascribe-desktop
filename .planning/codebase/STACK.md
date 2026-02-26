# Technology Stack

**Analysis Date:** 2026-02-26

## Languages

**Primary:**
- **Rust** — Tauri backend, system-level operations, audio processing, speech-to-text models
  - Edition: 2021
  - Core modules: `src-tauri/src/lib.rs` and submodules
- **TypeScript** — React frontend, UI and state management
  - Version: ^5.4.0
  - Target: ES2020
  - Strict mode enabled

**Secondary:**
- **JavaScript** — Configuration files
- **CSS** — Styling via Tailwind

## Runtime

**Environment:**
- Tauri 2.0 (desktop app framework)
  - Runs on macOS, Windows, Linux
  - Uses native window APIs for fullscreen overlay support

**Package Managers:**
- **Node.js/pnpm** (frontend)
  - Lock file: `package-lock.json` present
- **Cargo** (Rust backend)
  - Lock file: `Cargo.lock` managed by Cargo

## Frameworks

**Core:**
- **Tauri v2** (^2.0.0) — Desktop framework
  - Features: `tray-icon`, `macos-private-api`
  - Entry point: `src-tauri/src/lib.rs`
  - Frontend config: `src-tauri/tauri.conf.json`

- **React** (^18.3.0) — UI framework
  - Entry point: `src/main.tsx`
  - Root component: `src/App.tsx`

- **Zustand** (^4.5.0) — State management
  - Store: `src/lib/store.ts` (user settings)
  - Stores: `src/lib/historyStore.ts`, `src/lib/dictionaryStore.ts`, `src/lib/statsStore.ts`

**UI & Styling:**
- **Tailwind CSS** (^3.4.0) — Utility-first CSS
  - Config: `tailwind.config.js`
  - PostCSS integration: `postcss.config.js`
- **clsx** (^2.1.0) — Class name utility
- **tailwind-merge** (^2.2.0) — Merge Tailwind classes

**Build & Dev:**
- **Vite** (^5.2.0) — Frontend build tool
  - Config: `vite.config.ts`
  - Port: 1420 (dev server)
  - React plugin: `@vitejs/plugin-react` (^4.2.0)
- **TypeScript Compiler** (^5.4.0) — Type checking and compilation
  - Config: `tsconfig.json`

**Testing & Linting:**
- **ESLint** (^8.57.0) — Code linting
  - Parser: `@typescript-eslint/parser` (^7.0.0)
  - Plugin: `@typescript-eslint/eslint-plugin` (^7.0.0)
  - React plugins: `eslint-plugin-react` (^7.34.0), `eslint-plugin-react-hooks` (^4.6.0)
- **Prettier** (^3.2.0) — Code formatting
- **Autoprefixer** (^10.4.0) — CSS vendor prefixing

## Key Dependencies

**Critical - Audio & Speech-to-Text:**
- **whisper-rs** (0.15) — Whisper.cpp Rust bindings for local speech-to-text
  - Features: `coreml`, `metal` (macOS GPU acceleration)
  - Models downloaded from: https://huggingface.co/ggerganov/whisper.cpp
  - Cache location: `~/.mentascribe/models/`

- **cpal** (0.15) — Cross-platform audio capture
  - Used in: `src-tauri/src/audio/capture.rs`

- **hound** (3.5) — WAV audio encoding/decoding
  - Used in: `src-tauri/src/transcription/cloud.rs` (audio format conversion)

- **rubato** (0.16) — Audio resampling
  - Used in: Audio processing pipeline

**Critical - Text Injection & Interaction:**
- **enigo** (0.2) — Cross-platform text injection (simulate keyboard input)
  - Used in: `src-tauri/src/injection/mod.rs`

- **arboard** (3) — Cross-platform clipboard access
  - Used in: Text output methods

**Critical - System Integration:**
- **global-hotkey** (0.5) — System-wide hotkey registration
  - Used in: `src-tauri/src/hotkey/mod.rs`
  - Supports F5, F6, and other hotkeys across platforms

- **keyring** (2) — OS keychain/credential storage
  - Used in: `src-tauri/src/api/client.rs` for secure token storage
  - macOS: uses Keychain
  - Windows: uses Credential Manager
  - Linux: uses Secret Service

**Critical - Tauri Plugins:**
- **tauri-plugin-shell** (2) — Shell command execution
- **tauri-plugin-dialog** (2) — Native file dialogs
- **tauri-plugin-fs** (2) — File system operations
- **tauri-plugin-http** (2) — HTTP client for API calls
- **tauri-plugin-global-shortcut** (2) — Global shortcut registration

**macOS Specific:**
- **tauri-nspanel** (git: https://github.com/ahkohd/tauri-nspanel, branch: v2)
  - Converts Tauri window to NSPanel for fullscreen overlay capability
  - Used in: `src-tauri/src/lib.rs` function `setup_dictation_panel()`
  - Cocoa re-exports: `tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior`

- **core-graphics** (0.23) — macOS display coordinate handling
- **core-foundation** (0.10) — macOS foundation types
- **cocoa** (0.25) — macOS Cocoa bindings
- **objc** (0.2) — Objective-C runtime bindings
- **accessibility-sys** (0.1) — macOS accessibility APIs

**Windows Specific:**
- **windows** (0.54) — Windows API bindings
  - Features: keyboard/mouse input, data exchange, memory, Foundation
- **clipboard-win** (5) — Windows clipboard access

**Linux Specific:**
- **x11** (2.21) — X11 display server bindings
  - Features: xtest for input simulation

**Infrastructure & Utilities:**
- **reqwest** (0.11) — Async HTTP client
  - Features: JSON serialization
  - Used in: `src-tauri/src/api/client.rs` for API calls

- **tokio** (1) — Async runtime (full features enabled)

- **serde** (1) — Serialization/deserialization
  - Features: derive macros

- **serde_json** (1) — JSON serialization

- **thiserror** (1) — Error type derivation
- **anyhow** (1) — Flexible error handling

- **chrono** (0.4) — Date/time handling
  - Features: serde support
  - Used in: Dashboard timestamps, stats

- **uuid** (1) — UUID generation
  - Features: v4 generation, serde support

- **log** (0.4) & **env_logger** (0.11) — Structured logging

- **lazy_static** (1.5.0) & **once_cell** (1.19) — Static initialization

- **dirs** (6.0.0) — Platform-independent directory paths
  - Used in: Settings path resolution, model cache locations

- **regex** (1) — Regular expression matching
- **libc** (0.2) — C library bindings
- **foreign-types** (0.5) — Safe wrappers for foreign types
- **cc** (1) — C compilation for build script

## Configuration Files

**Frontend:**
- `tsconfig.json` — TypeScript compiler options (ES2020 target, strict mode)
- `vite.config.ts` — Vite build configuration
- `tailwind.config.js` — Tailwind CSS utilities
- `postcss.config.js` — PostCSS plugins (autoprefixer)
- `package.json` — Node.js dependencies and scripts

**Backend:**
- `src-tauri/Cargo.toml` — Rust dependencies and features
  - Build dependencies: `tauri-build`, `cc`
  - Default features: `custom-protocol`
  - Optional feature: `voxtral` (alternative speech-to-text engine)
- `src-tauri/build.rs` — Build script (Tauri setup)

**Desktop App:**
- `src-tauri/tauri.conf.json` — Tauri configuration
  - App name: `MentaScribe`
  - Version: 1.0.0
  - Bundle targets: dmg, msi, nsis, appimage, deb, rpm
  - macOS minimum: 10.15
  - Security: CSP allows https://api.voice.mentaflux.ai

**Development:**
- `.gitignore` — Git ignore rules
- License: `LICENSE` (Apache-2.0)

## Environment Configuration

**Build & Runtime:**
- Dev environment: `pnpm dev` starts Vite on port 1420
- Build command: `pnpm build` (TypeScript compilation + Vite bundling)
- Tauri build: `pnpm tauri build`

**No .env File Requirements Detected:**
The project does not use .env files in the repository root. Environment-specific configs are stored via:
- OS Keychain: API tokens (via `keyring` crate)
- Disk config: `~/.config/mentascribe/settings.json` (user settings)
- Zustand store: In-memory frontend state

## Platform Requirements

**Development:**
- Node.js (for pnpm/npm)
- Rust toolchain (cargo, rustc)
- macOS 10.15+ for native API features

**Production:**
- macOS 10.15+ (minimum)
- Windows (MSI/NSIS installer)
- Linux (AppImage, deb, rpm packages)

---

*Stack analysis: 2026-02-26*
