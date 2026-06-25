---
phase: 22-mitarbeiter-statistik-hr
plan: "01"
subsystem: backend
tags: [reporting, statistics, hr, rest-api, openapi, unit-tests]
dependency_graph:
  requires: []
  provides: [GET /report/{id}/weekly-statistics, EmployeeWeeklyStatisticsTO, ReportingService::get_employee_weekly_statistics]
  affects: [service/reporting, service_impl/reporting, rest/report, rest-types]
tech_stack:
  added: []
  patterns: [pure-free-function-formula, HR-permission-gate-first, TOs-with-service-impl-feature-gate]
key_files:
  created:
    - service_impl/src/test/reporting_avg_weekly.rs
  modified:
    - service/src/reporting.rs
    - service_impl/src/reporting.rs
    - rest-types/src/lib.rs
    - rest/src/report.rs
    - service_impl/src/test/mod.rs
decisions:
  - "Pure free function `average_worked_hours_per_week` kept in service crate (not service_impl) for direct testability without mocks"
  - "EmployeeWeeklyStatistics exposes included_weeks + total_worked_hours alongside the average so the frontend can show context (D-22 discretion)"
  - "Route /{id}/weekly-statistics registered before /{id} to avoid Axum path conflict"
metrics:
  duration: "~15 minutes"
  completed: 2026-06-26
  tasks_completed: 3
  tasks_total: 3
  files_changed: 5
  files_created: 1
---

# Phase 22 Plan 01: Mitarbeiter-Statistik HR — Backend Summary

Average worked hours per week (A-22-1) computation + HR-gated REST endpoint with
OpenAPI documentation and full unit test coverage.

## What Was Built

### New Types (service/src/reporting.rs)

**`EmployeeWeeklyStatistics`** struct:
```rust
pub struct EmployeeWeeklyStatistics {
    pub average_worked_hours_per_week: f32,
    pub included_weeks: u32,
    pub total_worked_hours: f32,
}
```

**`average_worked_hours_per_week(weeks: &[GroupedReportHours]) -> EmployeeWeeklyStatistics`**
Pure free function implementing A-22-1:
- `worked = overall_hours + volunteer_hours` per week
- `absence = vacation_hours + sick_leave_hours + unpaid_leave_hours + holiday_hours`
- Fully-absent week (worked==0 && absence>0) excluded from denominator
- Empty included set → 0.0 (no division by zero)
- No reference to expected_hours/contract_weekly_hours (flexible-contract safe)

**New trait method on `ReportingService`:**
```rust
async fn get_employee_weekly_statistics(
    &self,
    sales_person_id: &Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<EmployeeWeeklyStatistics, ServiceError>;
```

### Implementation (service_impl/src/reporting.rs)

`get_employee_weekly_statistics` implementation:
1. **HR gate FIRST** (STAT-01/D-22-05): `self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;`
2. Current year/week via `self.clock_service.date_now().to_iso_week_date()` (D-22-01)
3. Delegates to existing `get_report_for_employee` (D-22-06 reuse)
4. Applies pure `average_worked_hours_per_week` formula over `report.by_week`

### DTO (rest-types/src/lib.rs)

`EmployeeWeeklyStatisticsTO` with `ToSchema`, `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`.
`#[cfg(feature = "service-impl")] impl From<&EmployeeWeeklyStatistics>` (field-by-field copy).

### REST Endpoint (rest/src/report.rs)

- `GET /report/{id}/weekly-statistics` handler `get_weekly_statistics`
- `#[utoipa::path]` with 200/403/500 responses, `EmployeeWeeklyStatisticsTO` body
- Registered in `generate_route` before `/{id}` to avoid Axum path conflict
- Added to `ReportApiDoc` paths and components(schemas)

### Unit Tests (service_impl/src/test/reporting_avg_weekly.rs)

9 tests, all passing:
- `fully_absent_week_excluded` — 3 weeks, 1 fully absent, correct average over 2
- `partial_absence_week_included_with_actual_worked` — worked=12, vacation=8 → counted as 12
- `zero_work_no_absence_counts_as_zero` — worked=0, absence=0 → included as 0
- `flexible_contract_no_expected_hours` — contract_weekly_hours=0, no panic/NaN
- `volunteer_counts_toward_worked` — overall=10, volunteer=5 → 15 worked
- `fully_absent_sick_leave_excluded` — sick-leave-only absence → excluded
- `empty_input_returns_zero` — empty slice → 0.0, included_weeks=0
- `all_absence_categories_cause_exclusion` — all 4 categories individually cause exclusion
- `volunteer_only_week_included` — volunteer-only week (no absence) → included

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new trust boundaries beyond what the plan's threat model covers (T-22-01, T-22-02, T-22-03).
The HR gate as first statement is verified above (line 970-972 of service_impl/src/reporting.rs).

## Self-Check: PASSED

- service/src/reporting.rs: EmployeeWeeklyStatistics struct + average_worked_hours_per_week fn + trait method — present
- service_impl/src/reporting.rs: get_employee_weekly_statistics with HR_PRIVILEGE as first check — present
- rest-types/src/lib.rs: EmployeeWeeklyStatisticsTO with ToSchema — present
- rest/src/report.rs: get_weekly_statistics handler + route + #[utoipa::path] + ReportApiDoc — present
- service_impl/src/test/reporting_avg_weekly.rs: 9 unit tests — created
- cargo build --workspace: green
- cargo test --workspace: green (470+ tests in service_impl, all passing)

## Note for Plan 22-02 (Frontend)

Frontend consumes:
- `GET /report/{id}/weekly-statistics`
- Response: `EmployeeWeeklyStatisticsTO { average_worked_hours_per_week: f32, included_weeks: u32, total_worked_hours: f32 }`
- 403 for non-HR callers (gate on HR role)
