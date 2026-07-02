---
phase: 41-avg-anwesenheit-flexible
fixed_at: 2026-07-02T00:00:00Z
review_path: .planning/phases/41-avg-anwesenheit-flexible/41-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 41: Code Review Fix Report

**Fixed at:** 2026-07-02
**Source review:** .planning/phases/41-avg-anwesenheit-flexible/41-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, IN-01, IN-02; WR-03 explicitly excluded per task brief)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: OpenAPI-Body-Typ nullable gemacht

**Files modified:** `rest/src/report.rs`
**Commit:** c98f9d7
**Applied fix:** `body = EmployeeAttendanceStatisticsTO` → `body = inline(Option<EmployeeAttendanceStatisticsTO>)` in der `#[utoipa::path]`-Annotation von `get_attendance_statistics`. Der Handler-Rückgabetyp bleibt unverändert; nur das OpenAPI-Schema modelliert jetzt korrekt die nullable Response.

---

### WR-02: Happy-Path-Service-Test für flexiblen Mitarbeiter

**Files modified:** `service_impl/src/test/reporting_attendance_gate.rs`
**Commit:** d31e241
**Applied fix:** Neuer Test `attendance_statistics_returns_some_for_flexible` (T-41-05) in `reporting_attendance_gate.rs`. Fixture: 2 Shiftplan-Tage in KW23/2024 (je 4h) für einen dynamischen Mitarbeiter (`is_dynamic=true`). Prüft die vollständige Kette: HR-Gate → is_dynamic-Filter → `get_report_for_employee` → `average_hours_per_attendance_day` → `Ok(Some(stats))` mit `attendance_days >= 2` und `average ≈ 4.0`. Importe für `BTreeMap`, `ExtraHours`, `ShiftplanReportDay`, `DayOfWeek`, `fixture_sales_person`, `fixture_work_details_dynamic_mon_fri` ergänzt.

---

### IN-01: Redundante `until_week`-Klammerung entfernt

**Files modified:** `service_impl/src/reporting.rs`
**Commit:** 91d4b11
**Applied fix:** Das `let until_week = until_week.min(time::util::weeks_in_year(year as i32));` in `get_employee_attendance_statistics` (Zeile 1218) wurde durch einen erklärenden Kommentar ersetzt. Die Klammerung existiert identisch in `get_report_for_employee` (Zeile 664) und ist daher hier redundant. Verhalten unverändert.

---

### IN-02: `assert_eq!` auf f32-Struct durch Epsilon-Vergleiche ersetzt

**Files modified:** `service_impl/src/test/reporting_avg_attendance.rs`
**Commit:** e32fe4d
**Applied fix:** Im Test `user_example` wurde `assert_eq!(stats, EmployeeAttendanceStatistics { ... })` (Struct-Vergleich mit f32-Feldern) durch einzelne Epsilon-Assertions ersetzt: `(avg - 4.5).abs() < 0.001` für den Durchschnitt. `total_worked_hours` war bereits epsilon-geprüft (bleibt so). Nun einheitliches Muster für alle f32-Felder. Nicht mehr benötigter Import `EmployeeAttendanceStatistics` entfernt.

---

## Nicht behandelt

### WR-03: `is_dynamic`-Filter nicht zeitraum-eingeschränkt (bewusst ausgelassen)

**Reason:** Laut Task-Brief explizit ausgeschlossen. WR-03 beschreibt ein bestehendes Design-Muster, das identisch in `billing_period_report.rs:475–477` verwendet wird. Der "current is_dynamic"-Ansatz entspricht D-AVG-05 und ist für diesen Scope in-scope. Eine zeitraum-bewusste Vertragshistorie würde eine eigenständige Feature-Änderung mit Datenmodell-Erweiterungen erfordern und liegt außerhalb von Phase 41.

---

## Gate-Ergebnisse

| Gate | Ergebnis |
|---|---|
| `cargo test -p service_impl reporting_attendance_gate` | 3/3 grün |
| `cargo test -p service_impl reporting_avg_attendance` | 7/7 grün |
| `cargo test --workspace` | alle grün (569 unit + 64 integration) |
| `cargo clippy --workspace -- -D warnings` | sauber |
| `cargo build --workspace` | Finished dev |
| `grep CURRENT_SNAPSHOT_SCHEMA_VERSION.*12` | 12 (unverändert) |

---

_Fixed: 2026-07-02_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
