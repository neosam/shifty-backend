# Phase 62: Personen-Verzeichnis — eine Liste statt Tabs - Pattern Map

**Mapped:** 2026-09-19
**Files analyzed:** 13 (neu + geändert)
**Analogs found:** 12 / 13

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|--------------------|------|-----------|-----------------|----------------|
| `dao/src/person_directory.rs` (neu, empfohlen statt Erweiterung von `PersonDao`) | model/trait (DAO) | CRUD (read) | `dao/src/person.rs` | exact (gleiches Modul-Paar, gleicher Outer-Join-Stil) |
| `dao_impl_sqlite/src/person_directory.rs` (neu) | DAO-Impl | CRUD (read, outer join) | `dao_impl_sqlite/src/person.rs` (`sales_persons_without_user`/`users_without_sales_person`) | exact |
| `service/src/person_directory.rs` (neu) | service (trait) | request-response / CRUD (aggregate read) | `service/src/voluntary_stats.rs` | exact (Business-Logic-Tier, Context/Transaction generisch, `#[automock]`) |
| `service_impl/src/person_directory.rs` (neu) | service (impl) | request-response / CRUD | `service_impl/src/voluntary_stats.rs` | exact (`gen_service_impl!`, Gate zuerst, mehrere Basic-Deps) |
| `rest-types/src/lib.rs` — `PersonEntryTO` (+ `PersonKindTO`) (neu, Anhang) | DTO | transform | `rest-types/src/lib.rs:1058-1078` `SalesPersonAbsenceTO` | exact (kleines, zweckgebautes Listen-DTO + `From`-Impl) |
| `rest/src/person_directory.rs` (neu) | route/controller | request-response | `rest/src/permission.rs` (`get_all_users`, Zeile 19, 305-333) | exact (admin-gated GET-Liste, `error_handler`, `#[utoipa::path]`) |
| `rest/src/lib.rs` (geändert — `mod`, `.nest(...)`, `ApiDoc`) | route registration | request-response | `rest/src/lib.rs:23,612-635` (bestehende `mod`/`ApiDoc`-Liste) | exact |
| `shifty_bin/src/main.rs` (geändert — DI) | config/DI wiring | request-response | `shifty_bin/src/main.rs:352-378` (`VoluntaryStatsServiceDependencies`) | exact |
| `service_impl/src/test/person_directory.rs` (neu) | test | CRUD (unit) | `service_impl/src/test/` Muster generell; Gate-Test analog `voluntary_stats`-Impl-Struktur | role-match |
| `service_impl/src/test/person_candidates.rs` (geändert, WR-02) | test | CRUD (unit) | bestehende Datei selbst (Erweiterung) | exact |
| `shifty_bin/src/integration_test/person_directory.rs` (neu) | test (integration, `tower::oneshot`) | request-response | `shifty_bin/src/integration_test/voluntary_stats.rs` | exact |
| `shifty-dioxus/src/page/user_management.rs` (geändert) | component/page | request-response (UI) | Datei selbst (Tab-Entfernung) + Guard-Analog `shifty-dioxus/src/page/settings.rs:549-570` | exact (Guard), self (Umbau) |
| `shifty-dioxus/src/service/user_management.rs` (geändert) | store/service (Dioxus coroutine) | request-response (single aggregate load, ersetzt N+1) | Datei selbst (`load_all_sales_person_user_links` etc. werden abgelöst); Lade-Stil analog `loader::load_sales_persons` (`shifty-dioxus/src/loader.rs:31`) | role-match |
| `shifty-dioxus/Dioxus.toml` (geändert) | config | — | bestehende `[[web.proxy]]`-Liste (`Dioxus.toml:45-64`) | exact |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` (geändert) | i18n config | — | bestehende Key-Definitionen (`ColumnType`, `ColumnLinkedUser`, `ColumnRoles`, `Unlinked`, `SearchPlaceholder`, `Paid`, `Volunteer`, `Inactive`) | exact |

## Pattern Assignments

### `dao/src/person_directory.rs` (DAO trait, CRUD read)

**Analog:** `dao/src/person.rs` (voll gelesen)

**Modul-Header/Konventions-Kommentar-Stil** (`dao/src/person.rs:1-18`):
```rust
//! Phase 61 (PERS-05 / D-61-02): Basic-Tier Person-DAO.
//! ...
//! Das Trait bleibt bewusst auf genau vier Methoden beschraenkt. Vorratshaltung
//! fuer Phase 62/63 ist ausdruecklich unerwuenscht (Research Pitfall 5).
```
→ Genau dieser Kommentar ist der Grund, warum die neue Methode NICHT auf
`PersonDao` landen soll — ein separates, schlankes Trait ist die sicherere Lesart.

**Trait-Form** (`dao/src/person.rs:36-70`):
```rust
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait PersonDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(&self, id: Uuid, tx: Self::Transaction) -> Result<Option<PersonEntity>, DaoError>;
    async fn sales_persons_without_user(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError>;
}
```
Kopiervorlage für `PersonDirectoryDao`:
```rust
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait PersonDirectoryDao {
    type Transaction: crate::Transaction;

    async fn list_directory_entries(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[PersonDirectoryRow]>, DaoError>;
}
```

**Entity-Struct-Stil** (`dao/src/person.rs:28-34`): flaches Struct, keine `deleted`-Spalte auf `PersonEntity` selbst — die neue `PersonDirectoryRow` sollte analog flach sein mit `Option<...>`-Feldern für die optionale `sales_person`/`user`-Seite.

---

### `dao_impl_sqlite/src/person_directory.rs` (DAO-Impl, outer join)

**Analog:** `dao_impl_sqlite/src/person.rs` (voll gelesen, 145-189)

**Row-Layout + `TryFrom`-Konvention** (`dao_impl_sqlite/src/person.rs:31-49`):
```rust
#[derive(Debug)]
struct PersonDb {
    id: Vec<u8>,
    created: String,
    update_version: Vec<u8>,
}

impl TryFrom<&PersonDb> for PersonEntity {
    type Error = DaoError;
    fn try_from(db: &PersonDb) -> Result<Self, Self::Error> {
        Ok(PersonEntity {
            id: Uuid::from_slice(&db.id)?,
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}
```

**Existierender Kandidaten-Join, WÖRTLICHE Vorlage für den Aggregat-Join** (`dao_impl_sqlite/src/person.rs:152-189`):
```rust
async fn sales_persons_without_user(
    &self,
    tx: Self::Transaction,
) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
    Ok(query_as!(
        CandidateSalesPersonDb,
        r#"SELECT sp.id AS "id!: Vec<u8>", sp.name AS "name!: String",
                  sp.background_color AS "background_color!: String",
                  sp.is_paid AS "is_paid!: bool", sp.inactive AS "inactive!: bool",
                  sp.deleted AS "deleted?: String",
                  sp.update_version AS "update_version!: Vec<u8>"
           FROM sales_person sp
           LEFT JOIN user u ON u.person_id = sp.person_id
           WHERE u.name IS NULL AND sp.deleted IS NULL"#,
    )
    .fetch_all(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?
    .iter()
    .map(SalesPersonEntity::try_from)
    .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?)
}

async fn users_without_sales_person(
    &self,
    tx: Self::Transaction,
) -> Result<Arc<[Arc<str>]>, DaoError> {
    Ok(query!(
        r#"SELECT u.name AS "name!: String"
           FROM user u
           LEFT JOIN sales_person sp ON sp.person_id = u.person_id
           WHERE sp.id IS NULL"#,           // <-- WR-02-Bug: fehlender sp.deleted-Filter
    )
    .fetch_all(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?
    .iter()
    .map(|row| Arc::<str>::from(row.name.as_str()))
    .collect::<Arc<[Arc<str>]>>())
}
```

**WR-02-Fix, wörtlich zu übernehmen für `users_without_sales_person`** (aus RESEARCH.md, `61-REVIEW.md:330-337`):
```sql
SELECT u.name AS "name!: String"
FROM user u
LEFT JOIN sales_person sp
       ON sp.person_id = u.person_id
      AND sp.deleted IS NULL
WHERE sp.id IS NULL
```
Filter wandert aus der `WHERE`- in die `ON`-Klausel — sonst degeneriert der `LEFT JOIN`.

**Neue Aggregat-Query-Form** (aus RESEARCH.md Code Examples, strukturell konsistent mit obigem Muster):
```sql
SELECT
    p.id           AS "person_id!: Vec<u8>",
    sp.id          AS "sales_person_id?: Vec<u8>",
    sp.name        AS "sales_person_name?: String",
    sp.background_color AS "background_color?: String",
    sp.is_paid     AS "is_paid?: bool",
    sp.inactive    AS "inactive?: bool",
    u.name         AS "username?: String"
FROM person p
LEFT JOIN sales_person sp ON sp.person_id = p.id AND sp.deleted IS NULL
LEFT JOIN user u ON u.person_id = p.id
WHERE sp.id IS NOT NULL OR u.name IS NOT NULL  -- Ghost-Person ausschliessen (Pitfall 5)
```

**Error handling:** `.map_db_error()` nach jedem `.fetch_...()` — Projekt-Konvention, siehe `use crate::ResultDbErrorExt` (`dao_impl_sqlite/src/person.rs:24`).

---

### `service/src/person_directory.rs` (Business-Logic-Trait)

**Analog:** `service/src/voluntary_stats.rs` (voll gelesen, 1-97)

**Trait-Form + `#[automock]`** (`service/src/voluntary_stats.rs:78-97`):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait VoluntaryStatsService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_voluntary_stats(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError>;
}
```
Kopiervorlage für `PersonDirectoryService::list`:
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait PersonDirectoryService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn list(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[PersonEntry]>, ServiceError>;
}
```

**Domain-Struct-Stil** (`service/src/voluntary_stats.rs:56-76`) — dokumentiertes `#[derive(Clone, Debug, PartialEq)]`-Struct mit Doc-Kommentaren pro Feld; für `PersonEntry` analog mit `kind: PersonKind`-Enum-Feld statt `Option`-Zerfaserung (D-62-01).

**Wichtig — Gate-Unterschied zu diesem Analog:** `VoluntaryStatsService` liefert bei Non-HR ein `None`-gefülltes Ergebnis (API-Level-Redaktion, KEIN 403). Für `PersonDirectoryService` gilt das NICHT — D-62-08 verlangt echtes `check_permission("admin", ...)` mit Fehler-Propagation (403), analog `PermissionServiceImpl::get_all_users`, nicht analog `VoluntaryStatsServiceImpl::get_voluntary_stats`. Nur die Tier-/DI-Form ist die Analogie, nicht die Gate-Semantik.

---

### `service_impl/src/person_directory.rs` (Business-Logic-Impl)

**Analog:** `service_impl/src/voluntary_stats.rs` (Zeilen 1-80 gelesen)

**`gen_service_impl!`-Form mit mehreren Basic-Deps** (`service_impl/src/voluntary_stats.rs:41-56`):
```rust
gen_service_impl! {
    struct VoluntaryStatsServiceImpl: VoluntaryStatsService = VoluntaryStatsServiceDeps {
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        AbsenceService: AbsenceService<Transaction = Self::Transaction, Context = Self::Context> = absence_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```
Kopiervorlage (minimalere Deps — nur DAO + Permission + Transaction, da der neue Service den Join direkt über die neue DAO-Methode bezieht, keine anderen Domain-Services braucht laut RESEARCH):
```rust
gen_service_impl! {
    struct PersonDirectoryServiceImpl: PersonDirectoryService = PersonDirectoryServiceDeps {
        PersonDirectoryDao: PersonDirectoryDao<Transaction = Self::Transaction> = person_directory_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Gate-ZUERST-Muster (Admin, mit 403-Propagation statt Redaktion)** — Vorbild `service_impl/src/permission.rs` (`get_all_users`, `create_user`, `delete_user`):
```rust
async fn get_all_users(
    &self,
    context: Authentication<Self::Context>,
) -> Result<Arc<[service::User]>, service::ServiceError> {
    self.check_permission("admin", context).await?;
    Ok(self.permission_dao.all_users().await?.iter().map(service::User::from).collect())
}
```
→ Für `PersonDirectoryServiceImpl::list`:
```rust
async fn list(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[PersonEntry]>, ServiceError> {
    self.permission_service.check_permission("admin", context).await?;   // D-62-08: admin, NICHT hr
    let tx = self.transaction_dao.use_transaction(tx).await?;
    let rows = self.person_directory_dao.list_directory_entries(tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    Ok(rows.iter().map(PersonEntry::from).collect())
}
```

**Transaction-Pattern** (Standard-Konvention aus `CLAUDE.md`):
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... DAO-Calls ...
self.transaction_dao.commit(tx).await?;
```

---

### `rest-types/src/lib.rs` — `PersonEntryTO` / `PersonKindTO`

**Analog:** `SalesPersonAbsenceTO` (`rest-types/src/lib.rs:1058-1078`, voll gelesen)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SalesPersonAbsenceTO {
    pub sales_person_id: Uuid,
    pub name: Arc<str>,
    pub hours: f32,
}
#[cfg(feature = "service-impl")]
impl From<&SalesPersonAbsence> for SalesPersonAbsenceTO {
    fn from(sales_person_absence: &SalesPersonAbsence) -> Self {
        Self {
            sales_person_id: sales_person_absence.sales_person_id,
            name: sales_person_absence.name.clone(),
            hours: sales_person_absence.hours,
        }
    }
}
```
→ Direkt übertragbares Muster: kleines `ToSchema`-DTO + `#[cfg(feature = "service-impl")] impl From<&Domain> for TO`. `PersonEntryTO` folgt derselben Form; `PersonKindTO` als eigenes `ToSchema`-Enum (Serde-Tag-Form ist Discretion, siehe RESEARCH Code-Beispiel mit `#[serde(tag = "kind", rename_all = "snake_case")]`).

---

### `rest/src/person_directory.rs` (neuer Endpoint)

**Analog:** `rest/src/permission.rs` (`generate_route`, `get_all_users` — Zeilen 1-33, 305-333, voll gelesen)

**Router-Aufbau**:
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/user", get(get_all_users::<RestState>))
        // ...
}
```

**Admin-gated GET-Handler mit `error_handler`** (`rest/src/permission.rs:305-333`):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/user",
    tags = ["Permission"],
    responses(
        (status = 200, description = "List all users", body = [UserTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all_users<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let users: Arc<[UserTO]> = rest_state
                .permission_service()
                .get_all_users(context.into())
                .await?
                .iter()
                .map(UserTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&users).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```
Direkt übertragbar auf `get_person_directory` — `context.into()` reicht den `Authentication`-Wrapper an den Service durch, das Gate lebt im Service, nicht im Handler (D-62-08, keine doppelte Prüfung).

**`RestStateDef`-Erweiterung** — Vorbild `rest/src/lib.rs:516,521`:
```rust
fn voluntary_stats_service(&self) -> Arc<Self::VoluntaryStatsService>;
fn person_service(&self) -> Arc<Self::PersonService>;
```
→ `fn person_directory_service(&self) -> Arc<Self::PersonDirectoryService>;` plus `type PersonDirectoryService: ...;` im Trait, Implementierung in `shifty_bin/src/main.rs` analog Zeile 1005/1020.

---

### `rest/src/lib.rs` (mod + nest + ApiDoc)

**Analog:** bestehende Liste (`rest/src/lib.rs:23, 612-635`)
```rust
mod permission;
// ...
(path = "/permission", api = permission::PermissionApiDoc),
```
→ `mod person_directory;` + `(path = "/person-directory", api = person_directory::PersonDirectoryApiDoc),` (Routen-Präfix ist Discretion, muss aber mit dem `Dioxus.toml`-Eintrag übereinstimmen).

---

### `shifty_bin/src/main.rs` (DI-Wiring)

**Analog:** `VoluntaryStatsServiceDependencies` (Zeilen 352-378, voll gelesen) — repräsentiert exakt das Muster "Business-Logic-Tier nach dem Basic-Block":
```rust
pub struct VoluntaryStatsServiceDependencies;
impl service_impl::voluntary_stats::VoluntaryStatsServiceDeps
    for VoluntaryStatsServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ReportingService = ReportingService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type SalesPersonService = SalesPersonService;
    type AbsenceService = AbsenceService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type VoluntaryStatsService =
    service_impl::voluntary_stats::VoluntaryStatsServiceImpl<VoluntaryStatsServiceDependencies>;
```
Zusätzlich `PersonServiceDependencies` (Zeilen 335-350) als Vorbild für ein reines Basic-Tier-Muster (nur DAO+Permission+Clock+Uuid+Transaction) — falls der neue Service doch minimal bleibt und keine anderen Domain-Services braucht, ist die Konstruktion NACH `PersonService`, aber weiterhin im Business-Logic-Block (weil er DAO-Aggregation über mehrere Aggregate liefert, nicht weil er andere Services konsumiert — Tier-Einordnung ist inhaltlich, nicht nur strukturell, siehe CONTEXT.md Tier-Regel).

**Wichtig:** Konstruktions-Reihenfolge — der neue `PersonDirectoryServiceDependencies`-Block muss NACH dem gesamten Basic-Block stehen (insbesondere nach `PersonServiceDependencies`, Zeile ~335-350), keine `OnceLock`-Tricks.

---

### `shifty_bin/src/integration_test/person_directory.rs` (REST-Integrationstest)

**Analog:** `shifty_bin/src/integration_test/voluntary_stats.rs` (voll gelesen, 1-170)

**Setup + Router + `tower::oneshot`-Muster**:
```rust
use rest::report::generate_route;
use rest::{Context as RestContext, RestStateDef};
use tower::ServiceExt;

let test_setup = TestSetup::new().await;
let router = axum::Router::new()
    .nest("/report", generate_route::<crate::RestStateImpl>())
    .with_state(test_setup.rest_state.clone())
    .layer(Extension(Some(Arc::<str>::from("DEVUSER")) as RestContext));

let req = Request::builder()
    .method("GET")
    .uri(&uri)
    .body(Body::empty())
    .unwrap();

let resp = router.oneshot(req).await.unwrap();
assert_eq!(resp.status(), StatusCode::OK, "HR must receive 200");
```
→ Für Phase 62: `.nest("/person-directory", person_directory::generate_route::<crate::RestStateImpl>())`; zwei Tests: Admin → 200, Nicht-Admin (z.B. HR-Only-User ohne Admin-Rolle) → `StatusCode::FORBIDDEN` (Pitfall 4 aus RESEARCH — HR ≠ Admin).

**Seed-Helper-Stil** (`seed_voluntary_scenario`, Zeilen 41-127): direkte Service-Aufrufe über `test_setup.rest_state.<service>()` mit `Authentication::Full`, keine rohen SQL-Inserts — gleiches Muster für Seed einer verknüpften Person / reinem User / reinem SalesPerson in der neuen Testdatei verwenden.

---

### `shifty-dioxus/src/page/user_management.rs` (Umbau + Guard)

**Analog (Guard):** `shifty-dioxus/src/page/settings.rs:549-570` (voll im Kontext gelesen)
```rust
// WR-02: Component-level admin guard. The nav hides this route for non-admins,
// but a direct URL access would bypass that. Reuse the same AUTH/has_privilege
// pattern used by billing_periods, shiftplan, absences pages.
let is_admin = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("admin"))
    .unwrap_or(false);
if !is_admin {
    return rsx! {
        TopBar {}
        div { class: "p-md text-ink-muted", "Not authorized." }
    };
}
```
→ Wörtlich in `UserManagementPage` einfügen (existiert dort heute nicht, siehe RESEARCH Pitfall/`code_context`).

**Umzubauende Stelle** (`shifty-dioxus/src/page/user_management.rs:1-60`, komplette Datei zusätzlich zu lesen für Tab-Removal): heutiger Tab-State
```rust
let mut active_tab = use_signal(|| UserMgmtTab::SalesPersons);
let sales_persons_search = use_signal(String::new);
let users_search = use_signal(String::new);
// ...
use_effect(move || {
    user_management_service.send(UserManagementAction::LoadAllUsers);
    user_management_service.send(UserManagementAction::LoadAllSalesPersons);
});
```
→ ersetzen durch einen `use_effect`, der eine einzelne neue Action sendet (z.B. `UserManagementAction::LoadPersonDirectory`), ein Such-Signal statt zwei, keine `active_tab`-Signal mehr.

**Wiederverwendbar unverändert:** `matches_search` (`user_management.rs:32-36`) — pure Funktion, auf `display_name` UND `username` anzuwenden (OR-verknüpft).

---

### `shifty-dioxus/src/service/user_management.rs` (Store + Loader)

**Analog:** Datei selbst — bestehende Store-Struktur (`UserManagementStore`, Zeilen 46-79) und die drei N+1-Loader (Zeilen 174-227, alle drei voll gelesen), die abgelöst werden:
```rust
pub async fn load_all_sales_person_user_links() {
    let sales_persons = USER_MANAGEMENT_STORE.read().sales_persons.clone();
    let config = CONFIG.read().clone();
    let fetches = sales_persons.iter().map(|sp| {
        let cfg = config.clone();
        let id = sp.id;
        async move { (id, loader::load_user_for_sales_person(cfg, id).await) }
    });
    let results = join_all(fetches).await;
    let (map, errors) = partition_results(results);
    report_errors(errors);
    USER_MANAGEMENT_STORE.write().sales_person_user_links = Rc::new(map);
}
```
→ Diese drei Funktionen (`load_all_sales_person_user_links`, `load_all_user_sales_person_links`, `load_all_user_roles`) fallen an der Listen-Seite weg (Success Criterion 5 — ein Aggregat-Request). Ersatz: eine einzelne `load_person_directory()`-Funktion nach dem einfachen Einzel-Loader-Stil, siehe:

**Analog (einfacher Single-Fetch-Loader):** `shifty-dioxus/src/loader.rs:31`
```rust
pub async fn load_sales_persons(config: Config) -> Result<Rc<[SalesPerson]>, ShiftyError> { ... }
```
→ Neuer Loader `loader::load_person_directory(config: Config) -> Result<Rc<[PersonEntry]>, ShiftyError>`, aufgerufen aus einer neuen `UserManagementAction::LoadPersonDirectory`-Handler-Funktion im Store-Coroutine-Match, Ergebnis in ein neues `pub directory: Rc<[PersonEntry]>`-Feld auf `UserManagementStore` geschrieben (statt der drei Link-Maps).

---

### `shifty-dioxus/Dioxus.toml` (Proxy-Eintrag)

**Analog:** bestehende `[[web.proxy]]`-Liste (`Dioxus.toml:45-64`, Auszug gelesen)
```toml
[[web.proxy]]
backend = "http://localhost:3000/billing-period"
[[web.proxy]]
backend = "http://localhost:3000/blocks"
```
→ Neuer Eintrag `[[web.proxy]]` / `backend = "http://localhost:3000/person-directory"` (Präfix muss exakt mit dem `rest/src/lib.rs`-`.nest(...)`-Pfad übereinstimmen) — im selben Commit wie der Endpoint (Pitfall 2, dreimal in diesem Repo verpasst).

---

## Shared Patterns

### Admin-Gate (Backend)
**Source:** `service_impl/src/permission.rs:158-170` (`get_all_users`, `create_user`, `delete_user`)
**Apply to:** `service_impl/src/person_directory.rs::list` — `self.permission_service.check_permission("admin", context).await?;` ZUERST, vor jedem DAO-Call. NICHT `HR_PRIVILEGE` (Verwechslungsgefahr mit `service_impl/src/person.rs`, das HR-gated ist — Pitfall 4).
```rust
self.check_permission("admin", context).await?;
```

### Admin-Gate (Frontend, Defense-in-Depth)
**Source:** `shifty-dioxus/src/page/settings.rs:549-570`
**Apply to:** `shifty-dioxus/src/page/user_management.rs::UserManagementPage` (Guard existiert dort heute NICHT — muss neu eingefügt werden, kein Refactoring).
```rust
let is_admin = AUTH.read().auth_info.as_ref().map(|a| a.has_privilege("admin")).unwrap_or(false);
if !is_admin {
    return rsx! { TopBar {} div { class: "p-md text-ink-muted", "Not authorized." } };
}
```

### Transaction-Pattern
**Source:** `CLAUDE.md` §Transaction Management, gespiegelt in jedem `service_impl`-Modul
**Apply to:** `service_impl/src/person_directory.rs`
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... DAO-Calls ...
self.transaction_dao.commit(tx).await?;
```

### Zentrales Error-Mapping (REST)
**Source:** `rest/src/permission.rs` (`error_handler`-Wrapper um jeden Handler-Body)
**Apply to:** `rest/src/person_directory.rs::get_person_directory`
```rust
error_handler((async { /* ... */ Ok(Response::builder().status(200).body(...).unwrap()) }).await)
```

### Soft-Delete in `ON`- statt `WHERE`-Klausel bei LEFT JOIN
**Source:** WR-02-Fix aus `61-REVIEW.md` (siehe RESEARCH.md, Pitfall 1)
**Apply to:** sowohl `dao_impl_sqlite/src/person.rs::users_without_sales_person` (Fix, D-62-05) als auch die neue `list_directory_entries`-Query — beide Stellen müssen den `sp.deleted IS NULL`-Filter in der `ON`-Klausel tragen, niemals in `WHERE`.

### `gen_service_impl!` für Business-Logic-Tier
**Source:** `service_impl/src/voluntary_stats.rs:41-56`
**Apply to:** `service_impl/src/person_directory.rs` — Makro-Aufruf mit den tatsächlich benötigten Basic-Deps (DAO + Permission + Transaction; ggf. weitere Domain-Services nur falls die Rollen-Bulk-Query in einen anderen Service ausgelagert wird).

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` (neue Keys) | i18n config | — | Kein "Analog" im klassischen Sinn nötig — Muster ist reine Datenerweiterung bestehender Key-Struct/Match-Arme; Planner soll bestehende Keys (`ColumnType`, `ColumnLinkedUser`, `ColumnRoles`, `Unlinked`, `SearchPlaceholder`, `Paid`, `Volunteer`, `Inactive`) direkt wiederverwenden statt neue zu duplizieren, wo inhaltlich passend. |

## Metadata

**Analog search scope:** `dao/src/`, `dao_impl_sqlite/src/`, `service/src/`, `service_impl/src/`, `rest-types/src/`, `rest/src/`, `shifty_bin/src/`, `shifty-dioxus/src/page/`, `shifty-dioxus/src/service/`, `shifty-dioxus/src/loader.rs`, `shifty-dioxus/Dioxus.toml`
**Files scanned/read:** `dao/src/person.rs` (voll), `dao_impl_sqlite/src/person.rs` (voll), `service/src/voluntary_stats.rs` (voll), `service_impl/src/voluntary_stats.rs` (Header+Impl-Anfang), `service_impl/src/permission.rs` (Gate-Ausschnitt), `shifty_bin/src/main.rs` (DI-Ausschnitt VoluntaryStats/Person), `rest/src/permission.rs` (voll), `rest/src/lib.rs` (Modul-/ApiDoc-Ausschnitt), `rest-types/src/lib.rs` (`SalesPersonAbsenceTO`-Ausschnitt), `shifty_bin/src/integration_test/voluntary_stats.rs` (voll bis Zeile 170), `shifty-dioxus/src/page/user_management.rs` (Kopf), `shifty-dioxus/src/page/settings.rs` (Guard-Ausschnitt), `shifty-dioxus/src/service/user_management.rs` (Store + drei N+1-Loader), `shifty-dioxus/src/loader.rs` (Signatur-Grep), `shifty-dioxus/Dioxus.toml` (Proxy-Liste-Ausschnitt).
**Pattern extraction date:** 2026-09-19
