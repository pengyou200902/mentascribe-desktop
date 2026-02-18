# Testing Patterns

**Analysis Date:** 2026-02-18

## Test Framework

**Runner:**
- Not configured in current codebase
- No test files present (0 `.test.ts` or `.spec.ts` files found)
- No test runner in `package.json` scripts (no jest/vitest/mocha config)

**Assertion Library:**
- Not configured

**Run Commands:**
- Testing not currently set up
- Recommendation: Jest or Vitest should be configured for future tests

## Test File Organization

**Location:**
- Currently no test files; recommend co-located with source
- Pattern when implemented: `src/components/__tests__/Component.test.tsx` or `src/components/Component.test.tsx`

**Naming:**
- Not yet established; recommend: `[filename].test.ts` or `[filename].spec.ts`

**Structure:**
- To be defined; recommend standard describe/test blocks

## Test Coverage

**Requirements:** None enforced

**Current State:**
- No test infrastructure
- No coverage reporting configured

## Testing Approach (Recommendations)

Given the codebase structure, testing should focus on these key areas:

### 1. Store Tests (Zustand)
The following stores should be tested:
- `src/lib/store.ts` - Core settings store with `loadSettings()` and `updateSettings()`
- `src/lib/historyStore.ts` - History management with pagination
- `src/lib/dictionaryStore.ts` - Dictionary CRUD operations
- `src/lib/statsStore.ts` - Statistics loading

**Pattern to follow:**
```typescript
// Example test structure for Zustand store (not yet implemented)
import { renderHook, act } from '@testing-library/react';
import { useStore } from '../lib/store';

describe('useStore', () => {
  it('should load settings', async () => {
    const { result } = renderHook(() => useStore());

    await act(async () => {
      await result.current.loadSettings();
    });

    expect(result.current.settings).toBeDefined();
    expect(result.current.isLoading).toBe(false);
  });
});
```

### 2. Component Tests
Priority components for testing:
- `src/components/DictationBar.tsx` - Complex animation and state logic
  - Test waveform animation states (idle, recording, processing)
  - Test error display
  - Test mouse tracking and hover states
- `src/components/Settings.tsx` - Settings form with model download
  - Test form state updates
  - Test model selection and download
- `src/App.tsx` - Main application logic
  - Test window type detection (dictation/settings/history/dashboard)
  - Test hotkey event handling (toggle vs hold mode)
  - Test recording lifecycle

### 3. Utility Functions
Functions that should be tested:
- `src/lib/tauri.ts` - All async wrapper functions
- `src/lib/theme.tsx` - Theme switching and storage logic
- Error handling utilities (type narrowing for unknown errors)

## Async Testing Pattern (To Be Implemented)

Based on the codebase's use of async operations with Zustand and Tauri:

```typescript
// Example pattern for async store testing
describe('Async Store Operations', () => {
  it('should handle async state updates', async () => {
    const { result } = renderHook(() => useHistoryStore());

    await act(async () => {
      await result.current.loadHistory();
    });

    expect(result.current.entries).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it('should handle async errors', async () => {
    const { result } = renderHook(() => useHistoryStore());

    await act(async () => {
      await result.current.loadHistory();
    });

    // Verify error state set on failure
    if (result.current.error) {
      expect(result.current.isLoading).toBe(false);
    }
  });
});
```

## Error Testing Pattern (To Be Implemented)

The codebase has consistent error handling that should be tested:

```typescript
// Example error test pattern
describe('Error Handling', () => {
  it('should handle unknown error types', () => {
    const unknownError = new Error('Test error');
    const result = unknownError instanceof Error
      ? unknownError.message
      : String(unknownError);

    expect(result).toBe('Test error');
  });

  it('should timeout error messages', async () => {
    const { result } = renderHook(() => {
      const [error, setError] = useState<string | null>(null);
      return { error, setError };
    });

    act(() => {
      result.current.setError('Test error');
    });

    expect(result.current.error).toBe('Test error');

    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 5100));
    });

    // Error should have timed out
  });
});
```

## Mocking Patterns (To Be Implemented)

### Mock Tauri Invocations
Since the app heavily relies on `@tauri-apps/api/core` invoke:

```typescript
// Mock pattern for Tauri invoke
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('Tauri Commands', () => {
  beforeEach(() => {
    (invoke as jest.Mock).mockClear();
  });

  it('should start recording', async () => {
    (invoke as jest.Mock).mockResolvedValue(undefined);

    await startRecording();

    expect(invoke).toHaveBeenCalledWith('start_recording');
  });

  it('should handle recording errors', async () => {
    (invoke as jest.Mock).mockRejectedValue(new Error('Recording failed'));

    await expect(startRecording()).rejects.toThrow('Recording failed');
  });
});
```

### Mock Event Listeners
For testing event-driven features:

```typescript
jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn(),
}));

describe('Event Listeners', () => {
  it('should listen to hotkey events', async () => {
    const mockListener = jest.fn();
    (listen as jest.Mock).mockImplementation((event, callback) => {
      if (event === 'hotkey-pressed') {
        mockListener(callback);
      }
      return Promise.resolve(() => {});
    });

    // Test hotkey handling
  });
});
```

### Mock Zustand Stores
For component testing:

```typescript
jest.mock('../lib/store', () => ({
  useStore: () => ({
    settings: {
      hotkey: { mode: 'toggle' },
      transcription: { model_size: 'small' },
    },
    loadSettings: jest.fn(),
  }),
}));
```

## What NOT to Mock

- React hooks (use actual hooks with renderHook)
- localStorage (use real localStorage or mock sparingly)
- Component rendering (test actual output, not mocks)

## Fixtures and Factories (To Be Implemented)

Create test data builders in `src/__tests__/factories/`:

```typescript
// src/__tests__/factories/userSettings.ts
export function createUserSettings(overrides?: Partial<UserSettings>): UserSettings {
  return {
    transcription: {
      provider: 'whisper',
      language: 'en',
      model_size: 'small',
      ...overrides?.transcription,
    },
    cleanup: {
      enabled: false,
      remove_filler: false,
      add_punctuation: false,
      format_paragraphs: false,
      ...overrides?.cleanup,
    },
    hotkey: {
      key: 'F6',
      mode: 'toggle',
      ...overrides?.hotkey,
    },
    output: {
      insert_method: 'paste',
      auto_capitalize: true,
      ...overrides?.output,
    },
  };
}

export function createTranscriptionEntry(overrides?: Partial<TranscriptionEntry>): TranscriptionEntry {
  return {
    id: crypto.randomUUID(),
    text: 'Test transcription',
    word_count: 2,
    duration_ms: 1000,
    timestamp: new Date().toISOString(),
    synced: false,
    ...overrides,
  };
}
```

## Integration Testing Considerations

Since this is a Tauri desktop app with Rust backend:
- Tauri command integration: Mock `invoke()` calls in unit tests
- For integration tests: Run against actual Tauri backend in test mode
- Event flow testing: Mock Tauri events but test React state updates

## Missing Test Infrastructure

To implement testing, add to `package.json`:

```json
{
  "devDependencies": {
    "@testing-library/react": "^14.0.0",
    "@testing-library/jest-dom": "^6.0.0",
    "@testing-library/user-event": "^14.0.0",
    "@types/jest": "^29.5.0",
    "jest": "^29.7.0",
    "jest-environment-jsdom": "^29.7.0",
    "ts-jest": "^29.1.0"
  },
  "scripts": {
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage"
  }
}
```

And create `jest.config.js`:

```javascript
export default {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',
  roots: ['<rootDir>/src'],
  testMatch: ['**/__tests__/**/*.ts?(x)', '**/?(*.)+(spec|test).ts?(x)'],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx'],
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/main.tsx',
  ],
};
```

---

*Testing analysis: 2026-02-18*
