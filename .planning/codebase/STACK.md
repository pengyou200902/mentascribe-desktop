# Technology Stack

**Analysis Date:** 2026-04-06

## Languages

**Primary:**
- **TypeScript** (ES2020 target) - Frontend UI, ~5,600 LOC across `src/`
- **Rust** (2021 edition) - Backend/native layer, ~7,000 LOC across `src-tauri/src/`

**Secondary:**
- **C** - Voxtral speech model inference engine, ~11,300 LOC in `src-tauri/voxtral/`
- **Objective-C** - Metal GPU bridge for macOS (`src-tauri/voxtral/voxtral_metal.m`)
- **Metal Shading Language** - GPU compute shaders (`src-tauri/voxtral/voxtral_shaders.metal`, ~1,200 LOC)
- **CSS** - Styling with Tailwind utilities + custom properties (`src/styles/globals.css`, ~770 LOC)

## Runtime

**Environment:**
- Tauri 2.x desktop runtime (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux)
- Rust binary compiled per-platform (no cross-compilation)
- Node.js required for frontend dev server only (not shipped)

**Package Managers:**
- **pnpm** - Frontend (referenced in `src-tauri/tauri.conf.json` build commands)
- **npm** - Lockfile present (`package-lock.json`, 226KB)
- **Cargo** - Rust dependencies (`src-tauri/Cargo.lock` present)

## Frameworks

**Core:**
- **Tauri** 2.x - Desktop app framework, IPC bridge between frontend and Rust backend
- **React** ^18.3.0 - Frontend UI rendering
- **Zustand** ^4.5.0 - Global state management (single store at `src/lib/store.ts`)

**CSS/Styling:**
- **Tailwind CSS** ^3.4.0 - Utility-first CSS with custom design tokens in `tailwind.config.js`
- **PostCSS** ^8.4.0 + **Autoprefixer** ^10.4.0 - CSS processing pipeline
- **clsx** ^2.1.0 + **tailwind-merge** ^2.2.0 - Conditional class name composition

**Build/Dev:**
- **Vite** ^5.2.0 - Frontend bundler and dev server (port 1420)
- **@vitejs/plugin-react** ^4.2.0 - React Fast Refresh + JSX transform
- **TypeScript** ^5.4.0 - Type checking (`strict: true`, `noUnusedLocals`, `noUnusedParameters`)
- **cc** 1.x (Rust build dep) - C/Objective-C compilation via `build.rs`
- **tauri-build** 2.x - Tauri build integration

**Linting/Formatting:**
- **ESLint** ^8.57.0 with `@typescript-eslint/parser` ^7.0.0 and `eslint-plugin-react` ^7.34.0
- **Prettier** ^3.2.0

## Key Dependencies (Rust)

**Speech-to-Text:**
- **whisper-rs** 0.16 - Whisper.cpp bindings for local speech recognition
  - macOS: `coreml` + `metal` features (Apple Neural Engine + GPU acceleration)
  - Windows: `cuda` feature (NVIDIA GPU acceleration)
  - Linux: CPU-only (default)
- **Voxtral C library** (in-tree, `src-tauri/voxtral/`) - Mistral Voxtral Mini 4B model, compiled via `cc` crate when `voxtral` Cargo feature is enabled

**Audio:**
- **cpal** 0.15 - Cross-platform audio capture (microphone input)
- **hound** 3.5 - WAV file encoding/decoding
- **rubato** 0.16 - Audio resampling (device sample rate -> 16kHz for Whisper)

**Text Injection:**
- **enigo** 0.2 - Cross-platform keyboard/mouse simulation
- **arboard** 3 - Cross-platform clipboard access
- **clipboard-win** 5 - Windows clipboard (direct Win32 API)

**Platform APIs (macOS):**
- **cocoa** 0.25 - Objective-C bridge for AppKit (NSPanel, NSPasteboard, NSEvent)
- **objc** 0.2 - Objective-C runtime bindings
- **core-graphics** 0.23 - CGEvent keyboard injection
- **core-foundation** 0.10 - CFString, CFType wrappers
- **accessibility-sys** 0.1 - AXUIElement API for text insertion
- **foreign-types** 0.5 - FFI pointer type safety
- **tauri-nspanel** (git, v2 branch) - Convert Tauri windows to NSPanel for fullscreen overlay

**Platform APIs (Windows):**
- **windows** 0.54 - Win32 API bindings (SendInput, window management, clipboard, process querying)

**Platform APIs (Linux):**
- **x11** 2.21 (with `xtest` feature) - X11 keyboard simulation

**Networking:**
- **reqwest** 0.11 (with `json` feature) - HTTP client for API calls and model downloads
- **tauri-plugin-http** 2 - HTTP from frontend context

**Data/Serialization:**
- **serde** 1 (with `derive`) + **serde_json** 1 - JSON serialization everywhere
- **chrono** 0.4 (with `serde`) - Date/time handling for stats and history
- **uuid** 1 (with `v4`, `serde`) - Unique IDs for history and dictionary entries
- **regex** 1 - Dictionary replacement patterns

**Security:**
- **keyring** 2 - OS keychain storage for auth tokens (macOS Keychain, Windows Credential Manager)

**Async Runtime:**
- **tokio** 1 (full features) - Async runtime for network operations and background tasks

**Hotkey:**
- **global-hotkey** 0.5 - System-wide hotkey registration
- **tauri-plugin-global-shortcut** 2 - Tauri plugin wrapper for global shortcuts

**Error Handling:**
- **thiserror** 1 - Derive macro for error types
- **anyhow** 1 - Flexible error handling

**Logging:**
- **log** 0.4 - Logging facade
- **env_logger** 0.11 - Environment-based log filtering

**Utility:**
- **once_cell** 1.19 - Lazy static initialization
- **lazy_static** 1.5.0 - Static lazy evaluation
- **dirs** 6.0.0 - Platform-appropriate config/home directories
- **libc** 0.2 - C FFI types

## Key Dependencies (Frontend npm)

**Tauri Plugins:**
- `@tauri-apps/api` ^2.0.0 - Core Tauri IPC (`invoke`, `listen`, `emit`)
- `@tauri-apps/plugin-shell` ^2.0.0 - Shell command execution
- `@tauri-apps/plugin-dialog` ^2.0.0 - Native file/message dialogs
- `@tauri-apps/plugin-fs` ^2.0.0 - Filesystem access
- `@tauri-apps/plugin-http` ^2.0.0 - HTTP client from frontend

## Configuration

**TypeScript:**
- `tsconfig.json`: ES2020 target, strict mode, bundler module resolution, `react-jsx` transform
- `tsconfig.node.json`: Separate config for Vite/Node tooling

**Vite:**
- `vite.config.ts`: React plugin, dev server on port 1420, env prefix `VITE_` and `TAURI_`
- Build targets: `es2021`, `chrome100`, `safari13`
- Minification via esbuild (production), sourcemaps in debug

**Tailwind:**
- `tailwind.config.js`: Custom amber/stone color palette, DM Sans + JetBrains Mono fonts, custom animations (glow, float, shimmer, waveform), dark mode via `class` strategy

**Tauri:**
- `src-tauri/tauri.conf.json`: App identifier `ai.mentaflux.voice.mentascribe`, two windows (dictation overlay + dashboard)
- CSP: `default-src 'self'; connect-src https://api.voice.mentaflux.ai`
- Bundle targets: DMG, MSI, NSIS, AppImage, DEB, RPM

**Cargo Features:**
- `default = ["custom-protocol"]` - Tauri custom protocol for production builds
- `voxtral` - Optional Voxtral speech engine (compiles C library via `build.rs`)

**Release Profile:**
- `panic = "abort"`, `codegen-units = 1`, `lto = true`, `opt-level = 3`, `strip = true`

## Build System

**Frontend Pipeline:**
1. `pnpm dev` -> Vite dev server (localhost:1420)
2. `tsc` type-check -> `vite build` -> `dist/` output
3. Tauri bundles `dist/` into native binary

**Rust/Native Pipeline:**
1. `build.rs` runs `tauri_build::build()`
2. If `voxtral` feature enabled: `cc` crate compiles 7 C source files + optional Metal bridge
   - macOS/aarch64: Metal shaders compiled to `.metallib`, embedded as C headers
   - macOS: Links Accelerate, Metal, Foundation frameworks
   - Linux: Links OpenBLAS
   - Windows: CPU-only (no BLAS by default)
3. Cargo compiles Rust with platform-conditional dependencies
4. Tauri packages into installer (DMG/MSI/NSIS/AppImage/DEB/RPM)

**GPU Acceleration Matrix:**
| Platform | Whisper | Voxtral |
|----------|---------|---------|
| macOS (Apple Silicon) | CoreML + Metal | Metal GPU + Accelerate BLAS |
| macOS (Intel) | CoreML (if available) | Accelerate BLAS |
| Windows | CUDA (requires CUDAHOSTCXX workaround for VS 2022) | CPU-only |
| Linux | CPU-only | OpenBLAS |

## Platform Requirements

**Development:**
- Node.js + pnpm (frontend tooling)
- Rust toolchain (stable, 2021 edition)
- Platform SDK: Xcode (macOS), Visual Studio 2022 (Windows), webkit2gtk dev libs (Linux)
- CUDA toolkit (Windows, for GPU-accelerated Whisper)

**Production (macOS):**
- Minimum macOS 10.15 (Catalina)
- Microphone permission (`NSMicrophoneUsageDescription` in `Info.plist`)
- Accessibility permission for text injection via AXUIElement API

**Production (Windows):**
- WebView2 Runtime (bundled or system)
- NVIDIA GPU + CUDA drivers (optional, for Whisper CUDA)

**Production (Linux):**
- X11 display server (Wayland not supported for text injection)
- webkit2gtk runtime

---

*Stack analysis: 2026-04-06*
