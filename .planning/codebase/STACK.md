# Technology Stack

**Analysis Date:** 2026-02-18

## Languages

**Primary:**
- TypeScript 5.4.0 - Frontend React components in `src/`
- Rust 2021 edition - Backend/desktop runtime in `src-tauri/`

**Secondary:**
- JavaScript - Configuration files (vite, postcss, tailwind)

## Runtime

**Environment:**
- Tauri 2.x - Desktop application framework using webview + system integration
- Node.js - Development environment (implied by package.json)

**Package Manager:**
- npm - JavaScript dependencies
- Cargo - Rust dependencies
- Lockfiles: `package-lock.json` (present), `Cargo.lock` (present)

## Frameworks

**Core Desktop:**
- Tauri 2.0.0 - Desktop application framework
  - Provides inter-process communication (IPC) between frontend and backend
  - Tray icon support via `tauri-plugin-tray-icon`
  - macOS private API support for fullscreen overlay capabilities

**Frontend UI:**
- React 18.3.0 - UI framework
- React DOM 18.3.0 - React DOM rendering

**Styling:**
- Tailwind CSS 3.4.0 - Utility-first CSS framework
- PostCSS 8.4.0 - CSS processing
- Autoprefixer 10.4.0 - Browser vendor prefix support
- Tailwind Merge 2.2.0 - Utility class merging for Tailwind
- CLSX 2.1.0 - Utility for classname management

**Build:**
- Vite 5.2.0 - Frontend build tool
- TypeScript - Language compilation
- ESLint 8.57.0 - Linting
- Prettier 3.2.0 - Code formatting

**State Management:**
- Zustand 4.5.0 - Lightweight state management library

**Testing:**
- Not detected

## Core Rust Dependencies

**Audio:**
- cpal 0.15 - Cross-platform audio I/O
- hound 3.5 - WAV file writing/reading

**Speech-to-Text:**
- whisper-rs 0.11 - Local Whisper speech-to-text model
- (Cloud providers: OpenAI, AWS Transcribe, AssemblyAI - not yet implemented)

**System Integration:**
- global-hotkey 0.5 - Global keyboard hotkey binding
- enigo 0.2 - Text injection and keyboard/mouse control
- arboard 3 - Cross-platform clipboard access
- tauri-plugin-shell 2 - Shell command execution
- tauri-plugin-dialog 2 - Native file/folder dialogs
- tauri-plugin-fs 2 - Filesystem operations
- tauri-plugin-http 2 - HTTP client plugin
- tauri-plugin-global-shortcut 2 - Global shortcut handling

**macOS-Specific:**
- tauri-nspanel (from GitHub ahkohd/tauri-nspanel v2) - NSPanel support for fullscreen overlays
- core-graphics 0.23 - macOS graphics primitives
- cocoa 0.25 - macOS Cocoa framework bindings
- objc 0.2 - Objective-C runtime
- foreign-types 0.5 - FFI wrapper types

**Windows-Specific:**
- windows 0.54 - Windows API bindings (keyboard/mouse input)

**Linux-Specific:**
- x11 2.21 - X11 protocol support

**Networking:**
- reqwest 0.11 - Async HTTP client (for API calls)
- serde 1 - Serialization framework
- serde_json 1 - JSON serialization

**Storage & Security:**
- keyring 2 - Secure credential storage (OS keychain)
- dirs 6.0.0 - Standard directories (config, home, etc.)

**Utilities:**
- tokio 1 - Async runtime (full feature set)
- log 0.4 - Logging abstraction
- env_logger 0.11 - Environment-based logging configuration
- chrono 0.4 - Date/time handling
- uuid 1 - UUID generation (v4 + serde support)
- regex 1 - Regular expressions
- thiserror 1 - Error handling macros
- anyhow 1 - Error context handling
- lazy_static 1.5.0 - Lazy static initialization
- once_cell 1.19 - One-time initialization

## Configuration

**Environment Variables:**
- `VITE_*` - Vite-specific env vars
- `TAURI_*` - Tauri-specific env vars
- Set via `envPrefix` in `vite.config.ts`
- API endpoint configuration in `src-tauri/src/api/client.rs` (hardcoded to `https://api.voice.mentaflux.ai/v1`)

**Files:**
- `vite.config.ts` - Frontend build configuration
- `tsconfig.json` - TypeScript compiler options (strict mode enabled)
- `tailwind.config.js` - Tailwind CSS configuration
- `postcss.config.js` - PostCSS configuration
- `src-tauri/tauri.conf.json` - Tauri application configuration (windows, tray, security policy)
- `src-tauri/Cargo.toml` - Rust dependencies and build profile
- `.eslintrc` - Not detected (using default or implicit rules)
- `.prettierrc` - Not detected (using defaults)

## Platform Support

**Development:**
- macOS (tested/primary)
- Windows (Windows API support included)
- Linux (X11 support included)

**Production:**
- Bundling targets: DMG (macOS), MSI/NSIS (Windows), AppImage/DEB/RPM (Linux)
- Minimum macOS version: 10.15
- Build requires: Rust toolchain, Node.js, npm

## Application Configuration

**Tauri Windows:**
1. **Dictation Window:**
   - Size: 340x120px
   - Properties: transparent, always-on-top, skipTaskbar, no decorations, non-interactive overlay
   - Location: `src/components/TranscriptionOverlay.tsx`
   - Window level: 25 (above main menu) on macOS for fullscreen overlay support

2. **Settings Window:**
   - Size: 480x640px (resizable)
   - Route: `index.html#settings`
   - Component: `src/components/Settings.tsx`

3. **Dashboard Window:**
   - Size: 800x600px (resizable, min 640x480)
   - Route: `index.html#dashboard`
   - Component: `src/components/dashboard/Dashboard.tsx`

**Security Policy:**
- CSP: `default-src 'self'; connect-src https://api.voice.mentaflux.ai; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'`
- Allows connections only to `api.voice.mentaflux.ai`

**Local Data Storage:**
- Settings: `~/.config/mentascribe/settings.json`
- History: `~/.config/mentascribe/history.json`
- Tokens: OS keychain (via `keyring` library)

## Tauri IPC Commands

**Frontend invokes Backend via:**
- `start_recording()` - Begin audio capture
- `stop_recording()` - End recording, return transcribed text
- `inject_text(text)` - Inject text into active application
- `login(email, password)` - Authenticate with MentaFlux API
- `download_model(size)` - Download Whisper model
- `get_available_models()` - List available speech models

---

*Stack analysis: 2026-02-18*
