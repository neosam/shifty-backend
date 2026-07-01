---
phase: 38-frontend-build-hygiene
plan: 01
subsystem: ui
tags: [dioxus, wasm, cargo-fix, warnings, dead-code, time-crate]

requires: []
provides:
  - "shifty-dioxus cargo build: auto-fixable and deprecated warning buckets cleared to 0"
  - "34 dead-code warnings remaining (exclusively), ready for plan 38-02"
  - "Re-captured live warning baseline (50 warnings = 14 auto-fixable + 2 deprecated + 34 dead-code)"
affects: [38-02-PLAN]

tech-stack:
  added: []
  patterns:
    - "Test-only imports placed inside #[cfg(test)] module, not top-level, to avoid unused-import warnings in non-test builds"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/component/mod.rs
    - shifty-dioxus/src/component/warning_list.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/page/shiftplan.rs
    - shifty-dioxus/src/page/user_details.rs
    - shifty-dioxus/src/router.rs

key-decisions:
  - "Use parse_borrowed::<2> for the time::format_description::parse migration (D-05) — format strings [day].[month] and [hour]:[minute] are identical in v1 and v2 syntax"
  - "Restore generate/Locale imports inside #[cfg(test)] module (not top-level) after cargo fix incorrectly removed test-only imports"

patterns-established:
  - "Pattern: cargo fix removes imports used only in #[cfg(test)] — always rerun cargo test after cargo fix to catch false-positive import removal"

requirements-completed: [HYG-01]

coverage:
  - id: D1
    description: "Auto-fixable warning bucket (unused imports/vars/mut) cleared to 0 via cargo fix"
    requirement: HYG-01
    verification:
      - kind: other
        ref: "cargo build 2>&1 | grep -E 'unused import|unused variable|does not need to be mutable' — NONE FOUND"
        status: pass
    human_judgment: false
  - id: D2
    description: "Deprecated time::format_description::parse migrated to parse_borrowed::<2> at 2 sites in shiftplan.rs"
    requirement: HYG-01
    verification:
      - kind: other
        ref: "cargo build 2>&1 | grep deprecated — NONE FOUND"
        status: pass
    human_judgment: false
  - id: D3
    description: "shifty-dioxus cargo build compiles with 34 dead-code-only warnings (no new errors)"
    requirement: HYG-01
    verification:
      - kind: other
        ref: "cargo build — Finished dev profile with 34 warnings (exit 0)"
        status: pass
    human_judgment: false
  - id: D4
    description: "WASM build still compiles (cargo build --target wasm32-unknown-unknown)"
    requirement: HYG-01
    verification:
      - kind: other
        ref: "cargo build --target wasm32-unknown-unknown — Finished dev profile with 29 warnings (exit 0)"
        status: pass
    human_judgment: false
  - id: D5
    description: "cargo test -p shifty-dioxus: no new test failures (only pre-existing i18n_impersonation_keys_match_german_reference failure)"
    requirement: HYG-01
    verification:
      - kind: unit
        ref: "cargo test -p shifty-dioxus — 727 passed; 1 failed (pre-existing)"
        status: pass
    human_judgment: false

duration: 11min
completed: 2026-07-01
status: complete
---

# Phase 38 Plan 01: Frontend Build Hygiene — Auto-fix + Parse Migration Summary

**Cleared shifty-dioxus auto-fixable and deprecated warning buckets to 0 via `cargo fix` (14 warnings) and `parse_borrowed::<2>` migration (2 sites); 34 dead-code-only warnings remain for plan 38-02.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-07-01T16:39:28Z
- **Completed:** 2026-07-01T16:51:00Z
- **Tasks:** 2 (Task 1: re-baseline; Task 2: cargo fix + parse migration)
- **Files modified:** 7

## Re-captured Live Warning Baseline

**Captured 2026-07-01 from the current tree (post phases 36/37):**

| Bucket | Count | Action |
|--------|-------|--------|
| Auto-fixable (unused imports, unused variables, unnecessary mut) | **14** | Cleared this plan (cargo fix) |
| Deprecated `time::format_description::parse` | **2** | Cleared this plan (parse_borrowed::<2>) |
| Dead-code (never-used fns/methods/enums/variants/structs/consts/fields) | **34** | Deferred to plan 38-02 |
| **Total** | **50** | |

**Delta from CONTEXT "50" list:** Zero delta. The live baseline matches the CONTEXT exactly — phases 36/37 did not add new warnings and did not eliminate any previously-dead symbols.

### Auto-fixable detail (14 — all cleared)
- `src/api.rs:4` — unused imports: `AbsenceCategoryTO`, `DayFractionTO`, `ExtraHoursMarkerTO`, `WarningTO`
- `src/component/warning_list.rs:18` — unused imports: `Locale`, `generate`
- `src/component/mod.rs:35` — `AbsenceConvertModal`; `:36` — `AddExtraHoursForm`; `:37` — `TupleRow`; `:44` — `EmployeeWorkDetailsForm`; `:45` — `EmployeesList`; `:49` — `FormCheckbox`
- `src/router.rs:4` — unused import: `AbsencesPage`
- `src/page/absences.rs:1269` — `on_close`; `:1346` — `date_iso_format_clone1`; `:1347` — `date_iso_format_clone2`
- `src/page/shiftplan.rs:212` — `mut block_error` (unnecessary mut)
- `src/page/user_details.rs:26` — `error_store`

### Deprecated detail (2 — all migrated)
- `src/page/shiftplan.rs:122` → `parse_borrowed::<2>("[day].[month]")`
- `src/page/shiftplan.rs:1291` → `parse_borrowed::<2>("[hour]:[minute]")`

### Dead-code detail (34 — deferred to 38-02)
Functions: `get_slots`, `get_bookings_for_week`, `add_booking`, `get_absence_period`, `parse_time_input`, `has_sunday_slots` (×2 locations), `is_escape_key`, `partition_nav_items`, `ColumnViewSlot`, `slot_to_column_view_item_with_tooltips`, `day_total_label`, `load_bookings`, `load_slots`, `register_user_to_slot`, `generate_custom_report`

Enums/structs/consts: `AddExtraHoursFormAction`, `ModalMode`, `STORAGE_KEY`, `DARK_MEDIA_QUERY`, `WorkingSchedule`

Variants: `Sheet`, `LoadBillingPeriod`, `LoadWeekMessage`, `Refresh`, `ClearSelection` (×2), `Delete`, `LoadTemplate`, `ClearFilter`, `SystemThemeChanged`, `SaveSalesPerson`, `LoadAllSalesPersonUserLinks`, `LoadAllUserSalesPersonLinks`, `LoadAllUserRoles`

Methods/traits: `id` (trait method), `has_sunday_slots` (method), `from_str`, `as_str`

Fields: field `0` in `AbsenceModalEvent::Network(String)`

## Accomplishments

- Re-baselined the live warning set: 50 total (14 auto-fixable, 2 deprecated, 34 dead-code) — confirmed same count as CONTEXT (phases 36/37 caused no new warnings)
- Cleared 14 auto-fixable warnings via `cargo fix` across 7 source files
- Migrated both deprecated `time::format_description::parse` sites to `parse_borrowed::<2>` in `src/page/shiftplan.rs`
- Plan 38-02 starts with a clean dead-code-only warning list (34 warnings)

## Task Commits

Task 1 (re-baseline) produced no source changes — analysis only, recorded in this SUMMARY.

1. **Task 2: cargo fix + parse_borrowed migration** - `71c7435` (chore)

## Files Created/Modified

- `shifty-dioxus/src/api.rs` — removed 4 unused imports (`AbsenceCategoryTO`, `DayFractionTO`, `ExtraHoursMarkerTO`, `WarningTO`)
- `shifty-dioxus/src/component/mod.rs` — removed 6 unused re-exports (`AbsenceConvertModal`, `AddExtraHoursForm`, `TupleRow`, `EmployeeWorkDetailsForm`, `EmployeesList`, `FormCheckbox`)
- `shifty-dioxus/src/component/warning_list.rs` — moved `generate`/`Locale` imports into `#[cfg(test)]` module (cargo fix incorrectly removed them from top-level)
- `shifty-dioxus/src/page/absences.rs` — removed 3 unused variables (`on_close`, `date_iso_format_clone1/2`)
- `shifty-dioxus/src/page/shiftplan.rs` — removed unnecessary `mut` on `block_error`; migrated 2 deprecated `parse` → `parse_borrowed::<2>`
- `shifty-dioxus/src/page/user_details.rs` — removed unused variable `error_store`
- `shifty-dioxus/src/router.rs` — removed unused import `AbsencesPage`

## Decisions Made

- Used `parse_borrowed::<2>` (not `parse_borrowed::<1>`) for the migration: the format strings `[day].[month]` and `[hour]:[minute]` are syntactically identical in format description version 1 and version 2; version 2 is current and what the deprecation message recommends
- Moved `generate`/`Locale` imports into the `#[cfg(test)]` module rather than restoring them at top-level, to preserve the clean no-import-warning state in non-test builds (D-06 inline pattern)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `cargo fix` incorrectly removed `generate` and `Locale` imports used in `#[cfg(test)]`**
- **Found during:** Task 2 (post-cargo-fix verification via `cargo test`)
- **Issue:** `cargo fix` removed `generate` and `Locale` from `src/component/warning_list.rs` imports because they appeared unused in the non-test build, but they are used in `#[cfg(test)] mod tests { use super::*; ... }` at line 186 (`generate(Locale::De)`). Test compilation failed with E0425/E0433.
- **Fix:** Added `use crate::i18n::{generate, Locale};` inside the `#[cfg(test)] mod tests` block directly (not at top-level, to avoid re-introducing an unused-import warning in the non-test build)
- **Files modified:** `shifty-dioxus/src/component/warning_list.rs`
- **Verification:** `cargo test -p shifty-dioxus` compiles and 727 tests pass (only pre-existing failure remains)
- **Committed in:** `71c7435` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 bug)
**Impact on plan:** Fix was necessary for test compilation. No scope creep. Committed inline with Task 2.

## Issues Encountered

`cargo fix` has a known limitation: it cannot see `#[cfg(test)]` usage and removes imports it considers "unused" in the non-test build path. Always run `cargo test` (not just `cargo build`) after `cargo fix` to catch false-positive import removals.

## Known Stubs

None — this plan removes code/renames API calls; it does not add UI components or data flows.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Next Phase Readiness

Plan 38-02 can now start directly from a clean dead-code-only warning list:
- 34 dead-code warnings remain (functions, enum variants, structs, constants, methods, fields)
- The full list is categorized above under "Dead-code detail"
- Key landmark: `has_sunday_slots` at `src/state/shiftplan.rs:315` (also `src/component/day_aggregate_view.rs:194`)
- Per D-01, default policy is DELETE dead code; D-03 exceptions require inline `// reason: <why>`

## Self-Check

---
*Phase: 38-frontend-build-hygiene*
*Completed: 2026-07-01*
