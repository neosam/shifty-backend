---
phase: 41-avg-anwesenheit-flexible
reviewed: 2026-07-02T00:00:00Z
depth: deep
files_reviewed: 14
files_reviewed_list:
  - service/src/reporting.rs
  - service_impl/src/reporting.rs
  - service_impl/src/test/reporting_avg_attendance.rs
  - service_impl/src/test/reporting_attendance_gate.rs
  - service_impl/src/test/mod.rs
  - rest-types/src/lib.rs
  - rest/src/report.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/service/employee.rs
  - shifty-dioxus/src/component/employee_view.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 41: Code Review — Ø-Anwesenheit bei flexiblen Stunden

**Reviewed:** 2026-07-02
**Depth:** deep (cross-file)
**Files Reviewed:** 14
**Status:** issues_found — kein BLOCKER/HIGH, 3 MEDIUM, 2 LOW

## Summary

Phase 41 implementiert die AVG-01-Metrik (durchschnittliche Arbeitsstunden pro
Anwesenheitstag für flexible Mitarbeiter). Die Kernlogik — pure Funktion
`average_hours_per_attendance_day`, HR-Gate zuerst, `is_dynamic`-Filter
server-seitig, `EmployeeAttendanceStatisticsTO`-DTO, FE-Rendering mit zwei
Options-Ebenen — ist korrekt umgesetzt. A-22-1 (`average_worked_hours_per_week`)
ist byte-unverändert (kein Remove-Diff). Snapshot-Version bleibt 12; kein neuer
`BillingPeriodValueType`. i18n vollständig in de/en/cs. Tests decken die
Filterkette (HR-Gate, static-Filter, Formel) ab.

Drei mittelschwere Befunde wurden gefunden, davon ist der signifikanteste das
falsche OpenAPI-Body-Schema (nullable fehlt).

---

## Warnings (MEDIUM)

### WR-01: OpenAPI-Body-Typ spiegelt nicht die nullable Antwort wider

**File:** `rest/src/report.rs:209`
**Issue:** Das utoipa-Attribut deklariert `body = EmployeeAttendanceStatisticsTO`
(nicht-nullable), obwohl der Endpunkt für nicht-flexible Mitarbeiter JSON `null`
zurückgibt. Jeder aus der OpenAPI-Spec generierte Client wird `null`-Antworten
nicht korrekt deserialisieren, da das Schema kein optionales/nullable Feld zeigt.

```rust
// aktuell (falsch):
(status = 200, ..., body = EmployeeAttendanceStatisticsTO, ...),

// korrekt:
(status = 200, ..., body = inline(Option<EmployeeAttendanceStatisticsTO>), ...),
// oder alternativ via nullable = true in utoipa 4.x:
(status = 200, ..., body = EmployeeAttendanceStatisticsTO, ..., nullable = true),
```

Der Endpunkt `GET /report/{id}/weekly-statistics` hat dieses Problem nicht, weil er
nie `null` zurückgibt. Hier ist es ein echter Vertragsbruch zwischen Schema und
Laufzeitverhalten.

---

### WR-02: Kein Happy-Path-Service-Test für flexiblen Mitarbeiter

**File:** `service_impl/src/test/reporting_attendance_gate.rs`
**Issue:** `reporting_attendance_gate.rs` enthält nur zwei Tests:
- `attendance_statistics_requires_hr` — HR-Gate (Forbidden, Frühabbruch)
- `attendance_statistics_returns_none_for_static` — statischer MA, Ok(None)

Der dritte Pfad — flexibler Mitarbeiter + HR-Berechtigung → `Ok(Some(stats))` — ist
nicht als Service-Level-Test abgedeckt. Die Integrationskette
`is_dynamic==true → get_report_for_employee → average_hours_per_attendance_day`
wird nicht end-to-end getestet. Die Pure-Function-Tests in
`reporting_avg_attendance.rs` testen nur die reine Formel, nicht den Service-Aufruf.

**Fix:** Einen dritten Test in `reporting_attendance_gate.rs` ergänzen, der:
- `permission_service.expect_check_permission()` → `Ok(())`
- `employee_work_details_service.expect_find_by_sales_person_id()` → dynamisches
  Fixture (is_dynamic=true)
- `shiftplan_report_service` etc. → minimale Stubs für `get_report_for_employee`
- Prüft, dass das Ergebnis `Ok(Some(stats))` ist und `attendance_days >= 0` hat

---

### WR-03: `is_dynamic`-Filter nicht auf den Report-Zeitraum eingeschränkt

**File:** `service_impl/src/reporting.rs:1207–1215`
**Issue:** Der Check `work_details.iter().any(|w| w.is_dynamic)` prüft, ob
der Mitarbeiter *irgendeinen* dynamischen Arbeitsvertrag hat — unabhängig vom
angefragten Zeitraum (`year`, `until_week`). Ein Mitarbeiter, der früher flexibel
war (2023) und jetzt einen Festvertrag hat (2026), bekommt die Metrik auch für
2026 berechnet. Die Metrik ist dann inhaltlich irreführend.

```rust
// aktuell: alle work_details, kein Datumsbezug
if !work_details.iter().any(|w| w.is_dynamic) {
    return Ok(None);
}

// konsistenter Ansatz (sofern work_details from/until_week Felder hat):
let report_from = ShiftyDate::first_day_in_year(year);
let report_until = ShiftyDate::new(year, until_week, DayOfWeek::Sunday)
    .unwrap_or_else(|_| ShiftyDate::last_day_in_year(year));
if !work_details.iter().any(|w| {
    w.is_dynamic
        && w.from.map_or(true, |f| f <= report_until)
        && w.until.map_or(true, |u| u >= report_from)
}) {
    return Ok(None);
}
```

**Hinweis:** Dieses Muster ist identisch mit dem in `billing_period_report.rs:475–477`
verwendeten. Die Inkonsistenz ist somit kein neues Problem von Phase 41, sondern
ein bestehendes Design-Muster. Dennoch ist es im Kontext der neuen Metrik
dokumentationswürdig, da die Metrik bei geändertem Vertragsstatus falsche Signale
senden kann.

---

## Info (LOW)

### IN-01: Redundante `until_week`-Klammerung

**File:** `service_impl/src/reporting.rs:1218`
**Issue:** `get_employee_attendance_statistics` klemmt `until_week` auf
`weeks_in_year` (Zeile 1218), bevor es `get_report_for_employee` aufruft.
`get_report_for_employee` führt die identische Klammerung selbst durch (Zeile 664).
Die Vorklammerung ist redundant.

**Fix:** Entweder die Vorklammerung in `get_employee_attendance_statistics` entfernen
oder einen Kommentar ergänzen, der erklärt, warum sie trotzdem hilfreich ist
(z.B. zur Dokumentation der Invariante).

---

### IN-02: `assert_eq!` auf `f32`-Struct in `user_example`-Test

**File:** `service_impl/src/test/reporting_avg_attendance.rs:53–59`
**Issue:** Der Test `user_example` verwendet `assert_eq!` um eine
`EmployeeAttendanceStatistics`-Instanz mit `f32`-Feldern zu vergleichen.
`PartialEq` für `f32` folgt IEEE 754: für exakt darstellbare Werte wie `4.5` und
`54.0` ist das sicher. Aber die Mischung mit Epsilon-Vergleichen
(`assert!((stats.total_worked_hours - 54.0).abs() < 0.001)` in Zeile 52)
direkt vor dem `assert_eq!` ist inkonsistent und das Muster ist für andere
Testwerte, die nicht exakt darstellbar sind, riskant.

**Fix:** Einheitlich Epsilon-Vergleiche verwenden oder dokumentieren, warum
Exakt-Vergleich hier sicher ist:
```rust
// Statt assert_eq! auf die gesamte Struktur:
let avg = stats.average_hours_per_attendance_day.expect("should be Some");
assert!((avg - 4.5).abs() < 0.001, "avg={avg}");
```

---

## Nicht beanstandet

- **A-22-1 unverändert**: `average_worked_hours_per_week` wurde nicht angefasst
  (Diff zeigt nur Additions, keine Deletions in diesem Bereich).
- **HR-Gate-Reihenfolge**: `check_permission` ist das erste `await`, kein Datenabruf
  davor — korrekt (D-AVG-05). Bewiesen durch `.times(0)` in
  `attendance_statistics_requires_hr`.
- **Snapshot-Version bleibt 12**: kein neuer `BillingPeriodValueType`,
  `billing_period_report.rs` unverändert.
- **Kein Scope-Creep**: Kein Datepicker, keine Multi-Mitarbeiter-Übersicht,
  kein A-22-1-Change.
- **Formel korrekt**: Σ Arbeitsstunden / Anzahl distinkte Anwesenheitstage.
  Abwesenheitskategorien (Vacation, SickLeave, Holiday, UnpaidLeave, Unavailable,
  Custom) aus Zähler und Nenner korrekt ausgeschlossen.
- **Zwei Options-Ebenen im FE korrekt**: Outer-None = nicht-flexibel/nicht-HR;
  Inner-None = flexibel aber <2 Anwesenheitstage → "–"-Anzeige.
- **i18n vollständig**: de/en/cs je 3 Schlüssel, Completeness-Test vorhanden.
- **IDOR akzeptiert**: HR sieht alle Mitarbeiter — by design.

---

_Reviewed: 2026-07-02_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
