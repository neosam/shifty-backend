# Testing Patterns — shifty-dioxus (frontend)

**Analysis Date:** 2026-05-07

## Headline assessment

Testing is **moderate but uneven**. The redesigned atoms (`Btn`, `NavBtn`, `PersonChip`, `Field`, `TupleRow`, `Dialog`, `WorkingHoursOverviewLayoutToggle`) and a handful of redesigned pages/components carry rich SSR-based render tests with strong invariant coverage. Pure helpers (i18n, formatting, week math, error mapping, TO conversion) are well-tested.

The **rest of the codebase is sparsely tested**: the 14 service coroutines have no end-to-end coverage (no mockito wiring exists despite the `mockito` dev-dep), the network layer (`src/api.rs`, 1269 lines) has zero tests, large pages (`shiftplan.rs` 1377 lines, `top_bar.rs` 1166 lines, `employee_view.rs` 997 lines) are untested, and several files under `src/tests/` are dead code that does not compile against the current API.

Numbers (counted 2026-05-07):
- 543 `#[test]` functions across 54 source files (`src/` minus `target/`).
- 6 `#[wasm_bindgen_test]` functions (only in `src/tests/api_tests.rs:7` and `src/tests/utils_tests.rs:10-65`).
- 60 `VirtualDom::new` / `rebuild_in_place` invocations — all SSR-based component tests.
- 18 `no_legacy_classes_in_source` self-tests enforcing the design-token migration.
- Zero `mockito::Server` / `MockServer` usages anywhere despite `mockito = "1.2"` being a dev-dep.
- Zero `tokio_test` usages despite `tokio-test = "0.4"` being a dev-dep.

**Take-away for new code:** add SSR render tests to every redesigned atom/component, add pure helper tests to every formatting/conversion function, but do not expect to find a working integration-test harness — there is none.

## Test Framework

**Runner:** Rust's built-in `cargo test` harness. No project-wide test config file.

**Dev-dependencies declared** (`Cargo.toml:76-81`):
- `wasm-bindgen-test = "0.3"` — used in 2 files for browser-only sanity tests of `js::get_current_year/week`.
- `tokio-test = "0.4"` — declared but **never used** in `src/`.
- `mockito = "1.2"` — declared but **never used** in `src/`.
- `dioxus-ssr = "0.6"` — heavily used: 60 sites render components to HTML strings for assertion.

**Run commands:**
```bash
cargo test                    # Run all tests
cargo test --package shifty-dioxus
cargo test --lib              # Unit tests only
cargo test btn                # Filter by name substring
cargo test -- --nocapture     # Show println!/eprintln! output
```

WASM-targeted tests under `src/tests/api_tests.rs` and the `target_arch = "wasm32"` arms of `src/tests/mod.rs::utils_tests` require `wasm-pack test --headless --firefox` (or `--chrome`) and do not run under plain `cargo test` on the host. The headless harness is configured in-file with `wasm_bindgen_test_configure!(run_in_browser);` (`src/tests/api_tests.rs:5`).

## Test File Organization

**Tests are co-located with the code they test** as `#[cfg(test)] mod tests { … }` at the bottom of the source file. This is the dominant pattern. Examples (top of the leaderboard for tests-per-file):
- `src/component/top_bar.rs` — 39 tests
- `src/component/working_hours_mini_overview.rs` — 31 tests
- `src/component/week_view.rs` — 29 tests
- `src/component/dialog.rs` — 21 tests
- `src/component/employee_weekly_histogram.rs` — 18 tests
- `src/page/user_management.rs` — 17 tests
- `src/page/my_shifts.rs` — 15 tests
- `src/component/form/inputs.rs` — 16 tests
- `src/component/atoms/person_chip.rs` — 16 tests
- `src/component/atoms/btn.rs` — 15 tests
- `src/page/weekly_overview.rs` — 13 tests

54 of 114 source files (47%) have at least one `#[cfg(test)]` block.

**Centralized test directory** at `src/tests/` exists alongside co-located tests. Only four files are wired into the build (`src/tests/mod.rs:1-4`):
```
pub mod error_tests;          // src/tests/error_tests.rs
pub mod integration_tests;    // src/tests/integration_tests.rs
pub mod volunteer_work_tests; // src/tests/volunteer_work_tests.rs
pub mod week_tests;           // src/tests/week_tests.rs
```

`src/tests/mod.rs` itself also defines six inline `#[cfg(test)] mod` blocks for `unit_tests`, `i18n_tests`, `service_tests`, `invitation_tests`, `delete_billing_period_tests`, `utils_tests`, `shiftplan_catalog_tests`.

**Dead test files** (present on disk but NOT declared in `src/tests/mod.rs`):
- `src/tests/api_tests.rs` — uses `crate::js::get_current_week` etc. (these symbols exist) but the file is unreachable.
- `src/tests/i18n_tests.rs` — uses `I18n::new(Locale::En)` (single-arg) and `i18n.format_weekday(&weekday)`; both API shapes are stale (`I18n::new` takes `(current, fallback)` — see `src/i18n/i18n.rs:16`, and there is no `format_weekday` method).
- `src/tests/utils_tests.rs` — partially-stale.
- `src/tests/service_tests.rs`, `src/tests/state_tests.rs` — stale field references (`Config::backend_url`, `Employee::id/name/active/extra_hours_september_2024`) that don't exist on the current types.

These files DO NOT compile against the current crate. They are also not exercised because `mod.rs` does not include them. **Do not assume they reflect current behavior.** Either delete them, fix them, or leave them strictly as historical artifacts.

**Naming:** test files are `<feature>_tests.rs` under `src/tests/`. Inline test modules follow `mod tests` or `mod <topic>_tests`. Test functions are `snake_case` and describe the contract: `renders_button_with_secondary_classes_by_default`, `error_preempts_hint`, `closed_dialog_renders_nothing`, `auto_variant_resolves_to_center_outside_wasm`.

## Test Structure

**Suite organization** (canonical example, `src/component/atoms/btn.rs:108-284`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 1) Pure helper tests
    #[test]
    fn variant_classes_primary_has_accent_tokens() {
        let s = variant_classes(BtnVariant::Primary);
        assert!(s.contains("bg-accent"), "primary missing bg-accent: {s}");
    }

    // 2) Default and conditional behavior
    #[test]
    fn default_variant_is_secondary() {
        assert_eq!(BtnVariant::default(), BtnVariant::Secondary);
    }

    // 3) Render helper, used by all SSR tests below
    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    // 4) SSR component tests
    #[test]
    fn renders_button_with_secondary_classes_by_default() {
        fn app() -> Element { rsx! { Btn { "Save" } } }
        let html = render(app);
        assert!(html.starts_with("<button"), "expected <button> root: {html}");
        assert!(html.contains("bg-surface"), "missing secondary class: {html}");
    }

    // 5) Variant-specific render checks
    #[test]
    fn renders_primary_variant() { /* … */ }
    #[test]
    fn renders_ghost_variant() { /* … */ }
}
```

**Patterns:**
- Tests have one assertion target each; multiple `assert!` calls within the same test only confirm related facets of one behavior.
- Failure messages always include the relevant variable, often the rendered HTML: `assert!(html.contains("Save"), "missing children: {html}");`. This is the project-wide convention — every assertion-with-substring includes the haystack in the message.
- The `render(comp: fn() -> Element) -> String` helper is **duplicated locally in every component test module** that needs SSR (10 copies confirmed via `grep -rn "fn render(" src/`). There is no shared test util crate.

**No `setup`/`teardown`** infrastructure. Each test constructs its own state inline. Some modules expose private `make_*` helpers as in-module conveniences (`src/component/employee_short.rs:69-97` defines `employee_with`; `src/page/my_shifts.rs:340-356` defines `make_block`/`make_i18n`). These are file-private helpers, not a shared fixture system.

## Mocking

**There is no mocking layer.** `mockito` and `tokio-test` are listed as dev-deps but never imported. Service-layer tests are explicitly limited to pure functions because of this gap. The longest single comment in any test module documents this honestly (`src/service/employee.rs:280-300`):

> The async coroutine handler in `employee_service` reaches into the global `CONFIG` / `EMPLOYEE_STORE` / `ERROR_STORE` signals and issues real `reqwest` calls via `api::*`. There is no mock-API layer in this crate today, so we cannot drive the `UpdateExtraHours` arm end-to-end from a unit test without standing up an HTTP server.

**What IS used in lieu of mocks:**
- **Pure-function extraction:** business logic is pulled out of the coroutine into a `pub fn` that takes plain inputs. See `build_update_payload` in `src/service/employee.rs:171-189`, tested at `:325-360`.
- **`cfg(target_arch = …)` fallbacks:** functions that need a browser API in WASM provide deterministic stubs for non-WASM tests. Example, `src/component/extra_hours_modal.rs:50-59` returns a fixed `datetime!(2026-01-01 00:00:00)` outside WASM so tests are deterministic.
- **`use_hook` Drop-guard listeners** in `dialog.rs` and `media_query.rs` only register their JS callbacks under `#[cfg(target_arch = "wasm32")]`, so SSR tests don't try to touch `window`.

**What to mock:** when actual HTTP testing is added, the `mockito::Server` API is the dev-dep target. Today this is unused.

**What NOT to mock:** the i18n system (`I18N` global is constructed deterministically by `crate::i18n::generate(Locale::De)` — see `src/page/my_shifts.rs:354-356`). The `Week` type, `format_hours` helper, `WorkingHoursCategory` round-trips, and similar pure logic don't need mocks.

## Fixtures and Factories

**No central fixtures directory.** When test data is needed, the convention is one of:

1. **Local `make_<thing>` factory** at the bottom of the test module:
```rust
fn make_block(day: DayOfWeekTO, fh: u8, fm: u8, th: u8, tm: u8, person: Option<&str>) -> BlockTO {
    BlockTO { /* … */ }
}
fn make_i18n() -> I18n<Key, Locale> {
    crate::i18n::generate(Locale::De)
}
```
(`src/page/my_shifts.rs:340-356`, `src/component/employee_short.rs:69-97`)

2. **Inline construction** of full struct literals — frequent in `src/tests/integration_tests.rs` and `src/tests/mod.rs`. This is verbose but explicit; refactor to a factory once the same struct is built in 3+ tests.

3. **`Default::default()` + override** for stores: `let store = TextTemplateStore::default();` followed by direct field manipulation (`src/tests/mod.rs:103-144`).

**Test JSON payloads** are inline `r#"…"#` raw strings, used heavily in `src/tests/mod.rs::invitation_tests` (lines 147-299) to lock the deserialization contract against real backend response shapes.

## Coverage

**No formal coverage requirement.** No `tarpaulin`, `llvm-cov`, or coverage CI step in the repo (`grep -rn "tarpaulin\|llvm-cov" .` returns nothing).

**Run coverage manually with:**
```bash
cargo install cargo-llvm-cov   # one-time
cargo llvm-cov --html
```

## Test Types

### Unit Tests (the bulk)

**Pure helper tests** — class-string builders, `format_hours`, `format_time_range`, `block_hours`, `sum_hours`, `signed_hours_diff`, `hours_text_class`, `matches_search`, `target_hours_for`, `resolve_variant`, `backdrop_layout`, `panel_style`, `is_escape_key`. These compose into >60% of all tests.

**Enum / type-conversion tests:**
- `src/tests/volunteer_work_tests.rs` — locks the `WorkingHoursCategory ↔ ExtraHoursCategoryTO ↔ ExtraHoursReportCategoryTO` round-trips for the volunteer-work category.
- `src/tests/mod.rs::shiftplan_catalog_tests` — locks `ShiftplanTO` JSON deserialization shapes (`is_planning` default, `$version` rename, missing fields).
- `src/tests/mod.rs::invitation_tests` — locks `InvitationResponse` deserialization for null/Z-suffixed/missing `redeemed_at` shapes against the exact backend payload (`mod.rs:202-249`).

**i18n tests** in `src/i18n/mod.rs:422-552`:
- `i18n_*_keys_present_in_all_locales` — iterates `[En, De, Cs]` and asserts every listed `Key` returns a non-empty, non-`"??"` translation.
- `i18n_employees_keys_match_german_reference` — locks specific German strings.
- `shiftplan_filled_of_need_substitutes_placeholders` and `shiftplan_delete_confirm_body_interpolates_name` — verify `t_m` placeholder substitution.
- This is the canonical pattern for every new translation key. **Adding a key without adding a presence test is informally allowed but reviewer-discouraged.**

### Component Tests (SSR)

**60 SSR-render tests** across 12 component/page files. Pattern (`src/component/atoms/btn.rs:187-283`):

```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[test]
fn renders_primary_variant() {
    fn app() -> Element {
        rsx! { Btn { variant: BtnVariant::Primary, "Go" } }
    }
    let html = render(app);
    assert!(html.contains("bg-accent"));
    assert!(html.contains("text-accent-ink"));
}
```

**What SSR tests verify:**
- Class strings appear in the rendered HTML (token correctness).
- Conditional rendering branches: `closed_dialog_renders_nothing`, `subtitle_renders_when_provided`, `error_preempts_hint`, `no_hint_and_no_error_renders_neither`.
- ARIA attributes and roles: `dialog_root_has_presentation_role_and_modal_panel_has_dialog_role` (`src/component/dialog.rs:657-681`).
- Children ordering: `icon_renders_in_mono_span_before_children` does substring `find` to confirm `<span class="font-mono">` precedes the children block (`src/component/atoms/btn.rs:256-271`).
- Disabled state: native `disabled` attribute + opacity classes both rendered.
- Variant-specific styles: each enum variant of `DialogVariant`, `BtnVariant`, `WorkingHoursLayout` gets a render test.

**SSR limitations:**
- No interaction. SSR is a one-shot render, so `onclick` handlers cannot be driven from tests. Click behavior is verified indirectly: tests assert the `disabled` attribute is rendered, that an `aria-label` propagates, that the `onclick` handler is *registered* (by source-grep tests like `no_legacy_classes_in_source`), but never that it *runs*.
- No `use_effect` execution, no signal updates after rebuild, no real media-query evaluation. `use_media_query` returns the non-WASM stub (`false`) under SSR, which is documented in `src/component/dialog.rs:582-605` (`auto_variant_resolves_to_center_outside_wasm`).
- The `rebuild_in_place` step renders the first frame only.

**Source self-tests** (the `no_legacy_classes_in_source` family) are technically `#[test]` functions that `include_str!` the file they live in and grep for forbidden tokens. 18 such tests across pages and components. They are pure compile-time hygiene checks — fast and 100% deterministic. **Add one to every new page or redesigned component** (template at `src/page/employees.rs:18-43`).

### Integration Tests

`src/tests/integration_tests.rs` is the only "integration" file wired in. Its tests are still pure-function in nature — they exercise multi-module flows but do not spin up Dioxus, JS, or a backend:

- `test_authenticated_user_workflow` — locks `AuthInfo::has_privilege` semantics across multiple privileges.
- `test_week_calculation_with_i18n` — uses a real `Week`, asks each locale to format the same date, and verifies the cross-locale invariants (German contains `.`, week formatting includes the week number).
- `test_template_management_workflow` — drives a `TextTemplateStore` through default → populated → filtered states.

**No real HTTP integration tests.** No fixtures simulate a backend. No Playwright / browser-driver tests under `src/`. The repo root contains a `.playwright-mcp/` folder with screenshots from manual exploration but no automated browser tests in the build.

### E2E Tests

**Not used.** No Cypress, Playwright, or WebDriver harness in `Cargo.toml`/`package.json`. Manual exercise on `dx serve --hot-reload` is the documented validation path (per `CLAUDE.md`).

### WASM Browser Tests

Six `#[wasm_bindgen_test]` functions exist:
- `src/tests/api_tests.rs:7-16` — `test_week_calculation` runs `js::get_current_week()` / `get_current_year()` in a real browser.
- `src/tests/mod.rs::utils_tests::test_year_week_ranges` (the `cfg(target_arch = "wasm32")` arm) — same JS Date-bound invariants.
- 4 more under the same module covering UUID generation, date validation, error result handler, and bounds checks.

These do not run under `cargo test`. They require `wasm-pack test --headless --firefox`. There is no CI step running them today.

## Common Patterns

### Async testing

The repo's async tests are mostly pure: they `.await` a constructor like `crate::i18n::generate(Locale::En)` (sync) or call a `pub fn` extracted from a coroutine. Because there is no `tokio_test::block_on` usage, async coroutines are not driven from unit tests. When a service action needs verification, the convention is:

1. Extract the data-shaping logic into a sync `pub fn`.
2. Test the sync helper directly.
3. Document the dispatch wiring as "covered by inspection" in a comment.

This is explicit in `src/service/employee.rs:280-300`. Replicate the comment when you find yourself in the same gap.

### Error testing

Use deliberately-invalid inputs to construct error values in-test (because `reqwest::Error` cannot be constructed manually). Examples (`src/tests/error_tests.rs`):

```rust
let invalid_date = time::Date::from_calendar_date(2024, time::Month::February, 30);
match invalid_date {
    Err(time_error) => {
        let shifty_error = ShiftyError::TimeComponentRange(time_error);
        assert!(matches!(shifty_error, ShiftyError::TimeComponentRange(_)));
    }
    Ok(_) => { /* fallback construction */ }
}
```

For week-bounds errors, use `Week { year: 2024, week: 0 }` or `week: 55` (`src/tests/error_tests.rs:121-159`).

### Signal-write tests under VirtualDom

To exercise `GlobalSignal` reads/writes inside a unit test, you must drive them from inside a Dioxus runtime. The harness is the same `VirtualDom::new + rebuild_in_place` pattern used for SSR, but the assertions go inside the component body (`src/service/employee.rs:308-323`):

```rust
#[test]
fn bump_employees_list_refresh_increments_observable_signal() {
    fn assertion_app() -> Element {
        let before = *EMPLOYEES_LIST_REFRESH.read();
        bump_employees_list_refresh();
        let after = *EMPLOYEES_LIST_REFRESH.read();
        assert_eq!(after, before.wrapping_add(1));
        rsx! {}
    }
    let mut vdom = VirtualDom::new(assertion_app);
    vdom.rebuild_in_place();
}
```

### Locking JSON contracts against the live backend

`src/tests/mod.rs::invitation_tests::test_exact_backend_response` (lines 202-249) and `shiftplan_catalog_tests::test_shiftplan_to_*` (lines 437-633) embed real backend payloads as `r#"…"#` literals and `serde_json::from_str` them. Use this pattern whenever a TO field gets a `#[serde(rename)]`, `#[serde(default)]`, or new optional field — it is the only safety net catching cross-repo breaking changes between `shifty-backend` and `shifty-dioxus`.

## Coverage Gaps (be honest)

Treat as candidates for new tests in any phase that touches them:

- **`src/api.rs`** (1269 lines): zero tests. Every REST call is uncovered. The `mockito` dep was added with the intent to cover this surface but no test was ever written.
- **`src/service/*.rs`** (14 files): coroutine bodies are uncovered; only `service::employee::{build_update_payload, bump_employees_list_refresh}` and a handful of store-default tests in `src/tests/mod.rs::service_tests` exist.
- **`src/loader.rs`** (865 lines): one test (`#[cfg(test)]` count = 1, used for an internal helper). The TO→domain conversions inside loaders are uncovered.
- **`src/component/week_view.rs`** (1631 lines): 29 tests, but the file is the largest and most complex in the crate; coverage per LOC is low.
- **`src/component/top_bar.rs`** (1166 lines): 39 tests focused on class strings, no coverage of dropdown/i18n switch interactions.
- **`src/page/shiftplan.rs`** (1377 lines): no `#[cfg(test)]` block. The most-used page in the app is fully untested.
- **`src/component/employee_view.rs`** (997 lines): no `#[cfg(test)]` block.
- **`src/auth.rs`**, **`src/router.rs`**, **`src/js.rs`**: no tests.
- **`src/tests/api_tests.rs`, `i18n_tests.rs`, `service_tests.rs`, `state_tests.rs`, `utils_tests.rs`**: dead — not wired into `mod.rs` and reference stale APIs that no longer exist.

When a phase touches any of the above, the executor MUST add tests rather than rely on existing coverage.

---

*Testing analysis: 2026-05-07*
