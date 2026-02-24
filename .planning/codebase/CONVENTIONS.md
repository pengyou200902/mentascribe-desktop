# Coding Conventions

**Analysis Date:** 2026-02-24

## Naming Patterns

**Files:**
- React components: PascalCase (e.g., `DictationBar.tsx`, `HistoryPage.tsx`)
- Utilities/stores: camelCase (e.g., `historyStore.ts`, `dictionaryStore.ts`)
- Config files: camelCase (e.g., `widget.ts`)
- Rust modules: snake_case (e.g., `settings/mod.rs`, `audio/capture.rs`)
- Rust files: snake_case matching their module purpose

**Functions:**
- TypeScript/JavaScript: camelCase (e.g., `startRecording()`, `loadHistory()`)
- Rust: snake_case (e.g., `get_settings_path()`, `setup_hotkey()`)
- Tauri commands: snake_case (e.g., `start_recording`, `stop_recording`, `inject_text`)

**Variables:**
- React hooks/state: camelCase (e.g., `isRecording`, `audioLevel`, `isLoading`)
- Constants: UPPER_SNAKE_CASE in config files (e.g., `WAVEFORM_BAR_COUNT`, `CURSOR_POLL_INTERVAL_MS`)
- Zustand stores: camelCase properties (e.g., `entries`, `totalCount`, `error`)
- Rust structs: PascalCase (e.g., `TranscriptionSettings`, `DictionaryEntry`)
- Rust constants: UPPER_SNAKE_CASE (e.g., `OVERLAY_WINDOW_LEVEL`, `NS_NONACTIVATING_PANEL_MASK`)

**Types:**
- TypeScript interfaces: PascalCase (e.g., `DictationBarProps`, `HistoryStore`, `UserSettings`)
- Rust enums/structs: PascalCase (e.g., `WhisperError`, `SettingsError`, `HotkeyError`)
- Union types: Use `type` keyword for small unions (e.g., `type WindowType = 'dictation' | 'dashboard'`)

## Code Style

**Formatting:**
- Prettier configured for TypeScript/TSX (version ^3.2.0)
- ESLint configured (versions ^8.57.0 with TypeScript support)
- Run: `npm run format` to apply Prettier formatting
- Run: `npm run lint:fix` to fix ESLint issues

**Linting:**
- TypeScript strict mode enabled: `"strict": true` in `tsconfig.json`
- Unused variable detection: `"noUnusedLocals": true`, `"noUnusedParameters": true`
- No fallthrough switch cases: `"noFallthroughCasesInSwitch": true`
- ESLint plugins: `eslint-plugin-react`, `eslint-plugin-react-hooks`
- Command: `npm run lint` to check only, `npm run lint:fix` to auto-fix

**Rust style:**
- Standard Rust formatting via rustfmt (implicit via Cargo.toml edition="2021")
- Error types use `#[derive(Error, Debug)]` with `thiserror` crate
- Logging with `log::info!()`, `log::error!()`, and `eprintln!()` for debugging
- Documentation comments on public functions with `///`

## Import Organization

**Order (TypeScript):**
1. React and external libraries (`import React from 'react'`, `import { create } from 'zustand'`)
2. Tauri API imports (`import { invoke } from '@tauri-apps/api/core'`, `import { listen } from '@tauri-apps/api/event'`)
3. Internal components (relative paths from current file)
4. Internal utilities/stores (relative paths from current file)
5. Type imports (use `import type` to avoid circular dependencies)

**Pattern observed:**
```typescript
import { useEffect, useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { useStore } from './lib/store';
import type { UserSettings } from '../types';
```

**Rust import order:**
1. Standard library (`use std::...`)
2. External crates (`use serde::{...}`, `use tauri::{...}`)
3. Crate modules (`use crate::audio::{...}`)
4. Re-exports from super modules (`use super::{...}`)

**Path Aliases:**
- None configured for TypeScript — uses relative imports throughout
- Rust uses module paths relative to crate root

## Error Handling

**TypeScript Patterns:**

- Try-catch blocks with type narrowing:
```typescript
try {
  const text = await invoke<string>('stop_recording');
} catch (err: unknown) {
  const errorMessage = err instanceof Error ? err.message : String(err);
}
```

- Console error logging: `console.error('Failed to load history:', error)`
- Error state in stores: `error: string | null` field, set on catch
- Error display in UI: Show errors temporarily, auto-clear with setTimeout
- Zustand actions throw errors for caller to handle (e.g., `deleteEntry()` throws)
- Refs to avoid stale closures in async handlers (e.g., `isRecordingRef.current`)

**Rust Patterns:**

- Custom error enums with `#[derive(Error, Debug)]`:
```rust
#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
```

- Tauri command return type: `Result<T, String>` (error as string for IPC serialization)
- Detailed logging before errors: `eprintln!("[module] Descriptive message")` for debugging
- `log::info!()` and `log::error!()` for production logging
- Lock unwrapping: `.map_err(|e| e.to_string())?` pattern for Mutex locks
- Module-specific error types for internal functions

## Logging

**Framework:** Built-in `console.*` in TypeScript, `log` crate in Rust with `env_logger`

**Patterns:**

- TypeScript console: `console.log()` for info, `console.error()` for errors
- Rust: `log::info!()`, `log::error!()` for structured logging
- Rust debug: `eprintln!()` with prefixes like `[recording]`, `[nspanel]`, `[poll]` for CLI debugging
- Frontend to Rust logging via `invoke('frontend_log', { msg })` command
- Disable logging in invoke calls: `.catch(() => {})` for non-critical ops

**When to log:**
- State transitions (recording started/stopped)
- Async operation start/end
- Error conditions
- Event listener setup/teardown
- Monitor polling (logged at frequency intervals to avoid spam)

## Comments

**When to Comment:**
- Explain WHY, not WHAT (code shows what it does)
- Complex algorithms: Explain the logic and intent
- Non-obvious Tauri/platform workarounds
- Temporary workarounds with issue references (e.g., `// FIXME: Tauri issue #7890`)
- Critical performance notes or gotchas

**JSDoc/TSDoc:**
- Used on public functions in utilities (e.g., `tauri.ts` API wrappers)
- Format: `/** Doc comment */` above function
```typescript
/**
 * Stop recording and get transcribed text
 */
export async function stopRecording(): Promise<string> {
  return invoke('stop_recording');
}
```

- Rust: `///` doc comments on public functions
```rust
/// Parse a key name string to a Code enum
fn parse_key_code(key: &str) -> Result<Code, HotkeyError> {
```

## Function Design

**Size:**
- Prefer < 50 lines for React components; larger components split into helper render functions (e.g., `renderRecording()`, `renderError()`)
- Helper render functions extracted within component body
- Zustand store methods typically 10-30 lines

**Parameters:**
- React components use typed Props interfaces
- Zustand actions destructure state/get into parameters
- Rust commands accept minimal parameters; use `AppState` for shared data
- Optional parameters use TypeScript `?` (e.g., `isPreloading?: boolean = false`)

**Return Values:**
- Promises in async functions: `Promise<T>` for TypeScript, `async fn` in Rust
- Tauri commands return `Result<T, String>`
- Void operations return `Promise<void>` or `void`
- Render functions return JSX.Element implicitly

**Refs and Closures:**
- Use refs to capture current values in event listeners (e.g., `isRecordingRef.current`)
- Prevents stale closure bugs in async handlers
- Common pattern: set ref in useEffect dependency, read in callback

## Module Design

**Exports:**

- Default exports: React components (one per file)
- Named exports: Utilities, stores, types
- Example: `export const useHistoryStore = create<HistoryStore>(...)`
- Tauri commands exported via `invoke('command-name')` pattern

**Barrel Files:**
- None used — imports are explicit and relative
- All imports go directly to their source files

**Type Imports:**
- Use `import type` for TypeScript types to avoid circular dependencies
- Pattern: `import type { DashboardPage } from '../../types'`

**Store Pattern (Zustand):**
- Create store with `create<Interface>((set, get) => ({...}))`
- All store methods are async and handle their own error logging
- Errors either thrown or stored in `error` field
- State destructured in components: `const { entries, isLoading } = useHistoryStore()`

## Special Patterns

**Tauri Invoke Pattern:**
- Type-safe: `await invoke<ReturnType>('command-name', { arg1, arg2 })`
- Commands live in Rust (`src-tauri/src/lib.rs` and modules)
- Frontend calls via imported `invoke()` from `@tauri-apps/api/core`
- Errors bubble as strings or thrown errors

**Event Listening:**
- Pattern: `const unlisten = listen('event-name', (event) => { ... })`
- Cleanup in useEffect return: `unlisten.then((fn) => fn())`
- No unlistening in cleanup = memory leak (all async unlistens)

**Zustand with Tauri:**
- Stores invoke Tauri commands
- Set state after successful invoke
- Throw errors or store in error field for caller handling
- No automatic retry logic — caller decides

**Refs in React:**
- Sync refs with state in useEffect: `useEffect(() => { ref.current = value }, [value])`
- Read in callbacks to avoid stale closures
- Common for recording, animation, and polling state

---

*Convention analysis: 2026-02-24*
