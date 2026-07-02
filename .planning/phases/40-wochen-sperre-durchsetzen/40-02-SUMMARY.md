---
phase: 40-wochen-sperre-durchsetzen
plan: 02
subsystem: frontend
tags: [rust, dioxus, wasm, week-lock, i18n, button-mode, ux]

# Dependency graph
requires:
  - phase: 39-kw-status-grundlage
    provides: WEEK_STATUS_STORE, WeekStatus FE enum (Unset/InPlanning/Planned/Locked), red "Gesperrt" badge
  - phase: 40-wochen-sperre-durchsetzen
    plan: 01
    provides: ServiceError::WeekLocked + HTTP 423 (server-side enforcement this FE change complements)
provides:
  - "button_mode priority-2 branch: week_status == Locked && !is_shift_editor -> WeekViewButtonTypes::None (D-40-03)"
  - "Key::WeekLockedError i18n key + de/en/cs translations for the 423 response body (D-40-05)"
affects: [40-03, 40-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Subtractive UX gate: hide controls via existing WeekViewButtonTypes::None branch — no new component, no banner (D-40-04)"
    - "is_shift_editor (shiftplan.edit) is the bypass privilege, consistent with BE helper (D-40-02), NOT is_shiftplanner"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/shiftplan.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "button_mode priority order preserved: (1) change_structure_mode -> Dropdown, (2 NEW) Locked && !is_shift_editor -> None, (3) 2-week staleness && !is_hr -> None, (4) else AddRemove"
  - "Bypass privilege is shiftplan.edit (is_shift_editor), per locked decision D-40-02 — overrides RESEARCH/UI-SPEC which had mentioned is_shiftplanner"
  - "WeekLockedError key added to the existing WeekStatus namespace (WST-05); presence covered by extending i18n_week_status_keys_present_in_all_locales rather than a new test fn"
  - "No FE banner (D-40-04): the WeekLockedError key is not rendered yet — it serves the 423 response body and future use"

patterns-established:
  - "Reuse WeekViewButtonTypes::None to make +/- controls vanish from the DOM (show_add/show_remove=false) — DayAggregateView inherits the same button_mode automatically"

requirements-completed: [WST-03]

status: complete
---

# Phase 40 Plan 02: Wochen-Sperre FE Buttons ausblenden + i18n Summary

Purely subtractive frontend change: in a Locked week, non-`shiftplan.edit` users lose the +/- buttons (they disappear from the DOM via `WeekViewButtonTypes::None`), while `shiftplan.edit` holders keep them; plus a localized `WeekLockedError` i18n key in de/en/cs for the 423 response body. No banner, no new component (D-40-04).

## What Was Built

### Task 1: Hide +/- buttons in Locked week for non-shift-editors (D-40-03)
- Added a priority-2 `else if` branch in the `button_mode` computation in `shifty-dioxus/src/page/shiftplan.rs` (after `change_structure_mode`, before the 2-week staleness heuristic).
- Condition: `week_status == WeekStatus::Locked && !is_shift_editor` -> `WeekViewButtonTypes::None`.
- `WeekStatus` and `WeekViewButtonTypes` were already imported (lines 43, 19). `week_status` is a cloned value (no deref needed).
- `WeekViewButtonTypes::None` sets `show_add:false`/`show_remove:false` on every ColumnViewItem, so the existing `if props.item_data.show_add { … }` branches emit no DOM node. DayAggregateView inherits the same button_mode automatically. No new Tailwind class, no new component. The Phase-39 red "Gesperrt" badge stays unchanged.
- Commit: `15c665f`

### Task 2: i18n WeekLockedError de/en/cs + presence test (D-40-05)
- Added `Key::WeekLockedError` to `pub enum Key` in `mod.rs` (WST-05 WeekStatus namespace).
- Translations:
  - de: "Diese Woche ist gesperrt — Änderungen sind nicht möglich."
  - en: "This week is locked — changes are not possible."
  - cs: "Tento týden je uzamčen — změny nejsou možné."
- Extended `i18n_week_status_keys_present_in_all_locales` to cover `WeekLockedError` across all three locales (non-empty, non-"??").
- Commit: `a1f6914`

## Deviations from Plan

None - plan executed exactly as written. `WeekStatus` was already imported, so no `use` statement was needed. The plan's naming fallback ("or matching Phase-39 namespace") resolved to `WeekLockedError` as specified.

## Verification / Gate Results

- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus/): green (23.4s).
- `cargo test -p shifty-dioxus i18n`: new/updated i18n presence test green; only failure is the pre-existing, out-of-scope `i18n_impersonation_keys_match_german_reference` (explicitly not this plan's — untouched).
- `cargo clippy --workspace -- -D warnings` (backend root): green.

## Known Stubs

- `Key::WeekLockedError` is defined and translated but not yet rendered in the FE (D-40-04: no banner). It is intended for the 423 response body and potential future use. This is an intentional, documented stub — not a gap.

## Self-Check: PASSED
- shifty-dioxus/src/page/shiftplan.rs: FOUND (button_mode Locked branch)
- shifty-dioxus/src/i18n/{mod,de,en,cs}.rs: FOUND (WeekLockedError key + translations)
- Commit 15c665f: FOUND
- Commit a1f6914: FOUND
