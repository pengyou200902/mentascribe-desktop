# Technology Stack

**Analysis Date:** 2026-02-24

## Languages

**Primary:**
- TypeScript 5.4 - React frontend components and build scripts
- Rust 2021 edition - Tauri backend, audio processing, transcription

**Supporting:**
- JavaScript (Vite config, PostCSS config)
- CSS (Tailwind utilities, PostCSS processing)

## Runtime

**Environment:**
- Node.js (frontend development and build) - specified via npm scripts
- Tauri v2 runtime - Desktop application host

**Package Manager:**
- npm (with npm-lock.json)
- Cargo (Rust dependency management)

## Frameworks

**Core Frontend:**
- React 18.3.0 - UI framework
- Vite 5.2.0 - Development server and build tool
- Tauri 2.0 - Desktop application framework with macOS private API support

**Core Backend:**
- Tauri Plugins (v2):
  - `tauri-plugin-shell` - Process execution
  - `tauri-plugin-dialog` - File dialogs
  - `tauri-plugin-fs` - File system access
  - `tauri-plugin-http` - HTTP requests
  - `tauri-plugin-global-shortcut` - Global hotkey registration

**Specialized Plugins:**
- `tauri-nspanel` (v2, git branch) - macOS NSPanel overlay support for fullscreen overlay
- `accessibility-sys` - macOS accessibility APIs

**Styling:**
- Tailwind CSS 3.4.0 - Utility-first CSS framework
- PostCSS 8.4.0 - CSS processing pipeline
- Autoprefixer 10.4.0 - Browser prefix automation

## Key Dependencies

**Critical Audio/Transcription:**
- `cpal` 0.15 - Cross-platform audio capture
- `hound` 3.5 - WAV file encoding/decoding
- `whisper-rs` 0.15 - OpenAI Whisper speech-to-text (with CoreML/Metal on macOS)
- `rubato` 0.16 - Audio resampling
- `global-hotkey` 0.5 - Global keyboard hotkey handling

**Text Input/Clipboard:**
- `enigo` 0.2 - Text injection/keyboard emulation
- `arboard` 3 - Cross-platform clipboard access

**Platform-Specific Audio:**
- macOS: CoreML and Metal GPU acceleration built into `whisper-rs`
- Windows: `windows` 0.54 crate with Win32 keyboard/clipboard features, `clipboard-win` 5
- Linux: `x11` 2.21 with xtest for input simulation

**Data & Serialization:**
- `serde` 1.0 - Serialization framework
- `serde_json` 1.0 - JSON serialization
- `chrono` 0.4 - Date/time handling (with serde support)
- `uuid` 1.0 - UUID generation (v4, with serde)

**Networking & API:**
- `reqwest` 0.11 - HTTP client (with JSON support)
- `tokio` 1.0 - Async runtime (full features enabled)

**Security & Storage:**
- `keyring` 2.0 - OS keychain integration (auth token storage)

**Logging & Error Handling:**
- `log` 0.4 - Logging facade
- `env_logger` 0.11 - Environment-based logging configuration
- `thiserror` 1.0 - Error type derivation
- `anyhow` 1.0 - Error handling utilities

**Utilities:**
- `lazy_static` 1.5.0 - Lazy static initialization
- `once_cell` 1.19 - One-time initialization cells
- `dirs` 6.0.0 - Platform-specific config directory resolution
- `regex` 1.0 - Text pattern matching
- `libc` 0.2 - System C library bindings

**Frontend Utilities:**
- `zustand` 4.5.0 - State management store
- `clsx` 2.1.0 - Classname utility
- `tailwind-merge` 2.2.0 - Tailwind CSS conflict resolution

**Dev Tools:**
- TypeScript `5.4` - Type checking
- ESLint 8.57.0 with plugins (`@typescript-eslint`, `react`, `react-hooks`)
- Prettier 3.2.0 - Code formatting
- Vite React plugin 4.2.0 - JSX transformation
- Tauri CLI 2.0.0 - Desktop build orchestration

## Configuration

**Environment:**
- Settings loaded from platform config directory: `~/.config/mentascribe/settings.json`
- Settings persisted via Tauri invoke to Rust AppState
- Auth tokens stored securely in OS keychain via `keyring` crate

**Build:**
- `vite.config.ts` - Vite frontend build configuration
- `tsconfig.json` - TypeScript compiler options (ES2020, strict mode, JSX react-jsx)
- `tsconfig.node.json` - Node build tools TypeScript config
- `postcss.config.js` - PostCSS with Tailwind and Autoprefixer
- `tailwind.config.js` - Tailwind theme customization (amber/stone colors, animations)
- `src-tauri/Cargo.toml` - Rust backend dependencies and features

**Feature Flags:**
- `custom-protocol` - Default feature for Tauri custom protocols
- `voxtral` - Optional feature for Voxtral transcription engine (disabled by default)

**Release Build:**
- Panic abort mode
- LTO enabled
- Code gen units: 1 (maximum optimization)
- Optimization level: 3
- Binary stripping enabled

## Platform Requirements

**Development:**
- macOS: Xcode command line tools for Objective-C compilation
- Rust 1.70+ (2021 edition)
- Node.js 16+
- Supports mixed-DPI multi-monitor setups (handles Retina + external displays)

**Production (macOS):**
- Target: macOS 10.13+ (specified in Tauri build)
- Deployment: App bundle (.app) with code signing capability
- Window level: NSMainMenuWindowLevel + 1 (25) for fullscreen overlay support
- NSPanel with NSNonactivatingPanelMask (128) for focus-stealing prevention

**Production (Windows/Linux):**
- Windows: Windows 7+ via Win32 APIs
- Linux: X11 with xtest support

---

*Stack analysis: 2026-02-24*
