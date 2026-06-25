---
phase: 20-absences-indikator-jahres-histogramm
plan: "01"
subsystem: frontend-dioxus
tags: [uv-03, absences, i18n, ux, warning-indicator]
dependency_graph:
  requires: []
  provides: [UV-03-warn-indicator]
  affects: [shifty-dioxus/src/page/absences.rs, shifty-dioxus/src/i18n/]
tech_stack:
  added: []
  patterns: [SSR-test, TDD-red-green, i18n-3-locale]
key_files:
  created: []
  modified:
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "⚠️ indicator wraps existing column-1 div in flex row; inner flex-col preserves name+description layout"
  - "warn_tooltip variable fetched alongside other label variables at top of HourlyMarkerRow"
metrics:
  duration: "~15 min"
  completed: "2026-06-26"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 20 Plan 01: UV-03 ⚠️-Indikator auf stundenbasierten Absences-Markern

One-liner: Added ⚠️ warning indicator (with i18n title+aria-label in De/En/Cs) as leading element in HourlyMarkerRow column 1 to flag unconverted hourly markers on /absences.

## Tasks Completed

| # | Name | Status | Files |
|---|------|--------|-------|
| 1 | i18n-Key AbsenceHourlyWarnIndicator in De/En/Cs | Done | mod.rs, de.rs, en.rs, cs.rs |
| 2 | ⚠️-Indikator in HourlyMarkerRow + SSR-Test | Done | absences.rs |

## What Was Built

### Task 1: i18n Key
- Added `AbsenceHourlyWarnIndicator` to `Key` enum in `mod.rs` (Phase 8.5 Plan 06 marker group, after `AbsenceHourlyAmountLabel`)
- Added translations:
  - De: "Noch nicht in einen Zeitraum umgewandelt — bitte konvertieren"
  - En: "Not yet converted to a period — please convert"
  - Cs: "Dosud nepřevedeno na období — převeďte prosím"
- Added `Key::AbsenceHourlyWarnIndicator` to `i18n_absence_hourly_marker_keys_present_in_all_locales` test

### Task 2: ⚠️ Indicator + SSR Test
- In `HourlyMarkerRow`: fetched `warn_tooltip = i18n.t(Key::AbsenceHourlyWarnIndicator)` alongside other label variables
- Restructured column 1 div from `flex-col` to `flex items-start gap-1.5` to accommodate leading indicator
- Added `span` with `class: "shrink-0 text-warn text-body leading-none"`, `title`, `aria-label`, `role: "img"`, content `"⚠️"`
- Name+description remain in an inner `flex-col` div — existing layout preserved
- Existing `stundenbasiert` badge in column 3 unchanged
- Added SSR test `hourly_marker_row_renders_warn_indicator` asserting `html.contains("⚠")` and `html.contains("bitte konvertieren")`

## Test Results

- `cargo test` (shifty-dioxus): 635 passed, 0 failed
- HourlyMarkerRow tests: 5/5 green
- WASM build (`cargo build --target wasm32-unknown-unknown`): exit 0

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- `shifty-dioxus/src/page/absences.rs` - modified with ⚠️ indicator
- `shifty-dioxus/src/i18n/mod.rs` - Key::AbsenceHourlyWarnIndicator added
- `shifty-dioxus/src/i18n/de.rs` - De translation added
- `shifty-dioxus/src/i18n/en.rs` - En translation added
- `shifty-dioxus/src/i18n/cs.rs` - Cs translation added
- All 635 tests pass; WASM build exit 0
