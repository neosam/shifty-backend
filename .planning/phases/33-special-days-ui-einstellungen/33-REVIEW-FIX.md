---
phase: 33-special-days-ui-einstellungen
fixed_at: 2026-06-30T13:30:00Z
review_path: .planning/phases/33-special-days-ui-einstellungen/33-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 33: Code Review Fix Report

**Fixed at:** 2026-06-30T13:30:00Z
**Source review:** .planning/phases/33-special-days-ui-einstellungen/33-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (all WARNING; the 3 INFO findings are out of scope for `critical_warning`)
- Fixed: 6
- Skipped: 0

**Gates run:**
- Backend: `cargo test -p service_impl special_day` → 12 passed, 0 failed.
- Backend: `cargo clippy -p service_impl -p dao_impl_sqlite --all-targets -- -D warnings` → clean.
- Frontend: `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` → built successfully (only pre-existing dead-code warnings; dioxus is not clippy-gated).

## Fixed Issues

### WR-01: DAO `try_from` uses `.unwrap()` for `version`

**Files modified:** `dao_impl_sqlite/src/special_day.rs`
**Commit:** ec28e52
**Applied fix:** Changed `Uuid::from_slice(&entity.update_version).unwrap()` to `?`, matching the sibling `id` field and the codebase `TryFrom` convention. A malformed `update_version` blob now returns a `DaoError` instead of panicking the request task.

### WR-02: No duplicate guard for `(year, calendar_week, day_of_week)`

**Files modified:** `service_impl/src/special_days.rs`, `service_impl/src/test/special_days.rs`, `shifty-dioxus/src/page/settings.rs`
**Commit:** 0a1a214 (backend guard + tests), 74d271f (Settings form reset)
**Applied fix:** Added a service-level pre-create existence check in `SpecialDayServiceImpl::create`: it calls `find_by_week(year, calendar_week)` and returns `ServiceError::ValidationError([Duplicate])` if any existing entry shares the same `day_of_week`. The check runs after the id/version-nil checks so it is only reached for otherwise-valid creates. The Settings Add path now also resets the create form after a successful create so an immediate re-click cannot resubmit an exact duplicate. Covered by `test_create_rejects_duplicate`. (SpecialDayService stays Basic-tier — only its own DAO + Permission are consumed.)

### WR-03: Backend `create` performed no input validation

**Files modified:** `service_impl/src/special_days.rs`, `service_impl/src/test/special_days.rs`
**Commit:** 0a1a214
**Applied fix:** `create` now validates before persisting (D-33-06, server-side enforcement allowed by D-33-07):
- rejects `ShortDay` without a `time_of_day` (`ValidationError(InvalidValue)`);
- normalizes a `Holiday` to `time_of_day = None`;
- bounds-checks `calendar_week` against `time::util::weeks_in_year(year)`.
Covered by `test_create_rejects_shortday_without_time` and `test_create_rejects_calendar_week_out_of_range`.

### WR-04: Year-boundary entries silently disappear from the Settings list

**Files modified:** `shifty-dioxus/src/page/settings.rs`
**Commit:** 74d271f
**Applied fix:** After a successful create, the Settings year picker is switched to the created entry's ISO-week-year (the `iso_year` already parsed from the date), and the form is reset. An entry created from a date whose ISO year differs from the picker year (e.g. `2027-01-01` → ISO 2026/W53) now stays visible because the list reloads for the year the entry actually lives in.
**Note:** Frontend behavior change; verified by successful WASM build only. Dioxus signal/date-input behavior is not unit-testable in this project (see project memory on datepicker signals) — recommend a quick browser confirmation that the picker jumps to the entry's year.

### WR-05: Missing tests for the create happy-path and `get_by_week`

**Files modified:** `service_impl/src/test/special_days.rs`
**Commit:** 0a1a214
**Applied fix:** Added `test_create_success` (asserts `dao.create` is invoked once with a non-nil id/version, `created` stamped from the clock, and the expected entity), `test_create_rejects_nonnil_version`, and `test_get_by_week_delegates_and_maps` (mirrors `test_get_by_year_delegates_and_maps`). All pass.

### WR-06: Shiftplan inline ShortDay confirm fails silently on unparseable time

**Files modified:** `shifty-dioxus/src/page/shiftplan.rs`
**Commit:** ddf8c61
**Applied fix:** The inline ShortDay confirm now accepts both `[hour]:[minute]:[second]` and `[hour]:[minute]` (reusing the dual-format approach from Settings), and on parse failure sets `special_day_error` for that weekday instead of returning silently.
**Note:** Frontend behavior change; verified by successful WASM build only — recommend a quick browser confirmation that an unparseable time shows the inline error.

## Out of Scope (not fixed)

The 3 INFO findings (IN-01 OpenAPI `path = ""`, IN-02 Add button enabled with no type, IN-03 delete lacks optimistic-concurrency guard) are out of scope for `fix_scope = critical_warning` and were not addressed.

---

_Fixed: 2026-06-30T13:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
