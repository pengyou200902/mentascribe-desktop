# Coding Conventions

**Analysis Date:** 2026-04-06

## Language Style

**TypeScript (Frontend):**
- Strict mode enabled (`tsconfig.json`: `"strict": true`)
- `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch` enforced
- Target: ES2020, module resolution: `bundler`, JSX: `react-jsx`
- ESM modules (`"type": "module"` in `package.json`)

**Rust (Backend):**
- Edition 2021
- Release profile: `panic = "abort"`, LTO enabled, `opt-level = 3`, `strip = true`
- Conditional compilation with `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "windows")]`, `#[cfg(target_os = "linux")]`, and `#[cfg(feature = "voxtral")]`

## Naming Conventions

**Files:**
- React components: PascalCase (`DictationBar.tsx`, `HomePage.tsx`, `Dashboard.tsx`)
- Utilities/stores: camelCase (`store.ts`, `historyStore.ts`, `statsStore.ts`, `tauri.ts`)
- Config: camelCase (`widget.ts`)
- Types: camelCase barrel file (`types/index.ts`)
- CSS: kebab-case (`styles/globals.css`)
- Rust modules: snake_case (`mod.rs`, `capture.rs`, `client.rs`)

**Functions:**
- React event handlers: `handle` prefix (`handleCopy`, `handleDelete`, `handleSave`, `handleToggle`)
- Data loading: `load` prefix (`loadSettings`, `loadHistory`, `loadDictionary`, `loadModels`)
- State setters: `set` prefix (`setIsRecording`, `setError`, `setCurrentPage`)
- Sub-render helpers: `render` prefix (`renderRecording`, `renderProcessing`, `renderError`)
- Rust public API: snake_case (`get_history`, `add_entry`, `load_settings`)
- Tauri commands: snake_case matching Rust function names (`start_recording`, `stop_recording`, `inject_text`)

**Variables:**
- React state: camelCase (`isRecording`, `audioLevel`, `showClearConfirm`)
- Refs: camelCase + `Ref` suffix (`audioLevelRef`, `isRecordingRef`, `widgetRef`, `prevDraggableRef`)
- Module constants: UPPER_SNAKE_CASE in `src/config/widget.ts` (`WAVEFORM_BAR_COUNT`, `ERROR_TIMEOUT_MS`)
- Store page size: UPPER_SNAKE_CASE (`const PAGE_SIZE = 50`)

**Types:**
- Interfaces: PascalCase, context-prefixed for props (`DictationBarProps`, `SidebarProps`, `EditModalProps`)
- Store interfaces: PascalCase with `Store` suffix (`HistoryStore`, `DictionaryStore`, `StatsStore`)
- Settings interfaces: PascalCase with `Settings` suffix (`TranscriptionSettings`, `CleanupSettings`)
- Rust structs: PascalCase (`TranscriptionEntry`, `WidgetSettings`, `AudioData`, `ModelInfo`)
- Union types: PascalCase (`type DashboardPage = 'home' | 'history' | 'dictionary' | 'settings'`)

## Component Patterns

**Two export styles coexist -- use named function exports for new code:**

1. **Arrow function with FC type (legacy components):**
   Used in `src/components/Settings.tsx`, `src/components/DictationBar.tsx`, `src/components/History.tsx`, `src/components/MenuBar.tsx`, `src/components/TranscriptionOverlay.tsx`
   ```typescript
   export const DictationBar: FC<DictationBarProps> = ({
     isRecording,
     isProcessing,
     audioLevel = 0,
   }) => {
     // ...
   };
   ```

2. **Named function declaration (preferred for new code):**
   Used in `src/components/dashboard/Dashboard.tsx`, `src/components/dashboard/HomePage.tsx`, `src/components/dashboard/HistoryPage.tsx`, `src/components/dashboard/DictionaryPage.tsx`, `src/components/dashboard/Sidebar.tsx`, `src/components/dashboard/SettingsPage.tsx`
   ```typescript
   export function HomePage() {
     // hooks
     // helper functions
     // return JSX
   }
   ```

**Default export exception:** Only `src/App.tsx` uses `export default App` (Vite entry point convention).

**Props pattern:**
- Define an interface directly above the component: `interface DictationBarProps { ... }`
- Use destructured props with defaults for optional values
- Example from `src/components/DictationBar.tsx`:
  ```typescript
  interface DictationBarProps {
    isRecording: boolean;
    isProcessing: boolean;
    isPreloading?: boolean;
    audioLevel?: number;
    error?: string | null;
  }
  ```

**Inline SVG Icon pattern:**
- Icons are defined as arrow-function components within the same file that uses them
- Not extracted to a shared icon library
- Follow this pattern from `src/components/dashboard/Sidebar.tsx`:
  ```typescript
  const HomeIcon = ({ active }: { active?: boolean }) => (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={active ? 2 : 1.5}>
      <path strokeLinecap="round" strokeLinejoin="round" d="..." />
    </svg>
  );
  ```

**Sub-render pattern for complex state UI:**
- Used in `src/components/DictationBar.tsx` to break rendering into state-specific helpers:
  ```typescript
  const renderRecording = () => ( <div className="wispr-waveform">...</div> );
  const renderProcessing = () => ( <div className="wispr-processing">...</div> );
  const renderError = () => ( <div className="wispr-error">...</div> );
  ```

## Import Organization

**Order (follow this for all new files):**
1. React imports (`import { useState, useEffect, useRef, useCallback } from 'react'`)
2. Tauri API imports (`import { invoke } from '@tauri-apps/api/core'`)
3. Tauri event imports (`import { listen } from '@tauri-apps/api/event'`)
4. Internal component imports (`import { DictationBar } from './components/DictationBar'`)
5. Store/lib imports (`import { useStore } from './lib/store'`)
6. Config/constants imports (`import { ERROR_TIMEOUT_MS } from '../config/widget'`)
7. Type imports (`import type { TranscriptionEntry } from '../types'`)

**Reference example from `src/App.tsx`:**
```typescript
import { useEffect, useState, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { Dashboard } from './components/dashboard/Dashboard';
import { useStore } from './lib/store';
import {
  MAX_HISTORY_ENTRIES, MIC_ERROR_TIMEOUT_MS, ERROR_TIMEOUT_MS,
  // ...
} from './config/widget';
```

**Path aliases:** None configured. All imports use relative paths.

**Rust imports:**
- External crates first, then internal modules
- Use statements grouped at top of file
- Example from `src-tauri/src/history/mod.rs`:
  ```rust
  use chrono::Local;
  use serde::{Deserialize, Serialize};
  use std::path::PathBuf;
  use thiserror::Error;
  use uuid::Uuid;
  ```

## Type System Usage

**Interface over type for object shapes:**
- All component props, store state, and data models use `interface`
- `type` reserved for union types: `type WindowType = 'dictation' | 'dashboard'`

**Optional fields:**
- Use `?` suffix for optional properties:
  ```typescript
  export interface TranscriptionSettings {
    provider?: string;
    language?: string;
    model_size?: string;
    engine?: string;
  }
  ```

**Nullability pattern:**
- State that can be absent: `settings: UserSettings | null`
- Errors: `error: string | null`
- Optional with nullish coalescing: `settings?.widget?.draggable ?? false`

**Generic typing for Tauri invoke:**
```typescript
const entries = await invoke<TranscriptionEntry[]>('get_history', { limit, offset });
const text = await invoke<string>('stop_recording');
const moved = await invoke<boolean>('reposition_to_mouse_monitor');
```

**Shared types in `src/types/index.ts`:**
- Data transfer types that mirror Rust backend structs
- Frontend/backend type definitions are kept in sync manually

## Error Handling

**Frontend error pattern (standard for all async operations):**
```typescript
try {
  const result = await invoke<ReturnType>('command_name', { param });
  // success handling
} catch (error) {
  console.error('Failed to X:', error);
  const errorMsg = error instanceof Error ? error.message : String(error);
  setError(errorMsg);
  setTimeout(() => setError(null), ERROR_TIMEOUT_MS);
}
```

**Fire-and-forget for non-critical operations:**
```typescript
invoke('frontend_log', { msg }).catch(() => {});
```

**Zustand store error pattern (used consistently in all stores):**
```typescript
loadHistory: async () => {
  if (get().isLoading) return;          // Guard against concurrent loads
  set({ isLoading: true, error: null });
  try {
    const entries = await invoke<T[]>('get_history', { ... });
    set({ entries, isLoading: false });
  } catch (error) {
    console.error('Failed to load history:', error);
    set({ isLoading: false, error: String(error) });
  }
},
```

**Rust error pattern:**
- Custom error enums with `thiserror::Error` derive macro per module:
  ```rust
  #[derive(Error, Debug)]
  pub enum SettingsError {
      #[error("IO error: {0}")]
      IoError(#[from] std::io::Error),
      #[error("Serialization error: {0}")]
      SerdeError(#[from] serde_json::Error),
  }
  ```
- Tauri commands map errors to String: `.map_err(|e| e.to_string())`
- Error propagation with `?` operator in internal functions
- Non-critical errors logged and continued: `if let Err(e) = history::add_entry(...) { eprintln!("WARNING: ..."); }`

## Code Organization Within Files

**Component file structure (follow this order):**
1. Imports
2. Inline SVG icon components (if needed)
3. Type/interface definitions
4. Constants
5. Helper sub-components (e.g., `EditModal` in `DictionaryPage.tsx`)
6. Main exported component
7. Internal helper functions (render helpers, formatters)
8. JSX return

**Store file structure (`src/lib/*.ts`):**
1. Imports
2. Interface for the store shape
3. Constants (e.g., `PAGE_SIZE`)
4. `create<StoreType>()` call with all state and actions

**Zustand store pattern (use this for all new stores):**
```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SomeType } from '../types';

interface SomeStore {
  items: SomeType[];
  isLoading: boolean;
  error: string | null;
  loadItems: () => Promise<void>;
  refresh: () => Promise<void>;
}

export const useSomeStore = create<SomeStore>((set, get) => ({
  items: [],
  isLoading: false,
  error: null,

  loadItems: async () => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const items = await invoke<SomeType[]>('get_items');
      set({ items, isLoading: false });
    } catch (error) {
      console.error('Failed to load items:', error);
      set({ isLoading: false, error: String(error) });
    }
  },

  refresh: async () => {
    await get().loadItems();
  },
}));
```

**Rust module file structure:**
1. Module doc comment (`//! Description`)
2. Use statements (external crates, then std, then internal)
3. Error enum with `thiserror`
4. Data structs with `#[derive(Debug, Clone, Serialize, Deserialize)]`
5. Static/lazy globals (if needed)
6. Private helper functions
7. Public API functions
8. `#[cfg(test)] mod tests { ... }` at bottom

**Rust data persistence pattern (used in settings, history, dictionary, stats):**
```rust
fn get_data_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("data.json")
}

pub fn load_data() -> Result<Data, DataError> {
    let path = get_data_path();
    if !path.exists() { return Ok(Data::default()); }
    let contents = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

pub fn save_data(data: &Data) -> Result<(), DataError> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, contents)?;
    log::info!("Data saved to {:?}", path);
    Ok(())
}
```

## Styling Conventions

**Approach:** Tailwind CSS utility classes for dashboard components + custom CSS for dictation widget

**Dashboard components use Tailwind exclusively:**
- Dark mode via `dark:` prefix (`bg-stone-50 dark:bg-stone-950`)
- Amber accent color palette (brand color)
- Stone neutral palette for text and backgrounds
- Rounded corners: `rounded-xl` or `rounded-2xl`
- Transitions: `transition-all duration-200` or `transition-colors duration-200`
- Animation classes: `animate-fade-in`, `animate-scale-in`, `animate-spin`
- Template literal conditional classes (not clsx):
  ```typescript
  className={`px-4 py-2 ${isActive ? 'bg-amber-500 text-white' : 'bg-stone-100 text-stone-600'}`}
  ```

**Dictation widget uses custom CSS classes:**
- Defined in `src/styles/globals.css` with `wispr-` prefix
- CSS custom properties for theming (`--pill-collapsed-h`, `--pill-expanded-bg`, `--bar-color`)
- BEM-like naming: `.wispr-pill`, `.wispr-bar`, `.wispr-error-text`, `.wispr-idle-dots`

**Utility class merging:** `clsx` and `tailwind-merge` are in dependencies but barely used. Prefer template literals for conditional classes.

## Logging

**Frontend:** `console.log` and `console.error` with descriptive prefixes
- Flow logging: `console.log('Starting recording...')`
- Error logging: `console.error('Failed to start recording:', error)`
- Bracketed context: `console.log('[drag] Starting native drag via NSEvent monitors')`
- Bracketed polling: `console.log('[poll] poll #N, draggable=...')`
- Forward to Rust via: `invoke('frontend_log', { msg }).catch(() => {})`

**Rust backend:**
- `eprintln!()` for diagnostic output with `[tag]` prefixes (`[recording]`, `[nspanel]`, `[windows]`, `[settings]`)
- `log::info!()` / `log::error!()` / `log::warn!()` for structured logging via `env_logger`
- Verbose diagnostic output: sample counts, timing, buffer lengths

## Formatting and Linting

**Prettier:** Listed in devDependencies (`^3.2.0`) but no config file (`.prettierrc`) exists. Uses Prettier defaults.
- Run: `npm run format` -- formats `src/**/*.{ts,tsx,css}`

**ESLint:** Listed in devDependencies (`^8.57.0`) but no `.eslintrc` config file exists at project root.
- Plugins: `@typescript-eslint/eslint-plugin`, `@typescript-eslint/parser`, `eslint-plugin-react`, `eslint-plugin-react-hooks`
- Run: `npm run lint` / `npm run lint:fix`

**TypeScript checking:** `npm run typecheck` runs `tsc --noEmit`

**Rust:** Standard `rustfmt` defaults (no `.rustfmt.toml` detected). Run: `cargo fmt` in `src-tauri/`

## Tauri IPC Conventions

**Frontend calls Rust via `invoke`:**
```typescript
const result = await invoke<ReturnType>('snake_case_command', { camelCaseParam: value });
```

**Rust commands use `#[tauri::command]`:**
```rust
#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<settings::UserSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}
```

**Rust emits events to frontend via `app.emit()`:**
```rust
app.emit("audio-level", level).ok();
app.emit("settings-changed", &new_settings).ok();
app.emit("model-preload-start", "model-name").ok();
```

**Frontend listens via `listen()` with cleanup:**
```typescript
useEffect(() => {
  const unlisten = listen('event-name', (event) => { /* handle */ });
  return () => { unlisten.then((fn) => fn()); };
}, []);
```

## Ref Pattern for Stale Closures

**Use refs alongside state to avoid stale closures in event listeners:**
```typescript
const [isRecording, setIsRecording] = useState(false);
const isRecordingRef = useRef(isRecording);

// Keep ref in sync
useEffect(() => {
  isRecordingRef.current = isRecording;
}, [isRecording]);

// Use ref in event handlers (not state)
const startRecording = useCallback(async () => {
  if (isRecordingRef.current) return; // ref, not state
  isRecordingRef.current = true;      // set ref immediately
  // ... await async work ...
  setIsRecording(true);               // then update state for render
}, []);
```

This pattern is used extensively in `src/App.tsx` for `isRecordingRef`, `isProcessingRef`, and `settingsRef`.

## Constants Pattern

**Centralize magic numbers in config files:**
- Frontend: `src/config/widget.ts` exports all timing, size, and default constants
- Use UPPER_SNAKE_CASE: `WAVEFORM_BAR_COUNT`, `ERROR_TIMEOUT_MS`, `DEFAULT_HOTKEY_LABEL`
- Import named constants rather than using inline numbers

**Rust constants:**
- Module-level `const` for fixed values: `const MODEL_BASE_URL: &str = "..."`
- `lazy_static!` or `once_cell::sync::Lazy` for runtime-initialized statics

---

*Convention analysis: 2026-04-06*
