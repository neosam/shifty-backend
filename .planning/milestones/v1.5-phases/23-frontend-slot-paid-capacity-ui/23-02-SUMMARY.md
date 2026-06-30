---
phase: 23-frontend-slot-paid-capacity-ui
plan: 02
subsystem: shifty-dioxus (frontend)
tags: [frontend, dioxus, slot-editor, i18n, paid-capacity, ssr-tests]
requires:
  - "rest-types SlotTO.max_paid_employees (Phase 5, existing)"
  - "state/shiftplan.rs Slot.current_paid_count (Phase 6, existing)"
provides:
  - "Editable max_paid_employees field in the slot editor (Option<u8>, empty = no limit)"
  - "Display-only current_paid_count threaded page -> service -> store"
  - "Non-blocking inline overage banner when limit < current paid count"
  - "3 i18n keys (MaxPaidEmployeesLabel/Hint/OverageHint) in En/De/Cs"
affects:
  - "shifty-dioxus slot editor dialog (SlotEditInner)"
tech-stack:
  added: []
  patterns:
    - "Option<u8> number input: empty -> None, parseable u8 -> Some, parse-failure -> ignore"
    - "Display-only state field on dialog container (SlotEdit), never on server payload (SlotEditItem)"
    - "Non-blocking inline warn banner (no dialog, Save stays enabled)"
key-files:
  created: []
  modified:
    - shifty-dioxus/src/component/slot_edit.rs
    - shifty-dioxus/src/state/slot_edit.rs
    - shifty-dioxus/src/service/slot_edit.rs
    - shifty-dioxus/src/page/shiftplan.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "current_paid_count placed on SlotEdit (dialog container), not SlotEditItem (server payload) — Pitfall 2"
  - "Overage banner uses warn tokens (border-warn/bg-warn-soft), not bad — it is non-blocking (D-23-02)"
  - "Banner rendered as separate div after the Field, not via Field error prop (would mark field invalid)"
metrics:
  duration: ~25m
  tasks: 4
  files-modified: 8
  completed: 2026-06-27
---

# Phase 23 Plan 02: Slot Paid-Capacity Editor Summary

Added an editable `max_paid_employees` number field (empty = no limit, `Option<u8>`) to the
Dioxus slot editor, threaded a display-only `current_paid_count` from the loaded plan into the
editor so a non-blocking inline warn banner appears when the chosen limit is below the current
paid count, and added 3 i18n keys across En/De/Cs — all proven by SSR + coverage tests.

## What Was Built

### Task 1 — i18n keys + translations + coverage test
- Appended 3 `Key` variants to `i18n/mod.rs`: `MaxPaidEmployeesLabel`, `MaxPaidEmployeesHint`,
  `MaxPaidEmployeesOverageHint` (with doc comments).
- Added `add_text` entries in all 3 locales with exact contract strings (correct umlauts/diacritics):
  - En: "Max paid employees" / "Empty = no limit" / "Currently {current} paid ({limit} allowed)"
  - De: "Max. bezahlte Mitarbeiter" / "Leer = kein Limit" / "Aktuell {current} bezahlt (Limit: {limit})"
  - Cs: "Max. placených zaměstnanců" / "Prázdné = bez limitu" / "Aktuálně {current} placených (limit: {limit})"
- Added coverage test `i18n_slot_paid_capacity_keys_present_in_all_locales` (mirror of the
  booking-warning analog). `BookingWarningPaidLimitExceeded` left untouched.

### Task 2 — Thread current_paid_count (display-only)
- `state/slot_edit.rs`: added `current_paid_count: u8` to the `SlotEdit` struct + `new_edit()`.
  NOT added to `SlotEditItem` (server-write payload) — Pitfall 2 respected.
- `service/slot_edit.rs`: `LoadSlot(Uuid, u32, u8)` -> `LoadSlot(Uuid, u32, u8, u8)`;
  `load_slot_edit` gained a 4th param and sets `store.current_paid_count = current_paid_count;`;
  dispatch forwards the 4th arg; `new_slot_edit` sets `store.current_paid_count = 0` for the New path.
- `page/shiftplan.rs`: Edit-slot dropdown closure now looks up the slot in
  `shift_plan_context.read_unchecked()` (no reactive subscription) and passes its
  `current_paid_count` (default 0) as the 4th `LoadSlot` arg.

### Task 3 — max_paid_employees Field + inline banner + SSR tests (TDD)
- Added `current_paid_count: u8` prop to `SlotEditProps`; `SlotEdit` wrapper forwards
  `slot_edit.current_paid_count`.
- Added the `max_paid_employees` Field (FORM_INPUT_CLASSES, `type=number min=0`, `hint`) directly
  after `min_resources` and before the `has_errors` paragraph. Parsing: empty -> `None`,
  parseable `u8` -> `Some`, parse-failure -> silently ignored.
- Added the non-blocking overage `div` (`border-l-[3px] border-warn bg-warn-soft rounded-md p-2.5
  text-body text-ink`), visible only when `max_paid_employees.is_some_and(|n| current_paid_count > n)`.
  Save button untouched (stays enabled).
- Extended `mod tests` with the SSR harness (`render` + `pin_de_locale`) and 4 tests rendering
  `SlotEditInner` directly (Pitfall 4): value-rendered, empty-when-None, overage-banner-shown,
  no-banner-when-ok. TDD RED confirmed (3 of 4 failed before impl), then GREEN.

### Task 4 — Plan-wide gate
- `cargo test -- slot_edit i18n`: PASS (53 tests, 0 failed).
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus dev shell): PASS.
- `cargo clippy --workspace -- -D warnings`: 198 pre-existing workspace lints (matching the
  documented ~199 baseline). 0 NEW lints from this plan's code — verified by grepping the clippy
  output: zero hits in `component/slot_edit.rs`, `state/slot_edit.rs`, `i18n/en|de|cs.rs`, and all
  `page/shiftplan.rs` / `service/slot_edit.rs` / `i18n/mod.rs` hits fall on pre-existing lines
  outside this plan's edits (shiftplan edit region 662-675; all hits at 69/112-126/204-224/840/1012-1155).

## Deviations from Plan

### Minor adaptation (no behavior change)
- The visibility predicate uses `Option::is_some_and(...)` instead of the plan's literal
  `map_or(false, ...)`. `is_some_and` is the clippy-idiomatic equivalent (avoids a
  `clippy::unnecessary_map_or` lint under rustc 1.93) and is byte-for-byte semantically identical.
  Applied to keep this plan's code at 0 new clippy lints (Rule 1 — avoid introducing a lint that
  the hard gate would flag).

### Verify-command note
- The plan's `cargo test slot_edit i18n` is not valid cargo syntax (cargo test accepts a single
  positional filter). Ran as `cargo test -- slot_edit i18n` (both filters), all green. No code impact.

### Toolchain split (environment, not a code deviation)
- WASM build run in the `shifty-dioxus` dev shell (has `lld` + rustc 1.95.0 + wasm32 target).
- `cargo test` and `cargo clippy` run in the backend-root `nix develop` shell (matched rustc/clippy
  1.93.0) with `OPENSSL_NO_VENDOR=1` + OPENSSL_{DIR,LIB_DIR,INCLUDE_DIR} exported — per the
  documented toolchain quirk (clippy E0514 inside shifty-dioxus).

## Authentication Gates

None.

## Known Stubs

None. All wiring is end-to-end: editor writes `max_paid_employees` via the existing
`SlotTO` round-trip; `current_paid_count` is read live from the loaded plan into the editor.

## Threat Flags

None. No new endpoint, no new payload field (max_paid_employees round-trip pre-exists),
no new auth surface. `current_paid_count` is display-only and never written back.

## Self-Check: PASSED

- All 8 modified files + SUMMARY.md exist on disk.
- Markers verified: `MaxPaidEmployeesLabel` in slot_edit.rs, `current_paid_count` in state,
  "Max. bezahlte Mitarbeiter" in de.rs, `LoadSlot(Uuid, u32, u8, u8)` in service.
- `current_paid_count` is NOT present in `SlotEditItem` (server payload) — Pitfall 2 confirmed.
- 23-01's `week_view.rs` edits left untouched (I never opened it for writing).
- Gates: cargo test PASS; WASM build PASS; clippy 0 NEW lints (198 pre-existing baseline).
