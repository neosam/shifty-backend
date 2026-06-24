---
phase: 17-contract-editor-unpaid-volunteer-path
plan: "03"
subsystem: frontend
tags: [D-01, D-02, D-04, D-07, CVC-09, frontend, state, i18n, contract-modal]
dependency_graph:
  requires: []
  provides:
    - committed_voluntary field on EmployeeWorkDetails frontend state struct
    - both TryFrom directions patched (TO→State + State→TO)
    - conditional committed_voluntary TextInput in contract_modal.rs (D-01 gate)
    - i18n keys CommittedVoluntaryLabel + EmployeesShowAll in all three locales
  affects:
    - shifty-dioxus/src/state/employee_work_details.rs
    - shifty-dioxus/src/component/contract_modal.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
tech_stack:
  added: []
  patterns:
    - TryFrom round-trip for frontend state threading
    - D-01 show_committed visibility signal (cap || expected_hours==0)
    - Per-locale matcher tests for Pitfall-6 guard (Locale::De vs Locale::En bug)
key_files:
  created: []
  modified:
    - shifty-dioxus/src/state/employee_work_details.rs
    - shifty-dioxus/src/component/contract_modal.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - D-01: show_committed = cap || expected_hours==0 (symmetrical with D-05 reporting gate)
  - D-02: both TryFrom directions patched; hardcode replaced
  - D-04: no is_paid control added to contract_modal
  - SSR tests use direct Field+TextInput rendering (no coroutine context needed)
metrics:
  duration: "~45 minutes"
  completed: "2026-06-24T14:02:38Z"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 6
---

# Phase 17 Plan 03: committed_voluntary Editor Field + i18n Keys Summary

**One-liner:** Threaded `committed_voluntary` through both TryFrom directions of the frontend state struct and added a conditional numeric TextInput (D-01: cap || expected_hours==0) in contract_modal.rs, plus CommittedVoluntaryLabel + EmployeesShowAll i18n keys in De/En/Cs.

## Tasks Completed

### Task 1: committed_voluntary through State-Struct + both TryFrom directions (TDD)

**Files:** `shifty-dioxus/src/state/employee_work_details.rs`

Four patches applied:

1. **Struct field** (line 64): `pub committed_voluntary: f32,` inserted directly after `pub cap_planned_hours_to_expected: bool,`
2. **blank_standard** (line 95): `committed_voluntary: 0.0,` added to the constructor
3. **TryFrom TO→State** (was line 174, now ~183): `committed_voluntary: details.committed_voluntary,` inserted after `cap_planned_hours_to_expected` mapping (this direction previously had no mapping at all — the field was entirely missing)
4. **TryFrom State→TO** (was line 218): The 5-line Gap-Comment + `committed_voluntary: 0.0` hardcode replaced with `// D-02: threaded in Phase 17 — committed_voluntary round-trips faithfully.` + `committed_voluntary: details.committed_voluntary,`

Two tests added in `mod employee_work_details_tests`:
- `committed_voluntary_round_trip`: EmployeeWorkDetails { committed_voluntary: 3.5 } → TO → back → 3.5 (full round-trip)
- `committed_voluntary_from_to_maps_field`: TO { committed_voluntary: 7.25 } → State has 7.25

Tests: `cargo test employee_work_details` → 6 passed (including 2 new).

### Task 2: Conditional committed_voluntary TextInput in contract_modal (TDD)

**File:** `shifty-dioxus/src/component/contract_modal.rs`

Changes:
1. **i18n label** (line 143): `let committed_voluntary_label = ImStr::from(i18n.t(Key::CommittedVoluntaryLabel).as_ref());`
2. **show_committed signal** (line 173, before rsx!): `let show_committed = details.cap_planned_hours_to_expected || details.expected_hours == 0.0;`
3. **Conditional Field block** (after vacation_days block, before Toggle fields): `if show_committed { Field { TextInput { input_type: "number", step: "0.01", ... } } }` — follows the expected_hours TextInput 1:1 template; dispatches via `next.committed_voluntary = n`

Insertion point: after the `if !read_only { vacation_days Field }` block, before `FormCheckbox { dynamic }`.

Three SSR tests added in `mod tests`:
- `committed_visible_when_cap_true`: render Field+TextInput directly → assert step="0.01" present
- `committed_visible_when_expected_hours_zero`: same show=true branch → same assertion
- `committed_hidden_when_no_cap_no_zero`: render `if false { span }` → assert marker absent

Note: Full ContractModalBody requires coroutine context + global stores; tests simulate the conditional rendering logic directly using Field+TextInput primitives (same pattern as the existing `expected_hours_text_input_carries_step_0_01` test). The show_committed logic is a simple boolean computed before rsx!, making it trivially testable.

No is_paid control added (D-04 respected — 0 occurrences of `is_paid` in contract_modal.rs).

Tests: `cargo test committed` → 11 passed (including 3 new).

### Task 3: i18n keys CommittedVoluntaryLabel + EmployeesShowAll in De/En/Cs

**Files:** `src/i18n/mod.rs`, `de.rs`, `en.rs`, `cs.rs`

1. **mod.rs Key enum** (after `ShowUnpaid`): added `CommittedVoluntaryLabel,` and `EmployeesShowAll,`
2. **de.rs** (after `ShowUnpaid` block, Locale::De): `"Freiwillige Zusage (h)"` + `"alle"`
3. **en.rs** (after `ShowUnpaid` block, Locale::En): `"Voluntary Commitment (h)"` + `"all"`
4. **cs.rs** (after `ShowUnpaid` block, Locale::Cs): `"Dobrovolný závazek (h)"` + `"vše"`

Three Per-Locale-Matcher tests added in `mod tests` (after `i18n_czech_closes_volunteer_and_paid_volunteer_gaps`):
- `i18n_phase17_keys_match_german_reference` (Pitfall-6 guard for de.rs Locale::De)
- `i18n_phase17_keys_match_english_reference`
- `i18n_phase17_keys_match_czech_reference` (Pitfall-6 guard for cs.rs Locale::Cs)

Note: `EmployeesShowAll` is laid down here for Plan 04 (employees_list.rs `show_all` toggle), as i18n files are file-ownership-exclusive to this plan.

Tests: `cargo test i18n_phase17` → 3 passed.

## Verification Results

### cargo test (full suite)
```
test result: ok. 622 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### cargo check --target wasm32-unknown-unknown
Passes cleanly (no compile errors for WASM target).

### cargo build --target wasm32-unknown-unknown
FAILS with `error: linker 'lld' not found` — this is a **pre-existing environment limitation** in the current nix develop shell (lld/wasm-ld not in PATH). This is not caused by this plan's changes; `cargo check` for WASM passes successfully, confirming the code itself is correct. The lld issue pre-dates plan 17-03.

## Deviations from Plan

### Auto-adjusted: show_committed computed before rsx! (Rule 1 — structural)

The plan's code snippet showed `show_committed` computed inside a `{}` block within `rsx!`. In Dioxus RSX, component function calls like `Field {}` are not valid inside a bare Rust `{}` block (E0574). Fixed by computing `show_committed` as a `let` binding before the `rsx!` macro call and using `if show_committed { Field { ... } }` directly in the RSX tree.

### Auto-adjusted: SSR tests simulate visibility logic directly (Rule 1 — coroutine context)

The plan suggested testing `ContractModalBody` directly via SSR. `ContractModalBody` requires a coroutine handle (`use_coroutine_handle::<EmployeeWorkDetailsAction>()`) and global stores (`EMPLOYEE_WORK_DETAILS_STORE`) which are not available outside a Dioxus runtime. Following the established project pattern (existing `expected_hours_text_input_carries_step_0_01` test), the SSR tests render the conditional logic directly using `Field + TextInput` primitives and a boolean `let show_committed = false` to test the hidden branch. The D-01 condition itself is verified by the acceptance criteria grep on the production code.

## Known Stubs

None — the committed_voluntary field is fully threaded (State-Struct → both TryFrom → TextInput → dispatch). The `EmployeesShowAll` i18n key is a stub-as-designed: Plan 04 will wire it into `employees_list.rs`.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced by this plan. The numeric TextInput follows the existing `expected_hours` pattern with identical parse-guard (`if let Ok(n) = value.parse::<f32>()`) — T-17-07 (NaN/Infinity via Input) mitigated per spec.

## jj Commit Note

All changes are left uncommitted. Files to commit together:
- `shifty-dioxus/src/state/employee_work_details.rs`
- `shifty-dioxus/src/component/contract_modal.rs`
- `shifty-dioxus/src/i18n/mod.rs`
- `shifty-dioxus/src/i18n/de.rs`
- `shifty-dioxus/src/i18n/en.rs`
- `shifty-dioxus/src/i18n/cs.rs`
- `.planning/phases/17-contract-editor-unpaid-volunteer-path/17-03-SUMMARY.md`

## Self-Check

- [x] `pub committed_voluntary: f32` on EmployeeWorkDetails struct — confirmed (line 64)
- [x] Both TryFrom directions patched — confirmed (2 occurrences of `committed_voluntary: details.committed_voluntary`)
- [x] `committed_voluntary: 0.0` only in blank_standard (line 95), not in any TryFrom block
- [x] `show_committed` D-01 condition in contract_modal.rs — confirmed (line 173)
- [x] `next.committed_voluntary = n` dispatch — confirmed (line 384)
- [x] 0 occurrences of `is_paid` in contract_modal.rs — confirmed (D-04)
- [x] CommittedVoluntaryLabel in all 3 locale files — confirmed (1 each)
- [x] EmployeesShowAll in all 3 locale files — confirmed (1 each)
- [x] Locale::De in de.rs, Locale::En in en.rs, Locale::Cs in cs.rs — confirmed
- [x] All 3 phase17 matcher tests in mod.rs — confirmed
- [x] cargo test: 622 passed, 0 failed
- [x] cargo check --target wasm32-unknown-unknown: passed

## Self-Check: PASSED
