---
phase: 17-contract-editor-unpaid-volunteer-path
plan: "04"
subsystem: frontend
tags: [D-03, D-07, CVC-10, show_all, unpaid-volunteer, employees-list]
dependency_graph:
  requires: [17-01, 17-03]
  provides: [show_all-toggle, unpaid-dummy-loader, employee-visible-predicate]
  affects: [shifty-dioxus/src/component/employees_list.rs, shifty-dioxus/src/loader.rs, shifty-dioxus/src/state/employee.rs]
tech_stack:
  added: []
  patterns:
    - use_signal(|| false) for boolean toggle (analog to billing_period_details.rs show_paid)
    - use_resource second call for disjoint data source (no dedup needed)
    - extracted employee_visible() predicate for unit-testability
key_files:
  created: []
  modified:
    - shifty-dioxus/src/component/employees_list.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/state/employee.rs
decisions:
  - "Dummy constructor as associated function Employee::unpaid_placeholder(SalesPerson) — avoids From<&SalesPerson> conflict with existing From<&SalesPersonTO>-based conversions in shiftplan.rs"
  - "Second use_resource always loads unpaid list (not gated on show_all signal) — Dioxus resources run once at mount; gating at render time via combined list merge when show_all_val=true is simpler and avoids signal-inside-closure issues"
  - "config cloned to config2 before first use_resource closure to avoid move conflict — standard Dioxus pattern"
  - "Dedup not needed: load_unpaid_volunteer_employees filters on !is_paid, load_employees comes from GET /report which backend-filters on is_paid=true — sets are disjoint by construction"
metrics:
  duration: "~25 minutes"
  completed: "2026-06-24"
  tasks_completed: 3
  files_modified: 3
  tests_added: 5
---

# Phase 17 Plan 04: show_all-Toggle + Unpaid-Volunteer-Merge Summary

One-liner: Employees list gains a show_all checkbox (default: paid-only) that merges unpaid non-inactive volunteers via a second GET /sales-person call as null-hours Employee dummies.

## Tasks Completed

### Task 1: Loader + Employee-Dummy-Konstruktor + Unit-Tests

**Files:** `shifty-dioxus/src/state/employee.rs`, `shifty-dioxus/src/loader.rs`

Added `Employee::unpaid_placeholder(sales_person: SalesPerson) -> Employee` as an associated function. Chose associated function over `From<&SalesPerson>` to avoid conflicts with the existing `From<&SalesPersonTO> for SalesPerson` pattern in `shiftplan.rs`. All reporting/hours fields set to 0.0/[]/0 per D-03 Null-Stunden-Invariante. Comment explains the design: "D-03: Dummy für unbezahlte Freiwillige im show_all-Modus — Reporting-Werte 0, da GET /report sie paid-only ausschließt."

Added `loader::load_unpaid_volunteer_employees(config: Config) -> Result<Rc<[Employee]>, ShiftyError>` which:
- Calls `api::get_sales_persons(config)` (GET /sales-person)
- Converts `SalesPersonTO` → `SalesPerson` via `From`
- Filters on `!sp.is_paid && !sp.inactive`
- Maps each to `Employee::unpaid_placeholder`
- Returns `Rc<[Employee]>`
- Maps `reqwest::Error` to `ShiftyError` automatically via the existing `#[from]` impl

Two unit tests in `state/employee.rs`:
- `unpaid_dummy_preserves_is_paid_false`: asserts dummy.sales_person.is_paid == false and id/name match source
- `unpaid_dummy_has_zero_hours`: asserts all 13 numeric fields == 0.0/0 and all slice fields are empty

Both tests pass.

### Task 2: show_all-Toggle + Filter-Kette + Merge + Filter-Tests

**File:** `shifty-dioxus/src/component/employees_list.rs`

Extracted pure filter predicate `employee_visible(e: &Employee, show_all: bool, term: &str) -> bool` (module-level, `pub(crate)`) for unit-testability. Predicate: `!inactive && (show_all || is_paid) && matches_search(name, term)`.

Added to `EmployeesList` component:
1. `let config2 = config.clone()` before first use_resource (avoids move conflict)
2. Second `use_resource` calling `load_unpaid_volunteer_employees(config2)` — always runs, merged only when `show_all_val=true`
3. `let mut show_all = use_signal(|| false)` — Default: only paid employees
4. Toggle UI: `<label>` with `<input type=checkbox>` + `show_all_label` (Key::EmployeesShowAll from Plan 17-03)
5. Render match updated from 3-arm `(Some(Ok), Some(Err), None)` to 3-arm `((Some(Ok), _), (Some(Err), _), (None, _))` pattern to handle both resources
6. When `show_all_val=true`: `combined.extend(unpaid_list)` before filter chain
7. Filter chain uses `employee_visible(e, show_all_val, &term)` for all employees

Three new filter tests:
- `filter_default_hides_unpaid`: show_all=false hides is_paid=false; shows is_paid=true
- `filter_show_all_reveals_unpaid`: show_all=true shows is_paid=false && !inactive
- `filter_inactive_always_hidden`: inactive persons hidden regardless of show_all or is_paid

All 627 frontend tests pass (627 vs 614 baseline — 13 new across all phase 17 plans).

### Task 3: WASM Build Gate

`cargo build --target wasm32-unknown-unknown` fails due to pre-existing NixOS `lld` linker limitation (not a code problem — pre-existing constraint documented in environment notes). Fell back to `cargo check --target wasm32-unknown-unknown` per plan instructions — exits with code 0 (Finished dev profile). WASM gate: PASSED (check).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Config move error in EmployeesList**
- **Found during:** Task 2 compilation
- **Issue:** `config` was moved into the first `use_resource` closure; the second closure for `load_unpaid_volunteer_employees` used it after the move — `E0382` use-of-moved-value
- **Fix:** Added `let config2 = config.clone()` before the first closure; used `config2` in the second closure
- **Files modified:** `shifty-dioxus/src/component/employees_list.rs`

## Security / Threat Model

T-17-10 (Information Disclosure): Mitigated — `Employee::unpaid_placeholder` sets `is_paid=false` on the SalesPerson, preserving the identity. All hours fields are 0.0. Backend gating from Plan 17-01 prevents paid_hours leak.

T-17-11 (Tampering — inactive leaks via show_all): Mitigated — `employee_visible` checks `!inactive` first, independently of `show_all`. `filter_inactive_always_hidden` test pins this.

T-17-12 (DoS — duplicate persons): Accepted — `load_unpaid_volunteer_employees` filters `!is_paid`; `load_employees` (via GET /report) returns only `is_paid=true` persons. Sets are disjoint by construction. Confirmed in SUMMARY.

## Known Stubs

None — all wired data. The unpaid resource always loads; the merge is gated on `show_all_val` at render time. No placeholder text or empty data flows to UI.

## Self-Check

### Files created/modified
- [x] `shifty-dioxus/src/state/employee.rs` — unpaid_placeholder + tests present
- [x] `shifty-dioxus/src/loader.rs` — load_unpaid_volunteer_employees present
- [x] `shifty-dioxus/src/component/employees_list.rs` — show_all signal, toggle UI, filter predicate, 3 tests present

### Tests
- [x] `unpaid_dummy_preserves_is_paid_false` — passes
- [x] `unpaid_dummy_has_zero_hours` — passes
- [x] `filter_default_hides_unpaid` — passes
- [x] `filter_show_all_reveals_unpaid` — passes
- [x] `filter_inactive_always_hidden` — passes
- [x] Full test suite: 627 passed, 0 failed

### Build gate
- [x] `cargo check --target wasm32-unknown-unknown` — Finished (exit 0)

## Self-Check: PASSED

## jj Commit Note

All changes are left uncommitted in the working tree per VCS policy (`commit_docs: false`, jj-managed repo). The user should commit the following files:
- `shifty-dioxus/src/state/employee.rs`
- `shifty-dioxus/src/loader.rs`
- `shifty-dioxus/src/component/employees_list.rs`
- `.planning/phases/17-contract-editor-unpaid-volunteer-path/17-04-SUMMARY.md`

Suggested jj commit message: `feat(17-04): show_all-Toggle + unbezahlte-Freiwillige-Merge in employees_list`

## CVC-10 Phase-Complete Candidacy

This plan closes CVC-10 (Mitarbeiteransicht has show_all-Filter; unbezahlte Freiwillige visible/selectable; is_paid-Gating via Plan 17-01; get_week-Seiteneffekt-Test via Plan 17-02). Together with Plans 17-01 through 17-04, Phase 17 is a phase-complete candidate for the v1.4 Milestone.
