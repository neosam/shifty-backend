# Phase 33: Special-Days-UI in den Einstellungen - Research

**Researched:** 2026-06-30
**Domain:** Rust/Axum backend CRUD extension + Dioxus 0.6.x frontend CRUD wiring
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-33-01:** Special-Days-Pflege wird durchgängig auf `shiftplanner` gegated (beide Flächen).
`SHIFTPLANNER_PRIVILEGE = "shiftplanner"`. Kein Backend-Permission-Change nötig.

**D-33-02:** Settings-Sektion wird per-Card shiftplanner-gegated, NICHT hinter das pauschale
`admin`-Gate der SettingsPage. Card-3 bekommt eigenen Inner-Guard.

**D-33-03:** Schichtplan-Seite: Per-Tag-Dropdown (Mo–So) mit Optionen Feiertag / Kurzer Tag /
Nichts. Nichts = delete. Kein Datepicker-Caveat (Wochen-Kontext liefert `(year, KW, weekday)` direkt).

**D-33-04:** Settings-Seite: Kalenderdatum-Picker → Mapping Datum→(year, iso_week, weekday) über
`time::Date::parse` + `.to_iso_week_date()`. WASM-Datepicker-Caveat (D-25-06) gilt → `oninput`/
`event.data.value()` via `TextInput.on_change`.

**D-33-05:** Neuer Backend-Read-Endpoint (z. B. `GET /special-days/for-year/{year}`) speist die
Settings-Jahres-Liste. DAO `find_by_year`, Service `get_by_year`, REST Route + `#[utoipa::path]` +
ApiDoc-Eintrag. Read-Permission ungegated (wie `for-week`).

**D-33-06:** Bei Typ `ShortDay` ist `time_of_day` Pflichtfeld (Submit erst aktiv wenn gültig).
Bei `Holiday` kein Uhrzeitfeld.

**D-33-07:** Duplikat am selben `(year, calendar_week, day_of_week)` → Inline-Hinweis auf
Settings-Fläche (live duplicate check gegen geladene Liste). Auf Schichtplan-Fläche: Backend-
Fehler → inline error span.

**D-33-08:** Chronologisch aufsteigend, nach Jahr gruppiert. Typ als Badge. ShortDay zeigt
Uhrzeit. Empty-State mit Hinweistext.

### Claude's Discretion

- Exakte Seiten-Gate-Verdrahtung (Page-Gate lockern vs. Card-eigener Guard) — solange FE-Gate
  `shiftplanner` ist.
- Konkreter Endpoint-Name (`for-year/{year}` vs. Range-Query) und Read-Permission.
- Ob Duplikat-Prüfung zusätzlich serverseitig erzwungen wird.
- Konkretes UI-Layout (Badge-Styling, Empty-State-Styling) — ausgerichtet am bestehenden Set.
- Alle i18n-Labels/Texte (de/en/cs) — Typen, Optionen, Form-Labels, Listen-Kontext,
  Empty-State, Inline-Hinweise.

### Deferred Ideas (OUT OF SCOPE)

- ShortDay-Soll-Automatik im Report (anteilig, `time_of_day`) — Future-Story.
- Hover-Tooltip auf Feiertags-Zelle in der Schichtplan-Tabelle — Phase-34-Differentiator.
- Weitere „Tag-Einstellungen" im Dropdown über Special Days hinaus.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPD-01 | Shiftplanner kann Special Day per Kalenderdatum anlegen (Holiday/ShortDay; ShortDay mit Uhrzeit). Date→ISO-Woche/Wochentag-Mapping im FE. WASM-Datepicker-Caveat beachten. | Settings Card-3 Create-Form mit `TextInput {input_type: "date"}`, `oninput` via TextInput.on_change, Mapping via `time::Date::parse` + `.to_iso_week_date()`, POST zu `/special-days/` via `create_special_day(config, body)`. |
| SPD-02 | Shiftplanner sieht vorhandene Special Days als Liste mit Datum im locale-üblichen Format plus abgeleitetem Kontext (Wochentag, KW, Jahr). | `get_special_days_for_year(config, year)` → neuer `GET /special-days/for-year/{year}` Endpoint. Date context string in Rust aus `SpecialDayTO`-Feldern via `time::Date::from_iso_week_date` + `i18n.format_date()`. |
| SPD-03 | Shiftplanner kann vorhandenen Special Day löschen (FE gegen `DELETE /special-days/{id}` verdrahtet). | `delete_special_day(config, id: Uuid)` in api.rs, DELETE-Button in Settings-Liste und "Nichts"-Option im Schichtplan-Dropdown. |
| SPD-04 | Special-Days-Pflege shiftplanner-gated auf beiden Flächen. Alle Texte i18n de/en/cs. | D-33-01/02: FE-Gate `has_privilege("shiftplanner")`, 17 neue i18n-Keys in Key-Enum + de/en/cs. |
</phase_requirements>

---

## Summary

Phase 33 ist frontend-zentriert mit einem kleinen Backend-Anteil. Das Backend-CRUD (`POST /special-days/`, `DELETE /special-days/{id}`, `GET /special-days/for-week/{year}/{week}`) existiert vollständig und ist produktionsreif. Nur ein neuer Read-Endpoint (`GET /special-days/for-year/{year}`) muss ergänzt werden — er folgt dem identischen Muster wie `for-week`. Die gesamte Frontend-Seite (Create/Delete verdrahten, Settings-Card-3 mit Jahres-Liste, Schichtplan-Per-Tag-Dropdown) ist neu.

Der technische Stack ist ausschließlich Bestandscode: Axum/SQLx (Backend), Dioxus 0.6.x + Tailwind CSS 3 (Frontend), mockall (Tests). Keine neuen crates nötig.

**Primary recommendation:** Clone `get_special_days_for_week` end-to-end (DAO→Service→REST→FE-api.rs) für den `for-year`-Endpoint; dann Settings-Card-3 nach dem Card-2-Muster (`settings.rs`) aufbauen; dann Schichtplan-Dropdown nach dem `field_dropdown_entries`-Muster (`shiftplan.rs:695`).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Special Day CRUD (create/delete) | API / Backend | — | Permission-Gate, Business-Logic, Persistenz bereits in Backend |
| Range/Jahr-Read-Endpoint | API / Backend | — | Neue DAO-Query + Route, konsistent mit `for-week` |
| Settings-Sektion Card-3 | Frontend (WASM) | — | Purely UI: Form, Liste, Duplicate-Check in FE |
| Schichtplan-Per-Tag-Dropdown | Frontend (WASM) | — | UI-State aus `for-week`-Daten, in-column Interaktion |
| Date→ISO-Woche/Wochentag-Mapping | Frontend (WASM) | — | `time::Date::parse` + `.to_iso_week_date()` läuft in WASM |
| i18n (de/en/cs) | Frontend (WASM) | — | `Key`-Enum + Locale-Files in `shifty-dioxus/src/i18n/` |
| Permission-Gate (shiftplanner) | API / Backend | Frontend (WASM) | Backend ist gate-of-record; FE-Gate verhindert 403-Mismatch |

---

## Standard Stack

### Core (no new crates)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | (workspace) | Compile-time SQL query macros (`query_as!`) | Bestehende DB-Schicht |
| `time` | (workspace) | Datum-Arithmetik: `Date::parse`, `.to_iso_week_date()`, `Date::from_iso_week_date` | Bereits im gesamten Stack genutzt |
| `uuid` | (workspace) | Entity-IDs und Nil-UUID-Guard (`Uuid::nil()`) | Bestehender Standard |
| `async_trait` | (workspace) | `#[async_trait]` auf Trait-Definitionen | Bestehender Standard |
| `mockall` | (workspace) | `#[automock]` für DAO + Service-Traits in Tests | Alle anderen Service-Tests nutzen mockall |
| `utoipa` | (workspace) | `#[utoipa::path]` Annotation für neuen Endpoint | Pflicht für alle REST-Handler |
| `dioxus 0.6.x` | (workspace) | Rust/WASM Frontend-Framework | Bestehender Standard |
| `reqwest` | (workspace) | HTTP-Client in WASM für api.rs-Funktionen | Bestehender Standard |
| Tailwind CSS 3 | (workspace) | Styling via CSS-Variable-Tokens | Bestehender Standard |

**Installation:** Keine neuen crates. Alles im bestehenden Cargo-Workspace.

---

## Package Legitimacy Audit

> No new packages are installed in this phase. All code uses existing workspace crates.

| Package | Registry | Verdict | Disposition |
|---------|----------|---------|-------------|
| (keine neuen) | — | — | — |

**Packages removed due to SLOP verdict:** none
**Packages flagged as suspicious:** none

---

## Architecture Patterns

### System Architecture Diagram

```
Settings-Page (WASM)                   Schichtplan-Page (WASM)
  Card-3 (shiftplanner-gated)            Per-Tag-Dropdown (shiftplanner-gated)
    │                                       │
    ├── get_special_days_for_year ─────────┼── get_special_days_for_week
    ├── create_special_day ────────────────┼── create_special_day
    └── delete_special_day ────────────────┘── delete_special_day
                │                                     │
    ┌───────────┴─────────────────────────────────────┘
    │
    ▼  HTTP REST (reqwest in WASM)
  Backend API (Axum)
    ├── GET  /special-days/for-year/{year}   [NEW]  → SpecialDayService::get_by_year
    ├── GET  /special-days/for-week/{year}/{week}   → SpecialDayService::get_by_week
    ├── POST /special-days/                         → SpecialDayService::create
    └── DELETE /special-days/{id}                  → SpecialDayService::delete
                │
    SpecialDayServiceImpl (Basic-Tier)
    deps: SpecialDayDao + PermissionService + ClockService + UuidService
                │
    SpecialDayDaoImpl (SQLite)
    Table: special_day (year, calendar_week, day_of_week, day_type, time_of_day, deleted)
```

### Recommended Project Structure

No new files/folders except:
```
service_impl/src/test/
└── special_days.rs          # NEW: mockall unit tests for service methods

shifty-dioxus/src/
├── api.rs                   # +3 functions: create_special_day, delete_special_day, get_special_days_for_year
├── page/
│   ├── settings.rs          # +Card-3 section (shiftplanner-gated)
│   └── shiftplan.rs         # +Per-Tag-Dropdown per weekday column
└── i18n/
    ├── mod.rs               # +17 Keys in Key enum
    ├── de.rs                # +17 German translations
    ├── en.rs                # +17 English translations
    └── cs.rs                # +17 Czech translations

dao/src/special_day.rs       # +find_by_year trait method
dao_impl_sqlite/src/special_day.rs # +find_by_year impl
service/src/special_days.rs  # +get_by_year trait method
service_impl/src/special_days.rs   # +get_by_year impl
rest/src/special_day.rs      # +get_special_days_for_year handler + route + ApiDoc
service_impl/src/test/mod.rs # +pub mod special_days
```

### Pattern 1: DAO `find_by_year` (clone von `find_by_week`)

**What:** Range-Query über ein Kalenderjahr, sortiert nach `(calendar_week, day_of_week)`.
**When to use:** Settings-Jahres-Liste braucht alle Special Days eines Jahres in einem Request.

```rust
// dao/src/special_day.rs — Trait-Erweiterung
// Source: Codebase analog zu find_by_week [VERIFIED: codebase]
async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError>;

// dao_impl_sqlite/src/special_day.rs — Implementierung
async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError> {
    let year = year as i64;
    Ok(query_as!(
        SpecialDayDb,
        r#"
        SELECT id, year, calendar_week, day_of_week, day_type, time_of_day, created, deleted, update_version
        FROM special_day
        WHERE year = ? AND deleted IS NULL
        ORDER BY calendar_week ASC, day_of_week ASC
        "#,
        year
    )
    .fetch_all(&*self.pool)
    .await
    .map_db_error()?
    .iter()
    .map(SpecialDayEntity::try_from)
    .collect::<Result<_, _>>()?)
}
```

### Pattern 2: Service `get_by_year` (ungegated, wie `get_by_week`)

```rust
// service/src/special_days.rs — Trait-Erweiterung
// Source: Codebase analog zu get_by_week [VERIFIED: codebase]
async fn get_by_year(
    &self,
    year: u32,
    context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError>;

// service_impl/src/special_days.rs — Implementierung
async fn get_by_year(
    &self,
    year: u32,
    _context: Authentication<Self::Context>,
) -> Result<Arc<[SpecialDay]>, ServiceError> {
    Ok(self
        .special_day_dao
        .find_by_year(year)
        .await?
        .iter()
        .map(SpecialDay::from)
        .collect())
}
```

### Pattern 3: REST Handler `get_special_days_for_year` (clone von `get_special_days_for_week`)

```rust
// rest/src/special_day.rs [VERIFIED: codebase]
// Route in generate_route(): .route("/for-year/{year}", get(get_special_days_for_year::<RestState>))

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/for-year/{year}",
    tags = ["Special Days"],
    params(
        ("year" = u32, Path, description = "The year")
    ),
    responses(
        (status = 200, description = "Get special days for a year", body = [SpecialDayTO], content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_special_days_for_year<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(year): Path<u32>,
) -> Response {
    error_handler(
        (async {
            let special_days: Arc<[SpecialDayTO]> = rest_state
                .special_day_service()
                .get_by_year(year, context.into())
                .await?
                .iter()
                .map(SpecialDayTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&special_days).unwrap()))
                .unwrap())
        })
        .await,
    )
}

// ApiDoc — get_special_days_for_year zur paths-Liste hinzufügen:
#[derive(OpenApi)]
#[openapi(
    tags((name = "Special Days", description = "Special Days API")),
    paths(
        get_special_days_for_week,
        get_special_days_for_year,   // NEU
        create_special_days,
        delete_special_day
    ),
    components(schemas(SpecialDayTO))
)]
pub struct SpecialDayApiDoc;
```

### Pattern 4: FE API-Funktionen (api.rs)

```rust
// Source: Codebase analog zu get_special_days_for_week und delete_absence_period
// [VERIFIED: codebase]

pub async fn get_special_days_for_year(
    config: Config,
    year: u32,
) -> Result<Rc<[SpecialDayTO]>, reqwest::Error> {
    let url = format!("{}/special-days/for-year/{}", config.backend, year);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn create_special_day(
    config: Config,
    mut body: SpecialDayTO,
) -> Result<SpecialDayTO, reqwest::Error> {
    // Defensives Nil-Setzen: Backend lehnt non-nil id/version mit IdSetOnCreate/
    // VersionSetOnCreate (ServiceError) ab. Analog zu create_absence_period.
    body.id = Uuid::nil();
    body.version = Uuid::nil();
    let url = format!("{}/special-days/", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    response.error_for_status_ref()?;
    let result: SpecialDayTO = response.json().await?;
    Ok(result)
}

pub async fn delete_special_day(
    config: Config,
    id: Uuid,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/special-days/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

### Pattern 5: Date→ISO-Woche/Wochentag-Mapping (Settings-Flow)

```rust
// Source: Codebase — settings.rs Card-2 + service/src/datetime_utils.rs + shifty-utils
// [VERIFIED: codebase]

// Input: date_str aus TextInput {input_type: "date"} via oninput/event.data.value()
// Format: "YYYY-MM-DD"
let date_format = time::macros::format_description!("[year]-[month]-[day]");
if let Ok(date) = time::Date::parse(&date_str, date_format) {
    let (iso_year, iso_week, weekday) = date.to_iso_week_date();
    // weekday: time::Weekday → DayOfWeekTO via From<DayOfWeek>
    // DayOfWeek::from(weekday) → DayOfWeekTO: From<DayOfWeek> existiert in rest-types
    let day_of_week: DayOfWeekTO = shifty_utils::DayOfWeek::from(weekday).into();
    let body = SpecialDayTO {
        id: Uuid::nil(),
        version: Uuid::nil(),
        year: iso_year as u32,
        calendar_week: iso_week,
        day_of_week,
        day_type: selected_type,   // SpecialDayTypeTO::Holiday / ::ShortDay
        time_of_day: parsed_time,  // None für Holiday, Some(time) für ShortDay
        created: None,
        deleted: None,
    };
}
```

**WASM-Datepicker-Caveat:** `TextInput` verwendet intern `oninput` (nicht `onchange`) via `event.data.value()` (inputs.rs:54). Das `on_change`-Callback von `TextInput` ist deshalb WASM-kompatibel — kein Extra-Workaround nötig. [VERIFIED: codebase]

### Pattern 6: Date-Kontext-String (SPD-02)

```rust
// Source: 33-UI-SPEC.md [VERIFIED: codebase/UI-spec]
// entry: &SpecialDayTO

let weekday: time::Weekday = shifty_utils::DayOfWeek::from_number(entry.day_of_week as u8)
    .unwrap_or(shifty_utils::DayOfWeek::Monday)
    .into();
let date = time::Date::from_iso_week_date(
    entry.year as i32, entry.calendar_week, weekday
)?;
let weekday_key = match weekday {
    time::Weekday::Monday    => Key::Monday,
    time::Weekday::Tuesday   => Key::Tuesday,
    time::Weekday::Wednesday => Key::Wednesday,
    time::Weekday::Thursday  => Key::Thursday,
    time::Weekday::Friday    => Key::Friday,
    time::Weekday::Saturday  => Key::Saturday,
    time::Weekday::Sunday    => Key::Sunday,
};
let display = format!(
    "{} ({}, {} {}, {})",
    i18n.format_date(&date),         // locale-aware: "15.08.2026" / "08/15/2026"
    i18n.t(weekday_key),             // "Samstag" / "Saturday" / "Sobota"
    i18n.t(Key::SettingsSpecialDaysCalendarWeekAbbr),  // "KW" / "W" / "KT"
    entry.calendar_week,
    entry.year
);
```

Note: `DayOfWeekTO` muss zu `u8` konvertiert werden (via `to_number()` equivalent) oder der enum-Arm wird direkt gemappt. Prüfen wie `DayOfWeekTO` in rest-types definiert ist.

### Pattern 7: Settings Card-3 Inner-Guard (D-33-02)

```rust
// Source: shiftplan.rs:102-105 als Muster [VERIFIED: codebase]
// In SettingsPage() NACH dem is_admin-Check, vor dem rsx!:
let is_shiftplanner = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("shiftplanner"))
    .unwrap_or(false);

// Im rsx!-Block, nach Card-2:
if is_shiftplanner {
    // render Card-3 (special days)
}
```

### Pattern 8: Schichtplan-Dropdown für einen Tag

```rust
// Source: shiftplan.rs:695-738 als Muster [VERIFIED: codebase]
// Für jeden Wochentag (day_of_week: DayOfWeekTO):

let existing_special_day: Option<SpecialDayTO> = special_days_for_week
    .iter()
    .find(|sd| sd.day_of_week == day_of_week)
    .cloned();

let entries: Rc<[DropdownEntry]> = {
    let has_entry = existing_special_day.is_some();
    let existing_id = existing_special_day.as_ref().map(|sd| sd.id);
    vec![
        (i18n.t(Key::ShiftplanDayTypeHoliday),
         Box::new(move |_ctx| {
             // spawn: create_special_day(Holiday, time=None) + reload for-week
         }),
         false  // immer enabled
        ).into(),
        (i18n.t(Key::ShiftplanDayTypeShortDay),
         Box::new(move |_ctx| {
             // ShortDay: Show inline time prompt (Signal-basiert)
         }),
         false
        ).into(),
        (i18n.t(Key::ShiftplanDayTypeNone),
         Box::new(move |_ctx| {
             // spawn: delete_special_day(existing_id) + reload for-week
         }),
         !has_entry  // disabled wenn kein Entry
        ).into(),
    ].into()
};
```

**Wichtig:** `DropdownBase` rendert nur Entries mit `disabled == false` (dropdown_base.rs:52: `.filter(|entry| entry.disabled == false)`). Die "Nichts"-Option wird angezeigt aber disabled=true wenn kein Entry — muss als `disabled: true` DropdownEntry kodiert werden.

Für ShortDay-Inline-Prompt: Extra `Signal<Option<(u32, u8, DayOfWeekTO)>>` für den Zustand
"welcher Tag zeigt gerade den Zeit-Prompt". Das Dropdown-close löst diesen Zustand aus, und statt
dem DropdownTrigger wird die Inline-Form gerendert.

### Anti-Patterns to Avoid

- **Non-nil id/version im POST-Body:** Backend gibt `ServiceError::IdSetOnCreate` /
  `ServiceError::VersionSetOnCreate` zurück (→ HTTP 422). Immer `Uuid::nil()` setzen vor Create.
- **onchange statt oninput für Date-Inputs:** In WASM triggert `onchange` beim Date-Input nicht
  zuverlässig. `TextInput.on_change` nutzt intern `oninput` → WASM-safe. [VERIFIED: codebase]
- **Transaktion in `get_by_year` erzwingen:** Der Service braucht keine `TransactionDao`-Dep
  (existiert nicht in `SpecialDayServiceImpl`). Read-Methoden sind transaktionslos.
- **Clippy ignorieren:** `cargo clippy --workspace -- -D warnings` ist Pflicht-Gate vor jedem
  Commit. `cargo test` allein reicht nicht. [ASSUMED — policy per CLAUDE.md/Memory]
- **git commit statt jj:** Dieses Repo ist jj-managed; nur `jj`-Commands für Commits.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Date-Arithmetik (Date→KW) | Eigene KW-Berechnung | `time::Date::to_iso_week_date()` | ISO-8601 edge cases (KW 53, Jahreswechsel) |
| DayOfWeek↔Weekday-Konvertierung | Eigene Mapping-Tabelle | `DayOfWeek::from(time::Weekday)` + `Into<time::Weekday>` in shifty-utils | Existiert, getestet |
| HTTP-Client | `fetch` API direkt | `reqwest` (WASM-feature bereits aktiviert) | Bestehender Standard |
| Locale-Datum-Formatierung | Manuelle Formatierung | `i18n.format_date(&date)` | Trait existiert in `shifty-dioxus/src/i18n/mod.rs:35` |
| Dropdown-State-Management | Eigenes State-System | `DROPDOWN`-Signal + `DropdownAction` + `DropdownBase` | Bestehende globale Infrastruktur |
| Permission-Check | Eigene Prüflogik | `PermissionService::check_permission(SHIFTPLANNER_PRIVILEGE, context)` | Einheitliches Fehlermuster |

---

## Common Pitfalls

### Pitfall 1: DayOfWeekTO-Konvertierung in Date-Kontext-String

**What goes wrong:** `SpecialDayTO.day_of_week` ist `DayOfWeekTO` (rest-types enum), nicht
`time::Weekday`. Direktes `.from_iso_week_date(..., entry.day_of_week)` kompiliert nicht.

**Why it happens:** Zwei separate Enum-Typen (domain `DayOfWeek` in shifty-utils, wire
`DayOfWeekTO` in rest-types) mit je eigenen `From`-Impls.

**How to avoid:** Konversionskette: `DayOfWeekTO` → `DayOfWeek` (shifty-utils) →
`time::Weekday`. In rest-types `From<DayOfWeekTO> for DayOfWeek` oder umgekehrt prüfen
ob vorhanden; sonst `match`-Arm für die Konvertierung schreiben.

**Warning signs:** Compiler-Error `mismatched types expected time::Weekday found DayOfWeekTO`.

### Pitfall 2: SpecialDayTO.`$version` bei POST

**What goes wrong:** `SpecialDayTO` hat `#[serde(rename = "$version")]` für das `version`-Feld.
Wenn der Caller `Default::default()` auf dem DTO nutzt, könnte eine non-nil UUID mitgesendet
werden (abhängig von Default-Impl).

**Why it happens:** `#[serde(default)]` auf `version` Feld, aber `Uuid::default()` == `Uuid::nil()`
— eigentlich OK. Risiko: wenn SpecialDayTO von einer bestehenden Entity kopiert wird.

**How to avoid:** Defensiv vor Create immer: `body.id = Uuid::nil(); body.version = Uuid::nil();`.
Das Muster ist im Codebase etabliert (create_absence_period). [VERIFIED: codebase]

### Pitfall 3: DropdownBase zeigt disabled entries NICHT

**What goes wrong:** Man denkt, `disabled: true` in `DropdownEntry` zeigt einen greyed-out Eintrag.
In der Realität filtert `DropdownBase` alle `disabled == true` Entries heraus (filtert sie komplett
aus der Anzeige, rendert sie nicht).

**Why it happens:** `dropdown_base.rs:52: .filter(|entry| entry.disabled == false)`.

**How to avoid:** Für "Nichts"-Option: wenn kein Special Day für den Tag existiert, die "Nichts"-
Option entweder weglassen (kein Entry) oder mit anderem Label ("Kein Sondertag") ohne disabled.
Laut UI-SPEC: `disabled: true` wenn kein Entry — d.h. es wird komplett ausgeblendet. [VERIFIED: codebase]

### Pitfall 4: ShortDay-Zeit-Prompt Zustandsmanagement

**What goes wrong:** Das Schichtplan-Dropdown hat nach "Kurzer Tag..." Auswahl keinen
natürlichen Ort für den Inline-Prompt.

**Why it happens:** `DropdownBase` schließt sich nach Auswahl; die Inline-Form muss stattdessen
im Column-DOM erscheinen.

**How to avoid:** Extra Signal `shortday_prompt_day: Signal<Option<DayOfWeekTO>>` im ShiftPlan-
Komponenten-Scope. Wenn `shortday_prompt_day.read() == Some(day)`, wird der DropdownTrigger für
diesen Tag durch die Inline-Form ersetzt (conditional render in der day-column).

### Pitfall 5: `#[utoipa::path]` vergessen oder falsch

**What goes wrong:** Neuer Endpoint ist funktional, erscheint aber nicht in Swagger UI / OpenAPI.

**Why it happens:** Handler-Funktion braucht `#[utoipa::path(...)]` UND muss in `SpecialDayApiDoc`'s
`paths(...)` eingetragen sein.

**How to avoid:** Immer beide Stellen gleichzeitig pflegen (rest/src/special_day.rs). [VERIFIED: codebase — bestehende Muster zeigen beide Stellen]

### Pitfall 6: `cargo clippy` nicht als Gate

**What goes wrong:** `cargo test` und `cargo build` grünen, aber `nix build` schlägt fehl wegen
Clippy-Warnings.

**Why it happens:** `nix build` führt `cargo clippy -- --deny warnings` aus; `cargo test` tut das
NICHT.

**How to avoid:** Vor jedem jj-Commit `cargo clippy --workspace -- -D warnings` im Backend-Root
ausführen. Im Frontend (shifty-dioxus) Clippy aus dem Backend-Shell-Kontext aufrufen (E0514 im
eigenen nix-shell-Kontext). [ASSUMED — Policy per CLAUDE.md + Memory]

---

## Runtime State Inventory

> Nicht anwendbar — kein Rename/Refactor/Migration. Neuer Feature-Code.

---

## Validation Architecture

> `workflow.nyquist_validation` ist nicht gesetzt in `.planning/config.json` → treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Backend Framework | `tokio::test` + `mockall` (kein separates test framework nötig) |
| Frontend Framework | `cargo test` (unit) + WASM-Build-Gate |
| Backend test run | `cargo test special_day` (aus `shifty-backend/`) |
| WASM build gate | `cargo build --target wasm32-unknown-unknown` (aus `shifty-backend/shifty-dioxus/`) |
| Full backend suite | `cargo test` + `cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPD-01 | create_special_day POST wired, Holiday + ShortDay mit Uhrzeit | unit (BE service) | `cargo test -p service_impl test_create_special_day` | ❌ Wave 0 |
| SPD-01 | id/version nil guard → IdSetOnCreate / VersionSetOnCreate | unit (BE service) | `cargo test -p service_impl test_create_special_day_fails_with_id_set` | ❌ Wave 0 |
| SPD-01 | ShortDay ohne Uhrzeit: form_valid = false (FE-only) | manual / WASM | Browser-Verifikation | ❌ manual |
| SPD-02 | get_by_year returns sorted `(calendar_week, day_of_week)` | unit (BE service) | `cargo test -p service_impl test_get_by_year` | ❌ Wave 0 |
| SPD-02 | Date-Kontext-String aus SpecialDayTO-Feldern korrekt | unit (FE) | `cargo test -p shifty-dioxus` (falls WASM-unabhängige Logik extrahiert) | ❌ Wave 0 |
| SPD-03 | delete_special_day DELETE wired, Liste aktualisiert | unit (BE service) | `cargo test -p service_impl test_delete_special_day` | ❌ Wave 0 |
| SPD-03 | delete nicht-existenter ID → EntityNotFound | unit (BE service) | `cargo test -p service_impl test_delete_special_day_not_found` | ❌ Wave 0 |
| SPD-04 | create ohne shiftplanner → PermissionDenied | unit (BE service) | `cargo test -p service_impl test_create_special_day_forbidden` | ❌ Wave 0 |
| SPD-04 | delete ohne shiftplanner → PermissionDenied | unit (BE service) | `cargo test -p service_impl test_delete_special_day_forbidden` | ❌ Wave 0 |
| SPD-04 | i18n Keys in de/en/cs vorhanden | unit (FE) | `cargo test -p shifty-dioxus` (i18n completeness test existiert) | ✅ existing |
| All | WASM-Build-Gate: kein WASM-Compile-Fehler | build | `cargo build --target wasm32-unknown-unknown` | ✅ existing |
| All | Keine Clippy-Warnings | lint | `cargo clippy --workspace -- -D warnings` | ✅ existing |

### Sampling Rate

- **Per Task Commit:** `cargo test -p service_impl special_day && cargo clippy --workspace -- -D warnings`
- **Per Wave Merge:** `cargo test && cargo clippy --workspace -- -D warnings && cargo build --target wasm32-unknown-unknown` (im shifty-dioxus-Verzeichnis)
- **Phase Gate:** Full suite grün vor `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `service_impl/src/test/special_days.rs` — covers SPD-01, SPD-02, SPD-03, SPD-04 (Backend)
- [ ] `service_impl/src/test/mod.rs` — `pub mod special_days;` hinzufügen
- [ ] FE-Logik für Date-Kontext-String: wenn pure Rust-Funktion extrahierbar → Unit-Test ohne WASM

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — (kein neuer Auth-Flow) |
| V3 Session Management | no | — |
| V4 Access Control | yes | `check_permission(SHIFTPLANNER_PRIVILEGE, context)` in `create` + `delete`; `get_by_year` ungegated (konsistent mit `get_by_week`) |
| V5 Input Validation | yes | id/version nil guard in `create` (ServiceError::IdSetOnCreate); ShortDay time_of_day Pflicht (FE-Validation + implizit durch Domain-Logik) |
| V6 Cryptography | no | — |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Privilege escalation: FE-only gate | Elevation of Privilege | BE-Gate in `create`/`delete` via `check_permission` — FE-Gate ist nur UX, nicht gate-of-record |
| 403-Mismatch: Admin sieht UI, kriegt 403 | Denial of Service (UX) | D-33-02: Card-3 per-Card shiftplanner-gated; page-level admin-gate bleibt aber Card nur für shiftplanner visible |
| Double-submit während Save | Tampering | `saving.set(true)` guard vor `spawn`; Button disabled während Save |

---

## Code Examples

### Vollständiges `find_by_year` DAO-SQL

```sql
-- Source: Codebase analog zu find_by_week [VERIFIED: codebase]
SELECT id, year, calendar_week, day_of_week, day_type, time_of_day, created, deleted, update_version
FROM special_day
WHERE year = ? AND deleted IS NULL
ORDER BY calendar_week ASC, day_of_week ASC
```

### Service-Test-Template (mockall-Pattern)

```rust
// Source: service_impl/src/test/slot.rs als Muster [VERIFIED: codebase]
#[cfg(test)]
mod tests {
    use super::*;
    use dao::special_day::MockSpecialDayDao;
    use service::{clock::MockClockService, uuid_service::MockUuidService, MockPermissionService};
    use mockall::predicate::eq;
    use service::permission::SHIFTPLANNER_PRIVILEGE;
    use uuid::{uuid, Uuid};

    fn make_service() -> SpecialDayServiceImpl<
        MockSpecialDayDao, MockPermissionService, MockClockService, MockUuidService
    > {
        SpecialDayServiceImpl::new(
            Arc::new(MockSpecialDayDao::new()),
            Arc::new(MockPermissionService::new()),
            Arc::new(MockClockService::new()),
            Arc::new(MockUuidService::new()),
        )
    }

    #[tokio::test]
    async fn test_get_by_year_returns_entries() {
        let mut dao = MockSpecialDayDao::new();
        dao.expect_find_by_year()
            .with(eq(2026u32))
            .returning(|_| Ok(Arc::from([])));
        // ... assert Ok([])
    }

    #[tokio::test]
    async fn test_create_forbidden_without_shiftplanner() {
        let mut permission = MockPermissionService::new();
        permission
            .expect_check_permission()
            .with(eq(SHIFTPLANNER_PRIVILEGE), always())
            .returning(|_, _| Err(ServiceError::Forbidden));
        // ... assert Err(ServiceError::Forbidden)
    }
}
```

### i18n-Key-Ergänzung (mod.rs)

```rust
// Source: shifty-dioxus/src/i18n/mod.rs [VERIFIED: codebase]
// Nach SettingsHolidayAutoCreditUnsetHint einfügen (nach Key 621):
SettingsSpecialDaysSectionLabel,
SettingsSpecialDaysSectionDescription,
SettingsSpecialDaysYearLabel,
SettingsSpecialDaysDateLabel,
SettingsSpecialDaysTypeLabel,
SettingsSpecialDaysTypeHoliday,
SettingsSpecialDaysTypeShortDay,
SettingsSpecialDaysTimeLabel,
SettingsSpecialDaysAddBtn,
SettingsSpecialDaysEmptyBody,
SettingsSpecialDaysDuplicateHint,
SettingsSpecialDaysDeleteBtn,
SettingsSpecialDaysDeleteError,
SettingsSpecialDaysCalendarWeekAbbr,
ShiftplanDayTypeHoliday,
ShiftplanDayTypeShortDay,
ShiftplanDayTypeNone,
ShiftplanDayShortDayConfirm,
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FE liest nur Special Days (kein Create/Delete) | FE-CRUD voll verdrahtet (Phase 33) | v1.10 | SPD-01/03 |
| ~53 `for-week`-GETs für Settings-Jahres-Liste | Neuer `for-year`-Endpoint (1 Request) | v1.10 (D-33-05) | Performance, SPD-02 |
| Settings-Page vollständig admin-gated | Card-3 per-Card shiftplanner-gated | Phase 33 | D-33-02 |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `DayOfWeekTO` → `DayOfWeek` (shifty-utils) Konversion existiert in rest-types oder ist trivial umsetzbar | Code Examples Pattern 6 | Zusätzlicher Boilerplate für Konvertierung nötig; kein Risk für Korrektheit |
| A2 | Clippy-Gate muss vor jedem Commit laufen (per Memory/CLAUDE.md) | Anti-Patterns | Wenn Gate fehlt: nix build schlägt fehl |
| A3 | `DropdownEntry` mit `disabled: true` wird komplett ausgeblendet (nicht nur disabled) | Pattern 8 / Pitfall 3 | Falls doch angezeigt: "Nichts"-Option sichtbar auch wenn kein Entry → korrekt (harmlos, nicht klickbar) |

---

## Open Questions (RESOLVED)

1. **DayOfWeekTO-Konvertierung in Date-Kontext-String**
   - Was wir wissen: `DayOfWeekTO` ist in rest-types definiert; `DayOfWeek` in shifty-utils hat `From<time::Weekday>`
   - Was unklar: Ob `From<DayOfWeekTO> for DayOfWeek` bereits existiert in rest-types (vermutlich ja für Backend-Seite, aber FE nutzt `rest-types` direkt)
   - Recommendation: Im Implementierungs-Task `rest-types/src/lib.rs` prüfen und `DayOfWeekTO` → `time::Weekday` Konversionspfad ggf. ergänzen

2. **ShortDay-Zeit-Prompt in Schichtplan: Komponentengrenzen**
   - Was wir wissen: DropdownBase schließt sich nach Auswahl; Prompt muss im Tag-Column erscheinen
   - Was unklar: Ob Signal auf ShiftPlan-Ebene oder sub-component-Ebene gehalten wird
   - Recommendation: Signal auf ShiftPlan-Komponent-Ebene (wie bestehende `slot_edit_service`-State)

---

## Environment Availability

> Nur bestehende Toolchain benötigt.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (stable) | Backend Cargo build/test | ✓ | (workspace) | — |
| wasm32-unknown-unknown target | WASM-Build-Gate | ✓ | (per nix flake) | — |
| SQLx + sqlx-cli | DAO compile-time checks | ✓ | (per nix develop) | — |
| npx tailwindcss | FE CSS-Build | ✓ | (per package.json) | — |

**Missing dependencies with no fallback:** none

---

## Sources

### Primary (HIGH confidence)

- `rest/src/special_day.rs` — vollständige REST-Schicht als Clone-Vorlage
- `dao/src/special_day.rs` + `dao_impl_sqlite/src/special_day.rs` — DAO-Trait + Impl
- `service/src/special_days.rs` + `service_impl/src/special_days.rs` — Service-Trait + Impl
- `shifty-dioxus/src/page/settings.rs` — Card-2 als Settings-Card-Muster
- `shifty-dioxus/src/page/shiftplan.rs:695-738` — Dropdown-Muster
- `shifty-dioxus/src/api.rs:608-667` — create_/delete_ Absence als FE-API-Muster
- `shifty-dioxus/src/component/form/inputs.rs:54` — TextInput oninput (WASM-safe)
- `shifty-dioxus/src/component/dropdown_base.rs:52` — disabled-Filter-Verhalten
- `shifty-utils/src/date_utils.rs` — DayOfWeek, ShiftyDate, iso_week_date
- `.planning/phases/33-special-days-ui-einstellungen/33-CONTEXT.md` — D-33-01..08
- `.planning/phases/33-special-days-ui-einstellungen/33-UI-SPEC.md` — i18n-Keys, Layout
- `.planning/REQUIREMENTS.md` — SPD-01..04

### Secondary (MEDIUM confidence)

- `service_impl/src/test/slot.rs` — mockall-Testmuster für Basic-Tier-Services

### Tertiary (LOW confidence)

- keine

---

## Metadata

**Confidence breakdown:**
- Backend-Range/Jahr-Read: HIGH — direkter Clone von `for-week`, alle Muster code-verifiziert
- Frontend-CRUD-Verdrahtung: HIGH — api.rs/settings.rs/shiftplan.rs Muster code-verifiziert
- Date-Mapping: HIGH — `time::Date` API in WASM bereits genutzt (employee_weekly_histogram.rs)
- i18n-Keys: HIGH — UI-SPEC komplett, Key-Enum-Struktur code-verifiziert
- Tests: MEDIUM — Kein bestehender SpecialDays-Test; Muster von slot.rs abgeleitet

**Research date:** 2026-06-30
**Valid until:** 2026-07-30 (stabiler Stack, keine fast-moving dependencies)
