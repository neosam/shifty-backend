---
phase: 17-contract-editor-unpaid-volunteer-path
plan: "01"
subsystem: backend/reporting
tags: [D-06, D-GATING-STYLE, CVC-10, is_paid-gate, reporting, billing-period]
dependency_graph:
  requires: []
  provides:
    - is_paid-Gate in reporting.rs::get_week (D-06/CVC-10)
    - is_paid-Gate in billing_period_report.rs::build_new_billing_period (D-GATING-STYLE)
    - CVC-10 Integrationstests (get_week_skips_unpaid_person, get_week_unpaid_no_paid_hours_leak)
  affects:
    - service_impl/src/reporting.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/reporting_additive_merge.rs
tech_stack:
  added: []
  patterns:
    - is_paid.unwrap_or(false) + continue-Gate vor result.push (reporting.rs Z.881-898)
    - is_paid.unwrap_or(false) + continue-Gate im billing Loop (billing_period_report.rs Z.322)
key_files:
  created: []
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/reporting_additive_merge.rs
decisions:
  - "D-06: is_paid-Gate in get_week vor result.push; SalesPerson vorab gefetcht statt inline"
  - "D-GATING-STYLE: is_paid-Gate als erste Zeile im build_new_billing_period sales_persons-Loop"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 7 (kein value_type-Change, nur Personen-Set-Filter)"
metrics:
  duration: ~15min
  completed: "2026-06-24"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 17 Plan 01: is_paid-Gates (reporting.rs + billing_period_report.rs) Summary

**One-liner:** Explizite is_paid-Gates in get_week (reporting.rs) + build_new_billing_period (billing_period_report.rs) mit zwei CVC-10-Integrationstests, die No-Leak-Verhalten + Personen-Set-Konsistenz pinnen.

## What Changed

### Task 1: is_paid-Gate in reporting.rs::get_week

**Datei:** `service_impl/src/reporting.rs` (ca. Z. 881-898)

Die historische `result.push(ShortEmployeeReport { sales_person: Arc::new(self.sales_person_service.get(...).await?), ... })`-Struktur wurde restrukturiert:

1. SalesPerson wird **vor** dem `result.push` einmalig gefetcht (war: inline im push).
2. Direkt nach dem Fetch: `if !sales_person.is_paid.unwrap_or(false) { continue; }` springt zur naechsten Iteration.
3. Im `result.push` wird das vorgefetchte `sales_person` via `Arc::new(sales_person)` verwendet.

**Gate-Kommentar:** D-06 / CVC-10 — Unbezahlte Freiwillige (is_paid=false, expected_hours=0) halten ab Phase 17 einen EmployeeWorkDetails-Record, duerfen aber NICHT in paid_hours / WorkingHoursPerSalesPerson / Year-Summary lecken.

**Referenz-Pattern genutzt:** `get_reports_for_all_employees` Z.164 (`.filter(|employee| employee.is_paid.unwrap_or(false))`).

### Task 1 (Tests): Zwei CVC-10-Integrationstests

**Datei:** `service_impl/src/test/reporting_additive_merge.rs`

Neue Hilfsfixtures:
- `unpaid_person_id()` — deterministische Uuid fuer unbezahlte Person
- `unpaid_volunteer_sales_person()` — `is_paid=Some(false)`, `inactive=false`
- `unpaid_volunteer_work_details()` — `expected_hours=0.0`, `committed_voluntary=5.0`, gueltig KW22-25/2024

Neue Tests:
- `get_week_skips_unpaid_person` — zwei Work-Details (paid + unpaid) im `all_for_week`-Mock; `get()` liefert typgerechten SalesPerson. Assert: Ergebnis hat len=1, nur paid-Person-Id, keine unpaid-Person-Id.
- `get_week_unpaid_no_paid_hours_leak` — dasselbe Setup; Assert: `paid_hours = Σ dynamic_hours` > 0 (paid-Person traegt bei), unpaid-Person-Id nicht im Ergebnis-Set, len=1.

### Task 2: is_paid-Gate in billing_period_report.rs::build_new_billing_period

**Datei:** `service_impl/src/billing_period_report.rs` (ca. Z. 322-332)

Erste Zeile im `for sales_person in sales_persons.iter()`-Body:
```rust
// D-GATING-STYLE / CVC-10: unbezahlte Personen (is_paid=false) werden ab Phase 17
// EmployeeWorkDetails-Records halten (rein freiwillige Helfer). Sie duerfen NICHT als
// BillingPeriodSalesPerson-Eintraege im Snapshot erscheinen — Personen-Set-Konsistenz
// mit get_week (year-summary) + get_reports_for_all_employees (all-employees-report).
// KEIN value_type-Change -> KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump (bleibt 7).
if !sales_person.is_paid.unwrap_or(false) {
    continue;
}
```

**Snapshot-Version:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt unveraendert `7`. Das Gate aendert keinen persistierten `BillingPeriodValueType` und keine value_type-Berechnung — es filtert nur das Personen-Set vor. Regressionstest `test_snapshot_schema_version_pinned` bestaetigt dies (weiterhin gruen).

## Deviations from Plan

None — Plan wurde exakt wie beschrieben ausgefuehrt. Die einzige Korrektur war die Verwendung des korrekten `ServiceError::EntityNotFound(id)`-Variants statt `ServiceError::NotFound` (letzteres existiert nicht) in den neuen Tests — eine triviale Rule-1-Anpassung waehrend des Compile-Schritts.

## Test Results

- `cargo test -p service_impl get_week`: 4 passed (beide neuen Tests + bestehende get_week-Tests)
- `cargo test -p service_impl billing_period`: 32 passed (inkl. `test_snapshot_schema_version_pinned`)
- `cargo test -p service_impl`: 442 passed, 0 failed
- `cargo check --workspace`: clean (0 errors)
- `cargo test` (workspace-weit): alle Suites gruen

## Acceptance Criteria Verification

- `grep -n "is_paid" service_impl/src/reporting.rs`: 3 Treffer (Z.164 bestehend bei get_reports_for_all_employees, Z.881/889 NEU im get_week-Bereich) — ERFUELLT
- `grep -c "continue" service_impl/src/reporting.rs`: 1 Treffer im get_week-Loop (Z.890) — ERFUELLT
- `grep -n "get_week_skips_unpaid_person\|get_week_unpaid_no_paid_hours_leak" service_impl/src/test/reporting_additive_merge.rs`: beide Tests vorhanden (Z.1147, Z.1247) — ERFUELLT
- `grep -n "CURRENT_SNAPSHOT_SCHEMA_VERSION" service_impl/src/billing_period_report.rs`: zeigt `= 7` (unveraendert) — ERFUELLT
- `grep -n "is_paid" service_impl/src/billing_period_report.rs`: Treffer bei Z.328 im build_new_billing_period-Bereich — ERFUELLT

## Known Stubs

None.

## Threat Flags

None — die Aenderungen reduzieren die Angriffsoberflaeche (Datenleck-Praevention), fuegen keine neue hinzu.

## jj-Commit-Hinweis

Alle Aenderungen sind **uncommitted** im Working-Tree. Der User committet manuell via `jj`. Folgende Dateien wurden veraendert:

1. `service_impl/src/reporting.rs` — is_paid-Gate in get_week (D-06/CVC-10)
2. `service_impl/src/billing_period_report.rs` — is_paid-Gate in build_new_billing_period (D-GATING-STYLE)
3. `service_impl/src/test/reporting_additive_merge.rs` — zwei CVC-10-Integrationstests + Hilfsfixtures

## Self-Check: PASSED

- `service_impl/src/reporting.rs` existiert und enthaelt is_paid-Gate: FOUND
- `service_impl/src/billing_period_report.rs` existiert und enthaelt is_paid-Gate: FOUND
- `service_impl/src/test/reporting_additive_merge.rs` enthaelt beide neuen Tests: FOUND
- `cargo test -p service_impl`: 442 passed, 0 failed — PASSED
- `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` unveraendert — CONFIRMED
