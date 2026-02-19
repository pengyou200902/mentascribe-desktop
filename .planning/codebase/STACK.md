# Technology Stack

**Analysis Date:** 2026-02-19

## Languages

**Primary:**
- TypeScript 5.4 - Frontend React components and Tauri API bindings
- Rust 2021 edition - Tauri v2 backend (audio, transcription, system integration)

**Secondary:**
- JavaScript/JSX - Vite configuration
- CSS - Tailwind CSS styling

## Runtime

**Environment:**
- Tauri v2 - Cross-platform desktop framework (Rust backend + web frontend)
- Node.js - Development build system (Vite)

**Package Manager:**
- pnpm - Node.js packages
- Cargo - Rust dependencies

## Frameworks

**Core:**
- React 18.3 - Frontend UI library
- Tauri 2 - Desktop application framework with macOS-private API support
- Vite 5.2 - Frontend build tool and dev server (port 1420)

**Styling:**
- Tailwind CSS 3.4 - Utility-first CSS framework
- PostCSS 8.4 - CSS transformations (autoprefixer)

**State Management:**
- Zustand 4.5 - Minimal React state management for settings and app state

**Testing:**
- No testing framework detected in dependencies

**Build/Dev:**
- @tauri-apps/cli 2 - Tauri CLI for building and development
- TypeScript 5.4 - Type checking
- ESLint 8.57 - Linting
- Prettier 3.2 - Code formatting
- @vitejs/plugin-react 4.2 - Vite React plugin

## Key Dependencies

**Critical:**
- `@tauri-apps/api` 2.0 - Tauri runtime API for IPC with Rust backend
- `whisper-rs` 0.15 (with CoreML feature for macOS) - Local speech-to-text using Whisper model
- `reqwest` 0.11 - HTTP client for API calls
- `tokio` 1 - Async runtime

**Audio Processing:**
- `cpal` 0.15 - Cross-platform audio capture
- `hound` 3.5 - WAV file format reading/writing
- `rubato` 0.16 - Audio resampling

**System Integration:**
- `tauri-nspanel` (git: ahkohd/tauri-nspanel, branch: v2) - macOS NSPanel for fullscreen overlay support
- `enigo` 0.2 - Cross-platform text injection (keyboard simulation)
- `arboard` 3 - Cross-platform clipboard access
- `global-hotkey` 0.5 - Global hotkey registration
- `keyring` 2 - OS keychain for secure token storage

**Platform-Specific:**
- macOS: `core-graphics` 0.23, `cocoa` 0.25, `objc` 0.2 - macOS native APIs for window management and graphics
- Windows: `windows` 0.54 - Windows API bindings
- Linux: `x11` 2.21 - X11 window system

**Tauri Plugins:**
- `tauri-plugin-shell` 2 - Shell command execution
- `tauri-plugin-dialog` 2 - Native file dialogs
- `tauri-plugin-fs` 2 - File system access
- `tauri-plugin-http` 2 - HTTP requests from Rust backend
- `tauri-plugin-global-shortcut` 2 - Global keyboard shortcuts

**Serialization & Utilities:**
- `serde` 1 - Serialization framework
- `serde_json` 1 - JSON serialization
- `uuid` 1 - UUID generation with v4 and serde support
- `chrono` 0.4 - Date/time handling with serde support
- `regex` 1 - Regular expressions
- `log` 0.4 - Logging facade
- `env_logger` 0.11 - Logging implementation
- `thiserror` 1 - Error handling macros
- `anyhow` 1 - Flexible error handling
- `lazy_static` 1.5 - Static initialization
- `once_cell` 1.19 - One-time initialization
- `dirs` 6.0 - Platform-specific directory paths

## Configuration

**Environment:**
- No .env files present - Configuration via Tauri settings system in user config directory
- Settings path: `~/.config/mentascribe/settings.json` (Linux/macOS) or platform default
- Tauri env prefix: `VITE_` and `TAURI_`

**Build:**
- `vite.config.ts` - Vite configuration with React plugin
- `tsconfig.json` - TypeScript compiler options (strict mode enabled, ES2020 target)
- `tailwind.config.js` - Tailwind CSS configuration
- `postcss.config.js` - PostCSS configuration
- `src-tauri/Cargo.toml` - Rust dependencies and features

**Feature Flags:**
- Tauri: `custom-protocol` (default) - Custom protocol for frontend/backend communication
- Tauri macOS: `macos-private-api` enabled - Required for NSPanel fullscreen overlay
- Whisper: `coreml` feature on macOS - Apple Neural Engine acceleration

## Platform Requirements

**Development:**
- Node.js (with pnpm)
- Rust toolchain
- macOS minimum: 10.15 (Catalina)
- Requires Xcode for macOS builds

**Production:**
- Deployment targets:
  - macOS: DMG bundle format
  - Windows: MSI and NSIS installers
  - Linux: AppImage, deb, rpm packages
- Tauri 2 provides single-code-base deployment across platforms

**Performance Optimization (Release Build):**
- Panic: abort (no stack unwinding)
- LTO: enabled (link-time optimization)
- Optimization level: s (size over speed)
- Strip symbols: yes
- Codegen units: 1

---

*Stack analysis: 2026-02-19*
