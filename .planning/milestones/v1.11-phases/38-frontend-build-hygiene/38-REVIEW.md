---
phase: 38-frontend-build-hygiene
reviewed: 2026-07-01T17:30:03Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/component/add_extra_hours_form.rs
  - shifty-dioxus/src/component/base_components.rs
  - shifty-dioxus/src/component/day_aggregate_view.rs
  - shifty-dioxus/src/component/dialog.rs
  - shifty-dioxus/src/component/mod.rs
  - shifty-dioxus/src/component/top_bar.rs
  - shifty-dioxus/src/component/warning_list.rs
  - shifty-dioxus/src/component/week_view.rs
  - shifty-dioxus/src/loader.rs
  - shifty-dioxus/src/page/absences.rs
  - shifty-dioxus/src/page/billing_period_details.rs
  - shifty-dioxus/src/page/shiftplan.rs
  - shifty-dioxus/src/page/user_details.rs
  - shifty-dioxus/src/router.rs
  - shifty-dioxus/src/service/absence.rs
  - shifty-dioxus/src/service/billing_period.rs
  - shifty-dioxus/src/service/employee_work_details.rs
  - shifty-dioxus/src/service/text_template.rs
  - shifty-dioxus/src/service/theme.rs
  - shifty-dioxus/src/service/user_management.rs
  - shifty-dioxus/src/state/employee.rs
  - shifty-dioxus/src/state/shiftplan.rs
findings:
  critical: 0
  warning: 0
  info: 4
  total: 4
status: issues_found
---

# Phase 38: Code Review Report

**Reviewed:** 2026-07-01T17:30:03Z
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found (info-only; no blockers, no warnings)

## Summary

Phase 38 is a frontend build-hygiene pass over `shifty-dioxus` (38-01: `cargo fix` +
two deprecated-`parse` migrations; 38-02: removal of ~34 dead symbols plus 11
`#[allow(dead_code)]` suppressions). I reviewed the diff against `f340e65` with the
four deletion-phase risk vectors in mind (wrongly-deleted live code, `parse_borrowed`
correctness, lazy `#[allow(dead_code)]`, accidental behavior change) and verified the
result by compiling both targets.

**Verification performed (not just diff-reading):**
- `cargo check --target wasm32-unknown-unknown` — **compiles clean** (real deployment
  target; this is the check that would surface a wrongly-deleted wasm-gated caller,
  which native builds hide).
- `cargo clippy --all-targets` (native) — compiles; **zero new `dead_code`/`never used`
  warnings**, no errors. Remaining 175 warnings are the pre-existing dioxus style lints
  (D-08/D-09, explicitly out of scope; count is down from the ~198 baseline).

**Cross-checks on the highest-risk deletions:**
- Every removed `pub use` re-export (`AddExtraHoursForm`, `EmployeeWorkDetailsForm`,
  `EmployeesList`, `TupleRow`, `FormCheckbox`, `AbsenceConvertModal`, `AbsencesPage`)
  still has a live consumer via its **direct module path** — e.g. `employees_shell.rs`
  imports `component::employees_list::EmployeesList`, `employee_view.rs` imports
  `component::atoms::TupleRow`. No render site orphaned.
- All 11 `#[allow(dead_code)]` symbols are genuinely reachable through paths rustc
  cannot trace: `ColumnViewSlot` / `slot_to_column_view_item_with_tooltips` are invoked
  inside `rsx!` (week_view.rs:442/460/562), `parse_time_input` from an `rsx!` oninput
  handler + tests, and the theme/dialog helpers (`from_str`, `as_str`,
  `handle_system_theme_change`, `is_escape_key`) from `#[cfg(target_arch="wasm32")]`
  blocks (e.g. `handle_system_theme_change` at theme.rs:134).
- Deleted service wrapper `text_template::generate_custom_report` was safe to drop:
  `billing_period_details.rs:222` calls `loader::generate_custom_report` directly, not
  the service layer. The removed `UserManagementAction::LoadAll*` variants dropped only
  the enum arms; the underlying `load_all_*` fns remain called from other arms.

**`parse_borrowed::<2>` migration (shiftplan.rs:121, 1277):** correct. The format
strings `"[day].[month]"` and `"[hour]:[minute]"` use only plain components that are
identical between format-description v1 and v2, so semantics are unchanged; the borrow
is from `'static` string literals and the formatter is consumed in-scope; `.unwrap()`
error handling is preserved verbatim. Version `2` is the right (forward-looking) target.

**Behavior-change checks:** the `mut` removed from `block_error` (shiftplan.rs:211) is
legitimate — Dioxus `Signal` uses interior mutability, and all `.set()`/`.read()` calls
(lines 473, 492, 506, 534, 570, 636, 1323) remain, so the 409 hard-block feature is
fully intact. The `AbsenceModalEvent::Network` payload drop and the
`billing_period_details` coroutine→`use_effect` swap are functionally equivalent (see
Info below).

No blockers or warnings. The four Info items are documentation/judgment observations.

## Info

### IN-01: `has_sunday_slots` free helper suppressed with the weakest of the 11 justifications

**File:** `shifty-dioxus/src/component/day_aggregate_view.rs:171`
**Issue:** The `#[allow(dead_code)]` reason is *"no production caller yet, kept as
documented API for future use"* — the only references are its own `cfg(test)` tests.
This is the one suppression that does not meet the policy bar of "trait/signature
symmetry", "planned API tied to an **open requirement**", or "removal blows scope": it
is genuinely unused code (the sibling `DayAggregate::has_sunday_slots` *method* was
deleted in this same phase at `state/shiftplan.rs`), retained on a speculative "future
use" basis. Not a defect — the code compiles and is tested — but it is dead code
deferred rather than removed, which is the exact category this phase set out to clear.
**Fix:** Either cite the concrete open requirement/plan that will consume it (upgrading
the justification), or delete the function together with its three unit tests in a
follow-up. No action required for correctness.

### IN-02: `DialogVariant::Sheet` and `AddExtraHoursFormAction` suppressions defer, rather than resolve, dead code

**File:** `shifty-dioxus/src/component/dialog.rs:27` (`Sheet`),
`shifty-dioxus/src/component/add_extra_hours_form.rs:17` (`AddExtraHoursFormAction`)
**Issue:** Both reasons openly describe deferral: `Sheet` is a *"planned layout
variant"* kept because *"removing would also delete its tests (out of hygiene scope)"*,
and `AddExtraHoursFormAction` is *"used internally by … component [that] is unrendered
legacy code pending formal removal."* These are defensible "removal blows scope" calls
for a hygiene pass, but they leave known-dead constructs behind under an allow. Worth
tracking so they do not become permanent.
**Fix:** File a follow-up cleanup item for the unrendered `AddExtraHoursForm`
component + `AddExtraHoursFormAction`, and for `DialogVariant::Sheet` if no consumer
materializes. No correctness impact.

### IN-03: `billing_period_details` swapped a coroutine for `use_effect` — a behavior-adjacent change inside a deletion-only phase

**File:** `shifty-dioxus/src/page/billing_period_details.rs:67-69`
**Issue:** Removing the unused `BillingPeriodDetailsAction` enum required dropping the
`use_coroutine` that consumed it; it was replaced with a `use_effect` that fires the
initial `LoadBillingPeriod`. This is functionally equivalent — the effect body reads
only a plain `Copy` `Uuid` (`billing_period_id`) and a non-reactive coroutine handle, so
it subscribes to no signals and runs exactly once after first render, matching the
coroutine's single on-mount send. The only difference is timing (effect runs
post-render vs. coroutine spawned during build), with no observable effect here. Flagged
only because it is a logic refactor in a phase advertised as pure deletions/cleanup.
**Fix:** None needed; behavior verified equivalent. Note it in the phase summary so the
"deletions only" framing is accurate.

### IN-04: `AbsenceModalEvent::Network` dropped its `String` payload — confirm no downstream reads it

**File:** `shifty-dioxus/src/service/absence.rs:68`, consumed at
`shifty-dioxus/src/page/absences.rs:1182`
**Issue:** The variant changed from `Network(String)` to `Network`. Verified safe: the
sole consumer (absences.rs:1182) only clears `ABSENCE_MODAL_EVENT` and never rendered
the message, and the underlying error is still written to `ERROR_STORE` at the three
producer sites (absence.rs:142/163/199), which drives the global error banner. So the
user-visible error path is preserved and the payload was redundant.
**Fix:** None; documented for traceability.

---

_Reviewed: 2026-07-01T17:30:03Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
