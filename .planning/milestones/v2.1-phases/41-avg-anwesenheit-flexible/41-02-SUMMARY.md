---
phase: 41-avg-anwesenheit-flexible
plan: 02
subsystem: api
tags: [reporting, attendance, avg-01, hr-gate, is-dynamic, service]

requires:
  - phase: 41-avg-anwesenheit-flexible
    provides: 41-01 average_hours_per_attendance_day pure fn + EmployeeAttendanceStatistics struct
provides:
  - "ReportingService::get_employee_attendance_statistics trait method (service/src/reporting.rs)"
  - "ReportingServiceImpl::get_employee_attendance_statistics (HR-gate first → is_dynamic filter → report aggregate)"
  - "service_impl/src/test/reporting_attendance_gate.rs (2 mock tests: HR-gate + is_dynamic None)"
affects: [41-03, 41-04]

tech-stack:
  added: []
  patterns:
    - "HR_PRIVILEGE check as the first await; server-side is_dynamic filter returning Ok(None) for non-flexible employees"
    - "Read aggregate: flatten report.by_week[*].days into the 41-01 pure fn; no new DAO query, no persistence"

key-files:
  created:
    - service_impl/src/test/reporting_attendance_gate.rs
  modified:
    - service/src/reporting.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/mod.rs

key-decisions:
  - "HR_PRIVILEGE is the FIRST await; is_dynamic filter runs before the report fetch — both proven server-side via .times(0) mocks (D-AVG-05)"
  - "Reused find_by_sales_person_id (no all()+filter fallback needed — method exists with expected signature)"
  - "until_week clamped via time::util::weeks_in_year(year); aggregation over get_report_for_employee range (D-AVG-04)"
  - "No snapshot bump — CURRENT_SNAPSHOT_SCHEMA_VERSION stays 12 (D-AVG-08); A-22-1 untouched"

patterns-established:
  - "Optional metric return (Ok(None)) as the server-side scope filter for HR statistics"

requirements-completed: [AVG-01, AVG-02]

coverage:
  - id: D1
    description: "Trait method + impl: HR-gated, is_dynamic-filtered attendance statistics over the report range"
    requirement: "AVG-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_attendance_gate.rs::attendance_statistics_requires_hr (non-HR → Forbidden, .times(0) on data mocks)"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/reporting_attendance_gate.rs::attendance_statistics_returns_none_for_static (static employee → Ok(None), report not fetched)"
        status: pass
    human_judgment: false
  - id: D2
    description: "No snapshot bump + A-22-1 regression clean"
    requirement: "AVG-01"
    verification:
      - kind: build
        ref: "cargo test --workspace (all green, 568 service_impl unit tests) + grep CURRENT_SNAPSHOT_SCHEMA_VERSION = 12"
        status: pass
    human_judgment: false

duration: ~12min
completed: 2026-07-02
status: complete
---

# Phase 41 Plan 02: ReportingService attendance statistics (HR-gate + is_dynamic) Summary

**`ReportingService::get_employee_attendance_statistics` — HR_PRIVILEGE as the first await, server-side `is_dynamic` filter (non-flexible → `Ok(None)`), aggregating the 41-01 pure fn over the displayed report range; 2 mock tests prove auth-first and the scope filter, snapshot version stays 12.**

## Performance

- **Duration:** ~12 min
- **Tasks:** 3
- **Files created/modified:** 4

## Accomplishments
- Trait method `get_employee_attendance_statistics(&self, sales_person_id, year, until_week, context, tx) -> Result<Option<EmployeeAttendanceStatistics>, ServiceError>` added to `ReportingService` with D-AVG-04/05/08 doc.
- Impl in `ReportingServiceImpl`: (1) `check_permission(HR_PRIVILEGE)` first await; (2) `find_by_sales_person_id` + `is_dynamic` filter → `Ok(None)` for static employees; (3) `until_week.min(weeks_in_year(year))` clamp; (4) `get_report_for_employee` fetch; (5) flatten `by_week[*].days`; (6) `average_hours_per_attendance_day` pure fn → `Ok(Some(stats))`.
- Two mock tests in `reporting_attendance_gate.rs`: HR-gate (non-HR → Forbidden with `.times(0)` proving no data fetch) and is_dynamic filter (static → `Ok(None)` with report `.times(0)`).
- No new DAO query, no persistence, `billing_period_report.rs` untouched; A-22-1 unchanged.

## Task Commits

1. **Task 1: trait method declaration** - `673226a` (feat)
2. **Task 2: impl (HR-gate → is_dynamic → report → pure fn)** - `62d6f72` (feat)
3. **Task 3: mock tests HR-gate + is_dynamic** - `a814974` (test)

## Files Created/Modified
- `service/src/reporting.rs` - trait method `get_employee_attendance_statistics` (modified)
- `service_impl/src/reporting.rs` - impl of the method in `ReportingServiceImpl` (modified)
- `service_impl/src/test/reporting_attendance_gate.rs` - 2 mock gate tests (created)
- `service_impl/src/test/mod.rs` - registered `reporting_attendance_gate` module (modified)

## Decisions Made
- Used the existing `find_by_sales_person_id` (Uuid, Authentication::Full, tx) — the documented `all()`+filter fallback was not needed since the method exists with the expected signature.
- Passed `context` (not `Authentication::Full`) into `get_report_for_employee` so the report path honors the caller's authorization; work-details/is_dynamic lookup uses `Authentication::Full` as in the sibling report path.

## Deviations from Plan
None - plan executed as written. Note: the plan's Task-1 acceptance criterion anticipated `cargo build -p service` failing on the missing impl; in this workspace the impl lives in a separate crate (`service_impl`), so `service` builds green on its own — no functional deviation.

## Issues Encountered
Test call sites initially passed `()` for the context param; corrected to `Authentication::Full` (the method takes `Authentication<Self::Context>`). No impact on behavior.

## Scope Guard
- A-22-1 (`average_worked_hours_per_week`) byte-for-byte unchanged.
- No `BillingPeriodValueType` added; no persistence/migration.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 12 (grep-confirmed).

## Gate Results
- `cargo build --workspace`: green
- `cargo clippy --workspace -- -D warnings`: clean
- `cargo test --workspace`: all green (incl. 568 service_impl unit tests, 2 new gate tests)
- Snapshot grep: `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` (no bump)

## Next Phase Readiness
- Wave 3 (41-03) can expose the REST TO/endpoint over `get_employee_attendance_statistics`.
- No blockers.

## Self-Check: PASSED
- FOUND: service/src/reporting.rs (get_employee_attendance_statistics trait method)
- FOUND: service_impl/src/reporting.rs (impl)
- FOUND: service_impl/src/test/reporting_attendance_gate.rs
- FOUND commit: 673226a (Task 1)
- FOUND commit: 62d6f72 (Task 2)
- FOUND commit: a814974 (Task 3)

---
*Phase: 41-avg-anwesenheit-flexible*
*Completed: 2026-07-02*
