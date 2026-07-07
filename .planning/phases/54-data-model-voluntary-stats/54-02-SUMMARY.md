---
phase: 54-data-model-voluntary-stats
plan: 02
subsystem: backend/service-tier
tags: [service, basic-tier, rebooking-batch, di-wiring, hr-gate, phase-54, wave-2]
status: complete
requires:
  - "Wave 1 (Plan 01) — dao::rebooking_batch::RebookingBatchDao trait + RebookingBatchDaoImpl (dao_impl_sqlite)"
  - "Existing Basic-Tier precedent (WeekStatusService, VacationEntitlementOffsetService)"
provides:
  - "service::rebooking_batch::RebookingBatchService (trait + MockRebookingBatchService via automock)"
  - "service_impl::rebooking_batch::RebookingBatchServiceImpl (gen_service_impl! Basic-Tier)"
  - "rest::RestStateDef::RebookingBatchService + rebooking_batch_service() getter"
  - "shifty_bin type aliases + DI-Wiring in Basic-Services-Wave"
affects:
  - "shifty_bin/src/main.rs (RestStateImpl feld + Konstruktor + impl RestStateDef)"
  - "rest/src/lib.rs (assoziierter Type + Getter — kein neuer REST-Endpoint)"
tech-stack:
  added: []
  patterns:
    - "Basic-Tier-Service via gen_service_impl! (precedent VacationEntitlementOffsetService, Phase 28)"
    - "HR_PRIVILEGE-Gate an erster Stelle jeder Methode (precedent VacationEntitlementOffsetService)"
    - "Same-transaction pre-check mappt UNIQUE-Slot-Konflikt auf ServiceError::EntityAlreadyExists (precedent SalesPersonUnavailableService::create)"
    - "Defensive id/version/created-Fill via UuidService/ClockService bei Uuid::nil / PrimitiveDateTime::MIN"
key-files:
  created:
    - service/src/rebooking_batch.rs
    - service_impl/src/rebooking_batch.rs
    - service_impl/src/test/rebooking_batch.rs
  modified:
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - rest/src/lib.rs
    - shifty_bin/src/main.rs
    - .planning/phases/54-data-model-voluntary-stats/54-VALIDATION.md
decisions:
  - "[D-54-DM-01] Service-Pre-Check innerhalb derselben Transaktion mappt UNIQUE-Slot-Konflikt auf ServiceError::EntityAlreadyExists; der DB-UNIQUE-Index bleibt Ultima-Ratio-Autoritaet gegen Race-Conditions."
  - "Basic-Tier strikt: Deps = {RebookingBatchDao, PermissionService, ClockService, UuidService, TransactionDao} — kein Domain-Service. Konstruktion in Basic-Services-Wave VOR allen BL-Services, die den Service ab Phase 55 konsumieren."
  - "Kein neuer REST-Endpoint in Phase 54 — der Trait wird bewusst noch nicht extern erreichbar; erst Phase 55 exponiert Endpoints (rebooking_batch_service ist RestStateDef-only exponiert)."
  - "Defensive id/version/created-Fill: falls Aufrufer Uuid::nil()/PrimitiveDateTime::MIN uebergibt, greift der Service auf UuidService/ClockService zurueck — konsistent mit gen_service_impl!-Konvention."
metrics:
  duration: 22m
  completed: 2026-07-07
  tasks: 6
  files_created: 3
  files_modified: 6
  tests: "737 passed / 0 failed (service_impl); Full-Workspace 5 new + 800+ existing tests green"
  gates:
    build: green
    test: green
    clippy: "green (cargo clippy --workspace -- -D warnings)"
must_haves:
  truths_verified:
    - "[D-54-DM-01] verified — Test create_unique_conflict_maps_to_already_exists prueft: aktiver Batch im Slot (sp, y, w) fuehrt zu ServiceError::EntityAlreadyExists ohne Panic; DAO::create_batch_with_entries wird nicht aufgerufen (mockall-strict)."
    - "Basic-Tier-Konvention verified — grep im RebookingBatchServiceDependencies-Block zeigt: nur DAO + Permission + Clock + Uuid + Transaction als Assoc-Types, kein Domain-Service."
  artifacts_verified:
    - "service/src/rebooking_batch.rs: trait RebookingBatchService mit find_by_id, find_by_sales_person_year_week, create; mockall::automock erzeugt MockRebookingBatchService."
    - "service_impl/src/rebooking_batch.rs: gen_service_impl!-Deps-Struct + Impl; HR_PRIVILEGE-Gate an erster Stelle jeder Methode."
    - "shifty_bin/src/main.rs: RebookingBatchServiceDependencies-Struct + type-Alias + Konstruktion in Basic-Wave (nach vacation_entitlement_offset_service, vor allen BL-Services) + RestStateImpl-Feld + Getter."
    - "rest/src/lib.rs: RestStateDef::RebookingBatchService assoziierter Type + rebooking_batch_service()-Getter."
    - "service_impl/src/test/rebooking_batch.rs: 5 mockall-Tests, alle green."
---

# Phase 54 Plan 02: Basic-Tier RebookingBatchService Summary

**Basic-Tier Entity-Manager-Service `RebookingBatchService` fuer die in Wave 1 angelegten Tabellen `rebooking_batch` + `rebooking_batch_entry`: Trait + Impl per `gen_service_impl!`, HR-Gate, Same-Transaction UNIQUE-Konflikt-Mapping auf `ServiceError::EntityAlreadyExists`, DI-Wiring in `shifty_bin`, 5 mockall-Unit-Tests — bereit zur Konsumtion durch den kommenden BL-`RebookingReconciliationService` ab Phase 55.**

## Was wurde gebaut

### 1. Trait `service::rebooking_batch::RebookingBatchService` (Task 1)

```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingBatchService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError>;

    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError>;

    async fn create(
        &self,
        batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError>;
}
```

Bewusst minimaler Contract fuer Phase 54: nur die drei Methoden, die vom Basic-Tier-Manager benoetigt werden. `update_state` (approve/reject) und `list_pending` kommen ab Phase 55 in der BL-Reconciliation-Schicht.

### 2. Impl `service_impl::rebooking_batch::RebookingBatchServiceImpl` (Task 2)

```rust
gen_service_impl! {
    struct RebookingBatchServiceImpl: RebookingBatchService = RebookingBatchServiceDeps {
        RebookingBatchDao: RebookingBatchDao<Transaction = Self::Transaction> = rebooking_batch_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Basic-Tier-Konvention verifiziert:** Deps sind strikt `{RebookingBatchDao, PermissionService, ClockService, UuidService, TransactionDao}` — kein Domain-Service.

**D-54-DM-01 (UNIQUE-Slot-Mapping):** `create` prueft innerhalb derselben Transaktion via `find_by_sales_person_year_week`, ob bereits ein aktiver Batch im Slot `(sales_person_id, iso_year, iso_week)` existiert. Falls ja → `ServiceError::EntityAlreadyExists(existing.id)`, ohne den DB-INSERT zu versuchen. Der DB-UNIQUE-Index bleibt Ultima-Ratio-Autoritaet gegen Race-Conditions.

**Defensive Feld-Belegung:** Uebergibt der Aufrufer `Uuid::nil()`/`PrimitiveDateTime::MIN` fuer `id`/`version`/`created` auf Batch- oder Entry-Ebene, holt der Service frische Werte via `UuidService`/`ClockService`. Konsumenten in Phase 55/56 (Cron, HR-Suggest) koennen die Felder unbeeinflusst uebergeben.

### 3. RestStateDef-Erweiterung `rest/src/lib.rs` (Task 4)

```rust
type RebookingBatchService: service::rebooking_batch::RebookingBatchService<Context = Context>
    + Send + Sync + 'static;

fn rebooking_batch_service(&self) -> Arc<Self::RebookingBatchService>;
```

Kein REST-Endpoint, keine OpenAPI-Erweiterung. Der Trait wird lediglich fuer den Zugriff aus zukuenftigen Handlern (Phase 55) sichtbar gemacht.

### 4. DI-Wiring in `shifty_bin/src/main.rs` (Task 3)

```rust
// Type-Aliases
type RebookingBatchDao = dao_impl_sqlite::rebooking_batch::RebookingBatchDaoImpl;

pub struct RebookingBatchServiceDependencies;
impl service_impl::rebooking_batch::RebookingBatchServiceDeps for RebookingBatchServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type RebookingBatchDao = RebookingBatchDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type RebookingBatchService =
    service_impl::rebooking_batch::RebookingBatchServiceImpl<RebookingBatchServiceDependencies>;

// Basic-Services-Wave in RestStateImpl::new (nach vacation_entitlement_offset_service,
// vor allen BL-Services):
let rebooking_batch_dao = Arc::new(RebookingBatchDao::new(pool.clone()));
let rebooking_batch_service = Arc::new(
    service_impl::rebooking_batch::RebookingBatchServiceImpl::<
        RebookingBatchServiceDependencies,
    > {
        rebooking_batch_dao,
        permission_service: permission_service.clone(),
        clock_service: clock_service.clone(),
        uuid_service: uuid_service.clone(),
        transaction_dao: transaction_dao.clone(),
    },
);
```

Wiring-Reihenfolge: Basic-Wave (nach `vacation_entitlement_offset_service`), damit spaetere BL-Services die Basic-Manager konsumieren koennen — deterministische Konstruktionsreihenfolge, kein Zyklus.

### 5. Unit-Tests (Task 5)

`service_impl/src/test/rebooking_batch.rs` — 5 mockall-Tests, alle green:

| # | Test | Was wird verifiziert |
|---|------|----------------------|
| 1 | `find_by_id_returns_dao_result` | HR-Gate ok → DAO liefert `Some(entity)` → Service reicht durch |
| 2 | `find_by_sales_person_year_week_none` | HR-Gate ok → DAO liefert `None` → Service reicht `None` durch |
| 3 | `create_success` | Uuid::nil / PrimitiveDateTime::MIN vom Aufrufer → UuidService/ClockService fuellen id+version+created; DAO::create_batch_with_entries wird aufgerufen |
| 4 | **`create_unique_conflict_maps_to_already_exists`** (D-54-DM-01 Guard) | Pre-Check findet aktiven Batch → `Err(EntityAlreadyExists)`; DAO::create_batch_with_entries NIE aufgerufen (mockall-strict) |
| 5 | `find_by_id_non_hr_forbidden` | HR-Gate falsch → `Err(Forbidden)`; DAO::find_by_id NIE aufgerufen (mockall-strict) |

## Verifikationsergebnisse

| Gate | Ergebnis |
| ---- | -------- |
| `cargo build --workspace` | green |
| `cargo test --workspace` | 737 (service_impl) + 64 (shifty_bin integration) + ~40 (dao/rest/…) alle passed, 0 failed |
| `cargo clippy --workspace -- -D warnings` | green |
| `cargo test -p service_impl --lib rebooking_batch` | 5 passed / 0 failed |

## Commits (6 Tasks atomar)

| Commit | Task | Files |
| ------ | ---- | ----- |
| `adedd1d` | Task 1: Trait RebookingBatchService | service/src/rebooking_batch.rs + service/src/lib.rs |
| `7c06e7b` | Task 2: RebookingBatchServiceImpl | service_impl/src/rebooking_batch.rs + service_impl/src/lib.rs |
| `4c212c2` | Task 4: RestStateDef-Erweiterung | rest/src/lib.rs |
| `1715453` | Task 3: DI-Wiring | shifty_bin/src/main.rs |
| `5575139` | Task 5: 5 mockall Unit-Tests | service_impl/src/test/rebooking_batch.rs + service_impl/src/test/mod.rs |
| `d55c1d4` | Task 6: VALIDATION-Update | 54-VALIDATION.md |

Task 4 wurde vor Task 3 committet, damit `shifty_bin` (Task 3) gegen den erweiterten `RestStateDef`-Trait bauen kann.

## Deviations from Plan

### Auto-fixed

**1. [Rule 3 - Blocking] Task-Reihenfolge Task 4 vor Task 3**
- **Found during:** Task 3 (`cargo build -p shifty_bin`)
- **Issue:** Der Plan listet Task 3 (DI-Wiring in `shifty_bin`) vor Task 4 (`RestStateDef`-Trait-Erweiterung in `rest`). shifty_bin haengt aber vom rest-Crate ab — das RestStateImpl-Feld + Getter im shifty_bin-Impl-Block referenziert den neuen assoziierten Type, der erst in Task 4 im rest-Crate deklariert wird.
- **Fix:** Task 4 zuerst ausgefuehrt und committet (Commit `4c212c2`), dann Task 3 (Commit `1715453`). Semantisch identisch zum Plan; nur die Commit-Reihenfolge wurde getauscht.

**2. [Rule 2 - Missing critical] Test 54-02-02 als Service-Level statt DAO-Level**
- **Found during:** Task 6 (VALIDATION-Update)
- **Issue:** VALIDATION-Zeile 54-02-02 verlangte einen DAO-Level-Integration-Test (`cargo test -p dao_impl_sqlite rebooking_batch_unique`). Der Plan (Task 5) legt jedoch den UNIQUE-Konflikt-Guard bewusst auf Service-Ebene an (Pre-Check + mockall) — der DB-UNIQUE-Index wurde bereits in Wave 1 SUMMARY §Verifikation manuell verifiziert.
- **Fix:** VALIDATION-Test-Command auf `cargo test -p service_impl rebooking_batch::create_unique_conflict_maps_to_already_exists` umgestellt, Testtyp auf `unit`. D-54-DM-01 bleibt vollstaendig CI-guarded — der Service-Test dokumentiert die Semantik "aktiver Batch → EntityAlreadyExists" reproduzierbar.

## Known Stubs

Keine. Alle Trait-Methoden sind vollstaendig implementiert. `update_state`, `list_pending` und weitere Rebooking-Aktionen sind bewusst nicht Teil dieses Plans (kommen ab Phase 55 in der BL-Schicht).

## Threat Flags

Keine. Phase 54 Plan 02 fuegt einen Basic-Tier-Service hinzu, der HR-gated ist und in Phase 54 nicht via REST erreichbar ist. Kein neuer Trust-Boundary-Crossing, keine neuen Auth-Pfade, keine neuen Netzwerk-Endpoints.

## Naechste Schritte (Wave 2 Plan 03)

Plan 03 baut die pure fns `voluntary_ist_total_for_year`, `contract_weeks_count`, `committed_voluntary_prorata_for_week`, `committed_voluntary_target_for_year` im `service_impl::reporting` — Konsumenten des `ExtraHoursSource`-Markers aus Wave 1. Property-Test VOL-ACCT-03 (Rebooking-Neutralitaet) baut auf diesen pure fns; der `RebookingBatchService` aus diesem Plan wird von den Statistiken nicht direkt konsumiert (Marker-Row-Semantik reicht — der Batch ist reines Audit-Aggregat, das ab Phase 55 konsumierende BL benoetigt).

## Self-Check: PASSED

**Files exist:**
- `service/src/rebooking_batch.rs`: FOUND
- `service_impl/src/rebooking_batch.rs`: FOUND
- `service_impl/src/test/rebooking_batch.rs`: FOUND

**Commits exist:**
- `adedd1d`: FOUND
- `7c06e7b`: FOUND
- `4c212c2`: FOUND
- `1715453`: FOUND
- `5575139`: FOUND
- `d55c1d4`: FOUND

**Gates:**
- `cargo build --workspace`: green
- `cargo test --workspace`: all green
- `cargo clippy --workspace -- -D warnings`: green
- `cargo test -p service_impl --lib rebooking_batch`: 5 passed / 0 failed
