# Phase 02 — Deferred Items

Ausserhalb des Wave-0-Scopes von Plan 02-01 entdeckt, aber nicht behandelt
(per `<deviation_rules>` Scope-Boundary: nur Fixes fuer Issues die DIREKT
vom aktuellen Task verursacht werden).

## Pre-existing Phase-1-Luecke: fehlende `absence_period`-Migration

**Entdeckt waehrend:** Phase-Gate-Verification (`cargo test --workspace`) am Ende
von Plan 02-01.

**Symptom:**
- `shifty_bin/src/integration_test/absence_period.rs` (8 Tests) scheitern alle mit
  `SqliteError { code: 1, message: "no such table: absence_period" }`.
- Kein `migrations/sqlite/<TS>_create-absence-period.sql` existiert im Workspace.

**Was fehlt:**
- Migration `<TS>_create-absence-period.sql` mit:
  - `CREATE TABLE absence_period (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version)`
  - 3 partial unique indexes (vermutlich nach Phase-1-Plan-01-00-Spec)
  - CHECK constraint `to_date >= from_date`
  - SQLx-Offline-Files (`.sqlx/`-Verzeichnis)

**Warum nicht hier gefixt:**
- Liegt **ausserhalb des Wave-0-Test-Scaffolding-Scopes** von Plan 02-01.
- Plan 02-01 fokussiert sich auf Test-Fixtures + Locking-Tests + Stubs in
  `service_impl/src/test/`. Diese Tests benoetigen die `absence_period`-Tabelle
  NICHT (alle Stubs sind `#[ignore]`d; Locking-Test ist DB-frei).
- Der DateRange-Fix war ein direkter Build-Blocker fuer Wave-0-Tests
  (`cargo build -p service_impl --tests` war ROT). Die fehlende Migration ist
  das nicht — `cargo build`/`cargo test -p service_impl` sind gruen.
- Migration-Schreiben + sqlx prepare braucht `nix-shell -p sqlx-cli`-Setup
  und ist eine eigene Wave (die Phase-1 haette vollziehen muessen).

**Phase-1-Hygiene-Empfehlung:**
- Einen Phase-1-Cleanup-Plan auflegen, der die fehlende Migration nachreicht
  und SQLx-Offline-Files regeneriert. Idealerweise vor Wave-1 von Phase 2
  (Plan 02-02 / 02-03), damit die Phase-2-Wave-1-Tests die DB tatsaechlich
  benutzen koennen.

**Status fuer Phase 2:**
- Plan 02-01 (Wave 0) ist auf service_impl-Lib-Test-Ebene komplett gruen modulo
  intentional-rotem Pin-Test. Phase-Plan-1-Wave-1 (Plan 02-02 oder 02-03) wird
  voraussichtlich keine DB-Tests einfuehren — beide Wave-1-Plans sind
  Mock-basiert.
- Wave-2-Plan (02-04) braucht die Locking-Pin-Map mit Mock-Setup, ebenfalls
  DB-frei.
- **Falls Wave 1 oder Wave 2 in einer Test-Session DB-Tests braucht**: zuerst
  Phase-1-Migration nachreichen.

## Zusatz-Anmerkung: Carryover-Phase-1-Fix

Der DateRange-Fix in `shifty-utils` und die `OverlappingPeriod`-Variante in
`service::ValidationFailureItem` wurden in Plan 02-01 als Rule-3-Auto-Fix
nachgereicht (commit `d8dad0aa`). Phase-1-Hygiene-Plan sollte diesen Fix in
seine Doku/Decisions aufnehmen, damit klar ist welcher Phase die Symbole gehoeren.
