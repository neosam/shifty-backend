# Feature: Wochen-Metadaten (Special Days, Week Status, Week Message, Warnings)

> **Kurzform:** Vier lose gekoppelte Sub-Features, die einer ISO-(Jahr, KW)
> zusätzliche Fakten zuordnen: Feiertage/Kurztage (Special Days), einen
> Planungs-Zustand (Week Status), einen Info-Text (Week Message) und
> abgeleitete Cross-Source-Anomalien (Warnings). Sie versorgen Planer und
> Mitarbeiter mit Wochen-Kontext, der über den reinen Slot-/Booking-Datenkern
> hinausgeht.

**Cluster-ID:** F09
**Status:** produktiv
**Erstmalig eingeführt:** Special Days v0.x (Migration 2024-10), Week Message v0.x (Migration 2025-01), Warnings v1.6 (Phase 3), Week Status v2.x (Phase 39, Migration 2026-07)
**Zuständige Crates:**
- `service::special_days`, `service::week_status`, `service::week_message`, `service::warning`
- `service_impl::special_days`, `service_impl::week_status`, `service_impl::week_message` (Warning ist rein Datentyp)
- `dao::special_day`, `dao::week_status`, `dao::week_message` (kein `warning`-DAO)
- `rest::special_day`, `rest::week_status`, `rest::week_message` (Warnings reisen als Teil anderer Wrapper-Responses)

---

## 1. Was ist das? (Fachlich)

Der Schichtplan-Kern (Slots + Bookings) beantwortet die Frage *"Wer arbeitet
wann?"*. F09 ergänzt drei orthogonale Dimensionen für eine konkrete
ISO-(Jahr, KW):

- **Special Days** — Feiertage (`Holiday`) und Kurztage (`ShortDay`) mit
  Einfluss auf die erwarteten Stunden und die Slot-Kürzung. Beispiel:
  1. Mai als Holiday → Slot am Feiertag zählt nicht als Vertrags-Erwartung
  und triggert je nach Toggle Auto-Gutschrift (Kapitel 4).
- **Week Status** — Freigabezustand einer KW im Planungs-Workflow:
  `Unset` (implizit, Zeile fehlt) → `InPlanning` → `Planned` → `Locked`. Rein
  informativ; keine harte Sperre auf Booking-Ebene (siehe Randfälle).
- **Week Message** — Freier Info-Text pro (Jahr, KW), z.B. "Achtung, verlängerte
  Öffnung bis 22 Uhr". Wird prominent oberhalb des Shiftplans gerendert.
- **Warnings** — Nicht persistierte, aus mehreren Quellen aggregierte Konflikt-
  bzw. Regel-Hinweise (Booking auf Absence-Tag, Absence überlappt Booking,
  bezahltes-Personal-Limit überschritten). Kommen ausschließlich als
  Bestandteil erfolgreicher Wrapper-Responses (200/201) der schreibenden
  Endpoints zurück — kein Error-Pfad.

**Beispiel-Workflow aus User-Sicht:**

1. Der Planer öffnet die Kalenderwoche 20/2025 im Shiftplan.
2. Frontend lädt parallel Special Days (`/special-days/for-week/2025/20`),
   Week Status (`/week-status/by-year-and-week/2025/20`) und Week Message
   (`/week-message/by-year-and-week/2025/20`).
3. Am Donnerstag ist "Christi Himmelfahrt" als Holiday markiert → die
   Feiertags-Kachel wird eingefärbt, die Slots dieses Tages werden für die
   Balance-Rechnung anders gewertet.
4. Der Planer setzt den Week Status auf `Planned` und trägt eine Info-Message
   ein ("Bitte pünktlich, hoher Andrang").
5. Beim Anlegen eines neuen Bookings, das eine bestehende Urlaubsphase des
   Mitarbeiters überlappt, liefert das Backend die Buchung + eine
   `BookingOnAbsenceDay`-Warning zurück; das Frontend zeigt sie als Banner
   (nicht als Blocking-Dialog).

## 2. Fachliche Regeln

### Special Days

- **Kategorien:** `Holiday` (ganztägig) und `ShortDay` (verkürzter Tag mit
  Pflicht-`time_of_day`).
  Verifiziert: `service/src/special_days.rs:13-16`, Validierung
  `service_impl/src/special_days.rs:131-139`.
- **Eindeutigkeit pro (year, calendar_week, day_of_week):** Fach-Regel — beim
  Anlegen wird ein bereits aktiver Eintrag am selben Tag **in-place** ersetzt
  (Preserve-`id`, Preserve-`created`), kein Duplicate-Error, kein PUT-Endpoint.
  `service_impl/src/special_days.rs:170-195` (Same-Date-Replacement SDF-01).
- **Type/Time-Kopplung:** `Holiday` **darf keinen** `time_of_day` haben — bei
  Bedarf im Service normalisiert (`special_days.rs:156-159`). `ShortDay` **muss
  einen** `time_of_day` haben, sonst 400 (`ValidationError`).
- **calendar_week-Bereich:** 1..=`time::util::weeks_in_year(year)`.
  `service_impl/src/special_days.rs:140-149`.
- **Permission:** `create` / `delete` benötigen `SHIFTPLANNER_PRIVILEGE`;
  Reads sind offen. `service_impl/src/special_days.rs:122, 216`.
- **ISO-Wochenjahr vs. Kalenderjahr:** `get_by_year` liefert alle Special Days,
  deren tatsächliches **Datum** ins Kalenderjahr fällt — inklusive Einträgen,
  die aus ISO-Wochen-Jahr `year - 1` stammen (z.B. Neujahrs-Einträge).
  `service_impl/src/special_days.rs:77-116` (SDF-03 post-ship).
- **Wirkung auf Balance / Slots:**
  - `Holiday` → Auto-Credit von Feiertagsstunden im Reporting, sofern Toggle
    `holiday_auto_credit` aktiv (`service_impl/src/reporting.rs:151-243`) und
    Feiertagsstunden nur in Absence-Bezug (`service_impl/src/absence.rs:447,
    755`).
  - `ShortDay` → Slot-Kürzung via `shortday_gate::should_clip` +
    `Slot::clip_to(cutoff)`, gesteuert durch Stichtag-Toggle (D-51-07).
    `service_impl/src/shortday_gate.rs:1-40, 204`.

### Week Status

- **Vier Domain-Werte, drei persistiert:** `Unset` lebt nur in Service/FE;
  DAO-Enum `WeekStatusKind` kennt nur `InPlanning | Planned | Locked`. Zeilen-
  Absenz == `Unset` (D-39-04). Explizit `Unset` (nicht `None`), um
  Option-Shadowing zu vermeiden (D-39-03).
  `service/src/week_status.rs:12-18`, `dao/src/week_status.rs:8-13`.
- **Freie Übergänge:** Jede Transition ist erlaubt, es gibt keine
  State-Machine mit Guards (D-39-02). `set_week_status` upsertet ohne
  Transition-Validierung. `service_impl/src/week_status.rs:94-125`.
- **Read offen, Write geschützt:** `get_week_status` hat **kein** Permission-
  Gate (T-39-03); `set_week_status` verlangt `SHIFTPLANNER_PRIVILEGE`
  (Gate vor jedem DAO-Zugriff, D-39-01/T-39-01).
  `service_impl/src/week_status.rs:44-75`.
- **Transaktions-Atomarität:** `find` + `write` laufen in derselben TX
  (TOCTOU-frei, T-39-04). `service_impl/src/week_status.rs:78-128`.
- **`Unset`-Semantik beim Setzen:** Soft-Delete der aktiven Zeile falls
  vorhanden, sonst No-Op. `service_impl/src/week_status.rs:86-92`.
- **Keine Kaskade auf Bookings:** `Locked` sperrt Bookings **nicht** auf DAO/
  Service-Ebene — die Sperre ist Frontend-seitig (`shiftplan.rs:304`: nur
  Nicht-Editor werden hinter `Locked` geblockt). **[Zu prüfen]** Ob dies
  bewusste Konvention ist oder eine Lücke im BE.

### Week Message

- **Freitext, ein Eintrag pro Woche.** UNIQUE-Constraint `(year,
  calendar_week)` — **plain UNIQUE**, nicht partial (siehe Randfälle).
  Migration `20250123000000_add-week-message-table.sql:12`.
- **Permission:** `create` / `update` / `delete` verlangen
  `SHIFTPLANNER_PRIVILEGE`; Reads sind offen.
  `service_impl/src/week_message.rs:78, 118, 143`.
- **Keine Content-Validierung:** Kein Length-Cap, kein Sanitizing im Backend
  — die Message ist reines Passthrough. **[Zu prüfen]** ob ein Length-Cap
  fachlich sinnvoll wäre.
- **`id`/`version` bei Create nil-Guard:** `IdSetOnCreate` /
  `VersionSetOnCreate` bei Nicht-Nil.
  `service_impl/src/week_message.rs:87-91`.

### Warnings

- **Erfolg, nicht Error:** Warnings reisen im 200/201-Pfad als `warnings: Vec<WarningTO>`
  in Wrapper-Response-DTOs (nicht als `ServiceError`, nicht als
  `ValidationFailureItem`/422). `service/src/warning.rs:1-10`,
  `rest-types/src/lib.rs:1919-1993`.
- **Granularität:** Eine Warning pro betroffenem Booking-Tag (D-Phase3-15),
  **keine De-Duplikation** über mehrere Bookings hinweg (Copy-Week akkumuliert).
  `service/src/shiftplan_edit.rs:25-32`.
- **Fünf Varianten** (`service/src/warning.rs:23-73`):
  - `BookingOnAbsenceDay` — Reverse-Warning BOOK-02, Buchung auf Absence-Tag.
  - `BookingOnUnavailableDay` — Buchung auf manuell blockiertem Tag
    (`sales_person_unavailable`).
  - `AbsenceOverlapsBooking` — Forward-Warning BOOK-01, neue Absence überlappt
    bestehendes Booking.
  - `AbsenceOverlapsManualUnavailable` — Absence überdeckt manuellen Unavail,
    **kein Auto-Cleanup** (D-Phase3-16).
  - `PaidEmployeeLimitExceeded` — Phase 5 (D-08): Slot-`max_paid_employees`
    strikt überschritten. Buchung wird trotzdem persistiert (D-07),
    NULL-Limit triggert nicht (D-15).
- **Deferred:** `ManualUnavailableOnAbsenceDay` als 6. Variante ist
  aufgeschoben (D-Phase3-17). Kein Code-Pfad heute.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `special_day` | Feiertag/Kurztag pro ISO-(Jahr, KW, Wochentag) | `id`, `year`, `calendar_week`, `day_of_week`, `day_type` (`TEXT`), `time_of_day`, `created`, `deleted`, `update_process`, `update_version` |
| `week_status` | Freigabezustand einer ISO-(Jahr, KW) | `id`, `year`, `calendar_week`, `status` (`TEXT`), `created`, `deleted`, `update_process`, `update_version`. **Partial-UNIQUE-Index** `idx_week_status_active WHERE deleted IS NULL` |
| `week_message` | Freitext-Info pro ISO-(Jahr, KW) | `id`, `year`, `calendar_week`, `message`, `created`, `deleted`, `update_process`, `update_version`. **Plain-UNIQUE** `(year, calendar_week)` |

Warnings haben **keine Tabelle** — sie werden in der Service-Schicht aus
Booking-, Absence- und `sales_person_unavailable`-Daten synthetisiert.

### Migrations

- `20241020064536_add-special-day-table.sql` — Basistabelle Special Day
  (Oktober 2024, v0.x).
- `20250123000000_add-week-message-table.sql` — Week Message mit Plain-UNIQUE
  (Januar 2025).
- `20260702000000_create-week-status.sql` — Week Status mit Partial-UNIQUE
  (Juli 2026, Phase 39). Der Kommentar in der Migration hebt explizit den
  Unterschied zu Week Messages hervor ("RESEARCH Pitfall P-6").

Kein separater Warning-DDL — Warnings sind read-only Aggregate.

### Beziehungen

- Special Day, Week Status, Week Message koppeln **nur** an ISO-(Jahr, KW),
  nicht an `sales_person_id`. Keine FKs auf andere Aggregate.
- Warnings referenzieren zur Laufzeit `booking_id`, `absence_id`,
  `unavailable_id`, `slot_id` — reine `Uuid`-Zeiger im Payload, kein
  DB-Constraint.

## 4. Service-API

### Traits

- `service::special_days::SpecialDayService`
  - `get_by_week(year, calendar_week, ctx) -> Arc<[SpecialDay]>`
  - `get_by_year(year, ctx) -> Arc<[SpecialDay]>` — Wochenjahr → Kalenderjahr-
    Filter, SDF-03.
  - `create(&SpecialDay, ctx) -> SpecialDay` — same-date-replace, ID/Version-
    Nil-Guard.
  - `delete(uuid, ctx) -> SpecialDay` — soft-delete.
  - **Keine** `Option<Transaction>` — dieser Service ist prä-Transaction-Era
    umgesetzt (**[Zu prüfen]** ob absichtlich).
- `service::week_status::WeekStatusService` (`week_status.rs:32-57`)
  - `get_week_status(year, calendar_week, ctx, tx) -> WeekStatus`
  - `set_week_status(year, calendar_week, status, ctx, tx) -> WeekStatus`
- `service::week_message::WeekMessageService` (`week_message.rs:56-102`)
  - `get_by_id`, `get_by_year_and_week`, `get_by_year`, `create`, `update`,
    `delete` — alle mit `Option<Transaction>`.
- `service::warning::Warning` (`warning.rs`) — **kein Trait**, reines Daten-
  Enum. Erzeugt durch:
  - `service::shiftplan_edit::ShiftplanEditService::book_slot_with_conflict_check`
    → `BookingCreateResult { booking, warnings }`.
  - `service::shiftplan_edit::ShiftplanEditService::copy_week` →
    `CopyWeekResult { copied, warnings }`.
  - `service::absence::AbsenceService::create` → `AbsencePeriodCreateResult`.

### Auth-Gates

| Sub-Feature | Read | Write |
| --- | --- | --- |
| Special Days | offen (jede Rolle) | `SHIFTPLANNER_PRIVILEGE` |
| Week Status | offen (T-39-03) | `SHIFTPLANNER_PRIVILEGE` (D-39-01/T-39-01) |
| Week Message | offen | `SHIFTPLANNER_PRIVILEGE` |
| Warnings | n/a — Read-Side-Effekt anderer Endpoints | erzeugt auf dem Write-Pfad ihrer Host-Endpoints |

### TX-Verhalten

- **Special Day:** öffnet **keine** TX; DAO-Aufrufe laufen isoliert (Legacy-
  Signatur ohne `Option<Transaction>`, siehe `dao/src/special_day.rs:29-39`).
  Same-Date-Replace macht `find_by_week` + `update` in getrennten SQL-Statements
  — theoretisch ein TOCTOU-Fenster, praktisch durch SQLite Single-Writer
  entschärft (siehe `edge-cases.md#7-transaktionen--atomarität`).
- **Week Status / Week Message:** öffnen TX selbst wenn `tx = None`, `commit`
  am Ende. `find` + `write` in derselben TX (Week Status atomar per Design,
  T-39-04; `service_impl/src/week_status.rs:78-128`).

### Dependencies

Alle drei Basis-Services konsumieren nur DAOs + Support-Services — **kein**
Domain-Cross-Coupling:

- `SpecialDayServiceImpl` → `SpecialDayDao`, `PermissionService`,
  `ClockService`, `UuidService`.
- `WeekStatusServiceImpl` → `WeekStatusDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `WeekMessageServiceImpl` → `WeekMessageDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.

Damit gehören alle drei zur **Basic-Services**-Schicht (siehe
`CLAUDE.md` → Service-Tier-Konventionen).

Warnings sind Wire-Format; die Produktion passiert in
Business-Logic-Services (`ShiftplanEditService`, `AbsenceService`), die
mehrere Basics kombinieren (`booking_dao`, `absence_service`,
`sales_person_unavailable_service`, `slot_service`).

## 5. REST-Endpoints

Base-Paths gemäß `rest/src/lib.rs:669-675`:
- `/special-days` (nested)
- `/week-status` (nested, Phase 39)
- `/week-message` (nested)

### Special Days (`rest/src/special_day.rs`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/special-days/for-week/{year}/{calendar_week}` | Liste für eine KW | — | `[SpecialDayTO]` | 500 |
| `GET` | `/special-days/for-year/{year}` | Liste für ein Kalenderjahr (SDF-03-Filter) | — | `[SpecialDayTO]` | 500 |
| `POST` | `/special-days` | Anlegen bzw. Same-Date-Replace | `SpecialDayTO` | `SpecialDayTO` | 400, 403 |
| `DELETE` | `/special-days/{id}` | Soft-Delete | — | 204 | 404, 403 |

### Week Status (`rest/src/week_status.rs`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/week-status/by-year-and-week/{year}/{week}` | Status; `Unset` wenn keine Zeile | — | `WeekStatusTO` | — |
| `PUT` | `/week-status/by-year-and-week/{year}/{week}` | Upsert (auch für `Unset` = Soft-Delete) | `WeekStatusTO` | `WeekStatusTO` | 403 |

Design-Entscheidung D-39-06: GET und PUT auf demselben KW-Pfad, **kein**
Id-Endpoint (`rest/src/week_status.rs:15-28`).

### Week Message (`rest/src/week_message.rs`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/week-message` | Anlegen | `WeekMessageTO` | `WeekMessageTO` | 400, 403 |
| `GET` | `/week-message/{id}` | By Id | — | `WeekMessageTO` | 404 |
| `PUT` | `/week-message/{id}` | Update (Pfad-Id überschreibt Body-Id) | `WeekMessageTO` | `WeekMessageTO` | 400, 403, 404 |
| `DELETE` | `/week-message/{id}` | Soft-Delete | — | 204 | 403, 404 |
| `GET` | `/week-message/by-year/{year}` | Alle Messages eines Jahres | — | `[WeekMessageTO]` | — |
| `GET` | `/week-message/by-year-and-week/{year}/{week}` | Message einer KW | — | `WeekMessageTO` | 404 |

### Warnings

Kein eigener Endpoint. Wire-Form `WarningTO` (Tag-Enum, 5 Varianten) in
`rest-types/src/lib.rs:1919-2054` reist als Feld in u.a.:

- `BookingCreateResultTO` (`POST /shiftplan-edit/booking`)
- `CopyWeekResultTO`
- `AbsencePeriodCreateResultTO`

DTOs für alle Sub-Features siehe `rest-types::lib` — `SpecialDayTO`
(:1190-1241), `WeekMessageTO` (:1310-1354), `WeekStatusTO` /
`WeekStatusKindTO` (:1361-1396), `WarningTO` (:1942-2054).

## 6. Frontend-Integration

- **Pages:** `shifty-dioxus/src/page/shiftplan.rs` — Hauptkonsument aller
  drei Sub-Features; `shifty-dioxus/src/page/settings.rs` für Special-Days-
  Pflege.
- **Components:**
  - `component/warning_list.rs` — geteilte `WarningList`-Komponente, rendert
    alle `WarningTO`-Varianten als **Inline-Banner** (nicht als
    Blocking-Dialog, siehe `feedback_warnings_inline_not_dialog`).
  - `component/week_status_dropdown.rs` + `component/atoms/week_status_badge.rs`
    — UI-Element für Status-Setzen und Anzeige.
  - `component/top_bar.rs` — bindet Week-Status-Badge ein.
- **Services:** `shifty-dioxus/src/service/week_status.rs` (Coroutine
  `WeekStatusAction::Load` / `Set`), `service/absence.rs` (Warning-
  Propagation im Absence-Flow).
- **State:** `state/week_status.rs` (Store), Warnings werden im Page-lokalen
  Signal gehalten.
- **i18n-Keys:** `i18n/{en,de,cs}.rs` — u.a.
  `BookingWarningDialogHeaderSingular` / `-Plural`; das Backend liefert nur
  strukturierte Warning-Daten, Übersetzung passiert im Frontend
  (`warning.rs:63-64`).
- **Proxy (`shifty-dioxus/Dioxus.toml`):**
  - Zeile 60: `/special-days` → `http://localhost:3000/special-days`
  - Zeile 88: `/week-message` → `http://localhost:3000/week-message`
  - Zeile 90: `/week-status` → `http://localhost:3000/week-status`
  Fehlender Proxy-Eintrag = 404 im `dx serve`-Dev-Modus (siehe
  `feedback_dioxus_proxy_for_new_backend_endpoints`).

## 7. Randfälle

Zentrale Randfall-Referenz: [`../domain/edge-cases.md`](../domain/edge-cases.md).
Relevante Sektionen: [§4 Zeit & Zeitzone](../domain/edge-cases.md#4-zeit--zeitzone)
und [§1 Stundenkonto](../domain/edge-cases.md#1-stundenkonto) (dort v.a.
§1.1 "Neuer Feiertag im abgeschlossenen Jahr" und §1.4 "Special Days &
Feiertage").

- **Feiertag am Wochenende:** Ein `Holiday`-Eintrag am Samstag/Sonntag wird
  fachlich toleriert (keine Ablehnung im Service), wirkt aber im Reporting
  nur, wenn der Mitarbeiter an dem Wochentag Contract-Stunden hätte
  — Auto-Credit nutzt `EmployeeWorkDetails::holiday_hours()`
  (`reporting.rs:230-233`). **[Zu prüfen]** ob das UI hier eine
  Vorwarnung gibt.
- **Rückwirkender Special-Day-Eintrag in ein abgeschlossenes Jahr:** Balance-
  Rechnung ändert sich live, Carryover bleibt statisch → Drift. Konvention
  laut `edge-cases.md#1-stundenkonto` §1.1: **Nicht tun**, außer Carryover
  wird manuell neu berechnet.
- **KW-über-Jahreswechsel (ISO-Wochenjahr ≠ Kalenderjahr):** Ein Feiertag am
  01.01. eines Jahres wird intern unter `(year=vorjahr, week=53, day=Mo/Di)`
  geführt. `get_by_year` löst das via Zwei-Jahres-Load + Kalender-Filter auf
  (`special_days.rs:77-116`). Das UI/Excel sollte niemals blind über
  `special_day.year` sortieren.
- **`Locked` ohne Backend-Enforcement:** `Locked`-Status sperrt Bookings
  nicht auf DAO-/Service-Ebene, sondern nur UI-seitig für Nicht-Editor
  (`shiftplan.rs:304`). Ein Client mit `SHIFTPLANNER_PRIVILEGE` und
  API-Zugriff kann in einer `Locked`-Woche weiter buchen. **[Zu prüfen]**
  ob eine BE-Sperre gewünscht ist (Fat-Backend-Prinzip, siehe
  `feedback_fat_backend_thin_client`).
- **Week Message UNIQUE-Kollision:** Weil `week_message` **kein** Partial-
  Index hat, kollidiert ein wiederholtes Insert für (year, week) **auch
  gegen soft-deleted Zeilen** — DAO-Error kaskadiert zu 500. Verhalten der
  DAO **[Zu prüfen]** in `dao_impl_sqlite/src/week_message.rs`.
- **Special-Day-Replace bei Race:** Zwei parallele POSTs auf denselben Tag
  können in getrennten SQL-Statements (`find_by_week` + `update`) einen
  Doppel-Insert produzieren. SQLite-Single-Writer entschärft das in
  Produktion, aber es ist kein hartes Constraint.
- **`Unset` als Client-Payload:** `PUT /week-status/.../unset` löscht die
  Zeile soft. Frontend darf `Unset` senden, um Status zu clearen — nicht
  DELETE. `service_impl/src/week_status.rs:86-92`.
- **Warning-Volumen bei `copy_week`:** Da keine De-Dup über Bookings
  (D-Phase3-15), kann eine Copy-Week-Operation dutzende Warnings
  produzieren. Frontend rendert als scrollbare Liste, keine Pagination.
- **`PaidEmployeeLimitExceeded` mit `NULL`-Limit:** Warning triggert nicht,
  wenn Slot kein `max_paid_employees` gesetzt hat (D-15,
  `warning.rs:56-72`). Slot ohne Limit ist unbegrenzt.

## 8. Tests

### Unit / Integration

- **Special Days:** `service_impl/src/test/special_days.rs` (843 Zeilen).
  Deckt u.a. ab:
  - `test_get_by_year_returns_new_year_day_under_calendar_year` — SDF-03
    Wochenjahr → Kalenderjahr.
  - `test_create_replaces_same_date_entry` — Same-Date-Replace SDF-01.
  - `test_create_switches_holiday_to_shortday` /
    `test_create_switches_shortday_to_holiday` — Type-Switch atomar.
  - `test_holiday_shortday_roundtrip_atomic` — Round-Trip.
  - `test_create_rejects_shortday_without_time` — Type/Time-Kopplung.
  - `test_create_rejects_calendar_week_out_of_range` — KW-Bound.
  - `test_create_rejects_nonnil_id` / `_version` — Nil-Guards.
  - `test_create_forbidden_without_shiftplanner`,
    `test_delete_forbidden_without_shiftplanner` — Auth-Gates.
- **Week Status:** `service_impl/src/test/week_status.rs` (446 Zeilen).
  U.a.:
  - `test_set_permission_denied_no_dao_write` — Gate vor DAO (T-39-01).
  - `test_set_unset_soft_deletes_existing` /
    `test_set_unset_noop_when_absent` — `Unset`-Semantik.
  - `test_set_creates_when_absent` / `test_set_updates_when_present` —
    Upsert.
  - `test_transitions_free` — D-39-02.
  - `test_get_returns_unset_when_absent` / `test_get_maps_kind` —
    Row-Absence-Semantik.
- **Warnings:** implizit über Tests von `service_impl/src/test/absence.rs`,
  `test/shiftplan_edit.rs`, `test/slot.rs`, `test/booking_log.rs` — Cross-
  Source-Konflikte werden dort assertiert (grep-Nachweis).

### Bekannte Lücken

- **Kein dedizierter `week_message`-Testfile** (grep findet nichts unter
  `service_impl/src/test/`). Deckung nur indirekt über REST-Smoke.
  **[Zu prüfen]**.
- **Kein Test für `PaidEmployeeLimitExceeded`-Warning-Suppression bei
  `NULL`-Limit** in dieser Cluster-Doku belegt — vermutlich in Phase-5-Tests
  vorhanden, **[Zu prüfen]**.
- **Kein BE-Test für `Locked` + Booking-Attempt** (siehe Randfall oben).

## 9. Historie & Kontext

- **Special Days** — Oktober 2024, Migration
  `20241020064536_add-special-day-table.sql`. Post-Ship-Fixes SDF-01 (Same-
  Date-Replace) und SDF-03 (Wochenjahr-Filter) in v2.x
  (`service_impl/src/special_days.rs:76, 106`). Server-Side-Validation
  D-33-06/07 kam in Phase 33 dazu (`special_days.rs:127-152`).
- **Week Message** — Januar 2025, Migration
  `20250123000000_add-week-message-table.sql`. Plain-UNIQUE ist Legacy;
  Week Status übernimmt die "richtige" Partial-UNIQUE-Form (siehe Migrations-
  Kommentar 2026-07 "RESEARCH Pitfall P-6").
- **Warnings** — Phase 3 (v1.6, 2025), Cross-Source-Konflikt-Warnungen im
  Absence/Booking/Unavailable-Dreieck; 5. Variante
  `PaidEmployeeLimitExceeded` in Phase 5 nachgezogen (`warning.rs:54-72`).
  6. Variante `ManualUnavailableOnAbsenceDay` bewusst aufgeschoben
  (D-Phase3-17).
- **Week Status** — Phase 39 (v2.x, Juli 2026), Migration
  `20260702000000_create-week-status.sql`. Design-Entscheidungen: `Unset`-
  Variante (D-39-03/04), freie Transitionen (D-39-02), Read offen /
  Write gated (T-39-01/03), TX-Atomarität (T-39-04), einheitlicher
  KW-Pfad (D-39-06).
- **Nicht F09, aber verwandt:** `shortday_gate` (Phase 51, D-51-07)
  konsumiert Special Days und Toggle `SHORTDAY_ACTIVE_FROM`, um die Slot-
  Kürzung Stichtag-gerecht zu rollout-en
  (`service_impl/src/shortday_gate.rs:1-40`).

---

**Fazit:** F09 ist eine Sammlung von vier eigenständigen, schwach gekoppelten
Sub-Features rund um ISO-(Jahr, KW)-Metadaten; jedes Sub-Feature ist einzeln
verständlich, aber sie teilen sich Auth-Modell (Read offen, Write per
`SHIFTPLANNER_PRIVILEGE`) und den Wochen-Pfad im Frontend. Die relevanten
scharfen Kanten liegen bei ISO-Wochenjahr ↔ Kalenderjahr, dem fehlenden
Backend-Enforcement für `Locked` und bei der Plain-UNIQUE-Falle in
`week_message` — alles andere ist erprobter Standard-CRUD.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
