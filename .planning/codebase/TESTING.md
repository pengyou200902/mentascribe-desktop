# Testing Patterns

**Analysis Date:** 2026-02-19

## Test Framework

**Status:** No testing framework currently configured or in use.

**Runner:** Not configured
- No `jest.config.*`, `vitest.config.*`, or test runner detected
- No test scripts in `package.json` (`npm run test` undefined)

**Assertion Library:** Not configured

**Run Commands:** Not available
- No testing infrastructure present in the codebase

## Test File Organization

**Current State:** No test files detected
- Search across project for `*.test.ts`, `*.test.tsx`, `*.spec.ts`, `*.spec.tsx` returned no results
- No `tests/` or `__tests__/` directories present

**Recommended Pattern (if testing is added):**
- Co-located: Test files should live alongside source files with `.test.tsx` suffix
- Directory structure: `src/components/DictationBar.tsx` → `src/components/DictationBar.test.tsx`
- Store tests: `src/lib/store.ts` → `src/lib/store.test.ts`

## Test Structure

**No existing patterns to document.** The following is recommended if testing is adopted:

**Suggested Framework Choice:**
- Vitest (modern, Vite-integrated, fast)
- Alternative: Jest (broader ecosystem, more mature)

**Example Structure (recommended):**
```typescript
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { useStore } from './store';

describe('useStore', () => {
  beforeEach(() => {
    // Reset store state before each test
  });

  it('should load settings', async () => {
    const { result } = renderHook(() => useStore());
    await act(async () => {
      await result.current.loadSettings();
    });
    expect(result.current.settings).toBeDefined();
  });
});
```

## Mocking

**Framework:** Not configured
- No mock library (e.g., Vitest, Jest built-in, or MSW) configured

**What to Mock (if testing is added):**
- Tauri `invoke()` calls: Mock all backend invocations to test UI layer in isolation
- Browser APIs: `localStorage`, `window.matchMedia` (theme), `crypto.randomUUID` (history)
- Event listeners: Mock Tauri event `listen()` to test event-driven state changes
- `ResizeObserver` and `requestAnimationFrame` for animation testing in `DictationBar.tsx`

**What NOT to Mock:**
- React hooks state setters (let them run naturally in tests)
- Component render logic (test actual JSX output)
- Store creation logic (use actual Zustand stores)

**Suggested Pattern:**
```typescript
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: any) => {
    if (cmd === 'get_settings') {
      return defaultSettings;
    }
    // ... other mocks
  }),
}));
```

## Fixtures and Factories

**No test data fixtures exist.** If testing is added:

**Recommended Pattern:**
Create `src/__fixtures__/` directory with factory functions:

```typescript
// src/__fixtures__/settingsFactory.ts
import type { UserSettings } from '../lib/store';

export function createDefaultSettings(): UserSettings {
  return {
    transcription: {
      provider: 'whisper-cpp',
      language: 'en',
      model_size: 'small',
    },
    cleanup: {
      enabled: false,
      remove_filler: false,
      add_punctuation: false,
      format_paragraphs: false,
    },
    hotkey: { key: 'cmd', mode: 'toggle' },
    output: { insert_method: 'paste', auto_capitalize: false },
    widget: { draggable: true, opacity: 1.0 },
  };
}

export function createHistoryEntry(overrides = {}) {
  return {
    id: crypto.randomUUID(),
    text: 'Sample transcription',
    word_count: 2,
    duration_ms: 1000,
    timestamp: new Date().toISOString(),
    synced: false,
    ...overrides,
  };
}
```

**Location:**
- Factories at `src/__fixtures__/` directory
- Import in test files: `import { createDefaultSettings } from '../__fixtures__/settingsFactory'`

## Coverage

**Requirements:** None enforced
- No coverage configuration present
- No CI/CD hooks checking coverage thresholds

**Recommended Targets (if testing is added):**
- Unit tests: 80%+ for stores (`lib/*.ts`), types, utilities
- Component tests: 70%+ for dashboard pages, settings
- Lower priority: Animation components (DictationBar), theme context

**View Coverage (if Vitest is adopted):**
```bash
npm run test -- --coverage
```

## Test Types

**No tests currently exist.** Recommended structure if testing is adopted:

**Unit Tests:**
- Scope: Individual stores, utility functions, type conversions
- Approach: Test Zustand store actions with mocked Tauri invoke calls
- Examples:
  - `useStore.loadSettings()` returns settings correctly
  - `useHistoryStore.deleteEntry(id)` removes entry from state
  - Theme getter functions resolve correctly

**Integration Tests:**
- Scope: Store + component interaction, Tauri invoke workflows
- Approach: Render components with mocked stores, trigger actions, verify state changes
- Examples:
  - DictationBar shows recording state when `isRecording=true`
  - Hotkey event triggers `startRecording` correctly
  - Settings changes persist via `updateSettings` invoke

**E2E Tests:**
- Framework: Not currently configured
- Recommendation: If added, use Tauri's built-in test utilities or Playwright for full app testing
- Low priority: Most user flows are tested manually (hotkey recording, UI interactions)

## Common Patterns

**Async Testing (recommended if testing is added):**
```typescript
// Using Vitest + React Testing Library
it('should load history on mount', async () => {
  const { result } = renderHook(() => useHistoryStore());

  await waitFor(() => {
    expect(result.current.isLoading).toBe(false);
  });

  expect(result.current.entries).toHaveLength(greaterThan(0));
});

// With act() for state updates
await act(async () => {
  await result.current.loadHistory();
});
```

**Error Testing (recommended):**
```typescript
it('should handle failed invocations', async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error('Invoke failed'));

  const { result } = renderHook(() => useStore());

  await act(async () => {
    await result.current.loadSettings();
  });

  expect(result.current.isLoading).toBe(false);
  expect(result.current.error).toBeTruthy();
});
```

**Component Event Testing (recommended):**
```typescript
it('should handle mouse down drag start', async () => {
  const { container } = render(
    <DictationBar draggable={true} {...defaultProps} />
  );

  const pill = container.querySelector('.wispr-pill');
  await userEvent.pointer({ keys: '[MouseLeft>]', target: pill });

  expect(vi.mocked(invoke)).toHaveBeenCalledWith('start_native_drag');
});
```

## Current Testing Reality

**No active testing infrastructure.** All current validation is:
- Manual testing by developer (hotkey recording, UI interactions)
- TypeScript compilation (`npm run typecheck`)
- ESLint static analysis (`npm run lint`)

**Risks of no tests:**
- Regressions in Tauri integration layer not caught
- Store action bugs only discovered in runtime
- Component refactors may break event listeners silently
- Coordinate conversion bugs in DictationBar hard to verify

**Recommended Next Steps:**
1. Install Vitest: `npm install -D vitest @vitest/ui @testing-library/react`
2. Create `vitest.config.ts` with React support
3. Add `npm run test` and `npm run test:ui` scripts
4. Start with store tests (highest ROI): `useStore`, `useHistoryStore`, `useDictionaryStore`
5. Graduate to component tests for critical UI flows

---

*Testing analysis: 2026-02-19*
