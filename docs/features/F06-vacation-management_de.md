# Feature: Vacation Management — Balance, Offset & Carryover

> **Kurzform:** Der Cluster berechnet, wieviel Urlaub ein Mitarbeiter im
> Kalenderjahr hat, wieviel er verbraucht/geplant hat, was aus dem Vorjahr
> übertragen wurde und trägt den offenen Rest am Jahresende weiter. HR kann den
> Vertrags-Anspruch pro (Person, Jahr) über einen signierten Offset korrigieren.

**Cluster-ID:** F06
**Status:** produktiv (v2.4+, seit Milestone 8)
**Erstmalig eingeführt:** Milestone 8 (Vacation-Balance-Endpoint), Milestone 28 (Vacation-Entitlement-Offset)
**Zuständige Crates:**
`service::vacation_balance`, `service::vacation_entitlement_offset`, `service::carryover`,
`service_impl::vacation_balance`, `service_impl::vacation_entitlement_offset`, `service_impl::carryover`,
`service_impl::scheduler` (Cron-Trigger für Carryover),
`dao::vacation_entitlement_offset`, `dao::carryover`,
`rest::vacation_balance`, `rest::vacation_entitlement_offset`,
`rest-types::{VacationBalanceTO, VacationEntitlementOffsetTO}`

---

## 1. Was ist das? (Fachlich)

Im Arbeitsvertrag jedes Mitarbeiters steht eine Zahl bezahlter Urlaubstage pro
Jahr (`employee_work_details.vacation_days`). Dieser Cluster beantwortet für
jeden Mitarbeiter drei Kernfragen:

- **Wieviel Urlaub steht mir dieses Jahr zu?** — Vertragsanspruch, aliquot bei
  Vertragswechsel und optional per HR-Offset korrigiert.
- **Wieviel habe ich schon verbraucht und wieviel ist noch geplant?** — Summe
  über die Vacation-Absence-Periods des Jahres, aufgeteilt in `used` (bis
  heute) und `planned` (ab morgen).
- **Was habe ich noch übrig?** — `entitled + carryover − (used + planned)`.

Der **Carryover** ist der Rest-Saldo am Jahresende, den ein Mitarbeiter ins
nächste Jahr mitnimmt (analog zum Stunden-Carryover für die Balance-Hours). Er
wird nächtlich vom Scheduler frisch berechnet und persistiert — sowohl für das
Vorjahr (Rückwirkungen aus Nachtrag-Buchungen) als auch für das laufende Jahr
(kontinuierliche Vorschau).

Der **Vacation-Entitlement-Offset** ist eine HR-only Korrektur pro
(Mitarbeiter, Jahr): eine Ganzzahl in Tagen, die auf den Vertrags-Anspruch
addiert wird — z. B. `+2` als Prämie, `-3` als einmaliger Abzug. Der Offset
ist bewusst *ganztägig* und wird NACH der `.round()`-Ganzzahl-Konvertierung
des Anspruchs addiert, damit er niemals durch Rundung „verschwindet".

**Beispiel-Workflow aus User-Sicht (HR):**

1. HR öffnet den Reiter „Abwesenheiten", wählt Jahr 2026 und einen
   Mitarbeiter aus.
2. Die `VacationEntitlementCard` zeigt fünf Kacheln:
   `Vertrag`, `Übertrag Vorjahr`, `Genommen`, `Geplant`, `Rest`.
3. HR sieht zusätzlich zum effektiven Anspruch den *rohen* Vertragsanspruch
   und den aktuellen Offset. Ein Inline-Editor erlaubt, den Offset zu setzen
   oder zu löschen. Die Kachel aktualisiert sich sofort.
4. Am 01.01. läuft der Cron: das Vorjahr wird endgültig „geschlossen"
   (letzter Nachtrag), das neue Jahr bekommt initial einen Carryover-Eintrag.

**Beispiel-Workflow (Mitarbeiter):**

1. Mitarbeiter öffnet seine Selbstansicht → sieht `entitled`, `carryover`,
   `used`, `planned`, `remaining`. Der **rohe** Vertragsanspruch und der
   Offset werden ihm serverseitig NICHT ausgeliefert (API-Hiding, D-28-03).

## 2. Fachliche Regeln

### Vertrags-Anspruch (`entitled_days`)

- Quelle: `EmployeeWorkDetails.vacation_days: u8` — pro Vertragsabschnitt eine
  Jahreszahl an Urlaubstagen (`service/src/employee_work_details.rs:37`).
- Aliquotierung: `EmployeeWorkDetails::vacation_days_for_year(year)`
  (`service/src/employee_work_details.rs:158-194`) rechnet für Verträge, die
  nur einen Teil des Jahres abdecken, den Anteil `ordinal_days / days_in_year`
  ab und liefert einen `f32`. **Phase-28-Fix (D-28-04):** Der Abzug am
  Vertrags*start* startet erst bei Tag 1 (`ordinal - 1`), damit ein
  Vertrag mit Start `01.01.` KEIN Sechzigstel abzieht.
- Aggregation über alle nicht-gelöschten Verträge des Jahres, dann
  `.round()` auf ganze Tage (`vacation_balance.rs:195-200` — konsistent mit
  `reporting.rs`).
- **Offset addiert nach der Rundung:** `entitled_effective = round(base) +
  offset_days` (`vacation_balance.rs:213-214`, D-28-02). Der Offset ist eine
  ganze Zahl in Tagen — er kann negativ sein.

### Used / Planned (verbrauchte / geplante Tage)

- Datenquelle: `AbsenceService::derive_hours_for_range(year_start,
  year_end, sales_person_id, …)` — liefert pro Tag im Jahr eine
  `ResolvedAbsence { category, hours, days }`.
- Nur Tage mit `category == AbsenceCategory::Vacation` zählen; andere
  Kategorien (Sick, UnpaidLeave, …) werden übersprungen
  (`vacation_balance.rs:248-249`).
- Konflikt-Resolution (Sick > Vacation > UnpaidLeave) passiert bereits im
  `derive_hours_for_range`; hier wird nur noch gefiltert.
- Split am Stichtag `today`: `date <= today` → `used_days`,
  `date > today` → `planned_days` (`vacation_balance.rs:255-262`).
- **Tage kommen exakt aus `ResolvedAbsence.days`** — Halbtage (via
  `day_fraction`, z. B. bei Halbtags-Feiertagen) und Wochendeckelung sind
  bereits eingerechnet. Frühere naive „Kalendertag-Zählung" hätte
  Wochenenden/Feiertage falsch gezählt.
- Stunden werden parallel mitsummiert, aber aktuell nur defensiv aufbewahrt
  (`_ = (used_hours, planned_hours)` in `vacation_balance.rs:264`).

### Übertrag (`carryover_days`)

- Gelesen aus `CarryoverService::get_carryover(sales_person_id, year - 1, …)`
  (`vacation_balance.rs:270-273`). **Year-Semantik:** Ein
  `Carryover`-Eintrag mit `year = Y` speichert den Ende-von-Y-Saldo, der in
  Y+1 einfließt. Für den Übertrag *in* `year` muss also `year - 1` gefragt
  werden — dies ist ein historischer Bug-Fix, der ursprünglich `year` direkt
  weitergereicht hat (siehe Modul-Doc-Kommentar in `vacation_balance.rs:30-35`).
- Soft-gelöschte Zeilen werden ignoriert (`filter(|c| c.deleted.is_none())`,
  `vacation_balance.rs:275`).

### Rest (`remaining_days`)

```
remaining_days = entitled_effective + carryover_days − (used_days + planned_days)
                                                                 ↑             ↑
                                                              inkl. Halbtage / Konfliktresolution
```

(`vacation_balance.rs:279-280`)

### Offset-Semantik

- Genau **eine aktive Zeile pro (sales_person_id, year)**, enforced durch
  Partial-Unique-Index auf `WHERE deleted IS NULL`
  (`migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql`).
- `set` ist Upsert: gibt es bereits eine aktive Zeile, wird `offset_days` und
  `version` aktualisiert; sonst neue Row angelegt
  (`service_impl/src/vacation_entitlement_offset.rs:67-102`).
- `delete` ist Soft-Delete (`deleted = now(), new version`,
  `vacation_entitlement_offset.rs:130-139`).
- `get` liefert nur die aktive Zeile
  (`find_by_sales_person_id_and_year` mit `WHERE deleted IS NULL`).
- **Alle CRUD-Ops sind HR-gated** (D-28-06b). Ein Nicht-HR-Aufrufer bekommt
  `ServiceError::Forbidden`.

### API-Hiding für HR-only-Felder

- `VacationBalance` trägt zwei zusätzliche Felder — `offset_days` und
  `computed_entitled_days` — als `Option<..>`.
- Für HR-Aufrufer werden beide als `Some(..)` gesetzt, für self-only als
  `None` (`vacation_balance.rs:127-128`, `vacation_balance.rs:292-298`).
- **Wichtig:** `entitled_days` (der effektive Wert) ist für beide Rollen
  identisch — Offset ist niemals versteckt in der Rechnung, nur der
  *Breakdown* wird geschützt (D-28-03).

### Permission-Modell

- `VacationBalanceService::get`: **HR ∨ self**. Umsetzung via `tokio::join!`
  über `check_permission(HR)` und `verify_user_is_sales_person`
  (`vacation_balance.rs:114-128`).
- `VacationBalanceService::get_team`: **HR-only**
  (`vacation_balance.rs:147-149`). Aggregiert nur *bezahlte* Sales Persons
  (`get_all_paid`, `vacation_balance.rs:151-154`).
- `VacationEntitlementOffsetService::{get,set,delete}`: **HR-only**
  (`vacation_entitlement_offset.rs:39-41, 63-65, 116-118`).
- `CarryoverService::{get,set}`: **kein expliziter Gate** — der Context wird
  ignoriert (`_context`) und die Ops sind als internes Aggregat der Pipeline
  gedacht (Cron-Scheduler ruft mit `Authentication::Full`, Reporting mit
  `Authentication::Full`). Ein direkter REST-Endpoint auf Carryover existiert
  **nicht**.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `employee_yearly_carryover` | Jahresend-Saldo pro (Person, Jahr) — Stunden **und** Urlaubstage in *einer* Row | `sales_person_id`, `year`, `carryover_hours REAL`, `vacation INTEGER`, `deleted`, `update_process`, `update_version` (PK: `(sales_person_id, year)`) |
| `vacation_entitlement_offset` | Signierte HR-Korrektur pro (Person, Jahr) | `id BLOB PK`, `sales_person_id`, `year`, `offset_days INTEGER`, `deleted`, `update_process`, `update_version` |
| `employee_yearly_carryover_pre_cutover_backup` | **Nur historisch (Milestone 8.6 gelöscht).** Cutover-Backup vor Absence-Cutover. | — |

### Migrations

Chronologisch:

- `20241215063132_add_employee-yearly-carryover.sql` — Basistabelle
  `employee_yearly_carryover(sales_person_id, year, carryover_hours,
  created, deleted, update_process, update_version)`. Primary Key
  `(sales_person_id, year)`.
- `20241231065409_add_employee-yearly-vacation-carryover.sql` — additive Spalte
  `vacation INTEGER NOT NULL DEFAULT 0`. **Beide Werte teilen sich eine Zeile**
  — Stunden-Carryover und Urlaubstage-Carryover werden gemeinsam upserted
  (`CarryoverEntity`, `dao/src/carryover.rs:5-14`).
- `20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql`
  (Milestone 4 Cutover) — Snapshot-Table für Rollback vor Absence-Cutover;
  in Prod nie befüllt.
- `20260611000001_drop-employee-yearly-carryover-pre-cutover-backup.sql`
  (Milestone 8.6, D-04) — Löschung der Backup-Tabelle. Forward-only.
- `20260629000000_create-vacation-entitlement-offset.sql` (Milestone 28,
  VAC-OFFSET-01, D-28-01) — neue Tabelle `vacation_entitlement_offset` mit
  eigenem `id`-PK und `UNIQUE INDEX WHERE deleted IS NULL` auf
  `(sales_person_id, year)`.

### Beziehungen

- `employee_yearly_carryover.sales_person_id` → `sales_person(id)` (FK).
- `vacation_entitlement_offset.sales_person_id` → `sales_person(id)` (FK).
- Beide Tabellen sind an `SalesPerson` gebunden, nicht an
  `EmployeeWorkDetails` (also NICHT an einen bestimmten Vertragsabschnitt).
- Zwei separate Carryover-Konzepte teilen **eine** Zeile mit **zwei** Spalten
  (`carryover_hours` für die Stundenbilanz, `vacation` für Urlaubstage). Das
  ist bewusst so — beide werden vom selben nächtlichen Update-Pfad
  (`shiftplan_edit.update_carryover`, s. Kap. 4) gemeinsam neu geschrieben.

## 4. Service-API

### Traits

**`service::vacation_balance::VacationBalanceService`** — Business-Logic-Tier
(kombiniert Cross-Entity-Daten aus 4 anderen Domain-Services).

```rust
#[async_trait]
pub trait VacationBalanceService {
    type Context: …; type Transaction: …;

    async fn get(&self, sales_person_id: Uuid, year: u32,
                 context: Authentication<Self::Context>,
                 tx: Option<Self::Transaction>)
        -> Result<VacationBalance, ServiceError>;

    async fn get_team(&self, year: u32,
                      context: Authentication<Self::Context>,
                      tx: Option<Self::Transaction>)
        -> Result<Arc<[VacationBalance]>, ServiceError>;
}
```

**`service::vacation_entitlement_offset::VacationEntitlementOffsetService`** —
Basic-Tier (Entity-Manager, KEIN Domain-Service als Dep).

```rust
async fn get   (&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<Option<VacationEntitlementOffset>, ServiceError>;
async fn set   (&self, sales_person_id: Uuid, year: u32, offset_days: i32, ctx, tx)
    -> Result<VacationEntitlementOffset, ServiceError>;   // upsert
async fn delete(&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<(), ServiceError>;                          // soft-delete
```

**`service::carryover::CarryoverService`** — Basic-Tier
(`service/src/carryover.rs:49-69`).

```rust
async fn get_carryover(&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<Option<Carryover>, ServiceError>;
async fn set_carryover(&self, carryover: &Carryover, ctx, tx)
    -> Result<(), ServiceError>;                          // upsert
```

### Auth-Gates

| Op | Gate | Ref |
| --- | --- | --- |
| `VacationBalanceService::get` | HR ∨ self | `vacation_balance.rs:114-128` |
| `VacationBalanceService::get_team` | HR-only | `vacation_balance.rs:147-149` |
| `VacationEntitlementOffsetService::get/set/delete` | HR-only | `vacation_entitlement_offset.rs:39,63,116` |
| `CarryoverService::get/set_carryover` | **kein Gate** — Kontext ignoriert; darf nur intern (Scheduler / Reporting) aufgerufen werden | `carryover.rs:31,45` |

### TX-Verhalten

- Beide Services öffnen eine TX per `transaction_dao.use_transaction(tx)`,
  arbeiten atomar und committen am Ende — Standard-Pattern.
- `VacationBalanceService::get_team` iteriert über alle bezahlten Sales
  Persons und ruft `compute_balance` innerhalb *einer* TX auf
  (`vacation_balance.rs:157-161`). Bei einem Fehler an *einer* Person bricht
  die ganze Team-Abfrage ab (kein Partial-Result).
- `VacationEntitlementOffsetService::set` ist upsert-atomar (Find →
  Update/Create → Commit in einer TX).
- **Scheduler-Update** (`shiftplan_edit.update_carryover_all_employees`)
  läuft in *einer* Master-TX über alle Mitarbeiter
  (`shiftplan_edit.rs:414-440`). Fehlt ein Bericht für einen Mitarbeiter,
  bricht der gesamte Nightly-Run ab. **[Zu prüfen]** ob das gewünschtes
  Verhalten ist oder ob per-Person-TX robuster wäre.

### Cron-Trigger für Carryover

Zwei Cron-Jobs im `SchedulerServiceImpl`
(`service_impl/src/scheduler.rs:59-74`):

```rust
shiftplan_edit_service.update_carryover_all_employees(year - 1, Full, None)
shiftplan_edit_service.update_carryover_all_employees(year, Full, None)
```

- Cron-Expression: **`"0 * * * * *"`** (`scheduler.rs:45`) — **[Zu prüfen]**:
  die Notation ist 6-stellig; die Kommentare im Cluster sprechen von
  „nächtlich", der Ausdruck wirkt aber wie „jede Minute im Sekundenslot 0".
  Der Codepfad ist gleichwohl idempotent (Upsert), also unproblematisch,
  aber der Intent gehört geklärt.
- Warum zwei Jahre? Rückwirkende Änderungen am Vorjahr (z. B. neu erfasste
  Krankheit für Dezember) sollen den Vorjahres-Carryover korrigieren, ohne
  auf einen manuellen Trigger zu warten.

### `update_carryover(sales_person_id, year)` — der Rechen-Kern

`service_impl/src/shiftplan_edit.rs:362-407` (nicht in
`service_impl/src/carryover.rs`; das ist bewusst — der Rechen-Kern gehört
zur Business-Logic-Schicht des Shiftplan-Aggregats, siehe D-04 im
Service-Tier-Modell):

1. Holt `employee_report = reporting_service.get_report_for_employee(sp_id,
   year, weeks_in_year, Full, tx)`.
2. `new_carryover_hours = employee_report.balance_hours`
3. `new_vacation_entitlement = floor(vacation_entitlement − vacation_days) as
   i32` — der Urlaubstage-Übertrag ist der **abgerundete** Rest aus dem
   Reporting.
4. Beide Werte gemeinsam als `Carryover{ carryover_hours, vacation }` per
   `set_carryover` (Upsert) persistieren.

### Dependencies

- `VacationBalanceServiceImpl` (`vacation_balance.rs:58-69`):
  `AbsenceService`, `EmployeeWorkDetailsService`, `CarryoverService`,
  `SalesPersonService`, `VacationEntitlementOffsetService`,
  `PermissionService`, `ClockService`, `TransactionDao`.
- `VacationEntitlementOffsetServiceImpl` (`vacation_entitlement_offset.rs:14-22`):
  `VacationEntitlementOffsetDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao` — **keine Domain-Service-Dependency**
  (D-28-06, Anti-Zyklen-Regel: `VacationBalance` konsumiert Offset, also
  darf Offset keinen Business-Logic-Service konsumieren).
- `CarryoverServiceImpl` (`carryover.rs:14-19`): `CarryoverDao`,
  `TransactionDao` — minimal, kein Permission-Gate im Service.

## 5. REST-Endpoints

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/vacation-balance/{sales_person_id}/{year}` | Resturlaub für eine Person | — | `VacationBalanceTO` | 403 (kein HR + nicht self), 404 |
| `GET` | `/vacation-balance/team/{year}` | Aggregat über alle bezahlten Sales Persons | — | `[VacationBalanceTO]` | 403 |
| `POST` | `/vacation-entitlement-offset` | Upsert des HR-Offsets | `VacationEntitlementOffsetTO` | `VacationEntitlementOffsetTO` | 403, 500 |
| `DELETE` | `/vacation-entitlement-offset/{sales_person_id}/{year}` | Soft-Delete des Offsets | — | 204 no content | 403, 404 |

Registrierung: `rest/src/lib.rs:36, 590-591, 657-660`.

**Route-Reihenfolge:** `/vacation-balance/team/{year}` **muss vor**
`/vacation-balance/{sales_person_id}/{year}` registriert werden, sonst
matcht Axum `"team"` als Uuid-Parse → 400 statt der Team-Route
(`rest/src/vacation_balance.rs:34-42`).

`VacationBalanceTO` (`rest-types/src/lib.rs:2158-2178`):

```
sales_person_id, year,
entitled_days: f32,          // effektiv (round(base) + offset)
carryover_days: i32,
used_days: f32,
planned_days: f32,
remaining_days: f32,
offset_days: Option<i32>,            // HR-only, sonst None
computed_entitled_days: Option<f32>, // HR-only, sonst None
```

`VacationEntitlementOffsetTO` (`rest-types/src/lib.rs:2224-2230`): reines
Plain-DTO ohne `id`/`version` — der Endpoint ist upsert-basiert und der
Client identifiziert die Row über `(sales_person_id, year)`.

**Keine REST-Endpoints für `CarryoverService`.** Der Service wird nur
serverintern (Scheduler + Reporting) aufgerufen; ein direkter Client-Zugriff
auf Carryover-Rows ist nicht vorgesehen.

## 6. Frontend-Integration

- **Pages / Components:**
  - `shifty-dioxus/src/page/absences.rs:394-472` — `VacationEntitlementCard`
    (5 Stat-Kacheln, self/team-Modus, HR-Toggle für Inline-Offset-Editor).
  - `shifty-dioxus/src/page/absences.rs:750-…` — `VacationPerPersonList`
    (HR-Aggregat-Liste).
  - `shifty-dioxus/src/component/employee_view.rs:186-442` — Carryover-Balance
    und Vacation-Carryover werden als Read-only-Felder im Employee-Details-View
    angezeigt (Basis: HR-Employee-Report, nicht der VacationBalance-Endpoint).
- **Service:** `shifty-dioxus/src/service/vacation_balance.rs` — Coroutine
  mit `VacationBalanceAction`-Kanal, Store für Self und Team.
- **State:** `shifty-dioxus/src/state/vacation_balance.rs` — Frontend-Domain
  (`From<&VacationBalanceTO>`).
- **API-Client:** `shifty-dioxus/src/api.rs:638-694` — `get_vacation_balance`
  und `get_team_vacation_balance`.
- **Loader:** `shifty-dioxus/src/loader.rs:841-860` — Dünne Wrapper.
- **i18n-Keys (`shifty-dioxus/src/i18n/mod.rs:491-508`):**
  `VacationCardSelfTitle`, `VacationCardSelfSubtitle`, `VacationCardTeamTitle`,
  `VacationCardTeamSubtitle`, `VacationStatContract`, `VacationStatCarryover`,
  `VacationStatUsed`, `VacationStatPending`, `VacationStatRemaining`,
  `VacationEntitlementHero`, `VacationDaysRemaining`, `VacationPerPersonHeader`,
  `VacationPerPersonShowAll/Less`, `VacationOffsetLabel`,
  `VacationOffsetComputedLabel`.
- **Proxy** (`shifty-dioxus/Dioxus.toml:99-102`):
  - `backend = "http://localhost:3000/vacation-balance"`
  - `backend = "http://localhost:3000/vacation-entitlement-offset"`
  Beide sind eingetragen — Standardfehler bei neuen Endpoints
  („Dioxus.toml Proxy vergessen") ist hier NICHT aktiv.

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektion
[Stundenkonto](../domain/edge-cases.md#1-stundenkonto).

- **Rückwirkende Änderung im abgeschlossenen Jahr driftet den Carryover.**
  Wird eine `AbsencePeriod` im letzten Dezember nachträglich eingetragen
  (z. B. Krankheitsnachweis für alten Zeitraum), stimmt der Vorjahres-
  Carryover nicht mehr. Der Scheduler adressiert das mit dem
  „(year - 1)"-Job (`scheduler.rs:60`), der pro Tick beide Jahre neu
  berechnet. Solange der Backend-Prozess läuft, konvergiert der Carryover
  von selbst — nach einem Server-Restart erst mit dem nächsten Cron-Tick.
- **Mitte-Jahr-Neueinstellung.** `vacation_days_for_year` liefert einen
  aliquoten `f32` (z. B. 15,25). Erst nach Aggregation über alle
  Vertragsabschnitte wird `.round()` angewandt
  (`vacation_balance.rs:195-200`). Der HR-Offset wird DANACH addiert
  (D-28-02) — ein `-1`-Offset kann also nicht durch Rundung
  „verpuffen".
- **Jahresrollover-Race.** Läuft der Cron am 01.01. genau nach dem
  Datumswechsel, wird zunächst `update_carryover_all_employees(prev_year,
  …)` (finaler Snapshot) und dann `(current_year, …)` (initialer Snapshot)
  aufgerufen. `set_carryover` ist upsert, also idempotent — mehrfache
  Ausführungen sind ungefährlich. Ein manueller `POST` auf Absence während
  des Cron-Ticks kann jedoch zu einem „veralteten" Snapshot für ein paar
  Sekunden führen (bis zum nächsten Tick).
- **`representative_hours_per_day` bei Vertragswechsel mitten im Jahr.**
  Modell A (Decision 2026-06-12) wählt einen *einzigen* repräsentativen
  `hours_per_day` pro Jahr — den jüngsten Vertragsabschnitt, der `year`
  berührt (`vacation_balance.rs:78-97`). Bei zwei Verträgen mit
  unterschiedlichem `hours_per_day` ist die Stunden→Tage-Umrechnung eine
  Approximation. Aktuell wird `hours_per_day` nur noch defensiv berechnet;
  die Tageszahlen kommen exakt aus `ResolvedAbsence.days`.
- **Carryover-Year-Off-by-one.** Historischer Bug: die alte Implementierung
  las `carryover(sp, year)` statt `carryover(sp, year - 1)`. Ergebnis: der
  Übertrag *aus* dem laufenden Jahr (den es noch gar nicht gab) wurde
  gelesen → immer 0. Der Modul-Doc-Kommentar in `vacation_balance.rs:30-35`
  beschreibt den Fix, Test `carryover_read_uses_prior_year`
  (`test/vacation_balance.rs:892`) hält Regression fest.
- **Offset ganztägig, nie fraktional.** `offset_days: i32` — es gibt keine
  Möglichkeit, einen halben Tag als Offset einzutragen. Ist bewusst so, um
  API-Hiding einfach zu halten (`i32` serialisiert konsistent auch bei
  self-only, dort ist es aber `None`).
- **Kein Permission-Gate im `CarryoverService`.** Der Service akzeptiert
  jeden `Authentication<Ctx>` und ignoriert ihn. Da es keinen REST-Endpoint
  gibt, ist das aktuell akzeptabel — sollte jedoch beim Auftauchen eines
  Admin-UIs für Carryover-Overrides nachgezogen werden.
- **`get_report_for_employee` als Abhängigkeit.** Der Rechen-Kern
  (`shiftplan_edit.update_carryover`) baut auf dem *vollen*
  Employee-Report. Änderungen an der Report-Formel wirken sich sofort auf
  den geschriebenen `carryover_hours`/`vacation`-Wert aus. Es gibt kein
  Snapshot-Schema-Versioning für die Carryover-Tabelle. **[Zu prüfen]**
  ob das bewusst so ist oder ob analog zum Billing-Period-Snapshot
  (siehe `CLAUDE.md` — Snapshot-Schema-Versioning) auch für Carryover eine
  Version-Spalte gebraucht wird.

## 8. Tests

- **Unit — VacationBalance** (`service_impl/src/test/vacation_balance.rs`,
  1 113 Zeilen, mock-basiert):
  - Happy Path self (`get_returns_entitlement_minus_used_minus_planned`,
    Z. 238),
  - Happy Path HR (`get_with_hr_succeeds`, Z. 314),
  - AuthZ (`get_other_sales_person_without_hr_is_forbidden` Z. 364,
    `get_team_without_hr_is_forbidden` Z. 393),
  - Team-Aggregat (`get_team_aggregates_per_paid_sales_person`, Z. 409),
  - Edge (`get_with_no_active_contract_returns_zero_entitlement` Z. 489,
    `get_rounds_aliquot_entitlement_to_whole_number` Z. 542,
    `get_year_without_carryover_returns_zero_carryover` Z. 593),
  - Halbtag (`half_day_vacation_counts_as_half_day`, Z. 667),
  - Days-Field-Direct (`part_time_contract_used_days_come_from_days_field`,
    Z. 693),
  - Used/Planned-Split (`active_period_splits_used_and_planned_at_today`,
    Z. 714),
  - Kategorie-Filter (`non_vacation_categories_are_ignored`, Z. 741),
  - Carryover-Year-Semantik (`get_carryover_is_called_with_previous_year`
    Z. 796, `carryover_from_previous_year_is_included_in_balance` Z. 835,
    `carryover_read_uses_prior_year` Z. 892),
  - Offset (`offset_calc` Z. 975, `offset_delta` Z. 1013,
    `offset_api_hiding` Z. 1033).
- **Unit — VacationEntitlementOffset**
  (`service_impl/src/test/vacation_entitlement_offset.rs`, 331 Zeilen):
  `get`/`set`/`delete` Happy Path, HR-Gate-Denial, Upsert-Semantik,
  Soft-Delete-Semantik.
- **Unit — Carryover** (`service_impl/src/test/carryover.rs`, 187 Zeilen):
  `get_carryover_found/not_found`, DAO-Error-Propagation,
  `set_carryover_success`, DAO-Error auf Set.
- **Frontend** (`shifty-dioxus/src/page/absences.rs:3531+`): Snapshot-Tests
  für `VacationEntitlementCard` mit `selected_person` (HR-Detail-Ansicht).
- **Bekannte Lücken:**
  - Kein Integration-Test (In-Mem-SQLite Roundtrip) für den
    Scheduler-Zyklus `update_carryover_all_employees` — die Interaktion
    Reporting↔Carryover↔VacationBalance ist nur mock-basiert abgedeckt.
  - Kein Test für den Race-Fall „Absence-Änderung während laufendem
    Cron-Tick" (praktisch schwer reproduzierbar; **[Zu prüfen]**).
  - **[Zu prüfen]** Regressionstest für die Off-by-one im
    Vertragsstart-Abzug (D-28-04): `vacation_days_for_year` bei
    `from_date = 01.01.` — der Test-Nachweis liegt in
    `service/src/employee_work_details.rs:287+ (mod vacation_days_for_year_tests)`
    laut Grep, aber die genaue Coverage sollte bei Änderungen an
    `vacation_days_for_year` geprüft werden.

## 9. Historie & Kontext

- **Ende 2024 — Basis-Carryover.**
  `20241215063132_add_employee-yearly-carryover.sql` legt die
  Stundenbilanz-Carryover-Tabelle an (nur `carryover_hours`).
  `20241231065409_add_employee-yearly-vacation-carryover.sql` (2 Wochen
  später) ergänzt additiv die `vacation`-Spalte. Es war eine bewusste
  Design-Entscheidung, Stunden- und Urlaubstage-Carryover in **einer** Row
  zu halten, weil beide vom gleichen Nightly-Job aus dem gleichen Report
  geschrieben werden.
- **Milestone 4 — Cutover-Backup.** Kurz vor dem Absence-Cutover wurde ein
  Backup-Table (`…_pre_cutover_backup`) angelegt, um im Notfall auf den
  pre-cutover-Zustand zurückzurollen. In Prod wurde er nie befüllt und ist
  in Milestone 8.6 (D-04) wieder entfernt — forward-only.
- **Milestone 8 — Vacation-Balance-Endpoint.** Der Business-Logic-Service
  `VacationBalanceService` (D-04 in `08-CONTEXT.md`) wurde als
  Aggregations-Layer eingeführt, um die Frontend-`VacationEntitlementCard`
  und `VacationPerPersonList` mit einem einzigen Roundtrip zu bedienen.
  Wave-4 lieferte den Endpoint, Wave-5 den Frontend-Wire-Up.
- **Milestone 28 — Vacation-Entitlement-Offset (VAC-OFFSET-01).**
  Fach-Anforderung: HR muss den Vertragsanspruch einzelfallweise korrigieren
  können, ohne den Vertrag zu ändern (Prämien, Sonderabzüge, gerichtliche
  Anordnungen). Design-Entscheidungen:
  - D-28-01: Eigene Tabelle `vacation_entitlement_offset` mit `id`-PK
    (nicht als Spalte an `employee_yearly_carryover`, um Domain-Grenzen zu
    wahren).
  - D-28-02: Offset NACH `.round()` addieren.
  - D-28-03: API-Hiding für Nicht-HR — nur der effektive Wert ist sichtbar.
  - D-28-04: Fix des Off-by-one im Vertragsstart-Abzug.
  - D-28-06/06b: Basic-Tier für Offset-Service (kein Domain-Dep, sonst
    Zyklus mit VacationBalance).
  - D-28-07: Frontend-Inline-Editor nur auf HR-Detail-Pfad, nie in der
    Employee-Selbstansicht.
- **Kontext-Reads:**
  - `.planning/phases/08-…` — Vacation-Balance-Foundation (Business-Logic-
    Service-Klassifizierung, Test-Coverage-Anforderungen).
  - `.planning/phases/28-…` — VAC-OFFSET-01-Design.
  - `.planning/phases/*-51*` — Toggle-Service-Full-Context-Bypass (Auswirkung
    auf `derive_hours_for_range`, das VacationBalance intern konsumiert).

---

**Fazit:** F06 löst die Trias Anspruch → Verbrauch → Übertrag pro
Mitarbeiter/Jahr mit einem stundenbasierten Kern (`derive_hours_for_range`)
und einem HR-Ganzzahl-Offset ohne die Rundung zu zerstören. Der Cron-getriebene
Carryover-Rewrite hält Vor- und Aktuellesjahr kontinuierlich konsistent — ein
Snapshot-Versioning wie bei Billing-Period fehlt derzeit bewusst.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
