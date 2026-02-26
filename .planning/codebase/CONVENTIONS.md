# Coding Conventions

**Analysis Date:** 2026-02-26

## Naming Patterns

**Files:**
- React components: PascalCase (e.g., `DictationBar.tsx`, `Settings.tsx`)
- Utilities/hooks: camelCase (e.g., `store.ts`, `tauri.ts`)
- Directories: kebab-case for feature directories (e.g., `src/components/dashboard`)
- Stores: Named pattern `useXStore` (e.g., `useStore`, `useHistoryStore`, `useStatsStore`)

**Functions:**
- React components: PascalCase, declared as `export const ComponentName: FC<Props> = ({}) => {}`
- Regular functions: camelCase (e.g., `startRecording`, `loadSettings`, `saveToHistory`)
- Internal helper functions: camelCase with underscore prefix for truly private (e.g., `_internalHelper`)
- Rust functions: snake_case (e.g., `load_settings`, `save_history_data`, `get_history_path`)
- Rust public functions: snake_case (e.g., `pub fn add_entry()`, `pub fn load_history()`)

**Variables:**
- Local state variables: camelCase (e.g., `isRecording`, `audioLevel`, `settings`)
- Constants (module-level): UPPER_SNAKE_CASE (e.g., `WAVEFORM_BAR_COUNT`, `CURSOR_POLL_INTERVAL_MS`)
- Ref variables (useRef): camelCase with Ref suffix (e.g., `audioLevelRef`, `isRecordingRef`, `widgetRef`)
- Rust struct fields: snake_case (e.g., `word_count`, `duration_ms`, `timestamp`)

**Types:**
- TypeScript interfaces: PascalCase, prefixed with feature context (e.g., `DictationBarProps`, `HistoryStore`, `TranscriptionSettings`)
- Type aliases: PascalCase (e.g., `WindowType = 'dictation' | 'dashboard'`)
- Rust structs: PascalCase (e.g., `TranscriptionEntry`, `WidgetSettings`, `AudioData`)
- Rust enums: PascalCase variants (e.g., `AudioError::NoInputDevice`)

## Code Style

**Formatting:**
- Tool: Prettier (with default configuration, no explicit config file found)
- Line length: Implicit enforcement via VS Code/editor defaults
- Indentation: 2 spaces (TypeScript) / 4 spaces (Rust - Rust default)

**Linting:**
- Tool: ESLint (`^8.57.0`) with TypeScript support
- Parser: `@typescript-eslint/parser`
- Plugins: `eslint-plugin-react`, `eslint-plugin-react-hooks`
- Configuration: No explicit `.eslintrc` file; uses package.json script targets `src --ext .ts,.tsx`
- Commands:
  ```bash
  npm run lint        # Check code
  npm run lint:fix    # Fix issues
  npm run format      # Format with prettier
  npm run typecheck   # TypeScript validation (strict mode)
  ```

**TypeScript Strictness:**
- Configuration in `tsconfig.json`:
  - `"strict": true` - Enables all strict type checking
  - `"noUnusedLocals": true` - Error on unused variables
  - `"noUnusedParameters": true` - Error on unused parameters
  - `"noFallthroughCasesInSwitch": true` - Error on missing switch cases
- Target: `ES2020`
- Module resolution: `bundler`
- JSX: `react-jsx` (React 17+ automatic JSX transform)

## Import Organization

**Order:**
1. External dependencies (React, third-party packages)
2. Tauri imports (`@tauri-apps/api`, `@tauri-apps/plugin-*`)
3. Internal modules (relative imports)
4. Type imports (separated as needed)

**Examples:**
```typescript
// From src/App.tsx
import { useEffect, useState, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { Dashboard } from './components/dashboard/Dashboard';
import { useStore } from './lib/store';

// From src/lib/store.ts
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { TranscriptionEntry } from '../types';
```

**Path Aliases:**
- No explicit path aliases configured in `tsconfig.json` or `vite.config.ts`
- All imports use relative paths

## Error Handling

**TypeScript/React Patterns:**
- Try-catch blocks with explicit error logging to console: `console.error('Failed to X:', error)`
- Error casting: `const errorMsg = error instanceof Error ? error.message : String(error)`
- Temporary error display states (e.g., `setError('message')` then `setTimeout(() => setError(null), timeout)`)
- Promise `.catch()` chains for Tauri invokes: `invoke(...).catch(() => {})` (silent failures for non-critical operations)

**Examples from `src/App.tsx`:**
```typescript
// Inline try-catch with error messaging
try {
  await invoke('start_recording');
  setIsRecording(true);
} catch (error) {
  isRecordingRef.current = false;
  console.error('Failed to start recording:', error);
  const errorMsg = error instanceof Error ? error.message : String(error);
  if (errorMsg.includes('Model not found')) {
    setError('Model not loaded — download in Settings');
    setTimeout(() => setError(null), ERROR_TIMEOUT_MS);
  }
}

// Fire-and-forget pattern
invoke('frontend_log', { msg }).catch(() => {});
```

**Rust Patterns:**
- Custom error types using `thiserror::Error` derive macro
- `Result<T, CustomError>` return types for fallible operations
- Error propagation with `?` operator
- Logging with `log::info!()`, `log::error!()`

**Example from `src-tauri/src/settings/mod.rs`:**
```rust
#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub fn load_settings() -> Result<UserSettings, SettingsError> {
    let path = get_settings_path();
    if !path.exists() {
        return Ok(UserSettings::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    let settings = serde_json::from_str(&contents)?;
    Ok(settings)
}
```

## Logging

**Framework:** Console API for frontend, `log` crate for Rust backend

**TypeScript/React Patterns:**
- Verbose console.log for debugging flow: `console.log('Starting recording...')`
- Error logging: `console.error('Failed to X:', error)`
- Contextual logging with prefixes: `console.log('[drag] Starting native drag')`
- Conditional suppression: Some debug logs include "poll" context for tracking

**Rust Patterns:**
- Debug output: `println!()` (for startup diagnostics, e.g., `[nspanel] setup_dictation_panel called`)
- Info level: `log::info!("Dictation window successfully converted to NSPanel")`
- Error level: `log::error!("Failed to convert dictation window to NSPanel: {:?}")`

## Comments

**When to Comment:**
- Complex algorithms or non-obvious logic (e.g., waveform animation in `DictationBar.tsx` has inline comments explaining center-factor calculation)
- Multi-step procedures with important side effects
- Apple/platform-specific limitations (extensive comments in Rust setup functions explaining NSPanel behavior)
- Performance-critical sections or workarounds

**JSDoc/TSDoc:**
- Function/component parameter documentation via TypeScript interface comments
- Inline `///` documentation for complex Rust types and functions
- Doc comments explain "why" not "what" (e.g., why NSPanel is needed, not that it exists)

**Example from `src/lib/store.ts`:**
```typescript
export interface TranscriptionSettings {
  provider?: string;
  language?: string;
  model_size?: string;
  // Comment explains the auto-detect behavior
  // Rare but useful for unclear enum values
}

// From src/components/DictationBar.tsx comments:
// "Cursor proximity detection via Rust — works regardless of window focus"
```

**Rust Doc Comments:**
```rust
/// Convert the dictation window to an NSPanel for fullscreen overlay support on macOS.
///
/// IMPORTANT: Only NSPanel can appear above fullscreen applications on macOS.
/// Regular NSWindow cannot do this regardless of window level settings.
```

## Function Design

**Size:**
- Most functions are 10-50 lines
- Async callbacks often 30-80 lines (with error handling and state updates)
- Complex renderers split into sub-render functions (e.g., `renderRecording()`, `renderProcessing()`)

**Parameters:**
- React components use destructured props with defaults: `{ isRecording = false, opacity = 1.0 } = {}`
- Zustand stores expose state and actions in a single object: `{ settings, isLoading, loadSettings, updateSettings }`
- Tauri commands are simple: `invoke('command_name', { param: value })`

**Return Values:**
- React components: JSX elements (implicit return with arrow functions)
- Async functions: `Promise<T>` where T is the Tauri-serializable result
- Store methods: `Promise<void>` for side effects, state changes are synchronous via `set()`
- Rust functions: `Result<T, Error>` for fallible operations

**Example from `src/lib/historyStore.ts` (Zustand pattern):**
```typescript
export const useHistoryStore = create<HistoryStore>((set, get) => ({
  entries: [],
  totalCount: 0,
  isLoading: false,

  loadHistory: async (reset = true) => {
    if (get().isLoading) return;  // Guard clause
    set({ isLoading: true, error: null });
    try {
      const entries = await invoke<TranscriptionEntry[]>('get_history', { ... });
      set({ entries: reset ? entries : [...get().entries, ...entries], isLoading: false });
    } catch (error) {
      console.error('Failed to load history:', error);
      set({ isLoading: false, error: String(error) });
    }
  },
}));
```

## Module Design

**Exports:**
- Zustand stores: `export const useStoreName = create<StoreType>(...)` - Always named with `use` prefix
- React components: `export const ComponentName: FC<PropsType> = ...` - Always named exports, no default exports
- Utilities: Named exports for all functions: `export function utilName() {}`
- Rust modules: `pub fn`, `pub struct`, `pub enum` for public API; no pub use re-exports unless necessary

**Barrel Files:**
- Not used in TypeScript codebase
- Direct imports from source files: `import { DictationBar } from './components/DictationBar'`

**Module Organization:**
- Feature-based structure: `src/lib/` for state/utilities, `src/components/` for UI, `src/config/` for constants
- Rust follows layered structure: `src/audio/`, `src/transcription/`, `src/settings/`, `src/history/`

**Example Rust Module (`src-tauri/src/text/mod.rs`):**
```rust
//! Text processing module for post-transcription transformations

pub fn process_text(text: &str, auto_capitalize: bool) -> String { ... }

fn capitalize_sentences(text: &str) -> String { ... }

#[cfg(test)]
mod tests { ... }  // Tests inline, gated with #[cfg(test)]
```

---

*Convention analysis: 2026-02-26*
