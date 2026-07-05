# Feature: Booking (F03)

> **Kurzform:** Ein Booking ordnet eine Sales Person einem Slot in einer
> bestimmten Kalenderwoche/Jahr zu — der zentrale Schreibvorgang im
> Schichtplan-Editor. Ein Booking-Log liefert dazu den lesenden Audit-Trail
> pro Woche.

**Cluster-ID:** F03
**Status:** produktiv
**Erstmalig eingeführt:** Booking-Tabelle 2024-05 (Migration `20240507063704`),
User-Tracking 2025-01 (Migrations `20250115000000/01`), Konflikt-aware
Persist-Pfad in Phase 3 (v2.x), Paid-Limit-Hard-Enforcement in Phase D-24,
Wochen-Sperre-Gate in Phase 40, Absence-Konflikt-Flag in v2.2.1.
**Zuständige Crates:** `service::booking`, `service::booking_information`,
`service::booking_log`, `service_impl::booking`, `service_impl::booking_information`,
`service_impl::booking_log`, `dao::booking`, `dao::booking_log`,
`dao_impl_sqlite::booking`, `dao_impl_sqlite::booking_log`, `rest::booking`,
`rest::booking_information`, `rest::booking_log`, `rest-types::BookingTO` /
`BookingLogTO` / `BookingConflictTO` / `WeeklySummaryTO`.

Verwandt: `service::shiftplan_edit` (Konflikt-aware Persist-Pfad, Wochen-Sperre)
und `service::warning` (Emissions-Kanal der Konflikte). Siehe F02 (Slot &
Shiftplan) und F05 (Absence).

---

## 1. Was ist das? (Fachlich)

Ein **Booking** ist die eigentliche Zuweisung "Person X arbeitet an Slot Y in
Kalenderwoche N / Jahr J". Es ist das schmale, technisch billige Objekt, an
dem im Editor tatsächlich geschraubt wird — alle darüber liegenden Metriken
(Wochen-Summary, Berichte, Balance, Warnings) leiten sich aus dieser einen
Tabelle ab.

Ein **Booking-Log** ist die read-only, denormalisierte Woche-Ansicht auf die
gleichen Zeilen inkl. der Sichtbarmachung von Soft-Deletes und
User-Tracking (wer hat's angelegt, wer hat's ausgetragen). Er dient dem
Shiftplanner als Audit-Trail und Recherche-Werkzeug im Frontend.

**Beispiel-Workflow aus User-Sicht:**

1. Shiftplanner öffnet den Schichtplan für Woche 27/2026.
2. Klickt auf einen leeren Slot → Auswahl einer Sales Person.
3. Der Konflikt-aware POST auf `/shiftplan-edit/booking` persistiert den
   Eintrag und liefert ggf. Warnings (Booking auf Urlaubstag, auf einem
   „Ich-nicht-verfügbar“-Tag, Paid-Limit überschritten).
4. Warnings werden inline im Editor angezeigt — nicht als Bestätigungs-
   Dialog (User-Präferenz).
5. Zum Rausbuchen: Klick auf den Eintrag → `DELETE /booking/{id}` läuft
   über `ShiftplanEditService::delete_booking` (inkl. Wochen-Sperre-Gate).
6. Über den Reiter „Booking-Log“ sieht der Planner die vollständige
   Historie der Woche inkl. der ausgetragenen Bookings.

## 2. Fachliche Regeln

- **Eindeutigkeit pro Woche:** Ein Tupel `(sales_person_id, slot_id,
  calendar_week, year)` darf nur einmal aktiv existieren.
  `BookingServiceImpl::create` prüft das über
  `booking_dao.find_by_booking_data` und wirft `ValidationFailureItem::Duplicate`
  (`service_impl/src/booking.rs:241–253`).
- **Fremdschlüssel-Validität:** `sales_person_id` und `slot_id` müssen
  existieren; Verletzung → `IdDoesNotExist`
  (`service_impl/src/booking.rs:216–239`).
- **Kalenderwoche-Range:** `calendar_week` muss `1..=53` sein
  (`service_impl/src/booking.rs:210–215`).
- **Keine Client-IDs / -Versionen / -Timestamps:** `id`, `version` und
  `created` müssen beim Create leer sein (`IdSetOnCreate`, `VersionSetOnCreate`,
  `InvalidValue("created")`, `service_impl/src/booking.rs:191–201`).
- **Shiftplan-Eligibility:** Wenn der Slot einem Shiftplan zugeordnet ist,
  prüft der Service via `SalesPersonShiftplanService::is_eligible`; ohne
  Freigabe → `Forbidden` (`service_impl/src/booking.rs:260–277`).
- **User-Tracking (2025-01):** `created_by`/`deleted_by` werden serverseitig
  aus dem authentifizierten Kontext gefüllt. Bei internem
  `Authentication::Full` fällt der Service auf `booking.created_by` aus der
  Aufrufer-Payload zurück, letzter Fallback ist das Sentinel `"system"`
  (`service_impl/src/booking.rs:281–300` und `423–434`).
- **Wochen-Sperre-Gate:** Alle Schreibpfade — inkl. `delete_booking` — laufen
  über `ShiftplanEditService::assert_week_not_locked` (`shiftplan_edit.rs:598–604`).
  Reine `shiftplanner`-Rechte reichen **nicht**, um das Gate zu umgehen — nur
  `shiftplan.edit` (D-40-02).
- **Paid-Limit (D-24):** Hat der Slot ein `max_paid_employees`, prüft der
  Konflikt-aware Pfad:
  1. Bei aktivem Toggle `paid_limit_hard_enforcement` und Nicht-Shiftplanner:
     hartes `ServiceError::PaidLimitExceeded { current, max }`
     (`shiftplan_edit.rs:618–652`).
  2. Andernfalls post-persist Soft-Warning
     `Warning::PaidEmployeeLimitExceeded` (`shiftplan_edit.rs:758–779`).
- **`min_resources`-Signal:** Führt selbst kein Booking-Gate; wird im
  Reporting/Ampel (F02 Slot & Shiftplan) als Soll-Wert für „Slot
  unterbesetzt“ herangezogen. **[Zu prüfen]** Ob im Editor eine
  Frontend-Warnung an das Backend gekoppelt ist — im Booking-Service selbst
  ist `min_resources` nicht Teil der Validierung.
- **Absence-Konflikt (v2.2.1):** `book_slot_with_conflict_check` legt pro
  überlappender AbsencePeriod ein `Warning::BookingOnAbsenceDay` an
  (`shiftplan_edit.rs:708–725`); Halbtags-Absences werden schweigend geduldet
  (D-08.3-05).
- **Manual-Unavailable-Konflikt:** Analog wird pro Wochen-Tag ein
  `Warning::BookingOnUnavailableDay` emittiert
  (`shiftplan_edit.rs:726–739`).
- **Copy-Week ist idempotent-ish:** `copy_week` verwirft aus der Quelle alle
  Bookings, deren `(sales_person_id, slot_id)` in der Zielwoche schon liegt,
  bevor kopiert wird (`service_impl/src/booking.rs:351–370`) — Doppelbuchungen
  können so nicht entstehen.
- **Berechtigungs-Matrix (Kurzform):**
  - Lesen (`get_all`, `get`, `get_for_week`, `get_for_slot_id_since`):
    `SHIFTPLANNER_PRIVILEGE` ∨ `SALES_PRIVILEGE`.
  - Schreiben (`create`, `delete`): Shiftplanner **oder** der authentifizierte
    User ist die zugewiesene Sales Person (`check_booking_permission`,
    `service_impl/src/booking.rs:34–68`).
  - `copy_week`: strikt Shiftplanner.
  - Konflikt-aware Persist (`shiftplan_edit`-Pfad): Shiftplanner ∨ self,
    zusätzlich Wochen-Sperre.
  - Booking-Log (`booking_log`): strikt Shiftplanner.

## 3. Datenmodell

### Tabellen & Views

| Objekt | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `booking` (Table) | Aktive und soft-gelöschte Bookings | `id BLOB(16) PK`, `sales_person_id BLOB(16)`, `slot_id BLOB(16)`, `calendar_week INT`, `year INT`, `created TEXT`, `deleted TEXT NULL`, `created_by TEXT NULL`, `deleted_by TEXT NULL`, `update_timestamp TEXT`, `update_process TEXT`, `update_version BLOB(16)` |
| `bookings_view` (View) | UUID-formatierte Sicht mit Join auf `sales_person` und `slot` — Grundlage des Booking-Logs | `booking_hex`, `sales_person_hex`, `slot_hex`, `name`, `year`, `calendar_week`, `day_of_week`, `time_from`, `time_to`, `created`, `deleted`, `created_by`, `deleted_by` |

Beim DAO-Lesen des aktiven Datenbestands filtert `booking_dao` immer über
`WHERE deleted IS NULL` (`dao_impl_sqlite/src/booking.rs:62–66`). Der
Booking-Log-DAO liest bewusst **inklusive** Soft-Deletes aus
`bookings_view`, um ausgetragene Bookings sichtbar zu machen
(`dao_impl_sqlite/src/booking_log.rs:131`).

### Migrations

Chronologische Liste:

- `migrations/sqlite/20240507063704_add-booking.sql` — Basistabelle `booking`.
- `migrations/sqlite/20240728155625_add-bookings-view.sql` — read-only View
  `bookings_view` mit UUID-Formatter + Slot/SalesPerson-Join.
- `migrations/sqlite/20250115000000_add-user-tracking-to-booking.sql` — fügt
  `created_by TEXT NULL`, `deleted_by TEXT NULL` hinzu. Vorheriger Bestand
  bleibt mit NULL erhalten (bewusst, `dao_impl_sqlite/src/booking_log.rs:35–39`).
- `migrations/sqlite/20250115000001_update-bookings-view-add-user-tracking.sql`
  — `DROP VIEW`/`CREATE VIEW` zieht die neuen Spalten in die
  `bookings_view` nach.

### Beziehungen

`booking` ist Kind zweier Aggregate:

```
sales_person ─┐
              ├──< booking >── slot ── shiftplan
special_day ──┘  (view join)
```

- FK-Referenz auf `sales_person.id` und `slot.id` (in SQLite nicht formal
  erzwungen, siehe Kommentar in `20250115000000_...sql`).
- `created_by` / `deleted_by` referenzieren logisch `user.name` — laut
  Migration-Kommentar bewusst nur auf Anwendungsebene gekoppelt.
- Der `bookings_view`-Join denormalisiert Person + Slot in eine Zeile,
  damit der Booking-Log ohne Backend-Joins auskommt.

## 4. Service-API

Zwei Basic-Services und ein Business-Logic-Service:

- **Basic:** `BookingService` (Aggregat-Manager der `booking`-Tabelle),
  `BookingLogService` (Read-Aggregat auf `bookings_view`).
- **Business-Logic:** `BookingInformationService` (aggregiert `Slot` +
  `SalesPerson` + Absence + Working-Hours zu Konflikt-Listen und
  Wochen-Summaries).

Für die Schreibpfade mit Cross-Source-Warnings ist zusätzlich
`ShiftplanEditService::book_slot_with_conflict_check` /
`copy_week_with_conflict_check` / `delete_booking` involviert (siehe F02).

### Trait `BookingService` (Basic)

`service/src/booking.rs:62–114`

```rust
#[async_trait]
pub trait BookingService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_all(&self, ctx, tx) -> Result<Arc<[Booking]>, ServiceError>;
    async fn get(&self, id: Uuid, ctx, tx) -> Result<Booking, ServiceError>;
    async fn get_for_week(&self, cw: u8, year: u32, ctx, tx) -> …;
    async fn get_for_slot_id_since(&self, slot_id: Uuid, year: u32, cw: u8, ctx, tx) -> …;
    async fn create(&self, booking: &Booking, ctx, tx) -> Result<Booking, ServiceError>;
    async fn copy_week(&self, from_cw: u8, from_year: u32, to_cw: u8, to_year: u32, ctx, tx) -> …;
    async fn delete(&self, id: Uuid, ctx, tx) -> Result<(), ServiceError>;
}
```

Auth-Gates (siehe `service_impl/src/booking.rs`):

| Methode | Permission |
| --- | --- |
| `get_all` / `get` / `get_for_week` / `get_for_slot_id_since` | `SHIFTPLANNER` ∨ `SALES` |
| `create` | Shiftplanner ∨ authentifizierter User ist die Sales Person (`check_booking_permission`, Zeile 34–68). |
| `copy_week` | `SHIFTPLANNER` (strikt). |
| `delete` | Shiftplanner ∨ self, zusätzlich Eligibility-Check gegen `SalesPersonShiftplanService`. |

TX-Verhalten:

- Jede Methode öffnet bei `tx=None` eine eigene Transaktion via
  `TransactionDao::use_transaction` und committet am Ende.
- `create` validiert komplett vor jedem Schreibzugriff; Rollback wenn eine
  der Validierungen oder der DAO-Insert fehlschlägt.
- `copy_week` läuft in **einer** Outer-TX und ruft `create` für jedes
  Quell-Booking; jede Duplikat-/Eligibility-Verletzung bricht die gesamte
  Kopie ab (kein Teil-Rollback, kein Teil-Commit).
- Cross-Service-Reads (`sales_person_service.exists`, `slot_service.exists`,
  `slot_service.get_slot`, `sales_person_shiftplan_service.is_eligible`)
  laufen unter `Authentication::Full` innerhalb derselben TX, damit sie
  denselben Read-Snapshot sehen.

Dependencies (`service_impl/src/booking.rs:20–31`):

- DAOs: `BookingDao`, `TransactionDao`.
- Andere Services: `PermissionService`, `ClockService`, `UuidService`,
  `SalesPersonService`, `SlotService`, `SalesPersonShiftplanService`.

Der Service ist zwar strikt betrachtet ein Basic-Service (Aggregat-Manager
für `booking`), zieht aber wegen Eligibility- und User-Trace-Anforderung
mehrere andere Domain-Services. Diese Abhängigkeiten sind einseitig — kein
konsumierender Domain-Service verweist zurück auf Booking, sodass die
Tier-Konvention (siehe `CLAUDE.md`) gewahrt bleibt.

### Trait `BookingInformationService` (Business-Logic)

`service/src/booking_information.rs:87–113`

```rust
async fn get_booking_conflicts_for_week(year, week, ctx, tx) -> Arc<[BookingInformation]>;
async fn get_weekly_summary(year, ctx, tx)                     -> Arc<[WeeklySummary]>;
async fn get_summery_for_week(year, week, ctx, tx)             -> WeeklySummary;
```

Auth-Gates:

- `get_booking_conflicts_for_week`: strikt `SHIFTPLANNER`
  (`service_impl/src/booking_information.rs:155–157`).
- `get_weekly_summary`, `get_summery_for_week`: `SHIFTPLANNER` ∨ `SALES`;
  darüber hinaus wird `is_shiftplanner` berechnet, um die
  `working_hours_per_sales_person`-Detailliste **nur** für Planer
  auszugeben (`booking_information.rs:274–278`).

**Full-Bypass durchgereicht:** Diese aggregierende Schicht ruft ihre
inneren Kollegen (`BookingService::get_for_week`,
`SalesPersonService::get_all`, `SlotService::get_slots_for_week_all_plans`,
`AbsenceService::find_all`/`find_overlapping_for_booking`,
`SalesPersonUnavailableService::get_by_week`,
`SpecialDayService::get_by_week`, `ReportingService::get_week`,
`ShiftplanReportService::extract_shiftplan_report_for_week`,
`ToggleService::…`) durchgängig mit `Authentication::Full`
(z.B. `booking_information.rs:160–169, 205–214, 283–294, 300–303, 519–528`).
Die äußere Permission wurde am Eingang geprüft; interne Reads sind
Aggregat-Details. Das ist der Standard-Pfad, an dem `ToggleService` seinen
`Authentication::Full`-Bypass (siehe MEMORY: „ToggleService Full-Context-
Bypass“) tatsächlich nutzt.

TX-Verhalten:

- `get_weekly_summary` läuft in einer Outer-TX über 1..=52/53(+3) Wochen des
  Jahres. Alle Loads werden vor der Wochenschleife einmalig gezogen
  (Load-once-Pattern: `all_work_details`, `all_absences`, `volunteer_ids`,
  `active_from`-Toggle) — siehe Kommentar „Pitfall 4“ und
  `booking_information.rs:290–310`.
- Für die Absence-Konflikt-Sicht (`get_booking_conflicts_for_week`) wird
  pro betroffenem Sales Person genau ein Absence-Lookup gemacht, dann
  in-memory pro Booking gefiltert (`booking_information.rs:187–213`).

Dependencies (`booking_information.rs:111–135`): `ShiftplanReportService`,
`SlotService`, `BookingService`, `SalesPersonService`,
`SalesPersonUnavailableService`, `ReportingService`, `SpecialDayService`,
`ToggleService`, `EmployeeWorkDetailsService`, `AbsenceService`,
`PermissionService`, `ClockService`, `UuidService`, `TransactionDao`.

### Trait `BookingLogService` (Basic, read-only)

`service/src/booking_log.rs:26–37`

```rust
async fn get_booking_logs_for_week(year, cw, ctx, tx) -> Arc<[BookingLog]>;
```

Auth-Gate: strikt `SHIFTPLANNER_PRIVILEGE`
(`service_impl/src/booking_log.rs:33–35`). Der Service ist ein reiner
Read-Mapper von `BookingLogEntity` auf `BookingLog` inkl. Soft-Deletes und
User-Tracking.

Dependencies: `BookingLogDao`, `PermissionService`, `TransactionDao`.

## 5. REST-Endpoints

Mount-Punkte (`rest/src/lib.rs:641–649`): `/booking`, `/booking-information`,
`/booking-log`, `/shiftplan-edit/booking` (Konflikt-aware Persist,
technisch im F02-Cluster verortet).

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/booking/` | Alle aktiven Bookings | — | `Vec<BookingTO>` | 401/403 |
| `GET` | `/booking/week/{year}/{cw}` | Bookings einer Woche | — | `Vec<BookingTO>` | 401/403 |
| `GET` | `/booking/{id}` | Einzelnes Booking | — | `BookingTO` | 404 |
| `POST` | `/booking/` | Legacy-Create (ohne Konflikt-Aggregation) | `BookingTO` | `BookingTO` | 400 (Validierung), 403 (Eligibility), 409 (Duplicate) |
| `DELETE` | `/booking/{id}` | Deletion via `ShiftplanEditService::delete_booking` (Wochen-Sperre-Gate) | — | 200 | 403 (Wochen-Sperre / Eligibility), 404 |
| `POST` | `/booking/copy?from_year&from_week&to_year&to_week` | Nicht-konflikt-aware Wochen-Kopie | — | 200 | 400, 403 |
| `POST` | `/shiftplan-edit/booking` | Konflikt-aware Persist mit Warnings | `BookingTO` | `BookingCreateResultTO` | 409 `PaidLimitExceeded` (D-24-08), Wochen-Sperre |
| `GET` | `/booking-information/conflicts/for-week/{year}/{week}` | Konflikte (Unavailable + Absence-Overlap) | — | `Vec<BookingConflictTO>` | 403 |
| `GET` | `/booking-information/weekly-resource-report/{year}` | Jahres-Roll-Out über alle KWs | — | `Vec<WeeklySummaryTO>` | 403 |
| `GET` | `/booking-information/weekly-resource-report/year/{week}` | Einzelwoche | — | `WeeklySummaryTO` | 403 |
| `GET` | `/booking-log/{year}/{week}` | Audit-Trail-Zeilen der Woche | — | `Vec<BookingLogTO>` | 403 |

**[Zu prüfen]** Der zweite Weekly-Report-Pfad heißt technisch
`/booking-information/weekly-resource-report/year/{week}` — der
Path-Parameter für das Jahr fehlt in der aktuellen Route (`rest/src/booking_information.rs:25–27`).
Der Handler erwartet `(year, week)`, aber das Jahr wird über den
Query/Path-Match als String `"year"` gebunden. Wirkt wie ein Copy-Paste-
Artefakt — sollte gegen die tatsächliche URL-Verwendung im Frontend
verifiziert werden.

DTOs (`rest-types/src/lib.rs`):

- `BookingTO` (Zeile 103–122): identisch zum Domain-`Booking`, inkl.
  `created_by`/`deleted_by`, `$version`-Rename.
- `BookingLogTO` (Zeile 158–177): denormalisiert (`name`, `time_from`,
  `time_to`, `day_of_week`) — kein Slot-/Person-Objekt.
- `BookingConflictTO` (Zeile 937–953): `booking` + `slot` + `sales_person`.
- `WeeklySummaryTO` (Zeile 989–1036): Wochen-Aggregat mit Paid-,
  Volunteer-, Committed-Voluntary-Hours (CVC-Serie) und Per-Day-Kapazitäten.

REST-Handler-Files:

- `rest/src/booking.rs` — GET/POST/DELETE für die reine Booking-Tabelle
  (DELETE ruft absichtlich `ShiftplanEditService::delete_booking`, damit
  das Wochen-Sperre-Gate greift, `booking.rs:158–173`).
- `rest/src/booking_information.rs` — Konflikte + Weekly-Report.
- `rest/src/booking_log.rs` — Audit-Trail (voll mit `#[utoipa::path]`).

## 6. Frontend-Integration

- **Pages:** `shifty-dioxus/src/page/shiftplan.rs` ist der zentrale Editor.
  Er integriert Booking-Konflikte (`BOOKING_CONFLICTS_STORE`,
  `booking_conflict.rs`), Booking-Log (`BOOKING_LOG_STORE`,
  `booking_log.rs`) und ruft den Konflikt-aware Persist-Endpoint
  (`api::book_slot_with_conflict_check`).
- **Services:** `shifty-dioxus/src/service/booking_conflict.rs`,
  `shifty-dioxus/src/service/booking_log.rs`.
- **Components:** `shifty-dioxus/src/component/booking_log_table.rs`,
  `warning_list.rs` (Warnings als Inline-Banner statt Dialog, siehe
  MEMORY-Feedback).
- **API-Wrapper:** `shifty-dioxus/src/api.rs`
  - `book_slot_with_conflict_check` → `POST /shiftplan-edit/booking`
    (Zeile 202–235).
  - `remove_booking` → `DELETE /booking/{id}` (Zeile 237–241).
  - `get_booking_conflicts_for_week` → `GET /booking-information/conflicts/for-week/{year}/{week}` (Zeile 900–914).
  - `get_booking_log` → `GET /booking-log/{year}/{week}` (Zeile 915–927).
- **i18n-Keys:** u.a. `ConflictBookingsHeader` (Editor-Panel-Titel);
  Filter-Labels für den Booking-Log werden pro Locale in
  `shifty-dioxus/src/i18n/{en,de,cs}.rs` geführt.
- **Proxy:** `shifty-dioxus/Dioxus.toml` mappt `/booking`,
  `/booking-information`, `/booking-log` sowie `/shiftplan-edit`
  auf `http://localhost:3000/*`. Neue Booking-Endpunkte **immer** hier
  ergänzen — sonst 404 im `dx-serve`-Dev-Modus (MEMORY-Feedback,
  Phase 28/49).

## 7. Randfälle

Alle allgemeinen Auth-/TX-/Time-Kanten in
[`../domain/edge-cases.md`](../domain/edge-cases.md). Booking-spezifisch:

- **Slot-Split doppelt zählen (Phase 23):** Beim Split eines Slots via
  `modify_slot`/`modify_slot_single_week` wandern Bookings auf das/die
  neue(n) Segment(e); ohne Atomarität + explizite Test-Guard entstünde
  eine Doppelzählung in Reports/Balance. Die Umbuchung läuft daher in
  **einer** Transaktion (siehe F02) — Regressions-Test siehe
  `service_impl/src/test/booking.rs`.
- **Löschen eines gebuchten Slots:** `modify_slot`/`remove_slot` löscht
  implizit gebundene Bookings und trägt für die Audit-Zeile `deleted_by`
  aus dem Aufrufer-Kontext ein. Läuft der Aufruf unter
  `Authentication::Full` (z.B. aus einem System-Job), greift der
  `"system"`-Fallback in `BookingServiceImpl::delete`
  (`service_impl/src/booking.rs:432–434`). Das Booking-Log zeigt diese
  Zeilen weiterhin an.
- **Absence-Konflikt (v2.2.1):** Bookings, die in eine aktive
  AbsencePeriod fallen, sind **kein** harter Fehler beim Legacy-`POST
  /booking`. Auf dem Konflikt-aware Pfad wird eine Warning
  (`BookingOnAbsenceDay`) emittiert; Halbtags-Absences (D-08.3-05) werden
  schweigend geduldet (`shiftplan_edit.rs:716–718`).
- **User-Tracking-Konsistenz:** `created_by`/`deleted_by` sind `NULL`
  ausschließlich für Zeilen, die vor Migration `20250115000000` geschrieben
  wurden. Live-Pfade tragen entweder den authentifizierten User, den
  vom Caller mitgegebenen `booking.created_by` oder das Sentinel
  `"system"` ein (`service_impl/src/booking.rs:281–300` und `423–434`).
  DAO-Kommentar: `dao_impl_sqlite/src/booking_log.rs:35–39`.
- **`copy_week` überspringt bereits vorhandene Ziel-Bookings:** Kein
  Doppel-Insert, kein Fehler; einfach silent-skip
  (`service_impl/src/booking.rs:351–360`). **[Zu prüfen]** Ob das
  aus Sicht der UX gewünscht ist oder ob eine „X wurde nicht kopiert“-
  Rückmeldung fehlt.
- **Paid-Limit-Hard-Enforcement** greift nur bei aktivem Toggle
  `paid_limit_hard_enforcement` **und** Nicht-Shiftplanner
  (`shiftplan_edit.rs:618–652`). Sonst nur Soft-Warning nach Persist.
- **Weekly-Report läuft `weeks_in_year + 3`:** `get_weekly_summary`
  überzieht das Jahr um drei Wochen ins Folgejahr, um Jahres-Randwochen
  konsistent anzuzeigen (`booking_information.rs:311–316`). Wichtig für
  Frontend-Konsumenten, die nur ein Jahr erwarten.
- **`is_shiftplanner`-Gate für Detail-Zeilen:** Ein reiner Sales-User sieht
  in `WeeklySummary` **keine** `working_hours_per_sales_person`
  (`booking_information.rs:449–465` / `582–598`). Frontend-Konsumenten
  müssen mit leerer Liste umgehen können.
- **Volunteer-Absence-Whole-Week-out (VFA-01, D-26):** Fällt eine
  Absence-Periode in eine Kalenderwoche, wird die Person für Band 1 +
  Band 2 dieser Woche komplett ausgeschlossen — pro Tag nicht anteilig
  (`booking_information.rs:317–343`, `374–402`). Bewusstes Design, nicht
  Bug.
- **Special-Day-Filter:** In `get_weekly_summary`/`get_summery_for_week`
  werden Slots, die auf einen `Holiday` fallen, hart aus der
  Kapazitätsrechnung ausgefiltert; für `ShortDay` wird via
  `shortday_gate::clip_slot_for_week` je nach Stichtag-Toggle geclippt
  oder gedroppt (`booking_information.rs:414–439`, `551–574`). Verhalten
  auf Legacy-Ast durch Chain-C-Regressions-Tests abgesichert.
- **Booking-Log liest Soft-Deletes bewusst mit:** Wer dort Zeilen für
  „ausgetragen“ erwartet, muss nach `deleted != NULL` filtern (Frontend
  macht das über `booking_log_status_filter`,
  `shiftplan-dioxus/src/page/shiftplan.rs:281`).

## 8. Tests

- **Unit / Roundtrip:** `service_impl/src/test/booking.rs` (1203 LOC) —
  deckt CRUD, Permission-Splits (Shiftplanner vs. Sales-User),
  Validation-Failures (`test_create_with_id`, `test_create_with_version`,
  `test_create_with_created_fail`, `test_create_sales_person_does_not_exist`,
  `test_create_booking_data_already_exists`, `test_create_slot_does_not_exist`,
  `test_delete_no_permission`, …) sowie Copy-Week ab.
- **Wochen-Konflikte & Summary:** `service_impl/src/test/booking_information.rs`
  (675 LOC) — Regeln D-01, D-04, D-05, CVC-04/06 (Band-1 Committed-Voluntary,
  Band-2 Surplus), Cap-Gate-Fälle, Multi-Person/Multi-Day.
- **Chain-C Regression (Phase 51):** `service_impl/src/test/booking_information_chain_c.rs`
  (650 LOC) — Legacy-Semantik pro Ast des Stichtag-Toggles gegen historische
  Wochen (siehe MEMORY „Stichtag-Rollout Legacy-Semantik pro Chain“).
- **VFA-01 Volunteer-Absence:** `service_impl/src/test/booking_information_vfa.rs`
  (368 LOC) — Kategorie-agnostische Whole-Week-out-Regel.
- **Booking-Log:** `service_impl/src/test/booking_log.rs` (87 LOC) —
  Read-Mapper, Permission-Gate.
- **Konflikt-aware Persist-Pfad + Wochen-Sperre + Paid-Limit:** Tests
  liegen in `service_impl/src/test/shiftplan_edit*.rs`
  (F02, siehe dortige Feature-Doku) und decken `PaidLimitExceeded`,
  `BookingOnAbsenceDay`, `BookingOnUnavailableDay`,
  `assert_week_not_locked` ab. **[Zu prüfen]** Explizite Testabdeckung
  der User-Tracking-Fallback-Kette (`current_user → payload → "system"`).
- **DAO-Ebene:** `dao_impl_sqlite`-Integrations-Tests decken den
  `WHERE deleted IS NULL`-Filter und die `bookings_view`-Read-Pfade.
  **[Zu prüfen]** Ob es einen dedizierten Test gibt, der die Migration
  `20250115000000` gegen alten Bestand simuliert (NULL-`created_by`).

## 9. Historie & Kontext

- **2024-05:** `booking`-Tabelle als Basis-Schreibfläche des Editors
  eingeführt. Modell: `sales_person × slot × (year, calendar_week)`.
- **2024-07:** `bookings_view` als denormalisierte, UUID-formattierte Sicht
  eingeführt — Grundlage für den späteren Audit-Log.
- **2025-01:** User-Tracking-Felder `created_by`, `deleted_by`
  nachgezogen; parallel View-Recreate. Motivation: nachvollziehbarer
  Audit-Trail im Booking-Log.
- **v2.x Phase 3 („BOOK-02“):** Konflikt-aware Persist-Pfad in
  `ShiftplanEditService::book_slot_with_conflict_check` — der neue
  Standard-Pfad des Editors. Legacy-`POST /booking` bleibt für
  API-Kompatibilität (D-Phase3-18).
- **v2.x Phase 5 (D-04/06/07/08/15/16):** Paid-Employee-Limit als
  Soft-Warning.
- **v2.x Phase 8.3:** Halbtags-Absence + Booking → schweigend geduldet
  (D-08.3-05).
- **v2.x Phase D-24:** Hard-Enforcement des Paid-Limits hinter Toggle
  `paid_limit_hard_enforcement`, Shiftplanner-Bypass (D-24-02).
- **v2.x Phase 40 (WST-04):** Wochen-Sperre-Gate um alle Schreibpfade
  gezogen; `delete_booking` rutscht in den `ShiftplanEditService`, damit
  der DELETE-REST-Handler den Gate ebenfalls durchläuft.
- **v2.2.1:** `get_booking_conflicts_for_week` reichert Konflikt-Liste um
  aktive AbsencePeriods an; Warnings-Emission (Absence + Manual-Unavailable)
  im Konflikt-aware Persist-Pfad.
- **v2.4 Phase 51 (Chain C, D-51-06/07):** Stichtag-Toggle für
  ShortDay-Clip; Legacy-Semantik pro Ast in Chain-C-Regressions-Tests
  eingefroren.
- **Toggle-Bypass-Fix (Phase 51 Gap-Closure):** `ToggleService`-Reads
  behandeln `Authentication::Full` als all-rights (siehe
  `service_impl/src/toggle.rs`), damit die internen `Full`-Aufrufer in
  `booking_information.rs` und `reporting.rs` konsistent lesen — Grund,
  warum diese Docs den Full-Bypass explizit erwähnen.

Weitere Kontext-Reads in `.planning/phases/…` (z.B. Phase 3, 5, D-24,
40, 51).

---

*Letzte Verifikation gegen Code:* siehe `git log`/`jj log` dieser Datei.

---

**Fazit:** Booking ist die schmale, harte Schreib-Achse des Editors —
`BookingService` (Basic) validiert und persistiert, `BookingInformationService`
(Business-Logic, Full-Bypass nach innen) macht daraus die aggregierten
Wochen-Zahlen, und `BookingLogService` liefert den Audit-Blick auf
`bookings_view` inkl. Soft-Deletes. Wer daran arbeitet, muss den
Konflikt-aware Persist-Pfad in `shiftplan_edit`, die Wochen-Sperre und die
Paid-Limit/Absence-/Unavailable-Warnings mitdenken — der Legacy-`POST /booking`
kennt nur die einfache Duplikat-/Eligibility-Prüfung.
