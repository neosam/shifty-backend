---
phase: quick-260624-ujk
plan: 01
subsystem: reporting
tags: [reporting, volunteer-hours, no-contract, snapshot-versioning]
tech-stack:
  patterns: [no-contract-detection, three-path-consistency]
key-files:
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/booking_information.rs
    - service_impl/src/test/mod.rs
  created:
    - service_impl/src/test/reporting_no_contract_volunteer.rs
decisions:
  - "no-contract detection via find_working_hours_for_calendar_week(...).next().is_none() in all three report paths"
  - "Only shiftplan hours without a contract become volunteer; ExtraWork ExtraHours stay in overall_hours (paid axis)"
  - "get_week implicitly has no no-contract persons (all_for_week semantics); verified via Detail-vs-Summary consistency test instead"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bumped 8->9 because volunteer_hours computation changed"
metrics:
  duration: ~25 min
  completed: 2026-06-24
  tasks_completed: 3
  files_modified: 5
  files_created: 1
  tests_added: 4
  tests_total: 561
---

# Quick Task 260624-ujk: Mitarbeiter-Report geleistete Stunden ohne Vertrag → Ehrenamt

**One-liner:** Shiftplan-Stunden in Wochen ohne EmployeeWorkDetails-Zeile werden jetzt als `volunteer_hours` (Ehrenamt) klassifiziert statt Soll=Ist-neutralisiert — in allen drei Report-Pfaden konsistent, mit Snapshot-Bump 8→9.

## Was geaendert wurde

### Task 1: no-contract-Erkennung in `service_impl/src/reporting.rs`

**Drei betroffene Report-Pfade**, alle jetzt mit einheitlicher Erkennung:

```rust
let has_contract_row =
    find_working_hours_for_calendar_week(working_hours, year, week)
        .next()
        .is_some();
```

**Pfad 1: `hours_per_week`** (Detail-Report, Kern-Funktion)
- Vor `apply_weekly_cap` wird `has_contract_row` bestimmt.
- Neue Variablen: `shiftplan_paid = if has_contract_row { shiftplan_hours } else { 0.0 }` und `no_contract_volunteer = if has_contract_row { 0.0 } else { shiftplan_hours }`.
- `expected_hours`-Block: dritter Zweig `!has_contract_row` => `expected_hours = 0.0` (vor dem dynamischen `== 0.0`-Zweig).
- `GroupedReportHours.overall_hours` und `.balance` verwenden `shiftplan_paid` statt `shiftplan_hours`.
- `GroupedReportHours.volunteer_hours` erhaelt zusaetzlich `+ no_contract_volunteer`.

**Pfad 2: `get_reports_for_all_employees`** (Summary)
- `has_contract_row`-Erkennung vor dem `if expected_hours <= 0.0`-Block.
- Neuer erster Zweig `!has_contract_row`: `shiftplan_hours: 0.0`, `planned_hours: 0.0`, `volunteer_hours: auto_volunteer_hours + shiftplan_hours`; ExtraWork bleibt in `extra_working_hours`.
- Bisheriger `if expected_hours <= 0.0`-Zweig wird zu `else if expected_hours <= 0.0` (dynamischer Vertrag — unveraendert).

**Pfad 3: `get_week`** (Wochen-Report)
- `has_contract_row`-Erkennung hinzugefuegt mit dokumentierendem Kommentar.
- `abense_hours_for_balance` und `absence_derived_balance_total` Guard erweitert: `if !has_contract_row || planned_hours <= 0.0`.
- `shiftplan_paid`/`no_contract_volunteer`-Pattern analog den anderen Pfaden.
- Kommentar dokumentiert: `all_for_week`-Semantik bedeutet, dass Personen OHNE Vertragszeile die Schleife gar nicht erreichen — `has_contract_row` ist in `get_week` implizit immer `true`. Der Guard bleibt fuer Konsistenz und Dokuemtation.

**An allen drei Stellen Kommentare** mit:
- Zitat der User-Regel (KW ohne Zeile = Ehrenamt, Zeile mit expected=0 = dynamisch = Soll=Ist)
- Abgrenzung zur `booking_information`-Band-Logik (`is_paid=false`-Gate, disjunkt, keine Doppelzaehlung)

### Task 2: Tests in `service_impl/src/test/reporting_no_contract_volunteer.rs`

Vier Testfaelle, alle gruen:

| Fall | Szenario | Erwartung |
|------|----------|-----------|
| A | Keine Vertragszeile, 30h Shiftplan | volunteer=30, overall=0, balance=0, expected=0 |
| B | is_dynamic=true (Zeile vorhanden, expected weighted=0), 30h | overall=30, expected=30, balance=0, volunteer=0 (Soll=Ist unveraendert) |
| C | Zeile 40h, 30h Shiftplan | expected=40, overall=30, balance=-10, volunteer=0 |
| D | Konsistenz: Detail (hours_per_week) vs. Summary (get_reports_for_all_employees) fuer no-contract | beide: volunteer=30, overall=0, balance=0 |

**Randbemerkung zu `get_week` (Fall D):** `all_for_week` liefert nur Persons MIT Vertragszeile fuer die KW. Personen ohne Zeile werden in der `get_week`-Schleife nicht iteriert — der no-contract-Fall existiert dort strukturell nicht. Fall D verifiziert stattdessen die Detail-vs-Summary-Konsistenz (die beiden betroffenen Pfade stimmen ueberein).

Modul registriert in `service_impl/src/test/mod.rs`.

### Task 3: Snapshot-Schema-Bump in `service_impl/src/billing_period_report.rs`

- `CURRENT_SNAPSHOT_SCHEMA_VERSION`: `8` → `9`
- Begruendungs-Kommentar im Code:
  > Bump 8->9 (quick-260624-ujk): Die Berechnung des persistierten value_type `volunteer_hours` aendert sich — geleistete Shiftplan-Stunden in Wochen OHNE EmployeeWorkDetails-Vertragszeile zaehlen jetzt als Ehrenamt (volunteer) statt Soll=Ist-neutralisiert.

**Aktualisierte Pinning-Tests** (update auf 9):
- `service_impl/src/test/billing_period_snapshot_locking.rs`: `test_snapshot_schema_version_pinned` (Assertion + Kommentar aktualisiert)
- `service_impl/src/test/booking_information.rs`: `snapshot_schema_version_pinned_at_8` → `snapshot_schema_version_pinned_at_9` (Funktionsname + Assertion + Kommentar)

## Verifikation

- `cargo build --workspace`: gruen (Finished in ~20s)
- `cargo test --workspace`: 561 Tests, 0 failures (452 service_impl + 61 shifty_bin + weitere)
- 4 neue Tests (Fall A/B/C/D) alle gruen
- Bestehende Regressionstests (cap_overflow, billing_period, snapshot_locking, booking_information) alle gruen

## Deviations from Plan

None — plan executed exactly as written. The only design decision made was the pre-noted one: `get_week` structurally never sees no-contract persons (all_for_week semantics), so Fall D tests Detail-vs-Summary consistency instead of a get_week-specific test. This was documented in the plan as the expected fallback.

## Self-Check

Files created/modified:
- [x] `service_impl/src/reporting.rs` — modified (no-contract detection in 3 paths)
- [x] `service_impl/src/billing_period_report.rs` — modified (version 9)
- [x] `service_impl/src/test/reporting_no_contract_volunteer.rs` — created (4 tests)
- [x] `service_impl/src/test/mod.rs` — modified (module registered)
- [x] `service_impl/src/test/billing_period_snapshot_locking.rs` — modified (assert == 9)
- [x] `service_impl/src/test/booking_information.rs` — modified (assert == 9)

All files exist and tests pass: SELF-CHECK PASSED
