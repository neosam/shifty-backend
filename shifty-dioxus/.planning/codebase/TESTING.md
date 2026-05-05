# Testing Patterns

**Analysis Date:** 2026-05-05

## Test Framework

**Runner:**
- Cargo's built-in test runner (no custom framework)
- WASM-bindgen tests available via `wasm-bindgen-test = "0.3"` in dev-dependencies
- Can run tests with: `cargo test`

**Assertion Library:**
- Built-in Rust `assert!()`, `assert_eq!()`, `assert_ne!()` macros
- No external assertion framework (e.g., `pretty_assertions`) used

**Run Commands:**
```bash
cargo test              # Run all unit tests (non-WASM; most of shifty-dioxus)
cargo test --lib       # Run library tests only
cargo test --test '*'  # Run integration tests
cargo test -- --nocapture  # Show println!() output during tests
cargo test -- --test-threads=1  # Run tests serially for debugging
```

## Test File Organization

**Location:**
- Tests co-located in a dedicated `src/tests/` directory
- **NOT** using Rust's standard pattern of `#[cfg(test)] mod tests;` within source files
- Tests are in separate test modules, isolated from production code

**Directory Structure:**
```
src/
├── tests/
│   ├── i18n_tests.rs           # Translation and locale tests
│   ├── utils_tests.rs          # Utility function tests
│   ├── week_tests.rs           # Week calculation tests
│   ├── error_tests.rs          # Error handling tests
│   ├── integration_tests.rs    # Full-stack integration scenarios
│   ├── state_tests.rs          # State/signal tests
│   ├── service_tests.rs        # Service store and action tests
│   ├── api_tests.rs            # API client tests
│   ├── volunteer_work_tests.rs # Volunteer work domain logic
│   └── [mod.rs] (referenced but not visible)
└── [production code]
```

**Naming:**
- Test files use `_tests` suffix: `error_tests.rs`, `service_tests.rs`
- Test modules wrapped in `#[cfg(test)]` module blocks
- Test functions use `test_` prefix: `test_locale_variants()`, `test_i18n_key_translation()`, `test_error_display()`

## Test Structure

**Suite Organization:**
Each test file contains multiple `#[cfg(test)]` modules, one per feature area. Example from `src/tests/service_tests.rs`:

```rust
#[cfg(test)]
mod text_template_service_tests {
    use crate::service::text_template::*;
    use crate::state::text_template::{TextTemplate, TemplateEngine};
    use std::rc::Rc;
    use uuid::Uuid;

    #[test]
    fn test_text_template_store_default() {
        let store = TextTemplateStore::default();
        
        assert_eq!(store.templates.len(), 0);
        assert!(store.selected_template.is_none());
    }
}

#[cfg(test)]
mod billing_period_service_tests {
    // Tests for billing period service...
}
```

**Patterns:**
- **Setup:** Explicit object construction inline (no setup fixtures)
  ```rust
  let template = TextTemplate {
      id: Uuid::new_v4(),
      name: Some("Test Template".into()),
      template_type: "billing-period".into(),
      // ...
  };
  ```

- **Teardown:** None needed; test scope cleanup is automatic (no mocks, no connections)

- **Assertion pattern:** Direct equality and boolean checks
  ```rust
  assert_eq!(store.templates.len(), 0);
  assert!(store.selected_template.is_none());
  assert!(!formatted.is_empty());
  assert_eq!(result, Some("success".to_string()));
  ```

## Mocking

**Framework:**
- `mockito = "1.2"` available in dev-dependencies (but not heavily used)
- Most tests avoid mocking by testing in-memory data structures

**Patterns:**
- **Minimal mocking:** Tests construct real instances rather than mocking
- Example from `src/tests/service_tests.rs`:
  ```rust
  let store = TextTemplateStore::default();  // Use real store, not mock
  assert_eq!(store.templates.len(), 0);      // Test real behavior
  ```

**What to Mock:**
- External HTTP requests would be mocked if testing API layer (via `mockito`), but API tests currently use real objects
- For services that depend on `GlobalSignal` stores, tests create instances directly

**What NOT to Mock:**
- Data model structures (`TextTemplate`, `Employee`, `BillingPeriodTO`)
- Enum variants and action types
- Pure functions and business logic (validate in-place)
- State/signal behavior (test with real signal-like patterns)

## Fixtures and Factories

**Test Data:**
- No factory pattern or test fixture builder observed
- Test data created inline in test functions
- Example from `src/tests/service_tests.rs`:
  ```rust
  let employee = Employee {
      id: Uuid::new_v4(),
      // Set fields directly...
  };
  ```

**Location:**
- Fixtures and test helpers not extracted; all test data defined within test functions
- No separate `fixtures/` or `test_helpers/` module

**Approach:**
- Inline construction preferred for simplicity and clarity
- For complex objects, direct field initialization used

## Coverage

**Requirements:**
- No coverage requirements or CI enforcement observed
- Coverage tooling not configured (no `tarpaulin`, `llvm-cov`, etc.)

**View Coverage:**
- Not configured; would require manual setup with `cargo tarpaulin` or `cargo llvm-cov`

**Coverage State:**
- Tests focus on core functionality but gaps exist:
  - Component rendering not tested (Dioxus RSX hard to test without browser context)
  - Service coroutine logic (`use_coroutine`) not directly testable in unit tests
  - API error handling tested, but not full HTTP scenarios

## Test Types

**Unit Tests:**
- **Scope:** Individual functions, structs, enums
- **Approach:** Direct object construction and assertion
- **Examples:**
  - Translation key tests in `src/tests/i18n_tests.rs` — test all locales return non-empty strings
  - Error variant tests in `src/tests/error_tests.rs` — test error types can be created and matched
  - Service store tests in `src/tests/service_tests.rs` — test `Default` impl and action enum variants

- **Typical Test Count:** 130+ test functions (from grep of `#[test]` in tests directory)

**Integration Tests:**
- **File:** `src/tests/integration_tests.rs`
- **Scope:** Not examined in detail, but likely tests interaction between service and state layers
- **Approach:** Probably constructs multi-component scenarios

**E2E Tests:**
- **Framework:** Not used
- **Why:** Dioxus web framework; E2E would require browser test runner (Playwright, Puppeteer)
- **Alternative:** Manual testing or Dioxus SSR (`dioxus-ssr` available as dev-dep) for headless scenarios

## Common Patterns

**Async Testing:**
No async tests observed. Reason: 
- Service coroutines are tested indirectly via their effects (state mutation)
- API functions are not directly tested with real HTTP (would require mocking)
- Tests generally avoid `#[tokio::test]` or `async` test blocks

If async testing needed:
```rust
#[test]
fn test_async_operation() {
    // Would use tokio-test crate (available in dev-deps)
    tokio_test::block_on(async {
        // async test code
    });
}
```

**Error Testing:**
Pattern: Create invalid inputs, verify errors are returned and correctly handled

From `src/tests/error_tests.rs`:
```rust
#[test]
fn test_time_component_range_error() {
    use time::{Date, Month};

    // Try to create an invalid date to trigger ComponentRange error
    let invalid_date_result = Date::from_calendar_date(2024, Month::February, 30);

    match invalid_date_result {
        Err(time_error) => {
            let shifty_error = ShiftyError::TimeComponentRange(time_error);
            match shifty_error {
                ShiftyError::TimeComponentRange(_) => {
                    assert!(true); // Test passes
                }
                _ => panic!("Expected TimeComponentRange error"),
            }
        }
        Ok(_) => {
            // Fallback for edge case
            let week_error = time::Date::from_iso_week_date(2024, 54, time::Weekday::Monday);
            if let Err(time_error) = week_error {
                let shifty_error = ShiftyError::TimeComponentRange(time_error);
                assert!(matches!(shifty_error, ShiftyError::TimeComponentRange(_)));
            }
        }
    }
}
```

**Exhaustive Enum Testing:**
Pattern: Test all enum variants can be instantiated

From `src/tests/i18n_tests.rs`:
```rust
#[test]
fn test_basic_keys_have_translations() {
    let i18n_en = I18n::new(Locale::En);
    let i18n_de = I18n::new(Locale::De);
    let i18n_cs = I18n::new(Locale::Cs);
    
    let keys = vec![
        Key::Save,
        Key::Cancel,
        Key::Edit,
        Key::Delete,
        // ...
    ];
    
    for key in keys {
        let en_text = i18n_en.t(key);
        let de_text = i18n_de.t(key);
        let cs_text = i18n_cs.t(key);
        
        assert!(!en_text.is_empty(), "English translation missing for key: {:?}", key);
        // Repeats for all locales
    }
}
```

## CI/CD Integration

**GitHub Actions:**
- Configured in `.github/workflows/rust.yml`
- Current workflow runs `dx build` (Dioxus build) but **does NOT run `cargo test`**
- Test execution is optional/manual

**To Enable Testing in CI:**
Add to `.github/workflows/rust.yml`:
```yaml
- name: Run tests
  run: cargo test --lib
```

## Known Gaps

**Not Tested:**
1. **Component rendering** — RSX components hard to test without browser
2. **Service coroutines** — `use_coroutine()` behavior not unit-testable
3. **API layer with real HTTP** — Would require mock HTTP server or integration test setup
4. **Event handlers and user interactions** — Would need Dioxus testing utilities
5. **Tailwind CSS classes** — Generated CSS not tested; only source patterns

**Why Gaps Exist:**
- WASM/browser context required for component testing
- Service architecture (GlobalSignal + coroutines) not designed for isolated unit testing
- Frontend testing traditionally weak in Rust web frameworks

---

*Testing analysis: 2026-05-05*
