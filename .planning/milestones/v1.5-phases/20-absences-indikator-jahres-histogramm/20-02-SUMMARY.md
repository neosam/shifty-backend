---
phase: 20-absences-indikator-jahres-histogramm
plan: 02
subsystem: ui
tags: [dioxus, svg, i18n, histogram, volunteer-hours, stacked-bars, ssr-tests]

# Dependency graph
requires:
  - phase: 20-absences-indikator-jahres-histogramm
    provides: "20-01: absence indicator in employee view (foundation for YV series)"
provides:
  - "YV-01: SVG <title> tooltip on every histogram bar showing KW + from–to date"
  - "YV-02: from–to date range in WeekListExpanded rows (two-line layout)"
  - "YV-03: stacked volunteer segment in histogram bars + separate volunteer value in WeekListExpanded and WeekDetailPanel"
  - "compute_max_y updated to account for stacked total (overall + volunteer)"
affects:
  - "employee_weekly_histogram.rs consumers"
  - "employee_view.rs WeekListExpanded/WeekDetailPanel"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pre-formatted date_labels prop pattern: outer wrapper component formats dates from i18n, passes as Rc<[(Rc<str>, Rc<str>)]> to inner View component to avoid I18N re-reads on every SSR frame"
    - "SVG <title> child element for hover tooltips (not HTML title attribute)"
    - "extract_text_labels() helper for cadence tests: parses <text> tags, ignoring <title> tooltip content"
    - "Stacked SVG rects: regular rect first (y=bar_y(overall)), volunteer rect second (y=bar_y(overall+volunteer), height=y_regular-y_volunteer)"

key-files:
  created: []
  modified:
    - "shifty-dioxus/src/component/employee_weekly_histogram.rs"
    - "shifty-dioxus/src/component/employee_view.rs"

key-decisions:
  - "D-20-02-01: overall_hours does NOT include volunteer_hours (confirmed from reporting.rs line 931: overall_hours = shiftplan_paid + extra_working_hours; volunteer is tracked separately). Stacking is additive and correct with no double-counting."
  - "D-20-02-02: Volunteer segment color = var(--ink-muted) opacity 0.35, consistent with weekly_overview_chart.rs reference."
  - "D-20-02-03: date_labels passed as Rc<[(Rc<str>, Rc<str>)]> prop (pre-formatted in outer EmployeeWeeklyHistogram) rather than calling I18N inside EmployeeWeeklyHistogramView — keeps the View pure for SSR test determinism."
  - "D-20-02-04: X-axis date augmentation deferred — bar labels appear every 4 weeks and adding date text there would cause overlap. Hover-title (YV-01) and KW-list date (YV-02) cover the navigation requirement without overloading the SVG axis."
  - "D-20-02-05: ssr_label_cadence_every_fourth_week test updated to use extract_text_labels() helper that parses only <text> SVG elements, ignoring <title> tooltip content — test intention preserved."

patterns-established:
  - "SVG tooltip pattern: use <title> as first child of <g> — not title HTML attribute"
  - "Stacked SVG bar geometry: regular segment anchors at bar_y(overall), volunteer segment fills gap between bar_y(overall+volunteer) and bar_y(overall)"

requirements-completed: [YV-01, YV-02, YV-03]

# Metrics
duration: 35min
completed: 2026-06-26
---

# Phase 20 Plan 02: YV-01/YV-02/YV-03 Summary

**Stacked SVG histogram bars (overall + volunteer) with per-bar <title> tooltips showing KW+date, and from-to date + volunteer value in WeekListExpanded/WeekDetailPanel**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-06-26
- **Completed:** 2026-06-26
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- YV-01: Every histogram bar `<g>` now has a SVG `<title>` child showing `KW {n} · {from} – {to}` (browser renders as native tooltip on hover)
- YV-02: Each WeekListExpanded row shows a two-line left column with `KW {n}` + a subdued date-range line `{from} – {to}`; WeekDetailPanel header already had the date (unchanged)
- YV-03: Histogram bars are split into a regular segment (`overall_hours`, existing color token) stacked below a volunteer segment (`volunteer_hours`, `var(--ink-muted)` opacity 0.35); `compute_max_y` now uses `overall + volunteer` as bar total; WeekListExpanded and WeekDetailPanel both show a separate volunteer value when `volunteer_hours > 0`

## Double-Count Finding (Claude's Discretion)

**`overall_hours` does NOT include `volunteer_hours`.**

Confirmed in `service_impl/src/reporting.rs` line 931:
```rust
let overall_hours = shiftplan_paid + extra_working_hours;
```
Volunteer hours are computed separately as `manual_volunteer_hours + auto_volunteer_hours + no_contract_volunteer`. Therefore stacking `overall_hours + volunteer_hours` is additive and correct — no double-counting occurs.

## Task Commits

No commits (GSD auto-commit is disabled for this repo; user commits via jj).

Files changed in working tree:
1. `shifty-dioxus/src/component/employee_weekly_histogram.rs` — compute_max_y stacked, View adds date_labels prop + `<title>` + volunteer rect
2. `shifty-dioxus/src/component/employee_view.rs` — WeekListExpanded date-range + volunteer; WeekDetailPanel volunteer section

## Files Created/Modified

- `shifty-dioxus/src/component/employee_weekly_histogram.rs` — Stacked bars (YV-03), per-bar SVG title (YV-01), updated compute_max_y, new date_labels prop flow, 5 new SSR tests (23 total, all green)
- `shifty-dioxus/src/component/employee_view.rs` — WeekListExpanded two-line layout with date + volunteer (YV-02/YV-03), WeekDetailPanel volunteer section (YV-03), 5 new SSR tests (15 total, all green)

## New i18n Keys

No new i18n keys. Existing keys reused:
- `Key::Volunteer` (De: "Freiwillig" / En: "Volunteer" / Cs: "Dobrovolné") — used for volunteer label in WeekListExpanded and WeekDetailPanel
- `Key::WeekShort` (De: "KW" / En: "W" / Cs: "T") — used in SVG `<title>` tooltip format string

## Decisions Made

1. **overall_hours vs volunteer stacking:** `overall_hours` is shiftplan_paid + extra_work only; `volunteer_hours` is separate → stacking is safe, no double-counting.
2. **date_labels as prop:** Pre-formatted in outer `EmployeeWeeklyHistogram`, passed as `Rc<[(Rc<str>, Rc<str>)]>` to inner `EmployeeWeeklyHistogramView`. Keeps View pure and testable via synthetic date labels without requiring I18N store in tests.
3. **X-axis date augmentation not implemented:** Bar labels appear every 4 weeks at 9px font. Adding date text below them would create unreadable overlap. The plan explicitly marks this as "Claude's Discretion" — YV-01 hover title + YV-02 KW-list date provide complete date navigation without axis overload.
4. **Label cadence test updated (Rule 1 auto-fix):** Adding `<title>` elements to each `<g>` caused `ssr_label_cadence_every_fourth_week` to falsely match "KW 2" in title tooltips. Fixed by introducing `extract_text_labels()` helper that parses only `<text>` elements.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ssr_label_cadence_every_fourth_week false-positive on <title> content**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** The test used `html.contains("KW 2")` which matched the new `<title>KW 2 · ...</title>` SVG tooltip elements, not just visible X-axis `<text>` labels. Test failed even though cadence logic was unchanged and correct.
- **Fix:** Introduced `extract_text_labels()` helper that extracts only `<text>...</text>` tag content; updated test to use it. Test intention (X-axis label cadence) preserved exactly.
- **Files modified:** `employee_weekly_histogram.rs` (test module only)
- **Verification:** Test passes; label cadence logic unchanged
- **Committed in:** (working tree — no commit, jj-managed repo)

---

**Total deviations:** 1 auto-fixed (Rule 1 - test logic bug introduced by new SVG title elements)
**Impact on plan:** Minimal. Test fix preserves original intention. Production code unaffected.

## Issues Encountered

None beyond the label cadence test fix above.

## Known Stubs

None — all data is wired from `WorkingHours.volunteer_hours`, `WorkingHours.from`, `WorkingHours.to` which are populated from the backend `WorkingHoursReportTO`.

## Threat Flags

None — frontend-only changes, no new network endpoints, no auth paths, no schema changes.

## Self-Check

### Files exist:
- `shifty-dioxus/src/component/employee_weekly_histogram.rs` — modified (exists)
- `shifty-dioxus/src/component/employee_view.rs` — modified (exists)

### Test results:
- `cargo test employee_weekly_histogram`: 23 passed, 0 failed
- `cargo test employee_view`: 15 passed, 0 failed
- `cargo test` (full suite): 645 passed, 0 failed
- WASM gate: `cargo build --target wasm32-unknown-unknown` exit 0

## Self-Check: PASSED

## Next Phase Readiness

- YV-01/YV-02/YV-03 complete; histogram and KW list show dates and volunteer hours
- No blockers; WASM gate green; all 645 tests pass

---
*Phase: 20-absences-indikator-jahres-histogramm*
*Completed: 2026-06-26*
