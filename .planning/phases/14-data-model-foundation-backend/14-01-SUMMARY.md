---
phase: 14-data-model-foundation-backend
plan: 01
subsystem: database
tags: [committed_voluntary, employee_work_details, sqlx, migration, dao, service, rest-types, CVC-01, CVC-02]

# Dependency graph
requires:
  - phase: 08-absence-crud-page-foundation
    provides: employee_work_details entity + cap_planned_hours_to_expected threading pattern
provides:
  - "committed_voluntary: f32 field end-to-end on EmployeeWorkDetails (SQLite -> DAO -> Service -> rest-types)"
  - "CVC-02 carry-forward in service_impl update() spread"
  - "additive migration 20260623120000 + regenerated .sqlx cache"
  - "CVC-02 unit test: update_propagates_committed_voluntary_to_dao"
affects: [14-02, phase-15-reporting, phase-16-frontend-display, phase-17-frontend-editor]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Numeric-Field-Threading (f64 in DAO-Row, as f32 cast in TryFrom, f32 everywhere else) — cap_planned_hours_to_expected Position als Anker, expected_hours als Typ-Vorbild"
    - "Additive SQLite Migration mit NOT NULL DEFAULT 0 fuer bestandsdatensichere Column-Erweiterung"
    - "#[serde(default)] auf neuen EmployeeWorkDetailsTO-Feldern fuer Wire-Backward-Compat"

key-files:
  created:
    - migrations/sqlite/20260623120000_add-committed-voluntary-to-employee-work-details.sql
    - (CVC-02 test in service_impl/src/test/employee_work_details.rs erweitert)
  modified:
    - dao/src/employee_work_details.rs
    - dao_impl_sqlite/src/employee_work_details.rs
    - service/src/employee_work_details.rs
    - service_impl/src/employee_work_details.rs
    - service_impl/src/test/employee_work_details.rs
    - rest-types/src/lib.rs
    - rest/src/dev.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_phase2_fixtures.rs
    - service_impl/src/test/billing_period_report.rs
    - service_impl/src/test/vacation_balance.rs
    - shifty_bin/src/integration_test.rs
    - shifty_bin/src/integration_test/employee_work_details_update.rs

key-decisions:
  - "CVC-01: committed_voluntary: f32 durch alle 5 Layer (Migration, DAO, Service, rest-types, .sqlx) — Typ-Vorbild expected_hours, Positions-Vorbild cap_planned_hours_to_expected"
  - "CVC-02: Carry-Forward-Spread-Zeile (entity.committed_voluntary = employee_work_details.committed_voluntary) in service_impl::update() — einzige Stelle ohne Compiler-Schutz, per Test gepinnt"
  - "Kein Snapshot-Bump (CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 7) — Feld ist inert in Phase 14, kein value_type aendert sich"
  - "Kein ToSchema/utoipa an EmployeeWorkDetailsTO — Struct ist serde-transparent, OpenAPI unveraendert"

patterns-established:
  - "Numeric-Field-Threading: DAO-Row f64 + as f32 Cast, alle anderen Layer f32 (keine Bool-Coercion)"
  - "Additive Migration NOT NULL DEFAULT 0 — Bestandsdaten driftfrei, kein reset noetig"

requirements-completed: [CVC-01, CVC-02]

# Metrics
duration: 45min
completed: 2026-06-23
---

# Phase 14 Plan 01: committed_voluntary Field-Threading Backend Summary

**Additive SQLite-Spalte `committed_voluntary REAL NOT NULL DEFAULT 0` end-to-end durch DAO (f64-Row + as f32 TryFrom + 4 SELECT + INSERT + UPDATE), Service-Struct + beide Konversionen, EmployeeWorkDetailsTO (#[serde(default)]) + beide From-Impls, und CVC-02 Carry-Forward-Spread in service_impl::update() — Workspace baut, 561+ Tests gruen, kein Snapshot-Bump**

## Performance

- **Duration:** ca. 45 min
- **Started:** 2026-06-23
- **Completed:** 2026-06-23
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Additive Migration `20260623120000_add-committed-voluntary-to-employee-work-details.sql` angelegt und via `sqlx migrate run` applied; `.sqlx`-Offline-Cache via `cargo-sqlx prepare --workspace` regeneriert
- `committed_voluntary: f32` durch alle Backend-Layer: DAO-Trait (Entity), DAO-Impl (Row f64, TryFrom as f32, 4 SELECT, INSERT, UPDATE), Service (Struct + From<&Entity> + TryFrom<&EmployeeWorkDetails>), rest-types (TO + beide From-Impls mit #[serde(default)])
- CVC-02 Carry-Forward-Spread in `service_impl::employee_work_details.rs::update()` eingetragen; neuer Unit-Test `update_propagates_committed_voluntary_to_dao` pinnt die Semantik per Epsilon-Float-Vergleich
- `cargo check --workspace` + `cargo test --workspace` gruen (0 Failures); Binary-Smoke-Test exit 124 (Startup ok)

## Task Commits

Keine Commits erstellt — GSD-Auto-Commit ist deaktiviert, User committed manuell via jj.

Abgearbeitet:
1. **Task 1: Additive Migration + .sqlx-Regen** — Migration angelegt, applied, .sqlx-Cache regeneriert
2. **Task 2: Field-Threading DAO-Trait + DAO-Impl + Service + rest-types + Carry-Forward** — alle Layer erledigt, zusaetzliche Struct-Init-Stellen in Tests und dev.rs automatisch mitgezogen (Rule 1), CVC-02 Test hinzugefuegt

## Files Created/Modified

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20260623120000_add-committed-voluntary-to-employee-work-details.sql` - Additive Migration ALTER TABLE ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao/src/employee_work_details.rs` - EmployeeWorkDetailsEntity: +committed_voluntary: f32
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao_impl_sqlite/src/employee_work_details.rs` - EmployeeWorkDetailsDb: +committed_voluntary: f64; TryFrom: as f32; 4 SELECT; INSERT; UPDATE
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/employee_work_details.rs` - EmployeeWorkDetails: +committed_voluntary: f32; From<&Entity>; TryFrom<&EmployeeWorkDetails>
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/employee_work_details.rs` - CVC-02 carry-forward: entity.committed_voluntary = employee_work_details.committed_voluntary
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/employee_work_details.rs` - entity_with_cap_and_committed Fixture-Helper; neuer CVC-02-Test update_propagates_committed_voluntary_to_dao
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs` - EmployeeWorkDetailsTO: +#[serde(default)] committed_voluntary: f32; beide From-Impls
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/dev.rs` - Dev-Fixture +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/reporting.rs` - Test-Fixtures +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/reporting_phase2_fixtures.rs` - +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/billing_period_report.rs` - +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/vacation_balance.rs` - +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test.rs` - 11x +committed_voluntary: 0.0 (Rule 1 fix)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test/employee_work_details_update.rs` - +committed_voluntary: 0.0 (Rule 1 fix)

## Decisions Made

- **Kein ToSchema/utoipa**: `EmployeeWorkDetailsTO` hat bewusst kein `ToSchema` — serde-transparent, unveraendert gelassen
- **Kein Snapshot-Bump**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7 — Feld ist in Phase 14 inert (kein persistierter value_type aendert sich; Bump ist Phase 15)
- **Typ-Wahl**: Row-Struct `f64` (wie expected_hours), alle anderen Layer `f32` — keine Bool-Coercion wie bei cap_planned_hours_to_expected
- **CVC-02 Test**: Epsilon-Float-Vergleich (`(e.committed_voluntary - 2.5).abs() < f32::EPSILON`) statt `==`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Struct-Init fehlschlaegegebeschleuniger in dev.rs und Test-Fixtures**
- **Found during:** Task 2 (cargo check nach Feld-Hinzufuegen)
- **Issue:** Alle bestehenden `EmployeeWorkDetails`-Struct-Inits in Fixtures und Tests wurden nach dem neuen Pflichtfeld `committed_voluntary` zu Compile-Errors. Betroffen: rest/src/dev.rs, service_impl/src/reporting.rs (2x), service_impl/src/test/reporting_phase2_fixtures.rs, service_impl/src/test/billing_period_report.rs, service_impl/src/test/vacation_balance.rs, shifty_bin/src/integration_test.rs (11x), shifty_bin/src/integration_test/employee_work_details_update.rs
- **Fix:** Alle Struct-Inits um `committed_voluntary: 0.0,` ergaenzt (default, semantisch korrekt fuer Bestandstests)
- **Files modified:** rest/src/dev.rs, service_impl/src/reporting.rs, service_impl/src/test/reporting_phase2_fixtures.rs, service_impl/src/test/billing_period_report.rs, service_impl/src/test/vacation_balance.rs, shifty_bin/src/integration_test.rs, shifty_bin/src/integration_test/employee_work_details_update.rs
- **Verification:** cargo test --workspace exit 0

---

**Total deviations:** 1 auto-fixed (Rule 1 - notwendige Compile-Fehler in Struct-Inits behoben)
**Impact on plan:** Alle Fixes notwendig fuer Korrektheit. Kein Scope Creep. Keine Regressionen.

## Issues Encountered

- `cargo sqlx prepare --workspace` konnte nicht direkt ueber nix develop aufgerufen werden, da `~/.cargo/bin/cargo-sqlx` (ohne libssl) gegenueber dem nix-Pfad Vorrang hatte. Geloest durch explizites Setzen von `CARGO=$(which cargo)` und direktem Aufruf des nix-`cargo-sqlx` Binaries: `CARGO=$(which cargo) /nix/store/...-sqlx-cli-0.8.6/bin/cargo-sqlx sqlx prepare --workspace`

## User Setup Required

Keine — additive Migration wurde via `sqlx migrate run` applied; keine externen Services benoetigt.

## Next Phase Readiness

- `committed_voluntary: f32` ist durchgehend persistierbar und transportierbar im gesamten Backend
- `.sqlx`-Cache regeneriert; `cargo check --workspace` + `cargo test --workspace` gruen
- Phase 14-02 (Tests: CVC-03 SUM-Aggregations-Test + Round-Trip) kann direkt aufsetzen
- Phase 15 (Reporting-Integration) kann das Feld lesen und den Snapshot-Bump durchfuehren
- Phase 16/17 (Frontend) kann das Feld konsumieren und editierbar machen

---
*Phase: 14-data-model-foundation-backend*
*Completed: 2026-06-23*
