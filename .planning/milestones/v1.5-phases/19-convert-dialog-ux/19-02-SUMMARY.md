---
phase: 19-convert-dialog-ux
plan: "02"
subsystem: frontend
tags: [absence, convert-modal, ux, i18n, wasm]
dependency_graph:
  requires: [19-01]
  provides: [UV-01-frontend, UV-02-frontend]
  affects: [shifty-dioxus/src/state/absence_period.rs, shifty-dioxus/src/component/absence_convert_modal.rs, shifty-dioxus/src/page/absences.rs]
tech_stack:
  added: []
  patterns: [dioxus-ssr-tests, tdd-red-green]
key_files:
  created: []
  modified:
    - shifty-dioxus/src/state/absence_period.rs
    - shifty-dioxus/src/component/absence_convert_modal.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "D-19-FE-01: Frontend does zero calendar math; it only threads suggested_end/is_full_week from the TO to state to modal props."
  - "D-19-FE-02: end_str (Bis) initialised from suggested_end_str, not initial_str. start_str (Von) unchanged."
  - "D-19-FE-03: quantity_label branches on is_full_week: true -> AbsenceOneWeek i18n key, false -> existing N-Tage/1-Tag path."
metrics:
  duration: "~25 minutes"
  completed: "2026-06-26"
  tasks_completed: 3
  files_modified: 7
---

# Phase 19 Plan 02: Convert-Dialog UX – Frontend Wiring Summary

Frontend wiring of backend-computed `suggested_end` + `is_full_week` fields through FE state, modal props, and HourlyMarkerRow display, with i18n key `AbsenceOneWeek` in all three locales.

## Tasks Completed

### Task 1: Thread suggested_end + is_full_week through ExtraHoursMarker state + i18n key

- Added `pub suggested_end: time::Date` and `pub is_full_week: bool` to `ExtraHoursMarker` struct in `state/absence_period.rs`.
- Updated `From<&ExtraHoursMarkerTO>` impl to map both new fields 1:1 from the TO.
- Updated `sample_marker()` and `test_marker()` fixtures in `page/absences.rs` tests to include `suggested_end: when, is_full_week: false`.
- Added `AbsenceOneWeek` to `Key` enum in `i18n/mod.rs`.
- Added translations: De "1 Woche" (`Locale::De`), En "1 week", Cs "1 týden".
- Added 2 unit tests: `extra_hours_marker_roundtrips_suggested_end_and_is_full_week` and `extra_hours_marker_suggested_end_equals_when_for_half_day`.

### Task 2: Pre-fill modal 'bis' from suggested_end + pass it from call site

- Added `suggested_end: time::Date` prop to `AbsenceConvertModal`.
- `end_str` (Bis) now initialised from `suggested_end_str`, NOT `initial_str`.
- `start_str` (Von) remains initialised from `initial_date` — unchanged.
- P-7 submit-defense (parse + s<=e + inline error) — completely unchanged (D-19-04).
- Added `suggested_end: m.suggested_end` to the `AbsenceConvertModal { ... }` call site in `absences.rs`.
- Updated existing SSR tests to pass `suggested_end` prop.
- Added 2 new SSR tests: `absence_convert_modal_bis_prefilled_from_suggested_end` (asserts Von=initial_date, Bis=suggested_end) and `absence_convert_modal_half_day_von_equals_bis` (asserts both inputs = same date when suggested_end == initial_date).

### Task 3: HourlyMarkerRow shows "1 Woche" for full-week markers + WASM gate

- In `HourlyMarkerRow`, replaced the single `days_label + days_unit` computation with a `quantity_label` branch:
  - `marker.is_full_week == true` → `i18n.t(Key::AbsenceOneWeek).to_string()` ("1 Woche")
  - `marker.is_full_week == false` → existing `format!("{days_label} {days_unit}")` path unchanged
- Render site updated to `"{quantity_label} ({amount_str} {amount_label})"`.
- Added 2 SSR tests: `hourly_marker_row_full_week_shows_one_week_label` (asserts "1 Woche" present, "5 Tage" absent) and `hourly_marker_row_partial_week_shows_n_days_label` (asserts "3 Tage" present, "1 Woche" absent).
- WASM build gate passed via `nix develop --command cargo build --target wasm32-unknown-unknown`.

## Test Results

```
cargo test (shifty-dioxus): 634 passed; 0 failed — all tests green
cargo build --target wasm32-unknown-unknown (via nix develop): exit 0
```

New tests added:
- `state::absence_period::tests::extra_hours_marker_roundtrips_suggested_end_and_is_full_week`
- `state::absence_period::tests::extra_hours_marker_suggested_end_equals_when_for_half_day`
- `component::absence_convert_modal::tests::absence_convert_modal_bis_prefilled_from_suggested_end`
- `component::absence_convert_modal::tests::absence_convert_modal_half_day_von_equals_bis`
- `page::absences::tests::hourly_marker_row_full_week_shows_one_week_label`
- `page::absences::tests::hourly_marker_row_partial_week_shows_n_days_label`

## i18n Keys Added

| Key | De | En | Cs |
|-----|----|----|-----|
| `AbsenceOneWeek` | "1 Woche" | "1 week" | "1 týden" |

All three locales use the correct `Locale::De`, `Locale::En`, `Locale::Cs` variants (no historical `Locale::En`-for-De regression).

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — no new network endpoints or auth paths introduced. All changes are pure FE display wiring consuming read-only server-computed hints, staying within the existing T-19-04/T-19-05 accepted boundaries.

## Self-Check: PASSED

- `shifty-dioxus/src/state/absence_period.rs` — modified, contains `suggested_end` field
- `shifty-dioxus/src/component/absence_convert_modal.rs` — modified, contains `suggested_end` prop and `end_str` init from `suggested_end_str`
- `shifty-dioxus/src/page/absences.rs` — modified, contains `suggested_end: m.suggested_end` call-site pass-through and `AbsenceOneWeek` display branch
- `shifty-dioxus/src/i18n/mod.rs` — modified, contains `AbsenceOneWeek` key
- `shifty-dioxus/src/i18n/de.rs` — modified, De "1 Woche"
- `shifty-dioxus/src/i18n/en.rs` — modified, En "1 week"
- `shifty-dioxus/src/i18n/cs.rs` — modified, Cs "1 týden"
- All 634 tests passed; WASM build exit 0.
