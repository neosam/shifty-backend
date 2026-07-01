# Phase 39: KW-Status Grundlage (BE+FE) — Pattern Map

**Mapped:** 2026-07-02
**Files analyzed:** 12 neue/geänderte Dateien
**Analogs found:** 11 / 12 (1 teilweise neu: WeekStatus-Enum-Diskriminant)

---

## File Classification

| Neue/geänderte Datei | Role | Data Flow | Nächster Analog | Match-Qualität |
|---|---|---|---|---|
| `migrations/sqlite/YYYYMMDD_create-week-status.sql` | migration | — | `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` | exact (soft-delete + partial UNIQUE) |
| `dao/src/week_status.rs` | DAO-Trait + Entity | CRUD | `dao/src/week_message.rs` | exact |
| `dao_impl_sqlite/src/week_status.rs` | DAO-Impl | CRUD | `dao_impl_sqlite/src/week_message.rs` + `dao_impl_sqlite/src/special_day.rs` (TEXT-enum TryFrom) | role-match |
| `service/src/week_status.rs` | Service-Trait + Domain-Struct | request-response | `service/src/week_message.rs` | exact |
| `service_impl/src/week_status.rs` | Service-Impl Basic-Tier | request-response | `service_impl/src/week_message.rs` | exact |
| `rest/src/week_status.rs` | REST-Handler + Router | request-response | `rest/src/week_message.rs` | exact |
| `rest-types/src/lib.rs` (Ergänzung) | DTO | request-response | `WeekMessageTO` in `rest-types/src/lib.rs:1238` | role-match (message:String → status:WeekStatus enum) |
| `shifty_bin/src/main.rs` (Ergänzung) | DI-Wiring | — | `main.rs:409-420 + 1034-1040` | exact |
| `shifty-dioxus/src/api.rs` (Ergänzung) | API-Client | request-response | `api.rs:1152-1202` (get/post/put_week_message) | exact |
| `shifty-dioxus/src/loader.rs` (Ergänzung) | Loader | request-response | `loader.rs:649-696` (load/save_week_message) | role-match |
| `shifty-dioxus/src/page/shiftplan.rs` (Ergänzung) | FE-Integration | event-driven | `shiftplan.rs:1580-1625` (WeekMessage-Block) + `shiftplan.rs:103-113` (is_shiftplanner) | role-match |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` (Ergänzung) | i18n | — | `i18n/mod.rs:84` (`WeekMessage`-Key) | exact |

---

## Pattern Assignments

### Migration: `migrations/sqlite/YYYYMMDD_create-week-status.sql`

**Analog:** `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` (vollständig)

**Copy-ready Skelett:**
```sql
CREATE TABLE IF NOT EXISTS week_status (
    id BLOB NOT NULL PRIMARY KEY,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    status TEXT NOT NULL,          -- 'InPlanning' | 'Planned' | 'Locked'
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

-- Nur eine aktive Zeile pro (year, calendar_week); Soft-Delete-History erlaubt.
CREATE UNIQUE INDEX IF NOT EXISTS idx_week_status_active
    ON week_status (year, calendar_week)
    WHERE deleted IS NULL;
```

**Adaptierungen vs. Analog:**
- Kein `FOREIGN KEY` (week_status gehört zu keiner anderen Entität)
- Zusätzliches `status TEXT NOT NULL`-Feld (statt `offset_days INTEGER`)
- Kein `sales_person_id`-Spalte

---

### `dao/src/week_status.rs` (Trait + Entity)

**Analog:** `dao/src/week_message.rs` (vollständig, 55 Zeilen)

**Entity-Struct-Skelett** (Analog: `week_message.rs:5-14`):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct WeekStatusEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusKind,    // ← statt message: String
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatusKind {
    InPlanning,
    Planned,
    Locked,
}
```

**Trait-Skelett** (Analog: `week_message.rs:16-55`):
```rust
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait WeekStatusDao {
    type Transaction: crate::Transaction;
    async fn find_by_id(&self, id: Uuid, tx: Self::Transaction) -> Result<Option<WeekStatusEntity>, DaoError>;
    async fn find_by_year_and_week(&self, year: u32, calendar_week: u8, tx: Self::Transaction) -> Result<Option<WeekStatusEntity>, DaoError>;
    async fn find_by_year(&self, year: u32, tx: Self::Transaction) -> Result<Vec<WeekStatusEntity>, DaoError>;
    async fn create(&self, entity: &WeekStatusEntity, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
    async fn update(&self, entity: &WeekStatusEntity, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
    async fn delete(&self, id: Uuid, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
}
```

**Adaptierungen:**
- `message: String` → `status: WeekStatusKind`
- `WeekStatusKind`-Enum im selben File definieren (nur 3 Varianten, `Unset` wird NICHT persistiert — D-39-04)

---

### `dao_impl_sqlite/src/week_status.rs`

**Analog A:** `dao_impl_sqlite/src/week_message.rs` — Gesamtstruktur (DB-Struct, TryFrom, DaoImpl, alle Methoden)

**Analog B (TEXT-Enum TryFrom):** `dao_impl_sqlite/src/special_day.rs:36-39`

**DB-Struct + TryFrom-Skelett:**
```rust
struct WeekStatusDb {
    id: Vec<u8>,
    year: i64,
    calendar_week: i64,
    status: String,                 // ← statt message: String
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&WeekStatusDb> for WeekStatusEntity {
    type Error = DaoError;
    fn try_from(db: &WeekStatusDb) -> Result<Self, Self::Error> {
        Ok(WeekStatusEntity {
            id: Uuid::from_slice(&db.id)?,
            year: db.year as u32,
            calendar_week: db.calendar_week as u8,
            // TEXT-Enum-Muster aus special_day.rs:36-39:
            status: match db.status.as_str() {
                "InPlanning" => WeekStatusKind::InPlanning,
                "Planned"    => WeekStatusKind::Planned,
                "Locked"     => WeekStatusKind::Locked,
                value => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            deleted: db.deleted.as_ref()
                .map(|d| PrimitiveDateTime::parse(d, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}
```

**SQL-Queries:** alle identisch zu `week_message.rs`, nur Tabellenname `week_status` und Spalte `status` statt `message`.

**Adaptierungen:**
- `WeekStatusKind` → `&str`-Konvertierung in `create`/`update` (Enum → String):
  ```rust
  let status_str = match entity.status {
      WeekStatusKind::InPlanning => "InPlanning",
      WeekStatusKind::Planned    => "Planned",
      WeekStatusKind::Locked     => "Locked",
  };
  ```
  (kein direktes `.to_string()` — explizites `match` wie `special_day.rs`)

---

### `service/src/week_status.rs` (Trait + Domain-Struct)

**Analog:** `service/src/week_message.rs` (vollständig, 102 Zeilen)

**Domain-Struct-Skelett** (Analog: `week_message.rs:11-52`):
```rust
#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatus {
    Unset,         // ← nur im Service/FE, nie persistiert (D-39-04)
    InPlanning,
    Planned,
    Locked,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeekStatusItem {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatus,        // ← statt message: Arc<str>
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
```

**From-Impls** (Analog: `week_message.rs:21-52`):
- `From<&WeekStatusEntity> for WeekStatusItem` — Entity-`WeekStatusKind` → Service-`WeekStatus`
- `TryFrom<&WeekStatusItem> for WeekStatusEntity` — `WeekStatus::Unset` kann nicht persistiert werden → `Err(ServiceError::InvalidInput(...))` oder `Unset` als Lösch-Indikator behandeln (per D-39-04: Unset = Zeile löschen, nicht speichern)

**Trait-Skelett** (Analog: `week_message.rs:54-102`):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait WeekStatusService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_by_id(&self, id: Uuid, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Option<WeekStatusItem>, ServiceError>;
    async fn get_by_year_and_week(&self, year: u32, calendar_week: u8, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Option<WeekStatusItem>, ServiceError>;
    async fn get_by_year(&self, year: u32, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Arc<[WeekStatusItem]>, ServiceError>;
    async fn upsert(&self, year: u32, calendar_week: u8, status: WeekStatus, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Option<WeekStatusItem>, ServiceError>;
    // Hinweis: statt separater create/update/delete — upsert-Methode empfohlen (D-39-04: Unset = soft-delete)
}
```

**Adaptierungen:**
- `message: Arc<str>` → `status: WeekStatus` (Enum mit 4 Varianten inkl. `Unset`)
- Erwäge `upsert`-Methode statt separater `create`/`update`/`delete` (vereinfacht FE-Aufruf bei D-39-06)
- `WeekStatus::Unset` im upsert → soft-delete der vorhandenen Zeile (D-39-04)

---

### `service_impl/src/week_status.rs` (Basic-Tier-Impl)

**Analog:** `service_impl/src/week_message.rs` (vollständig, 154 Zeilen)

**gen_service_impl!-Block** (Analog: `week_message.rs:16-24`):
```rust
const WEEK_STATUS_SERVICE_PROCESS: &str = "week-status-service";

gen_service_impl! {
    struct WeekStatusServiceImpl: WeekStatusService = WeekStatusServiceDeps {
        WeekStatusDao: WeekStatusDao<Transaction = Self::Transaction> = week_status_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**SHIFTPLANNER_PRIVILEGE-Gate** (Analog: `week_message.rs:79-80`):
```rust
self.permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context)
    .await?;
```
Gilt für `upsert` (Schreib-/Lösch-Aktionen); `get_*`-Methoden benötigen keinen Gate (alle Rollen lesen).

**Transaktions-Muster** (Analog: `week_message.rs:37-40`):
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
let result = self.week_status_dao.find_by_year_and_week(year, calendar_week, tx.clone()).await?;
self.transaction_dao.commit(tx).await?;
Ok(result.map(|e| (&e).into()))
```

**Adaptierungen:**
- Kein separater `create`/`update`/`delete`-Handler erforderlich wenn `upsert` gewählt: intern `find_by_year_and_week` → exists? → `update` : `create`; Status `Unset` → `delete`
- Tier: Basic (nur DAO + Permission + Clock + Uuid + Transaction — keine Domain-Services)

---

### `rest/src/week_status.rs`

**Analog:** `rest/src/week_message.rs` (vollständig, 272 Zeilen)

**Router-Skelett** (Analog: `week_message.rs:18-32`):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/by-year-and-week/{year}/{week}", get(get_week_status_by_year_and_week::<RestState>))
        .route("/by-year-and-week/{year}/{week}", put(upsert_week_status::<RestState>))
        .route("/by-year/{year}", get(get_week_statuses_by_year::<RestState>))
}
```

**Handler-Skelett** (Analog: `week_message.rs:34-67`):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/by-year-and-week/{year}/{week}",
    tags = ["Week Status"],
    request_body = WeekStatusTO,
    responses(
        (status = 200, description = "Week status updated", body = WeekStatusTO),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn upsert_week_status<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
    Json(week_status): Json<WeekStatusTO>,
) -> Response {
    error_handler((async {
        let result = rest_state.week_status_service()
            .upsert(year, week, (&week_status).into(), context.into(), None)
            .await?;
        // ...
    }).await)
}
```

**ApiDoc-Struct** (Analog: `week_message.rs:260-272`):
```rust
#[derive(OpenApi)]
#[openapi(
    paths(get_week_status_by_year_and_week, get_week_statuses_by_year, upsert_week_status),
    components(schemas(WeekStatusTO))
)]
pub struct WeekStatusApiDoc;
```

**Adaptierungen:**
- Pfad `/week-status/...` statt `/week-message/...`
- Kein separates `/{id}`-GET/PUT/DELETE — `by-year-and-week`-Endpunkt genügt (D-39-06: FE sendet immer year+week)
- Tag: `"Week Status"`

---

### `rest-types/src/lib.rs` — `WeekStatusTO`

**Analog:** `rest-types/src/lib.rs:1238-1282` (`WeekMessageTO` + From-Impls)

**TO-Skelett:**
```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WeekStatusKindTO {
    InPlanning,
    Planned,
    Locked,
    Unset,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WeekStatusTO {
    #[serde(default)]
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusKindTO,   // ← statt message: Arc<str>
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
```

**Adaptierungen:**
- `message: Arc<str>` → `status: WeekStatusKindTO`
- `WeekStatusKindTO`-Enum separat mit `ToSchema`-Derive
- `#[cfg(feature = "service-impl")]`-From-Impls analog `WeekMessageTO:1254-1282` (Enum-Mapping `WeekStatusKindTO` ↔ `service::week_status::WeekStatus`)

---

### `shifty_bin/src/main.rs` — DI-Wiring

**Analog:** `main.rs:409-420` (Dependencies-Struct) + `main.rs:1034-1040` (Instanziierung)

**Dependencies-Struct** (Analog: `main.rs:409-420`):
```rust
pub struct WeekStatusServiceDependencies;
impl service_impl::week_status::WeekStatusServiceDeps for WeekStatusServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type WeekStatusDao = WeekStatusDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type WeekStatusService =
    service_impl::week_status::WeekStatusServiceImpl<WeekStatusServiceDependencies>;
```

**Instanziierung** (Analog: `main.rs:1034-1040`):
```rust
let week_status_service = Arc::new(WeekStatusService {
    week_status_dao: Arc::new(WeekStatusDao::new(pool.clone())),
    permission_service: permission_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```

**Platzierung:** Basic-Service-Schicht (vor Business-Logic), direkt neben dem `week_message_service`-Block (ca. Zeile 1034).

**Adaptierungen:**
- Neuer Typ `WeekStatusDao` (Import hinzufügen), Neues `week_status_service`-Feld in `AppState`-Struct + `RestStateDef`-Impl

---

### FE: `shifty-dioxus/src/api.rs` — API-Client

**Analog:** `api.rs:1152-1202` (get/post/put_week_message)

**Skelett:**
```rust
pub async fn get_week_status(config: Config, year: u32, week: u8) -> Result<Option<WeekStatusTO>, reqwest::Error> {
    let url = format!("{}/week-status/by-year-and-week/{}/{}", config.backend, year, week);
    let response = reqwest::get(url).await?;
    if response.status() == 404 { return Ok(None); }
    response.error_for_status_ref()?;
    Ok(Some(response.json().await?))
}

pub async fn put_week_status(config: Config, week_status: WeekStatusTO) -> Result<(), reqwest::Error> {
    let url = format!("{}/week-status/by-year-and-week/{}/{}", config.backend, week_status.year, week_status.calendar_week);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&week_status).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

**Adaptierungen:**
- Kein separates `post_week_status` nötig — Backend-upsert via PUT (D-39-06)
- URL: `/week-status/by-year-and-week/{year}/{week}`

---

### FE: `shifty-dioxus/src/loader.rs` — Loader

**Analog:** `loader.rs:649-696` (load/save_week_message)

**Skelett:**
```rust
pub async fn load_week_status(config: Config, year: u32, week: u8) -> Result<WeekStatusKindTO, ShiftyError> {
    match api::get_week_status(config, year, week).await? {
        Some(ws) => Ok(ws.status),
        None => Ok(WeekStatusKindTO::Unset),   // D-39-04: keine Zeile = Unset
    }
}

pub async fn save_week_status(config: Config, year: u32, week: u8, status: WeekStatusKindTO) -> Result<(), ShiftyError> {
    // GET existing (für id+version bei Update), dann PUT
    let existing = api::get_week_status(config.clone(), year, week).await?;
    let week_status = WeekStatusTO {
        id: existing.as_ref().map(|e| e.id).unwrap_or_default(),
        year, calendar_week: week, status,
        created: existing.as_ref().and_then(|e| e.created),
        deleted: None,
        version: existing.as_ref().map(|e| e.version).unwrap_or_default(),
    };
    api::put_week_status(config, week_status).await?;
    Ok(())
}
```

**Adaptierungen:**
- Rückgabe `WeekStatusKindTO` (Enum), nicht `String`
- Keine POST/PUT-Unterscheidung — immer PUT (Backend upsert)
- `None` → `Unset` (D-39-04)

---

### FE: `shifty-dioxus/src/page/shiftplan.rs` — Badge + Dropdown

**Analog A (is_shiftplanner-Gate):** `shiftplan.rs:103-113`
```rust
let is_shiftplanner = auth_info
    .as_ref()
    .map(|auth_info| auth_info.has_privilege("shiftplanner"))
    ...
```

**Analog B (Wochenkontext-Block):** `shiftplan.rs:1580-1625` (WeekMessage-Block — Signal + bedingtes Rendering)

**Badge-Logik (NEU — kein direkter Analog):**
```rust
// Farb-Klassen per D-39-08
let badge_class = match *week_status.read() {
    WeekStatusKindTO::InPlanning => Some("bg-amber-100 text-amber-800 ..."),
    WeekStatusKindTO::Planned    => Some("bg-green-100 text-green-800 ..."),
    WeekStatusKindTO::Locked     => Some("bg-red-100 text-red-800 ..."),
    WeekStatusKindTO::Unset      => None,  // D-39-05: kein Badge
};

// Rendering oberhalb der Wochenansicht
if let Some(cls) = badge_class {
    span { class: cls, {i18n.t(week_status_key)} }
}
// Nur für Schichtplaner: Dropdown statt reines Badge
if is_shiftplanner {
    DropdownTrigger { entries: status_entries, ... }
}
```

**Dropdown-Entries-Aufbau** (Analog: `shiftplan.rs:811-897` — SpecialDay-DropdownEntry-Konstruktion):
```rust
// DropdownEntry pro Status (inkl. Unset zum Zurücksetzen, D-39-07)
let entries: Rc<[DropdownEntry]> = [Unset, InPlanning, Planned, Locked]
    .into_iter()
    .map(|s| DropdownEntry { label: ..., on_click: Arc::new(move || { /* PUT status */ }) })
    .collect();
```

**Signal-Lade-Muster** (Analog: `shiftplan.rs:368-374`):
```rust
// Nach Wochenwechsel (und nach jeder Mutation): Status frisch vom Server laden
let status = loader::load_week_status(config.clone(), *year.read(), *week.read()).await;
week_status.set(status.unwrap_or(WeekStatusKindTO::Unset));
```
Kein optimistisches Signal-Update (D-39-06: immer Server-Reload nach Mutation).

---

### FE: `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs`

**Analog:** `i18n/mod.rs:84` (`WeekMessage`-Key), analog in de/en/cs.rs

**Neue Keys (in `Key`-Enum hinzufügen):**
```rust
WeekStatusUnset,
WeekStatusInPlanning,
WeekStatusPlanned,
WeekStatusLocked,
WeekStatusLabel,   // "KW-Status" / "Week Status" / "Stav týdne"
```

**Translations** (D-39-09):
| Key | de | en | cs |
|---|---|---|---|
| `WeekStatusUnset` | Kein | None | Žádný |
| `WeekStatusInPlanning` | In Planung | In planning | V plánování |
| `WeekStatusPlanned` | Geplant | Planned | Naplánováno |
| `WeekStatusLocked` | Gesperrt | Locked | Uzamčeno |

---

## Shared Patterns

### SHIFTPLANNER_PRIVILEGE-Gate
**Quelle:** `service_impl/src/week_message.rs:79-80`
**Anwenden auf:** `service_impl/src/week_status.rs` in `upsert()`
```rust
self.permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context)
    .await?;
```

### Soft-Delete-Muster
**Quelle:** `dao_impl_sqlite/src/week_message.rs:192-211`
**Anwenden auf:** `dao_impl_sqlite/src/week_status.rs::delete()`
```rust
let now_str = time::OffsetDateTime::now_utc().format(&Iso8601::DATE_TIME).map_db_error()?;
query!(r#"UPDATE week_status SET deleted = ?, update_process = ? WHERE id = ?"#,
    now_str, process, id_vec)
    .execute(tx.tx.lock().await.as_mut()).await.map_db_error()?;
```

### Transaktions-Wrapper
**Quelle:** `service_impl/src/week_message.rs:37-40`
**Anwenden auf:** alle Service-Methoden
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... DAO-Aufruf ...
self.transaction_dao.commit(tx).await?;
```

### error_handler + utoipa-Annotations
**Quelle:** `rest/src/week_message.rs:34-67`
**Anwenden auf:** alle Handler in `rest/src/week_status.rs`

---

## Kein sauberer Analog

| Datei / Konzept | Reason |
|---|---|
| `WeekStatus::Unset`-Diskriminant (Service-Enum) | Bisher hat kein Service einen „Null"-Status-Wert, der explizit im Enum lebt aber nie persistiert wird. `Unset` = Zeile-fehlt-Semantik muss im upsert-Handler implementiert werden (D-39-04). |
| Badge-Farbklassen (FE) | Kein bestehendes farbkodiertes Status-Badge-Atom; nächster Analog ist der Dot-Indikator in `shiftplan.rs:988-994` (Holiday=accent, ShortDay=warn). Farben per D-39-08: `Locked`=rot, `Planned`=grün, `InPlanning`=amber. |
| Dropdown-Trigger oberhalb der Wochenansicht (FE-Position) | `DropdownTrigger` existiert (`component/dropdown_base.rs`) und wird in der Wochenansicht genutzt, aber immer im Sub-Header-Grid, nie im globalen Wochen-Header. Neue Positionierung erforderlich. |

---

## Metadata

**Analog-Suchbereich:** `dao/`, `dao_impl_sqlite/`, `service/`, `service_impl/`, `rest/`, `rest-types/src/lib.rs`, `shifty_bin/src/main.rs`, `shifty-dioxus/src/`
**Dateien gelesen:** 12
**Mapping-Datum:** 2026-07-02
