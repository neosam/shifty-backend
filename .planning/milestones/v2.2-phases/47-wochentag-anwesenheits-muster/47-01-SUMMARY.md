---
phase: 47-wochentag-anwesenheits-muster
plan: 1
subsystem: backend/reporting
tags: [reporting, attendance, hr, weekday, dto, breaking-change]
status: complete
requirements: [RPT-01, RPT-02, RPT-03]
requires: []
provides:
  - service::reporting::weekday_attendance_distribution
  - service::reporting::EmployeeAttendanceStatistics (v2.2 shape)
  - service::reporting::WeekdayAttendanceStat
  - rest_types::WeekdayAttendanceTO
  - rest_types::EmployeeAttendanceStatisticsTO (v2.2 shape)
affects:
  - rest/src/report.rs (utoipa response desc + schemas registration)
  - service_impl/src/reporting.rs::get_employee_attendance_statistics (adapter swap)
tech-stack:
  added: []
  patterns:
    - pure-fn + separate TDD test module (mirrors v2.1 AVG-01 pattern)
    - fixed-length weekday array Mon..Sun with Vec<T> transport (utoipa fixed-array workaround)
key-files:
  created:
    - service_impl/src/test/reporting_weekday_attendance.rs
  modified:
    - service/src/reporting.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/mod.rs
    - service_impl/src/test/reporting_attendance_gate.rs
    - rest-types/src/lib.rs
    - rest/src/report.rs
  deleted:
    - service_impl/src/test/reporting_avg_attendance.rs
decisions:
  - D-47-BE — Endpoint URL /report/{id}/attendance-statistics reused, only response shape swapped (breaking, released together with FE in v2.2)
  - D-AVG-02/03 — Attendance-day definition unchanged from v2.1 (Shiftplan|ExtraWork|VolunteerWork with hours > 0)
  - D-AVG-05 — HR gate stays FIRST await; is_dynamic filter stays before report fetch
  - RPT-03 — Read-aggregate only; snapshot version 12 untouched; no new BillingPeriodValueType
metrics:
  duration: 15m2s
  completed: 2026-07-02T21:58Z
---

# Phase 47 Plan 1: Backend Weekday-Anwesenheits-Muster Summary

Replace the v2.1 scalar `average_hours_per_attendance_day` metric with a per-weekday attendance-day distribution (`count` + `share`) at the same HR-gated endpoint `/report/{id}/attendance-statistics`. Response shape changes; endpoint URL, HR gate, and `is_dynamic` filter are unchanged. Snapshot version 12 is grep-verified untouched (read aggregate only).

## New Pure Function

**Signature** (in `service/src/reporting.rs`):

```rust
pub fn weekday_attendance_distribution(
    days: &[WorkingHoursDay],
    counted_calendar_weeks: u32,
) -> EmployeeAttendanceStatistics
```

**Semantics** (locked in 47-CONTEXT):

- Attendance-day filter (byte-identical to v2.1 D-AVG-02/03): `d.hours > 0.0 && matches!(d.category, Shiftplan | ExtraWork | VolunteerWork)`. Absence categories and `Custom(_)` are excluded.
- Distinct-date dedupe per weekday: each calendar date counts ONCE per weekday, even with multiple work entries (`BTreeSet<time::Date>` per weekday bucket).
- `share = min(count / counted_calendar_weeks, 1.0)`, rounded to two decimals via `((x * 100.0).round() / 100.0)`.
- `counted_calendar_weeks == 0` → all shares 0.0 (finite, no NaN, no +Inf).
- Result array is ALWAYS length 7, ordered `Monday..Sunday`; every weekday present even when count=0.

**Weekday derivation:** `d.date.weekday()` (from `time` crate) → local 7-arm helper `weekday_index_mon0`.

**`counted_calendar_weeks`** in the adapter (`service_impl/src/reporting.rs::get_employee_attendance_statistics`): `report.by_week.len() as u32`. This mirrors the CONTEXT decision "gezählte Kalenderwochen identisch v2.1" — one row per counted week from `get_report_for_employee`, which already clamps `until_week` to `weeks_in_year` server-side.

## Interface Contract for Plan 47-02 (Frontend)

**Endpoint URL (unchanged):** `GET /report/{id}/attendance-statistics?year={year}&until_week={until_week}`
**Auth:** HR privilege required (403 otherwise). Non-flexible employees → JSON `null`.

**Response body (flexible + HR):**

```json
{
  "attendance_by_weekday": [
    { "weekday": "Monday",    "count": 8, "share": 0.80 },
    { "weekday": "Tuesday",   "count": 3, "share": 0.30 },
    { "weekday": "Wednesday", "count": 7, "share": 0.70 },
    { "weekday": "Thursday",  "count": 5, "share": 0.50 },
    { "weekday": "Friday",    "count": 2, "share": 0.20 },
    { "weekday": "Saturday",  "count": 0, "share": 0.00 },
    { "weekday": "Sunday",    "count": 0, "share": 0.00 }
  ],
  "counted_calendar_weeks": 10
}
```

- `attendance_by_weekday` is always length 7, ordered Monday..Sunday (server invariant).
- `weekday` values use the existing `DayOfWeekTO` enum (PascalCase serialization: `"Monday"..."Sunday"`).
- `count: u32` = distinct attendance dates on that weekday.
- `share: f32` in `0.0..=1.0`, two decimals, `0.0` when `counted_calendar_weeks == 0`.
- `counted_calendar_weeks: u32` = number of report weeks (denominator).

**Response body (non-flexible OR non-HR):** JSON `null`.

## RPT-03 Grep-Gate Evidence

```text
$ grep -n 'CURRENT_SNAPSHOT_SCHEMA_VERSION' service_impl/src/billing_period_report.rs
117:pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
369:            // KEIN value_type-Change -> KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump
390:            snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,

$ grep -v '^#' service/src/reporting.rs service_impl/src/reporting.rs rest-types/src/lib.rs rest/src/report.rs | grep -c 'average_hours_per_attendance_day'
0

$ grep -rn 'BillingPeriodValueType::' service/src service_impl/src --include='*.rs' | grep -v test | wc -l
45   # baseline was 45 before the phase, unchanged
```

Snapshot version stays 12. Zero `average_hours_per_attendance_day` occurrences on the backend surface. `BillingPeriodValueType::` reference count in non-test code is unchanged.

## Gates

- `cargo test --workspace`: green (571 tests in service_impl, including 8 new weekday-distribution tests; 7 v2.1 AVG unit tests removed with the deleted `reporting_avg_attendance.rs`; existing HR-gate test updated to the new shape).
- `cargo clippy --workspace -- -D warnings`: clean.
- `cargo build --workspace`: clean.
- RPT-03 grep gates: all match expected values (evidence above).
- No new `Cargo.toml` deps introduced.
- No `.sqlx` regeneration needed (no `query!` / `query_as!` added).

## What Was Deleted (For the Record)

- Pure fn `service::reporting::average_hours_per_attendance_day` (v2.1 AVG-01 scalar).
- Struct `service::reporting::EmployeeAttendanceStatistics` v2.1 shape (`average_hours_per_attendance_day`, `attendance_days`, `total_worked_hours` fields) — replaced in-place by the v2.2 weekday-distribution shape.
- 7 unit tests in `service_impl/src/test/reporting_avg_attendance.rs`; the file itself is removed, and `mod reporting_avg_attendance;` is deleted from `test/mod.rs`.

The DTO struct name `EmployeeAttendanceStatisticsTO` and the service struct name `EmployeeAttendanceStatistics` are reused; only the fields inside change (breaking wire format, coordinated with FE in Plan 47-02).

## Frontend Impact

The frontend WASM build is expected to break on this phase boundary — `shifty-dioxus/src/component/employee_view.rs`, `service/employee.rs`, and `api.rs` still reference the removed v2.1 fields (`average_hours_per_attendance_day`, `attendance_days`, `total_worked_hours`). This is intentional and documented in the plan:

- WASM build failure surfaces: `error[E0609]: no field 'average_hours_per_attendance_day' on type '&Rc<EmployeeAttendanceStatisticsTO>'` — available fields are `attendance_by_weekday`, `counted_calendar_weeks`, confirming the DTO renamed cleanly.
- **Plan 47-02** wires the FE HR-stats block onto the new payload (per-weekday line with `Mo: 8 (80%) · Di: 3 (30%) · …`), adds i18n keys for weekday labels and tooltip, and restores the WASM build.

## Self-Check: PASSED

Files created:

- FOUND: `service_impl/src/test/reporting_weekday_attendance.rs`
- FOUND: `.planning/phases/47-wochentag-anwesenheits-muster/47-01-SUMMARY.md`

Files deleted:

- CONFIRMED: `service_impl/src/test/reporting_avg_attendance.rs` removed.

Commits will be recorded by GSD auto-commit (co-located jj/git).

---

## Post-Ship-Nachtrag (2026-07-03, vor Release-Tag)

RPT-02-Fix in Phase 47-02 hat die Wochentag-Zeile nur für flexible Employees
gerendert (Backend `is_dynamic`-Filter). User wollte sie für ALLE Employees
sehen.

**Nachträglicher Fix (v2.2, post-ship):**
- Backend: `ReportingServiceImpl::get_employee_attendance_statistics`
  `is_dynamic`-Gate entfernt (HR-Gate bleibt). Test
  `attendance_statistics_returns_none_for_static` durch
  `attendance_statistics_returns_some_for_static_after_rpt02` ersetzt.
- Frontend: neuer i18n-Key `WeekdayAttendanceLabel` (de „Anwesenheit / Tag",
  en „Attendance / day", cs „Docházka / den") als kurzes Label; der lange
  Text bleibt als `title`-Tooltip.

Files: `service_impl/src/reporting.rs`,
`service_impl/src/test/reporting_attendance_gate.rs`,
`shifty-dioxus/src/component/employee_view.rs`,
`shifty-dioxus/src/i18n/{mod,de,en,cs}.rs`. Siehe auch `MILESTONES.md`
v2.2-Post-Ship-Sektion.
