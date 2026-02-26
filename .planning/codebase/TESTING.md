# Testing Patterns

**Analysis Date:** 2026-02-26

## Test Framework

**Runner:**
- Not explicitly configured in project
- No `jest.config.*`, `vitest.config.*`, or test runner dependency found in `package.json`
- Testing infrastructure: **Not detected** in frontend

**Assertion Library:**
- Not detected

**Rust Testing:**
- Built-in Rust test framework via `#[test]` attribute
- Run command: `cargo test`
- Located in source files alongside implementation code

**Run Commands:**
- Frontend: No test runner configured
- Rust backend: `cargo test` (standard Rust testing)
- No npm scripts for running tests in `package.json`

## Test File Organization

**Location:**
- **Frontend:** No test files found in repository
- **Rust:** Tests are co-located with implementation code using `#[cfg(test)]` module gates

**Naming:**
- Rust: Test functions named `test_*` (e.g., `test_capitalize_sentences`, `test_process_text_enabled`)

**Structure:**
- No separate test directories
- Tests embedded in source files at bottom of module

## Test Structure

**Rust Test Organization:**

```rust
// From src-tauri/src/text/mod.rs
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

**Patterns:**
- Each test is a separate `#[test]` function
- Multiple assertions per test function (test multiple scenarios in one function)
- Direct imports via `use super::*;` to access functions being tested
- Simple `assert_eq!` for comparing expected vs. actual output

## Mocking

**Framework:** Not detected for frontend

**Rust Testing:**
- No mocking framework detected (no `mockito` or similar in dependencies)
- Pure unit tests of deterministic functions (e.g., text capitalization)
- Tests call functions directly without mocking external dependencies

**What to Mock:**
- Frontend: Not applicable (no test framework)
- Rust: For integration tests requiring external resources (file I/O, network), tests would need to use `#[ignore]` or be skipped in CI

**What NOT to Mock:**
- Internal pure functions are tested directly without mocks

## Fixtures and Factories

**Test Data:**
- No explicit test fixtures or factories detected
- Test data created inline in test assertions:

```rust
// Direct string literals in assertions
assert_eq!(
    capitalize_sentences("hello. how are you"),
    "Hello. How are you"
);
```

**Location:**
- Not applicable; no dedicated fixture infrastructure

## Coverage

**Requirements:** Not enforced

**View Coverage:**
- No coverage tooling configured
- Frontend: No test infrastructure to measure coverage
- Rust: Can run with `cargo tarpaulin` (not configured by default)

## Test Types

**Unit Tests:**
- **Scope:** Individual pure functions
- **Approach:** Direct function calls with assertions
- **Example:** `src-tauri/src/text/mod.rs` tests text processing functions (`capitalize_sentences`, `process_text`)
- **Current coverage:** Only one module (`text`) has unit tests

**Integration Tests:**
- **Scope:** Not detected
- **Approach:** No integration test infrastructure found

**E2E Tests:**
- **Framework:** Not used
- **Rationale:** Tauri app requires macOS/platform-specific UI testing; manual testing likely used instead

## Common Patterns

**Async Testing:**
- Not applicable for TypeScript (no test framework configured)
- Rust async testing: Not present in codebase (async functions exist but no tests)

**Error Testing:**
- Not detected in existing tests
- For Rust, errors can be tested with `#[should_panic]` attribute (not currently used)

```rust
// Pattern that could be used but isn't:
#[test]
#[should_panic(expected = "assertion failed")]
fn test_error_case() {
    panic!("assertion failed");
}
```

## Missing Testing Infrastructure

**Frontend:**
- No test runner (Jest, Vitest, etc.)
- No test files for React components
- Components like `DictationBar.tsx`, `Settings.tsx` are untested
- Zustand stores in `src/lib/store.ts`, `src/lib/historyStore.ts`, etc. are untested

**Rust Backend:**
- Only one module (`text/mod.rs`) has tests
- Large modules like `src-tauri/src/audio/capture.rs`, `src-tauri/src/transcription/whisper.rs` are untested
- Settings I/O, history management, dictionary operations have no tests

**Recommendations for Adding Tests:**

1. **Frontend Testing Setup:**
   - Add Vitest or Jest to devDependencies
   - Create `__tests__` or `.test.tsx` files next to components
   - Test React components with @testing-library/react

2. **Rust Backend Testing:**
   - Add unit tests for pure functions in each module
   - Test error paths for settings I/O
   - Add integration tests for Tauri command handlers

3. **Test File Locations (proposed):**
   - TypeScript: `src/components/__tests__/DictationBar.test.tsx`
   - TypeScript: `src/lib/__tests__/store.test.ts`
   - Rust: Inline tests in each `.rs` file under `#[cfg(test)]` module

---

*Testing analysis: 2026-02-26*
