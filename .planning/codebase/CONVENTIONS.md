# Coding Conventions

**Analysis Date:** 2026-02-19

## Naming Patterns

**Files:**
- Components: PascalCase (e.g., `DictationBar.tsx`, `HistoryPage.tsx`, `Dashboard.tsx`)
- Utilities/Libraries: camelCase (e.g., `historyStore.ts`, `dictionaryStore.ts`, `tauri.ts`)
- Types: camelCase with `.ts` extension (e.g., `index.ts` containing interfaces)
- Directories: camelCase for utility directories (`lib/`, `components/`, `types/`), PascalCase for grouped features (`dashboard/`)

**Functions:**
- camelCase for all function names (e.g., `startRecording`, `stopRecording`, `loadSettings`, `getInitialPage`)
- Factory/hook functions: `use*` prefix for Zustand stores and React hooks (e.g., `useStore`, `useHistoryStore`, `useDictionaryStore`, `useTheme`)
- Helper functions: descriptive camelCase with clear action verbs (e.g., `saveToHistory`, `getWindowType`, `handleMouseDown`, `renderIdle`)

**Variables:**
- State variables: camelCase, prefixed with `is` for booleans (e.g., `isRecording`, `isProcessing`, `isLoading`, `isHovered`, `isPreloading`)
- Ref variables: camelCase with `Ref` suffix (e.g., `audioLevelRef`, `widgetRef`, `settingsRef`, `prevLevelsRef`, `targetHeightsRef`)
- Constants: UPPER_SNAKE_CASE for module-level constants (e.g., `PAGE_SIZE`, `THEME_KEY`)
- Time/interval values: descriptive names indicating units (e.g., `updateInterval`, `pollCount`, `animationFrameId`)

**Types:**
- Interfaces: PascalCase ending with `Props` for component props (e.g., `DictationBarProps`, `SettingsProps`)
- Interfaces: PascalCase for data types and store shapes (e.g., `TranscriptionSettings`, `UserSettings`, `HistoryStore`, `DictionaryStore`)
- Type unions: PascalCase (e.g., `WindowType`, `DashboardPage`)
- Type aliases for simple unions: UPPER_SNAKE_CASE mapped to union type (e.g., `type Theme = 'light' | 'dark' | 'system'`)

## Code Style

**Formatting:**
- Prettier version: ^3.2.0 (configured via `package.json`)
- Run formatting: `npm run format` (formats `src/**/*.{ts,tsx,css}`)
- Line breaks and indentation: 2 spaces (inferred from codebase)
- Semicolons: Required
- Arrow functions preferred over function declarations in most contexts

**Linting:**
- ESLint version: ^8.57.0 with TypeScript support
- Run linting: `npm run lint` (checks `src` for `.ts,.tsx` files)
- Run auto-fix: `npm run lint:fix`
- Plugins active: `@typescript-eslint`, `eslint-plugin-react`, `eslint-plugin-react-hooks`
- No explicit `.eslintrc` file found - using default ESLint configuration with installed plugins

**TypeScript Strictness:**
- `strict: true` - All strict type-checking options enabled
- `noUnusedLocals: true` - Error on unused local variables
- `noUnusedParameters: true` - Error on unused function parameters
- `noFallthroughCasesInSwitch: true` - Prevent accidental fallthrough in switch statements

## Import Organization

**Order:**
1. External packages/libraries (React, Tauri APIs, Zustand)
2. Relative imports from project (types, stores, components, utilities)
3. CSS/styles (imported last, at bottom)

**Example from `App.tsx`:**
```typescript
import { useEffect, useState, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { Dashboard } from './components/dashboard/Dashboard';
import { useStore } from './lib/store';
```

**Path Aliases:**
- No explicit path aliases configured in `tsconfig.json`
- Use relative paths with clear directory navigation (e.g., `'./components/DictationBar'`, `'../lib/store'`)

## Error Handling

**Patterns:**
- Try-catch blocks with `console.error()` logging in error handlers
- Errors converted to strings for store state: `String(error)`
- Type narrowing for error messages: `err instanceof Error ? err.message : String(err)` when message needs extraction (see `App.tsx:115`)
- Async operations log both success and failure states via `console.log()` and `console.error()`
- State-based error display: errors stored in React state (e.g., `setError()`) and displayed in UI
- Auto-clearing errors: errors often cleared after timeout (e.g., `setTimeout(() => setError(null), 5000)`)

**Example from `App.tsx`:**
```typescript
catch (err: unknown) {
  console.error('Failed to stop recording:', err);
  const errorMessage = err instanceof Error ? err.message : String(err);

  if (errorMessage.includes('Model not found')) {
    // Handle specific error case
  } else {
    setError(`Failed: ${errorMessage}`);
  }
  setTimeout(() => setError(null), 5000);
}
```

## Logging

**Framework:** `console` (native browser/Node.js console)

**Patterns:**
- `console.log()` - For informational messages, especially action flow (starting/stopping recording, loading data)
- `console.error()` - For error conditions that are caught and handled
- Descriptive messages: Always include context (e.g., `'Failed to load settings:', error` not just `error`)
- Tagged logs: Use bracketed prefixes for subsystem identification (e.g., `[poll]`, `[drag]`, `[app]`)
- Progress tracking: Use counter logs for periodic events (e.g., `pollCount % 20 === 0` to log every 20th poll)

**Example tagged logging from `App.tsx`:**
```typescript
console.log(`[poll] reposition_to_mouse_monitor returned TRUE (moved) at poll #${pollCount}`);
console.log(`[poll] Started 150ms monitor tracking, draggable=${settings?.widget?.draggable}`);
console.log(`[drag] Starting native drag via NSEvent monitors`);
```

**Rust-side logging:** Use `invoke('frontend_log', { msg })` to forward debug messages to Rust terminal (see `DictationBar.tsx:50`)

## Comments

**When to Comment:**
- Complex logic requiring explanation (e.g., ref syncing to avoid stale closures in `App.tsx:19`)
- Workarounds and known limitations (e.g., "Use refs to avoid stale closures in event listeners")
- Multi-step processes requiring clarification
- Browser/platform-specific quirks (e.g., WKWebView coordinate bugs)

**JSDoc/TSDoc:**
- Used for exported functions and Tauri API wrappers in `lib/tauri.ts`
- Format: `/** description */` on the line before function
- Include parameter context and return value when helpful

**Example from `lib/tauri.ts`:**
```typescript
/**
 * Stop recording and get transcribed text
 */
export async function stopRecording(): Promise<string> {
  return invoke('stop_recording');
}
```

## Function Design

**Size:** Functions are typically 10-50 lines; longer functions (100+ lines) appear in event handler setup where state orchestration is complex (e.g., `App.tsx` effect hooks)

**Parameters:**
- Component props: Destructured in function signature with `FC<PropsInterface>` type annotation (see `DictationBar.tsx:17`)
- Async functions: Parameters passed as object literals to Tauri `invoke()` calls
- Default parameters: Destructured with defaults (e.g., `{ isPreloading = false, opacity = 1.0 }`)
- Type-safe callbacks: Wrapped with `useCallback` to prevent unnecessary re-renders

**Return Values:**
- Async functions: Return `Promise<T>` with explicit type (e.g., `Promise<void>`, `Promise<string>`)
- Components: Return JSX (implicit React element)
- Store functions: Return results directly (promise-wrapped for Tauri invokes)
- Helper functions: Return computed values or undefined (not null); booleans are clear (e.g., `hasMore: entries.length >= PAGE_SIZE`)

## Module Design

**Exports:**
- Named exports for utilities and hooks (e.g., `export const useStore`, `export function ThemeProvider`)
- Default export for main App component (`export default App`)
- Components use named exports with type annotations: `export const ComponentName: FC<Props> = (...) => (...)`
- Stores export hook factory: `export const useStoreName = create<StoreType>(...)` (Zustand pattern)

**Barrel Files:**
- `types/index.ts` acts as barrel file, exporting all type definitions
- Components in subdirectories exported individually (no barrel for `components/`)
- Each store (`historyStore.ts`, `dictionaryStore.ts`, etc.) is independent; no barrel re-export

**File-to-Export Mapping:**
- `lib/store.ts` → `useStore` (main settings store)
- `lib/historyStore.ts` → `useHistoryStore` (transcription history)
- `lib/dictionaryStore.ts` → `useDictionaryStore` (custom phrase replacements)
- `lib/statsStore.ts` → `useStatsStore` (usage statistics)
- `lib/tauri.ts` → Utility functions (wrappers around Tauri invoke calls)
- `lib/theme.tsx` → `ThemeProvider`, `useTheme` (theme context and hook)

---

*Convention analysis: 2026-02-19*
