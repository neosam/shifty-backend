---
phase: 37-modal-ux-politur
plan: "01"
subsystem: frontend/modal
tags: [tdd, ux, drag-safety, dialog, modal]
status: complete

dependency_graph:
  requires: []
  provides: [BackdropPress helper, drag-safe Dialog backdrop, drag-safe absence_convert_modal backdrop]
  affects: [dialog.rs, absence_convert_modal.rs, all Dialog users (contract_modal, extra_hours_modal, slot_edit, billing_periods, user_management, absences)]

tech_stack:
  added: []
  patterns: [signal-flag state machine, TDD RED/GREEN]

key_files:
  created: []
  modified:
    - shifty-dioxus/src/component/dialog.rs
    - shifty-dioxus/src/component/absence_convert_modal.rs

decisions:
  - "BackdropPress as a pub(crate) Copy struct with press_backdrop/press_panel/release methods — encodes D-01 rule as a pure testable state machine"
  - "use_signal(BackdropPress::default) in DialogContent — Dioxus reactive signal wrapping the helper; no web_sys target comparison"
  - "Panel onmousedown calls stop_propagation AND press_panel() — stop_propagation prevents event reaching backdrop; press_panel() clears any stale flag"
  - "Pre-existing i18n_impersonation_keys_match_german_reference test failure is out-of-scope (exists at base commit before this plan)"

metrics:
  duration: "7 minutes"
  completed_date: "2026-07-01"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 37 Plan 01: Central Drag-Safe Backdrop Close Summary

**One-liner:** BackdropPress signal-flag state machine in dialog.rs prevents panel-originated drag from closing any Dialog modal; identical fix applied inline to absence_convert_modal's own custom backdrop.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| RED  | Failing BackdropPress tests | fc5203c | dialog.rs |
| 1 GREEN | Central drag-safe backdrop in shared Dialog | fc91ec2 | dialog.rs |
| 2 GREEN | Apply signal-flag to absence_convert_modal | 493dd9e | absence_convert_modal.rs |

## What Was Built

### BackdropPress Helper (dialog.rs)

A `pub(crate)` Copy struct encoding the D-01 rule as a pure, unit-tested state machine:

```rust
pub(crate) struct BackdropPress { pressed_on_backdrop: bool }

impl BackdropPress {
    pub(crate) fn press_backdrop(&mut self) { self.pressed_on_backdrop = true; }
    pub(crate) fn press_panel(&mut self) { self.pressed_on_backdrop = false; }
    pub(crate) fn release(&mut self) -> bool {
        let was_pressed = self.pressed_on_backdrop;
        self.pressed_on_backdrop = false;
        was_pressed
    }
}
```

### DialogContent Wiring (dialog.rs)

`use_signal(BackdropPress::default)` declared in `DialogContent`. The backdrop `onmousedown` calls `press_backdrop()`. The panel `onmousedown` calls `stop_propagation()` + `press_panel()`. The backdrop `onclick` calls `on_close` only when `release()` returns true.

### absence_convert_modal.rs

Imports `BackdropPress` from `dialog.rs` and applies identical wiring to the component's own `fixed inset-0 bg-modal-veil` custom backdrop (D-03).

## Unit Tests Added (5 new backdrop_press tests, TDD D-10)

| Test | Scenario | Expected |
|------|----------|----------|
| `backdrop_press_new_release_returns_false` | New state, release() | false |
| `backdrop_press_panel_then_release_returns_false` | press_panel → release | false (MOD-01 core case) |
| `backdrop_press_backdrop_then_release_returns_true` | press_backdrop → release | true |
| `backdrop_press_backdrop_then_panel_clears_flag` | press_backdrop → press_panel → release | false |
| `backdrop_press_release_resets_flag` | Second release after true | false |

## Verification Gates

| Gate | Result | Notes |
|------|--------|-------|
| `cargo test -p shifty-dioxus component::dialog` | 26/26 PASS | 5 new + 21 pre-existing |
| `cargo test -p shifty-dioxus component::absence_convert_modal` | 4/4 PASS | Pre-existing SSR tests unchanged |
| `cargo test -p shifty-dioxus` (full suite) | 723/724 PASS | 1 pre-existing unrelated failure (see below) |
| WASM build (`--target wasm32-unknown-unknown`) | PASS | |
| Backend `cargo clippy --workspace -- -D warnings` | PASS | Backend unaffected |

## TDD Gate Compliance

- RED gate: `test(37-01)` commit `fc5203c` — failing BackdropPress stub tests (2 failures)
- GREEN gate: `feat(37-01)` commit `fc91ec2` — correct implementation, all tests pass

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Pre-existing Issues (out of scope)

**`i18n::tests::i18n_impersonation_keys_match_german_reference`** — fails at the base commit before this plan's changes. The test asserts `"Als diese Person agieren"` but finds `"🥸 Agieren"`. Pre-existing i18n string mismatch; no relation to MOD-01. Out of scope per scope boundary rule — logged to deferred-items.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The change is pure client-side modal open/close UI state. T-37-01 threat (low, accepted) — modal open/close gates no server action.

## Self-Check: PASSED

- [x] shifty-dioxus/src/component/dialog.rs exists with BackdropPress + wiring
- [x] shifty-dioxus/src/component/absence_convert_modal.rs imports and uses BackdropPress
- [x] Commits fc5203c, fc91ec2, 493dd9e exist in git log
- [x] 26 dialog tests pass, 4 absence_convert tests pass
- [x] WASM build green
- [x] Backend clippy green
