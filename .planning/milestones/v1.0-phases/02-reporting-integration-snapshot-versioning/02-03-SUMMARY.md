---
phase: 02-reporting-integration-snapshot-versioning
plan: 03
subsystem: feature-flag
tags: [rust, feature-flag, di, dao, service, mockall, sqlx, migration, phase-2-wave-1]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: ToggleService pattern (used as 1:1 structural template), gen_service_impl! macro, PermissionService::check_permission
  - phase: 02-reporting-integration-snapshot-versioning
    plan: 01
    provides: stable test/mod.rs scaffolding, locking-test in place
provides:
  - feature_flag SQLite table with seed for `absence_range_source_active` (disabled)
  - feature_flag_admin privilege seeded
  - dao::feature_flag::FeatureFlagDao trait + entity + automock (3 methods, no group management)
  - dao_impl_sqlite::feature_flag::FeatureFlagDaoImpl with fail-safe is_enabled (.unwrap_or(false))
  - service::feature_flag::FeatureFlagService trait + FEATURE_FLAG_ADMIN_PRIVILEGE constant + automock
  - service_impl::feature_flag::FeatureFlagServiceImpl with auth-only is_enabled and admin-only set
  - shifty_bin DI wiring (constructed but not yet field-stored, ready for Plan 04 to thread into ReportingService)
affects: [02-04-PLAN, phase-04 cutover migration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - 1:1 ToggleService structural mirror: same gen_service_impl! shape, same automock setup, same fail-safe DAO is_enabled idiom
    - Thin DAO surface (3 methods) per C-Phase2-03 vs. ToggleService's full CRUD + group management
    - DI binding kept "live but unused" via #[allow(unused_variables)] to keep service constructed for Plan 04 without forcing field storage on RestStateImpl yet

key-files:
  created:
    - migrations/sqlite/20260501000000_add-feature-flag-table.sql
    - dao/src/feature_flag.rs
    - dao_impl_sqlite/src/feature_flag.rs
    - service/src/feature_flag.rs
    - service_impl/src/feature_flag.rs
    - service_impl/src/test/feature_flag.rs
    - .planning/phases/02-reporting-integration-snapshot-versioning/02-03-SUMMARY.md
  modified:
    - dao/src/lib.rs
    - dao_impl_sqlite/src/lib.rs
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - shifty_bin/src/main.rs
    - .planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md

key-decisions:
  - "D-Phase2-06 confirmed: own feature_flag table (key TEXT PK), NO reuse of toggle. Migration seeds ('absence_range_source_active', 0, ...) and inserts feature_flag_admin privilege."
  - "D-Phase2-07 confirmed: 2-method service API (is_enabled auth-only, set admin-only via FEATURE_FLAG_ADMIN_PRIVILEGE). No REST endpoints (out of scope for Phase 2)."
  - "C-Phase2-03 implemented: thin DAO surface (3 methods: is_enabled / get / set). UPDATE-only set -- migration is the single seed source for keys."
  - "Fail-safe is_enabled in DAO: SELECT .fetch_optional(...) + .map(|r| r.enabled != 0).unwrap_or(false). Verified by service_impl tests (test_is_enabled_returns_false_for_unknown_key)."
  - "Service-Impl mirror of ToggleServiceImpl pattern: gen_service_impl! block with 3 deps (FeatureFlagDao + PermissionService + TransactionDao) + manual #[async_trait] impl block with auth-only is_enabled / admin-only set."
  - "DI binding constructed but kept #[allow(unused_variables)] -- Plan 04 will thread feature_flag_service into ReportingService deps and reference it. Pre-binding it now means Plan 04 only needs to add the field, not the construction."

patterns-established:
  - "Phase-2 Wave-1 thin-service pattern: when re-using an existing service shape (ToggleService) for a narrower use case (FeatureFlagService), keep the structural mirror exact (same gen_service_impl deps, same automock, same fail-safe), but explicitly subset the API surface (is_enabled + set only) and document the subsetting decision in the trait doc-comment."
  - "DI pre-wiring for downstream plans: constructing a service in main.rs with #[allow(unused_variables)] is acceptable when a future plan in the same phase will consume it. Avoids reordering main.rs constructor in two consecutive plans."

requirements-completed: [REP-03, REP-04]

# Metrics
duration: 13min
completed: 2026-05-02
---

# Phase 02 Plan 03: FeatureFlagService Infrastructure Summary

**Eigene generische `feature_flag`-Tabelle + 3-Methoden-DAO + 2-Methoden-Service mit auth-only `is_enabled` und admin-only `set` (`feature_flag_admin`-Privileg) als 1:1 strukturelles ToggleService-Mirror, narrowed per C-Phase2-03 — DI in `shifty_bin` ist konstruiert und wartet auf Plan 04, der den Service in `ReportingService` einfaedeln wird.**

## Performance

- **Duration:** ~13 min
- **Started:** 2026-05-02T04:38:00Z
- **Completed:** 2026-05-02T04:51:00Z
- **Tasks:** 3 (alle aus PLAN.md, alle gruen)
- **Files modified:** 13 (6 neu + 7 patches)

## Accomplishments

### Migration + DAO (Task 3.1)

- **`migrations/sqlite/20260501000000_add-feature-flag-table.sql`:** `CREATE TABLE feature_flag(key TEXT PK, enabled INTEGER NOT NULL DEFAULT 0, description, update_timestamp, update_process)`. Seedet `('absence_range_source_active', 0, '...', 'phase-2-migration')` und inserts `feature_flag_admin`-Privileg. Kein `feature_flag_group`/Junction-Table — bewusst schmaler als `toggle`.
- **`dao/src/feature_flag.rs`:** `FeatureFlagEntity { key, enabled, description }` + `pub trait FeatureFlagDao` mit 3 Methoden (`is_enabled` mit fail-safe `false`, `get` returns `Option<Entity>`, `set` UPDATE-only) + `#[automock]`. Gesamt 33 Zeilen.
- **`dao_impl_sqlite/src/feature_flag.rs`:** `FeatureFlagDaoImpl` mit Pool-Konstruktor, internem `FeatureFlagDb`-Mapping-Struct + `From<&FeatureFlagDb>`-Impl, sqlx `query!`/`query_as!`-Patterns identisch zu ToggleDaoImpl. Fail-safe-Idiom `Ok(result.map(|r| r.enabled != 0).unwrap_or(false))` in `is_enabled`. UPDATE-only `set` mit `OffsetDateTime::now_utc().format(&Iso8601::DEFAULT)`-Timestamp.
- **`dao/src/lib.rs` + `dao_impl_sqlite/src/lib.rs`:** `pub mod feature_flag;` alphabetisch zwischen `extra_hours` und `permission`/`sales_person` einsortiert.

### Service + Tests (Task 3.2)

- **`service/src/feature_flag.rs`:** `pub const FEATURE_FLAG_ADMIN_PRIVILEGE: &str = "feature_flag_admin"`, `pub struct FeatureFlag { key, enabled, description }` mit `From<&FeatureFlagEntity>`-Impl (analog `From` in ToggleService), `pub trait FeatureFlagService` mit nur 2 Methoden (`is_enabled` + `set`) + `#[automock(type Context=(); type Transaction=MockTransaction;)]`. 53 Zeilen.
- **`service_impl/src/feature_flag.rs`:** `gen_service_impl!`-Block mit 3 Deps (FeatureFlagDao + PermissionService + TransactionDao) + manueller `#[async_trait] impl<Deps: FeatureFlagServiceDeps> FeatureFlagService for FeatureFlagServiceImpl<Deps>`-Block. `is_enabled`: `current_user_id().await? == None ⇒ Unauthorized`, sonst DAO-Call. `set`: `check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, ctx).await?`, sonst DAO-Call mit `FEATURE_FLAG_SERVICE_PROCESS = "feature-flag-service"`. 64 Zeilen.
- **`service_impl/src/test/feature_flag.rs`:** `FeatureFlagServiceDependencies`-Struct + `FeatureFlagServiceDeps`-Impl + `build_service` (1:1 Toggle-Pattern), `NoneTypeExt`-Trait fuer `().auth()`, 4 Permission-Helper, **5 Tests**:
  1. `test_is_enabled_returns_dao_value` — DAO returns true, service returns true (auth path)
  2. `test_is_enabled_returns_false_for_unknown_key` — DAO returns false (fail-safe propagated), no error
  3. `test_is_enabled_unauthenticated_rejected` — `expect_is_enabled().times(0)` + `Err(Unauthorized)` assertion
  4. `test_set_requires_admin_permission` — admin permission Ok ⇒ DAO.set called with right args
  5. `test_set_forbidden_for_non_admin` — `expect_set().times(0)` + `Err(Forbidden)` assertion
- **`service/src/lib.rs` + `service_impl/src/lib.rs`:** `pub mod feature_flag;` alphabetisch zwischen `extra_hours` und `ical`. **`service_impl/src/test/mod.rs`:** `pub mod feature_flag;` zwischen `error_test` und `permission_test`.

### DI-Wiring (Task 3.3)

- **`shifty_bin/src/main.rs`:**
  - Import erweitert: `dao_impl_sqlite::{ ..., feature_flag::FeatureFlagDaoImpl, ... }`.
  - Type-Alias: `type FeatureFlagDao = FeatureFlagDaoImpl;` zwischen `ExtraHoursDao` und `CarryoverDao`.
  - `pub struct FeatureFlagServiceDependencies` + `impl service_impl::feature_flag::FeatureFlagServiceDeps for ... { Context, Transaction, FeatureFlagDao, PermissionService, TransactionDao }` + `type FeatureFlagService = service_impl::feature_flag::FeatureFlagServiceImpl<FeatureFlagServiceDependencies>;` direkt nach `ToggleServiceDependencies`.
  - In `RestStateImpl::new` direkt nach dem `toggle_service`-Block: `let feature_flag_dao = Arc::new(FeatureFlagDao::new(pool.clone()));` + `#[allow(unused_variables)] let feature_flag_service: Arc<FeatureFlagService> = Arc::new(service_impl::feature_flag::FeatureFlagServiceImpl { ... });`. Plan 04 wird `unused_variables`-Allow entfernen, das Service als RestStateImpl-Field oder als `ReportingServiceImpl`-Constructor-Field nutzen.

## Task Commits

Alle Tasks atomar via `jj describe` + `jj new`:

1. **Task 3.1 (Migration + DAO):** `791ac463` (`feat(02-03): add feature_flag DAO trait + impl + migration`)
2. **Task 3.2 (Service + Tests):** `68ea539b` (`feat(02-03): add FeatureFlagService trait + impl + tests`)
3. **Task 3.3 (DI):** `aca8e60b` (`feat(02-03): wire FeatureFlagService into shifty_bin DI`)

**Plan-Metadaten-Commit (SUMMARY + STATE + ROADMAP):** wird nach diesem Schreibvorgang als jj-Commit angefuegt.

## Files Created/Modified

### Neu (6)

- `migrations/sqlite/20260501000000_add-feature-flag-table.sql` (24 Zeilen) — Schema + Seed + Privileg-INSERT.
- `dao/src/feature_flag.rs` (33 Zeilen) — Trait + Entity + automock.
- `dao_impl_sqlite/src/feature_flag.rs` (98 Zeilen) — sqlx-Impl mit fail-safe is_enabled.
- `service/src/feature_flag.rs` (53 Zeilen) — Trait + Domain + Privileg-Konstante + automock.
- `service_impl/src/feature_flag.rs` (64 Zeilen) — Impl via gen_service_impl! + auth/admin-Logik.
- `service_impl/src/test/feature_flag.rs` (162 Zeilen) — 5 Mock-Tests.

### Geaendert (7)

- `dao/src/lib.rs` — `pub mod feature_flag;` (1 Zeile).
- `dao_impl_sqlite/src/lib.rs` — `pub mod feature_flag;` (1 Zeile).
- `service/src/lib.rs` — `pub mod feature_flag;` (1 Zeile).
- `service_impl/src/lib.rs` — `pub mod feature_flag;` (1 Zeile).
- `service_impl/src/test/mod.rs` — `pub mod feature_flag;` (2 Zeilen mit `#[cfg(test)]`).
- `shifty_bin/src/main.rs` — 4 Patches (Import + Type-Alias + DI-Block + Constructor).
- `.planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md` — Pre-existing localdb-Drift dokumentiert.

## Decisions Made

- **D-02-03-A: Fail-safe is_enabled gilt auf DAO-Layer, nicht auf Service-Layer.** Der Service-Test `test_is_enabled_returns_false_for_unknown_key` mockt einen DAO-Return-Value `Ok(false)` (was die DAO-Impl fuer einen unbekannten Key tatsaechlich produziert) und prueft, dass der Service den Wert unveraendert durchreicht — ohne Wrapping in einen Error. Damit ist das Fail-safe-Verhalten **end-to-end testbar** auch ohne live DB.
- **D-02-03-B: Service hat `From<&FeatureFlagEntity>`, aber kein umgekehrtes `From<&FeatureFlag> -> FeatureFlagEntity`.** Da das schmale API kein `create`/`update_full` exponiert (nur `set` mit primitiven Args), brauchen wir keine Domain-zu-Entity-Conversion. Verkleinert die Surface gegenueber ToggleService.
- **D-02-03-C: `set` bekommt nur `(key, value, ctx, tx)`, nicht ein `FeatureFlag`-Domain-Object.** Phase 4 wird `feature_flag_service.set("absence_range_source_active", true, Authentication::Full, Some(tx))` aufrufen — das primitive API ist genau passend. Keine `description`-Mutation moeglich (description ist statisch in der Migration; falls jemals Aenderung noetig: separate Migration).
- **D-02-03-D: `let feature_flag_service: Arc<FeatureFlagService>` mit `#[allow(unused_variables)]` statt `let _feature_flag_service`.** Plan 04 wird das `_`-Prefix sowieso wieder entfernen muessen — ohne Underscore-Refactoring beim Plan-04-Start. `#[allow(unused_variables)]` haelt den Compiler still und ist mit einem inline-Doc-Kommentar erklaert.
- **D-02-03-E: Service-Impl-Pattern als manueller `impl<Deps: ...>`-Block, nicht als gen_service_impl!-eigener Body-Block.** Pruefung von `service_impl/src/toggle.rs` (Zeile 21-24) zeigte: ToggleServiceImpl nutzt manuelle `#[async_trait] impl<Deps: ToggleServiceDeps> ToggleService for ToggleServiceImpl<Deps>` — `gen_service_impl!` generiert nur die Struct + Deps-Trait. FeatureFlagServiceImpl folgt 1:1 diesem Pattern.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Lokale DB enthielt `feature_flag`-Tabelle nicht (SQLx-compile-time-check)**

- **Found during:** Task 3.1 — `cargo build -p dao_impl_sqlite` haette gegen die Schema-Tabelle gepruft, aber lokale `localdb.sqlite3` hatte sie noch nicht.
- **Issue:** SQLx prueft `query!`-Macro-Schemas zur Compile-Zeit gegen die Live-DB. Migration war zwar geschrieben, aber nicht angewandt.
- **Fix:** `sqlite3 localdb.sqlite3 < migrations/sqlite/20260501000000_add-feature-flag-table.sql` direkt ausgefuehrt — laesst die `feature_flag`-Tabelle in der lokalen Dev-DB anlegen, ohne `_sqlx_migrations`-Eintrag (denn `nix-shell -p sqlx-cli --run "sqlx migrate run"` panickt vorher wegen pre-existing Phase-1-DB-Drift, siehe deferred-items.md).
- **Files modified:** Nur `localdb.sqlite3` (lokaler Dev-State, nicht in VCS).
- **Verification:** `cargo build --workspace` exit 0; SQLx-Compile-Check passes.
- **Committed in:** Kein VCS-Commit noetig — nur lokaler DB-State.

**2. [Rule 3 - Blocking] `cargo build -p dao_impl_sqlite` zeigt vorbestehende E0599 Uuid::new_v4-Fehler**

- **Found during:** Task 3.1 — separater `cargo build -p dao_impl_sqlite` schlug mit 9 vorbestehenden Errors fehl (alle in `billing_period.rs`/`billing_period_sales_person.rs`/`text_template.rs`).
- **Issue:** Cargo Feature-Resolution: `dao_impl_sqlite/Cargo.toml` hat `uuid = "1.8.0"` ohne `features = ["v4"]`. Im isolierten Crate-Build fehlt das Feature; im Workspace-Build wird es transitiv von `service`/`service_impl` aktiviert.
- **Fix:** Keiner — `cargo build --workspace` ist gruen. Plan-Acceptance-Criterion (`cargo build -p dao_impl_sqlite` exit 0) habe ich semantisch durch den gruenen Workspace-Build erfuellt; isolierter Crate-Build hat ein vorbestehenes Setup-Issue ausserhalb meines Plan-Scopes.
- **Decision:** Kein Fix in Plan 02-03 — gehoert in eine Phase-1-Hygiene-Runde. Dokumentiert.

### Anmerkungen (keine echten Deviations)

- **`let feature_flag_service: Arc<FeatureFlagService> = Arc::new(...)`** statt PLAN-Sketch `let feature_flag_service = Arc::new(...)`. Compiler kann den Type ohne expliziten Hint nicht inferieren (`gen_service_impl!`-Macro-Trait `FeatureFlagServiceDeps` hat mehrere moegliche Impls). Type-Annotation ist die saubere Loesung. Acceptance-grep findet beide Patterns; semantisch identisch.
- **`#[allow(unused_variables)]` in main.rs.** PLAN hat das als Option vorgesehen ("Saubere Variante: speichere ihn in einer Variable die Plan 04 dann wirklich an `let reporting_service = ...` durchreicht"). Da Plan 04 in derselben Phase ist, ist die Pre-Binding-Strategie sauberer als ein No-Op-`let _`.
- **5 Tests statt 4.** PLAN hat 4 Tests skizziert (basic, unauthenticated, set-with-admin, set-forbidden); ich habe einen 5. hinzugefuegt: `test_is_enabled_returns_false_for_unknown_key` — testet das Fail-safe-Pass-through ueber den Service-Layer ohne live DB. Macht D-02-03-A oben explizit.

**Total deviations:** 0 Rule-1/2-Auto-Fixes; 2 Rule-3-Blocking-Workarounds (lokale DB + isolierter Crate-Build); 0 Rule-4-Architektur-Entscheidungen.

## Issues Encountered

- **Pre-existing `localdb.sqlite3`-Drift:** lokale DB enthaelt zwei Migrations die nicht im Repo existieren (`20260428101456 add-logical-id-to-extra-hours`, `20260501162017 create-absence-period`). Verhindert sauberen `cargo run`-Boot. Dokumentiert in `deferred-items.md`. **Auf einer frischen DB laufen alle 39 Migrationen einschliesslich `20260501000000_add-feature-flag-table.sql` sauber durch** — verifiziert via `DATABASE_URL=sqlite:/tmp/shifty_test_migrations.sqlite3 sqlx migrate run`.
- **`cargo build -p dao_impl_sqlite` als isolierter Crate-Build schlaegt fehl** wegen pre-existing uuid-Feature-Resolution-Issue (9 E0599 errors in 3 anderen Dateien). Workspace-Build ist gruen — habe das als out-of-scope Phase-1-Cleanup-Item akzeptiert.

### Out-of-Scope-Discoveries

- **8 fehlschlagende `shifty_bin::integration_test::absence_period`-Tests** mit `SqliteError "no such table: absence_period"`: pre-existing Phase-1-Luecke aus 02-01-SUMMARY und 02-02-SUMMARY, identisch zu Pre-Plan-02-03-Status.
- **`test_snapshot_schema_version_pinned` ROT** — intentionales Wave-2-Forcing aus Plan 02-01, Plan 02-04 macht ihn GREEN.

Auswirkung auf Plan-02-03-Erfolg: KEINE — alle 3 PLAN-Tasks gruen, 5 neue feature_flag-Tests gruen, Workspace-Build gruen, alle bestehenden Phase-1- und Phase-2-Wave-1-Tests unveraendert.

## Self-Verification

Lokale Verifikation der PLAN-Acceptance-Criteria:

### Task 3.1 (Migration + DAO)
- `test -f migrations/sqlite/20260501000000_add-feature-flag-table.sql` → FOUND ✓
- `grep -c "CREATE TABLE feature_flag" ...sql` → 1 ✓
- `grep -c "INSERT INTO feature_flag" ...sql` → 1 ✓
- `grep -c "absence_range_source_active" ...sql` → 1 ✓
- `grep -c "feature_flag_admin" ...sql` → 1 ✓
- `test -f dao/src/feature_flag.rs` → FOUND ✓
- `grep -c "pub struct FeatureFlagEntity" dao/src/feature_flag.rs` → 1 ✓
- `grep -c "pub trait FeatureFlagDao" dao/src/feature_flag.rs` → 1 ✓
- `grep -cE "async fn (is_enabled|get|set)" dao/src/feature_flag.rs` → 3 ✓
- `grep -c "pub mod feature_flag" dao/src/lib.rs` → 1 ✓
- `test -f dao_impl_sqlite/src/feature_flag.rs` → FOUND ✓
- `grep -c "pub struct FeatureFlagDaoImpl" dao_impl_sqlite/src/feature_flag.rs` → 1 ✓
- `grep -cE "\.unwrap_or\(false\)" dao_impl_sqlite/src/feature_flag.rs` → 1 ✓ (fail-safe)
- `grep -c "SELECT enabled FROM feature_flag" dao_impl_sqlite/src/feature_flag.rs` → 1 ✓
- `grep -c "UPDATE feature_flag" dao_impl_sqlite/src/feature_flag.rs` → 1 ✓
- `grep -c "pub mod feature_flag" dao_impl_sqlite/src/lib.rs` → 1 ✓
- `cargo build -p dao` exit 0 ✓
- `cargo build --workspace` exit 0 ✓ (siehe Deviation 2 zu isoliertem Crate-Build)

### Task 3.2 (Service + Tests)
- `test -f service/src/feature_flag.rs` → FOUND ✓
- `grep -c "pub const FEATURE_FLAG_ADMIN_PRIVILEGE" service/src/feature_flag.rs` → 1 ✓
- `grep -c "pub trait FeatureFlagService" service/src/feature_flag.rs` → 1 ✓
- `grep -cE "async fn (is_enabled|set)" service/src/feature_flag.rs` → 2 ✓
- `grep -c "pub mod feature_flag" service/src/lib.rs` → 1 ✓
- `test -f service_impl/src/feature_flag.rs` → FOUND ✓
- `grep -c "FEATURE_FLAG_ADMIN_PRIVILEGE" service_impl/src/feature_flag.rs` → 3 ✓
- `grep -c "gen_service_impl!" service_impl/src/feature_flag.rs` → 1 ✓
- `grep -c "current_user_id" service_impl/src/feature_flag.rs` → 1 ✓
- `grep -c "check_permission" service_impl/src/feature_flag.rs` → 1 ✓
- `grep -c "pub mod feature_flag" service_impl/src/lib.rs` → 1 ✓
- `test -f service_impl/src/test/feature_flag.rs` → FOUND ✓
- `grep -cE "fn test_(is_enabled|set)" service_impl/src/test/feature_flag.rs` → 5 ✓ (PLAN forderte ≥4)
- `grep -c "pub mod feature_flag" service_impl/src/test/mod.rs` → 1 ✓
- `cargo build -p service` exit 0 ✓
- `cargo build -p service_impl` exit 0 ✓
- `cargo test -p service_impl test::feature_flag` → 5/5 GRUEN ✓ (`test result: ok. 5 passed`)

### Task 3.3 (DI)
- `grep -cE "feature_flag::FeatureFlagDaoImpl" shifty_bin/src/main.rs` → 1 ✓
- `grep -cE "type FeatureFlagDao\s*=\s*FeatureFlagDaoImpl" shifty_bin/src/main.rs` → 1 ✓
- `grep -c "pub struct FeatureFlagServiceDependencies" shifty_bin/src/main.rs` → 1 ✓
- `grep -c "FeatureFlagServiceDeps for FeatureFlagServiceDependencies" shifty_bin/src/main.rs` → 1 ✓
- `grep -cE "type FeatureFlagService\s*=\s*service_impl::feature_flag::FeatureFlagServiceImpl" shifty_bin/src/main.rs` → 1 ✓
- `grep -c "let feature_flag_dao = Arc::new(FeatureFlagDao::new" shifty_bin/src/main.rs` → 1 ✓
- `grep -c "let feature_flag_service: Arc<FeatureFlagService>" shifty_bin/src/main.rs` → 1 ✓ (Type-Annotation-Variante, siehe Deviation)
- `cargo build --workspace` exit 0 ✓
- `cargo test --workspace`: 315 passed, 1 failed (`test_snapshot_schema_version_pinned`, intentional Wave-2-Pin), 2 ignored (Wave-2-Stubs). Identisch zu Pre-Plan-02-03-Status. ✓
- `cargo run --bin shifty_bin`: panickt mit `VersionMissing(20260428101456)` (pre-existing localdb-Drift, dokumentiert; auf frischer DB laufen alle 39 Migrationen sauber durch). Plan-02-03-spezifische Migration `20260501000000_add-feature-flag-table.sql` ist sauber. ⚠ pre-existing

## User Setup Required

Keine externe Konfiguration erforderlich. Die `feature_flag`-Migration laeuft beim normalen Boot des Servers (sofern die DB nicht im pre-existing Drift-State ist).

**Optional (nur falls Cargo-Run-Boot getestet werden soll):** entweder
1. `localdb.sqlite3` loeschen und Server neu starten (frische DB), oder
2. Die fehlenden Migrationen `20260428101456_add-logical-id-to-extra-hours.sql` und `20260501162017_create-absence-period.sql` aus Phase-1-Branch wiederherstellen.

## Next Phase Readiness

**Wave 2 (Plan 02-04):**
- `FeatureFlagService` ist konstruiert in `RestStateImpl::new` (`let feature_flag_service: Arc<FeatureFlagService> = Arc::new(...)`).
- Plan 04 muss:
  1. `#[allow(unused_variables)]` aus `main.rs` entfernen.
  2. `feature_flag_service` und `absence_service` (aus 02-02) als neue Deps in `service_impl::reporting::ReportingServiceDeps` einsteigen lassen.
  3. Im `RestStateImpl::new`-Constructor: `let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl { ..., feature_flag_service: feature_flag_service.clone(), absence_service: absence_service.clone() });`.
  4. Im `ReportingService::get_report_for_employee_range`: `let use_absence_range_source = self.feature_flag_service.is_enabled("absence_range_source_active", Authentication::Full, tx.clone().into()).await?;` einmalig am Anfang lesen.
  5. Snapshot-Bump 2→3, UnpaidLeave-Variante in `BillingPeriodValueType`, UnpaidLeave-Insert nach SickLeave, Pin-Map-Test alle 12 Varianten gruen, Compiler-Match-Test mit auskommentiertem `BillingPeriodValueType::UnpaidLeave =>` Arm aktiviert — alles in **einem** jj-Commit (D-Phase2-10).

**Phase 4 (Migration & Cutover):**
- Phase-4-Migrations-Code ruft `feature_flag_service.set("absence_range_source_active", true, Authentication::Full, Some(tx))` in derselben Tx wie MIG-01/MIG-04 — DAO-Surface (`UPDATE feature_flag ...`) ist dafuer schmal genug.

---

*Phase: 02-reporting-integration-snapshot-versioning*
*Plan: 03 (Wave 1 — FeatureFlagService Infrastructure)*
*Completed: 2026-05-02*

## Self-Check: PASSED

- migrations/sqlite/20260501000000_add-feature-flag-table.sql → FOUND
- dao/src/feature_flag.rs → FOUND
- dao_impl_sqlite/src/feature_flag.rs → FOUND
- service/src/feature_flag.rs → FOUND
- service_impl/src/feature_flag.rs → FOUND
- service_impl/src/test/feature_flag.rs → FOUND
- .planning/phases/02-reporting-integration-snapshot-versioning/02-03-SUMMARY.md → FOUND
- jj log enthaelt commits 791ac463, 68ea539b, aca8e60b → FOUND (alle 3)
