# Testing Strategy

**Analysis Date:** 2026-04-05

## Test Framework

**Frontend (TypeScript/React):**
- **No test framework is configured.** No Jest, Vitest, or any test runner exists in `package.json` dependencies or devDependencies.
- No test configuration files (`jest.config.*`, `vitest.config.*`, `playwright.config.*`) exist.
- No test-related npm scripts in `package.json`.
- Zero frontend test files exist in the repository.

**Rust Backend:**
- Built-in Rust test framework via `#[test]` attribute
- No additional test dependencies (no `mockito`, `proptest`, `rstest`, etc. in `src-tauri/Cargo.toml`)
- Tests are co-located with source code using `#[cfg(test)]` module gates

## Test Structure

**Frontend:** No tests exist. No test directory structure.

**Rust backend test organization:**
- Tests live at the bottom of source files inside `#[cfg(test)] mod tests { ... }`
- Only one module has tests: `src-tauri/src/text/mod.rs`
- Pattern:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_function_name() {
          assert_eq!(actual, expected);
      }
  }
  ```

## Test Types

**Unit Tests (Rust only):**
- Scope: Individual pure functions (text processing)
- Location: `src-tauri/src/text/mod.rs`
- Approach: Direct function calls with `assert_eq!` assertions
- Example:
  ```rust
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
  ```

**Integration Tests:** None exist for either frontend or backend.

**E2E Tests:** None exist. No Playwright, Cypress, or Tauri-specific E2E framework.

## Coverage

**Requirements:** No coverage targets enforced.

**Current state:**
- Frontend: 0% (no test infrastructure)
- Rust: Only `src-tauri/src/text/mod.rs` has any tests. All other modules (audio, transcription, settings, history, dictionary, stats, injection, hotkey, api) have zero test coverage.

**Coverage tooling:** Not configured. Could use `cargo tarpaulin` for Rust.

## Test Patterns

**Rust unit test pattern (the only pattern in use):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange: inline test data
        let input = "hello world";

        // Act + Assert combined
        assert_eq!(function_under_test(input, true), "Hello world");
    }
}
```

**Characteristics:**
- No setup/teardown (no `#[cfg(test)]` fixtures)
- No mocking framework
- Multiple assertions per test function (testing multiple scenarios)
- Pure function testing only (no I/O, no file system, no network)
- No async test support used

## Running Tests

```bash
# Rust backend tests (the only tests that exist)
cd src-tauri && cargo test

# Frontend - NO test commands exist
# npm run test    <-- does not exist
# npm run lint    <-- linting only, not testing
# npm run typecheck  <-- type checking only
```

## CI/CD Testing

**No CI/CD pipeline exists.** No `.github/workflows/`, no `.gitlab-ci.yml`, no `Jenkinsfile`, or any other CI configuration detected.

Tests are run manually (if at all) via `cargo test` in the `src-tauri/` directory.

## Untested Areas (Prioritized)

**High priority -- pure logic that is easily testable:**
- `src-tauri/src/settings/mod.rs`: Settings serialization/deserialization, default value logic
- `src-tauri/src/history/mod.rs`: History CRUD operations, pagination logic
- `src-tauri/src/dictionary/mod.rs`: Dictionary entry management, toggle logic
- `src-tauri/src/stats/mod.rs`: Stats calculation, streak logic, daily aggregation
- `src-tauri/src/audio/vad.rs`: Voice activity detection thresholds

**Medium priority -- Zustand stores (would need Vitest + mocked Tauri invoke):**
- `src/lib/store.ts`: Settings load/update
- `src/lib/historyStore.ts`: Pagination, optimistic updates, guard clauses
- `src/lib/dictionaryStore.ts`: CRUD operations, toggle logic
- `src/lib/statsStore.ts`: Stats loading, default fallback

**Lower priority -- UI components (would need @testing-library/react):**
- `src/components/DictationBar.tsx`: State transitions, waveform rendering
- `src/components/dashboard/HomePage.tsx`: Stats display, date grouping
- `src/components/dashboard/HistoryPage.tsx`: Copy/delete interactions
- `src/components/dashboard/DictionaryPage.tsx`: Modal flow, entry editing

## Recommendations for Adding Tests

**1. Add Vitest for frontend testing:**
```bash
npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
```
Add to `vite.config.ts`:
```typescript
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
  });

  it('loads history entries', async () => {
    const mockEntries = [{ id: '1', text: 'hello', word_count: 1, duration_ms: 500, timestamp: '2026-01-01', synced: false }];
    vi.mocked(invoke).mockResolvedValueOnce(mockEntries).mockResolvedValueOnce(1);

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
// Add to src-tauri/src/history/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_add_entry_to_empty_history() {
        // Use temp directory for isolation
        let entry = add_entry_to_data(
            &mut HistoryData::default(),
            "Hello world".to_string(),
            2,
            1500,
        );
        assert_eq!(entry.text, "Hello world");
        assert_eq!(entry.word_count, 2);
    }
}
```

**5. Test file naming and location:**
- Frontend: co-located `*.test.ts` / `*.test.tsx` files next to source (e.g., `src/lib/historyStore.test.ts`)
- Rust: inline `#[cfg(test)] mod tests` at bottom of each module file

---

*Testing analysis: 2026-04-05*
