# Coding Conventions

**Analysis Date:** 2026-02-20

## Naming Patterns

**Files:**
- React components: PascalCase (e.g., `DictationBar.tsx`, `Dashboard.tsx`, `HistoryPage.tsx`)
- Utility/store files: camelCase (e.g., `store.ts`, `tauri.ts`, `historyStore.ts`)
- Rust modules: snake_case (e.g., `mod.rs`, `capture.rs`, `cloud.rs`)
- Type definition files: `index.ts` for type exports

**Functions:**
- TypeScript/JavaScript: camelCase (e.g., `startRecording()`, `stopRecording()`, `saveToHistory()`)
- Rust: snake_case (e.g., `calculate_rms()`, `get_current_level()`, `capitalize_sentences()`)
- React hooks: camelCase starting with `use` (e.g., `useStore()`, `useHistoryStore()`)

**Variables:**
- Constants: UPPER_SNAKE_CASE for window levels and magic numbers
  - Example: `OVERLAY_WINDOW_LEVEL = 25`, `NS_NONACTIVATING_PANEL_MASK = 128`
- State variables: camelCase (e.g., `isRecording`, `audioLevel`, `waveformBars`)
- Refs: camelCase with `Ref` suffix (e.g., `widgetRef`, `audioLevelRef`, `isRecordingRef`)

**Types:**
- Interfaces: PascalCase (e.g., `DictationBarProps`, `UserSettings`, `TranscriptionEntry`)
- Enums: PascalCase (e.g., `WindowType`, `DashboardPage`)
- Type unions: PascalCase (e.g., `WindowType = 'dictation' | 'dashboard'`)

## Code Style

**Formatting:**
- Prettier 3.2.0 - configured via package.json scripts
  - Run via `npm run format` to format `src/**/*.{ts,tsx,css}`
- Tab width: 2 spaces (default Prettier)
- Line length: Default Prettier (80 characters)
- Quotes: Single quotes in TypeScript, double quotes in JSX attributes

**Linting:**
- ESLint 8.57.0 with TypeScript support
  - Parser: `@typescript-eslint/parser`
  - Plugins: `@typescript-eslint/eslint-plugin`, `eslint-plugin-react`, `eslint-plugin-react-hooks`
  - Run via `npm run lint` to check, `npm run lint:fix` to auto-fix
  - Config: embedded in package.json (not checked; assume standard TS/React rules)

**TypeScript Configuration:**
- Target: ES2020
- Module: ESNext
- Strict mode enabled: `"strict": true`
- Unused variables flagged: `"noUnusedLocals": true`, `"noUnusedParameters": true`
- All `.ts` and `.tsx` files must pass `npm run typecheck` (tsc --noEmit)

## Import Organization

**Order:**
1. External React/Tauri imports (e.g., `import { FC } from 'react'`, `import { invoke } from '@tauri-apps/api/core'`)
2. Type imports from local types (e.g., `import type { DashboardPage } from '../../types'`)
3. Local component imports (e.g., `import { DictationBar } from './components/DictationBar'`)
4. Local utility/store imports (e.g., `import { useStore } from './lib/store'`)
5. CSS imports (e.g., `import '../../styles/main.css'`)

**Path Aliases:**
- No path aliases configured; use relative imports with `../` and `./`
- Imports within `src/` use relative paths (e.g., `'./components/'`, `'../lib/'`)

**Type Imports:**
- Use `import type` for TypeScript-only imports where possible
- Example: `import type { DashboardPage } from '../../types'`

## Error Handling

**Patterns:**
- Try-catch blocks are standard for async operations
- Error logging: Always use `console.error()` with descriptive message and error object
  - Example: `console.error('Failed to save to history:', e);`
  - Include context in message, don't just log the error
- User-facing errors: Set to state (e.g., `setError()`) and auto-clear with `setTimeout()`
  - Example: `setError('Mic busy — try again'); setTimeout(() => setError(null), 2000);`
  - Errors typically clear after 2-5 seconds depending on severity
- Rust errors: Use `thiserror` crate for custom error types with `#[error]` attributes
  - Example from `settings/mod.rs`: `#[error("IO error: {0}")]`
- Silent failures: Use `.catch(() => {})` when fire-and-forget operations should not block
  - Example: `invoke('frontend_log', { msg }).catch(() => {})`

**Special patterns:**
- Detecting error types: Check error message with `.includes()` for specific conditions
  - Example: `if (errorMessage.includes('Model not found')) { ... }`
- Catching unknown errors in Rust: Use `err: unknown` and convert with `instanceof Error ? err.message : String(err)`

## Logging

**Framework:** `console` object (no logging library in frontend)

**Patterns:**
- Info: `console.log('message')` for lifecycle events
- Error: `console.error('message', error)` for exceptions
- Conditional context logging: Use bracket prefixes for categorized logs
  - Examples: `console.log('[poll] ...')`, `console.log('[drag] ...')`, `console.log('[app] ...')`
  - Helps trace execution flow in mixed frontend-backend scenarios
- Rust: `log` crate (0.4) + `env_logger` (0.11)
  - Info level: `log::info!()`
  - Error level: `log::error!()`
  - Debug prints for critical errors: `println!("[context] ...")` for terminal output visibility

**What to log:**
- Async operation start/completion (with context about what changed)
- Error conditions with full error message and state
- State changes that affect UI behavior (e.g., draggable prop changes, monitor repositioning)
- Frontend-to-backend invocation boundaries (for debugging FFI issues)

**What NOT to log:**
- Sensitive data (API keys, tokens, passwords) — never log these
- High-frequency updates like audio level changes — can spam logs
- Internal implementation details that don't affect behavior

## Comments

**When to Comment:**
- Complex algorithms: Explain the "why" not the "what"
  - Example in `DictationBar.tsx`: `// Create a wave-like pattern with center bars taller`
- Non-obvious logic: When code behavior differs from naming
  - Example: Explaining why refs are used instead of state
- Workarounds and hacks: Explain the reason for unusual code patterns
  - Example in `App.tsx`: `// Set ref immediately to prevent duplicate calls during await`
- Critical macOS-specific behavior: Always document NSPanel and coordinate quirks
- FFI boundaries: Note when code bridges Rust and TypeScript/JavaScript

**JSDoc/TSDoc:**
- Used selectively, primarily for exported functions and interfaces
- Example from `tauri.ts`:
  ```typescript
  /**
   * Start audio recording
   */
  export async function startRecording(): Promise<void> {
  ```
- Not used for internal functions; inline comments preferred
- Type annotations are considered documentation

## Function Design

**Size:**
- Prefer functions under 50 lines when possible
- Complex state machines (like `App.tsx` event handlers) may exceed this
- Large components split concerns into sub-components

**Parameters:**
- Max 3-4 required parameters; use object destructuring for more
- Props interfaces use destructuring in function signature
  - Example: `export const DictationBar: FC<DictationBarProps> = ({ isRecording, isProcessing, ... }) => {`
- Optional parameters marked with `?` in interfaces
- Default values provided as fallback (e.g., `isPreloading = false`)

**Return Values:**
- Async functions return Promises with typed payloads
  - Example: `async function stopRecording(): Promise<string>`
- React components return JSX
- Utility functions return data or void
- Use early returns to reduce nesting:
  ```typescript
  if (!condition) return;
  // main logic here
  ```

**Arrow Functions vs Named Functions:**
- React components: Named functions preferred for readability and error traces
  - Example: `function DashboardContent() { ... }`
- Callbacks and handlers: Arrow functions for lexical `this` binding
  - Example: `const handleMouseMove = (e: MouseEvent) => { ... }`
- Store/Zustand: Arrow functions for closure access
  - Example: `loadHistory: async () => { ... }`

## Module Design

**Exports:**
- Named exports for utilities and stores
  - Example: `export const useStore = create<Store>(...)`
- Default exports for React components (both are present)
  - Example: `export default App;` and standalone export `export const DictationBar`
- Barrel files: `index.ts` in type directories for convenient imports
  - Example: `src/types/index.ts` exports all type interfaces

**Module Organization:**
- One component per file (e.g., `DictationBar.tsx` contains only `DictationBar`)
- Props interfaces in same file as component, above component definition
- Stores in separate `*Store.ts` files (e.g., `historyStore.ts`, `dictionaryStore.ts`)
- Utilities grouped by domain (e.g., `src/lib/tauri.ts` for Tauri FFI wrappers)

**Rust Module Structure:**
- `mod.rs` files contain public API and re-exports
  - Example from `audio/mod.rs`: `pub mod capture; pub mod vad; pub use capture::AudioData;`
- Submodules (`capture.rs`, `vad.rs`) contain implementation
- Error types defined in `mod.rs` or submodule with `pub enum ErrorType { ... }`
- Tests in same file as implementation using `#[cfg(test)] mod tests { ... }`

## Zustand Store Pattern

**Store Definition:**
```typescript
interface Store {
  // State properties
  settings: UserSettings | null;
  isLoading: boolean;

  // Methods
  loadSettings: () => Promise<void>;
  updateSettings: (settings: UserSettings) => Promise<void>;
}

export const useStore = create<Store>((set) => ({
  settings: null,
  isLoading: false,

  loadSettings: async () => {
    set({ isLoading: true });
    try {
      const data = await invoke('get_settings');
      set({ settings: data, isLoading: false });
    } catch (error) {
      console.error('Failed to load settings:', error);
      set({ isLoading: false });
    }
  },
  // ... more methods
}));
```

- Single store per domain (settings, history, dictionary, stats)
- Use `get()` inside actions to read current state
- Always wrap async operations in try-catch
- Set error state explicitly if needed

## React Patterns

**Hooks Usage:**
- `useState`: For local component state
- `useRef`: For mutable values that don't trigger re-render (e.g., animation refs)
- `useEffect`: For side effects with proper cleanup
- `useCallback`: For event handlers and functions passed to other components
- `FC<Props>`: Functional component with typed props

**State Synchronization:**
- Use refs to prevent stale closures in event listeners
- Keep refs in sync with state via separate `useEffect`
  - Example: `useEffect(() => { isRecordingRef.current = isRecording; }, [isRecording])`

**Event Listeners:**
- Always clean up listeners in return function from `useEffect`
- Use Tauri's `listen()` with proper unlisten in cleanup:
  ```typescript
  const unlisten = listen('event-name', (payload) => { ... });
  return () => { unlisten.then((fn) => fn()); };
  ```

**Conditional Rendering:**
- Use ternary operators for simple conditions
- Use if statements for complex logic before JSX
- Pattern in `DictationBar.tsx`: `{error ? renderError() : isProcessing ? renderProcessing() : isRecording ? renderRecording() : ...}`

---

*Convention analysis: 2026-02-20*
