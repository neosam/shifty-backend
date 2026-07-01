---
phase: 36-special-days-bugfixes
plan: "02"
subsystem: frontend
tags: [dioxus, controlled-input, settings, special-days, bugfix]
status: complete

dependency_graph:
  requires: []
  provides: [SelectInput-controlled-value, sd_type_to_select_value-helper]
  affects: [shifty-dioxus/src/component/form/inputs.rs, shifty-dioxus/src/page/settings.rs]

tech_stack:
  added: []
  patterns: [controlled-input-binding, optional-prop-backward-compat, TDD-red-green]

key_files:
  created: []
  modified:
    - shifty-dioxus/src/component/form/inputs.rs
    - shifty-dioxus/src/page/settings.rs

decisions:
  - "D-05: Added value: Option<ImStr> prop to SelectInputProps with #[props(!optional, default = None)] — when Some, binds the value attribute to the <select> element; when None, omits it for backward-compatible uncontrolled behavior."
  - "D-07: All 6 existing SelectInput callers (extra_hours_modal, billing_period_details, slot_edit, absences, text_template_management, plus Settings Card-3 onchange) compile unchanged — the optional prop pattern mirrors existing optional props in the codebase."
  - "D-06: sd_type_to_select_value(Option<SpecialDayTypeTO>) -> &'static str is the pure mapping helper mirroring the inverse of the onchange match; it is pub(crate) with unit tests."
  - "D-08: Confirmed date TextInput is already controlled via value: ImStr::from(sd_date_val.as_str()) — no change needed; sd_date_str.set(String::new()) already clears it visibly."

metrics:
  duration_minutes: 10
  completed_date: "2026-07-01"
  tasks_completed: 3
  files_modified: 2
---

# Phase 36 Plan 02: SDF-02 Controlled SelectInput for Settings Special-Days Card — Summary

Controlled select binding for the Settings Special-Days card so the Anlegen button reliably re-enables after each successful create.

## What Was Built

**Task 1 (RED + GREEN): Optional controlled `value` prop on SelectInput**

Added `value: Option<ImStr>` to `SelectInputProps` in `shifty-dioxus/src/component/form/inputs.rs`. When `Some`, the underlying `<select>` renders with a `value` attribute binding (controlled mode); when `None` (default), the attribute is omitted and the element is uncontrolled — backward compatible for all 6 existing callers (D-07).

Three new SSR regression tests guard the fix:
- `select_input_controlled_value_non_empty_reflected` — asserts `value="holiday"` renders on `<select>` (D-05).
- `select_input_controlled_empty_value_reflected` — asserts `value=""` renders on `<select>` for the post-reset empty case (D-05).
- `select_input_uncontrolled_when_no_value_prop` — asserts no `value=` attribute in the `<select>` opening tag when prop is absent (D-07 backward-compat).

**Task 2 (RED + GREEN): Wire Card-3 select + pure mapping helper**

Added `sd_type_to_select_value(Option<SpecialDayTypeTO>) -> &'static str` (module-level, `pub(crate)`) to `shifty-dioxus/src/page/settings.rs`. Maps `None -> ""`, `Holiday -> "holiday"`, `ShortDay -> "short_day"` — the inverse of the existing `onchange` match.

Passed `value: Some(ImStr::from(sd_type_to_select_value(sd_type_val.clone())))` to the Card-3 `SelectInput`. After `sd_type.set(None)` on successful create, the select's value attribute becomes `""` and the dropdown visibly resets, clearing the controlled-vs-uncontrolled desync that kept the Anlegen button disabled (SDF-02).

Confirmed (D-08): the date `TextInput` is already controlled via `value: ImStr::from(sd_date_val.as_str())`; `sd_date_str.set(String::new())` already clears it visibly — no change needed there.

**Task 3: Frontend gates**

- `cargo test -p shifty-dioxus form::inputs` — 21/21 passed.
- `cargo test -p shifty-dioxus settings` — 8/8 passed.
- `cargo test -p shifty-dioxus` full suite — 718 passed (1 pre-existing failure in `i18n::tests::i18n_impersonation_keys_match_german_reference`, see below).
- WASM build gate (`nix develop --command cargo build --target wasm32-unknown-unknown`) — passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SpecialDayTypeTO does not implement Copy**
- **Found during:** Task 2 GREEN compile
- **Issue:** `sd_type_val` (type `Option<SpecialDayTypeTO>`) was moved into `sd_type_to_select_value()` call, then used again in `if sd_type_val == Some(...)` — E0382 borrow of moved value.
- **Fix:** Added `.clone()` at the call site: `sd_type_to_select_value(sd_type_val.clone())`. No API change required.
- **Files modified:** `shifty-dioxus/src/page/settings.rs` (same commit, same task)

### Deferred Items

**Pre-existing i18n test failure (out of scope):**
- `i18n::tests::i18n_impersonation_keys_match_german_reference` fails in the base commit (`525f58c`) with a German translation mismatch ("🥸 Agieren" vs "Als diese Person agieren"). This is unrelated to the SelectInput or Settings Card-3 changes and was present before this plan. Logged to `deferred-items.md`.

## Verification Strategy

Per D-10/D-11 and D-25-06 (programmatic date inputs don't reliably trigger Dioxus signals in browser), verification is via cargo/SSR tests:
- SSR tests prove `SelectInput` reflects controlled value (including empty after reset) and stays uncontrolled without the prop.
- Helper unit test proves the `sd_type -> select-value` mapping used by the reset path.
- WASM build confirms no WASM-target compilation issues were introduced.
- No browser e2e required (tests provide sufficient coverage for the rendering logic; the live behavior follows from the correct signal binding).

## No New i18n Text

No new `Key::` entries were added. `git diff` on `src/i18n/` is empty — ROADMAP SC5 satisfied.

## TDD Gate Compliance

Plan frontmatter has `tdd="true"` on Tasks 1 and 2. Gate sequence completed:
- Task 1 RED commit: `1c921b7` (test(36-02): add RED tests...)
- Task 1 GREEN commit: `6ee4788` (feat(36-02): add optional controlled value prop...)
- Task 2 RED commit: `4b41a29` (test(36-02): add RED test for sd_type_to_select_value...)
- Task 2 GREEN commit: `c6b7f05` (feat(36-02): wire Card-3 select...)

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes. The fix is pure client-side rendering logic inside the existing shiftplanner-gated Settings card. The backend `SHIFTPLANNER_PRIVILEGE` check is unchanged.

## Self-Check: PASSED

- `shifty-dioxus/src/component/form/inputs.rs` — exists, modified.
- `shifty-dioxus/src/page/settings.rs` — exists, modified.
- Commit `1c921b7` exists (RED task 1).
- Commit `6ee4788` exists (GREEN task 1).
- Commit `4b41a29` exists (RED task 2).
- Commit `c6b7f05` exists (GREEN task 2).
- `cargo test -p shifty-dioxus form::inputs` — 21 passed.
- `cargo test -p shifty-dioxus settings` — 8 passed.
- WASM build — passed.
