---
phase: 41-avg-anwesenheit-flexible
plan: 03
subsystem: api
tags: [reporting, attendance, avg-02, hr-gate, rest, transport, openapi]

requires:
  - phase: 41-avg-anwesenheit-flexible
    provides: 41-01 EmployeeAttendanceStatistics struct (From-Impl source)
  - phase: 41-avg-anwesenheit-flexible
    provides: 41-02 ReportingService::get_employee_attendance_statistics trait method
provides:
  - "rest-types: pub struct EmployeeAttendanceStatisticsTO (ToSchema) + From<&EmployeeAttendanceStatistics>"
  - "rest: get_attendance_statistics handler (#[utoipa::path]) + route /{id}/attendance-statistics + ReportApiDoc registration"
affects: [41-04]

tech-stack:
  added: []
  patterns:
    - "Feature-gated From<&ServiceStruct> for TO under #[cfg(feature = \"service-impl\")] (mirrors EmployeeWeeklyStatisticsTO)"
    - "Option<Service> -> maybe.as_ref().map(TO::from) serialized via serde_json -> None becomes JSON null with status 200"

key-files:
  created: []
  modified:
    - rest-types/src/lib.rs
    - rest/src/report.rs

key-decisions:
  - "Separate endpoint GET /report/{id}/attendance-statistics (range-aware year/until_week) instead of extending year-to-date /weekly-statistics — avoids breaking change (D-AVG-04)"
  - "Handler holds no auth logic; HR-gate + is_dynamic filter live in the 41-02 service, error_handler maps Forbidden -> 403 (D-AVG-05, T-41-06)"
  - "None (non-flexible / <2 attendance days) serializes to JSON null, status 200 (D-AVG-05/06)"
  - "No new DAO/sqlx query, no persistence, snapshot version stays 12 (D-AVG-08)"

patterns-established:
  - "Optional HR statistic exposed as nullable JSON body: service Option -> serde null, single 200 path"

requirements-completed: [AVG-02]

coverage:
  - id: D1
    description: "EmployeeAttendanceStatisticsTO (ToSchema) + From-impl in rest-types"
    requirement: "AVG-02"
    verification:
      - kind: build
        ref: "cargo build -p rest-types --all-features (green)"
        status: pass
    human_judgment: false
  - id: D2
    description: "HR-gated range-aware endpoint + route + ApiDoc registration"
    requirement: "AVG-02"
    verification:
      - kind: build
        ref: "cargo build --workspace + cargo clippy --workspace -- -D warnings (green/clean)"
        status: pass
      - kind: grep
        ref: "get_attendance_statistics in paths(...) and EmployeeAttendanceStatisticsTO in components(schemas(...)) of ReportApiDoc"
        status: pass
    human_judgment: false

duration: ~6min
completed: 2026-07-02
status: complete
---

# Phase 41 Plan 03: EmployeeAttendanceStatisticsTO + HR-gated attendance-statistics endpoint Summary

**`EmployeeAttendanceStatisticsTO` (rest-types, single source of truth for the FE) plus the range-aware, HR-gated `GET /report/{id}/attendance-statistics?year=Y&until_week=W` handler/route/ApiDoc — a pure read surface over the 41-02 service method; `None` (non-flexible employee) serializes to JSON `null` at status 200, non-HR gets 403 from the service gate.**

## Performance

- **Duration:** ~6 min
- **Tasks:** 2
- **Files created/modified:** 2

## Accomplishments
- `EmployeeAttendanceStatisticsTO` added to `rest-types/src/lib.rs` after `EmployeeWeeklyStatisticsTO`: `ToSchema` derive, fields `average_hours_per_attendance_day: Option<f32>`, `attendance_days: u32`, `total_worked_hours: f32`, plus `#[cfg(feature = "service-impl")] impl From<&service::reporting::EmployeeAttendanceStatistics>` (1:1 mapping).
- `get_attendance_statistics<RestState>` handler in `rest/src/report.rs`: `Query<ReportRequest>` (year + until_week) → `reporting_service().get_employee_attendance_statistics(...)`; result `Option<..>` mapped via `maybe_stats.as_ref().map(EmployeeAttendanceStatisticsTO::from)` and serialized — `None` becomes body `null`, status 200. Wrapped in `error_handler` (403 from service Forbidden). Annotated with `#[utoipa::path]` (params id/year/until_week; responses 200/403/500) + `#[instrument(skip(rest_state))]`.
- Route `.route("/{id}/attendance-statistics", get(get_attendance_statistics::<RestState>))` added to `generate_route`.
- `ReportApiDoc` extended: `get_attendance_statistics` in `paths(...)` and `EmployeeAttendanceStatisticsTO` in `components(schemas(...))` — endpoint is Swagger-visible.

## Task Commits

1. **Task 1: EmployeeAttendanceStatisticsTO + From-impl** - `88cfbe5` (feat)
2. **Task 2: HR-gated endpoint + route + ApiDoc** - `d953687` (feat)

## Files Created/Modified
- `rest-types/src/lib.rs` - `EmployeeAttendanceStatisticsTO` + feature-gated From-impl (modified)
- `rest/src/report.rs` - handler, route, ApiDoc registration, import (modified)

## Decisions Made
- Separate range-aware endpoint rather than extending `/weekly-statistics` (year-to-date) — no breaking change (D-AVG-04).
- Handler contains no auth logic; the HR-gate and `is_dynamic` filter stay server-side in the 41-02 service, `error_handler` maps Forbidden → 403.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## Scope Guard
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 12 (grep-confirmed, no bump).
- No new sqlx/DAO query, no persistence, no migration; A-22-1 (`weekly-statistics` / `EmployeeWeeklyStatisticsTO`) untouched.
- No new dependencies (T-41-SC).

## Gate Results
- `cargo build -p rest-types --all-features`: green
- `cargo build --workspace`: green
- `cargo clippy --workspace -- -D warnings`: clean
- `cargo test --workspace`: all green
- Grep: `get_attendance_statistics` in `paths(...)` and `EmployeeAttendanceStatisticsTO` in `components(schemas(...))` — confirmed
- Snapshot grep: `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` (no bump)

## Next Phase Readiness
- 41-04 (frontend) can consume `EmployeeAttendanceStatisticsTO` and call `GET /report/{id}/attendance-statistics`.
- No blockers.

## Self-Check: PASSED
- FOUND: rest-types/src/lib.rs (EmployeeAttendanceStatisticsTO + From-impl)
- FOUND: rest/src/report.rs (get_attendance_statistics handler + route + ApiDoc)
- FOUND commit: 88cfbe5 (Task 1)
- FOUND commit: d953687 (Task 2)

---
*Phase: 41-avg-anwesenheit-flexible*
*Completed: 2026-07-02*
