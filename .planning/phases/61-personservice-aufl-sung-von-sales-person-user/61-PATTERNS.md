# Phase 61: PersonService + Auflösung von `sales_person_user` - Pattern Map

**Mapped:** 2026-09-17
**Files analysiert:** 15 (7 neu, 8 geändert)
**Analogs gefunden:** 15 / 15

## File Classification

| Neue/geänderte Datei | Rolle | Data Flow | Nächster Analog | Match-Qualität |
|---|---|---|---|---|
| `dao/src/person.rs` | model + DAO-Trait | CRUD | `dao/src/rebooking_batch.rs` | exakt (Basic-Tier-DAO-Trait, gleiche Deps-Form) |
| `dao_impl_sqlite/src/person.rs` | DAO-Impl | CRUD | `dao_impl_sqlite/src/rebooking_batch.rs` | exakt |
| `service/src/person.rs` | service-Trait | CRUD, request-response | `service/src/rebooking_batch.rs` | exakt (identischer Dependency-Schnitt: DAO+Permission+Clock+Uuid+Transaction) |
| `service_impl/src/person.rs` | service-Impl (Basic-Tier) | CRUD, request-response | `service_impl/src/rebooking_batch.rs` | exakt |
| `migrations/sqlite/2026091700000X_drop-sales-person-user.sql` | migration | batch (Nachzieh-Backfill + Assertion + DROP) | `migrations/sqlite/20260916000000_create-person-backfill.sql` | exakt (wörtliches Vorbild, D-60-07-Muster) |
| `service_impl/src/test/person_self_gate.rs` | test | request-response (Integration) | `service_impl/src/test/person_backfill.rs` (Harness) + `service_impl/src/rebooking_batch.rs` (Query-Vergleichslogik) | role-match |
| `service_impl/src/test/person_merge.rs` | test | CRUD/transform (Integration) | `service_impl/src/test/person_backfill.rs` (Harness) + `dao_impl_sqlite/src/rebooking_batch.rs` (Claim/Conflict-Pattern) | role-match |
| `service_impl/src/test/person_drop.rs` | test | batch (Migrations-Integrationstest) | `service_impl/src/test/person_backfill.rs::dangling_link_aborts_backfill` | exakt |
| `dao_impl_sqlite/src/sales_person.rs` (6 Methoden) | DAO-Impl | CRUD, request-response (Self-Gate) | sich selbst — Vorher-Zustand ist der Analog für die Umformung; SQL-JOIN-Umformung nach Muster `person_backfill.rs`-Migration | n/a (Rewrite in-place) |
| `dao_impl_sqlite/src/lib.rs` (`create_user`) | DAO-Impl | CRUD (Transaktions-Asymmetrie) | `dao_impl_sqlite/src/rebooking_batch.rs::create_batch_with_entries` (Multi-Insert in einer `tx`) — hier zusätzlich: `self.pool.begin()` statt injizierter `tx`, da Trait-Signatur ohne Transaction-Parameter | role-match |
| `dao_impl_sqlite/src/lib.rs::BasicDaoImpl::clear_all` | DAO-Impl (Test-Utility) | batch | sich selbst (bestehende `DELETE FROM ...`-Liste) | exakt (nur Zeile ergänzen/streichen) |
| `shifty_bin/src/main.rs` (DI) | config/wiring | request-response | `RebookingBatchService`-DI-Block (`:1006`, `:1190-1204`) | exakt |
| `docs/architecture/02-service-tiers.md` + `_de.md` | doc | — | bestehender `RebookingBatchService`-Eintrag | exakt |
| `docs/architecture/diagrams/service-graph-runtime.mmd` | doc/diagram | — | bestehender Diagramm-Knoten für `RebookingBatchService` | role-match |
| `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md` | doc | — | bestehende Self-Gate-Beschreibung (Vorher-Zustand über `sales_person_user`) | role-match |
| `docs/architecture/03-data-model.md` + `docs/architecture/diagrams/db-schema-er.mmd` | doc | — | Phase-60-Docs-Update für `person`-Tabelle (falls vorhanden) sonst Analogie zum Schema-Abschnitt jeder anderen Tabelle | role-match |

## Pattern Assignments

### `dao/src/person.rs` (model + DAO-Trait, CRUD)

**Analog:** `dao/src/rebooking_batch.rs` (Basic-Tier-DAO, gleicher Dependency-Schnitt: nur `DaoError`, kein Domain-Service)

**Imports & Entity-Definition** (`dao/src/rebooking_batch.rs:1-56`):
```rust
use std::sync::Arc;
use mockall::automock;
use uuid::Uuid;
use crate::DaoError;

#[derive(Clone, Debug, PartialEq)]
pub struct RebookingBatchEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    // ...
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
```
Für `PersonEntity` analog, aber deutlich schmaler (D-60-04: nur Identität + Audit):
```rust
pub struct PersonEntity {
    pub id: Uuid,
    pub created: time::PrimitiveDateTime,
    pub version: Uuid, // = update_version
}
```

**Trait mit `#[automock]`** (`dao/src/rebooking_batch.rs:80-89`):
```rust
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait RebookingBatchDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError>;
    // ...
}
```
`PersonDao` übernimmt exakt diese Form: `find_by_id`, `create`, plus die beiden „unverknüpfte Kandidaten"-Lesepfade aus RESEARCH.md Frage 6 (Outer-Join-Queries, siehe unten). **Keine** Vorratshaltung — Pitfall 5 in RESEARCH.md warnt explizit davor, das Trait für Phase 62/63 vorzubereiten.

---

### `dao_impl_sqlite/src/person.rs` (DAO-Impl, CRUD)

**Analog:** `dao_impl_sqlite/src/rebooking_batch.rs`

**`TryFrom<&XDb> for XEntity` + Uuid-BLOB-Handling** (`dao_impl_sqlite/src/rebooking_batch.rs:23-64`):
```rust
#[derive(Debug)]
struct RebookingBatchDb {
    id: Vec<u8>,
    sales_person_id: Vec<u8>,
    // ...
    update_version: Vec<u8>,
}

impl TryFrom<&RebookingBatchDb> for RebookingBatchEntity {
    type Error = DaoError;
    fn try_from(db: &RebookingBatchDb) -> Result<Self, Self::Error> {
        Ok(RebookingBatchEntity {
            id: Uuid::from_slice(&db.id)?,
            sales_person_id: Uuid::from_slice(&db.sales_person_id)?,
            // ...
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}
```
Schreiben (`.as_bytes().to_vec()` beim INSERT, `dao_impl_sqlite/src/rebooking_batch.rs:240-256`):
```rust
let id_vec = batch.id.as_bytes().to_vec();
let sp_vec = batch.sales_person_id.as_bytes().to_vec();
// ...
let version_vec = batch.version.as_bytes().to_vec();

query!(
    r#"INSERT INTO rebooking_batch
           (id, sales_person_id, iso_year, iso_week, kind, state, created,
            approved, approved_by, deleted, update_process, update_version)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    id_vec, sp_vec, /* ... */ version_vec,
)
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;
```
Für `PersonDaoImpl::create` identisch, nur mit den drei Spalten `id, created, update_process, update_version` (`person` hat kein `deleted`, D-60-04). Empfehlung aus RESEARCH.md Frage 5 (Option a): ID/`created` per SQL generieren (`randomblob(16)`, `strftime(...)`, wörtlich aus der Phase-60-Migration übernommen), **nicht** per injiziertem `UuidService`/`ClockService` — `PersonDaoImpl` hat wie `RebookingBatchDaoImpl` keinen Zugriff darauf, wenn `create` direkt vom Aufrufer mit vorgefertigter Entity aufgerufen wird (Muster: `RebookingBatchServiceImpl::create` generiert `id`/`version`/`created` im *Service*, nicht im DAO — dieselbe Aufteilung für `PersonService::create` empfohlen).

**Lock-Idiom** (`tx.tx.lock().await.as_mut()`) und Fehler-Mapping (`.map_db_error()?`) — siehe überall in obigen Snippets; das ist der durchgängige Zugriffspfad auf `TransactionImpl` im ganzen Repo, exakt zu kopieren.

**„Unverknüpfte Kandidaten"-Lesepfade** (RESEARCH.md Frage 6, wörtlich übernehmbar):
```sql
-- Sales Persons, deren Person keinen Benutzer trägt
SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive,
       sp.deleted, sp.update_version
FROM sales_person sp
LEFT JOIN user u ON u.person_id = sp.person_id
WHERE u.name IS NULL AND sp.deleted IS NULL;

-- Benutzer, deren Person keine Sales Person trägt
SELECT u.name
FROM user u
LEFT JOIN sales_person sp ON sp.person_id = u.person_id
WHERE sp.id IS NULL;
```

---

### `service/src/person.rs` + `service_impl/src/person.rs` (Basic-Tier-Service)

**Analog:** `service/src/rebooking_batch.rs` + `service_impl/src/rebooking_batch.rs` — **identischer Dependency-Schnitt** wie von D-61-Gesperrt gefordert (nur `PersonDao` + `PermissionService` + `ClockService` + `UuidService` + `TransactionDao`).

**`gen_service_impl!`-Block** (`service_impl/src/rebooking_batch.rs:28-36`):
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
Für `PersonServiceImpl` 1:1 übertragbar, nur `RebookingBatchDao` → `PersonDao`.

**Trait mit `#[automock]`** (`service/src/rebooking_batch.rs:29-33`):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingBatchService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;
    // ...
}
```

**Core-Pattern: Permission-Gate zuerst, dann Transaction, dann DAO, dann commit** (`service_impl/src/rebooking_batch.rs:43-58`):
```rust
async fn find_by_id(
    &self,
    id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Option<RebookingBatchEntity>, ServiceError> {
    self.permission_service
        .check_permission(HR_PRIVILEGE, context)
        .await?;

    let tx = self.transaction_dao.use_transaction(tx).await?;
    let result = self.rebooking_batch_dao.find_by_id(id, tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```
**Achtung für `PersonService`:** anders als `RebookingBatchService` (durchgängig HR-gated) braucht `PersonService` je nach Methode ein anderes Gate — Lese-Methoden für die eigene Person könnten Self-Access erlauben (Claude's Discretion). Das Grundskelett (Gate → Transaction → DAO → commit) bleibt identisch.

**`create` mit defensiver ID/Version/Timestamp-Generierung** (`service_impl/src/rebooking_batch.rs:92-152`, gekürzt):
```rust
let now = self.clock_service.date_time_now();
let new_batch = RebookingBatchEntity {
    id: if batch.id == Uuid::nil() {
        self.uuid_service.new_uuid(&format!("{PROCESS}::create batch id"))
    } else {
        batch.id
    },
    // ...
    created: if batch.created == time::PrimitiveDateTime::MIN { now } else { batch.created },
    version: if batch.version == Uuid::nil() {
        self.uuid_service.new_uuid(&format!("{PROCESS}::create batch version"))
    } else {
        batch.version
    },
};
```
Direkt übertragbar auf `PersonServiceImpl::create`.

---

### DAO-Methode, die über zwei Tabellen in einer `Transaction` schreibt

**Analog:** `dao_impl_sqlite/src/rebooking_batch.rs::create_batch_with_entries` (`:233-325`)

**Lock-Idiom für mehrere sequenzielle Statements auf derselben `tx`:**
```rust
query!(
    r#"INSERT INTO rebooking_batch (...) VALUES (...)"#,
    /* ... */
)
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;

for entry in entries {
    // weitere Werte vorbereiten...
    query!(
        r#"INSERT INTO rebooking_batch_entry (...) VALUES (...)"#,
        /* ... */
    )
    .execute(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?;
}

Ok(())
```
Für D-61-03 (Merge, `assign_to_user`): analoge Sequenz von `SELECT`/`UPDATE`/`DELETE` auf `tx.tx.lock().await.as_mut()`, jeweils gefolgt von `.map_db_error()?`. Jedes Statement ist ein eigener `.lock()`-Aufruf — **kein** einzelner Lock über mehrere Statements (das Repo hält den Lock nie länger als eine Query).

**`rows_affected`-Guard als Conflict-Erkennung** (`dao_impl_sqlite/src/rebooking_batch.rs::update_state_conditional`, `:350-369`):
```rust
let result = query!(
    r#"UPDATE rebooking_batch
           SET state = ?, approved = ?, approved_by = ?,
               update_process = ?, update_version = ?
           WHERE id = ? AND state = 'pending' AND deleted IS NULL"#,
    state_str, approved_str, approved_by_str, process, version_vec, id_vec,
)
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;

if result.rows_affected() != 1 {
    // Batch bereits entschieden / nicht vorhanden → Conflict (HTTP 409).
    return Err(DaoError::EntityAlreadyExists);
}
Ok(())
```
Für D-61-03/D-61-04: das `UPDATE user SET person_id = ? WHERE name = ? AND person_id = ?`-Statement aus RESEARCH.md Pattern 3 Schritt 3 sollte denselben `rows_affected`-Guard bekommen (Defense-in-Depth gegen Race innerhalb derselben Transaktion, wie RESEARCH.md explizit fordert).

**UNIQUE-Violation als Conflict-Signal** (`dao_impl_sqlite/src/rebooking_batch.rs::create_pending_suggestion`, `:419-445`):
```rust
fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.is_unique_violation())
}
// ...
let insert_result = query!(/* ... */).execute(tx.tx.lock().await.as_mut()).await;

if let Err(err) = insert_result {
    if is_unique_violation(&err) {
        return Err(DaoError::EntityAlreadyExists);
    }
    return Err(DaoError::DatabaseQueryError(Box::new(err)));
}
```
Relevant als Rückversicherung: selbst wenn der Pre-Check (Schritt 2 im Merge-Algorithmus) eine Race übersieht, fängt der UNIQUE-Index `user_person_id_unique_idx`/`sales_person_person_id_unique_idx` sie ab — dieses Muster zeigt, wie man die DB-Fehlermeldung in einen `DaoError` übersetzt.

---

### Service-Methode, die Konflikt erkennt und unterscheidbaren Fehler zurückgibt

**Analog:** `service_impl/src/rebooking_batch.rs::create` (`:92-117`) — Pre-Check innerhalb derselben Transaktion, dann `ServiceError::EntityAlreadyExists`:
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;

// D-54-DM-01: Pre-Check innerhalb derselben Transaktion.
if let Some(existing) = self
    .rebooking_batch_dao
    .find_by_sales_person_year_week(
        batch.sales_person_id, batch.iso_year, batch.iso_week, tx.clone(),
    )
    .await?
{
    return Err(ServiceError::EntityAlreadyExists(existing.id));
}
```
Und der `DaoError → ServiceError`-Übersetzungs-Match (`service_impl/src/rebooking_batch.rs::create_pending_suggestion`, `:247-253`):
```rust
let entity = match result {
    Ok(entity) => entity,
    Err(DaoError::EntityAlreadyExists) => {
        return Err(ServiceError::EntityAlreadyExists(id));
    }
    Err(other) => return Err(other.into()),
};
```

**Zentrales 409-Mapping** (`rest/src/lib.rs:175-198`, kein Handler-lokales Mapping):
```rust
Err(RestError::ServiceError(service::ServiceError::EntityAlreadyExists(id))) => {
    Response::builder().status(409).body(Body::new(id.to_string())).unwrap()
}
Err(RestError::ServiceError(err @ service::ServiceError::EntityConflicts(_, _, _))) => {
    Response::builder().status(409).body(Body::new(err.to_string())).unwrap()
}
```
`ServiceError::EntityAlreadyExists(Uuid)` ist bereits 409-gemappt und bereits unterscheidbar von `EntityConflicts` (`service/src/lib.rs:85` vs. `:94`) — RESEARCH.md empfiehlt explizit, diese Variante für D-61-04 wiederzuverwenden statt eine neue einzuführen (Claude's Discretion, aber mit klarer Empfehlung).

---

### Migration, die sich selbst prüft und abbricht

**Analog:** `migrations/sqlite/20260916000000_create-person-backfill.sql` — Block 4, D-60-07-Muster, **wörtlich verwendbar als Vorlage**:
```sql
CREATE TEMP TABLE IF NOT EXISTS __phase60_backfill_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);

INSERT INTO __phase60_backfill_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM user WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM person)
       = (SELECT COUNT(*) FROM sales_person)
       + (SELECT COUNT(*) FROM user)
       - (SELECT COUNT(*) FROM sales_person_user)
    THEN 1 ELSE 0
END;

DROP TABLE __phase60_backfill_assertion;
```
Prinzip: `CREATE TEMP TABLE` mit `CHECK(ok = 1)` + `INSERT` einer `CASE`-Bedingung — schlägt die Bedingung fehl, verletzt der `INSERT` den `CHECK`-Constraint, SQLx rollt die gesamte Migrations-Transaktion zurück. `TEMP` verhindert, dass bei einem Fehler im nachfolgenden `DROP` etwas in der persistenten Datei zurückbleibt.

RESEARCH.md liefert bereits den vollständigen Textentwurf für die DROP-Migration (Block A Nachzieh-Backfill, Block B Selbst-Assertion nach obigem Muster, Block C `DROP TABLE IF EXISTS sales_person_user;`) — direkt in RESEARCH.md §„Frage 4" nachzulesen, nicht hier dupliziert.

**Wichtig:** `DROP TABLE IF EXISTS` ist (anders als Phase 60s `ALTER TABLE ADD COLUMN`) von sich aus mehrfach ausführbar — **kein** Marker-Split-Kunstgriff für die DROP-Anweisung selbst nötig. Der Nachzieh-Backfill-Abschnitt (Block A) kann aber denselben Marker-Split-Test-Mechanismus wie Phase 60 bekommen, falls Idempotenz separat geprüft werden soll (laut D-61-06 aber **nicht** verlangt — nur ein Test, der den Zwischenzustand herstellt und den Nachzug nachweist).

---

### Test-Harness (Phase-60-Vorbild für die drei neuen Testmodule)

**Analog:** `service_impl/src/test/person_backfill.rs`

**`setup_pool`-Helfer** (`:97-108`):
```rust
async fn setup_pool() -> Arc<sqlx::SqlitePool> {
    let pool = Arc::new(
        sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Could not connect to in-memory SQLite"),
    );
    sqlx::migrate!("./../migrations/sqlite")
        .run(pool.as_ref())
        .await
        .expect("Could not run migrations");
    pool
}
```

**Rohe SQL-Fixture-Funktionen** (`:151-188`):
```rust
async fn seed_sales_person(pool: &sqlx::SqlitePool, id: Uuid, name: &str, deleted: Option<&str>) {
    sqlx::query(
        "INSERT INTO sales_person (id, name, inactive, deleted, update_process, update_version) \
         VALUES (?1, ?2, 0, ?3, 'seed', ?4)",
    )
    .bind(id.as_bytes().to_vec())
    .bind(name)
    .bind(deleted)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .execute(pool)
    .await
    .expect("seed sales_person");
}

async fn seed_user(pool: &sqlx::SqlitePool, name: &str) {
    sqlx::query("INSERT INTO user (name, update_process) VALUES (?1, 'seed')")
        .bind(name)
        .execute(pool)
        .await
        .expect("seed user");
}

async fn seed_link(pool: &sqlx::SqlitePool, sales_person_id: Uuid, user_name: &str) {
    sqlx::query(
        "INSERT INTO sales_person_user (sales_person_id, user_id, update_process) \
         VALUES (?1, ?2, 'seed')",
    )
    .bind(sales_person_id.as_bytes().to_vec())
    .bind(user_name)
    .execute(pool)
    .await
    .expect("seed sales_person_user");
}
```

**`count()`-Helfer** (`:300-306`):
```rust
async fn count(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query(sql)
        .fetch_one(pool)
        .await
        .expect("count query")
        .get::<i64, _>(0)
}
```

**`stored_ddl()`-Helfer** (`:986-996`, liest die tatsächlich gespeicherte DDL statt Quelltextsuche):
```rust
async fn stored_ddl(pool: &sqlx::SqlitePool, kind: &str, name: &str) -> String {
    sqlx::query("SELECT sql FROM sqlite_master WHERE type = ?1 AND name = ?2")
        .bind(kind)
        .bind(name)
        .fetch_optional(pool)
        .await
        .expect("read sqlite_master")
        .unwrap_or_else(|| panic!("{kind} '{name}' existiert nach dem Migrationslauf nicht"))
        .get::<Option<String>, _>("sql")
        .unwrap_or_else(|| panic!("{kind} '{name}' hat keine gespeicherte Definition"))
}
```

**Für `person_self_gate.rs`:** RESEARCH.md §„Frage 3" beschreibt exakt den Test-Mechanismus (Side-by-Side-Vergleich Alt-Query vs. Neu-Query gegen eine gemeinsame Fixture-DB) — dieser Text ist die primäre Spezifikation, `person_backfill.rs` liefert nur die Low-Level-Bausteine (Fixtures, `setup_pool`).

**Für `person_drop.rs`:** Vorbild ist konkret `person_backfill.rs::dangling_link_aborts_backfill` (Fixture mit Zwischenzustand → Assertion greift/bricht ab) — D-61-06 verlangt explizit keine größere Matrix als dieses eine Testmuster.

**Für `person_merge.rs`:** kein 1:1-Vorbild in `person_backfill.rs` (reine Backfill-Tests, keine Service-Aufrufe); struktureller Vorbild ist stattdessen das DAO-Test-Muster aus `rebooking_batch`-Tests (Conflict via `rows_affected`/UNIQUE, siehe oben) kombiniert mit der `setup_pool`/Fixture-Infrastruktur aus `person_backfill.rs`.

---

### `shifty_bin/src/main.rs` DI — Insertionspunkt

**Analog-Block:** `RebookingBatchDao`-Konstruktion im DAO-Block (`:1006`) und `RebookingBatchService`-Konstruktion im Basic-Service-Block (`:1190-1204`):
```rust
// DAO-Block (~Zeile 1000-1020):
let rebooking_batch_dao = Arc::new(RebookingBatchDao::new(pool.clone()));
// ... weitere DAOs ...

// Basic-Service-Block (~Zeile 1190):
// Phase 54 (D-54-DM-01): Basic-Tier Rebooking-Batch-Service — nur
// DAO + Permission + Clock + Uuid + Transaction. Kein Domain-Service
// als Dep; konsumiert wird der Service ab Phase 55 vom BL-
// RebookingReconciliationService.
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
`person_dao` gehört in den DAO-Block (analog Zeile 1006, direkt neben `rebooking_batch_dao`), `person_service` in den Basic-Service-Block — **vor** jedem Konsumenten, aber nach `permission_service`/`clock_service`/`uuid_service`/`transaction_dao` (diese existieren bereits früher im Konstruktionsgraphen). Type-Alias analog `RebookingBatchDao`/`RebookingBatchServiceDependencies` (`:67`, `:327`) anlegen. Kommentar-Konvention: Phase-Nummer + Decision-ID im Kommentar direkt über der Konstruktion (siehe Beispiel oben — hier `Phase 61 (D-61-...)`).

---

### Doc-Paar EN/DE (Scope-Referenz für die Docs-Aufgabe)

**Analog:** `docs/architecture/02-service-tiers.md:16-23` (EN) vs. `docs/architecture/02-service-tiers_de.md:16-24` (DE) — identische Struktur, Bullet-Liste unter „Basic Services (entity managers)" / „Basic Services (Entity-Manager)":

EN (`02-service-tiers.md:18-23`):
```markdown
- `RebookingBatchService` — entity manager for `rebooking_batch` +
  `rebooking_batch_entry` (v2.6 Phase 54). HR-gated CRUD only; the
  first consumer arrives in Phase 55 (`RebookingReconciliationService`).
  Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao`. See feature
  [F14](../features/F14-rebooking.md).
```

DE (`02-service-tiers_de.md:18-24`):
```markdown
- `RebookingBatchService` — Entity-Manager für `rebooking_batch` +
  `rebooking_batch_entry` (v2.6 Phase 54). Ausschließlich HR-gated
  CRUD; erster Konsument folgt in Phase 55
  (`RebookingReconciliationService`). Deps: `RebookingBatchDao`,
  `PermissionService`, `ClockService`, `UuidService`,
  `TransactionDao`. Siehe Feature
  [F14](../features/F14-rebooking.md).
```
Für `PersonService` genau ein neuer Bullet-Eintrag pro Sprachdatei, gleiche Form: Rollenbeschreibung, Phase-Referenz, Dep-Liste, Feature-Link (falls ein `F*`-Dokument existiert, sonst weglassen). Umfang der Docs-Aufgabe bleibt damit ehrlich klein: **ein** Bullet in zwei Dateien plus die vier in CONTEXT.md genannten Trigger-Dateien (`04-auth.md`/`F12-auth-session.md` für den umgeschriebenen Self-Gate, `03-data-model.md`/`db-schema-er.mmd` für die verschwindende Tabelle).

---

## Shared Patterns

### Uuid-BLOB-Persistenz
**Quelle:** `dao_impl_sqlite/src/rebooking_batch.rs` (`TryFrom`-Block + INSERT-Vorbereitung)
**Anwenden auf:** `dao_impl_sqlite/src/person.rs`, die umgeschriebenen sechs Methoden in `dao_impl_sqlite/src/sales_person.rs`
```rust
// Lesen:
id: Uuid::from_slice(&db.id)?,
// Schreiben:
let id_vec = entity.id.as_bytes().to_vec();
```

### Transaction-Lock-Idiom
**Quelle:** durchgängig in `dao_impl_sqlite/src/*.rs`
**Anwenden auf:** alle neuen/geänderten DAO-Methoden dieser Phase
```rust
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;
```

### Permission-Gate-zuerst
**Quelle:** `service_impl/src/rebooking_batch.rs` (jede Methode)
**Anwenden auf:** `service_impl/src/person.rs`
```rust
self.permission_service
    .check_permission(HR_PRIVILEGE, context)
    .await?;
let tx = self.transaction_dao.use_transaction(tx).await?;
// ...
self.transaction_dao.commit(tx).await?;
```

### Conflict-Fehler → 409
**Quelle:** `service/src/lib.rs:85` (`EntityAlreadyExists(Uuid)`), `rest/src/lib.rs:175-179`, Präzedenz `service_impl/src/rebooking_batch.rs::create`/`create_pending_suggestion`
**Anwenden auf:** D-61-04 Merge-Konflikt in `SalesPersonDaoImpl::assign_to_user`/`SalesPersonServiceImpl::set_user`
```rust
Err(DaoError::EntityAlreadyExists) => {
    return Err(ServiceError::EntityAlreadyExists(id));
}
```

### Migrations-Selbst-Assertion (Abort-on-Violation)
**Quelle:** `migrations/sqlite/20260916000000_create-person-backfill.sql` Block 4
**Anwenden auf:** die neue DROP-Migration (Block B laut RESEARCH.md Frage 4)
```sql
CREATE TEMP TABLE IF NOT EXISTS __phaseNN_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);
INSERT INTO __phaseNN_assertion (ok) SELECT CASE WHEN <bedingung> THEN 1 ELSE 0 END;
DROP TABLE __phaseNN_assertion;
```

## No Analog Found

Keine — alle 15 Dateien/Änderungsstellen haben einen konkreten Analog im Repo (Basic-Tier-Quartett `RebookingBatchService`, Migrations-Muster `20260916000000`, Test-Harness `person_backfill.rs`, DI-Muster `main.rs`, Doc-Paar `02-service-tiers*.md`). Der einzige Baustein ohne 1:1-Repo-Präzedenz ist der Merge-/Split-Algorithmus selbst (D-61-03/D-61-05) — das ist laut RESEARCH.md bewusste neue Synthese, aber jede Einzeltechnik darin (Conflict-Check via `rows_affected`, UNIQUE-Violation-Mapping, Multi-Statement-Transaktion) hat eine wörtliche Präzedenz in `rebooking_batch.rs` (siehe oben).

## Metadata

**Analog-Suchbereich:** `dao/src/`, `dao_impl_sqlite/src/`, `service/src/`, `service_impl/src/`, `service_impl/src/test/`, `migrations/sqlite/`, `shifty_bin/src/main.rs`, `docs/architecture/`, `rest/src/lib.rs`
**Dateien gescannt:** ~20 (gezielt via Grep/Read, keine Vollindizierung)
**Pattern-Extraktion:** 2026-09-17
