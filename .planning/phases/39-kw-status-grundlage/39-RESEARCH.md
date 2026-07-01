# Phase 39: KW-Status Grundlage (BE+FE) — Research

**Recherchiert:** 2026-07-02
**Domäne:** Rust/SQLite/Axum + Dioxus — interner Copy-Vorlage-Ansatz (keine neuen externe Deps)
**Confidence:** HIGH (alle Quellen: direkte Codebase-Inspektion dieser Session)

---

<user_constraints>
## User Constraints (aus CONTEXT.md)

### Locked Decisions
- **D-39-01:** Nur `SHIFTPLANNER_PRIVILEGE` darf setzen/ändern. Alle anderen: reine Anzeige. Muster analog `week_message`-Service.
- **D-39-02:** Alle Übergänge frei (inkl. `Locked` → `Unset`). Kein gesondertes Entsperr-Gate.
- **D-39-03:** Leer-Variante heißt `Unset` (kein `None` wg. Clippy/`Option`-Shadowing; kein `Open` wg. Semantik-Verwechslung).
- **D-39-04:** Persistenz-Modell = Zeilen-Abwesenheit. DB-Zeile fehlt ⇔ Status `Unset`. Rücksetzen = Soft-Delete. Kein `Unset`-Diskriminant in der DB — nur `InPlanning`/`Planned`/`Locked` bekommen Zeilen.
- **D-39-05:** Badge nur bei Status ≠ `Unset`. Nicht-Schichtplaner sehen bei `Unset` gar nichts.
- **D-39-06:** Schichtplaner erhalten ein Dropdown (kein controlled `<select>`) + Fresh-Fetch nach Mutation.
- **D-39-07:** Im Schichtplaner-Dropdown ist auch `Unset`/„Kein" wählbar.
- **D-39-08:** Farben: `Locked` = rot, `Planned` = grün, `InPlanning` = amber, `Unset` = grau (nur im Dropdown).
- **D-39-09:** i18n-Labels de/en/cs: Unset=Kein/None/Žádný, InPlanning=In Planung/In planning/V plánování, Planned=Geplant/Planned/Naplánováno, Locked=Gesperrt/Locked/Uzamčeno.
- **D-39-10:** Migration: ISO-`(year, calendar_week)` Composite, TEXT-Diskriminant, partial UNIQUE `WHERE deleted IS NULL`.
- **D-39-11:** ISO-Jahr immer aus `date.to_iso_week_date().0`, nie `date.year()`. KW-53-/Jahresgrenzen-Tests Pflicht.
- **D-39-12:** `WeekStatusService` = Basic-Tier. DI-Wiring in `main.rs` in der Basic-Schicht (vor Business-Logic).

### Claude's Discretion
- Exakter Tabellen-/Spaltenname, DTO-Feldnamen, REST-Pfad, Dropdown-vs-Popover-Detail, genaue Header-Position im Frontend. Labels dürfen sprachlich feinjustiert werden.

### Deferred Ideas (OUT OF SCOPE)
- Sperr-Durchsetzung (`assert_week_not_locked`, HTTP-423, Inline-Banner) → Phase 40.
- Bulk-KW-Status → WST-06 v2-Backlog.
- Publish-Notification → WST-07 v2-Backlog.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Support |
|----|-------------|------------------|
| WST-01 | Schichtplaner kann KW-Status setzen/ändern (neue Tabelle, Migration, Basic-Tier-Service, REST-CRUD) | Kapitel 1–5 |
| WST-02 | Status als farbkodiertes Badge in Wochenansicht (alle Rollen); Dropdown nur Schichtplaner | Kapitel 6 |
| WST-05 | i18n de/en/cs für alle vier Labels | Kapitel 6 + Pitfall P-3 |
</phase_requirements>

---

## Summary

Phase 39 ist ein reiner Copy-Vorlage-Ansatz: Die `week_message`-Stack-Dateien werden 1:1 auf `week_status` übertragen. Der einzige strukturelle Unterschied: statt einem Freitext-`message`-Feld trägt die Zeile ein TEXT-Enum-Feld `status`, das per manuellem `match` im `TryFrom` (Muster `special_day`/`extra_hours`) auf drei Varianten gemappt wird. Die vierte Variante `Unset` hat keinen DB-Diskriminant — Zeilen-Abwesenheit bedeutet `Unset`, Rücksetzen = Soft-Delete (Muster `vacation_entitlement_offset`).

Das Frontend integriert sich in `page/shiftplan.rs`: Das Badge erscheint im Wochen-Header direkt nach dem `calendar_week_str`-Span; der Schichtplaner-Dropdown sitzt an derselben Stelle und ersetzt das reine Badge für Schichtplaner. Nach jeder Mutation wird der Status frisch geladen (kein optimistisches Signal).

**Primäre Empfehlung:** Neue Dateien `dao/src/week_status.rs`, `dao_impl_sqlite/src/week_status.rs`, `service/src/week_status.rs`, `service_impl/src/week_status.rs`, `rest/src/week_status.rs` anlegen; `WeekStatusTO` in `rest-types/src/lib.rs`; Migration nach dem `vacation_entitlement_offset`-Muster; DI-Wiring direkt nach `WeekMessageService` in `main.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Status persistieren | DAO / SQLite | — | Einheitige CRUD-Entity, kein Cross-Entity-Lookup |
| Permission-Gate (SHIFTPLANNER) | Service (Basic) | — | Analog `week_message` — nur DAO + Permission + Transaction |
| Status lesen (alle Rollen) | Service (Basic) | REST | Unauthed-Read-Pfad; kein BL-Service benötigt |
| Status anzeigen (Badge) | Frontend (Dioxus) | — | Reine UI-Reaktion auf API-Antwort |
| Dropdown + Mutation | Frontend (Dioxus) | REST → Basic Service | Kein controlled-select; Fresh-Fetch nach Mutation |
| Sperr-Durchsetzung | Business-Logic-Tier (`ShiftplanEditService`) | — | **Phase 40** — außer Scope Phase 39 |

---

## 1. Ansatz-Bestätigung: week_message → week_status Mapping

### Neue Dateien und Symbole

[VERIFIED: Codebase-Inspektion]

| Pfad | Neue Symbole |
|------|-------------|
| `dao/src/week_status.rs` | `WeekStatusEntity { id, year, calendar_week, status: WeekStatusKindEntity, created, deleted, version }`, `WeekStatusDao` (Trait, `#[automock]`) |
| `dao_impl_sqlite/src/week_status.rs` | `WeekStatusDaoImpl`, `WeekStatusDb` (private Row-Struct), `TryFrom<&WeekStatusDb> for WeekStatusEntity` |
| `service/src/week_status.rs` | `WeekStatusKind` (Enum), `WeekStatusService` (Trait, `#[automock]`) |
| `service_impl/src/week_status.rs` | `WeekStatusServiceImpl` via `gen_service_impl!`, `WeekStatusServiceDeps` |
| `rest/src/week_status.rs` | `generate_route()`, Handler-Funktionen, `WeekStatusApiDoc` (`#[derive(OpenApi)]`) |
| `rest-types/src/lib.rs` | `WeekStatusTO { year: u32, calendar_week: u8, status: String }` |
| `migrations/sqlite/YYYYMMDD_create-week-status.sql` | Neue Tabelle `week_status` |

**Enum-Mapping:**

| Rust-Variante (`WeekStatusKind`) | DB-TEXT-Diskriminant | Persistiert? |
|---|---|---|
| `WeekStatusKind::Unset` | — (keine Zeile) | Nein — Zeilen-Abwesenheit |
| `WeekStatusKind::InPlanning` | `"InPlanning"` | Ja |
| `WeekStatusKind::Planned` | `"Planned"` | Ja |
| `WeekStatusKind::Locked` | `"Locked"` | Ja |

### DAO-Trait (analog `WeekMessageDao`)

```rust
// dao/src/week_status.rs
#[derive(Clone, Debug, PartialEq)]
pub struct WeekStatusEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub status: WeekStatusKindEntity,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatusKindEntity {
    InPlanning,
    Planned,
    Locked,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait WeekStatusDao {
    type Transaction: crate::Transaction;

    async fn find_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<WeekStatusEntity>, DaoError>;

    async fn create(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn update(
        &self,
        entity: &WeekStatusEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn delete(
        &self,
        id: Uuid,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
```

### Service-Trait

Die Service-Ebene exponiert `WeekStatusKind` (inkl. `Unset`) statt der DAO-Entität:

```rust
// service/src/week_status.rs
#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatusKind {
    Unset,
    InPlanning,
    Planned,
    Locked,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait WeekStatusService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Alle Rollen dürfen lesen. Gibt `WeekStatusKind::Unset` zurück wenn keine Zeile.
    async fn get_week_status(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatusKind, ServiceError>;

    /// Nur SHIFTPLANNER_PRIVILEGE. `Unset` = Soft-Delete der aktuellen Zeile.
    async fn set_week_status(
        &self,
        year: u32,
        calendar_week: u8,
        status: WeekStatusKind,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatusKind, ServiceError>;
}
```

### REST-Endpunkte

REST-Pfad: `/week-status` (analog `/week-message`).

| Method | Path | Auth | Beschreibung |
|--------|------|------|-------------|
| `GET` | `/week-status/by-year-and-week/{year}/{week}` | alle | Gibt `WeekStatusTO` zurück (status="Unset" wenn keine Zeile) |
| `PUT` | `/week-status/by-year-and-week/{year}/{week}` | SHIFTPLANNER | Setzt Status (body: `WeekStatusTO`) |

Kein `id`-basierter Endpunkt nötig (kein externer ID-Zugriff). `WeekStatusTO.status` enthält den STRING-Wert ("Unset"/"InPlanning"/"Planned"/"Locked").

---

## 2. WeekStatus-Enum und TEXT-Diskriminant

[VERIFIED: Codebase-Inspektion `dao_impl_sqlite/src/special_day.rs`, `dao_impl_sqlite/src/extra_hours.rs`]

Das `TryFrom<&WeekStatusDb> for WeekStatusEntity`-Muster exakt nach `special_day`:

```rust
// dao_impl_sqlite/src/week_status.rs — TryFrom-Snippet
status: match db.status.as_str() {
    "InPlanning" => WeekStatusKindEntity::InPlanning,
    "Planned"    => WeekStatusKindEntity::Planned,
    "Locked"     => WeekStatusKindEntity::Locked,
    value        => return Err(DaoError::EnumValueNotFound(value.into())),
},
```

**Serialize-Richtung** (DAO `create`/`update`): manuelles `match` auf `&str`:

```rust
let status_str: &str = match entity.status {
    WeekStatusKindEntity::InPlanning => "InPlanning",
    WeekStatusKindEntity::Planned    => "Planned",
    WeekStatusKindEntity::Locked     => "Locked",
};
```

**Kein `Unset`-Zweig** wird je serialisiert — `Unset` hat keinen Diskriminant. Wenn `service_impl` einen `Unset`-Status erhält, soft-deletet es die Zeile anstatt zu persistieren.

---

## 3. Migration

[VERIFIED: Codebase-Inspektion `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql`]

```sql
-- migrations/sqlite/YYYYMMDD_create-week-status.sql
-- Phase 39 (WST-01): pro ISO-(year, calendar_week) ein aktiver Status.
-- Soft-Delete + partial UNIQUE statt UNIQUE ohne WHERE (week_message hatte kein Soft-Delete-History).
CREATE TABLE IF NOT EXISTS week_status (
    id BLOB NOT NULL PRIMARY KEY,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    status TEXT NOT NULL,          -- "InPlanning" | "Planned" | "Locked"
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

-- Genau eine aktive Status-Zeile pro (year, calendar_week); Soft-Delete-History erlaubt.
CREATE UNIQUE INDEX IF NOT EXISTS idx_week_status_active
    ON week_status (year, calendar_week)
    WHERE deleted IS NULL;
```

**Wichtig:** Die alte `week_message`-Migration hatte `UNIQUE (year, calendar_week)` ohne `WHERE`-Klausel — das erlaubt keine Soft-Delete-History und ist für `week_status` nicht geeignet. Das `vacation_entitlement_offset`-Muster mit partial UNIQUE ist das korrekte Vorbild. [VERIFIED: Codebase-Inspektion]

**sqlx-prepare-Gate:**  
Nach dem Anlegen aller neuen `query!`/`query_as!`-Makros in `dao_impl_sqlite/src/week_status.rs`:

```bash
# In nix develop (nix-shell kaputt — nur nix develop):
cargo sqlx prepare --workspace
# Danach .sqlx-Verzeichnis committen!
```

---

## 4. ISO-Wochen-Korrektheit und KW-53 Edge-Cases

[VERIFIED: Codebase-Inspektion `service_impl/src/absence.rs:431`, `service_impl/src/absence.rs:741`]

Das Projekt nutzt `time 0.3.36` durchgehend. Die korrekte Ableitung des ISO-Jahres:

```rust
// KORREKT:
let (iso_year, iso_week, _weekday) = date.to_iso_week_date();
let year = iso_year as u32;

// FALSCH (gregorianisches Jahr, nicht ISO-Wochen-Jahr):
let year = date.year() as u32;  // Bricht an Jahresgrenzen!
```

### KW-53 und Jahresgrenz-Edge-Cases für Unit-Tests (WST Success-Criterion 3)

| Datum | `date.year()` | `to_iso_week_date()` | Erläuterung |
|-------|--------------|---------------------|-------------|
| 2020-12-28 (Mo) | 2020 | (2020, 53, Mon) | KW-53-Woche beginnt |
| 2020-12-31 (Do) | 2020 | (2020, 53, Thu) | KW53 2020: Dec 28–Jan 3 |
| 2021-01-01 (Fr) | 2021 | (2020, 53, Fri) | Jahr 2021, aber ISO-Woche 53 von 2020! |
| 2021-01-03 (So) | 2021 | (2020, 53, Sun) | Letzter Tag KW53 2020 |
| 2021-01-04 (Mo) | 2021 | (2021, 1, Mon) | Erste KW1 von 2021 |
| 2025-12-29 (Mo) | 2025 | (2026, 1, Mon) | Gregorianisch 2025, ISO-Jahr 2026 KW1! |
| 2025-12-31 (Mi) | 2025 | (2026, 1, Wed) | |
| 2026-01-01 (Do) | 2026 | (2026, 1, Thu) | |

**Pflicht-Unit-Tests (WST Success-Criterion 3):**

1. `2021-01-01` → `(year=2020, week=53)` — nicht `(2021, 1)`.
2. `2020-12-28` → `(year=2020, week=53)` — KW53 existiert.
3. `2025-12-29` → `(year=2026, week=1)` — ISO-Jahr ≠ gregorianisches Jahr.
4. `2025-12-28` → `(year=2025, week=52)` — noch in 2025.
5. Eine normale KW-Mitte: `2026-03-15` → `(year=2026, week=11)`.

Diese Tests sind reine `#[test]`-Funktionen ohne Service/DAO — sie prüfen nur die Mapping-Logik der `to_iso_week_date()`-Ableitung im künftigen `service_impl/src/week_status.rs`.

---

## 5. DI-Wiring in main.rs

[VERIFIED: Codebase-Inspektion `shifty_bin/src/main.rs`]

### Typ-Alias (oben in main.rs, Zeile ~46-55)

```rust
type WeekStatusDao = dao_impl_sqlite::week_status::WeekStatusDaoImpl;
```

### Dependencies-Struct (nach `WeekMessageServiceDependencies`, ca. Zeile ~420)

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

### RestStateImpl

- Feld `week_status_service: Arc<WeekStatusService>` in der Struct.
- `type WeekStatusService = WeekStatusService;` in `impl rest::RestStateDef for RestStateImpl`.
- Accessor `fn week_status_service(&self) -> Arc<Self::WeekStatusService>`.

### Konstruktion in `RestStateImpl::new()` (nach `week_message_service`, ca. Zeile ~880)

```rust
let week_status_dao = Arc::new(WeekStatusDaoImpl::new(pool.clone()));
let week_status_service = Arc::new(service_impl::week_status::WeekStatusServiceImpl {
    week_status_dao,
    permission_service: permission_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```

**Einordnung:** Unmittelbar nach `week_message_service` — beide sind Basic-Tier, beide konsumieren keine Domain-Services. Die Business-Logic-Schicht (`ShiftplanEditService`, `ReportingService` etc.) kommt danach.

### REST-Router (in `rest/src/lib.rs`)

Analog zu `week_message`:

```rust
mod week_status;  // hinzufügen

// In ApiDoc paths:
(path = "/week-status", api = week_status::WeekStatusApiDoc),

// In Router:
.nest("/week-status", week_status::generate_route::<RestState>())
```

`RestStateDef` bekommt:
```rust
type WeekStatusService: service::week_status::WeekStatusService<Context = Context, Transaction = Self::Transaction>
    + Send + Sync + 'static;
fn week_status_service(&self) -> Arc<Self::WeekStatusService>;
```

---

## 6. Frontend-Integration

[VERIFIED: Codebase-Inspektion `shifty-dioxus/src/page/shiftplan.rs`]

### Integrationspunkt

Die Schichtplan-Wochenansicht-Header-Leiste ist in `shiftplan.rs` ab ca. Zeile 1040. Die Struktur:

```
← [Vorwoche-Button] [calendar_week_str] [Nächste-Woche-Button]
   [Separator] [Wochen/Tag-Toggle]
   <-- HIER: Status-Badge (alle Rollen) / Dropdown (nur Schichtplaner) -->
   [flex-1 spacer] [iCal-Link] [Booking-Log-Button] [Mehr-Dropdown]
```

### Neue Signale und Aktion in shiftplan.rs

```rust
// Neues Signal:
let mut week_status: Signal<String> = use_signal(|| "Unset".to_string()); // "Unset"|"InPlanning"|"Planned"|"Locked"

// Neue Aktion:
ShiftPlanAction::SetWeekStatus(String),
ShiftPlanAction::LoadWeekStatus,
```

Im Coroutine-Handler: nach `PreviousWeek`/`NextWeek` und nach `SetWeekStatus` → `api::get_week_status(config, year, week).await` → `week_status.set(...)`.

### Neue API-Funktionen (api.rs)

```rust
pub async fn get_week_status(
    config: Config,
    year: u32,
    week: u8,
) -> Result<WeekStatusTO, reqwest::Error> { ... }

pub async fn put_week_status(
    config: Config,
    status_to: WeekStatusTO,
) -> Result<WeekStatusTO, reqwest::Error> { ... }
```

### Badge-RSX (alle Rollen, nur bei ≠ Unset)

Einbau **nach** dem `calendar_week_str`-Span (Zeile ~1052), **vor** dem `flex-1`-Spacer:

```rust
// In der Header-Flex-Row von shiftplan.rs:
if *week_status.read() != "Unset" {
    span {
        class: match week_status.read().as_str() {
            "Locked"     => "px-2 py-0.5 rounded-full text-small font-medium bg-bad-soft text-bad border border-bad",
            "Planned"    => "px-2 py-0.5 rounded-full text-small font-medium bg-good-soft text-good border border-good",
            "InPlanning" => "px-2 py-0.5 rounded-full text-small font-medium bg-warn-soft text-warn border border-warn",
            _            => "",
        },
        {
            match week_status.read().as_str() {
                "Locked"     => i18n.t(Key::WeekStatusLocked),
                "Planned"    => i18n.t(Key::WeekStatusPlanned),
                "InPlanning" => i18n.t(Key::WeekStatusInPlanning),
                _            => "",
            }
        }
    }
}
```

### Dropdown (nur Schichtplaner, D-39-06)

Kein controlled `<select>`. Stattdessen `DropdownTrigger` (vorhandene Komponente in `component/dropdown_base.rs`) mit 4 Einträgen:

```rust
if is_shiftplanner {
    DropdownTrigger {
        entries: [
            ("Kein", Box::new(move |_| cr.send(ShiftPlanAction::SetWeekStatus("Unset".to_string()))), false).into(),
            ("In Planung", Box::new(move |_| cr.send(ShiftPlanAction::SetWeekStatus("InPlanning".to_string()))), false).into(),
            ("Geplant", Box::new(move |_| cr.send(ShiftPlanAction::SetWeekStatus("Planned".to_string()))), false).into(),
            ("Gesperrt", Box::new(move |_| cr.send(ShiftPlanAction::SetWeekStatus("Locked".to_string()))), false).into(),
        ].into(),
        // Trigger-Button zeigt aktuellen Status (oder Badge):
        button { ... }
    }
}
```

Konkrete Dropdown-Label kommen aus dem i18n-System; die Strings oben sind Platzhalter.

### i18n-Schlüssel (neu in `i18n/mod.rs` + de/en/cs)

```rust
// i18n/mod.rs — Key-Enum erweitern:
WeekStatusUnset,
WeekStatusInPlanning,
WeekStatusPlanned,
WeekStatusLocked,
```

In allen drei Locale-Dateien (`de.rs`, `en.rs`, `cs.rs`) eintragen (D-39-09).

### Neue Komponente (empfohlen)

Eine eigenständige Komponente `component/week_status_badge.rs` oder `component/week_status_control.rs` reduziert die bereits große `shiftplan.rs`. Sie nimmt `status: String`, `is_shiftplanner: bool`, `on_change: EventHandler<String>` als Props.

---

## Don't Hand-Roll

| Problem | Nicht bauen | Stattdessen nutzen |
|---------|------------|-------------------|
| ISO-Wochen-Arithmetik | Eigene Wochenberechnungen | `time::Date::to_iso_week_date()` / `from_iso_week_date()` |
| TEXT-Enum-Serialisierung | `#[derive(Serialize)]` mit Serde-Rename | Manuelles `match` im `TryFrom` — identisch mit `special_day.rs` |
| Dropdown ohne controlled-select | `<select value=...>` | `DropdownTrigger` (vorhandene Komponente) |
| DI-Konstruktion | Keine OnceLock-Tricks | `gen_service_impl!`-Makro |

---

## Pitfalls

### Pitfall P-1: D-25-06 Controlled-Select-Desync (D-39-06)
**Was schiefgeht:** Ein `<select value={week_status}>` in Dioxus/WASM synchronisiert seinen angezeigten Wert nicht zuverlässig mit dem Signal, wenn das DOM-Element bereits existiert. Der Status-Dropdown zeigt nach einer Mutation den alten Wert, obwohl das Signal aktuell ist.  
**Warum:** Dioxus 0.6 WASM patcht DOM-Attribute inkrementell; der `value`-Attribute auf `<select>` wird nicht immer neu gesetzt, wenn das DOM-Element schon gemounted ist.  
**Wie vermeiden:** Kein controlled `<select>`. Stattdessen `DropdownTrigger` (individuelle Button-Einträge, keine `<select>`-Semantik) — entschieden in D-39-06. Der aktuell angezeigte Status wird durch das Badge im Trigger-Button kommuniziert, nicht durch den `selected`-State des `<select>`.

### Pitfall P-2: Clippy `None`-Shadowing
**Was schiefgeht:** Eine `WeekStatusKind::None`-Variante kollidiert in Match-Ausdrücken mit `Option::None`. Clippy `-D warnings` blockiert den Build bei `match status { WeekStatusKind::None => ... }`, wenn `Option` im Scope ist.  
**Warum:** Rust-Compiler meldet mögliche Namens-Ambiguität als Warning; `-D warnings` macht daraus einen Fehler.  
**Wie vermeiden:** Variante heißt `Unset` (D-39-03) — kein Konflikt mit `Option::None`.

### Pitfall P-3: Fehlender ApiDoc-Eintrag
**Was schiefgeht:** `WeekStatusApiDoc` ist zwar in `rest/src/week_status.rs` definiert, wird aber nicht in die `ApiDoc`-Struct in `rest/src/lib.rs` eingetragen. Swagger-UI zeigt die neuen Endpunkte nicht.  
**Wie vermeiden:** `(path = "/week-status", api = week_status::WeekStatusApiDoc)` in `rest/src/lib.rs` ergänzen (analog Zeile 539 für `week_message`).

### Pitfall P-4: Fehlende sqlx-prepare nach neuen Queries
**Was schiefgeht:** `cargo test`/`cargo build` laufen lokal grün; CI schlägt mit `error: failed to find data for query` fehl, weil SQLX_OFFLINE=true ist und die neue `.sqlx`-Datei fehlt.  
**Wie vermeiden:** Nach jeder neuen `query!`/`query_as!` in `dao_impl_sqlite/src/week_status.rs` → `cargo sqlx prepare --workspace` (in `nix develop`, nicht `nix-shell`!) → `.sqlx`-Verzeichnis committen.

### Pitfall P-5: `date.year()` statt `to_iso_week_date().0`
**Was schiefgeht:** Einträge für KW-53-Tage oder frühe Januartage landen in der falschen Zeile. Beispiel: `2021-01-01.year()` = 2021, aber der Tag gehört zu ISO-Woche 53/2020.  
**Wie vermeiden:** Ausschließlich `date.to_iso_week_date().0 as u32` verwenden (D-39-11). Die Pflicht-Unit-Tests (Abschnitt 4) sichern das ab.

### Pitfall P-6: week_message-Migrations-Muster falsch kopiert
**Was schiefgeht:** Die `week_message`-Tabelle hat `UNIQUE (year, calendar_week)` **ohne** `WHERE deleted IS NULL` — sie erlaubt keine Soft-Delete-History. Für `week_status` wird Soft-Delete-History benötigt (Nachverfolgung von Statuswechseln; Phase 40 kann ggf. darauf aufbauen).  
**Wie vermeiden:** Migrations-Template ist `vacation_entitlement_offset.sql` (partial UNIQUE `WHERE deleted IS NULL`), nicht `week_message.sql`.

---

## Validation Architecture

> `workflow.nyquist_validation` ist in `.planning/config.json` nicht explizit `false` → Abschnitt PFLICHT.

### Test-Framework

| Eigenschaft | Wert |
|------------|------|
| Framework | `cargo test` (Standard-Rust-Unittest-Framework) |
| Config-Datei | Kein separater Config-File; `Cargo.toml` pro Crate |
| Schnell-Lauf | `cargo test -p service_impl week_status` |
| Gesamt-Suite | `cargo test --workspace` |

### Validierbare Eigenschaften und Test-Map

| Eigenschaft | Test-Typ | Automatisierbarer Befehl | Datei |
|------------|---------|--------------------------|-------|
| **WST-01:** KW-53/Jahresgrenz-Mapping `to_iso_week_date` vs. `year()` | reine Unit-Tests | `cargo test -p service_impl week_status::tests::iso_week` | `service_impl/src/week_status.rs` |
| **WST-01:** Soft-Delete ⇔ `Unset`-Roundtrip | Integration (in-memory SQLite) | `cargo test -p service_impl week_status::tests::set_unset_roundtrip` | `service_impl/src/week_status.rs` |
| **WST-01:** Nur Schichtplaner darf mutieren (Permission-Gate) | Unit-Test mit Mock | `cargo test -p service_impl week_status::tests::permission` | `service_impl/src/week_status.rs` |
| **WST-02:** Badge-Sichtbarkeitsregel (`Unset` → kein Badge für Nicht-Schichtplaner) | reine Unit-Funktion (Rust-Logik, kein WASM) | `cargo test -p shifty-dioxus week_status_badge` | `shifty-dioxus/src/component/week_status_badge.rs` |
| **WST-05:** i18n-Label-Vollständigkeit (alle 4 Varianten × 3 Locales) | Unit-Test | `cargo test -p shifty-dioxus i18n` | `shifty-dioxus/src/i18n/` |
| **WST-01:** Alle vier Statustransitionen (InPlanning↔Planned↔Locked↔Unset) | Integration | `cargo test -p service_impl week_status::tests::transitions` | `service_impl/src/week_status.rs` |
| **WST-01:** TEXT-Diskriminant-Validierung (unbekannter DB-Wert → `DaoError::EnumValueNotFound`) | Unit-Test (TryFrom) | `cargo test -p dao_impl_sqlite week_status::tests::unknown_discriminant` | `dao_impl_sqlite/src/week_status.rs` |

### Pflicht-Unit-Tests für KW-53 (WST Success-Criterion 3)

```rust
// service_impl/src/week_status.rs oder tests/
#[cfg(test)]
mod iso_week_tests {
    use time::Date;
    use time::macros::date;

    fn iso_year_week(d: Date) -> (u32, u8) {
        let (y, w, _) = d.to_iso_week_date();
        (y as u32, w)
    }

    #[test]
    fn kw53_2020_jan01_2021_belongs_to_2020() {
        assert_eq!(iso_year_week(date!(2021-01-01)), (2020, 53));
    }
    #[test]
    fn kw53_2020_dec28_belongs_to_kw53() {
        assert_eq!(iso_year_week(date!(2020-12-28)), (2020, 53));
    }
    #[test]
    fn dec29_2025_belongs_to_iso_2026() {
        assert_eq!(iso_year_week(date!(2025-12-29)), (2026, 1));
    }
    #[test]
    fn dec28_2025_stays_in_2025() {
        assert_eq!(iso_year_week(date!(2025-12-28)), (2025, 52));
    }
    #[test]
    fn normal_week_mid_year() {
        assert_eq!(iso_year_week(date!(2026-03-15)), (2026, 11));
    }
}
```

### Sampling-Rate

- **Pro Task-Commit:** `cargo test -p service_impl -p dao_impl_sqlite week_status` + `cargo clippy --workspace -- -D warnings`
- **Pro Wave-Merge:** `cargo test --workspace` + `cargo build --target wasm32-unknown-unknown` (im Dioxus-Workspace)
- **Phasen-Gate:** Gesamt-Suite grün + `cargo sqlx prepare --workspace` committet vor `/gsd-verify-work`

### Wave-0-Gaps (vor Implementierung anlegen)

- [ ] `service_impl/src/week_status.rs` → `#[cfg(test)] mod tests` mit ISO-Wochen-Tests
- [ ] `service_impl/src/week_status.rs` → Integration-Tests mit In-Memory-SQLite (Roundtrip, Permission, Transitions)
- [ ] `dao_impl_sqlite/src/week_status.rs` → `#[cfg(test)]` für `TryFrom`-Fehlerfall
- [ ] `shifty-dioxus/src/component/week_status_badge.rs` → Unit-Test der Sichtbarkeits-Logik (reine Rust-Funktion)
- [ ] `shifty-dioxus/src/i18n/` → i18n-Vollständigkeits-Test für alle 4 Keys × 3 Locales

---

## Security Domain

| ASVS-Kategorie | Anwendbar | Standard-Control |
|----------------|----------|-----------------|
| V4 Access Control | Ja | `SHIFTPLANNER_PRIVILEGE`-Gate in `WeekStatusServiceImpl::set_week_status` — identisch mit `week_message`-Muster |
| V5 Input Validation | Ja | `match`-basierte TEXT-Enum-Validierung im DAO `TryFrom`; unbekannte Diskriminanten → `DaoError::EnumValueNotFound` |
| V2 Authentication | Nein (via bestehender `Authentication<Context>`) | — |

**Threat Pattern:** Ein nicht-Schichtplaner könnte direkt `PUT /week-status/by-year-and-week/{year}/{week}` aufrufen. Mitigation: `check_permission(SHIFTPLANNER_PRIVILEGE, context)` am Anfang von `set_week_status` — identisches Muster wie `week_message_service::create/update/delete`. Der READ-Pfad (`get_week_status`) ist für alle Rollen offen (kein Privilege-Check nötig — Status ist nicht sensitiv).

---

## Project Constraints (aus CLAUDE.md)

[VERIFIED: Codebase-Inspektion `shifty-backend/CLAUDE.md`]

- `cargo clippy --workspace -- -D warnings` ist Pflicht-Gate; `cargo test`/`cargo build` reichen nicht.
- `cargo sqlx prepare --workspace` nach jeder neuen `query!`/`query_as!`; `.sqlx` committen.
- OpenAPI: jeder neue REST-Handler braucht `#[utoipa::path]`; neues Modul muss in `ApiDoc`-Struct eingetragen werden.
- `gen_service_impl!`-Makro für DI; Basic-Tier vor Business-Logic-Tier in `main.rs`.
- i18n: alle neuen benutzersichtbaren Texte in de/en/cs.
- ISO-Wochen-Arithmetik: `time` 0.3.36, nie `chrono`.
- `nix develop` statt `nix-shell` (shell.nix kaputt).
- jj-Repo: GSD Auto-Commit aktiv (commit_docs:true); kein manuelles `git commit`.
- Frontend WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` (aus Dioxus-Workspace).

---

## Assumptions Log

| # | Claim | Abschnitt | Risiko wenn falsch |
|---|-------|----------|-------------------|
| A1 | `DropdownTrigger`-Komponente akzeptiert 4 Einträge ohne Anpassung | Abschnitt 6 | Kleine Komponenten-Anpassung nötig; geringes Risiko |
| A2 | Der `WeekStatusService` braucht kein `ClockService` (kein `created`-Feld-Setzen nötig, da `created` im DAO gesetzt wird) | Abschnitt 1 | Falls `created` vom Service gesetzt werden soll, Dep hinzufügen |
| A3 | Phase 40 wird `WeekStatusService` als dep in `ShiftplanEditService` wiren — kein Cycle | Abschnitt 5 | Wäre architectural breaking change in Phase 40 |

> **Hinweis zu A2:** Das `WeekMessageService` nutzt `ClockService` für `created`-Setzen. Wenn `WeekStatusService` dasselbe Verhalten soll, muss `ClockService` als Dep eingebunden werden. Empfehlung: Mit `ClockService` wiren (analog `week_message`) für Konsistenz — minimales Overhead.

---

## Environment Availability

| Dependency | Benötigt von | Verfügbar | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `nix develop` shell | sqlx, cargo, dx | ✓ | flake.nix gepinnt | — |
| SQLite | DAO-Tests | ✓ | In-Memory via sqlx | — |
| `cargo sqlx prepare` | CI-SQLX_OFFLINE | ✓ | via `nix develop` | — |
| `cargo build --target wasm32-unknown-unknown` | WASM-Gate | ✓ | wasm-pack im Nix-Shell | — |

---

## Sources

### Primary (HIGH confidence)
- Codebase-Inspektion `dao/src/week_message.rs` — WeekMessageDao-Trait-Struktur
- Codebase-Inspektion `dao_impl_sqlite/src/week_message.rs` — Impl-Muster, TryFrom, CRUD
- Codebase-Inspektion `service/src/week_message.rs` — Service-Trait, WeekMessage-Struct
- Codebase-Inspektion `service_impl/src/week_message.rs` — gen_service_impl!, SHIFTPLANNER_PRIVILEGE
- Codebase-Inspektion `rest/src/week_message.rs` — REST-Handler, ApiDoc, Router
- Codebase-Inspektion `dao_impl_sqlite/src/special_day.rs` + `extra_hours.rs` — TEXT-Enum-Diskriminant-Muster
- Codebase-Inspektion `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` — partial UNIQUE Template
- Codebase-Inspektion `service_impl/src/absence.rs:431,741,854` — `to_iso_week_date()`-Nutzung
- Codebase-Inspektion `shifty_bin/src/main.rs:409-420,742-890` — DI-Wiring-Muster und Konstruktions-Reihenfolge
- Codebase-Inspektion `shifty-dioxus/src/page/shiftplan.rs:1040-1200` — Frontend-Header-Bereich, `is_shiftplanner`, `week_message`-Signal-Muster
- Codebase-Inspektion `rest/src/lib.rs:36,350,426,539,624` — RestStateDef-Muster, Router-Registrierung, ApiDoc

---

## Metadata

**Confidence-Aufschlüsselung:**
- Standard-Stack: HIGH — alles direkt im Codebase verifiziert
- Architektur: HIGH — 1:1-Kopiervorlage aus `week_message`; kein spekulativer Anteil
- Pitfalls: HIGH — P-1/P-2/P-3/P-4 aus Praxis-Erfahrung vorheriger Phasen (MEMORY.md)

**Research-Datum:** 2026-07-02
**Gültig bis:** 2026-08-01 (stabiler Codebase; kein Fast-Moving-Ecosystem)
