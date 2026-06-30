---
phase: 24-paid-capacity-enforcement-config
plan: "05"
subsystem: ui
tags: [dioxus, frontend, shiftplan, paid-capacity, i18n, wasm]

dependency_graph:
  requires:
    - phase: 24-03
      provides: Key::BookingBlockedPaidLimit, Key::ShiftplanPaidOverageSectionHeader, Key::ShiftplanPaidOverageRow
    - phase: 24-01
      provides: 409 CONFLICT HTTP status for PaidLimitExceeded backend error
  provides:
    - Inline 409 hard-block error display at the shiftplan week view (D-24-05)
    - Persistent overage warning section for all roles above ShiftplanTabBar (D-24-03)
  affects: [24-verification, shiftplan-page]

tech-stack:
  added: []
  patterns:
    - "match arm guard on reqwest::StatusCode::CONFLICT before generic Err catch-all"
    - "slot-scoped Signal<Option<Uuid>> to track blocked slot; cleared on next success"
    - "client-side overage computation via filter_map over loaded Shiftplan.slots"
    - "i18n string placeholder substitution via .replace() chaining (mirrors warning_list.rs pattern)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/shiftplan.rs

key-decisions:
  - "Inline block error rendered just below the WeekView component (not inside it) — threading block_error into WeekView props would require modifying the component; rendering after the WeekView div is the minimal-touch approach consistent with 'same RSX scope'"
  - "block_error stores the slot_id (not just bool) to support per-slot scoping if WeekView is later extended; no per-slot rendering is done at this stage"
  - "Overage section computes i18n labels using time::format_description::parse with the same .unwrap() pattern used at line 117 of shiftplan.rs (static format string, safe)"
  - "block_error cleared on RemoveUserFromSlot as well as AddUserToSlot success — clearing on any positive booking action matches D-24-05 'clears on next success'"

requirements-completed: [D-24-03, D-24-05]

duration: ~25min
completed: 2026-06-27
tasks_completed: 2
tasks_total: 2
---

# Phase 24 Plan 05: Shiftplan Page — Overage Section + 409 Inline Block Error

**409 CONFLICT booking block wired inline at the shiftplan week view (D-24-05) and a persistent client-side overage section rendered above ShiftplanTabBar for all roles (D-24-03), both using 24-03 i18n keys.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-27T00:00:00Z
- **Completed:** 2026-06-27
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `block_error: Signal<Option<Uuid>>` signal to hold the hard-blocked slot id; cleared on any successful booking/removal action
- Added a new `Err(ShiftyError::Reqwest(...)) if status == CONFLICT` match arm in the `AddUserToSlot` handler (before the generic `Err(e)` catch-all), setting `block_error` instead of silently ignoring
- Rendered the inline block message (`text-bad text-small font-normal mt-1`, text `Key::BookingBlockedPaidLimit`) immediately below the WeekView in the week-mode RSX block — non-dismissible, disappears on next success
- Inserted the persistent overage section between the `booking_warnings` banner and the `ShiftplanTabBar` block: iterates `Shiftplan.slots` client-side, selects slots where `current_paid_count > max_paid_employees`, renders with `bg-warn-soft border border-warn rounded-md print:hidden`, heading `text-h2 font-semibold pb-2 text-warn` using `Key::ShiftplanPaidOverageSectionHeader`, and a list built from `Key::ShiftplanPaidOverageRow` with `{slot}/{current}/{max}` substitution; hidden when overage list is empty; no role gate (all roles)

## Task Summary

| # | Task | Status | Key Change |
|---|------|--------|-----------|
| 1 | Inline 409 hard-block error in AddUserToSlot (D-24-05) | Done | New CONFLICT arm + block_error signal + inline render |
| 2 | Persistent overage warning section (D-24-03) + WASM gate | Done | New overage section below booking_warnings banner |

## Files Created/Modified

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/page/shiftplan.rs`
  - Added `block_error: Signal<Option<Uuid>>` near line 206 (after `booking_warnings`)
  - Added `block_error` to coroutine `to_owned!` list
  - Added `block_error.set(None)` on `AddUserToSlot` Ok path (clears on success)
  - Added `block_error.set(Some(slot_id))` on 409 CONFLICT arm (new, before generic Err)
  - Added `block_error.set(None)` on `RemoveUserFromSlot` (clears on any removal)
  - Added inline `div { class: "text-bad text-small font-normal mt-1" }` after WeekView
  - Added persistent overage section between booking_warnings and ShiftplanTabBar

## Decisions Made

- **Inline error placement:** Rendered after `WeekView {}` in the same `div { class: "m-4" }` rather than threading into WeekView — minimal-touch; avoids WeekView API changes while keeping the message visually close to the week grid.
- **block_error signal stores Uuid** (not bool) so slot scoping can be extended later if WeekView grows a per-slot error prop.
- **Overage computation uses `time::format_description::parse(...).unwrap()`** following the existing pattern at shiftplan.rs:117 (static format strings are always valid).
- **Slot label format:** `"{weekday} HH:MM–HH:MM"` using `[hour]:[minute]` time format — matches the visual style of existing slot-time displays in the week view.

## Deviations from Plan

None — plan executed exactly as written.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| cargo build (native) | `cargo build` from `shifty-dioxus/` via `nix develop` | PASS (49 pre-existing warnings, 0 errors) |
| cargo test | `cargo test` — 669 passed, 0 failed | PASS |
| WASM build gate | `cargo build --target wasm32-unknown-unknown` | PASS (44 pre-existing warnings, 0 errors) |

(Clippy not run — dioxus workspace excluded from CI clippy gate per memory note; E0514 in dioxus shell.)

## Issues Encountered

None.

## Known Stubs

None — both surfaces are fully wired:
- Block error: reads `block_error` signal set by the actual 409 response handler
- Overage section: reads live `current_paid_count` and `max_paid_employees` from `shift_plan_context` (same source as WeekView render)

## Threat Flags

None — consistent with threat model in the plan (T-24-12 accepted, T-24-13 mitigated server-side, T-24-14 accepted via exact status match).

## Self-Check: PASSED

- `StatusCode::CONFLICT` arm in `src/page/shiftplan.rs`: FOUND
- `Key::BookingBlockedPaidLimit` in `src/page/shiftplan.rs`: FOUND
- `"text-bad text-small font-normal mt-1"` in `src/page/shiftplan.rs`: FOUND
- `Key::ShiftplanPaidOverageSectionHeader` in `src/page/shiftplan.rs`: FOUND
- `"bg-warn-soft border border-warn rounded-md"` in `src/page/shiftplan.rs`: FOUND
- `current_paid_count` in `src/page/shiftplan.rs`: FOUND
- `cargo build` (native): Finished dev profile exit 0
- `cargo test`: 669 passed, 0 failed
- `cargo build --target wasm32-unknown-unknown`: Finished dev profile exit 0

## Next Phase Readiness

- D-24-03 and D-24-05 frontend surfaces are complete
- Phase 24 backend (24-01 ServiceError, 24-02 enforcement guard) must ship for D-24-05 to be functionally exercised at runtime — the frontend code is compiled and waiting for the 409 response
- Phase 24-06 (Settings page) is independent and can proceed in parallel

---
*Phase: 24-paid-capacity-enforcement-config*
*Completed: 2026-06-27*
