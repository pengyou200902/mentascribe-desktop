# MentaScribe Desktop

An open-source, cross-platform desktop dictation app — similar to [Wispr Flow](https://wisprflow.ai/) — built with Tauri 2.x. Runs entirely on-device with no subscription required.

Screenshots available in the [`images/`](./images) folder.

![Demo](images/tiny_demo.gif)

## Features

- **Local speech-to-text** via Whisper.cpp (CoreML + Metal acceleration on macOS)
- **Global hotkey** activation (hold or toggle)
- **Text injection** into any application via clipboard or keystroke simulation
- **Dictation widget** — always-on-top transparent overlay (NSPanel on macOS)
- **Dashboard window** — history, statistics, dictionary, and settings
- **Custom dictionary** — user-defined word replacements and corrections
- **Cloud STT fallback** (optional)
- **MentaScribe account sync** (optional, via `api.voice.mentaflux.ai`)

## Platform Support

MentaScribe runs on macOS, Windows, and Linux. macOS is the primary development target and offers the most complete feature set.

| Feature | macOS | Windows | Linux |
|---------|:-----:|:-------:|:-----:|
| Whisper transcription | CoreML + Metal GPU | CPU only | CPU only (OpenBLAS) |
| Voxtral transcription | Metal GPU | CPU only (slow) | CPU only (OpenBLAS) |
| Dictation overlay | NSPanel (non-activating, above fullscreen) | Always-on-top (basic) | Always-on-top (basic) |
| Widget opacity | Native alpha control | Not yet implemented | Not yet implemented |
| Text injection (Accessibility) | macOS AX API | Falls back to keyboard sim | Falls back to keyboard sim |
| Text injection (Keyboard) | CGEvent | SendInput | X11 xtest |
| Global hotkey | Yes | Yes | Yes |
| Tray icon | Yes | Yes | Yes |

**macOS** provides the best experience thanks to hardware acceleration (Neural Engine via CoreML, Metal GPU), the NSPanel-based overlay that doesn't steal focus from other apps, and native accessibility text injection. If you're choosing a platform, macOS on Apple Silicon is recommended.

**Windows and Linux** support is functional — audio capture, Whisper transcription (CPU), hotkeys, clipboard/keyboard text injection, and the dashboard all work. GPU acceleration and the advanced overlay features are areas for future improvement. Contributions welcome!

## Getting Started

### Prerequisites

- Node.js 20.x+
- npm 10.x+
- Rust 1.75+ (via rustup)

<details>
<summary><strong>macOS</strong></summary>

```bash
xcode-select --install
```

The app uses macOS private APIs (`macOSPrivateApi: true`) for the NSPanel-based dictation overlay.
</details>

<details>
<summary><strong>Windows</strong></summary>

- Visual Studio Build Tools with C++ workload
- WebView2 (usually pre-installed on Windows 11)
</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
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
</details>

### Install and Run

```bash
git clone https://github.com/pengyou200902/mentascribe-desktop.git
cd mentascribe-desktop

# Install dependencies
npm install

# Start development (launches both Vite dev server and Tauri)
npm run tauri dev

# Build for current platform
npm run tauri build

# Start development with Voxtral model support
pnpm tauri dev --features voxtral
```

Build output:
- macOS: `src-tauri/target/release/bundle/dmg/`
- Windows: `src-tauri/target/release/bundle/msi/` or `nsis/`
- Linux: `src-tauri/target/release/bundle/appimage/`, `deb/`, `rpm/`

## Configuration

User settings are stored in:
- macOS: `~/Library/Application Support/mentascribe/`
- Windows: `%APPDATA%/mentascribe/`
- Linux: `~/.config/mentascribe/`

Whisper models are downloaded to `~/.mentascribe/models/`.

## Permissions

| Platform | Requirements |
|----------|-------------|
| **macOS** | Microphone access (system prompt) + Accessibility (System Settings > Privacy & Security, required for text injection) |
| **Windows** | Microphone access via Windows privacy settings |
| **Linux** | Audio access via PulseAudio/ALSA, input simulation requires X11 (xtest) or Wayland compositor support |

## App Windows

| Window | Purpose | Properties |
|--------|---------|------------|
| `dictation` | Always-on-top transcription overlay | 52x10, transparent, no decorations, skip taskbar |
| `dashboard` | Main app with history/settings/stats | 800x600 (min 640x480), resizable, routed via `#dashboard` |

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

<details>
<summary><strong>Project Structure</strong></summary>

```
mentascribe-desktop/
├── src/                          # React frontend
│   ├── components/
│   │   ├── dashboard/            # Dashboard window views
│   │   ├── DictationBar.tsx      # Dictation overlay widget
│   │   ├── TranscriptionOverlay.tsx
│   │   └── Settings.tsx
│   ├── lib/                      # State stores and utilities
│   │   ├── store.ts              # Main settings store (Zustand)
│   │   ├── historyStore.ts
│   │   ├── dictionaryStore.ts
│   │   └── theme.tsx
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio/                # Audio capture, VAD
│   │   ├── transcription/        # Whisper, Voxtral, cloud STT
│   │   ├── hotkey/               # Global hotkey registration
│   │   ├── injection/            # Text injection (enigo/clipboard)
│   │   ├── settings/             # User preferences (persisted)
│   │   ├── dictionary/           # Custom word dictionary
│   │   ├── history/              # Transcription history
│   │   ├── text/                 # Text processing
│   │   ├── lib.rs                # Tauri command handlers
│   │   └── main.rs               # Entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```
</details>

## Scripts

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Start Tauri development (frontend + backend) |
| `npm run tauri build` | Build for production |
| `npm run lint` | Lint TypeScript with ESLint |
| `npm run format` | Format with Prettier |
| `npm run typecheck` | Type-check without emitting |
| `cargo test` | Run Rust tests |
| `cargo clippy` | Lint Rust code |

## License

MIT
