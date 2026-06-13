---
phase: quick-260613-jxe
plan: "01"
subsystem: frontend
tags: [absences, vacation-balance, year-nav, i18n, filter]
dependency_graph:
  requires: [7c2e0a0]
  provides: [selectable_balances, year-nav-ui]
  affects: [shifty-dioxus/src/page/absences.rs, shifty-dioxus/src/i18n]
tech_stack:
  added: []
  patterns: [pure-fn-filter, reactive-signal, use_effect-subscription]
key_files:
  created: []
  modified:
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - Reuse is_selectable_employee predicate from commit 7c2e0a0 in the new selectable_balances helper (no duplication of the paid && active rule)
  - Year navigation is page-state only (no API/loader change); selected_year signal drives reactive use_effect re-dispatch
  - Per-person list empty-guard moved after selectable_balances filter so inactive-only lists collapse correctly
  - Test vacation_card_hr_without_selection_renders_team_aggregate updated to supply selectable sales_persons (required after filter was wired in)
metrics:
  duration: "35 minutes"
  completed: "2026-06-13T12:30:00Z"
  tasks_completed: 3
  files_changed: 5
---

# Phase quick-260613-jxe Plan 01: Absences Page — Inactive Filter + Year Navigation Summary

**One-liner:** Filter per-person vacation list to paid && active employees (reusing `is_selectable_employee`) and add reactive year navigation (◀ {year} ▶) that reloads per-year vacation data.

## Tasks Completed

| Task | Name | Commit | Key Changes |
|------|------|--------|-------------|
| 1 | Inactive filter on VacationPerPersonList | 0511d545 | `selectable_balances` pure fn + wire into component + 5 unit tests |
| 2 | Year navigation + i18n | d80d813c | `selected_year` signal, ◀/▶ UI, 2 i18n keys × 3 locales, test fix |
| 3 | WASM build gate + full test gate | (no code) | WASM: Finished dev profile; cargo test: 594 passed, 0 failed |

## Implementation Notes

### (a) Inactive filter — `is_selectable_employee` reuse

The `selectable_balances(rows, sales_persons) -> Vec<VacationBalance>` pure helper added in `absences.rs` reuses the existing `is_selectable_employee(sp)` predicate (`sp.is_paid && !sp.inactive`) from commit 7c2e0a0 without duplicating the rule. Semantics:

- A balance is KEPT only if its `sales_person_id` matches a `SalesPerson` in `sales_persons` for which `is_selectable_employee` returns `true`.
- A balance whose person is missing from the list or is inactive/unpaid is DROPPED.

The helper is wired into `VacationPerPersonList` before the empty-guard: `filtered = selectable_balances(...)` → empty-guard on `filtered` → remaining-days sort on `filtered`. The show-all counter and "Show all (N)" label reflect the filtered set.

### (b) Year navigation — page-state only, verified via WASM build gate + pure unit tests

The year navigation is pure page-state wiring: `selected_year = use_signal(current_year_for_init)`. No `VacationBalanceAction` variants were added or changed. The existing `use_effect` now reads `*selected_year.read()` inside the closure so Dioxus subscribes to the signal and re-fires the effect (which dispatches `LoadTeam(year)` / `LoadSelf(sp, year)`) whenever the user clicks ◀ or ▶.

The nav control (◀ · {year} · ▶) is placed above `VacationEntitlementCard` and is visible for both HR and employee paths. The prev button uses `saturating_sub(1)` to prevent u32 underflow.

**Why no headless click tests:** The existing test suite has no headless DOM-interaction tests (no wasm-bindgen-test runner is wired up). The click→signal→reload flow is verified indirectly by:
1. WASM build gate — confirms the rsx! compiles, `selected_year.set(...)` closure types check.
2. Pure unit tests — confirm `selectable_balances` and `is_selectable_employee` behave correctly.

This matches the existing convention in this codebase (see Task 3 note in the plan).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test `vacation_card_hr_without_selection_renders_team_aggregate` failed after filter wiring**

- **Found during:** Task 2 (running full cargo test after year-nav changes)
- **Issue:** The test passes `sales_persons: Rc::<[SalesPerson]>::from([])` — empty list. After wiring `selectable_balances` into `VacationPerPersonList`, all balances were dropped (no matching sales persons), so the per-person section returned early and the "Pro Person" header never appeared.
- **Fix:** Updated the test to supply two paid+active `SalesPerson` entries matching the test balance UUIDs, so the filter keeps both balances and the per-person section renders as expected.
- **Files modified:** `shifty-dioxus/src/page/absences.rs` (test only)
- **Commit:** d80d813c (included in Task 2 commit)

## Self-Check: PASSED

- `selectable_balances` function exists at top of absences.rs: confirmed
- 5 unit tests for `selectable_balances` exist and pass: confirmed (5/5 green)
- `AbsenceYearNavPrev` and `AbsenceYearNavNext` keys in mod.rs: confirmed
- All three locales have both keys: confirmed (de, en, cs)
- WASM build gate: Finished dev profile (via lld from Nix store)
- `cargo test` gate: 594 passed, 0 failed
- Commits: 0511d545 (task 1), d80d813c (task 2)
