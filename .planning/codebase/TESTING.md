# Testing Patterns

**Analysis Date:** 2026-02-24

## Test Framework

**Status:** Not detected

No test framework is installed or configured in this codebase. The project currently has:
- No `jest.config.*` or `vitest.config.*` file
- No test dependencies in `package.json`
- No test files (`*.test.ts`, `*.spec.ts`, etc.) in the codebase
- No test scripts in `package.json` (only `dev`, `build`, `lint`, `format`, `typecheck`)

**Recommendation:** To add testing, configure either:
- **Vitest** (lightweight, Vite-native)
- **Jest** (industry standard, needs extra config for Tauri)

## Run Commands

Current available commands:
```bash
npm run dev              # Start Vite dev server
npm run build            # Build frontend and run tsc type check
npm run preview          # Preview production build
npm run lint             # Check code with ESLint
npm run lint:fix         # Auto-fix ESLint issues
npm run format           # Auto-format with Prettier
npm run typecheck        # Run tsc type checking only
```

To add tests, commands would need to be added:
```bash
# Proposed (not yet configured)
npm run test             # Run all tests
npm run test:watch      # Watch mode
npm run test:ui         # UI browser dashboard
npm run test:coverage   # Generate coverage report
```

## Test File Organization

**Current structure:** Not applicable (no test files)

**Recommended approach for new tests:**

**Location:** Co-located pattern
- Tests next to source files (Vitest/Jest default)
- Store tests: `src/lib/historyStore.test.ts` next to `src/lib/historyStore.ts`
- Component tests: `src/components/DictationBar.test.tsx` next to `src/components/DictationBar.tsx`
- Utilities: `src/lib/tauri.test.ts` next to `src/lib/tauri.ts`

**Naming:**
- `*.test.ts` or `*.test.tsx` (matches TypeScript file type)
- Not `*.spec.ts` (codebase uses test terminology)

**Structure:**
```
src/
├── components/
│   ├── DictationBar.tsx
│   ├── DictationBar.test.tsx
│   └── ...
├── lib/
│   ├── historyStore.ts
│   ├── historyStore.test.ts
│   └── ...
└── types/
    └── index.ts
```

## Test Structure

**Proposed suite organization (based on codebase patterns):**

```typescript
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useHistoryStore } from './historyStore';

describe('useHistoryStore', () => {
  beforeEach(() => {
    // Reset store before each test
    vi.clearAllMocks();
  });

  describe('loadHistory', () => {
    it('should load history entries from Rust backend', async () => {
      // Test implementation
    });

    it('should set isLoading state correctly', async () => {
      // Test implementation
    });
  });

  describe('error handling', () => {
    it('should set error state on invoke failure', async () => {
      // Test implementation
    });
  });
});
```

**Patterns to follow:**

- **Setup:** `beforeEach()` to reset mocks and state
- **Teardown:** `afterEach()` for cleanup (Vitest auto-cleans in most cases)
- **Assertions:** Use `expect()` API, matching existing TypeScript style
- **Group related tests:** Describe blocks for features/methods

## Mocking

**Framework:** Vitest has built-in mocking via `vi` (from Vitest)

**Patterns:**

Mock Tauri invoke calls:
```typescript
import { describe, it, expect, vi } from 'vitest';
import * as tauriCore from '@tauri-apps/api/core';

describe('historyStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(tauriCore, 'invoke').mockResolvedValue({
      entries: [],
      total: 0,
    });
  });

  it('should invoke get_history with correct parameters', async () => {
    const { result } = renderHook(() => useHistoryStore());
    await act(async () => {
      await result.current.loadHistory();
    });
    expect(tauriCore.invoke).toHaveBeenCalledWith('get_history', {
      limit: 50,
      offset: 0,
    });
  });
});
```

Mock React components:
```typescript
vi.mock('./components/DictationBar', () => ({
  DictationBar: ({ isRecording }) => (
    <div data-testid="dictation-bar">{isRecording ? 'Recording' : 'Idle'}</div>
  ),
}));
```

**What to Mock:**
- Tauri `invoke()` calls — return predictable data
- Event listeners (`listen()`) — simulate events
- Component children or external dependencies
- Time-dependent functions (`setTimeout`, `Date.now()`)

**What NOT to Mock:**
- Store implementation details — test the public API
- Zustand state management — use real stores in tests
- TypeScript type definitions
- Internal utility functions (test them directly)

## Fixtures and Factories

**Pattern (proposed):**

Create test data factories in `src/__tests__/fixtures/` or alongside tests:

```typescript
// src/lib/historyStore.test.ts or src/__tests__/fixtures/history.ts
export const createMockTranscriptionEntry = (overrides = {}): TranscriptionEntry => ({
  id: 'test-id-123',
  text: 'Hello world',
  word_count: 2,
  duration_ms: 500,
  timestamp: new Date().toISOString(),
  synced: false,
  ...overrides,
});

export const mockHistoryResponse = {
  entries: [
    createMockTranscriptionEntry(),
    createMockTranscriptionEntry({ text: 'Second entry' }),
  ],
  totalCount: 2,
};
```

**Location:** Test files themselves or dedicated `__tests__/fixtures/` directory

**Usage:**
```typescript
it('should filter silent audio', () => {
  const silent = createMockTranscriptionEntry({ text: '' });
  expect(isSilentAudio(silent.text)).toBe(true);
});
```

## Coverage

**Requirements:** Not enforced (no coverage reporting configured)

**View Coverage (proposed):**
```bash
npm run test:coverage   # Would generate coverage/ directory with HTML report
npm run test:ui         # Browser-based coverage dashboard
```

**Target (recommended):**
- Store logic: >= 80% coverage (critical path)
- Components: >= 60% coverage (visual logic harder to test)
- Utilities: >= 90% coverage (pure functions)
- Tauri integration: minimal (mocked, hard to test real invocation)

## Test Types

**Unit Tests:**

Scope: Individual functions, stores, or small components
- Store methods: Test state mutations, invoke parameters, error handling
- Utilities: Test transformation, validation logic
- Examples: `historyStore.test.ts`, `tauri.test.ts`

Approach:
```typescript
it('should delete entry from store', async () => {
  const { result } = renderHook(() => useHistoryStore());
  // Setup initial state
  act(() => {
    result.current.entries = [{ id: '1', text: 'Test' }];
  });
  // Call action
  await act(async () => {
    await result.current.deleteEntry('1');
  });
  // Assert
  expect(result.current.entries).toHaveLength(0);
});
```

**Integration Tests:**

Scope: Multiple components or store + Tauri interaction
- Component + store interaction
- Event listeners + state updates
- Mock Tauri commands, test the flow

Approach:
```typescript
it('should load history on mount and display entries', async () => {
  vi.spyOn(tauriCore, 'invoke').mockResolvedValue({
    /* mock data */
  });
  render(<HistoryPage />);
  await waitFor(() => {
    expect(screen.getByText(/test entry/i)).toBeInTheDocument();
  });
});
```

**E2E Tests:**

Status: Not configured

Would require end-to-end test framework (Playwright, Cypress). Not typically used with Tauri desktop apps at this stage.

## Common Patterns

**Async Testing:**

```typescript
import { describe, it, expect } from 'vitest';

it('should handle async state updates', async () => {
  const { result } = renderHook(() => useHistoryStore());

  // Set loading state
  expect(result.current.isLoading).toBe(false);

  // Trigger async action
  const promise = result.current.loadHistory();
  expect(result.current.isLoading).toBe(true); // Optimistic update

  // Wait for completion
  await promise;
  expect(result.current.isLoading).toBe(false);
  expect(result.current.entries).toHaveLength(1);
});
```

Pattern:
- Mock `invoke()` to return resolved promise
- Use `await act(async () => { ... })` to apply state updates
- Wait for state changes with `waitFor()` or direct await

**Error Testing:**

```typescript
it('should handle invoke errors', async () => {
  vi.spyOn(tauriCore, 'invoke').mockRejectedValue(new Error('Network error'));

  const { result } = renderHook(() => useHistoryStore());

  await act(async () => {
    await result.current.loadHistory();
  });

  expect(result.current.error).toBe('Error: Network error');
  expect(result.current.entries).toEqual([]);
});
```

Pattern:
- Use `mockRejectedValue()` to simulate errors
- Assert error state is set
- Assert fallback state (empty arrays, null values)
- Verify error is logged (if using `console.error`)

**Component Testing:**

```typescript
import { render, screen } from '@testing-library/react';

it('should show recording state', () => {
  render(<DictationBar isRecording={true} isProcessing={false} />);
  expect(screen.getByTestId('waveform')).toBeInTheDocument();
});

it('should show error message', () => {
  render(<DictationBar error="Mic busy" />);
  expect(screen.getByText('Mic busy')).toBeInTheDocument();
});
```

Pattern:
- Use `render()` for components
- Query with `screen.getByTestId()`, `screen.getByText()`, etc.
- Assert rendered output matches props
- Use `data-testid` attributes in components for reliable selection

## Suggested Setup Path

**Phase 1: Add test infrastructure**
1. Install Vitest: `npm install -D vitest @vitest/ui`
2. Install testing utilities: `npm install -D @testing-library/react @testing-library/jest-dom`
3. Create `vitest.config.ts` in project root
4. Add test scripts to `package.json`

**Phase 2: Test critical paths**
1. Zustand stores (historyStore, dictionaryStore, settingsStore)
2. Tauri integration layer (`src/lib/tauri.ts`)
3. Event handling in `App.tsx`

**Phase 3: Component tests**
1. DictationBar (animation, state rendering)
2. Dashboard pages (state, navigation)
3. Forms (validation, submission)

---

*Testing analysis: 2026-02-24*
