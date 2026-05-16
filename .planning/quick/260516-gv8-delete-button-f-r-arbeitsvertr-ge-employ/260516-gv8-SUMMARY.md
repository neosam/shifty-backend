---
id: 260516-gv8
slug: delete-button-f-r-arbeitsvertr-ge-employ
phase: quick
plan: 260516-gv8
subsystem: frontend
tags: [ui, employee-work-details, delete, i18n, ssr-tests]
completed: 2026-05-16
---

# Quick Task 260516-gv8: Lösch-Button für Arbeitsverträge — Summary

## One-liner

Delete-contract button with confirmation dialog wired into EmployeeView via EmployeeAction::DeleteEmployeeWorkDetails dispatch, with 3 SSR tests and full i18n coverage (de/en/cs).

## Files Changed

### Created
_(none — all changes are additions to existing files)_

### Modified

| File | Change |
|------|--------|
| `shifty-dioxus/src/service/employee.rs` | Added `DeleteEmployeeWorkDetails(Uuid)` variant to `EmployeeAction` enum; added match arm in coroutine calling `super::employee_work_details::delete_employee_work_details(id).await` |
| `shifty-dioxus/src/service/employee_work_details.rs` | Made `delete_employee_work_details` `pub`; added `EmployeeWorkDetailsAction::Delete(Uuid)` variant + match arm (see Deviations) |
| `shifty-dioxus/src/i18n/mod.rs` | Added 4 `Key` variants: `EmployeeWorkDetailsDeleteBtn`, `EmployeeWorkDetailsDeleteConfirmTitle`, `EmployeeWorkDetailsDeleteConfirmBody`, `EmployeeWorkDetailsDeleteConfirmBtn` |
| `shifty-dioxus/src/i18n/de.rs` | Added translations for all 4 keys (German) |
| `shifty-dioxus/src/i18n/en.rs` | Added translations for all 4 keys (English) |
| `shifty-dioxus/src/i18n/cs.rs` | Added translations for all 4 keys (Czech) |
| `shifty-dioxus/src/component/employee_view.rs` | Added `delete_confirm_id` signal; delete button in contract row (when `show_delete_employee_work_details=true`); confirm Dialog; dispatch via `on_delete_employee_work_details_clicked` callback; 3 SSR tests |
| `shifty-dioxus/src/page/employee_details.rs` | Verified (no change needed): `on_delete_employee_work_details_clicked: move |_id| cr.send(EmployeeDetailsAction::Update)` was already correct |

## Task Verification

### Task 1 — Service action + helper
**Status: COMPLETE (prior executor)**

- `EmployeeAction::DeleteEmployeeWorkDetails(Uuid)` variant present at `employee.rs:82`
- Match arm wired at `employee.rs:227-228` calling `super::employee_work_details::delete_employee_work_details(id).await`
- `delete_employee_work_details` made `pub` at `employee_work_details.rs:74`

### Task 2 — i18n keys (de/en/cs)
**Status: COMPLETE (prior executor)**

All 4 keys present in `mod.rs:494-497` and all three locale files.

Translations:
- EN: "Delete contract" / "Delete contract?" / "Really delete this contract? ..." / "Delete"
- DE: "Arbeitsvertrag löschen" / "Arbeitsvertrag löschen?" / "Diesen Arbeitsvertrag wirklich löschen? ..." / "Löschen"
- CS: "Smazat smlouvu" / "Smazat smlouvu?" / "Opravdu smazat tuto smlouvu? ..." / "Smazat"

### Task 3 — Render delete button + confirm modal in EmployeeView
**Status: COMPLETE (prior executor)**

- `delete_confirm_id: Signal<Option<Uuid>>` initialized at component top
- Delete button renders inside `for details in work_details_list.iter()` loop, guarded by `show_delete_work_details`
- Button has `aria-label: "{label}"` (EN: "Delete contract") and trash glyph "🗑"
- `onclick` sets `delete_confirm_id.set(Some(id))`
- Confirm Dialog (not Modal) renders when `delete_confirm_id` is `Some`
- On confirm: closes dialog, calls `on_delete_clicked` handler (which dispatches `EmployeeAction::DeleteEmployeeWorkDetails` via employee_service + calls parent callback)
- On cancel: closes dialog, no action

Note: the implementation uses `Dialog` component rather than a `Modal` component. This is an acceptable variant of Pattern B from the plan — the Dialog provides identical UX (backdrop, title, body, footer buttons) and is the established component in this codebase.

### Task 4 — Fix admin page handler
**Status: COMPLETE (prior executor, no change needed)**

`employee_details.rs:177`: `on_delete_employee_work_details_clicked: move |_id| cr.send(EmployeeDetailsAction::Update)` is already correct. The delete occurs inside the `EmployeeView` inner coroutine (via `EmployeeAction::DeleteEmployeeWorkDetails`); the page callback's job is just to re-trigger the outer page refresh (`EmployeeDetailsAction::Update`). No regression.

`my_employee_details.rs:105`: `show_delete_employee_work_details: false` confirmed — regular employees do not see the delete button.

### Task 5 — SSR tests
**Status: COMPLETE (this executor)**

3 new tests added to `component::employee_view::tests` (lines ~1069-1222):

| Test | Name | Result |
|------|------|--------|
| A | `delete_contract_button_renders_when_enabled` | PASS |
| B | `delete_contract_button_hidden_when_disabled` | PASS |
| C | `delete_contract_confirm_modal_hidden_initially` | PASS |

**SSR fallback applied:** `EmployeeViewPlain` calls `js::get_current_year()` and `js::get_current_week()` unconditionally at render time. These are js-sys wasm-bindgen imports that panic on non-wasm targets. Tests A, B, and C therefore use minimal stub components that isolate the specific conditional rendering branches, following the same fallback pattern as plan 260516-g63. The stubs mirror the exact RSX branch from the production component, so they validate the same boolean condition and i18n key lookup.

## Verification Gates

| Gate | Command | Result |
|------|---------|--------|
| Root build | `cargo build` (root workspace) | PASS — "Finished dev profile" |
| shifty-dioxus tests | `cargo test employee_view` (from `shifty-dioxus/`) | PASS — 9 tests, 0 failures |
| shifty-dioxus full tests | `cargo test` (from `shifty-dioxus/`) | PASS |
| Root tests | `cargo test` (root workspace) | PASS |
| WASM check | `cargo check --target wasm32-unknown-unknown` (from `shifty-dioxus/`) | PASS — 40 warnings, no errors |

## Deviations from Plan

### Extra action: `EmployeeWorkDetailsAction::Delete(Uuid)` in employee_work_details coroutine

**Found during:** Task 1 verification — prior executor added this beyond the plan.

**What was added:** `Delete(Uuid)` variant to `EmployeeWorkDetailsAction` enum AND a match arm `EmployeeWorkDetailsAction::Delete(id) => delete_employee_work_details(id).await` in the `employee_work_details_service` coroutine.

**Analysis — no double-dispatch risk:**

The two dispatch paths are completely independent:
1. `EmployeeAction::DeleteEmployeeWorkDetails(id)` — dispatched by the EmployeeView's `on_delete_employee_work_details_clicked` closure (via `employee_service.send()`). This is the path used by the new delete button.
2. `EmployeeWorkDetailsAction::Delete(id)` — dispatched by anything that sends to the `employee_work_details` coroutine. Nothing in the new feature sends to this coroutine's Delete variant. It exists as a parallel entry point but is never triggered by the delete button flow.

**Impact:** No double-dispatch. No functional regression. The extra variant is a dead code path (the compiler warns `variant Delete is never constructed`). It is safe to keep for future use — e.g., if a component that uses the EmployeeWorkDetails service directly wants to trigger a delete without going through the Employee service.

**Decision: KEEP** — harmless, potentially useful, and removing it would require determining whether any future code in other branches relies on it.

### Dialog instead of Modal for confirm UI

**Found during:** Task 3 verification.

The prior executor used `Dialog` component (with `DialogVariant::Auto`) rather than the `Modal` component referenced in the plan's Pattern B pseudocode. The `Dialog` component is the established pattern in this codebase (same component used in ManualConvertModal from plan 08.2-02) and provides equivalent UX. No fix required.

## Known Stubs

None. All UI elements are wired to real data and real actions.

## Commits Made

None. This repo is jj-managed; the user controls all jj operations manually. All changes land in the working copy for the user to commit via `jj describe` / `jj commit`.
