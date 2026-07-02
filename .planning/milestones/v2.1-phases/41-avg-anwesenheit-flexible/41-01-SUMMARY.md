---
phase: 41-avg-anwesenheit-flexible
plan: 01
subsystem: api
tags: [reporting, pure-function, tdd, attendance, avg-01]

requires:
  - phase: 22-reporting-avg-weekly
    provides: A-22-1 average_worked_hours_per_week pure-fn as style template (untouched)
provides:
  - EmployeeAttendanceStatistics result struct in service/src/reporting.rs
  - average_hours_per_attendance_day pure fn (day-based AVG-01 metric)
  - service_impl/src/test/reporting_avg_attendance.rs (7 Nyquist pure-fn cases)
affects: [41-02, 41-03, 41-04]

tech-stack:
  added: []
  patterns:
    - "Pure aggregate fn over &[WorkingHoursDay] with BTreeSet<time::Date> date dedup"
    - "Category matches! filter (Shiftplan|ExtraWork|VolunteerWork, hours>0) for attendance classification"

key-files:
  created:
    - service_impl/src/test/reporting_avg_attendance.rs
  modified:
    - service/src/reporting.rs
    - service_impl/src/test/mod.rs

key-decisions:
  - "Separate function + result struct from A-22-1 (different input type &[WorkingHoursDay], different name) — A-22-1 byte-for-byte unchanged"
  - "Denominator = DISTINCT dates via BTreeSet; numerator = sum of work-category hours; <2 days → None (D-AVG-06)"
  - "No snapshot bump — CURRENT_SNAPSHOT_SCHEMA_VERSION stays 12 (D-AVG-08)"

patterns-established:
  - "Attendance-day classification via category matches! + positive-hours guard"
  - "Date deduplication via BTreeSet<time::Date>"

requirements-completed: [AVG-01]

coverage:
  - id: D1
    description: "Pure fn average_hours_per_attendance_day + EmployeeAttendanceStatistics struct implementing the day-based AVG-01 metric"
    requirement: "AVG-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_avg_attendance.rs (7 cases: user_example, absence_day_not_counted, mixed_day_counts_work_only, custom_category_not_attendance, empty_slice_returns_none, one_day_returns_none, two_days_returns_some)"
        status: pass
    human_judgment: false
  - id: D2
    description: "A-22-1 regression unchanged + no snapshot bump (version stays 12)"
    requirement: "AVG-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_avg_weekly.rs (9 cases pass) + grep CURRENT_SNAPSHOT_SCHEMA_VERSION = 12"
        status: pass
    human_judgment: false

duration: ~10min
completed: 2026-07-02
status: complete
---

# Phase 41 Plan 01: Ø-Anwesenheit pure Aggregat-Funktion Summary

**Day-based AVG-01 metric `average_hours_per_attendance_day` (+ `EmployeeAttendanceStatistics` struct) as a mock-free pure fn over `&[WorkingHoursDay]`, with 7 Nyquist tests — A-22-1 and snapshot version 12 untouched.**

## Performance

- **Duration:** ~10 min
- **Tasks:** 2 (RED + GREEN)
- **Files modified:** 3

## Accomplishments
- `EmployeeAttendanceStatistics { average_hours_per_attendance_day: Option<f32>, attendance_days: u32, total_worked_hours: f32 }` added below A-22-1.
- Pure fn `average_hours_per_attendance_day`: filters work-categories (Shiftplan|ExtraWork|VolunteerWork, hours>0), dedups dates via `BTreeSet<time::Date>`, returns None below 2 attendance days (D-AVG-06).
- New test module `reporting_avg_attendance.rs` with all 7 Nyquist cases; registered in `test/mod.rs`.
- A-22-1 (`average_worked_hours_per_week`) unchanged; snapshot version stays 12.

## Task Commits

1. **Task 1 (RED): Nyquist test module** - `466c288` (test)
2. **Task 2 (GREEN): pure fn + struct** - `7f2cda0` (feat)

## Files Created/Modified
- `service_impl/src/test/reporting_avg_attendance.rs` - 7 pure-fn Nyquist tests (created)
- `service/src/reporting.rs` - EmployeeAttendanceStatistics struct + average_hours_per_attendance_day fn (modified)
- `service_impl/src/test/mod.rs` - registered `reporting_avg_attendance` module (modified)

## Decisions Made
None beyond the plan — followed 41-01-PLAN.md as specified. Kept A-22-1 byte-for-byte unchanged; separate function, separate input type, separate result struct.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Wave 2 (41-02) can now wire the pure fn into `ReportingService::get_employee_attendance_statistics` (HR-gate → is_dynamic → report fetch → pure fn).
- No blockers.

## Self-Check: PASSED
- FOUND: service_impl/src/test/reporting_avg_attendance.rs
- FOUND: service/src/reporting.rs (EmployeeAttendanceStatistics + average_hours_per_attendance_day)
- FOUND commit: 466c288 (RED)
- FOUND commit: 7f2cda0 (GREEN)

---
*Phase: 41-avg-anwesenheit-flexible*
*Completed: 2026-07-02*
