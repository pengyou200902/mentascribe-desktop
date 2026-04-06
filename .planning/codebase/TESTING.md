# Testing Strategy

**Analysis Date:** 2026-04-06

## Test Framework

**Frontend (TypeScript/React):**
- **No test framework is configured.** No Jest, Vitest, or any test runner exists in `package.json` dependencies or devDependencies.
- No test configuration files (`jest.config.*`, `vitest.config.*`, `playwright.config.*`) exist.
- No test-related npm scripts in `package.json`.
- Zero frontend test files exist in the repository.

**Rust Backend:**
- Built-in Rust test framework via `#[test]` attribute
- No additional test dependencies (no `mockito`, `proptest`, `rstest`, `tempfile` etc. in `src-tauri/Cargo.toml`)
- Tests are co-located with source code using `#[cfg(test)]` module gates

## Test File Organization

**Frontend:** No tests exist. No test directory structure.

**Rust backend:**
- Tests live at the bottom of source files inside `#[cfg(test)] mod tests { ... }`
- Only one module has tests: `src-tauri/src/text/mod.rs`

**Location pattern for Rust:**
```
src-tauri/src/
├── text/
│   └── mod.rs          # Has 3 tests (capitalize_sentences, process_text_disabled, process_text_enabled)
├── audio/
│   ├── capture.rs      # No tests
│   └── vad.rs          # No tests
├── transcription/
│   ├── whisper.rs      # No tests
│   ├── cloud.rs        # No tests (stubs only)
│   └── voxtral.rs      # No tests
├── settings/mod.rs     # No tests
├── history/mod.rs      # No tests
├── dictionary/mod.rs   # No tests
├── stats/mod.rs        # No tests
├── injection/mod.rs    # No tests
├── hotkey/mod.rs       # No tests
└── api/client.rs       # No tests
```

## Test Structure

**Rust unit test pattern (the only pattern in use):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_sentences() {
        assert_eq!(
            capitalize_sentences("hello world"),
            "Hello world"
        );
        assert_eq!(
            capitalize_sentences("hello. how are you"),
            "Hello. How are you"
        );
        assert_eq!(
            capitalize_sentences("hello! what's up? not much"),
            "Hello! What's up? Not much"
        );
    }

    #[test]
    fn test_process_text_disabled() {
        assert_eq!(
            process_text("hello world", false),
            "hello world"
        );
    }

    #[test]
    fn test_process_text_enabled() {
        assert_eq!(
            process_text("hello world", true),
            "Hello world"
        );
    }
}
```

**Characteristics of existing tests:**
- No setup/teardown (no test fixtures)
- No mocking framework
- Multiple assertions per test function (testing multiple scenarios in one test)
- Pure function testing only (no I/O, no file system, no network)
- No async test support used
- Inline test data (no external fixtures)

## Test Types

**Unit Tests (Rust only):**
- Scope: Individual pure functions (text processing)
- Location: `src-tauri/src/text/mod.rs`
- Approach: Direct function calls with `assert_eq!` assertions

**Integration Tests:** None exist for either frontend or backend.

**E2E Tests:** None exist. No Playwright, Cypress, WebdriverIO, or Tauri-specific E2E framework.

**Manual Testing:** All testing is currently manual via running the app with `npm run tauri dev`.

## Running Tests

```bash
# Rust backend tests (the only tests that exist)
cd src-tauri && cargo test

# Frontend - NO test commands exist
# npm run test       <-- does not exist
# npm run lint       <-- linting only, not testing
# npm run typecheck  <-- type checking only, not testing
```

## Coverage

**Requirements:** No coverage targets enforced.

**Current state:**
- Frontend: 0% (no test infrastructure at all)
- Rust: Only `src-tauri/src/text/mod.rs` has tests (3 tests covering `capitalize_sentences` and `process_text`). All other modules have zero test coverage.

**Coverage tooling:** Not configured. Could use `cargo tarpaulin` for Rust or `cargo llvm-cov`.

## CI/CD Integration

**No CI/CD pipeline exists.** No `.github/workflows/`, no `.gitlab-ci.yml`, no `Jenkinsfile`, or any other CI configuration detected in the project root.

Tests are run manually (if at all) via `cargo test` in the `src-tauri/` directory.

## Mocking

**No mocking frameworks are in use.**

**Rust:** No `mockall`, `mockito`, or similar crate in `Cargo.toml`.

**Frontend:** No `vi.mock`, `jest.mock`, or MSW configured.

## Untested Areas (Prioritized)

**High priority -- pure logic that is easily unit-testable:**
- `src-tauri/src/settings/mod.rs`: Settings serialization/deserialization, default values, path resolution
- `src-tauri/src/history/mod.rs`: History CRUD, pagination logic, entry truncation (keeps last 500)
- `src-tauri/src/dictionary/mod.rs`: Dictionary entry management, toggle logic, cache invalidation
- `src-tauri/src/stats/mod.rs`: Stats calculation, streak logic, daily aggregation
- `src-tauri/src/audio/vad.rs`: Voice activity detection energy thresholds, state machine transitions

**Medium priority -- Zustand stores (would need Vitest + mocked Tauri invoke):**
- `src/lib/store.ts`: Settings load/update flow
- `src/lib/historyStore.ts`: Pagination, optimistic deletes, concurrent load guards
- `src/lib/dictionaryStore.ts`: CRUD operations, toggle logic
- `src/lib/statsStore.ts`: Stats loading, default fallback on error

**Lower priority -- UI components (would need @testing-library/react):**
- `src/components/DictationBar.tsx`: State transitions (idle/recording/processing/error), waveform animation
- `src/components/dashboard/HomePage.tsx`: Stats display, date grouping logic
- `src/components/dashboard/HistoryPage.tsx`: Copy/delete interactions, load-more pagination
- `src/components/dashboard/DictionaryPage.tsx`: Modal flow, entry editing, vocabulary vs auto-correct modes
- `src/components/dashboard/SettingsPage.tsx`: Form state, model download flow, engine switching

## Recommendations for Adding Tests

**1. Add Vitest for frontend testing:**
```bash
npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
```
Add to `vite.config.ts`:
```typescript
/// <reference types="vitest" />
export default defineConfig({
  // ...existing config
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
  },
});
```
Add to `package.json` scripts:
```json
"test": "vitest",
"test:run": "vitest run",
"test:coverage": "vitest run --coverage"
```

**2. Mock Tauri invoke for store tests:**
```typescript
// src/test/setup.ts
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
```

**3. Zustand store test pattern:**
```typescript
// src/lib/__tests__/historyStore.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useHistoryStore } from '../historyStore';

vi.mocked(invoke).mockResolvedValue([]);

describe('useHistoryStore', () => {
  beforeEach(() => {
    useHistoryStore.setState({
      entries: [],
      totalCount: 0,
      isLoading: false,
      hasMore: true,
      error: null,
    });
    vi.clearAllMocks();
  });

  it('loads history entries', async () => {
    const mockEntries = [{
      id: '1', text: 'hello', word_count: 1,
      duration_ms: 500, timestamp: '2026-01-01', synced: false
    }];
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockEntries)
      .mockResolvedValueOnce(1);

    await useHistoryStore.getState().loadHistory();

    expect(useHistoryStore.getState().entries).toEqual(mockEntries);
    expect(useHistoryStore.getState().isLoading).toBe(false);
  });

  it('guards against concurrent loads', async () => {
    useHistoryStore.setState({ isLoading: true });
    await useHistoryStore.getState().loadHistory();
    expect(invoke).not.toHaveBeenCalled();
  });
});
```

**4. Rust test expansion pattern:**
```rust
// Example: Add to src-tauri/src/settings/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = UserSettings::default();
        assert_eq!(settings.widget.opacity, 1.0);
        assert!(!settings.widget.draggable);
        assert!(!settings.cleanup.enabled);
    }

    #[test]
    fn test_settings_roundtrip() {
        let settings = UserSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.widget.opacity, settings.widget.opacity);
    }
}
```

```rust
// Example: Add to src-tauri/src/audio/vad.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        let mut vad = VoiceActivityDetector::new(VadConfig::default());
        let silence = vec![0.0f32; 1600]; // 100ms of silence
        assert!(!vad.process(&silence));
    }

    #[test]
    fn test_speech_detection() {
        let mut vad = VoiceActivityDetector::new(VadConfig::default());
        let speech: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        // Need enough consecutive speech frames to trigger
        for _ in 0..3 {
            vad.process(&speech);
        }
        assert!(vad.process(&speech));
    }
}
```

**5. Test file naming and location:**
- Frontend: co-located `*.test.ts` / `*.test.tsx` files next to source (e.g., `src/lib/historyStore.test.ts`)
- Rust: inline `#[cfg(test)] mod tests` at bottom of each module file (standard Rust convention)

**6. Priority order for adding tests:**
1. Rust pure functions first (cheapest, highest value): `text`, `vad`, `settings`, `dictionary`
2. Zustand stores second (medium effort): `historyStore`, `dictionaryStore`, `statsStore`
3. React components third (highest effort): dashboard pages
4. E2E tests last (requires Tauri test harness or WebDriver setup)

---

*Testing analysis: 2026-04-06*
