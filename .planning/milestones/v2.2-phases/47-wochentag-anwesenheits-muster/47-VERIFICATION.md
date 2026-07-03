---
phase: 47-wochentag-anwesenheits-muster
verified: 2026-07-03T00:00:00Z
status: passed
score: 11/11 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 47: Wochentag-Anwesenheits-Muster Verification Report

**Phase Goal:** Die v2.1-„Ø Std/Anwesenheitstag"-Kennzahl durch eine pro-Wochentag-Anzeige (Anzahl + Prozent) im HR-Stats-Block des Mitarbeiter-Reports ersetzen — inkl. Endpoint-Umbau und i18n.
**Verified:** 2026-07-03
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### ROADMAP Success Criteria

| # | Success Criterion | Status | Evidence |
|---|-------------------|--------|----------|
| SC-1 | `/report/{id}/attendance-statistics` liefert pro Wochentag `count` + `share`, pure-fn getestet | VERIFIED | `service::reporting::weekday_attendance_distribution` in `service/src/reporting.rs:281`; 8 pure-fn tests green in `service_impl/src/test/reporting_weekday_attendance.rs` |
| SC-2 | `average_hours_per_attendance_day` weg; neue Zeile „Mo: 8 (80%) · …" an gleicher Stelle | VERIFIED | `grep average_hours_per_attendance_day` → 0 hits across rest-types/service_impl/service/rest/shifty-dioxus/src; `format_weekday_attendance_line` at `shifty-dioxus/src/component/employee_view.rs:47` + call site at line 584; SSR test `weekday_row_renders_all_seven_segments_when_populated` green |
| SC-3 | Snapshot bleibt 12; i18n de/en/cs Presence-Test grün | VERIFIED | `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12` at `service_impl/src/billing_period_report.rs:117`; `phase_47_weekday_i18n_presence` green (9 keys × 3 locales) |

### Observable Truths (Plan 47-01)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | GET /report/{id}/attendance-statistics returns `{ attendance_by_weekday: [7], counted_calendar_weeks }` for flexible employees, JSON null otherwise | VERIFIED | `service_impl/src/reporting.rs:1194–1235`: HR-gate at line 1204 (FIRST await), `is_dynamic` short-circuit at line 1213 → `Ok(None)`, weekday_attendance_distribution call at line 1233 |
| 2 | attendance_by_weekday ordered Mon..Sun with `{weekday, count: u32, share: f32 ∈ 0.0..=1.0}` | VERIFIED | `service/src/reporting.rs:281–326`: array indexed `weekday_index_mon0` in `std::array::from_fn`; share formula `(count/weeks).min(1.0)`; test `all_seven_weekdays_present` pins Mon..Sun order; test `share_never_exceeds_one` pins clamp; test `zero_weeks_yields_zero_shares_not_nan` pins finite |
| 3 | Response no longer contains average_hours_per_attendance_day / attendance_days / total_worked_hours | VERIFIED | `grep -rn "average_hours_per_attendance_day" rest-types service_impl shifty-dioxus/src service/src rest` → 0 hits; `EmployeeAttendanceStatisticsTO` at `rest-types/src/lib.rs:646` only has `attendance_by_weekday` + `counted_calendar_weeks` |
| 4 | HR privilege check is still the FIRST await; is_dynamic filter still short-circuits to Ok(None) | VERIFIED | `service_impl/src/reporting.rs:1202-1205` HR check is first await; lines 1213-1215 `is_dynamic` short-circuits before report fetch |
| 5 | CURRENT_SNAPSHOT_SCHEMA_VERSION stays at 12; no new BillingPeriodValueType variant | VERIFIED | `grep 'CURRENT_SNAPSHOT_SCHEMA_VERSION' service_impl/src/billing_period_report.rs` → line 117: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;` |

### Observable Truths (Plan 47-02)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 6 | v2.1 `Ø Std/Anwesenheitstag` line is gone; new weekday-distribution line at same slot | VERIFIED | `grep AvgHoursPerAttendanceDay shifty-dioxus/src` → 0 hits; call site `format_weekday_attendance_line` at `component/employee_view.rs:584` inside HR-stats block |
| 7 | Line renders as `Mo: 8 (80%) · Di: 3 (30%) · …` with 7 segments joined by `·` | VERIFIED | SSR test `weekday_row_renders_all_seven_segments_when_populated` (line 1721) asserts all 7 segments + ≥6 middle-dot separators + tooltip attribute |
| 8 | When `counted_calendar_weeks < 1`, a single localized placeholder is shown instead | VERIFIED | SSR test `weekday_row_renders_empty_state_when_counted_weeks_zero` (line 1774); formatter branch on `counted_calendar_weeks == 0` returns `Key::WeekdayAttendanceEmpty` |
| 9 | Row rendered only when `attendance_statistics.is_some()` (D-AVG-05 preserved) | VERIFIED | Call site guarded by `if let Some(att) = props.attendance_statistics.as_ref()` at line 583; SSR test `weekday_row_absent_when_statistics_is_none` (line 1804) pins this |
| 10 | Row carries a localized tooltip on hover | VERIFIED | `title="{i18n.t(Key::WeekdayAttendanceTooltip)}"` in render at line 584+; SSR test asserts `title=` attribute present |
| 11 | i18n de/en/cs have 7 weekday-short keys + tooltip + empty-state — presence-tested | VERIFIED | `Key::WeekdayShortMon..Sun` + `WeekdayAttendanceTooltip` + `WeekdayAttendanceEmpty` at `mod.rs:606-622`; `add_text` in de.rs:1062-1076, en.rs:987-1001, cs.rs:1052-1066; test `phase_47_weekday_i18n_presence` green |

**Score:** 11/11 truths verified

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| `service::reporting::WeekdayAttendanceStat` | VERIFIED | `service/src/reporting.rs:254` — struct with `weekday: DayOfWeek, count: u32, share: f32` |
| `service::reporting::EmployeeAttendanceStatistics` (new shape) | VERIFIED | `service/src/reporting.rs:265` — fields `attendance_by_weekday: [WeekdayAttendanceStat; 7]` + `counted_calendar_weeks: u32` |
| `weekday_attendance_distribution` pure fn | VERIFIED | `service/src/reporting.rs:281` — signature matches plan |
| `rest_types::WeekdayAttendanceTO` + `EmployeeAttendanceStatisticsTO` (new shape) | VERIFIED | `rest-types/src/lib.rs:624` + `:646` — DTO + From impl |
| `service_impl/src/test/reporting_weekday_attendance.rs` (TDD tests) | VERIFIED | File exists (9054 bytes), 8 `#[test]` fns registered in `test/mod.rs:76`, all green |
| FE `format_weekday_attendance_line` | VERIFIED | `shifty-dioxus/src/component/employee_view.rs:47` |
| 9 new i18n Key variants | VERIFIED | mod.rs:606–622 |
| 3 deleted v2.1 keys (`AvgHoursPerAttendanceDay*`) | VERIFIED | grep across shifty-dioxus/src → 0 hits |
| SSR test module `weekday_row_*` | VERIFIED | 3 tests + 1 formatter unit test at lines 1721, 1774, 1804, 1838 |

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| `ReportingServiceImpl::get_employee_attendance_statistics` | `service::reporting::weekday_attendance_distribution` | Direct call at `service_impl/src/reporting.rs:1233` with `all_days` + `counted_calendar_weeks` from `report.by_week.len()` | WIRED |
| `rest-types::EmployeeAttendanceStatisticsTO` | `service::reporting::EmployeeAttendanceStatistics` | `From<&…>` impl at `rest-types/src/lib.rs:654` maps `attendance_by_weekday` + `counted_calendar_weeks` 1:1 | WIRED |
| `rest/src/report.rs` `#[utoipa::path]` | `EmployeeAttendanceStatisticsTO` + `WeekdayAttendanceTO` | schemas() block at line 262 registers `WeekdayAttendanceTO`; handler at line 214 returns Option payload | WIRED |
| FE HR-stats block | `format_weekday_attendance_line` | Call site at `component/employee_view.rs:584` inside `if let Some(att)` guard | WIRED |

### RPT-03 Grep Gate Spot-Checks

| Check | Command | Result | Status |
|-------|---------|--------|--------|
| Snapshot version | `grep CURRENT_SNAPSHOT_SCHEMA_VERSION service_impl/src/billing_period_report.rs` | `117: pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;` | PASS |
| Old BE field removal | `grep -rn average_hours_per_attendance_day rest-types service_impl service/src rest shifty-dioxus/src` | 0 hits | PASS |
| Old FE i18n key removal | `grep -rn AvgHoursPerAttendanceDay shifty-dioxus/src` | 0 hits | PASS |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Backend pure-fn tests | `cargo test -p service_impl reporting_weekday_attendance` | 8 passed; 0 failed | PASS |
| FE i18n presence test | `cargo test -p shifty-dioxus phase_47_weekday` | 1 passed | PASS |
| FE SSR tests | `cargo test -p shifty-dioxus weekday_row` | 3 passed; 0 failed | PASS |

### Anti-Patterns Found

None. Grep for `TBD|FIXME|XXX|TODO|HACK|PLACEHOLDER` in the phase-modified files surfaced no unresolved debt markers.

## Gaps Summary

None. All 11 must-have truths VERIFIED with codebase evidence. All three ROADMAP success criteria satisfied. All three RPT-03 grep gates hold. Pure-fn and SSR test suites green in-process.

---

_Verified: 2026-07-03_
_Verifier: Claude (gsd-verifier)_
