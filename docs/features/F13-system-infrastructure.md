# Feature: System-Infrastruktur (Feature Flags, Toggles, Scheduler, Clock, UUID, Shortday-Gate, Config)

> **Kurzform:** Querschnitts-Services, die weder Domänen-Entities pflegen noch
> User-Workflows exponieren, sondern die Plattform tragen: zwei Schaltmechanismen
> (Feature Flags & Toggles), ein Cron-Runner (Scheduler), Test-Abstraktionen
> (Clock, UUID), ein zentrales Stichtag-Gate (`shortday_gate`) und Env-Config.

**Cluster-ID:** F13
**Status:** produktiv
**Erstmalig eingeführt:** ursprünglich mit dem Backend-Kern; nennenswert
erweitert in Milestone v1.7 (Toggle-Values, HCFG-02), Phase 2 (`feature_flag`),
Phase 24 (`paid_limit_hard_enforcement`), Phase 25 (`holiday_auto_credit`),
Phase 48 (PDF-Scheduler) und Phase 51 (Toggle Full-Bypass + `shortday_gate`).
**Zuständige Crates:** `service::{feature_flag, toggle, scheduler, clock,
uuid_service, config}`, `service_impl::{…, shortday_gate}`, `dao::{feature_flag,
toggle}`, `dao_impl_sqlite::{feature_flag, toggle}`, `rest::{feature_flag,
toggle}`.

---

## 1. Was ist das? (Fachlich)

Fachlich sichtbar wird von diesem Cluster im UI nur ein Streifen: die Karten
auf `/settings` (Paid-Limit-Enforcement, Holiday-Auto-Credit-Stichtag,
Short-Day-Slot-Clipping-Stichtag). Alles andere ist Infrastruktur — aber
Infrastruktur, die *fachlich Konsequenzen hat*, weil sie darüber entscheidet,
**ob** und **ab welchem Datum** eine neue Regel überhaupt greift.

Sechs Sub-Services teilen sich die Rolle:

- **Feature Flags** — Boolean-Schalter für **Architektur-/Migrations-Cutover**
  (z. B. `absence_range_source_active` in Phase 2: schaltet die
  Vacation/Sick/UnpaidLeave-Berechnung von `extra_hours` auf `absence_period`
  um). Nicht user-facing, wird beim Cutover atomar per Migration geflippt.
- **Toggles** — User-facing Schalter mit optionalem **String-Wert** (typisch
  ein ISO-Datum). Werden über `/settings` von Admins verwaltet und
  implementieren Stichtag-Rollouts: "ab dem 2026-08-01 werden Slots am
  ShortDay geklippt".
- **Scheduler** — Cron-Runner, der stündlich `update_carryover_all_employees`
  für Vor- und aktuelles Jahr ausführt, damit Balance-Reports schnell bleiben.
- **Clock / UUID** — Zwei Ein-Methoden-Traits, die Systemzeit bzw.
  UUID-Erzeugung hinter ein `mockall`-fähiges Interface stellen. Kein
  Domänen-Fach, reine Test-Abstraktion.
- **Config** — Liest zur Startup-Zeit `TIMEZONE` und `ICAL_LABEL` aus Env-Vars.
- **Shortday-Gate** (`service_impl/src/shortday_gate.rs`) — Ein
  Helfer-Modul (kein Service), das den `shortday_slot_clipping_active_from`-
  Toggle liest, das Ergebnis parst und den zentralen Clip-Algorithmus für alle
  vier Aggregat-Ketten (Block, Shiftplan, BookingInformation, ShiftplanReport)
  bereitstellt.

**Beispiel-Workflow aus User-Sicht (nur was sichtbar ist):**

1. Admin öffnet `/settings`.
2. Karte 1 (Paid-Limit): Button-Toggle zwischen "hard" und "soft" Enforcement
   — flippt den boolean Toggle `paid_limit_hard_enforcement`.
3. Karte 2 / 2b: ISO-Datum eintippen und speichern — schreibt den Datums-Wert
   in `toggle.value` für `holiday_auto_credit` bzw.
   `shortday_slot_clipping_active_from`.
4. Alle anderen Aggregate lesen den Toggle bei jedem Read und passen ihr
   Verhalten datumsabhängig an.

## 2. Fachliche Regeln

### 2.1 Feature Flags vs Toggles — die Trennung

Das ist die einzige Regel, die man verinnerlichen muss, um diesen Cluster zu
verstehen. Die Trennung ist **explizit** und in
`migrations/sqlite/20260501000000_add-feature-flag-table.sql:1-4`
begründet:

> "Bewusst KEIN Reuse von toggle/ToggleService — semantische Trennung:
> Feature-Flags sind Architektur/Migrations-Schalter, Toggles sind
> User-Toggles."

| Achse | `feature_flag` | `toggle` |
| --- | --- | --- |
| Zweck | Cutover / Migration / Architektur | User-facing Rollouts |
| Zielgruppe | Deployment / Backend-intern | End-User (Admin im UI) |
| Wert | nur `bool` (`enabled`) | `bool` + optionales `value: TEXT` |
| Gruppierung | keine | `toggle_group` + Junction-Tabelle |
| Stichtag-Semantik | keine — flippt atomar | ISO-Datum in `value` = Cutoff |
| Admin-Privileg | `feature_flag_admin` | `toggle_admin` |
| Read-Schreibfluss REST | nur `GET /feature-flag/{key}` (Read-only) | volles CRUD (`/toggle`, `/toggle-group`) |
| Write-Weg | Migration (`INSERT` in Seed-SQL) oder `set()` mit Admin-Privileg — kein REST-PUT | REST-PUT `/toggle/{name}/enable\|disable\|value` |
| Auth-Bypass für Reads | `Authentication::Full` bypasst User-Check (`feature_flag.rs:36-42`) | dito, aber erst seit Phase 51 (`toggle.rs:46-51`) |
| Typische Beispiele | `absence_range_source_active` | `paid_limit_hard_enforcement`, `holiday_auto_credit`, `shortday_slot_clipping_active_from` |

**Konsequenz:** Neue User-facing Rollout-Schalter kommen als Toggle. Neue
Migrations-/Architektur-Umschalter (die man in Zukunft entfernen will, sobald
der Cutover abgeschlossen ist) kommen als Feature-Flag.

**[Zu prüfen]** Ob es außer `absence_range_source_active` (Phase 2) weitere,
aktiv gelesene Feature-Flags gibt — der Grep über `service_impl/src/*.rs`
(exklusive `feature_flag.rs` selbst) findet aktuell keinen Reader. Der Flag
existiert also als Cutover-Marker; das Service-API ist bereitgestellt, aber
noch keine Konsumkette liest ihn zur Laufzeit.

### 2.2 Toggle-Semantik

- **Fail-safe Default:** Unbekannter Toggle-Name → `is_enabled → false`,
  `get_toggle_value → None` (`ToggleDao`, siehe DAO-Impl).
- **Value-Semantik:** `value` ist zweckoffener `TEXT`. Praxis: ISO-Datum
  (`YYYY-MM-DD`) mit Semantik "aktiv ab diesem Datum inklusiv".
- **REST-Validierung:** `PUT /toggle/{name}/value` validiert das
  ISO-Datumsformat vor dem Persist (`rest/src/toggle.rs:350-368`).
- **Set-Value als Convenience:** `PUT` auf `/value` setzt implizit `enabled=1`
  (siehe DAO-Trigger-Verhalten in `dao_impl_sqlite/src/toggle.rs`
  — **[Zu prüfen]** in der Impl konkret); `DELETE` auf `/value` setzt
  `enabled=0` und `value=NULL`. Der Handler-Kommentar
  (`rest/src/toggle.rs:337`) beschreibt "Toggle value set; toggle enabled".
- **Read-Ops akzeptieren `Authentication::Full`** (Phase 51, siehe Kap. 7).

### 2.3 Feature-Flag-Semantik

- **UPDATE-only DAO:** Die Migration MUSS jeden bekannten Key seeden. Ein
  `INSERT` durch die App gibt es nicht (`dao/src/feature_flag.rs:26-32`).
- **Fail-safe:** Unbekannter Key → `is_enabled → false`
  (`dao/src/feature_flag.rs:17`).
- **Auth-Read:** Jeder authentifizierte User darf lesen. `Full` passiert
  ohne User-ID-Check (`service_impl/src/feature_flag.rs:36-42`).
- **Kein REST-Write:** Der einzige exponierte REST-Endpoint ist
  `GET /feature-flag/{key}` (`rest/src/feature_flag.rs:34-36`). Writes gehen
  ausschließlich über den Service-Trait mit `FEATURE_FLAG_ADMIN_PRIVILEGE` —
  im Backend heißt das "per Migration + Deployment", nicht "per HTTP".

### 2.4 Scheduler-Regeln

- **Cron-Expression Hardcoded:** `SchedulerServiceImpl::start()` schedulet den
  Carryover-Job mit `"0 * * * * *"` (jede Minute des 0. Sekunden-Ticks —
  effektiv jede Minute) (`service_impl/src/scheduler.rs:45`). Kein Env-Override
  im aktuellen Code.
- **Zwei Läufe pro Tick:** Erst `year-1`, dann `year` — jeweils mit
  `Authentication::Full`, `tx=None` (`scheduler.rs:59-70`).
- **Fehler-Isolation:** Beide Läufe werden in eigenen `if let Err(e) = ...`
  gewrappt und nur geloggt (`error!`), nicht propagiert — der nächste Tick
  versucht es neu.
- **PDF-Export-Scheduler** (`service_impl/src/pdf_export_scheduler.rs`) ist ein
  separater Scheduler für den PDF-Batch-Export (siehe F11 Export); er läuft
  neben dem Carryover-Scheduler und wird ebenfalls in `main.rs:1441-1445`
  gestartet.

### 2.5 Clock- und UUID-Service-Regeln

- **Clock:** UTC. `time_now()`, `date_now()`, `date_time_now()` liefern alle
  einen UTC-basierten `time::OffsetDateTime` und werfen die Zone weg
  (`service_impl/src/clock.rs:6-14`). **[Zu prüfen]** ob das für alle
  Konsumenten okay ist oder ob Berichte bereits lokale Zeit erwarten — dies
  ist die Standard-Falle für Randfall Zeit/Zeitzone (siehe
  `docs/domain/edge-cases.md#4-zeit--zeitzone`).
- **UUID:** `Uuid::new_v4()` — reine V4 Random-UUIDs. Das `usage`-Argument
  wird vom Prod-Impl verworfen (`service_impl/src/uuid_service.rs:6-8`); es
  existiert für Tests, damit Mocks pro Aufrufsite unterschiedliche UUIDs
  liefern können.

### 2.6 Shortday-Gate-Regeln (Phase 51, D-51-07)

- **Zentraler Ort:** `service_impl/src/shortday_gate.rs`. Kein eigener Service,
  kein DAO — ein Modul mit reinen Funktionen plus einem crate-lokalen Helper
  `read_active_from`, der den Toggle liest.
- **Toggle-Name-Konstante:** `TOGGLE_NAME =
  "shortday_slot_clipping_active_from"` (`shortday_gate.rs:42`).
- **Parse-Toleranz:** `parse_active_from(None|Some("")|Some(bad)) → None`
  (`shortday_gate.rs:51-57`). Keine Panics bei defekten Werten.
- **`Unauthorized`-Toleranz:** `read_active_from` schluckt `Unauthorized`
  ausdrücklich in `Ok(None)` — für Legacy-Setups und Mock-Auth-Tests
  (`shortday_gate.rs:105-117`). Nach Phase 51 tritt der Fall im Prod-Pfad
  nicht mehr auf, weil `ToggleService` `Full` durchlässt; die Toleranz bleibt
  als Sicherheitsgurt.
- **Modus-Split:** `ShortdayMode::{Modern, Legacy}` — Modern
  (Chain A'/D: `block.rs`, `shiftplan_report.rs`) behält bei Gate-aus den
  rohen Slot. Legacy (Chain B/C: `shiftplan.rs`, `booking_information.rs`)
  reproduziert Pre-Phase-51-Filterung, damit historische Daten stabil
  bleiben (`shortday_gate.rs:143-174`).

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `feature_flag` | Cutover-Schalter | `key PK`, `enabled INT`, `description`, `update_timestamp`, `update_process` |
| `toggle` | User-Toggle | `name PK`, `enabled INT`, `description`, `value TEXT` (nullable), `update_timestamp`, `update_process` |
| `toggle_group` | Gruppierung | `name PK`, `description`, `update_timestamp`, `update_process` |
| `toggle_group_toggle` | Junction | `toggle_group_name`, `toggle_name`, ON DELETE CASCADE, UNIQUE-Constraint |
| `privilege` | ergänzt | `toggle_admin`, `feature_flag_admin` werden bei Migration geseedet |

Beachten:

- Kein `deleted`-Feld — Toggles und Feature Flags kennen kein Soft-Delete.
  `DELETE` ist ein echter Row-Removal.
- `update_process` ist Pflichtfeld — der Service füllt es mit
  `"toggle-service"` bzw. `"feature-flag-service"`
  (`service_impl/src/toggle.rs:11`, `feature_flag.rs:10`).

### Migrations

- `20260105000000_app-toggles.sql` — Basis: `toggle`, `toggle_group`,
  `toggle_group_toggle` + `toggle_admin`-Privileg.
- `20260501000000_add-feature-flag-table.sql` — `feature_flag`-Tabelle +
  Seed für `absence_range_source_active` (Phase 2) + `feature_flag_admin`-
  Privileg.
- `20260627000000_seed-paid-limit-toggle.sql` — Seed
  `paid_limit_hard_enforcement` (Phase 24).
- `20260628000000_toggle-value-column.sql` — `ALTER TABLE toggle ADD COLUMN
  value TEXT;` — die Stichtag-Ära beginnt.
- `20260628000001_seed-holiday-auto-credit-toggle.sql` — Seed
  `holiday_auto_credit` (Phase 25, HCFG-02).
- `20260704000001_seed-shortday-slot-clipping-toggle.sql` — Seed
  `shortday_slot_clipping_active_from` (Phase 51, D-51-07).

### Beziehungen

`toggle` ↔ `toggle_group` über `toggle_group_toggle` (Junction, `UNIQUE`
verhindert Doppel-Assign). CASCADE-DELETE auf beiden FKs. `feature_flag` steht
allein und referenziert nichts.

## 4. Service-API

### 4.1 `FeatureFlagService` (Basic Service)

`service::feature_flag::FeatureFlagService`

```rust
#[async_trait]
pub trait FeatureFlagService {
    type Context; type Transaction;
    async fn is_enabled(&self, key: &str, context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>) -> Result<bool, ServiceError>;
    async fn set(&self, key: &str, value: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>) -> Result<(), ServiceError>;
}
```

- **`is_enabled`** — Auth-only, jeder authentifizierte User; `Full` bypasst
  (`service_impl/src/feature_flag.rs:36-42`).
- **`set`** — verlangt `FEATURE_FLAG_ADMIN_PRIVILEGE`
  (`service_impl/src/feature_flag.rs:56-59`).
- **Deps:** `FeatureFlagDao`, `PermissionService`, `TransactionDao` — reines
  Basic-Service-Muster, keine anderen Domain-Services.

### 4.2 `ToggleService` (Basic Service)

`service::toggle::ToggleService` — 16 Methoden, aufgeteilt in vier
Gruppen. Kern:

- **Read (Auth-only, `Full` bypasst):** `is_enabled`, `get_all_toggles`,
  `get_toggle`, `get_toggle_value` (`service_impl/src/toggle.rs:26-98,
  176-202`).
- **Write (Admin):** `create_toggle`, `enable_toggle`, `disable_toggle`,
  `set_toggle_value`, `delete_toggle` — alle mit
  `check_permission(TOGGLE_ADMIN_PRIVILEGE, …)`.
- **Group-Read/Write (Admin):** `create_toggle_group`, `delete_toggle_group`,
  `get_all_toggle_groups`, `get_toggle_group`. **Auffällig:** die Group-Reads
  sind Admin-gated (`service_impl/src/toggle.rs:287-290, 304-307`), obwohl
  Toggle-Reads Auth-only sind — Gruppen sind Admin-Fach.
- **Group-Membership (Admin):** `add_toggle_to_group`, `remove_toggle_from_group`,
  `get_toggles_in_group`, `enable_group`, `disable_group`.
- **`enable_toggle` / `disable_toggle`** sind Read-Modify-Write in *einer* TX:
  `get_toggle` → mutate `enabled` → `update_toggle` → `commit`
  (`service_impl/src/toggle.rs:131-144`).

### Auth-Gates

- Read-Auth-only für `toggle`/`feature_flag` mit `Full`-Bypass (Phase 51
  Fix).
- Admin-Privileg heißt `toggle_admin` bzw. `feature_flag_admin` und wird
  jeweils per Seed-Migration eingespielt.

### TX-Verhalten

- Jede Methode öffnet TX über `transaction_dao.use_transaction(tx)`, führt
  DAO-Call, committed. `enable_toggle`/`disable_toggle` machen einen
  atomaren Read-Modify-Write innerhalb *einer* TX. Kein Composite-Op über
  mehrere Aggregate hinweg — alles single-row-scope.

### Dependencies

- `FeatureFlagService`: `FeatureFlagDao`, `PermissionService`, `TransactionDao`.
- `ToggleService`: `ToggleDao`, `PermissionService`, `TransactionDao`.
- **Beide sind Basic Services** (Service-Tier-Konvention aus
  `CLAUDE.md`). Sie konsumieren keine Domain-Services.

### 4.3 `SchedulerService`

`service::scheduler::SchedulerService`

```rust
async fn start(&self) -> Result<(), ServiceError>;
async fn schedule_carryover_updates(&self, cron: &'static str) -> Result<(), ServiceError>;
```

Impl-Detail: `SchedulerServiceImpl` hält einen `Arc<Mutex<Scheduler<Local>>>`
(`tokio_cron`) als Custom-Field. **Deps:** `ShiftplanEditService` (das
`update_carryover_all_employees(year, Auth, tx)` liefert) —
`scheduler.rs:14-20`. Damit ist der Scheduler in die
Business-Logic-Tier-Zone: er konsumiert einen Domain-Service.

Der auskommentierte `tokio::spawn`-Loop in `start()`
(`service_impl/src/scheduler.rs:39-44`) zeigt, dass der Scheduler-Loop selbst
den Prozess nicht schlafen legt — `tokio_cron` verwaltet den Tick intern.
Ob das aktuell so gewollt ist oder ein Rest-Refactor: **[Zu prüfen]**.

### 4.4 `ClockService` und `UuidService`

Beides synchrone One-Method-Traits ohne Auth, ohne TX, ohne DAO. Zweck:
Test-Injektion. Konkrete Impls sind zwei Zeilen lang und wrappen
`OffsetDateTime::now_utc()` bzw. `Uuid::new_v4()`.

### 4.5 `ConfigService`

`service::config::ConfigService` — eine Methode `get_config() ->
Result<Config, ServiceError>`. `Config` enthält `timezone` und `ical_label`.
`ConfigServiceImpl` liest bei jedem Call frisch aus `std::env`
(`service_impl/src/config.rs:12-22`) — kein Caching. Fallbacks: `"UTC"` und
`"Schicht"`.

### 4.6 `shortday_gate`-Modul (kein Service)

Public API:

- `TOGGLE_NAME: &str` — Konstante, damit Konsumenten keine Magic-Strings
  streuen.
- `parse_active_from(Option<&str>) -> Option<Date>` — ISO-8601-Parse mit
  defensivem `None`-Fallback.
- `should_clip(booking_date, active_from) -> bool` — inklusiv am Stichtag
  (`shortday_gate.rs:66-71`).
- `resolve_active_from_for_week(year, week, dow, active_from) -> bool` —
  Convenience für Konsumenten, die nur `(year, week, day_of_week)` haben.
- `clip_slot_for_week(slot, special_days, year, week, active_from, mode) ->
  ClipOutcome` — der Master-Helper, wird von allen vier Chains benutzt
  (`shortday_gate.rs:193-240`).

Crate-lokal:

- `read_active_from<S: ToggleService>(svc, ctx) -> Result<Option<Date>>` —
  toleriert `Unauthorized`.

## 5. REST-Endpoints

### 5.1 `/feature-flag`

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/feature-flag/{key}` | Fail-safe Read; unbekannter Key → `enabled=false`. | — | `FeatureFlagTO` | 401 |

`FeatureFlagTO`: `{ key: String, enabled: bool, description: Option<String> }`
(`rest-types/src/lib.rs:2363-2370`). Der Handler setzt `description: None`
weil das Trait `is_enabled` es nicht liefert
(`rest/src/feature_flag.rs:60-66`). Schreibzugriff gibt es **nicht** über
REST — bewusst (Phase 8 08-07 Kommentar `rest/src/feature_flag.rs:80-88`).

### 5.2 `/toggle`

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/toggle` | Alle Toggles | — | `[ToggleTO]` | 401 |
| `POST` | `/toggle` | Toggle anlegen | `ToggleTO` | 201 | 401/403 |
| `GET` | `/toggle/{name}` | Einzelner Toggle | — | `ToggleTO` | 401/404 |
| `GET` | `/toggle/{name}/enabled` | Nur boolean | — | `bool` | 401 |
| `PUT` | `/toggle/{name}/enable` | Anschalten | — | 204 | 401/403/404 |
| `PUT` | `/toggle/{name}/disable` | Ausschalten | — | 204 | 401/403/404 |
| `DELETE` | `/toggle/{name}` | Löschen | — | 204 | 401/403 |
| `GET` | `/toggle/{name}/value` | Wert lesen (String) | — | `String` oder 204 | 401 |
| `PUT` | `/toggle/{name}/value` | Wert setzen (ISO-Datum validiert) | JSON-String | 204 | 400/401/403 |
| `DELETE` | `/toggle/{name}/value` | Wert löschen (+disable) | — | 204 | 401/403 |

### 5.3 `/toggle-group`

Analoge CRUD-Endpoints für Gruppen + `POST /toggle-group/{group}/toggle/{toggle}`
für Membership. Registrierung: `rest/src/lib.rs:587-588`.

DTOs (`rest-types/src/lib.rs`):

- `ToggleTO { name, enabled, description, value }`
- `ToggleGroupTO { name, description }`
- `FeatureFlagTO { key, enabled, description }`
- Serde-Default für `description`/`value` — Frontend darf sie weglassen.

## 6. Frontend-Integration

- **Page:** `shifty-dioxus/src/page/settings.rs` — die einzige UI-Site für
  diesen Cluster. Drei Cards zeigen die drei aktuellen Toggles:
  - Card 1 (`settings.rs:572-628`): `paid_limit_hard_enforcement`
    (Phase 24) über `loader::get_toggle_enabled` /
    `loader::set_toggle`.
  - Card 2 (`settings.rs:630-715`): `holiday_auto_credit` — ISO-Datums-Input,
    `loader::get_holiday_cutoff_date` / `set_holiday_cutoff_date`.
  - Card 2b (`settings.rs:717-…`, Phase 51 SHC-06):
    `shortday_slot_clipping_active_from` — Blaupause identisch zu Card 2,
    `loader::get_shortday_clipping_active_from` /
    `set_shortday_clipping_active_from`.
- **API-Client:** `shifty-dioxus/src/api.rs` — `set_toggle`,
  `get_toggle_enabled`, `get_toggle_value`, `set_toggle_value`,
  `clear_toggle_value`, `get_feature_flag`.
- **Loader:** `shifty-dioxus/src/loader.rs:893-957` bündelt die Toggle-Calls
  hinter fachlichen Namen (`get_holiday_cutoff_date`,
  `get_shortday_clipping_active_from`, …). Der `TOGGLE_NAME`-String im
  Frontend spiegelt bewusst `service_impl::shortday_gate::TOGGLE_NAME`
  (`loader.rs:932-935`).
- **i18n-Keys:** `SettingsPaidLimitToggleLabel/On/Off/Description`, Cards
  für Holiday + Shortday reihen sich analog ein — alle drei Locales (En, De,
  Cs) müssen bei Änderungen mitgepflegt werden.
- **Proxy:** `Dioxus.toml` muss `/toggle`, `/toggle-group`, `/feature-flag`
  auf das Backend proxien. Neue Routes brauchen eigenen `[[web.proxy]]`-
  Eintrag, sonst 404 im `dx-serve`-Dev-Modus (siehe Memory
  `feedback_dioxus_proxy_for_new_backend_endpoints`).

## 7. Randfälle

Zentrale Referenz:
[`../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts`](../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts).

Feature-spezifisch:

- **Randfall — Toggle-Read unter `Authentication::Full` (Phase 51
  Gap-Closure):** Interne Aggregat-Konsumenten (Chain C
  `booking_information.rs:310, 547`, Chain D `reporting.rs`,
  `shiftplan_report.rs:118, 211, 270`, Chain A' `block.rs:103, 292`) rufen
  `ToggleService`-Reads **mit `Authentication::Full`** durch, um HR-Bypass
  zu haben. Vor Phase 51 lieferte `PermissionService::current_user_id(Full)`
  `Ok(None)` → `ToggleService` warf `Unauthorized` → `shortday_gate`
  schluckte still zu `Ok(None)` → Slot-Kürzung griff nie, obwohl der Toggle
  gesetzt war (Live-Symptom: volle Slot-Stunde statt geklippter 0,5 h).
  Der Fix in `service_impl/src/toggle.rs:46-51, 66-71, 87-92, 191-196`
  behandelt `Full` als all-rights-Bypass. Regression-Guards liegen in
  `service_impl/src/test/toggle.rs:547-556` — die vier Tests dürfen
  `PermissionService::current_user_id` gar nicht aufrufen (Mock ohne
  `expect_*` würde panicken). Siehe auch Memory
  `reference_toggle_service_full_context_bypass`.
- **Randfall — Feature-Flag live umgeschaltet:** Das Service-API erlaubt
  `set()` zur Laufzeit, aber der einzige Prod-Konsument
  (`absence_range_source_active`) ist als Cutover gedacht: einmal atomar mit
  einer Phase-4-Migration flippen, nie manuell. Migrations-Kommentar
  (`20260501000000_add-feature-flag-table.sql:18`): "Flip atomically with
  Phase-4 migration; do NOT flip manually."
- **Randfall — Scheduler-Miss-Tick:** Der Cron-Job läuft mit `"0 * * * * *"`,
  also jede Minute. Fällt ein Tick aus (z. B. weil ein früherer Lauf
  hängt), springt `tokio_cron` einfach zum nächsten Slot — es gibt keinen
  Catch-up-Mechanismus. Für Carryover ist das harmlos: der nächste Tick
  rechnet die aktuelle Wahrheit neu. Wichtig zu wissen: Fehler werden nur
  geloggt (`error!` in `scheduler.rs:63, 71`), nicht monitored — wer echte
  Alarme braucht, muss die Logs auswerten.
- **Randfall — Toggle-Value setzt implizit `enabled=1`:** Die REST-Semantik
  von `PUT /toggle/{name}/value` ist "Wert setzen **und** aktivieren"
  (`rest/src/toggle.rs:337`). Ein value-basierter Rollout (ISO-Datum) kann
  also nicht "Wert gesetzt, aber deaktiviert" sein. `DELETE /value` setzt
  zurück auf `NULL` + `enabled=0`.
- **Randfall — `parse_active_from` bei defekten Werten:** `Some("garbage")`
  → `None` statt Panic. Die Konsumkette fällt in Legacy-Verhalten zurück,
  nicht in einen 500-Response (`shortday_gate.rs:51-57`, Testabdeckung
  `shortday_gate.rs:262-265`).
- **Randfall — Clock in UTC vs Report in lokaler Zeit:** `ClockService`
  liefert immer UTC. Konsumenten, die lokale Zeit brauchen (z. B.
  `Local`-basierter Scheduler in `SchedulerServiceImpl::new`), müssen selbst
  konvertieren. **[Zu prüfen]** wo das aktuell relevant beißt.

## 8. Tests

- **Unit — Feature-Flag:** `service_impl/src/test/feature_flag.rs` (196
  Zeilen). Deckt Auth-Kombinationen, Admin-Gate für `set()`, Fail-safe für
  unbekannte Keys.
- **Unit — Toggle:** `service_impl/src/test/toggle.rs` (652 Zeilen). Deckt
  vollständiges CRUD + Group-Ops + Value-Ops. Ab
  `service_impl/src/test/toggle.rs:547-556` explizit die Phase-51-
  Regression-Guards für den `Full`-Bypass (mit dem Mock-ohne-Expectation-
  Trick als Assertion).
- **Unit — Shortday-Gate:** `service_impl/src/shortday_gate.rs:242-479`
  in derselben Datei. Deckt `parse_active_from`, `should_clip`,
  `resolve_active_from_for_week` inklusive `Legacy` vs `Modern` Modus
  gegen alle Kombinationen von (Gate aktiv/inaktiv, ShortDay ja/nein,
  slot.to relativ zu cutoff).
- **Integration (Chain-Ebene):** Die vier Chains werden separat getestet —
  `service_impl/src/test/shiftplan.rs` (Chain B),
  `test/booking_information_chain_c.rs` (Chain C), etc. Sie decken das
  Verhalten mit Gate-an und Gate-aus + Legacy-Filter ab.
- **Integration — Scheduler:** **Nicht direkt getestet.** Der
  `SchedulerServiceImpl` hat keinen Test-File; er wird implizit dadurch
  abgedeckt, dass `update_carryover_all_employees` in
  `service_impl/src/test/shiftplan_edit/*` getestet wird. **Bekannte Lücke:**
  Cron-Parsing, Fehler-Isolation zwischen `year-1`- und `year`-Läufen ist
  nur durch Manual-Test verifiziert.
- **Clock / UUID:** Keine dedizierten Tests — sie sind selbst die
  Test-Abstraktion. Konsumenten mocken sie.
- **Config:** Keine dedizierten Tests — reines Env-Read.

## 9. Historie & Kontext

- **v1.0-Ära:** `toggle`, `toggle_group`, `toggle_group_toggle` als
  Basis-Infrastruktur (`20260105000000`).
- **Phase 2 (2026-05):** Feature-Flag-Tabelle als *bewusst separater*
  Mechanismus zur Absence-Cutover-Steuerung
  (`absence_range_source_active`). Design-Notiz in
  `openspec/changes/billing-period-snapshot-versioning/design.md` **[Zu
  prüfen ob dort auch die Feature-Flag-Trennung erklärt ist]**.
- **Phase 24 (2026-06-27):** Erster user-facing Boolean-Toggle
  (`paid_limit_hard_enforcement`) — hard/soft-Enforcement.
- **Phase 25 / HCFG-02 (2026-06-28):** Toggle-Wert-Spalte + erster
  Stichtag-Toggle (`holiday_auto_credit`). Das Muster
  "Value=ISO-Datum, Semantik ab-Stichtag" wird hier etabliert und in
  `service_impl/src/reporting.rs:164-180` prototypisch für Konsumenten
  festgehalten.
- **Phase 48:** PDF-Export-Scheduler wird eingeführt (`pdf_export_scheduler.rs`),
  parallel zum Carryover-Scheduler.
- **Phase 51 / D-51-07 (2026-07-04):**
  - Shortday-Slot-Clipping-Toggle geseedet.
  - `shortday_gate`-Modul als zentraler Ort für alle vier Konsumkette-Chains.
  - **Toggle-Full-Bypass in `ToggleService`-Reads** — der wichtige
    Gap-Closure. Ohne diesen Fix funktionieren die Chain-C-/Chain-D-
    Konsumenten nicht: siehe `service_impl/src/toggle.rs:32-51` und
    Regression-Tests in `service_impl/src/test/toggle.rs:547-556`.
- **Service-Tier-Konvention:** Sowohl `FeatureFlagService` als auch
  `ToggleService` sind **Basic Services** (nur DAO + Permission +
  Transaction). Das ist bewusst — sie werden von vielen Business-Logic-
  Services konsumiert, dürften aber selbst keine solchen konsumieren, um
  Zyklen zu vermeiden (siehe `CLAUDE.md` "Service-Tier-Konventionen"). Der
  Scheduler ist dagegen Business-Logic-Tier (konsumiert
  `ShiftplanEditService`).
- **Verweise auf Planning-Kontext:** `.planning/phases/` enthält für
  Phase 2 (Feature-Flag-Cutover), Phase 24, Phase 25, Phase 48, Phase 51
  detaillierte Design-Docs mit den Decision-Codes (`D-Phase2-06`, `D-24-06`,
  `D-25-06`, `D-51-06/07/09`, `HCFG-02`). Wer Kontext zu einer konkreten
  Regel braucht: dort suchen.

---

**Fazit:** F13 bündelt zwei ansonsten verwechselbare Schaltmechanismen
(architektureller `feature_flag` vs user-facing `toggle` mit
Stichtag-Value) plus den nötigen Kleinkram (Scheduler, Clock, UUID,
Config, Shortday-Gate). Die zentrale Lektion aus Phase 51: **Read-Ops für
Config-Daten müssen `Authentication::Full` als all-rights-Kontext
akzeptieren**, sonst brechen alle internen Aggregat-Ketten still.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
