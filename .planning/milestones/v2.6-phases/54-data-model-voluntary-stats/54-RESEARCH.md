# Phase 54: Data-Model + Voluntary Statistics (F1 + F2) — Research

**Researched:** 2026-07-06
**Domain:** Rust/Axum/SQLite Backend + Dioxus WASM Frontend — additive Data-Model + HR-only Read-Aggregate
**Confidence:** HIGH (alle Entscheidungen aus CONTEXT.md locked; alle Behauptungen direkt aus Repo-Dateien verifiziert)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-F1-01:** F1-Nenner = Wochen mit gültigem `working_hours`-Eintrag; `expected_hours = 0` zählt MIT.
- **D-F2-01:** F2-Soll pro-rata bei Mid-Week-Vertragswechsel (Σ `kontrakt_i.committed_voluntary × tage_i / 7`). Bewusst gegen "latest-active"-Research-Empfehlung.
- **D-54-DM-01:** UNIQUE-Constraint `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` auf `rebooking_batch`, global über alle Kinds.
- **D-54-DM-02:** Marker-Spalte `source TEXT NOT NULL DEFAULT 'manual'` auf `extra_hours`. Werte: `'manual'` | `'rebooking'`.
- **Fat Backend / Thin Client:** F1-Ist, F2-Soll, F2-Delta werden im BE berechnet; FE rendert nur DTO-Felder.
- **HR-Only via API-Level-Redaction:** `Option<f32> = None` für Non-HR (Präzedenz VAC-OFFSET-01 v1.8).
- **Snapshot-Version bleibt 12** in Phase 54. Bump 12→13 ist REB-AUTO-05 → Phase-56-discuss-phase.
- **Rebooking-Neutralität als Property-Test** (VOL-ACCT-03): CI-Guard in Phase 54 implementieren.
- **Toggle-Seed `voluntary_rebooking_auto_active_from`** in Phase 54 gesät (Default `None`).
- **Kind-Diskriminator** auf `rebooking_batch`: `TEXT NOT NULL` mit `manual | hr_suggestion | auto_cron | auto_cron_backfill`.
- **Service-Tier-Trennung:** `RebookingBatchService` = Basic; `VoluntaryStatsService` = BL.

### Claude's Discretion

- **REST-Route-Design für F1/F2:** additiv auf `EmployeeReportTO` ODER dedizierter Endpoint `GET /{id}/voluntary-stats`. Planer entscheidet basierend auf DTO-Größe + Cache-Semantik.
- **i18n-Wording final:** „Freiwillige Stunden Ist / Soll / Δ" vs. „Ist-Ø freiwillig pro Woche / Zugesagt / Konto" — Planer präzisiert.
- **FE-Row-Layout:** eine Zeile mit 3 Werten oder drei separate Zeilen.
- **Toggle-Seed-Migration:** eigene Datei oder inline in `rebooking_batch`-Migration.

### Deferred Ideas (OUT OF SCOPE)

- Kein Rebooking-Schreib-Pfad (Phase 55).
- Kein `RebookingReconciliationService` (Phase 55).
- Kein Cron, kein Backfill-Endpoint (Phase 56).
- Kein F5-Alert-Banner (Phase 55).
- Kein Snapshot-Schema-Version-Bump (Phase 56 discuss-phase).
- F5-Reject-Wochen-Slot-Freigabe (Phase 55).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VOL-STAT-01 | HR sieht Ø freiwillige Stunden / Vertragswoche im Jahresreport (Zähler = Σ VolunteerWork ohne Rebooking-Marker; Nenner = Wochen mit working_hours-Row) | Denominator-Loop über `find_working_hours_for_calendar_week` Präzedenz in `reporting.rs`; Filter: `source = 'manual'` per D-54-DM-02 |
| VOL-STAT-02 | HR-Only: `Option<f32> = None` für Non-HR im DTO; kein FE-Redact | Präzedenz VAC-OFFSET-01 v1.8; `check_permission(HR_PRIVILEGE, ctx)` im Service |
| VOL-ACCT-01 | Soll = Σ (committed_voluntary × Wochen-in-Kraft) pro-rata D-F2-01; Ist = VOL-STAT-01-Zähler; Delta = Ist − Soll | Neuer pure-fn-Helper `committed_voluntary_prorata_for_week`; iteriert EmployeeWorkDetails-Rows tagesweise |
| VOL-ACCT-02 | Soll- und Delta-Anzeige HR-Only (Option<f32>=None für Non-HR) | Analog VOL-STAT-02 |
| VOL-ACCT-03 | Property-Test: Rebooking-Pair-Row (source='rebooking') verändert weder F1-Ist noch F2-Soll noch Delta | Test in `service_impl/src/test/voluntary_stats.rs`; in-Memory-SQLite-Fixture |
</phase_requirements>

---

## Executive Summary (für den Planer)

Phase 54 ist eine additive Backend+Frontend-Phase mit drei technischen Kernen:

**1. Drei SQLite-Migrationen (keine Schema-Invasivität):** `rebooking_batch` + `rebooking_batch_entry` (zwei neue Tabellen mit Soft-Delete-Konvention und UNIQUE-Partial-Index), `source TEXT NOT NULL DEFAULT 'manual'` additiv auf `extra_hours` (ALTER TABLE, kein Backfill nötig), Toggle-Seed `voluntary_rebooking_auto_active_from`. Reihenfolge: zuerst Batch-Tabellen, dann source-Spalte, dann Toggle-Seed (am besten drei separate Dateien nach v2.4-SHC-04-Muster).

**2. Zwei neue Services:** `RebookingBatchService` (Basic, Entity-Manager, nur CRUD) und `VoluntaryStatsService` (BL, HR-gated, Read-only). Die F1/F2-Berechnungslogik lebt als pure fn in `service_impl/src/reporting.rs` (analog zu `committed_voluntary_for_calendar_week` und `apply_weekly_cap`). Für D-F2-01 pro-rata braucht der Planer einen neuen Helper `committed_voluntary_prorata_for_week` (nimmt `EmployeeWorkDetails`-Slice + ISO-Woche, iteriert Mo–So per `time::Date::from_iso_week_date`, summiert anteilig). Die vorhandene `find_working_hours_for_calendar_week`-Funktion liefert schon die Contract-Slots, gibt aber nur die Rows zurück — die tagesweise Aufsplittung ist neu.

**3. REST-Empfehlung:** Neuer dedizierter Endpoint `GET /reporting/employee/{id}/voluntary-stats` (analog zu `/{id}/weekly-statistics` in `rest/src/report.rs`) als neues Subroute im bestehenden Router. Kein Bloat auf `EmployeeReportTO`, separater Cache-Key, klare Grenze. Neues DTO `VoluntaryStatsTO` in `rest-types/src/lib.rs`.

**FE:** Neue Komponente `voluntary_stats_row.rs`, additiver Load via separatem `load_voluntary_stats`-Loader; HR-Gate = Felder `None` → Row wird nicht gerendert. Dioxus.toml `[[web.proxy]]` für den neuen Endpoint.

**Kritischer Path:** cargo sqlx prepare nach jeder neuen query!, `.sqlx/*.json` committen (CI läuft SQLX_OFFLINE=true).

---

## A. Migrations-Design

### Empfohlene Reihenfolge (drei separate Dateien)

```
20260707000000_create-rebooking-batch.sql         -- Tables + UNIQUE index + performance indices
20260707000001_add-source-column-to-extra-hours.sql -- ALTER TABLE extra_hours ADD COLUMN source
20260707000002_seed-voluntary-rebooking-toggle.sql  -- INSERT OR IGNORE into toggle
```

**Rationale für drei Dateien:** Trennung von Schema-Struktur, Daten-Schema-Change und Seed-Data entspricht v2.4-SHC-04-Muster (dort war `20260704000000` Fix + `20260704000001` Seed getrennt). Erlaubt selektives Rollback und klarere Migration-Historie.

### Migration 1: Rebooking-Tabellen

```sql
-- 20260707000000_create-rebooking-batch.sql
CREATE TABLE rebooking_batch (
    id              BLOB PRIMARY KEY,              -- Uuid BLOB, Präzedenz: week_status.id
    sales_person_id BLOB NOT NULL,                 -- Uuid BLOB (denormalisiert für UNIQUE-Constraint)
    iso_year        INTEGER NOT NULL,              -- ISO-Jahr des Rebookings
    iso_week        INTEGER NOT NULL,              -- ISO-Woche des Rebookings (NULL für kind='manual' möglich)
    kind            TEXT NOT NULL,                 -- 'manual' | 'hr_suggestion' | 'auto_cron' | 'auto_cron_backfill'
    state           TEXT NOT NULL,                 -- 'pending' | 'approved' | 'rejected' | 'skipped_locked'
    created         TIMESTAMP NOT NULL,
    approved        TIMESTAMP,
    approved_by     TEXT,                          -- username/user_id bei Approve
    deleted         TIMESTAMP,                     -- Soft-Delete-Konvention
    version         BLOB NOT NULL,                 -- Uuid, Optimistic-Lock
    update_process  TEXT NOT NULL                  -- Audit-Tag
);

CREATE TABLE rebooking_batch_entry (
    id                  BLOB PRIMARY KEY,
    batch_id            BLOB NOT NULL REFERENCES rebooking_batch(id),
    sales_person_id     BLOB NOT NULL,
    hours               REAL NOT NULL,             -- positiv: Betrag der Umbuchung
    balance_before      REAL NOT NULL,
    voluntary_actual    REAL NOT NULL,             -- F1-Ist zum Zeitpunkt des Rebookings
    voluntary_committed REAL NOT NULL,             -- F2-Soll zum Zeitpunkt des Rebookings
    extra_hours_out_id  BLOB,                      -- FK auf extra_hours.id (−N VolunteerWork), NULL bis approved
    extra_hours_in_id   BLOB,                      -- FK auf extra_hours.id (+N ExtraWork), NULL bis approved
    deleted             TIMESTAMP,
    version             BLOB NOT NULL,
    update_process      TEXT NOT NULL
);

-- D-54-DM-01: Globale Wochen-Sperre über alle Kinds (Claim-on-Suggest)
CREATE UNIQUE INDEX rebooking_batch_week_unique_idx
    ON rebooking_batch (sales_person_id, iso_year, iso_week)
    WHERE deleted IS NULL;

-- Performance-Indices (Präzedenz: ARCHITECTURE.md)
CREATE INDEX rebooking_batch_state_idx
    ON rebooking_batch (state)
    WHERE deleted IS NULL;

CREATE INDEX rebooking_batch_entry_sp_idx
    ON rebooking_batch_entry (sales_person_id)
    WHERE deleted IS NULL;
```

**SQLite-Konventionen:** `BLOB PRIMARY KEY` für Uuid (Präzedenz: alle anderen Tabellen im Repo, z.B. `week_status`, `pdf_export_config`), `TIMESTAMP` als String, `deleted TIMESTAMP` für Soft-Delete, `version BLOB` für Optimistic-Lock. [VERIFIED: repo grep auf existing migrations]

**FK-Semantik:** `REFERENCES rebooking_batch(id)` ohne `ON DELETE CASCADE` — Soft-Delete-Muster des Repos löscht nie physisch. Einträge werden über `deleted TIMESTAMP` tombstoned.

**Randfall iso_year/iso_week bei kind='manual':** Für Phase 54 braucht `kind='manual'` kein Rebooking-Pair — das ist Phase 55. Die Spalten können für manuelle Entries die Woche des Datums enthalten oder NULL. Da Phase 54 keinen Schreiber für `rebooking_batch` baut (nur DAO/Service-Skelett), ist `NOT NULL` auf `iso_year`/`iso_week` vorläufig vertretbar mit Dummy-Werten für Skeleton. Planer entscheidet ob nullable besser ist — NULL ist SQLite-konform und verhindert Constraint-Violations im Skeleton.

### Migration 2: source-Spalte auf extra_hours

```sql
-- 20260707000001_add-source-column-to-extra-hours.sql
ALTER TABLE extra_hours
    ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
```

**SQLite-Kompatibilität:** `ALTER TABLE ... ADD COLUMN ... DEFAULT` ist vollständig SQLite-kompatibel. Bestehende Rows bekommen automatisch `'manual'`. Kein separater UPDATE/Backfill nötig. [VERIFIED: SQLite-Doku + Repo-Muster ALTER TABLE in v1.x migrations]

**sqlx-prepare-Impact:** Nach dieser Migration müssen alle Queries, die `extra_hours` lesen, die neue Spalte in ihren `query_as!`/`query!`-Macros berücksichtigen. Konkret: `ExtraHoursDb`-Struct in `dao_impl_sqlite/src/extra_hours.rs` braucht ein neues Feld `source: String`. Dies ist ein harter sqlx-prepare-Gate. Planer muss `cargo sqlx prepare --workspace` nach Migration + DAO-Anpassung ausführen und `.sqlx/*.json` committen.

**Filter-Semantik (D-54-DM-02):** Alle Aggregat-Reader, die `VolunteerWork`-ExtraHours für F1/F2 zählen, filtern `WHERE source = 'manual'`. In der DAO-Schicht als SQL-Klausel — nicht als Service-Filter — effizienter und verhindert vergessene Filter in neuen Code-Pfaden. Präzedenz: `WHERE deleted IS NULL` wird überall als SQL-Bedingung, nicht als Rust-Iterator-Filter, geschrieben.

### Migration 3: Toggle-Seed

```sql
-- 20260707000002_seed-voluntary-rebooking-toggle.sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'voluntary_rebooking_auto_active_from',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), the voluntary rebooking auto-cron runs only for ISO weeks >= that date. Leave value NULL to disable (default, no rebooking).',
    'phase-54-migration'
);
```

**Präzedenz:** exakt `20260704000001_seed-shortday-slot-clipping-toggle.sql` (v2.4 SHC-04). [VERIFIED: migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql]

---

## B. Service-Layer

### B.1 RebookingBatchService — Basic-Tier, Entity-Manager

**Trait-Signatur (minimal für Phase 54 — nur was der Planer braucht):**

```rust
// service/src/rebooking_batch.rs  (NEU)
#[async_trait]
pub trait RebookingBatchService: Send + Sync + 'static {
    type Context: Send;
    type Transaction: Clone + Send;

    async fn create(
        &self,
        entity: &RebookingBatch,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatch, ServiceError>;

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatch, ServiceError>;

    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatch>, ServiceError>;

    // Defer to Phase 55: update_state, list_pending, reject, approve
}
```

**Phase-54-Scope:** Nur CRUD + Skeleton. `RebookingBatchDao`-Implementierung, DI-Wiring, keine BL-Konsumenten. Methoden für Phase 55 (state-transitions, list_pending) werden im Trait bereits definiert (return `ServiceError::NotImplemented` oder im Trait-Kommentar „Phase 55"), damit Phase 55 keinen Breaking Change am Trait braucht.

**Deps (Basic-Tier):**
```
RebookingBatchDao: dao::RebookingBatchDao
PermissionService: service::PermissionService
TransactionDao: dao::TransactionDao
UuidService: service::uuid_service::UuidService
ClockService: service::clock::ClockService
```

Kein anderer Domain-Service als Dep — Basic-Tier-Konvention strikt eingehalten.

### B.2 VoluntaryStatsService — BL-Tier, HR-gated, Read-only

**Trait-Signatur:**

```rust
// service/src/voluntary_stats.rs  (NEU)
#[async_trait]
pub trait VoluntaryStatsService: Send + Sync + 'static {
    type Context: Send;
    type Transaction: Clone + Send;

    /// F1 + F2 kombiniert: Ist / Soll / Delta für ein Jahr.
    /// HR-gated: Non-HR erhält ServiceError::Forbidden ODER DTO mit None-Feldern.
    /// Empfehlung: HR-Gate im Service, None-Felder im DTO via REST-Handler.
    async fn get_voluntary_stats(
        &self,
        sales_person_id: &Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError>;
}

pub struct VoluntaryStats {
    /// F1: Ø freiwillige Stunden / Vertragswoche. None wenn Non-HR.
    pub ist_per_contract_week: Option<f32>,
    /// F2: Σ committed_voluntary × Wochen-in-Kraft (pro-rata). None wenn Non-HR.
    pub soll_total: Option<f32>,
    /// F2: Ist-Gesamt (Zähler F1). None wenn Non-HR.
    pub ist_total: Option<f32>,
    /// F2: Delta = Ist − Soll. None wenn Non-HR.
    pub delta: Option<f32>,
}
```

**Deps (BL-Tier):**
```
ExtraHoursService: ExtraHoursService  (find_by_iso_year)
EmployeeWorkDetailsService: EmployeeWorkDetailsService  (find_by_sales_person_id)
SalesPersonService: SalesPersonService  (get_by_id — für Existenzprüfung)
PermissionService: PermissionService
TransactionDao: TransactionDao
```

**HR-Gate-Implementierung:** `check_permission(HR_PRIVILEGE, ctx)` im Service. Bei fehlender Berechtigung: `ServiceError::Forbidden`. Im REST-Handler alternativ: `None`-Felder statt 403 (Präzedenz VAC-OFFSET-01 v1.8). **Empfehlung:** Service gibt bei Non-HR `VoluntaryStats { ist_per_contract_week: None, soll_total: None, ist_total: None, delta: None }` zurück (kein 403) — konsistenter mit DTO-Redact-Muster. Planer entscheidet.

### B.3 Pure Functions — Standort und Signaturen

**Standort:** `service_impl/src/reporting.rs` (Präzedenz: `committed_voluntary_for_calendar_week`, `apply_weekly_cap`, `find_working_hours_for_calendar_week` sind alle module-level pure fns in reporting.rs). Alternativ: eigenes Modul `service_impl/src/voluntary_stats_impl.rs` für Kohäsion. **Empfehlung: reporting.rs** — dort liegen bereits alle verwandten pure fns und die Tests können sie direkt importieren.

**Pure fn 1 — F1-Zähler:**

```rust
/// VOL-STAT-01: Summe der VolunteerWork-ExtraHours im ISO-Jahr,
/// MIT Filter source='manual' (D-54-DM-02 Rebooking-Neutralität).
/// Die Rows werden bereits gefiltert aus find_by_iso_year (via DAO WHERE-Klausel)
/// oder hier per Iterator-Filter wenn DAO noch keinen source-Filter hat.
pub fn voluntary_ist_total_for_year(
    extra_hours: &[ExtraHours],  // bereits by find_by_iso_year geladen
    year: u32,                    // für ISO-Jahres-Zugehörigkeit via as_shifty_week
) -> f32 {
    extra_hours
        .iter()
        .filter(|eh| {
            matches!(eh.category, ExtraHoursCategory::VolunteerWork)
                && eh.source == ExtraHoursSource::Manual  // D-54-DM-02
                && ShiftyDate::from(eh.date_time.date()).as_shifty_week().year == year
        })
        .map(|eh| eh.amount)
        .sum()
}
```

Hinweis: `ExtraHoursSource` als neues Enum in `service/src/extra_hours.rs` oder als `&str`-Vergleich. Da `find_by_iso_year` bereits ISO-Jahres-Filter hat, ist der year-Filter in der fn redundant — aber defensive Programmierung.

**Pure fn 2 — F1-Nenner (D-F1-01):**

```rust
/// D-F1-01: Anzahl ISO-Wochen im Jahr, in denen eine gültige working_hours-Row
/// existiert. expected_hours=0 zählt MIT.
pub fn contract_weeks_count(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    total_weeks_in_year: u8,  // time::util::weeks_in_year(year as i32)
) -> u32 {
    (1..=total_weeks_in_year)
        .filter(|&week| find_working_hours_for_calendar_week(working_hours, year, week).next().is_some())
        .count() as u32
}
```

**Pure fn 3 — F2-Soll pro-rata (D-F2-01):**

```rust
/// D-F2-01: committed_voluntary für eine ISO-Woche, pro-rata nach Tagen.
/// Iteriert Mo–So, sucht für jeden Tag die aktive working_hours-Row,
/// summiert `committed_voluntary × (1/7)`.
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32 {
    let Ok(monday) = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday) else {
        return 0.0;
    };
    (0i64..7)
        .filter_map(|offset| {
            let day = monday + time::Duration::days(offset);
            let shifty = ShiftyDate::from(day);
            let (d_year, d_week) = shifty.as_iso_year_and_week();  // analog _iso_year-Helper
            find_working_hours_for_calendar_week(working_hours, d_year, d_week as u8)
                .next()
                .map(|wh| wh.committed_voluntary / 7.0)
        })
        .sum()
}

/// F2-Soll total: Σ über alle ISO-Wochen des Jahres.
pub fn committed_voluntary_target_for_year(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
) -> f32 {
    let total_weeks = time::util::weeks_in_year(year as i32) as u8;
    (1..=total_weeks)
        .map(|week| committed_voluntary_prorata_for_week(working_hours, year, week))
        .sum()
}
```

**Hinweis zu `as_iso_year_and_week()`:** Die v2.5 WOP-Follow-up-#3 `_iso_year`-Helper in `shifty_utils` liefern die korrekte ISO-Jahres-Zuordnung für Tage nahe dem Jahreswechsel. Planer prüft, ob `ShiftyDate::as_shifty_week()` bereits ISO-konform ist (in `reporting.rs` ist `as_shifty_week()` überall für ISO-Wochen-Zuordnung genutzt, z.B. Zeile 1041, 1140). Falls ja: `ShiftyDate::from(day).as_shifty_week().year` und `.week` sind die korrekten Werte. [VERIFIED: service_impl/src/reporting.rs Zeilen 1041, 1140, 590]

### B.4 Marker-Filter auf DAO-Ebene vs. Service-Ebene

**Empfehlung: WHERE-Klausel im DAO** für den primären source-Filter.

Konkret: `find_by_iso_year` in `dao_impl_sqlite/src/extra_hours.rs` wird nicht angepasst (keine Breaking Change am bestehenden Interface). Stattdessen neues DAO-Interface `find_by_iso_year_manual_only` mit SQL-Klausel `AND source = 'manual'` ODER Iterator-Filter im Service auf dem Ergebnis von `find_by_iso_year`.

**Praktische Empfehlung (Phase 54 Pragmatismus):** Iterator-Filter im Service (`eh.source == 'manual'`). Reason: `find_by_iso_year` wird von anderen Konsumenten genutzt, ein neues DAO-Interface erhöht die Änderungsfläche. Der result-set ist klein (eine Person, ein Jahr, O(52) Rows). In Phase 55/56 kann ein dedizierter DAO-Query folgen wenn nötig.

Für das Iterator-Filter braucht `ExtraHours` in `service/src/extra_hours.rs` ein neues Feld `source: ExtraHoursSource` (Enum `Manual | Rebooking`). Der `ExtraHoursDb`-Struct in `dao_impl_sqlite/src/extra_hours.rs` bekommt `source: String`.

---

## C. REST + DTO

### C.1 Empfehlung: Dedizierter Endpoint

**Empfehlung: `GET /reporting/employee/{id}/voluntary-stats` als Subroute in `rest/src/report.rs`.**

Begründung:
- Präzedenz: `/{id}/weekly-statistics` (Phase 22, AVG-01) ist exakt analog — HR-only, separater Endpunkt, eigenes DTO `EmployeeWeeklyStatisticsTO`. [VERIFIED: rest/src/report.rs Zeilen 29, 162-196]
- `EmployeeReportTO` enthält bereits `volunteer_hours: f32` (nicht HR-only). Additives Hinzufügen von HR-only-Feldern (die `None` für Non-HR sind) würde das DTO semantisch inkonsistent machen.
- Separater Endpoint = separater Cache-Key im FE = einfacherer Loader.
- Kein Bloat auf dem hot-path `GET /reporting/employee/{id}`.

**Nachteil:** ein zusätzlicher HTTP-Request im FE. Tolerabel weil F1/F2 nur im Employee-Detail-Report angezeigt werden (kein Aufruf für alle Employees).

**Route-Ergänzung in `generate_route`:**

```rust
.route(
    "/{id}/voluntary-stats",
    get(get_voluntary_stats::<RestState>),
)
```

**Parameter:** `/{id}` (Path, Uuid) + `year` (Query, u32). Kein `until_week` nötig (F1/F2 sind Jahres-Aggregate).

### C.2 Neues DTO: VoluntaryStatsTO

```rust
// rest-types/src/lib.rs (additiv, neuer Struct)

/// Phase 54 F1+F2: Voluntary hours statistics for a sales person in a year.
/// All fields are HR-only (None for non-HR responses).
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct VoluntaryStatsTO {
    /// F1: Average voluntary hours per contract week. None if not HR.
    #[serde(default)]
    pub ist_per_contract_week: Option<f32>,
    /// F2: Target (Soll) = Σ committed_voluntary × weeks_active (pro-rata). None if not HR.
    #[serde(default)]
    pub soll_total: Option<f32>,
    /// F2: Actual (Ist) total = F1 numerator. None if not HR.
    #[serde(default)]
    pub ist_total: Option<f32>,
    /// F2: Delta = Ist − Soll. None if not HR.
    #[serde(default)]
    pub delta: Option<f32>,
    /// Number of contract weeks (F1 denominator). None if not HR.
    #[serde(default)]
    pub contract_weeks: Option<u32>,
}
```

`#[serde(default)]` auf allen Feldern garantiert wire-backward-compat (Konsumenten die ältere DTO-Versionen kennen deserialisieren problemlos). [VERIFIED: Muster aus rest-types/src/lib.rs Zeilen 539, 637, 641]

### C.3 HR-Only-Redaction im REST-Handler

```rust
pub async fn get_voluntary_stats<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<VoluntaryStatsRequest>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let stats = rest_state
                .voluntary_stats_service()
                .get_voluntary_stats(&sales_person_id, query.year, context.into(), None)
                .await?;
            let to = VoluntaryStatsTO::from(&stats);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

Die HR-Gate-Logik liegt im `VoluntaryStatsService` (None-Felder bei Non-HR), nicht im Handler.

---

## D. Frontend

### D.1 Neuer Loader

```rust
// shifty-dioxus/src/loader.rs (additiv)
pub async fn load_voluntary_stats(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
) -> Result<VoluntaryStatsTO, String> {
    // GET /reporting/employee/{id}/voluntary-stats?year={year}
}
```

**Separater Loader vs. Erweiterung `load_employee_report`:** Separater Loader empfohlen — decoupled, einfacher zu testen, FE kann ihn lazy nach dem Main-Report-Load aufrufen. Analog wie `load_weekly_statistics` bereits separiert ist.

### D.2 Neue Komponente

```
shifty-dioxus/src/component/voluntary_stats_row.rs
```

HR-Gate im FE: Prüfung `if stats.ist_per_contract_week.is_none() { return; }`. Keine Role-Prüfung im FE-Code nötig — Backend liefert None, FE rendert nicht.

**Row-Layout:** Drei Zellen in einer Zeile (Präzedenz AVG-01 v2.1 „drei Zellen in einer Zeile"). Labels: Ist / Soll / Δ. Werte mit 1-2 Nachkommastellen in Stunden (h). Delta negativ = rote Farbe.

### D.3 Dioxus.toml Proxy

```toml
# shifty-dioxus/Dioxus.toml — additiv
[[web.proxy]]
backend = "http://localhost:3000/reporting/employee"
```

ACHTUNG: Falls bereits ein `[[web.proxy]]` für `/reporting/employee` existiert, deckt er den neuen Subroute `/voluntary-stats` bereits ab (Prefix-Match). Planer prüft bestehende Proxy-Einträge vor dem Hinzufügen. [ASSUMED: genaue Proxy-Konfiguration nicht aus Dioxus.toml gelesen]

### D.4 i18n

Neue Keys in `shifty-dioxus/src/i18n/` (de/en/cs):

| Key | Deutsch | Englisch | Tschechisch |
|-----|---------|----------|-------------|
| `VoluntaryHoursIst` | „Freiwillig Ist / Woche" | „Voluntary Actual / Week" | „Dobrovolné Skutečné / Týden" |
| `VoluntaryHoursSoll` | „Freiwillig Soll" | „Voluntary Target" | „Dobrovolné Cíl" |
| `VoluntaryHoursDelta` | „Freiwillig Δ" | „Voluntary Delta" | „Dobrovolné Δ" |

[ASSUMED: Tschechische Übersetzung — Planer verifiziert mit vorhandener i18n-Infrastruktur]

---

## E. Property-Test-Design (VOL-ACCT-03)

### E.1 Framework und Standort

**Framework:** Fixture-Test (kein `proptest` crate). Präzedenz: `service_impl/src/test/reporting_avg_weekly.rs` (pure-fn-Test ohne Mocks, vollständig data-driven) und `booking_information_vaa.rs` (Mock-basierter Service-Test).

**Datei:** `service_impl/src/test/voluntary_stats.rs` (NEU)

### E.2 Test-Setup

Für VOL-ACCT-03 (Rebooking-Neutralitäts-Property-Test):

1. Basis-Fixture: `SalesPerson` + `EmployeeWorkDetails` (committed_voluntary=4.0, expected_hours=40.0) für 10 ISO-Wochen in 2026.
2. `ExtraHours`-Rows: 5 Rows vom Typ `VolunteerWork`, `source='manual'`, Σ = 20h.
3. Assert Baseline: `voluntary_ist_total_for_year(...)` = 20.0, `contract_weeks_count(...)` = 10, F1 = 2.0h/Woche, F2-Soll = 10 × 4.0 = 40.0.
4. **Inject Rebooking-Pair:** ExtraHours Row `source='rebooking'`, Kategorie `VolunteerWork`, hours=−4.0 + ExtraHours Row `source='rebooking'`, Kategorie `ExtraWork`, hours=+4.0.
5. Assert nach Inject: F1-Ist (nur `source='manual'`) = **unverändert 20.0**, F2-Soll = **unverändert 40.0**.

### E.3 Test-Scope (pure-fn vs. Integration)

**Phase 54 Scope:** Pure-fn-Tests auf den Funktionen `voluntary_ist_total_for_year`, `contract_weeks_count`, `committed_voluntary_prorata_for_week`, `committed_voluntary_target_for_year`. Kein In-Memory-SQLite nötig — die pure fns operieren auf Slices.

**Für VOL-ACCT-03:** Test instanziiert `Vec<ExtraHours>` direkt (mit `source`-Feld), kein Mock. Assertions:

```rust
#[test]
fn rebooking_pair_does_not_affect_f1_ist() {
    let mut extra_hours = make_manual_volunteer_hours();  // 5 Rows, source=manual, Σ=20h
    let before = voluntary_ist_total_for_year(&extra_hours, 2026);
    // inject rebooking pair
    extra_hours.push(make_rebooking_eh(ExtraHoursCategory::VolunteerWork, -4.0)); // source=rebooking
    extra_hours.push(make_rebooking_eh(ExtraHoursCategory::ExtraWork, 4.0));       // source=rebooking
    let after = voluntary_ist_total_for_year(&extra_hours, 2026);
    assert!((before - after).abs() < f32::EPSILON, "F1 must be invariant: {before} != {after}");
}

#[test]
fn rebooking_pair_does_not_affect_f2_soll() {
    let working_hours = make_working_hours_10_weeks();
    let before = committed_voluntary_target_for_year(&working_hours, 2026);
    // F2-Soll hängt nicht von ExtraHours ab → kein Rebooking-Inject nötig
    // Test prüft nur: Soll-Berechnung ist stabil und korrekt
    let expected_soll = 10.0 * 4.0;  // 10 Vertragswochen × 4.0h committed_voluntary
    assert!((before - expected_soll).abs() < 0.01, "F2-Soll: {before} != {expected_soll}");
}
```

### E.4 Randfälle als separate Tests

```
test_f1_zero_expected_hours_counts_in_denominator()    -- D-F1-01: expected_hours=0 zählt
test_f2_midweek_contract_change_prorata()               -- D-F2-01: Wechsel Mittwoch
test_f2_iso_week_53_year_boundary()                     -- ISO-Woche 53 / Woche 1 Straddle
test_f1_empty_working_hours_zero()                      -- keine Vertragswochen → Nenner=0, Ist=0.0
test_f2_soll_no_committed_voluntary()                   -- committed_voluntary=0 → Soll=0
```

---

## F. DI-Wiring in shifty_bin/src/main.rs

### Reihenfolge

```
Basic-Services-Wave:
  ... (bestehende Basic-Services) ...
  RebookingBatchDao::new(pool.clone())                           // DAO
  RebookingBatchService::new(deps mit RebookingBatchDao, ...)    // Basic

BL-Services-Wave (nach ExtraHoursService, WorkingHoursService):
  VoluntaryStatsService::new(deps mit ExtraHoursService, EmployeeWorkDetailsService, ...)  // BL

REST-State:
  RestStateImpl::new(..., voluntary_stats_service, ...)

Boot (am Ende von main()):
  // kein neuer Scheduler — der Toggle-Seed wird nur gelegt, kein Consumer in Phase 54
```

**`VoluntaryStatsService`-Deps:** `ExtraHoursService` + `EmployeeWorkDetailsService` + `SalesPersonService` + `PermissionService` + `TransactionDao`.

**Zyklenprüfung:** `VoluntaryStatsService` (BL) → `ExtraHoursService` (Basic) + `EmployeeWorkDetailsService` (Basic) + `SalesPersonService` (Basic) → keine Zyklen. Analog `BookingInformationService → ReportingService` (BL→BL ohne Zyklus). [VERIFIED: ARCHITECTURE.md Zyklen-Check]

---

## G. Docs-Freshness-Gate

Folgende Dokumente MÜSSEN im selben Commit wie der Code-Diff aktualisiert werden (Memory `feedback_docs_always_current_no_followup`):

| Trigger-Datei | Docs-Update (EN + DE) | Neuer Inhalt |
|---------------|----------------------|--------------|
| `migrations/sqlite/20260707000000_*.sql` | `docs/architecture/03-data-model.md` + `_de.md` | Tables `rebooking_batch`, `rebooking_batch_entry`; Spalte `source` auf `extra_hours`; Toggle-Seed |
| `migrations/sqlite/20260707000000_*.sql` | `docs/architecture/diagrams/db-schema-er.mmd` | Neue Nodes + Relations für beide Tabellen |
| `service/src/rebooking_batch.rs` (Trait) | `docs/architecture/02-service-tiers.md` + `_de.md` | `RebookingBatchService` (Basic) + `VoluntaryStatsService` (BL) |
| `service/src/rebooking_batch.rs` | `docs/architecture/diagrams/service-graph-runtime.mmd` | Neue Nodes + Edges |
| `service/src/voluntary_stats.rs` | `docs/features/F07-reporting-balance.md` + `_de.md` | Notiz: Balance-Chain filtert `source='rebooking'` (Schreiber ab Phase 55) |
| `service/src/voluntary_stats.rs` | `docs/features/F08-billing-period.md` + `_de.md` | Notiz: ExtraWork-aus-Rebooking ab Phase 55; Snapshot-Version 12 unverändert in Phase 54 |
| NEU | `docs/features/F14-rebooking.md` **(NEU)** | Rebooking-Domäne: F1/F2/F3/F4/F5-Übersicht, Marker-Filter-Regel, Batch-Struktur |
| NEU | `docs/features/F14-rebooking_de.md` **(NEU)** | Gleicher Inhalt auf Deutsch |

**Keine Umlaute in Dateinamen** (Memory `feedback_no_umlauts_in_paths`). `F14-rebooking.md` und `F14-rebooking_de.md` sind korrekt.

Höchste verfügbare Feature-Doc-Nummer: F13 (`F13-system-infrastructure.md`). F14 ist der nächste freie Slot. [VERIFIED: ls docs/features/]

---

## Validation Architecture

### Test-Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` (kein proptest-Crate) |
| Config file | `service_impl/src/test/mod.rs` (add `mod voluntary_stats;`) |
| Quick run command | `cargo test -p service_impl voluntary_stats` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test-Typ | Automated Command | Datei existiert? |
|--------|----------|----------|-------------------|-----------------|
| VOL-STAT-01 | F1-Ist-Berechnung (Zähler + Nenner) | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::f1` | Nein — Wave 0 |
| VOL-STAT-01 | Filter `source='manual'` (D-54-DM-02) | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::rebooking_pair` | Nein — Wave 0 |
| VOL-STAT-02 | Non-HR → None-Felder im DTO | Integration (Service + Mock) | `cargo test -p service_impl voluntary_stats::hr_redaction` | Nein — Wave 0 |
| VOL-STAT-02 | HR → Some-Felder im DTO | Integration (Service + Mock) | `cargo test -p service_impl voluntary_stats::hr_some` | Nein — Wave 0 |
| VOL-ACCT-01 | F2-Soll pro-rata bei Mid-Week-Wechsel | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::f2_midweek` | Nein — Wave 0 |
| VOL-ACCT-01 | F2-Soll ISO-Woche 53/1 Straddle | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::f2_year_boundary` | Nein — Wave 0 |
| VOL-ACCT-01 | F2-Soll bei expected_hours=0 (D-F1-01) | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::f1_zero_expected` | Nein — Wave 0 |
| VOL-ACCT-03 | Rebooking-Neutralität F1 invariant | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::rebooking_f1_invariant` | Nein — Wave 0 |
| VOL-ACCT-03 | Rebooking-Neutralität F2-Soll invariant | Unit (pure fn) | `cargo test -p service_impl voluntary_stats::rebooking_f2_soll_invariant` | Nein — Wave 0 |

### Decision-Coverage-Assertions

| Decision | Assertion | Test |
|----------|-----------|------|
| D-F1-01 (expected_hours=0 zählt) | `contract_weeks_count` zählt Woche mit expected_hours=0 im Nenner | `voluntary_stats::f1_zero_expected_hours_counts_in_denominator` |
| D-F2-01 (pro-rata) | `committed_voluntary_prorata_for_week` mit Mittwoch-Wechsel liefert ≠ latest-active | `voluntary_stats::f2_midweek_prorata_vs_latest_active` |
| D-54-DM-01 (UNIQUE global) | UNIQUE-Index existiert; INSERT für gleiche (sp_id, year, week) schlägt fehl | Migration-Test oder manuell via sqlx-Roundtrip; MANUAL-VERIFY |
| D-54-DM-02 (source-Filter) | source='rebooking' ExtraHours erscheinen NICHT in F1-Ist | `voluntary_stats::rebooking_pair_does_not_affect_f1_ist` |

### Randfälle mit dedizierten Tests

| Randfall | Test-Name | Was wird geprüft |
|----------|-----------|-----------------|
| ISO-Woche 53 / Woche 1 Jahresübergang | `f2_iso_week_53_year_boundary` | committed_voluntary_prorata verwendet `as_shifty_week()` korrekt (kein Gregorianisches Datum) |
| Vertrag mit expected_hours=0 | `f1_zero_expected_hours_counts_in_denominator` | D-F1-01: Nenner wird +1 obwohl expected=0 |
| Mid-Week-Vertragswechsel (Mittwoch) | `f2_midweek_contract_change_prorata` | Wert = 3/7 × altes + 4/7 × neues committed_voluntary |
| Toggle-Seed-Idempotenz | Migrations-Direkttest mit `INSERT OR IGNORE` | Zweites `sqlx migrate run` ändert nichts |
| HR-Only-Redaction (Non-HR-Auth → None) | `hr_redaction_non_hr_gets_none` | Service gibt `VoluntaryStats { ist_per_contract_week: None, ... }` |
| HR-Auth → Some | `hr_gets_some_values` | Service gibt befülltes `VoluntaryStats` |

### Wave 0 Lücken

- [ ] `service_impl/src/test/voluntary_stats.rs` — alle oben gelisteten Tests
- [ ] `service/src/extra_hours.rs` — `source: ExtraHoursSource`-Feld auf `ExtraHours`-Struct + `ExtraHoursSource`-Enum
- [ ] `dao_impl_sqlite/src/extra_hours.rs` — `source: String`-Feld in `ExtraHoursDb` + SELECT/INSERT-Anpassung
- [ ] `cargo sqlx prepare --workspace` nach Migration + DAO-Änderung (CI SQLX_OFFLINE)

### Sampling-Rate

- **Per-Task-Commit:** `cargo test -p service_impl voluntary_stats`
- **Per-Wave-Merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase-Gate:** Full Suite grün + Docs-Freshness verified vor `/gsd-verify-work`

---

## Präzedenzen (Datei-Pfade für Planer)

| Was | Datei | Relevant für |
|-----|-------|-------------|
| Pure fns in reporting.rs | `service_impl/src/reporting.rs` Zeilen 87–119 | Standort + Signatur für neue pure fns |
| committed_voluntary_for_calendar_week | `service_impl/src/reporting.rs` Zeilen 111–119 | Direktes Vorbild für F2-Soll-Helper |
| apply_weekly_cap | `service_impl/src/reporting.rs` Zeilen 127–140 | Pure fn Signatur Pattern |
| find_working_hours_for_calendar_week | `service_impl/src/reporting.rs` Zeilen 87–96 | Basis-Iterator für Vertragswochen-Lookup |
| ExtraHoursCategory::VolunteerWork | `service/src/extra_hours.rs` Zeile 48 | Kategorie-Konstante |
| find_by_iso_year | `service/src/extra_hours.rs` Zeilen 232–237 | Einzige ISO-year ExtraHours-Methode |
| Authentication::Full | `service_impl/src/extra_hours.rs` Zeilen 50–59 | Internal-caller-Pattern |
| Toggle-Seed | `migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql` | Exaktes SQL-Muster |
| HR-Gate check_permission | `service_impl/src/reporting.rs` Zeile 865, 1296, 1661, 1695 | Permission-Check-Muster |
| Additive EmployeeReportTO-Felder | `rest-types/src/lib.rs` Zeile 539 (`volunteer_hours` mit `#[serde(default)]`) | DTO-Additivitäts-Muster |
| ShortEmployeeReportTO | `rest-types/src/lib.rs` Zeilen 373–395 | Struktur der Short-Report-DTOs |
| Reporting-REST-Handler weekly-statistics | `rest/src/report.rs` Zeilen 162–196 | Exaktes Vorbild für neuen voluntary-stats-Handler |
| EmployeeReportTO from-impl | `rest-types/src/lib.rs` Zeilen 557–596 | DTO-Mapping-Muster |
| is_hr-Gate im FE | `shifty-dioxus/src/page/billing_periods.rs` Zeilen 39–42 | HR-Gate-Check-Pattern |
| Pure-fn-Test-Muster | `service_impl/src/test/reporting_avg_weekly.rs` | Test ohne Mocks, nur Fixture-Structs |
| Mock-basierter Service-Test | `service_impl/src/test/booking_information_vaa.rs` | Test mit MockPermissionService etc. |
| CURRENT_SNAPSHOT_SCHEMA_VERSION = 12 | `service_impl/src/billing_period_report.rs` Zeile 117 | Bestätigung: kein Bump in Phase 54 |

---

## Landmines / Randfälle (PITFALLS.md-Referenzen)

### Pitfall 1 (Doppel-Zählung) — KRITISCH, Phase 54 schließt ihn

Die `source`-Spalte auf `extra_hours` + Filter in allen Aggregat-Readern ist der primäre Schutz. In Phase 54 hat `VoluntaryStatsService` als einziger neuer Konsument diese Pflicht. Balance-Chain (`reporting.rs`) und `booking_information.rs` müssen in Phase 54 NOCH NICHT angepasst werden (kein Rebooking-Schreiber existiert), aber die Migration legt die Infrastruktur. **Planer-Note:** Phase-55-Tasks müssen das Marker-Filter-Pattern auf `reporting.rs` + `booking_information.rs` anwenden.

### Pitfall 8 (HR-Only DTO-Redaction) — In Phase 54 relevant

Alle F1/F2-Felder im DTO MÜSSEN `Option<f32>` sein. Non-HR-User dürfen auf API-Ebene `null` sehen. Die REST-Test-Verifikation muss beide Flows prüfen:
- HR-Token → `VoluntaryStatsTO` mit `Some`-Werten
- Non-HR-Token → `VoluntaryStatsTO` mit `None`-Werten in allen Feldern

### Pitfall 9 (shared pure fn) — Phase 54 löst ihn

Durch eine einzige pure fn `voluntary_ist_total_for_year` (statt bspw. zweier separater Loops in F1 und F2) ist F1-Ist = F2-Ist per Konstruktion gleich. Kein Drift möglich. Test: beide rufen dieselbe Funktion auf.

### Pitfall 10 (Cap-Semantik bei Wochenmitte-Vertragswechsel) — D-F2-01 pinnt die Lösung

D-F2-01 entscheidet: pro-rata. Der neue Helper `committed_voluntary_prorata_for_week` implementiert diesen Entscheid. Planer muss einen dedizierten Test mit exaktem Mittwoch-Wechsel-Szenario (3/7 altes + 4/7 neues committed_voluntary) schreiben.

### Pitfall 13 (Docs-Drift) — Gate ist im selben Commit

Docs-Freshness (CLAUDE.md §"Docs-Freshness-Gate") triggert auf: `migrations/sqlite/*.sql`, `service/**/*.rs` (neue Traits), `dao/**/*.rs`. Alle drei Trigger feuern in Phase 54. F14-rebooking.md (EN + DE) NEU; 02-service-tiers + 03-data-model + Diagramme UPDATE.

### Randfall: ISO-Woche 53/1 bei D-F2-01 pro-rata

`time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)` für Woche 53 schlägt fehl wenn das Jahr nur 52 Wochen hat. Planer muss den `committed_voluntary_prorata_for_week`-Helper mit `time::util::weeks_in_year(year as i32)` absichern:

```rust
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32 {
    // Guard: ISO-Woche existiert für dieses Jahr
    if week as u8 > time::util::weeks_in_year(year as i32) {
        return 0.0;
    }
    // ... rest
}
```

### Randfall: sqlx-prepare nach source-Spalte

Jede neue `query!`/`query_as!`, die `extra_hours` liest, muss nach der Migration neu compiliert werden. CI läuft SQLX_OFFLINE=true. Ohne `cargo sqlx prepare --workspace` + committed `.sqlx/*.json` schlägt die CI fehl. [VERIFIED: MEMORY reference_sqlx_prepare_after_new_query]

### Randfall: EmployeeWorkDetails from_for_week → kein Lücken-Tag

`find_working_hours_for_calendar_week` liefert Rows, die (year, week) im Bereich `[from_year/from_calendar_week, to_year/to_calendar_week]` haben. Für D-F2-01 pro-rata braucht der Helper nur die aktive Row für den jeweiligen Tag — kein „Lücken-Tag" (kein Vertrag) → Beitrag 0. Planer stellt sicher: `filter_map(|_| ...)` gibt `None` zurück wenn kein Contract-Row → kein Beitrag.

### Randfall: Clippy

`cargo clippy --workspace -- -D warnings` MUSS grün sein (CLAUDE.md §"Clippy ist ein hartes Gate"). Dioxus-Workspace ist separat und von CI-Clippy ausgeschlossen (Memory `reference_dioxus_clippy_not_gated`). Backend-Clippy ist Pflicht.

---

## Divergenzen zu vorhandenen Research-Files

Keine inhaltlichen Divergenzen. Diese Phase-54-spezifische Research schärft und konkretisiert, was in ARCHITECTURE.md und FEATURES.md als Optionen beschrieben war:

1. **REST-Route:** ARCHITECTURE.md listet beide Optionen (additiv vs. dediziert). Diese Research schließt auf dedizierten Endpoint basierend auf Präzedenz `/{id}/weekly-statistics` in `report.rs`.

2. **Marker-Filter in DAO vs. Service:** CONTEXT.md ließ offen. Diese Research empfiehlt Iterator-Filter im Service für Phase 54 (pragmatisch: kleines result-set, kein Breaking Change am bestehenden DAO-Interface).

3. **ExtraHoursSource als Typ:** CONTEXT.md beschreibt `source: TEXT` in der DB. Diese Research konkretisiert: neues Enum `ExtraHoursSource { Manual, Rebooking }` im Service-Layer mit entsprechender DB-Konversion im DAO-Layer (analog `ExtraHoursCategoryEntity`-Pattern).

4. **`committed_voluntary_prorata_for_week` als eigene pure fn (nicht inline):** D-F2-01 verlangt neue Aggregation. Diese Research empfiehlt explizite pure fn in reporting.rs statt inline-Code in `committed_voluntary_target_for_year` — bessere Testbarkeit des Grenzfalls.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein | kein neuer Auth-Flow |
| V4 Access Control | ja | `check_permission(HR_PRIVILEGE, ctx)` im Service; `Option<f32> = None` für Non-HR |
| V5 Input Validation | ja | `year: u32` via Axum-Query-Deserialisierung; sales_person_id via Uuid-Parse-Guard |

---

## Sources

### PRIMARY (HIGH confidence — direkt aus Repo gelesen)

- `service_impl/src/reporting.rs` — pure-fn-Standort, `find_working_hours_for_calendar_week`, `committed_voluntary_for_calendar_week`, ShiftyWeek-Nutzung
- `rest/src/report.rs` — exaktes Vorbild `/{id}/weekly-statistics`-Endpoint
- `rest-types/src/lib.rs` — `EmployeeReportTO`, `ShortEmployeeReportTO`, `VoluntaryStatsTO`-Positionierung, `#[serde(default)]`-Muster
- `service/src/extra_hours.rs` — `ExtraHoursCategory::VolunteerWork`, `find_by_iso_year`
- `dao_impl_sqlite/src/extra_hours.rs` — `ExtraHoursDb`-Struct, bestehende Spalten
- `service_impl/src/billing_period_report.rs` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
- `migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql` — Toggle-Seed-Muster
- `service_impl/src/test/reporting_avg_weekly.rs` — pure-fn-Test-Muster
- `service_impl/src/test/booking_information_vaa.rs` — Mock-basierter Service-Test-Muster
- `service_impl/src/extra_hours.rs` — `Authentication::Full` internal-caller pattern
- `shifty-dioxus/src/page/billing_periods.rs` — `is_hr`-Gate im FE
- `docs/features/` — F13 ist höchste Nummer, F14 ist nächster freier Slot
- `.planning/phases/54-data-model-voluntary-stats/54-CONTEXT.md` — alle locked decisions
- `.planning/research/PITFALLS.md` — Pitfalls 1, 8, 9, 10, 13

### SECONDARY (HIGH confidence — MEMORY.md-verankert)

- `reference_sqlx_prepare_after_new_query` — sqlx-prepare-Gate nach neuen Queries
- `feedback_docs_always_current_no_followup` — Docs im gleichen Commit
- `feedback_dioxus_proxy_for_new_backend_endpoints` — Dioxus.toml proxy Pflicht
- `reference_dioxus_clippy_not_gated` — Dioxus Clippy separat, Backend-Clippy hart
- `feedback_no_umlauts_in_paths` — keine Umlaute in Dateinamen

---

## RESEARCH COMPLETE

**Phase:** 54 — Data-Model + Voluntary Statistics (F1 + F2)
**Confidence:** HIGH

### Key Findings (aufgelöste Kernfragen)

- **Migrations-Reihenfolge** aufgelöst: drei separate Dateien (Tabellen → source-Spalte → Toggle-Seed), exakt nach v2.4-SHC-04-Muster.
- **D-F2-01 pro-rata** konkretisiert: neuer Helper `committed_voluntary_prorata_for_week` (Mo–So-Iteration mit `time::Date::from_iso_week_date`), Standort `service_impl/src/reporting.rs`.
- **REST-Route** aufgelöst: dedizierter Endpoint `GET /{id}/voluntary-stats` analog `/{id}/weekly-statistics` (Präzedenz in `rest/src/report.rs`).
- **Filter-Strategie** (D-54-DM-02): Iterator-Filter im Service für Phase 54 (keine Breaking DAO-Change); SQL-WHERE in Phase 55/56 falls Performance nötig.
- **Property-Test VOL-ACCT-03**: pure-fn-Test in `service_impl/src/test/voluntary_stats.rs`, kein proptest-Crate, direkte Fixture-Manipulation.

### Noch offene Sub-Fragen

- **Dioxus.toml proxy:** Bestehende Proxy-Einträge müssen vor dem Hinzufügen geprüft werden (nicht aus Dioxus.toml gelesen in dieser Research).
- **Tschechische i18n-Labels:** `[ASSUMED]` — Planer verifiziert mit nativer Übersetzung.
- **iso_year/iso_week NULL-Fähigkeit auf rebooking_batch:** Planer entscheidet ob `iso_year INTEGER NOT NULL` oder `INTEGER` (nullable) besser für das Phase-55-Skeleton ist.
- **ExtraHoursSource als Enum vs. String-Konstante:** Planer entscheidet; Enum ist typsicherer, String-Konstante hat weniger Boilerplate. Beide funktionieren.
