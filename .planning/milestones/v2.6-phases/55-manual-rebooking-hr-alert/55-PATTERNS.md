# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — Pattern Map

**Mapped:** 2026-07-23
**Files analyzed:** 21 (14 Backend + 7 Frontend)
**Analogs found:** 20 / 21 (1 genuiner Neubau ohne exaktes Analog: state-conditional UPDATE)

> Alle Zeilen-/Signatur-Referenzen sind gegen die Codebase verifiziert (Stand HEAD `bb25c91c`).
> Identifier/Code im Original (Rust/Rust-RSX); Prosa auf Deutsch.

---

## File Classification

| Neu/Geändert | Rolle | Data Flow | Nächstes Analog | Match |
|--------------|-------|-----------|-----------------|-------|
| `service/src/rebooking_reconciliation.rs` | service (Trait, BL) | request-response / transform | `service/src/rebooking_batch.rs` (Trait) + `service/src/voluntary_stats.rs` | role-match |
| `service_impl/src/rebooking_reconciliation.rs` | service (Impl, BL) | CRUD (multi-write in 1 Tx) | `service_impl/src/voluntary_stats.rs` (BL-Tx-Orchestrierung) + `service_impl/src/rebooking_batch.rs` | exact |
| `service/src/rebooking_batch.rs` (+update_state, +find_pending) | service (Trait, Basic) | CRUD | vorhandene Methoden im selben File | exact (in-place) |
| `service_impl/src/rebooking_batch.rs` (+update_state, +find_pending) | service (Impl, Basic) | CRUD | `create`/`find_by_id` im selben File | exact (in-place) |
| `dao/src/rebooking_batch.rs` (+update_state_conditional, +find_pending) | dao (Trait) | CRUD | vorhandene Methoden im selben File | exact (in-place) |
| `dao_impl_sqlite/src/rebooking_batch.rs` (state-conditional UPDATE) | dao (Impl) | CRUD | `create_batch_with_entries` (selbes File) + `user_invitation::delete_expired` (`rows_affected()`) | role-match |
| `rest/src/rebooking.rs` (4 Routes) | route/controller | request-response | `rest/src/absence.rs` | exact |
| `rest-types/src/rebooking.rs` (DTOs) | model (DTO) | transform | `rest-types/src/lib.rs` (bestehende TOs) | exact |
| `rest-types/src/lib.rs` (`ShortEmployeeReportTO` +2 Felder) | model (DTO) | transform | `ShortEmployeeReportTO` selbst (Z.373) | exact (in-place) |
| `service_impl/src/reporting.rs` (source-Filter + `alert_predicate`) | service (aggregation) | transform | Aggregations-Sites Z.894/1401/2263 (in-place) | exact |
| `service/src/reporting.rs` (`ShortEmployeeReport` +2 Felder) | model | transform | Struct Z.148 (in-place) | exact |
| `shifty_bin/src/main.rs` (DI BL-Tier) | config (DI-Wiring) | — | `VoluntaryStatsServiceDependencies` (Z.334–351) | exact |
| `shifty-dioxus/Dioxus.toml` (Proxy) | config | — | bestehende `[[web.proxy]]` (Z.45+) | exact |
| `shifty-dioxus/src/loader.rs` (4 Loader) | utility (data-load) | request-response | `load_sales_persons` (Z.31) etc. | exact |
| `shifty-dioxus/src/state/rebooking.rs` | model (State-Mapper) | transform | `state/absence_period.rs` `From<&…TO>` | exact |
| `shifty-dioxus/src/component/rebooking_alert_banner.rs` | component | event-driven (click) | `component/warning_list.rs` + `impersonation_banner.rs` | role-match |
| `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` | component (modal) | request-response | `component/dialog.rs` (Shell) + `voluntary_stats_row.rs` | role-match |
| `shifty-dioxus/src/component/manual_rebooking_modal.rs` | component (modal) | request-response | `component/absence_convert_modal.rs` + `dialog.rs` | exact |
| `shifty-dioxus/src/page/employees.rs` | page (Integration) | — | (bestehende Seite, Banner-Einbindung) | in-place |
| `shifty-dioxus/src/page/employee_details.rs` | page (Integration) | — | `TopBar {}` (Z.141) | in-place |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` | config (i18n) | — | bestehende `Key`-Varianten | exact (in-place) |

---

## Pattern Assignments

### `service_impl/src/rebooking_reconciliation.rs` (service, BL, CRUD-in-1-Tx)

**Analog A (BL-Tx-Orchestrierung):** `service_impl/src/voluntary_stats.rs`
**Analog B (gen_service_impl! + HR-Gate + defensive Entity-Fill):** `service_impl/src/rebooking_batch.rs`

**gen_service_impl!-Deps-Block** (Vorbild `voluntary_stats.rs:42-57`) — die Deps kommen aus RESEARCH-Code-Beispiel „DI-Wiring":
```rust
gen_service_impl! {
    struct RebookingReconciliationServiceImpl: RebookingReconciliationService = RebookingReconciliationServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction, Context = Self::Context> = extra_hours_service,
        RebookingBatchService: RebookingBatchService<Transaction = Self::Transaction, Context = Self::Context> = rebooking_batch_service,
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**HR-Gate-ZUERST + Tx-Klammer** (kopiere Struktur aus `voluntary_stats.rs:72-137`):
```rust
// HR-Gate an erster Stelle — kein Datenabruf vor Auth.
self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;
let tx = self.transaction_dao.use_transaction(tx).await?;
// … cross-service calls IMMER mit Some(tx.clone()) und Authentication::Full als Bypass …
self.transaction_dao.commit(tx).await?;
```

**KRITISCH — Cross-Service-Calls im BL-Kontext** (Präzedenz `voluntary_stats.rs:95-98,106-116`):
Basic-Services (`ExtraHoursService::create`, `RebookingBatchService::create`) werden mit
`Some(tx.clone())` UND `Authentication::Full` aufgerufen (Auth wurde am Einstieg geprüft;
`reference_toggle_service_full_context_bypass`). Sonst öffnet jeder Basic-Service seine eigene
Tx und committet zu früh → kein Rollback über die Grenze (RESEARCH Pattern 1).

**Single write-path** (Pitfall 15): ein interner `rebook(direction, hours, week, kind)` den F3
(Manual) und F5-Approve (HrSuggestion) beide aufrufen. `ExtraHoursService::create`-Signatur
(`service/src/extra_hours.rs:286`): `create(&ExtraHours, Authentication<Ctx>, Option<Tx>)`.
Beide Pair-Rows tragen `source: ExtraHoursSource::Rebooking` (`service/src/extra_hours.rs:60,186`).

**RebookingDirection-Enum** — siehe RESEARCH Code-Beispiel (neu im Trait-Modul).

---

### `service/src/rebooking_reconciliation.rs` (service, Trait, BL)

**Analog:** `service/src/rebooking_batch.rs` (kompletter Trait-File)

**Trait-Header + automock** (kopiere Z.28-32 aus `rebooking_batch.rs`):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingReconciliationService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;
    // async fn rebook_manual(...), async fn get_suggestion(...),
    // async fn approve(...), async fn reject(...)
}
```
`automock` erzeugt `MockRebookingReconciliationService` für Downstream-Tests (wie
`rebooking_batch.rs:17`). Jede Methode: `context: Authentication<Self::Context>` +
`tx: Option<Self::Transaction>` als letzte zwei Params (Projekt-Konvention, überall belegt).

---

### `service/src/rebooking_batch.rs` + `service_impl/src/rebooking_batch.rs` (+`update_state`, +`find_pending_for_sales_person`)

**Analog:** die vorhandenen Methoden im selben File (in-place-Erweiterung).

**`find_pending_for_sales_person`** (D-55-02 Ripple): kopiere Struktur von
`find_by_sales_person_year_week` (`service_impl/src/rebooking_batch.rs:57-76`) — HR-Gate,
`use_transaction`, DAO-Delegation, `commit`. Prädikat: `state=Pending` (D-55-08).

**`update_state` (approve/reject)** — Basic-Service-Wrapper um das neue state-conditional
DAO-UPDATE. HR-Gate ZUERST (Z.47-49), dann DAO-Call, dann `commit`. Der `rows_affected()==0`-Fall
mappt auf `ServiceError::Conflict`/`EntityAlreadyExists` → REST 409.

> Der Trait-Doc-Kommentar (`service/src/rebooking_batch.rs:12-15`) kündigt genau diese
> Phase-55-Erweiterung bereits an („update_state approve/reject, list_pending").

---

### `dao_impl_sqlite/src/rebooking_batch.rs` (state-conditional UPDATE — genuiner Neubau)

**Analog A (SQL-Mechanik, TryFrom, Enum-Konversion):** `create_batch_with_entries` (selbes File, Z.225-317).
**Analog B (`rows_affected()`-API):** `dao_impl_sqlite/src/user_invitation.rs:211-226` (`delete_expired`).

**state-conditional UPDATE + Guard** (RESEARCH Pattern 2; SQL-Style aus `create_batch_with_entries`,
`rows_affected()` aus `user_invitation.rs:214-225`):
```rust
let result = sqlx::query!(
    r#"UPDATE rebooking_batch
       SET state = ?, approved = ?, approved_by = ?, update_process = ?, update_version = ?
       WHERE id = ? AND state = 'pending' AND deleted IS NULL"#,
    new_state_str, approved_ts, approved_by, process, version_vec, id_vec
)
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;
if result.rows_affected() != 1 {
    return Err(DaoError::/* Conflict-Variante */);
}
```
- Enum→String via bestehende `state_to_str` (Z.139-146) — NICHT neu bauen.
- Alle Byte-Params via `.as_bytes().to_vec()` (Uuid) und `.format(&Iso8601::DATE_TIME)` (Timestamps),
  exakt wie Z.232-248.
- `tx.tx.lock().await.as_mut()` ist das Tx-Handle-Muster (Z.268).
- **Reihenfolge (Pitfall 12):** state-UPDATE ZUERST; nur bei `rows_affected()==1` folgen die
  ExtraHours-Pair-Writes im aufrufenden BL-Service.

**`find_pending_for_sales_person` (DAO)**: kopiere `find_by_sales_person_year_week` (Z.198-223)
1:1, ergänze `AND state = 'pending'` in der `WHERE`-Klausel. `RebookingBatchDb`-Row-Struct +
`TryFrom` (Z.24-65) unverändert wiederverwenden.

**`cargo sqlx prepare`** nach neuen Queries (RESEARCH Runtime-Inventory).

---

### `rest/src/rebooking.rs` (route, request-response, 4 Routes)

**Analog:** `rest/src/absence.rs` (kompletter File — 6 Routes, gleiche Struktur).

**Imports + `error_handler`-Wrapper** (kopiere `absence.rs:19-38`):
```rust
use axum::{body::Body, extract::{Path, State}, response::Response, routing::{get, post}, Extension, Json, Router};
use tracing::instrument; use utoipa::OpenApi; use uuid::Uuid;
use crate::{error_handler, Context, RestStateDef};
```

**`generate_route`** (Muster `absence.rs:163-174`):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/manual", post(create_manual_rebooking::<RestState>))
        // GET /rebooking-suggestions, POST /rebooking-suggestions/{id}/approve|reject
        // eigenes Router-Modul (zweiter Path-Root) — siehe main.rs Route-Registrierung
}
```

**Handler-Skelett** (kopiere `create_absence_period` `absence.rs:176-210`): `#[instrument(skip(rest_state))]`
+ `#[utoipa::path(post, path="…", tags=["Rebooking"], request_body=…, responses(201, 403, 409, 422))]`
+ Body-`error_handler((async { … svc.method(&(&body).into(), context.into(), None).await? … }).await)`.

**Error-Mapping:** `ServiceError::EntityAlreadyExists` / `Conflict` → HTTP 409 (D-55-05,
`RebookingErrorSlotTaken`). Das `error_handler`-Wrapper macht das Status-Mapping zentral —
NICHT im Handler manuell mappen.

**`#[derive(OpenApi)] RebookingApiDoc`** (kopiere `absence.rs:523-546`): `paths(...)` +
`components(schemas(...))` + `tags(...)`; anschließend in die zentrale `ApiDoc` einhängen (CLAUDE.md OpenAPI-Regel).

---

### `rest-types/src/rebooking.rs` + `rest-types/src/lib.rs` (model, DTOs)

**Analog:** bestehende TOs in `rest-types/src/lib.rs` (`ShortEmployeeReportTO` Z.373-395).

> **Achtung Struktur:** `rest-types` hat aktuell NUR `lib.rs` (kein per-Modul-Split). Der Neubau
> `rest-types/src/rebooking.rs` erfordert ein `pub mod rebooking;` + Re-Export in `lib.rs`.
> Alternativ (einfacher, konsistent mit dem bestehenden Monolith) die neuen TOs direkt in `lib.rs`.
> Der Planner pinnt die Wahl; das Analog-Muster ist identisch.

**`ShortEmployeeReportTO`-Erweiterung** (D-55-02, in-place, RESEARCH Pattern 3 — Präzedenz
`volunteer_hours` Z.379-380):
```rust
pub struct ShortEmployeeReportTO {
    // … bestehende Felder …
    #[serde(default)]
    pub volunteer_hours: f32,
    #[serde(default)]                       // NEU
    pub has_pending_rebooking: bool,
    #[serde(default)]                       // NEU
    pub pending_rebooking_id: Option<Uuid>,
}
```
Das `From<&ShortEmployeeReport>`-Impl (Z.384-394) muss die zwei neuen Felder mappen; die
Service-Struct `ShortEmployeeReport` (`service/src/reporting.rs:148`) bekommt sie ebenfalls.

**Neue TOs** (`ManualRebookingRequestTO`, `RebookingSuggestionTO`, `RebookingBatchTO`): alle mit
`#[derive(Debug, Serialize, Deserialize, ToSchema)]` (Muster Z.397 im selben File). Alle
DANN-/`proposed_hours`-Felder sind `f32` und backend-berechnet (Fat Backend).

---

### `service_impl/src/reporting.rs` (aggregation — source-Filter + `alert_predicate`)

**Analog:** die drei Aggregations-Sites in-place (VERIFIED).

**source==Rebooking-Filter an 3 Sites** (VOL-ACCT-03, Pitfall 1). Aktueller Code filtert NUR
nach `category`:
- Z.894-901 (`manual_volunteer_hours`): `.filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork)`
- Z.1401-1406 (`week_extra_hours` volunteer): analog
- Z.2263-2268 (`filtered_extra_hours_list` volunteer): analog

Ergänze an jeder Site die source-Bedingung:
```rust
.filter(|eh| eh.category == ExtraHoursCategory::VolunteerWork
          && eh.source == service::extra_hours::ExtraHoursSource::Manual)
```
`ExtraHours.source` fließt bereits durch die Kette (`service/src/extra_hours.rs:186`,
`From<&ExtraHoursEntity>` Z.200-201 setzt ihn). Der Kommentar `reporting.rs:184` kündigt genau
diesen Phase-55-Filter an.

> **Filter-Scope (A1 — offene Semantik):** RESEARCH markiert als kritische Design-Frage, ob NUR
> die Voluntary-Read-Aggregate ODER auch die `ExtraWork`→balance-Summierung gefiltert werden.
> CONTEXT-Constraint spricht für vollständige Filterung beider Rows aus allen Read-Aggregaten.
> **Property-Test ZUERST** (tdd), der die Invariante festnagelt, BEVOR der Filter entsteht.

**`alert_predicate` als pure fn** (D-55-01, RESEARCH Pattern 4; Präzedenz reine Zählfunktionen
wie `committed_voluntary_target_in_range`, `contract_weeks_count_in_range` in `reporting.rs`):
```rust
fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}
```
Truth-Table-Unit-Test (`-0.49→false`, `-0.5→true`, `-0.51→true`, `vol_ist=0→false`,
`cap_active=false→false`) — Test-Modul-Struktur wie `absence.rs:548` (`mod ... _tests`).

**`proposed_hours = min(|balance|, voluntary_ist)`** (D-55-03) ebenfalls als pure fn + Test-Matrix.

---

### `shifty_bin/src/main.rs` (config, DI BL-Tier)

**Analog:** `VoluntaryStatsServiceDependencies` (Z.334-351) — der einzig richtige Vorbild-Block
(BL-Tier, konsumiert Domain-Services).

Kopiere Struktur (siehe auch RESEARCH Code-Beispiel „DI-Wiring"):
```rust
pub struct RebookingReconciliationServiceDependencies;
impl service_impl::rebooking_reconciliation::RebookingReconciliationServiceDeps
    for RebookingReconciliationServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursService = ExtraHoursService;        // Z.437-450
    type RebookingBatchService = RebookingBatchService; // Z.312-323
    type ReportingService = ReportingService;           // Z.501+
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type RebookingReconciliationService =
    service_impl::rebooking_reconciliation::RebookingReconciliationServiceImpl<RebookingReconciliationServiceDependencies>;
```
**Reihenfolge (CLAUDE.md Service-Tier-Konvention):** NACH `ExtraHoursService`,
`RebookingBatchService`, `ReportingService` konstruieren (alle bereits vor Z.351 vorhanden).
Kein `OnceLock`-Trick. Danach `.nest("/rebooking", …)` + `/rebooking-suggestions`
Route-Registrierung. `RestStateDef` bekommt eine `rebooking_reconciliation_service()`-Accessor-Methode
(Muster: bestehende Accessoren wie `absence_service()`).

---

## Frontend Pattern Assignments

### `shifty-dioxus/src/component/manual_rebooking_modal.rs` (F3)

**Analog:** `component/absence_convert_modal.rs` (Submit-Defense + Feld-Muster) + `dialog.rs` (Shell).

**Feld-Muster** (kopiere `absence_convert_modal.rs:121-163`): `label.flex.flex-col.gap-1` >
`span.text-small.text-ink-muted` + `input`/`select.border.border-border.rounded-md.p-2`.
`select` mit `onchange` + `option { selected: … }` für `RebookingDirection` (Z.143-162).
`input type="number" min=0 step=0.5` für Menge (UI-SPEC Surface 3).

**Inline-Error** (kopiere Z.164-167): `if let Some(e) = error_msg.read().clone() { span { class: "text-bad text-small", "{e}" } }`.
409 → `RebookingErrorSlotTaken`, Modal bleibt offen (D-55-05). UI-SPEC schreibt `text-bad text-small font-normal mt-1`.

**Submit-Defense** (Muster Z.175-199): parse/validate (Menge > 0, gültige Woche) → inline error
statt Dispatch. Submit-Button `bg-accent text-accent-ink`, während Request `disabled` (Doppelklick-Schutz).

**Shell:** `Dialog { variant: Center, width: 460, title, subtitle, footer }` (`dialog.rs`) — kein
eigenes Backdrop/Header/Footer bauen (UI-SPEC bindend).

### `shifty-dioxus/src/component/rebooking_alert_banner.rs` (F5)

**Analog:** `component/warning_list.rs` (Warn-Container) + `impersonation_banner.rs` (klickbare Zeile).

**Container-Klassen** (Basis `warning_list.rs:113`, erweitert um Klick — UI-SPEC kanonisch):
```
"border-l-[3px] border-warn bg-warn-soft rounded-md px-4 py-3 flex items-center justify-between gap-3 cursor-pointer hover:bg-surface-alt"
```
Zero-State: `if !has_pending_rebooking { return rsx!{}; }` (Muster `warning_list.rs:95-97`).
Klick öffnet Suggestion-Modal mit `pending_rebooking_id`. A11y: `role="button"`, `tabindex="0"`,
Enter/Space (UI-SPEC Accessibility). i18n via `I18N.read()` + `.t(Key)` + `.replace("{…}", …)`
(Muster `warning_list.rs:93,122-126`).

### `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` (F5)

**Analog:** `dialog.rs` (Shell) + `voluntary_stats_row.rs` (Zahlen-Formatierung/Färbung).

IST/DANN-Drei-Spalten-Grid `grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-1.5` (UI-SPEC —
`TupleRow` kann nur 2 Spalten). Werte `font-mono tabular-nums text-right`; DANN-Balance-Färbung
`text-warn`/`text-good`; Delta signed `{:+.2}` (Farb-Blind-Redundanz). Approve `BtnVariant::Primary`,
Reject `BtnVariant::Danger`; beide während in-flight `disabled`. Alle IST/DANN-Werte aus
`RebookingSuggestionTO` (backend-computed, kein FE-Rechnen).

### `shifty-dioxus/src/state/rebooking.rs`

**Analog:** `state/absence_period.rs` (`From<&…TO>`-Mapper, Z.95,142).
Thin `From<&RebookingSuggestionTO>` / `From<&…TO>` OHNE Arithmetik (Fat-Backend-Thin-Client).

### `shifty-dioxus/src/loader.rs` (4 Loader)

**Analog:** `load_sales_persons` (Z.31-38) — Loader delegiert an `api::…`:
```rust
pub async fn submit_manual_rebooking(config: Config, req: …) -> Result<…, ShiftyError> {
    api::post_rebooking_manual(config, req).await // + State-Mapping
}
```
4 Fns: `submit_manual`, `load_pending_suggestion`, `approve`, `reject`. Passende `api`-Funktionen
in `api.rs` ergänzen (bestehende `get_*/post_*`-Konvention).

### `shifty-dioxus/Dioxus.toml` (Proxy — Pitfall 14)

**Analog:** bestehende Einträge Z.45-54. Zwei neue Blöcke:
```toml
[[web.proxy]]
backend = "http://localhost:3000/rebooking"
[[web.proxy]]
backend = "http://localhost:3000/rebooking-suggestions"
```

---

## Shared Patterns

### Authentication (HR-Gate)
**Source:** `service_impl/src/rebooking_batch.rs:47-49`, `voluntary_stats.rs:73-77`
**Apply to:** ALLE neuen Service-Methoden + REST-Routes.
```rust
self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;
```
An ERSTER Stelle jeder Methode, VOR jedem Datenabruf. Cross-Service-Calls im BL-Kontext danach
mit `Authentication::Full` (Bypass, `voluntary_stats.rs:97,113`). Non-HR: API-Level-Redaktion
(`has_pending_rebooking=false`), kein 403 (Präzedenz `voluntary_stats.rs:79-89`).

### Transaktions-Klammer
**Source:** `voluntary_stats.rs:91,137`, `rebooking_batch.rs:100,198`
**Apply to:** Alle Service-Methoden.
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// … alle DAO/Service-Calls mit Some(tx.clone()) …
self.transaction_dao.commit(tx).await?;
```

### Error-Handling (REST)
**Source:** `rest/src/absence.rs:193-209` (`error_handler`-Wrapper)
**Apply to:** Alle 4 Routes. Status-Mapping (409/403/422) zentral im `error_handler`, nicht im Handler.

### DTO-additiv mit `#[serde(default)]`
**Source:** `rest-types/src/lib.rs:379-380` (`volunteer_hours`, Präzedenz VAA-04)
**Apply to:** `ShortEmployeeReportTO`-Erweiterung — Wire-Kompat für alte Konsumenten.

### DAO-Konvention
**Source:** `dao_impl_sqlite/src/rebooking_batch.rs` (gesamter File)
**Apply to:** Alle neuen Queries. `WHERE deleted IS NULL`, `TryFrom<&…Db>`-Row-Konversion,
`.as_bytes().to_vec()` für Uuid, `Iso8601::DATE_TIME` für Timestamps, `sqlx query!`/`query_as!`
(compile-time-checked → `cargo sqlx prepare`).

### i18n (alle 3 Locales)
**Source:** `i18n/mod.rs` enum `Key` + `{de,en,cs}.rs`
**Apply to:** Alle neuen Strings (UI-SPEC Copywriting-Contract). Test-Gate
`i18n_*_present_in_all_locales`.

---

## No Analog Found

| File | Rolle | Data Flow | Grund |
|------|-------|-----------|-------|
| `dao_impl_sqlite/src/rebooking_batch.rs` — **state-conditional UPDATE** | dao | CRUD | Kein Optimistic-State-Transition-Muster existiert in der Codebase. `user_invitation::delete_expired` (`user_invitation.rs:225`) liefert NUR die `rows_affected()`-API als Teil-Analog; die `WHERE state='pending'`-Guard-Semantik ist genuin neu (RESEARCH Pattern 2, „State of the Art"). SQL-Style/TryFrom aus `create_batch_with_entries` übernehmen. |

---

## Metadata

**Analog-Suchraum:** `service/`, `service_impl/`, `dao/`, `dao_impl_sqlite/`, `rest/`, `rest-types/`,
`shifty_bin/`, `shifty-dioxus/src/{component,state,loader,page,i18n}/`, `shifty-dioxus/Dioxus.toml`.
**Dateien gescannt:** ~18 gelesen (rebooking_batch ×4, voluntary_stats, reporting, extra_hours,
absence REST, user_invitation, lib.rs DTO, main.rs DI, warning_list, absence_convert_modal, dialog,
loader, state/absence_period, Dioxus.toml).
**Pattern-Extraktion:** 2026-07-23
