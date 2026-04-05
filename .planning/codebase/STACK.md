# Technology Stack

**Analysis Date:** 2026-04-05

## Languages

**Primary:**
- TypeScript 5.4+ - Frontend UI (React components, stores, types) in `src/`
- Rust (Edition 2021) - Backend/core logic in `src-tauri/src/`

**Secondary:**
- C - Custom Voxtral speech-to-text engine in `src-tauri/voxtral/` (feature-gated behind `voxtral` Cargo feature)
- Objective-C - Metal GPU bridge for macOS in `src-tauri/voxtral/voxtral_metal.m`
- Metal Shading Language - GPU compute shaders in `src-tauri/voxtral/voxtral_shaders.metal`
- CSS - Tailwind utilities + custom properties in `src/styles/globals.css`

## Runtime

**Environment:**
- Tauri 2.x - Desktop application framework (Rust backend + system webview frontend)
- System WebView - Renders the React frontend (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux)
- Tokio 1 (full features) - Async runtime for Rust backend

**Package Manager:**
- npm - Frontend dependencies
  - Lockfile: `package-lock.json` present (lockfileVersion 3)
  - **Note:** `src-tauri/tauri.conf.json` references `pnpm build`/`pnpm dev` in beforeBuildCommand/beforeDevCommand, but the actual lockfile is npm-format. Use npm or update `tauri.conf.json`.
- Cargo - Rust dependencies
  - Lockfile: `src-tauri/Cargo.lock` present (lock version 4)

## Frameworks

**Core:**
- Tauri 2 - Desktop app shell, IPC, system tray, window management (`src-tauri/Cargo.toml`)
- React 18.3 - Frontend UI framework (`package.json`)
- Vite 5.2 - Frontend dev server and bundler (`vite.config.ts`)

**State Management:**
- Zustand 4.5 - Lightweight React state stores
  - `src/lib/store.ts` - User settings
  - `src/lib/historyStore.ts` - Transcription history
  - `src/lib/statsStore.ts` - Usage statistics
  - `src/lib/dictionaryStore.ts` - Custom word replacements

**Styling:**
- Tailwind CSS 3.4 - Utility-first CSS (`tailwind.config.js`)
- PostCSS 8.4 + Autoprefixer 10.4 - CSS processing pipeline (`postcss.config.js`)
- clsx 2.1 + tailwind-merge 2.2 - Conditional class composition
- Google Fonts - DM Sans (primary), JetBrains Mono (monospace), loaded via CSS `@import` in `src/styles/globals.css`

**Testing:**
- No frontend test runner configured (no vitest/jest in devDependencies)
- Rust inline `#[cfg(test)]` modules exist (e.g., `src-tauri/src/text/mod.rs`)

**Build/Dev:**
- Vite 5.2 - Frontend bundler with `@vitejs/plugin-react` 4.2
- TypeScript 5.4 - Type checking via `tsconfig.json`
- tauri-build 2 - Rust build integration (`src-tauri/build.rs`)
- cc 1 - C/Obj-C compilation for Voxtral feature (`src-tauri/build.rs`)

## Key Dependencies

### Frontend (npm)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `@tauri-apps/api` | ^2.0.0 | Tauri IPC bridge — `invoke()` commands, `listen()` events |
| `@tauri-apps/plugin-shell` | ^2.0.0 | Open URLs in system browser |
| `@tauri-apps/plugin-dialog` | ^2.0.0 | Native file/dialog pickers |
| `@tauri-apps/plugin-fs` | ^2.0.0 | Filesystem access from frontend |
| `@tauri-apps/plugin-http` | ^2.0.0 | HTTP requests from frontend |
| `react` | ^18.3.0 | UI component framework |
| `react-dom` | ^18.3.0 | React DOM renderer |
| `zustand` | ^4.5.0 | Lightweight state management (4 stores) |
| `clsx` | ^2.1.0 | Conditional CSS class name composition |
| `tailwind-merge` | ^2.2.0 | Merge Tailwind classes without conflicts |

### Backend (Cargo) — Core

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tauri` | 2 | App framework (`tray-icon`, `macos-private-api` features) |
| `tauri-plugin-shell` | 2 | Shell/URL opening |
| `tauri-plugin-dialog` | 2 | Native dialogs |
| `tauri-plugin-fs` | 2 | Filesystem access |
| `tauri-plugin-http` | 2 | HTTP requests |
| `tauri-plugin-global-shortcut` | 2 | System-wide hotkey registration |
| `serde` + `serde_json` | 1 | JSON serialization/deserialization |
| `tokio` | 1 (full) | Async runtime |
| `reqwest` | 0.11 (json) | HTTP client for MentaFlux API |
| `log` + `env_logger` | 0.4 / 0.11 | Logging |
| `thiserror` | 1 | Ergonomic error types |
| `anyhow` | 1 | Flexible error handling |
| `chrono` | 0.4 (serde) | Date/time for stats and history |
| `uuid` | 1 (v4, serde) | Unique ID generation |
| `regex` | 1 | Text pattern matching |
| `dirs` | 6.0 | Cross-platform config/home directory resolution |
| `once_cell` | 1.19 | Lazy static initialization |
| `lazy_static` | 1.5 | Legacy lazy statics (audio module) |
| `libc` | 0.2 | C library bindings |

### Backend (Cargo) — Audio & Speech-to-Text

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `whisper-rs` | 0.15 | Local STT via whisper.cpp (CoreML + Metal on macOS) |
| `cpal` | 0.15 | Cross-platform audio capture from microphone |
| `hound` | 3.5 | WAV audio encoding/decoding |
| `rubato` | 0.16 | Real-time audio resampling (native rate -> 16kHz mono for Whisper) |

### Backend (Cargo) — Text Injection & System Integration

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `enigo` | 0.2 | Cross-platform keyboard simulation |
| `arboard` | 3 | Cross-platform clipboard access |
| `global-hotkey` | 0.5 | System-wide hotkey detection |
| `keyring` | 2 | OS keychain for secure token storage |

### Platform-Specific (macOS)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `core-graphics` | 0.23 | CGEvent keyboard simulation, monitor detection |
| `core-foundation` | 0.10 | macOS Foundation types |
| `cocoa` | 0.25 | NSWindow/NSPanel APIs |
| `objc` | 0.2 | Objective-C runtime bridging |
| `foreign-types` | 0.5 | FFI type safety for CGEvent |
| `accessibility-sys` | 0.1 | Accessibility permission checks |
| `tauri-nspanel` | git (v2 branch) | Convert Tauri window to NSPanel for fullscreen overlay |

### Platform-Specific (Windows)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `windows` | 0.54 | Win32 API (keyboard input, clipboard, data exchange, memory) |
| `clipboard-win` | 5 | Windows clipboard access |

### Platform-Specific (Linux)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `x11` | 2.21 | X11 display server bindings (xtest for input simulation) |

## Development Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `@tauri-apps/cli` | ^2.0.0 | Tauri CLI (`tauri dev`, `tauri build`) |
| `@types/react` | ^18.3.0 | React type definitions |
| `@types/react-dom` | ^18.3.0 | React DOM type definitions |
| `@typescript-eslint/eslint-plugin` | ^7.0.0 | TypeScript-aware ESLint rules |
| `@typescript-eslint/parser` | ^7.0.0 | TypeScript ESLint parser |
| `@vitejs/plugin-react` | ^4.2.0 | Vite React fast-refresh plugin |
| `autoprefixer` | ^10.4.0 | CSS vendor prefix automation |
| `eslint` | ^8.57.0 | JavaScript/TypeScript linter |
| `eslint-plugin-react` | ^7.34.0 | React-specific ESLint rules |
| `eslint-plugin-react-hooks` | ^4.6.0 | React hooks linting |
| `postcss` | ^8.4.0 | CSS transformation pipeline |
| `prettier` | ^3.2.0 | Code formatter |
| `tailwindcss` | ^3.4.0 | Utility-first CSS framework |
| `typescript` | ^5.4.0 | TypeScript compiler |
| `vite` | ^5.2.0 | Frontend build tool |

## Configuration

**TypeScript (`tsconfig.json`):**
- Target: ES2020 with bundler module resolution
- Strict mode enabled with `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`
- JSX: react-jsx (automatic runtime)
- `isolatedModules: true` for Vite compatibility

**Vite (`vite.config.ts`):**
- Dev server: port 1420 (strict port)
- Build target: es2021, chrome100, safari13
- Env prefix: `VITE_` and `TAURI_`
- Minification: esbuild in production, disabled in TAURI_DEBUG
- Sourcemaps: enabled in TAURI_DEBUG only

**Tailwind (`tailwind.config.js`):**
- Dark mode: class-based toggling
- Custom color palette: amber (brand accent), stone (warm neutrals)
- Custom fonts: DM Sans (sans), JetBrains Mono (mono)
- Custom animations: pulse-slow, glow, float, shimmer, fade-in, slide-in, scale-in, waveform
- Custom shadows: glow-amber, inner-glow, card, card-dark variants

**Rust Release Profile (`src-tauri/Cargo.toml`):**
- `panic = "abort"` - No unwinding overhead
- `codegen-units = 1` - Maximum optimization
- `lto = true` - Full link-time optimization
- `opt-level = 3` - Maximum speed
- `strip = true` - Remove debug symbols

## Cargo Features

| Feature | Default | Purpose |
|---------|---------|---------|
| `custom-protocol` | Yes | Tauri custom protocol for production builds |
| `voxtral` | No | Enables Voxtral Mini 4B engine — C library FFI, Metal GPU support, ~8.9GB model |

## Scripts

```bash
npm run dev          # Start Vite dev server (port 1420)
npm run build        # tsc + vite build (production frontend)
npm run preview      # Preview production build
npm run tauri        # Tauri CLI passthrough (e.g., npm run tauri dev)
npm run lint         # ESLint on src/**/*.{ts,tsx}
npm run lint:fix     # ESLint with autofix
npm run format       # Prettier on src/**/*.{ts,tsx,css}
npm run typecheck    # TypeScript type check (--noEmit)
```

## Platform Requirements

**Development:**
- Node.js (no pinned version; no `.nvmrc`)
- Rust toolchain (edition 2021; no `rust-toolchain.toml`)
- macOS: Xcode command-line tools (for CoreGraphics, Metal SDK, whisper.cpp CoreML)
- Windows: Visual Studio Build Tools (C++ workload)
- Linux: X11 dev headers, OpenBLAS (for `voxtral` feature)
- For `voxtral` feature: C compiler via `cc` crate, Metal SDK on macOS/aarch64

**Production Targets:**
- macOS: minimum 10.15 (Catalina) — DMG bundle
- Windows: MSI and NSIS installers
- Linux: AppImage, DEB, RPM packages

**macOS Entitlements:**
- Microphone access required (`src-tauri/Info.plist`: `NSMicrophoneUsageDescription`)
- Accessibility permission required at runtime for text injection
- macOS private API enabled (`macOSPrivateApi: true` in `tauri.conf.json`) for NSPanel fullscreen overlay

---

*Stack analysis: 2026-04-05*
