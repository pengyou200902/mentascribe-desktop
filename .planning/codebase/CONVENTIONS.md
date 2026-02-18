# Coding Conventions

**Analysis Date:** 2026-02-18

## Naming Patterns

**Files:**
- Components: PascalCase (e.g., `DictationBar.tsx`, `Settings.tsx`)
- Store files: camelCase with suffix (e.g., `historyStore.ts`, `dictionaryStore.ts`, `statsStore.ts`)
- Utility files: camelCase (e.g., `tauri.ts`, `theme.tsx`)
- Type files: `index.ts` in `types/` directory

**Functions:**
- React components: PascalCase (e.g., `HomePage`, `DictationBar`)
- Regular functions: camelCase (e.g., `loadModels`, `saveToHistory`, `getWindowType`)
- Hook functions: camelCase with `use` prefix (e.g., `useStore`, `useTheme`, `useHistoryStore`)
- Event listeners: camelCase with prefix (e.g., `handleChange`, `handleMouseMove`, `handlePointerEnter`)
- Tauri async commands: snake_case for invocations (e.g., `invoke('start_recording')`, `invoke('get_settings')`)

**Variables:**
- State variables: camelCase (e.g., `isRecording`, `audioLevel`, `isDownloadingModel`)
- Refs: camelCase with `Ref` suffix (e.g., `isRecordingRef`, `widgetRef`, `prevLevelsRef`)
- Constants: UPPER_SNAKE_CASE (e.g., `PAGE_SIZE`, `THEME_KEY`, `updateInterval`)
- Interface/type names: PascalCase (e.g., `UserSettings`, `DictationBarProps`, `HistoryStore`)

**Types:**
- Interfaces: PascalCase with `Props` suffix for component props (e.g., `DictationBarProps`, `SettingsProps`, `ThemeProviderProps`)
- Type unions: PascalCase or literal unions (e.g., `WindowType`, `Theme = 'light' | 'dark' | 'system'`)
- Enums: Not used; type unions preferred
- Store interfaces: PascalCase (e.g., `Store`, `HistoryStore`, `DictionaryStore`)

## Code Style

**Formatting:**
- Prettier v3.2.0 configured (via `package.json`)
- Command: `pnpm format` - formats `src/**/*.{ts,tsx,css}`
- Line length: No explicit limit in config; defaults to 80 characters
- Indentation: 2 spaces (default Prettier)
- Semicolons: Enabled (Prettier default)
- Quotes: Double quotes (Prettier default)

**Linting:**
- ESLint v8.57.0 with TypeScript support
- Config file location: Not found in root (likely using defaults)
- Plugins: `@typescript-eslint`, `react`, `react-hooks`
- Commands:
  - `pnpm lint` - Check `src` with `.ts,.tsx` extensions
  - `pnpm lint:fix` - Auto-fix issues
- Key rules enabled via tsconfig: `strict` mode, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`

**TypeScript:**
- `tsconfig.json` target: ES2020
- `strict: true` - Full type safety enforced
- `noUnusedLocals: true` - Errors on unused variables
- `noUnusedParameters: true` - Errors on unused function parameters
- `jsx: react-jsx` - React 18+ JSX transform (no import React needed)
- `skipLibCheck: true` - Skips type checking of declaration files

## Import Organization

**Order:**
1. External packages (e.g., `import React from 'react'`, `import { useState } from 'react'`)
2. Tauri APIs (e.g., `import { invoke } from '@tauri-apps/api/core'`, `import { listen } from '@tauri-apps/api/event'`)
3. Local components (e.g., `import { DictationBar } from './components/DictationBar'`)
4. Local utilities/stores (e.g., `import { useStore } from './lib/store'`)
5. Types (e.g., `import type { TranscriptionEntry } from '../types'`)

**Path Aliases:**
- Not configured; relative imports used throughout
- Example: `'../lib/store'`, `'../../lib/historyStore'`

**Type imports:**
- Use `import type` for pure type imports (e.g., `import type { UserSettings } from './store'`)
- Keep with regular imports if re-exporting interface with values

## Error Handling

**Patterns:**
- Try-catch blocks for async operations
- Error logging: `console.error('Context:', error)` - descriptive message followed by error object
- Type narrowing: `err instanceof Error ? err.message : String(err)` for unknown error types
- Silent errors acceptable for non-critical operations (e.g., monitor repositioning): wrapped in try-catch with comment
- Error state management: Errors stored in Zustand stores with `error: string | null` field
- UI error display: Temporary error messages that auto-clear after timeout (e.g., `setTimeout(() => setError(null), 5000)`)

Examples from codebase:
```typescript
// Example 1: Async with error state (from App.tsx)
try {
  await invoke('start_recording');
  setIsRecording(true);
} catch (error) {
  isRecordingRef.current = false;
  console.error('Failed to start recording:', error);
}

// Example 2: Error type narrowing (from App.tsx)
catch (err: unknown) {
  const errorMessage = err instanceof Error ? err.message : String(err);
  setError(`Failed: ${errorMessage}`);
}

// Example 3: Silent error handling (from App.tsx)
try {
  await invoke('reposition_to_mouse_monitor');
} catch (err) {
  // Silently ignore errors (window might not be visible, etc.)
}

// Example 4: Store error handling (from historyStore.ts)
catch (error) {
  console.error('Failed to load history:', error);
  set({ isLoading: false, error: String(error) });
}
```

## Logging

**Framework:** Native `console` object

**Patterns:**
- `console.log()` - Info/debug: Used for flow tracking (e.g., `console.log('Recording started')`)
- `console.error()` - Errors: Always prefix with context (e.g., `console.error('Failed to load models:', error)`)
- No custom logger; direct console calls throughout
- Informational logs in component/store state transitions
- Error logs include context and error object

## Comments

**When to Comment:**
- Complex algorithms or non-obvious logic (e.g., waveform animation timing, refs usage)
- Workarounds and explanations (e.g., "Use refs to avoid stale closures in event listeners")
- Section dividers for major code blocks (e.g., `// Keep refs in sync with state`)
- Commented explain "why" not "what"

**JSDoc/TSDoc:**
- Functions in `tauri.ts` have JSDoc-style comments:
  ```typescript
  /**
   * Start audio recording
   */
  export async function startRecording(): Promise<void> {
  ```
- Not extensively used across other files
- Type definitions are self-documenting via TypeScript interfaces

**Example comments:**
```typescript
// Helper to save transcription to history
const saveToHistory = useCallback((text: string) => {

// Use refs to avoid stale closures in event listeners
const isRecordingRef = useRef(isRecording);

// Keep refs in sync with state
useEffect(() => {
  isRecordingRef.current = isRecording;
}, [isRecording]);

// Check every 150ms for monitor changes (fast enough to feel responsive)
const intervalId = setInterval(checkMouseMonitor, 150);
```

## Function Design

**Size:**
- Functions range from 5-50 lines typically
- Complex components like `App.tsx` use useCallback to extract sub-functions
- Smaller, focused functions preferred

**Parameters:**
- Destructured props for React components (e.g., `FC<DictationBarProps> = ({ isRecording, isProcessing }) =>`)
- Explicit parameters for utility functions
- Optional parameters use `?` in types (e.g., `embedded?: boolean`)
- Default values in parameters: `audioLevel = 0`

**Return Values:**
- React components: JSX or conditional renders
- Async functions: Explicitly typed (e.g., `async () => Promise<void>`, `async () => Promise<string>`)
- Zustand actions: State updates via `set()` or reads via `get()`
- Callbacks wrapped in `useCallback` to maintain stable function identity

## Module Design

**Exports:**
- Named exports preferred (e.g., `export const useStore = create<Store>(...)`)
- Default exports used for React components (e.g., `export default App`)
- Type exports: `export interface`, `export type`

**Barrel Files:**
- Not used; no index.ts in component directories
- Each file exports what it defines directly

**Store Pattern (Zustand):**
- Single Zustand store per domain (e.g., `useStore`, `useHistoryStore`, `useDictionaryStore`)
- Store interface defines state and actions
- Actions are async, return Promises
- State updates via `set()` with state updater function
- State reads via `get()`
- Loading/error/data pattern: `isLoading`, `error`, data fields

Example from `historyStore.ts`:
```typescript
interface HistoryStore {
  entries: TranscriptionEntry[];
  totalCount: number;
  isLoading: boolean;
  hasMore: boolean;
  error: string | null;
  loadHistory: (reset?: boolean) => Promise<void>;
  loadMore: () => Promise<void>;
  // ... other actions
}

export const useHistoryStore = create<HistoryStore>((set, get) => ({
  entries: [],
  totalCount: 0,
  isLoading: false,
  hasMore: true,
  error: null,

  loadHistory: async (reset = true) => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    // ...
  },
}));
```

---

*Convention analysis: 2026-02-18*
