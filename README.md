# MentaScribe Desktop

Cross-platform desktop application for voice-to-text dictation, built with Tauri 2.x.

## Features

- **Local speech-to-text** via Whisper.cpp (CoreML + Metal acceleration on macOS)
- **Global hotkey** activation (hold or toggle)
- **Text injection** into any application via clipboard or keystroke simulation
- **Dictation widget** — always-on-top transparent overlay (NSPanel on macOS)
- **Dashboard window** — history, statistics, dictionary, and settings
- **Custom dictionary** — user-defined word replacements and corrections
- **Cloud STT fallback** (optional)
- **MentaScribe account sync** (optional, via `api.voice.mentaflux.ai`)

## Tech Stack

- **Framework:** Tauri 2.x (Rust backend + WebView frontend)
- **Backend:** Rust (edition 2021)
- **Frontend:** React 18 + TypeScript + Vite 5
- **Styling:** Tailwind CSS 3.4
- **State:** Zustand
- **STT Engine:** Whisper.cpp via whisper-rs (CoreML/Metal on macOS)
- **Audio:** cpal + rubato (resampling) + hound (WAV)
- **Text Injection:** enigo + arboard (clipboard)
- **Secure Storage:** keyring

## Prerequisites

- Node.js 20.x+
- npm 10.x+
- Rust 1.75+ (via rustup)
- Platform-specific dependencies (see below)

### macOS

```bash
xcode-select --install
```

The app uses macOS private APIs (`macOSPrivateApi: true`) for the NSPanel-based dictation overlay.

### Windows

- Visual Studio Build Tools with C++ workload
- WebView2 (usually pre-installed on Windows 11)

### Linux

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev
```

## Getting Started

```bash
# Install dependencies
npm install

# Start development (launches both Vite dev server and Tauri)
npm run tauri dev

# Build for current platform
npm run tauri build
```

Output locations:
- macOS: `src-tauri/target/release/bundle/dmg/`
- Windows: `src-tauri/target/release/bundle/msi/` or `nsis/`
- Linux: `src-tauri/target/release/bundle/appimage/`, `deb/`, `rpm/`

## Project Structure

```
mentascribe-desktop/
├── src/                          # React frontend
│   ├── components/
│   │   ├── dashboard/            # Dashboard window views
│   │   │   ├── Dashboard.tsx
│   │   │   ├── HomePage.tsx
│   │   │   ├── HistoryPage.tsx
│   │   │   ├── DictionaryPage.tsx
│   │   │   ├── SettingsPage.tsx
│   │   │   └── Sidebar.tsx
│   │   ├── DictationBar.tsx      # Dictation overlay widget
│   │   ├── TranscriptionOverlay.tsx
│   │   ├── MenuBar.tsx
│   │   ├── History.tsx
│   │   └── Settings.tsx
│   ├── lib/                      # State stores and utilities
│   │   ├── store.ts              # Main settings store (Zustand)
│   │   ├── historyStore.ts
│   │   ├── dictionaryStore.ts
│   │   ├── statsStore.ts
│   │   ├── tauri.ts
│   │   └── theme.tsx
│   ├── config/                   # Frontend configuration
│   ├── types/                    # TypeScript type definitions
│   ├── icons/                    # App icons
│   ├── styles/                   # CSS / Tailwind
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio/                # Audio capture, VAD
│   │   ├── transcription/        # Whisper, cloud STT
│   │   ├── hotkey/               # Global hotkey registration
│   │   ├── injection/            # Text injection (enigo/clipboard)
│   │   ├── settings/             # User preferences (persisted)
│   │   ├── api/                  # API client (mentaflux.ai)
│   │   ├── dictionary/           # Custom word dictionary
│   │   ├── history/              # Transcription history
│   │   ├── stats/                # Usage statistics
│   │   ├── text/                 # Text processing
│   │   ├── lib.rs                # Tauri command handlers
│   │   └── main.rs               # Entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── tsconfig.node.json
```

## Windows

The app has two windows configured in `src-tauri/tauri.conf.json`:

| Window | Purpose | Properties |
|--------|---------|------------|
| `dictation` | Always-on-top transcription overlay | 52x10, transparent, no decorations, skip taskbar |
| `dashboard` | Main app with history/settings/stats | 800x600 (min 640x480), resizable, routed via `#dashboard` |

## Permissions

### macOS

1. **Microphone Access** — granted via system prompt
2. **Accessibility** — must be manually enabled in System Settings → Privacy & Security → Accessibility (required for text injection)

### Windows

- Microphone access via Windows privacy settings

### Linux

- Audio access via PulseAudio/ALSA
- Input simulation requires X11 (xtest) or specific Wayland compositor support

## Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite frontend dev server |
| `npm run build` | Type-check and build frontend |
| `npm run tauri dev` | Start Tauri development (frontend + backend) |
| `npm run tauri build` | Build for production |
| `npm run lint` | Lint TypeScript with ESLint |
| `npm run lint:fix` | Auto-fix lint issues |
| `npm run format` | Format with Prettier |
| `npm run typecheck` | Type-check without emitting |
| `cargo test` | Run Rust tests |
| `cargo fmt` | Format Rust code |
| `cargo clippy` | Lint Rust code |

## Configuration

User settings are stored in:
- macOS: `~/Library/Application Support/mentascribe/`
- Windows: `%APPDATA%/mentascribe/`
- Linux: `~/.config/mentascribe/`

Whisper models are downloaded to:
- `~/.mentascribe/models/`

## License

Apache-2.0 — https://github.com/pengyou200902 © 2026
