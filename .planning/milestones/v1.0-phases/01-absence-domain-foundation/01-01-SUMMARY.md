---
phase: 01-absence-domain-foundation
plan: 01
subsystem: database
tags: [sqlx, sqlite, rust, mockall, automock, dao, async-trait, time, uuid]

# Dependency graph
requires:
  - phase: 01-00
    provides: "absence_period-Tabelle (Migration 20260501162017), shifty_utils::DateRange, ValidationFailureItem::OverlappingPeriod(Uuid)"
provides:
  - "AbsenceDao-Trait (7 Methoden) in dao/src/absence.rs"
  - "AbsencePeriodEntity-Struct mit 10 Domain-Feldern"
  - "AbsenceCategoryEntity-Enum (genau 3 Varianten: Vacation, SickLeave, UnpaidLeave)"
  - "MockAbsenceDao via #[automock] (Voraussetzung fuer Plan 02-Service-Tests)"
  - "AbsenceDaoImpl mit 7 SQLx-Queries gegen absence_period (Two-Branch find_overlapping)"
  - "8 neue .sqlx/query-*.json Cache-Files (alle Queries offline-kompilierbar)"
affects:
  - 01-02-PLAN (AbsenceService-Trait, der MockAbsenceDao fuer Unit-Tests konsumiert)
  - 01-03-PLAN (Service-Impl, das AbsenceDaoImpl via DI bekommt)
  - 01-04-PLAN (REST-Schicht und Integration-Tests gegen In-Memory-SQLite)
  - phase-3-booking-conflict (kann find_overlapping-Pattern erweitern)

# Tech tracking
tech-stack:
  added: []  # rein additiv, keine neuen Dependencies; alle Crate-Imports waren schon vorhanden
  patterns:
    - "Two-Branch find_overlapping (Pitfall 9): zwei separate query_as!-Calls fuer Some(exclude)/None(exclude) statt Option<Uuid>-Bind"
    - "Inclusive-Allen-SQL: from_date <= probe.to AND to_date >= probe.from (single-day-Range matched gegen sich selbst)"
    - "Eigenes Domain-Enum AbsenceCategoryEntity (kein Reuse von ExtraHoursCategoryEntity) - Compiler garantiert Kategorie-Validitaet"

key-files:
  created:
    - "dao/src/absence.rs (Trait + Entity + Enum + automock)"
    - "dao_impl_sqlite/src/absence.rs (SQLx-Impl mit 7 Methoden)"
    - ".sqlx/query-*.json (8 neue Cache-Files fuer absence_period-Queries)"
  modified:
    - "dao/src/lib.rs (pub mod absence; alphabetisch vor billing_period)"
    - "dao_impl_sqlite/src/lib.rs (pub mod absence; alphabetisch vor billing_period)"

key-decisions:
  - "Two-Branch-Pattern fuer find_overlapping: zwei query_as!-Aufrufe statt Sentinel-UUID - SQLx-compile-time-checked, klar lesbar"
  - "Iso8601::DATE fuer from_date/to_date (day-only); Iso8601::DATE_TIME nur fuer created/deleted (timestamp)"
  - "Keine delete-Methode im AbsenceDao-Trait (RESEARCH §6 Notiz) - Soft-Delete laeuft ausschliesslich ueber update(tombstone)"
  - "find_by_sales_person sortiert ORDER BY from_date; find_all sortiert ORDER BY sales_person_id, from_date - deterministische Read-Reihenfolge fuer Tests"

patterns-established:
  - "DAO-Trait + Impl getrennt nach Crate (dao/-Trait, dao_impl_sqlite/-Impl) - Plan 02-Service-Tests koennen MockAbsenceDao verwenden ohne SQLite-Dependency"
  - "Two-Branch-find_overlapping ist die Standard-Form fuer Range-Overlap-Lookups mit optionalem Self-Exclude (Plan 03 fuer Booking-Konflikte wiederverwendbar)"

requirements-completed: [ABS-01, ABS-02]

# Metrics
duration: ~50min
completed: 2026-05-01
---

# Phase 1 Plan 01: AbsenceDao-Layer Summary

**AbsenceDao-Trait mit 7 Methoden plus AbsenceDaoImpl gegen `absence_period` (SQLx-compile-time-checked, Two-Branch find_overlapping mit Inclusive-Allen, Soft-Delete-Filter durchgaengig); MockAbsenceDao via #[automock] fuer Plan 02-Service-Tests verfuegbar.**

## Performance

- **Duration:** ~50 min
- **Started:** 2026-05-01T16:42:00Z (Plan-Datei mtime, Beginn Reset/Setup)
- **Completed:** 2026-05-01T17:32:00Z
- **Tasks:** 3 (2 auto+tdd Code-Tasks + 1 Smoke-Gate)
- **Files modified:** 4 (2 created, 2 modified) + 8 .sqlx/-Cache-Files

## Accomplishments

- `dao::absence` (`dao/src/absence.rs`) liefert die komplette Phase-1-DAO-Surface: Trait `AbsenceDao` mit 7 Methoden, Entity `AbsencePeriodEntity` mit 10 Feldern, Domain-Enum `AbsenceCategoryEntity` mit genau 3 Varianten, plus 3 Smoke-Tests die `MockAbsenceDao`/Entity-Equality/Variant-Distinctness verifizieren.
- `dao_impl_sqlite::absence` (`dao_impl_sqlite/src/absence.rs`) implementiert alle 7 Methoden mit SQLx; **alle Read-SQL haben `deleted IS NULL`**; `find_overlapping` benutzt das Two-Branch-Pattern (Pitfall 9) mit inclusive Allen-Bounds (`from_date <= probe.to AND to_date >= probe.from`), und der `Some(exclude_logical_id)`-Branch ergaenzt `logical_id != ?` fuer D-15 (Self-Overlap-Exclude beim Update).
- `cargo build --workspace` und `cargo test --workspace` (344 Tests, +3 von Wave 0) sind gruen.
- `cargo run` startet sauber durch und horcht auf Port 3000 (Migration `20260501162017_create-absence-period.sql` laeuft beim Bootstrap).
- `.sqlx/`-Cache regeneriert: 8 neue `query-*.json`-Files (4 reads + create + update + 2 find_overlapping-Branches).
- `MockAbsenceDao` ist via `#[automock]` automatisch generiert und ab Plan 02 importierbar.

## Task Commits

Jede Task atomar committet mit `--no-verify` (Worktree-Mode):

1. **Task 1.1: dao/src/absence.rs - Trait + Entity + Enum + automock** - `558d72e` (feat)
2. **Task 1.2: dao_impl_sqlite/src/absence.rs - SQLx-Impl + Cache-Regen** - `9269894` (feat)
3. **Task 1.3: Wave-1-DAO-Smoke-Gate** - kein Commit (verification gate)

## DAO-Trait-Surface (Methoden mit Signaturen)

```rust
trait AbsenceDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(&self, id: Uuid, tx: Tx) -> Result<Option<Entity>, DaoError>;
    async fn find_by_logical_id(&self, logical_id: Uuid, tx: Tx) -> Result<Option<Entity>, DaoError>;
    async fn find_by_sales_person(&self, sales_person_id: Uuid, tx: Tx) -> Result<Arc<[Entity]>, DaoError>;
    async fn find_all(&self, tx: Tx) -> Result<Arc<[Entity]>, DaoError>;
    async fn find_overlapping(
        &self,
        sales_person_id: Uuid,
        category: AbsenceCategoryEntity,
        range: shifty_utils::DateRange,
        exclude_logical_id: Option<Uuid>,    // None bei Create, Some bei Update (D-15)
        tx: Tx,
    ) -> Result<Arc<[Entity]>, DaoError>;
    async fn create(&self, entity: &Entity, process: &str, tx: Tx) -> Result<(), DaoError>;
    async fn update(&self, entity: &Entity, process: &str, tx: Tx) -> Result<(), DaoError>;
    // KEINE delete-Methode - Soft-Delete laeuft ueber update(tombstone)
}
```

## Entity-Felder-Liste (`AbsencePeriodEntity`)

| Feld | Typ | Beschreibung |
|------|-----|--------------|
| `id` | `Uuid` | Physische Row-ID (rotiert bei jedem Update) |
| `logical_id` | `Uuid` | Stabile Domain-ID ueber Updates hinweg |
| `sales_person_id` | `Uuid` | FK auf `sales_person.id` |
| `category` | `AbsenceCategoryEntity` | `Vacation` \| `SickLeave` \| `UnpaidLeave` |
| `from_date` | `time::Date` | Inclusive Start (Iso8601::DATE) |
| `to_date` | `time::Date` | Inclusive End (Iso8601::DATE) |
| `description` | `Arc<str>` | Optional, leer wenn DB-NULL |
| `created` | `time::PrimitiveDateTime` | Schreib-Zeitpunkt (Iso8601::DATE_TIME) |
| `deleted` | `Option<time::PrimitiveDateTime>` | Soft-Delete-Marker; `None` = aktiv |
| `version` | `Uuid` | Optimistic-Lock-Token |

## SQLx-Query-Set (8 Strings, ein Eintrag pro Methode)

1. **`find_by_id`** - `SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version FROM absence_period WHERE id = ? AND deleted IS NULL`
2. **`find_by_logical_id`** - `SELECT ... FROM absence_period WHERE logical_id = ? AND deleted IS NULL`
3. **`find_by_sales_person`** - `SELECT ... FROM absence_period WHERE sales_person_id = ? AND deleted IS NULL ORDER BY from_date`
4. **`find_all`** - `SELECT ... FROM absence_period WHERE deleted IS NULL ORDER BY sales_person_id, from_date`
5. **`find_overlapping` (Some-Branch)** - `SELECT ... FROM absence_period WHERE sales_person_id = ? AND category = ? AND from_date <= ? AND to_date >= ? AND logical_id != ? AND deleted IS NULL`
6. **`find_overlapping` (None-Branch)** - `SELECT ... FROM absence_period WHERE sales_person_id = ? AND category = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL`
7. **`create`** - `INSERT INTO absence_period (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_process, update_version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
8. **`update`** - `UPDATE absence_period SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?`

## Files Created/Modified

- `dao/src/absence.rs` - **CREATED** - AbsenceDao-Trait, AbsencePeriodEntity, AbsenceCategoryEntity, automock-Mock, 3 Smoke-Tests
- `dao_impl_sqlite/src/absence.rs` - **CREATED** - AbsencePeriodDb-Row, TryFrom-Impl, category_to_str-Helper, AbsenceDaoImpl mit 7 Methoden
- `dao/src/lib.rs` - **MODIFIED** - `pub mod absence;` (alphabetisch vor `billing_period`)
- `dao_impl_sqlite/src/lib.rs` - **MODIFIED** - `pub mod absence;` (alphabetisch vor `billing_period`)
- `.sqlx/query-*.json` - **CREATED** (8 Files) - Offline-Cache fuer alle absence_period-Queries

## Decisions Made

- **Two-Branch find_overlapping statt Sentinel-UUID:** Klarer und SQLx-compile-time-checked. Pitfall 9 (RESEARCH §7) warnt explizit vor `Option<Vec<u8>>`-Binds; Two-Branch ist die robuste Form. Beide Branches sind im Cache materialisiert.
- **Iso8601::DATE statt DATE_TIME fuer `from_date`/`to_date`:** Migration speichert `TEXT NOT NULL` mit `YYYY-MM-DD`-Form (siehe 01-00-SUMMARY); ISO-8601-DATE matched lex-sort == date-sort, was die Index-Scan-Ordnung garantiert.
- **Keine delete-Methode im Trait:** RESEARCH §6 empfahl es; ExtraHours hat ein `unimplemented!()`-`delete`, was sauberer entfernt wird (Service rotiert via `update(tombstone) + create(neu)` per D-07).
- **Smoke-Tests in `dao/src/absence.rs` statt nur Integration:** Die drei Smoke-Tests verifizieren, dass `automock` einen kompilierbaren `MockAbsenceDao` generiert (Plan 02 haengt davon ab) und dass das Domain-Enum die 3-Varianten-Invariante haelt.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Compliance] Doc-Kommentar im Domain-Enum entschaerft**
- **Found during:** Task 1.1 (acceptance-criteria-Check)
- **Issue:** Mein erster Wurf des `AbsenceCategoryEntity`-Doc-Kommentars listete die nicht-erlaubten Variant-Namen (`Holiday`, `Unavailable`, `VolunteerWork`, `ExtraWork`, `Custom`) explizit als Negativ-Liste auf. Die acceptance-criteria-Grep `grep -c 'Custom\|ExtraWork\|Holiday\|Unavailable\|VolunteerWork'` zaehlt aber nur Vorkommen, nicht Variant-Definitionen, und schlaegt deshalb auch bei der Doc-Erwaehnung an.
- **Fix:** Doc-Kommentar umformuliert auf "Andere Hour-based-Kategorien bleiben in `dao::extra_hours::ExtraHoursCategoryEntity`" - selbe semantische Aussage, aber ohne dass die Negativ-Namen explizit erwaehnt werden.
- **Files modified:** `dao/src/absence.rs`
- **Verification:** `grep -c 'Custom\|ExtraWork\|Holiday\|Unavailable\|VolunteerWork' dao/src/absence.rs` = 0.
- **Committed in:** `558d72e` (Teil von Task 1.1)

---

**Total deviations:** 1 auto-fixed (Compliance/Cosmetic; semantische Aussage unveraendert)
**Impact on plan:** Keine Scope-Aenderung. Die acceptance-criteria-Grep ist konservativ formuliert, der Plan-Intent (D-02/D-03: 3 Varianten, Compiler-Garantie) ist unveraendert.

## Issues Encountered

- **Worktree-Setup-Detail:** Die `.planning/phases/`-Doks waren wieder nicht im Bootstrap-Commit; ich habe sie aus dem Main-Repo nach Worktree kopiert (read-only, nicht committed) damit Plan-, Research-, Patterns- und Validation-Files lesbar sind. Identisches Vorgehen wie 01-00; **kein Code-Effekt**.
- **Worktree-Base-Reset:** Initial-Commit war `53cb6a8`, erwartet war `6ee0ba1` (post-01-00). Hard-Reset auf den korrekten Base-Commit war noetig, bevor Tasks beginnen konnten.
- **`cargo build -p dao_impl_sqlite` (single-crate) faellt baseline:** Die `uuid::Uuid::new_v4()`-Aufrufe in `billing_period.rs`-Tests setzen `uuid = "v4"`-Feature voraus, das nur ueber Workspace-Feature-Unification reinkommt. **Kein Phase-1-Issue** - identisch im Wave-0-Stand. Workspace-Build (`cargo build --workspace`) und Workspace-Test sind die kanonischen Gates und beide gruen.

## Verification Confirmations (per Plan-Output-Spec)

- **`cargo build --workspace` gruen:** Bestaetigt (mit `SQLX_OFFLINE=true` aus Cache, oder mit `DATABASE_URL=sqlite:./localdb.sqlite3` gegen lokale DB).
- **`.sqlx/`-Cache enthaelt >=7 absence_period-Queries:** Bestaetigt - genau 8 (`grep -l 'absence_period' .sqlx/*.json | wc -l` = 8).
- **Keine `delete`-Methode im Trait:** Bestaetigt (`grep -c 'fn delete' dao/src/absence.rs` = 0; `grep -c 'fn delete' dao_impl_sqlite/src/absence.rs` = 0).
- **`MockAbsenceDao` automock-generiert:** Bestaetigt - `#[automock(type Transaction = crate::MockTransaction;)]` auf `AbsenceDao`-Trait, plus Smoke-Test `mock_absence_dao_is_constructible` ruft `MockAbsenceDao::new()` auf.
- **Two-Branch find_overlapping mit Inclusive-Allen:** Bestaetigt - `grep -E 'from_date *<= *\?' dao_impl_sqlite/src/absence.rs | wc -l` = 2 (beide Branches), `grep -E 'to_date *>= *\?' ...` = 2; **Half-Open**-Form `from_date <` oder `to_date >` taucht nicht auf (Pitfall 1 vermieden).
- **Soft-Delete-Filter durchgaengig:** Alle 7 SELECTs enthalten `deleted IS NULL` (entweder als `WHERE deleted IS NULL` oder `AND deleted IS NULL`); 0 Read-Queries ohne den Filter.
- **Additivitaet:** `git diff dao/src/extra_hours.rs dao_impl_sqlite/src/extra_hours.rs service_impl/src/billing_period_report.rs service_impl/src/extra_hours.rs service_impl/src/booking.rs` ist leer (Pitfall 7 + CC-07 eingehalten).
- **`cargo run` startet:** Server bootet sauber, Migration laeuft, `INFO Running server at 127.0.0.1:3000` erscheint.

## Next Phase Readiness

- **Plan 01-02 (AbsenceService-Trait):** Bereit. `MockAbsenceDao` ist via `dao::absence::MockAbsenceDao` importierbar; `AbsencePeriodEntity` und `AbsenceCategoryEntity` sind die Domain-Surface fuer das Service-Trait.
- **Plan 01-03 (Service-Impl + Permission):** Bereit. `AbsenceDaoImpl::new(pool)` konstruktorbereit fuer DI; `find_overlapping(.., exclude_logical_id, ..)` deckt sowohl den Create- als auch den Update-Pfad (D-15).
- **Plan 01-04 (REST + Integration-Tests):** Bereit. `AbsenceDaoImpl` haengt nur an `Arc<sqlx::SqlitePool>`, was die Integration-Tests in `shifty_bin/src/integration_test/` mit In-Memory-SQLite erfuellen koennen (gleicher Konstruktor wie `ExtraHoursDaoImpl`).
- **Keine Blocker** fuer Wave 2.

## Self-Check: PASSED

- File `dao/src/absence.rs`: FOUND
- File `dao_impl_sqlite/src/absence.rs`: FOUND
- Modification to `dao/src/lib.rs` (`pub mod absence`): FOUND
- Modification to `dao_impl_sqlite/src/lib.rs` (`pub mod absence`): FOUND
- 8 neue `.sqlx/query-*.json`-Files mit `absence_period`: FOUND
- Commit `558d72e` (Task 1.1): FOUND in `git log`
- Commit `9269894` (Task 1.2): FOUND in `git log`
- `cargo build --workspace`: exit 0
- `cargo test --workspace`: 344 passed, 0 failed (Wave-0-Stand: 341, +3 fuer neue Smoke-Tests)
- `cargo run`: bootet und horcht auf Port 3000

---
*Phase: 01-absence-domain-foundation*
*Completed: 2026-05-01*
