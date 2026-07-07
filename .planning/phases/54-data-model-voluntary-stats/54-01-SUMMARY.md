---
phase: 54-data-model-voluntary-stats
plan: 01
subsystem: backend/data-model
tags: [migrations, dao, extra-hours, rebooking, voluntary-stats, phase-54, wave-1]
status: complete
requires:
  - Existing extra_hours table + toggle table (v1.x baseline)
  - Existing dao / dao_impl_sqlite / service module conventions (week_status precedent)
provides:
  - migrations/sqlite/20260707000000_create-rebooking-batch.sql
  - migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql
  - migrations/sqlite/20260707000002_seed-voluntary-rebooking-toggle.sql
  - dao::rebooking_batch (RebookingBatchDao trait, RebookingBatchEntity, RebookingBatchEntryEntity, Kind + State enums)
  - dao_impl_sqlite::rebooking_batch::RebookingBatchDaoImpl
  - dao::extra_hours::ExtraHoursEntity.source (String field)
  - service::extra_hours::ExtraHoursSource (Manual | Rebooking)
  - service::extra_hours::ExtraHours.source (ExtraHoursSource field)
affects:
  - dao_impl_sqlite::extra_hours (5 SELECT + 1 INSERT extended by 'source')
  - rest-types::ExtraHoursTO -> service::ExtraHours mapper (defaults source=Manual)
  - service_impl::extra_hours::update (propagates source from active row)
  - service_impl::shiftplan_edit (absence-convert vacation writers default to Manual)
  - rest::dev seed helper (defaults source=Manual)
  - 12 fixture files across service_impl tests + shifty_bin integration tests
tech-stack:
  added: []
  patterns:
    - Partial UNIQUE index (WHERE deleted IS NULL) enforces active-only uniqueness (precedent Phase 39 week_status)
    - Additive ALTER TABLE with DEFAULT for zero-backfill migration (precedent v1.x additive columns)
    - INSERT OR IGNORE toggle seed for idempotency (precedent v2.4 SHC-04)
    - Basic-tier DAO trait with automock (precedent WeekStatusDao)
    - kind/state str<->enum helpers with DaoError::EnumValueNotFound on unknowns (precedent WeekStatusKind)
key-files:
  created:
    - migrations/sqlite/20260707000000_create-rebooking-batch.sql
    - migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql
    - migrations/sqlite/20260707000002_seed-voluntary-rebooking-toggle.sql
    - dao/src/rebooking_batch.rs
    - dao_impl_sqlite/src/rebooking_batch.rs
  modified:
    - dao/src/lib.rs
    - dao/src/extra_hours.rs
    - dao_impl_sqlite/src/lib.rs
    - dao_impl_sqlite/src/extra_hours.rs
    - service/src/extra_hours.rs
    - service_impl/src/extra_hours.rs
    - service_impl/src/shiftplan_edit.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/extra_hours.rs
    - service_impl/src/test/absence_conversion.rs
    - service_impl/src/test/reporting_additive_merge.rs
    - service_impl/src/test/reporting_holiday_auto_credit.rs
    - service_impl/src/test/reporting_phase2_fixtures.rs
    - service_impl/src/test/reporting_year_boundary.rs
    - rest-types/src/lib.rs
    - rest/src/dev.rs
    - shifty_bin/src/integration_test.rs
    - shifty_bin/src/integration_test/absence_projection.rs
    - shifty_bin/src/integration_test/convert_route_slash.rs
    - shifty_bin/src/integration_test/convert_to_absence.rs
    - shifty_bin/src/integration_test/extra_hours_update.rs
    - .sqlx/ (regenerated for extra_hours + rebooking_batch queries)
decisions:
  - "[D-54-DM-01] Partial UNIQUE index on (sales_person_id, iso_year, iso_week) WHERE deleted IS NULL — global across all kinds (Claim-on-Suggest)"
  - "[D-54-DM-02] extra_hours.source TEXT NOT NULL DEFAULT 'manual' — additive marker, no separate backfill"
  - "Basic-tier trait: RebookingBatchDao consumes only Transaction; no Domain-Service dependencies"
  - "Frontend-facing writers (REST TO -> Service, dev seed, absence-convert, HR-CRUD) default source=Manual — Rebooking source only set by Phase 55/56 writers"
metrics:
  duration: 38m
  completed: 2026-07-07
  tasks: 6
  files_created: 5
  files_modified: 22
  tests: "880 passed / 0 failed (cargo test --workspace)"
  gates:
    build: green
    test: green
    clippy: "green (cargo clippy --workspace -- -D warnings)"
    sqlx_prepare: "green (cargo sqlx prepare --workspace --check)"
must_haves:
  truths_verified:
    - "[D-54-DM-01] verified — UNIQUE-Partial-Index blocks second active row for same (sp, y, w) (sqlite3 constraint violation observed in manual test)"
    - "[D-54-DM-02] verified — extra_hours.source is 'manual' by DEFAULT; ExtraHoursSource enum in service layer"
    - "voluntary_rebooking_auto_active_from toggle exists exactly once after migration; idempotent (INSERT OR IGNORE tested)"
  artifacts_verified:
    - "20260707000000_create-rebooking-batch.sql: 2 tables + 1 partial UNIQUE index + 2 performance indices verified via .schema/.indexes"
    - "20260707000001_add-source-column-to-extra-hours.sql: PRAGMA table_info shows column 13 = source TEXT NOT NULL DEFAULT 'manual'"
    - "20260707000002_seed-voluntary-rebooking-toggle.sql: SELECT COUNT(*) = 1 after first and second migrate run"
    - "dao/src/rebooking_batch.rs: 4 trait methods present, RebookingBatchEntity + Entry + Kind + State exported, MockRebookingBatchDao available"
    - "dao_impl_sqlite/src/rebooking_batch.rs: RebookingBatchDaoImpl compiles; roundtrip tests for kind/state pass"
    - "dao/src/extra_hours.rs: ExtraHoursEntity.source: String added with doc"
    - "dao_impl_sqlite/src/extra_hours.rs: ExtraHoursDb.source added, 5 SELECT + 1 INSERT extended; .sqlx cache regenerated"
    - "service/src/extra_hours.rs: enum ExtraHoursSource + source field on ExtraHours; From/TryFrom mappers round-trip via String"
---

# Phase 54 Plan 01: Data-Model + DAO-Skeleton Summary

**Additive Datenmodell-Basis für die gesamte v2.6-Rebooking-Domäne: drei neue Migrationen (2 Tabellen + 1 additive Spalte + 1 Toggle-Seed), 2 neue DAO-Traits/Impls, ExtraHoursSource-Marker durchgängig auf Service- und Entity-Schicht.**

## Was wurde gebaut

### 1. Drei SQLite-Migrationen (idempotent)

**Migration 1 — `20260707000000_create-rebooking-batch.sql`:**
- Tabelle `rebooking_batch` (Parent) mit 12 Spalten (id/sales_person_id/iso_year/iso_week/kind/state/created/approved/approved_by/deleted/update_process/update_version).
- Tabelle `rebooking_batch_entry` (Child) mit 13 Spalten inkl. FK auf `rebooking_batch(id)` (kein CASCADE — Soft-Delete-Muster).
- **UNIQUE-Partial-Index `rebooking_batch_week_unique_idx`** auf `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` — global über alle Kinds (D-54-DM-01, Claim-on-Suggest).
- Performance-Indices `rebooking_batch_state_idx` und `rebooking_batch_entry_sp_idx`.

**Migration 2 — `20260707000001_add-source-column-to-extra-hours.sql`:**
```sql
ALTER TABLE extra_hours ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
```
Bestandsrows landen automatisch auf `'manual'`; kein Backfill-Sweep nötig.

**Migration 3 — `20260707000002_seed-voluntary-rebooking-toggle.sql`:**
Idempotenter `INSERT OR IGNORE` in `toggle` für `voluntary_rebooking_auto_active_from` (enabled=0, value NULL). Wirkung greift erst Phase 56 (F4-Cron-Gate).

### 2. Neues DAO-Skelett — `RebookingBatchDao`

**`dao/src/rebooking_batch.rs`:**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingBatchKind {
    Manual, HrSuggestion, AutoCron, AutoCronBackfill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingBatchState {
    Pending, Approved, Rejected, SkippedLocked,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait RebookingBatchDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(&self, id: Uuid, tx: Self::Transaction)
        -> Result<Option<RebookingBatchEntity>, DaoError>;
    async fn find_by_sales_person_year_week(
        &self, sales_person_id: Uuid, iso_year: u32, iso_week: u8, tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError>;
    async fn create_batch_with_entries(
        &self, batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        process: &str, tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn list_entries_for_batch(
        &self, batch_id: Uuid, tx: Self::Transaction,
    ) -> Result<Arc<[RebookingBatchEntryEntity]>, DaoError>;
}
```

**`dao_impl_sqlite/src/rebooking_batch.rs`:** vollständige `RebookingBatchDaoImpl` nach dem `WeekStatusDaoImpl`-Muster (`kind_to_str`/`state_to_str`-Helper, `TryFrom<&…Db>` mit `Uuid::from_slice` + `Iso8601::DATE_TIME`-Parsing, `DaoError::EnumValueNotFound` bei unbekannten Discriminants). Unit-Tests roundtripen Kind + State und prüfen den Unknown-Discriminant-Pfad.

### 3. ExtraHoursSource-Marker durchgängig

**`service/src/extra_hours.rs`:**

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExtraHoursSource {
    #[default] Manual,     // UI / HR-CRUD / Absence-Convert
    Rebooking,             // F3/F4/F5 writers (ab Phase 55) — gefiltert von F1/F2-Aggregatoren
}
```

`as_str()` + `TryFrom<&str>` + `#[derive(Default)]` mit `#[default]` auf `Manual`. Feld `pub source: ExtraHoursSource` auf `ExtraHours` (nach `version: Uuid`). From/TryFrom-Mapper zwischen Service-Enum und `ExtraHoursEntity.source: String` sind roundtrip-sicher (unbekannter DB-Wert → Fallback auf `Manual` mit `unwrap_or`).

**`dao/src/extra_hours.rs`:** `pub source: String` auf `ExtraHoursEntity`.

**`dao_impl_sqlite/src/extra_hours.rs`:** `ExtraHoursDb.source: String`; alle 5 SELECT-Queries und die INSERT-Query um `, source` erweitert; `.sqlx/`-Cache regeneriert.

## Verifikationsergebnisse

| Gate | Ergebnis |
| ---- | -------- |
| `cargo build --workspace` | green |
| `cargo test --workspace` | 880 passed / 0 failed |
| `cargo clippy --workspace -- -D warnings` | green |
| `cargo sqlx prepare --workspace --check` | green (offline cache aktuell) |
| `sqlx migrate run` (leere DB) | 3 Migrationen sauber applied |
| `sqlx migrate run` (zweiter Lauf) | idempotent — keine Änderung |
| UNIQUE-Constraint manueller INSERT-Test | zweiter aktiver Row → `UNIQUE constraint failed` (verifiziert) |
| Schema-Dump `.schema rebooking_batch` | `rebooking_batch`, `rebooking_batch_entry`, `rebooking_batch_week_unique_idx`, `rebooking_batch_state_idx`, `rebooking_batch_entry_sp_idx` alle vorhanden |
| `PRAGMA table_info(extra_hours)` | `source TEXT NOT NULL DEFAULT 'manual'` als Spalte 13 |
| `SELECT COUNT(*) FROM toggle WHERE name='voluntary_rebooking_auto_active_from'` | 1 (nach beiden Migrate-Runs) |

## Commits (6 Tasks atomar)

| Commit | Task | Files |
| ------ | ---- | ----- |
| `32b08ff` | Task 1: Rebooking-Batch-Tabellen | 1 Migration |
| `d0e87bd` | Task 2: source-Spalte + Toggle-Seed | 2 Migrationen |
| `b09ca43` | Task 3: DAO-Trait + Entities | dao/src/rebooking_batch.rs + lib.rs |
| `b7ef5c0` | Task 4: DAO-Impl (sqlite) + ExtraHoursDb.source + .sqlx | dao_impl_sqlite + dao/src/extra_hours.rs + .sqlx/ |
| `1e42975` | Task 5: ExtraHoursSource enum + service::ExtraHours.source | service + REST TO Mapper + Absence-Convert Writer |
| `10bf35f` | Task 6: Fixture-Sweep + Clippy-Fix | 12 Test-Fixtures + `#[derive(Default)]` |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixture-Sweep über den Test-Suite**
- **Found during:** Task 6 (Wave-1-Gate `cargo test --workspace`)
- **Issue:** Nachdem `service::ExtraHours` das additive `source`-Feld bekam, brachen 8 weitere ExtraHours-Konstruktionen in `service_impl` (reporting.rs + 4 Reporting-Test-Files + extra_hours-Test) und 12 in `shifty_bin` Integration-Tests mit `E0063: missing field 'source'`.
- **Fix:** Python-basiertes bulk-Insert von `source: service::extra_hours::ExtraHoursSource::Manual,` direkt nach der letzten `version: XXX,`-Zeile in jedem betroffenen Block. Frontend-Dateien in `shifty-dioxus/` (die einen eigenen `ExtraHours`-Typ mit anderer Struktur haben) wurden vom Skript reverted (`git checkout`).
- **Files modified:** 12 (siehe key-files.modified)
- **Commit:** `10bf35f`

**2. [Rule 1 - Bug] Clippy `derivable_impls` auf ExtraHoursSource**
- **Found during:** Task 6 (`cargo clippy --workspace -- -D warnings`)
- **Issue:** Manueller `impl Default for ExtraHoursSource { fn default() -> Self { Self::Manual } }` triggerte `clippy::derivable_impls`.
- **Fix:** Ersetzt durch `#[derive(Default)]` auf dem enum plus `#[default]`-Attribut auf `Manual`-Variante. Semantisch identisch, clippy-konform.
- **Commit:** `10bf35f`

**3. [Rule 2 - Missing critical] rest-types → service ExtraHours Mapper braucht source**
- **Found during:** Task 5 build
- **Issue:** `impl From<&ExtraHoursTO> for service::extra_hours::ExtraHours` in `rest-types/src/lib.rs:876` konstruierte `ExtraHours` ohne das neue `source`-Feld → E0063.
- **Fix:** Feld ergänzt mit Default `ExtraHoursSource::Manual` und Kommentar, dass FE-erzeugte Rows immer manuell sind (Rebooking-Schreiber leben ausschließlich Backend-seitig ab Phase 55). Kein DTO-Feld-Add auf `ExtraHoursTO` nötig — Marker-Semantik ist Backend-only.
- **Commit:** `1e42975`

**4. [Rule 3 - Blocking] absence-convert Vacation-Writer + service_impl::extra_hours::update Konstruktoren**
- **Found during:** Task 5 build
- **Issue:** `service_impl/src/shiftplan_edit.rs` (2× im Vacation-Auto-Create-Pfad) und `service_impl/src/extra_hours.rs::update` konstruieren `ExtraHours` bzw. `ExtraHoursEntity` direkt.
- **Fix:** Bei `shiftplan_edit.rs` (ExtraHours) default `ExtraHoursSource::Manual` gesetzt (Absence-Convert-Pfad ist immer manuell). Bei `extra_hours.rs::update` (ExtraHoursEntity) `source: active.source.clone()` — der Update-Pfad propagiert das Feld von der bestehenden Row.
- **Commit:** `1e42975`

### Environmental issue

**5. [Rule 3 - Blocking] `cargo sqlx prepare` scheiterte auf user-installiertem `~/.cargo/bin/cargo-sqlx` (libssl.so.3)**
- **Found during:** Task 4 (`cargo sqlx prepare --workspace`)
- **Issue:** `~/.cargo/bin/cargo-sqlx` (user-installiert) findet auf NixOS die dynamische libssl-Bibliothek nicht (`libssl.so.3: cannot open shared object file`). Der Cargo-Subcommand-Lookup priorisiert `~/.cargo/bin/` vor dem Nix-shell-`sqlx-cli`.
- **Fix:** In `nix develop`-Shell `~/.cargo/bin` aus PATH gestrippt und `CARGO_HOME` auf `mktemp -d` gesetzt, damit Cargo den nix-store-Pfad `/nix/store/…-sqlx-cli-0.9.0/bin/cargo-sqlx` findet. Dies ist ein reines Toolchain-Setup-Detail, keine Code-Änderung.
- **Note:** Für spätere Phasen sollte der User erwägen, den user-installierten cargo-sqlx zu deinstallieren oder via `alias cargo-sqlx=/nix/store/...` in der Shell zu maskieren.

## Known Stubs

Keine. Alle Trait-Methoden auf `RebookingBatchDao` sind implementiert; das DAO-Skelett kommt ab Phase 55 in Nutzung.

## Threat Flags

Keine. Phase 54 legt nur Datenmodell + Basic-DAO-CRUD an. Keine neuen REST-Endpoints, keine Auth-Pfade, keine Trust-Boundary-Änderungen (der Toggle-Seed liegt in der bestehenden `toggle`-Tabelle, ist enabled=0, hat keine Wirkung bis Phase 56).

## Nächste Schritte (Wave 2, Plan 02+03)

Plan 02+03 (Wave 2) konsumieren das Datenmodell:

- **Plan 02:** Basic-Tier `RebookingBatchService` (Entity-Manager) + DI-Wiring in `shifty_bin/src/main.rs`.
- **Plan 03:** BL-Tier `VoluntaryStatsService` mit pure fns `voluntary_ist_total_for_year`, `contract_weeks_count`, `committed_voluntary_prorata_for_week`, `committed_voluntary_target_for_year` in `service_impl/src/reporting.rs`. Property-Test VOL-ACCT-03 (Rebooking-Neutralität) auf diesen pure fns.

Der `source`-Marker ist ab jetzt für alle F1/F2-Aggregatoren verfügbar; Plan 03 filtert `ExtraHours::source == ExtraHoursSource::Manual` beim F1-Zähler.

## Self-Check: PASSED

**Files exist:**
- `migrations/sqlite/20260707000000_create-rebooking-batch.sql`: FOUND
- `migrations/sqlite/20260707000001_add-source-column-to-extra-hours.sql`: FOUND
- `migrations/sqlite/20260707000002_seed-voluntary-rebooking-toggle.sql`: FOUND
- `dao/src/rebooking_batch.rs`: FOUND
- `dao_impl_sqlite/src/rebooking_batch.rs`: FOUND

**Commits exist:**
- `32b08ff`: FOUND
- `d0e87bd`: FOUND
- `b09ca43`: FOUND
- `b7ef5c0`: FOUND
- `1e42975`: FOUND
- `10bf35f`: FOUND

**Gates:**
- `cargo build --workspace`: green
- `cargo test --workspace`: 880 passed / 0 failed
- `cargo clippy --workspace -- -D warnings`: green
- `cargo sqlx prepare --workspace --check`: green
