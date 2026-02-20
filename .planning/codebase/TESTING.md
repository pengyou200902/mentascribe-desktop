# Testing Patterns

**Analysis Date:** 2026-02-20

## Test Framework

**Runner:**
- Rust: `cargo test` (built-in Rust testing)
- TypeScript/React: Not configured — no Jest, Vitest, or other JavaScript test runner

**Assertion Library:**
- Rust: `assert_eq!()`, `assert!()` macros (standard library)
- TypeScript/React: Not applicable (no test framework)

**Run Commands:**
```bash
# Rust tests
cargo test                 # Run all Rust tests
cargo test -- --nocapture # Run with println! output visible
cargo test text::tests     # Run specific test module

# TypeScript type checking
npm run typecheck         # Type check all TypeScript files (no runtime tests)

# Linting (code quality, not testing)
npm run lint              # Check for linting errors
npm run lint:fix          # Auto-fix linting errors

# Formatting
npm run format            # Format code with Prettier
```

## Test File Organization

**Location:**
- Rust: Co-located with implementation
  - Pattern: `#[cfg(test)] mod tests { ... }` in same `.rs` file as implementation
  - Example: `src-tauri/src/text/mod.rs` contains `mod tests` at bottom

**Naming:**
- Rust test modules: `tests { ... }` block
- Rust test functions: `#[test] fn test_<feature>() { ... }`
- Pattern: `test_capitalize_sentences()`, `test_process_text_enabled()`

**Structure:**
```
src-tauri/src/
├── text/
│   └── mod.rs (contains implementation + #[cfg(test)] mod tests)
├── audio/
│   └── capture.rs (no tests currently)
└── [other modules]
```

## Test Structure

**Rust Test Suite Organization:**

From `src-tauri/src/text/mod.rs`:
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

**Patterns:**
- Each test function tests one scenario or feature
- Input-output assertions verify expected behavior
- Multiple assertions within single test for related conditions
- `use super::*;` imports private functions for testing

## Mocking

**Framework:** Not used in current codebase

**Patterns:**
- Rust: No mocking library detected (no `mockall`, `mock`, or `double` dependencies)
- TypeScript: No mocking framework configured (no Jest mocks, Vitest mocks, or Sinon)

**Current Approach:**
- Unit tests in Rust test pure functions directly
- Integration with external APIs (Tauri invokes, HTTP requests) not tested in automated tests
- Manual testing of frontend-backend integration via app execution

**What to Mock (if testing framework added):**
- Tauri `invoke()` calls in React components
- API responses from cloud providers
- File I/O operations
- System audio/input/output

**What NOT to Mock:**
- Pure utility functions (test with actual implementation)
- String processing logic (test with real inputs)
- Data type conversions (use concrete examples)

## Fixtures and Factories

**Test Data:**
- Rust tests use inline literal values:
  ```rust
  #[test]
  fn test_capitalize_sentences() {
      assert_eq!(
          capitalize_sentences("hello world"),
          "Hello world"
      );
  }
  ```
- No shared fixtures or factories detected
- Test inputs are small, isolated strings and booleans

**Location:**
- Same file as tests, within `#[cfg(test)] mod tests { ... }`
- No separate fixture/factory files

## Coverage

**Requirements:** None enforced

**Current State:**
- Rust: Minimal coverage (only `text/mod.rs` contains tests)
  - Untested modules: `audio/`, `transcription/`, `injection/`, `hotkey/`, `history/`, `dictionary/`, `api/`, `settings/`
  - Reason: Complex platform-specific code (audio capture, FFI, NSPanel), cloud APIs
- TypeScript/React: No tests (no test runner configured)

**View Coverage:**
```bash
# Rust coverage requires additional setup (cargo-tarpaulin)
# Not currently configured in this project
```

**Modules with No Tests:**
- `src-tauri/src/audio/` - audio capture, complex CPAL integration
- `src-tauri/src/transcription/` - Whisper models, cloud APIs
- `src-tauri/src/injection/` - platform-specific text injection (macOS/Windows/Linux)
- `src-tauri/src/hotkey/` - global hotkey registration
- `src-tauri/src/settings/` - file I/O and serialization
- `src-tauri/src/api/` - HTTP client and auth
- `src-tauri/src/history/` - database operations
- `src-tauri/src/dictionary/` - data persistence
- `src/` (all React/TypeScript) - no test framework

## Test Types

**Unit Tests:**
- **Scope:** Single pure functions with no side effects
- **Approach:** Direct assertion of input-output pairs
- **Examples:** `text/mod.rs` tests for `capitalize_sentences()`, `process_text()`
- **Current coverage:** Only simple utility functions in text processing

**Integration Tests:**
- **Scope:** Interactions between modules or with external systems
- **Approach:** Not implemented; would require:
  - Mocking Tauri invokes in React components
  - Testing audio capture + transcription pipeline end-to-end
  - Testing settings persistence + loading
- **Needed for:** Verifying recording + transcription + injection flow

**E2E Tests:**
- **Framework:** Not used
- **Alternative:** Manual testing via running the app and using keyboard shortcuts
- **Coverage needed:** Full user workflows (record → transcribe → inject text)

## Common Patterns

**Rust Unit Test Pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_sentences() {
        // Arrange
        let input = "hello world";

        // Act
        let result = capitalize_sentences(input);

        // Assert
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_process_text_disabled() {
        // When processing is disabled, return input unchanged
        assert_eq!(
            process_text("hello world", false),
            "hello world"
        );
    }

    #[test]
    fn test_process_text_enabled() {
        // When processing is enabled, apply transformations
        assert_eq!(
            process_text("hello world", true),
            "Hello world"
        );
    }
}
```

**Assertion Examples:**

```rust
// Simple equality
assert_eq!(actual, expected);

// Boolean assertions
assert!(condition, "optional message");
assert!(!condition);

// String comparisons
assert_eq!(result, "expected string");

// Multiple assertions in one test (related scenarios)
assert_eq!(test_func("input1"), "output1");
assert_eq!(test_func("input2"), "output2");
```

**Async Testing in React (if Jest were added):**

```typescript
// Pattern that would be used
test('loads settings on mount', async () => {
  const { getByText } = render(<App />);
  await waitFor(() => {
    expect(getByText('loaded')).toBeInTheDocument();
  });
});
```

**Error Testing:**

```rust
#[test]
fn test_error_handling() {
    // Test that function returns appropriate error
    let result = some_fallible_function();
    assert!(result.is_err());
}
```

## Missing Test Coverage

**Critical Gaps:**

| Component | Location | Why Not Tested | Risk |
|-----------|----------|---|---|
| Audio capture | `src-tauri/src/audio/capture.rs` | CPAL integration, platform-specific | Breaks in microphone scenarios |
| Transcription pipeline | `src-tauri/src/transcription/` | Whisper model integration, large binary | Silent transcription failures |
| Text injection | `src-tauri/src/injection/mod.rs` | macOS/Windows/Linux system APIs | Text not pasting correctly |
| Settings persistence | `src-tauri/src/settings/mod.rs` | File I/O and JSON serialization | Settings lost on crash |
| React components | `src/components/` | No test runner; complex Tauri FFI | UI state sync issues |
| Event handling | `src/App.tsx` | Tauri event listeners, async orchestration | Race conditions in recording |
| History/Dictionary | `src-tauri/src/history/`, `dictionary/` | Database operations | Data corruption or loss |

**Recommended Testing Priority (if adding tests):**

1. **High:** Text processing utilities (already partially done) + settings persistence
2. **High:** Audio capture pipeline (mock CPAL for basic capture flow)
3. **Medium:** Settings load/save cycle with file I/O
4. **Medium:** Basic React component rendering with mocked Tauri
5. **Low:** Platform-specific injection (harder to mock, test manually)

## Adding Tests

**To add Jest/Vitest for React components:**

```bash
npm install --save-dev jest @testing-library/react @testing-library/jest-dom ts-jest
npm install --save-dev vitest (alternative to Jest)
```

**To add Rust test coverage:**

```bash
# View coverage with cargo-tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

**Test Command Structure (if framework added):**

```bash
# Would follow this pattern
npm test                  # Run all tests
npm test -- --watch      # Watch mode
npm test -- --coverage   # Coverage report
```

## Current Test Status Summary

| Language | Tests Present | Framework | Config | Recommendation |
|----------|---|---|---|---|
| Rust | Yes (minimal) | Built-in | None | Add for critical modules (settings, audio) |
| TypeScript/React | No | None | None | Add Jest/Vitest for components |
| E2E | No | None | None | Manual testing sufficient for now |

**Blocking Issues for Test Expansion:**
1. No JavaScript test framework configured (Jest/Vitest)
2. Rust tests require careful mocking of platform-specific APIs (CPAL, NSPanel, enigo)
3. Frontend-backend integration tests need Tauri mock layer
4. No existing test utilities or helpers to build upon

---

*Testing analysis: 2026-02-20*
