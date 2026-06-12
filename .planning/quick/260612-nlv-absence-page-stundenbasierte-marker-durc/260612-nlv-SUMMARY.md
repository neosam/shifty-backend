---
quick_id: 260612-nlv
phase: quick
plan: 260612-nlv
subsystem: shifty-dioxus/page/absences
tags: [filter, absence, marker, frontend]
dependency_graph:
  requires: []
  provides: [marker_matches_filters, filtered_markers_rc, corrected-counters]
  affects: [AbsencesPage, AbsenceList]
tech_stack:
  added: []
  patterns: [pure-filter-function, TDD]
key_files:
  created: []
  modified:
    - shifty-dioxus/src/page/absences.rs
decisions:
  - "map_marker_category returns None for unmappable categories (ExtraWork/Holiday/Unavailable/VolunteerWork/Custom), causing them to be filtered out when a concrete category filter is active"
  - "filtered_markers_rc computed before total_count/filtered_count to keep ordering deterministic"
metrics:
  duration: ~10 minutes
  completed: 2026-06-12
---

# Quick Task 260612-nlv: Absence Page — stundenbasierte Marker durch Filter-Pipeline

**One-liner:** Added `marker_matches_filters` pure function and wired hourly markers through the category/person/status/show_past filter pipeline in `AbsencesPage`, with corrected `total_count`/`filtered_count` counters.

## Summary

The bug: `ExtraHoursMarker` entries from `ABSENCE_HOURLY_STORE` were passed raw (unfiltered) to `AbsenceList` and not counted in `total_count`/`filtered_count`. This caused the filter UI to appear broken when only hourly-based entries existed.

The fix:
1. New pure function `map_marker_category(&ExtraHoursCategoryTO) -> Option<AbsenceCategory>` mapping the three representable categories and returning `None` for all others.
2. New pure function `marker_matches_filters(marker, category_filter, person_filter, status_filter, show_past, today) -> bool` — exact analog of the inline Range-Absence filter closure, but for single-day markers.
3. `AbsencesPage` filter block extended with a `filtered_markers_rc` block using `marker_matches_filters`.
4. `total_count = absences.len() + hourly_markers.len()`.
5. `filtered_count = filtered_rc.len() + filtered_markers_rc.len()`.
6. `AbsenceList` now receives `filtered_markers_rc.clone()` instead of raw `hourly_markers.clone()`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | marker_matches_filters pure function + Unit-Tests | 8a620667 | shifty-dioxus/src/page/absences.rs |
| 2 | Marker durch Filter-Pipeline führen + Counter korrigieren | bb1eb652 | shifty-dioxus/src/page/absences.rs |

## Verification

- WASM-Gate: `nix develop -c cargo build --target wasm32-unknown-unknown` — exit 0 (43 pre-existing warnings, no new errors)
- Tests: `cargo test` — 565 passed, 0 failed (8 new marker_* tests all green)
- Clippy: pre-existing warnings only in absences.rs (unrelated to this task)
- Grep-Gates:
  - `grep -c "fn marker_matches_filters" src/page/absences.rs` = 1 (PASS)
  - `grep "hourly_markers: filtered_markers_rc" src/page/absences.rs` = match found (PASS)
  - `is_hr` count = 34 (unchanged vs. baseline — D-09 gate untouched, PASS)

## Deviations from Plan

### TDD Commit Granularity Note

The TDD RED phase was confirmed by running `cargo test marker_` BEFORE adding the implementation (compile errors confirmed function not yet defined). The GREEN phase was confirmed after adding the implementation. However, both tests and implementation ended up in the same jj change (`yrltlswp`) because jj has no staging area and the `jj describe` was done after both edits. The `feat` commit (`zpqkztvx`) contains only the Task 2 wiring changes.

No other deviations — plan executed as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `shifty-dioxus/src/page/absences.rs` modified: FOUND
- Commit 8a620667 exists: FOUND (jj log confirmed)
- Commit bb1eb652 exists: FOUND (jj log confirmed)
- All 565 tests pass: CONFIRMED
- WASM build exit 0: CONFIRMED
- `fn marker_matches_filters` present: CONFIRMED
- `hourly_markers: filtered_markers_rc` present: CONFIRMED
- D-09 is_hr count unchanged: CONFIRMED
