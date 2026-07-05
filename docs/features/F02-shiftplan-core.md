# Feature: Shiftplan Core (Slots, Katalog, Editor, Ansicht)

> **Kurzform:** Das fachliche Herzstück von Shifty — definiert wann gearbeitet
> werden kann (Slots), gruppiert diese in Schichtpläne (Katalog), rendert die
> Wochen-/Tages-Ansicht mit Buchungen (View) und stellt den atomaren
> Schreib-Aggregat für alle Slot-/Booking-Mutationen bereit (Edit).

**Cluster-ID:** F02
**Status:** produktiv
**Erstmalig eingeführt:** Slots seit Milestone 0 (Migration `20240502113031`);
Katalog / Multi-Plan-Support seit v2.x (Migration `20260330000000`).
**Zuständige Crates:**
`service::slot`, `service::shiftplan`, `service::shiftplan_catalog`,
`service::shiftplan_edit`, `service::shiftplan_report`, `service_impl::…`,
`dao::slot`, `dao::shiftplan`, `dao::shiftplan_report`,
`rest::slot`, `rest::shiftplan_catalog`, `rest::shiftplan_edit`,
`rest::shiftplan` (`shiftplan-info`).

---

## 1. Was ist das? (Fachlich)

Ein **Slot** ist ein wöchentlich wiederkehrendes Zeitfenster an einem
bestimmten Wochentag (z.B. „Dienstag 10:00–12:00"), an dem Mitarbeiter
gebucht werden können. Ein Slot definiert nur die *Möglichkeit* zu arbeiten,
noch keine konkrete Person — das macht das Booking (siehe
[`F03-booking.md`](./F03-booking.md)).

Ein **Shiftplan** (Schichtplan) ist eine benannte Sammlung von Slots. Seit
v2.x unterstützt Shifty **mehrere parallele Schichtpläne** pro Instanz (z.B.
„Laden-Shift" und „Büro-Shift"), jeder mit eigener Slot-Struktur. Der
Standard-Plan heißt `main` (fest kodierte UUID aus Migration
`20260330000000_add-shiftplan-table.sql`).

Der **Katalog** (`ShiftplanService` / `shiftplan_catalog`) verwaltet die
Meta-Daten der Schichtpläne (Name, `is_planning`-Flag, Versionierung).
Die **Wochen-/Tages-Ansicht** (`ShiftplanViewService`) aggregiert
Slot-Definitionen + Bookings + Sales-Persons zu einer renderbaren
Struktur für das Frontend. Der **Edit-Aggregat**
(`ShiftplanEditService`) ist der einzige Schreib-Pfad für strukturelle
Änderungen (Slot ändern/löschen, Woche kopieren, Booking konflikt-aware
anlegen, Woche sperren respektieren, Ferien eintragen).

**Beispiel-Workflow aus User-Sicht:**

1. HR/Shiftplanner ruft die Shiftplan-Seite auf, wählt einen Schichtplan
   und eine ISO-KW.
2. FE lädt die Woche via `GET /shiftplan-info/{shiftplan_id}/{year}/{week}`;
   Slots und existierende Bookings werden nebeneinander gerendert.
3. Klick auf einen Slot öffnet den Slot-Editor
   (`component/slot_edit.rs`); Shiftplanner ändert `min_resources` /
   `max_paid_employees` oder das Zeitfenster.
4. Speichern → `PUT /shiftplan-edit/slot/{year}/{week}` → Backend
   splittet den Slot per `valid_from`/`valid_to` in zwei Segmente und
   migriert offene Bookings atomar auf die neue Version (`modify_slot`).
5. Ein Mitarbeiter meldet sich am freien Slot an → FE ruft
   `POST /shiftplan-edit/booking` → Backend prüft Wochen-Sperre,
   Paid-Employee-Limit + AbsencePeriod/Manual-Unavailable-Konflikte und
   liefert Warnungen zurück (siehe [`F03-booking.md`](./F03-booking.md),
   [`F05-absence-system.md`](./F05-absence-system.md)).

## 2. Fachliche Regeln

**Slot-Regeln** (`service::slot::Slot`, verifiziert in
`service_impl/src/slot.rs`):

- **Zeitfenster:** `from <= to`; Verletzung → `ServiceError::TimeOrderWrong`
  (`service_impl/src/slot.rs:225-227`).
- **Wochentag:** `day_of_week` ist unveränderlich nach Anlage; ändern durch
  `update_slot` liefert `ValidationFailureItem::ModificationNotAllowed`
  (`service_impl/src/slot.rs:314-318`). Gleiches gilt für `from`, `to`,
  `valid_from` (`service_impl/src/slot.rs:319-329`).
- **Gültigkeit:** `valid_from` (inklusiv) und `valid_to` (optional,
  inklusiv). `valid_to < valid_from` → `ServiceError::DateOrderWrong`.
- **Slot-Overlap:** Innerhalb desselben Schichtplans und desselben
  Wochentags dürfen sich zwei aktive Slots nicht überschneiden, wenn ihre
  `valid_from`/`valid_to`-Ranges kollidieren
  (`service_impl/src/slot.rs::test_overlapping_slots`, Zeilen 55-60;
  Aufruf in `create_slot`, Zeilen 234-246). Overlap-Definition:
  strikte Überlappung ODER exakte Deckungsgleichheit
  (`slot_1.from == slot_2.from && slot_1.to == slot_2.to`) — Rand-Kontakt
  (`slot_1.to == slot_2.from`) ist erlaubt.
- **`min_resources`:** Erwartete Mindestbesetzung pro Slot (Default 2
  aus Migration `20240813080347_add-column-min-resources.sql`). Aktuell
  vor allem als UI-Hinweis genutzt; FE rendert Warnung bei Unter-
  Deckung. **[Zu prüfen]** ob backend-seitig hart geblockt wird.
- **`max_paid_employees`:** Optional. Deckelung wieviele **bezahlte**
  Personen pro Slot+Woche gebucht sein dürfen. Weiche Grenze — greift nur
  bei aktivem Toggle `paid_limit_hard_enforcement`
  (`service_impl/src/shiftplan_edit.rs:618-660`, D-24-02/-08). Ohne
  Toggle wird eine Warning zurückgegeben; Shiftplanner bypassen die harte
  Grenze immer.
- **Verfügbarkeits-Fenster über Wochentage:** Ein Slot existiert an genau
  einem Wochentag (`day_of_week`). Die Verteilung „ein Slot pro Wochentag
  9-10 Uhr" wird durch **mehrere Slot-Rows** modelliert (siehe
  Default-Slots-Set, Kapitel 3).
- **Optimistic Locking:** Jeder Slot hat eine `version`-UUID; Update mit
  falscher Version → `ServiceError::EntityConflicts`
  (`service_impl/src/slot.rs:300-306`). Bei erfolgreichem Update wird
  eine neue Version-UUID vergeben (`slot.rs:340-343`).
- **Soft Delete:** `deleted` ist ein Timestamp; `delete_slot` setzt ihn,
  löscht die Row nicht (`slot.rs:270-283`). Alle DAO-Queries filtern
  `WHERE deleted IS NULL`.

**Shiftplan-Katalog-Regeln** (`service_impl/src/shiftplan_catalog.rs`):

- Anlage/Update erfordern `shiftplanner` (`SHIFTPLANNER_PRIVILEGE`).
- `Shiftplan.is_planning` markiert einen Plan als „in Planung" (FE-Flag,
  Auswirkung **[Zu prüfen]** in Detail).
- `Shiftplan.name` frei wählbar; die Standard-Row `main` mit fester UUID
  `00000000-0000-4000-8000-000000000001` wird von der Migration angelegt
  und darf nicht gelöscht werden (Referenz aller Legacy-Slots).

**Shiftplan-Edit-Regeln** (`service_impl/src/shiftplan_edit.rs`):

- **Permission `shiftplan.edit`** für alle Slot-Mutationen (`modify_slot`,
  `modify_slot_single_week`, `remove_slot`) — separates Privileg von
  `shiftplanner` (getrennte Rolle seit Migration
  `20241118165756_add-role-shiftplan-edit.sql`).
- **Wochen-Sperre (Week-Lock)-Gate** (`assert_week_not_locked`,
  `shiftplan_edit.rs:908+`) läuft VOR jeder Mutation. Auch der
  `book_slot_with_conflict_check`-Pfad gatet dagegen — `shiftplanner`
  allein reicht nicht, `shiftplan.edit` wird als Bypass genutzt (siehe
  CR-01-Kommentar `shiftplan_edit.rs:591-604`).
- **Slot-Split-Semantik** (`modify_slot`, `shiftplan_edit.rs:56-151`):
  Ändern eines Slots ab `change_year`/`change_week` legt einen **neuen
  Slot** an (mit den neuen Werten und `valid_from = Montag change_week`)
  und schließt den Alten mit `valid_to = Sonntag change_week-1`. Alle
  Bookings ab `change_week` werden auf den neuen Slot re-pointet
  (alte Booking-Row soft-deleted, neue Row angelegt) — alles in einer
  Transaktion (siehe Kapitel 7 „Randfälle").
- **`modify_slot_single_week`** (D-35-01 Approach B, seit Phase 35):
  3-Segment-Split für **einmalige** Ausnahme genau einer Kalenderwoche.
  Erzeugt Segment 1 (Original bis KW-1), Segment 2 (Ausnahme in KW)
  und Segment 3 (Wiederherstellung ab KW+1). Bookings der KW → Segment 2,
  spätere Bookings → Segment 3.
- **Copy-Week** (`copy_week_with_conflict_check`,
  `shiftplan_edit.rs:788+`): Iteriert Bookings der Quell-Woche und ruft
  pro Booking `book_slot_with_conflict_check`. Akkumuliert Cross-Source-
  Warnings ohne De-Dup (D-Phase3-15).

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `slot` | Wiederkehrender Zeit-Slot | `id`, `day_of_week` (1=Mo … 7=So), `time_from`, `time_to`, `valid_from`, `valid_to`, `min_resources`, `max_paid_employees`, `shiftplan_id`, `deleted`, `update_version` |
| `shiftplan` | Meta-Katalog | `id`, `name`, `is_planning`, `deleted`, `update_version` |
| `sales_person_shiftplan` | N:M-Zuordnung Person→Plan | `sales_person_id`, `shiftplan_id`, `permission_level` (`available` / `planner_only`) |
| `bookings_view` (View) | Denormalisierte Read-Optimierung | Bookings + `sales_person.name` + `slot.day_of_week` / `time_from` / `time_to` + `shiftplan.name` |

### Migrations

Chronologische Aufbau-Geschichte der Slot-/Shiftplan-Tabellen:

- `20240502113031_add-slot.sql` — Basistabelle `slot` (ohne
  `min_resources`, ohne `shiftplan_id`, ohne `max_paid_employees`).
- `20240619085745_default-slots.sql` — **Default-Slot-Set:** 63 Slots
  (Mo-Sa, meist 09:00-19:30 in 1h-Blöcken; letzter Slot 19:00-19:30).
  Alle mit `valid_from = 2020-01-01`, kein `valid_to`. Werden von der
  Migration IDs+Versions fest verankert eingespielt.
- `20240813080347_add-column-min-resources.sql` — `min_resources INTEGER
  DEFAULT 2 NOT NULL`.
- `20260330000000_add-shiftplan-table.sql` — Multi-Plan-Support: legt
  `shiftplan`-Tabelle an, fügt `slot.shiftplan_id` FK ein, backfillt alle
  Legacy-Slots auf `main` (UUID `…0001`), erweitert `bookings_view` um
  `shiftplan_name`.
- `20260331000000_add-sales-person-shiftplan.sql` — N:M-Tabelle
  `sales_person_shiftplan`.
- `20260402000000_add-permission-level-to-sales-person-shiftplan.sql` —
  ergänzt `permission_level`.
- `20260503221640_add-max-paid-employees-to-slot.sql` —
  `max_paid_employees INTEGER` (nullable).

### Beziehungen

```
shiftplan ──1:N── slot ──1:N── booking
     │
     └──N:M── sales_person (via sales_person_shiftplan)
```

Ein Slot gehört (seit v2.x) genau einem Schichtplan; historisch (vor
Migration `20260330000000`) war `slot.shiftplan_id NULL` erlaubt und wurde
mittels UPDATE-Statement der Migration backfilled auf `main`.

## 4. Service-API

### Traits

**Basic-Tier** (nur DAO + Permission + Transaction als Deps):

- `service::slot::SlotService` (`service/src/slot.rs:98-154`) — CRUD +
  Read-by-Week. Konsumiert nur `SlotDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `service::shiftplan_catalog::ShiftplanService`
  (`service/src/shiftplan_catalog.rs:45-84`) — CRUD für die Katalog-Rows.
  Konsumiert nur `ShiftplanDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao`
  (`service_impl/src/shiftplan_catalog.rs:15-23`).

**Business-Logic-Tier** (konsumieren andere Services):

- `service::shiftplan::ShiftplanViewService`
  (`service/src/shiftplan.rs:86-143`) — Read-Aggregat. Deps:
  `SlotService`, `BookingService`, `SalesPersonService`,
  `SpecialDayService`, `ShiftplanService`, `AbsenceService`,
  `SalesPersonUnavailableService`, `ToggleService`
  (`service_impl/src/shiftplan.rs:216-231`).
- `service::shiftplan_edit::ShiftplanEditService`
  (`service/src/shiftplan_edit.rs:35-157`) — Write-Aggregat. Deps:
  `SlotService`, `BookingService`, `CarryoverService`, `ReportingService`,
  `SalesPersonService`, `SalesPersonUnavailableService`,
  `EmployeeWorkDetailsService`, `ExtraHoursService`, `AbsenceService`,
  `ToggleService`, `WeekStatusService` +
  Utility-Deps (`service_impl/src/shiftplan_edit.rs:27-48`).
- `service::shiftplan_report::ShiftplanReportService`
  (`service/src/shiftplan_report.rs:33-63`) — Aggregiert Stunden pro
  Person aus rohen Booking-Zeiten. Deps: `ShiftplanReportDao`,
  `SpecialDayService`, `ToggleService`, `TransactionDao`
  (`service_impl/src/shiftplan_report.rs:36-43`).

### Wichtigste Methoden-Signaturen

```rust
// SlotService
async fn create_slot(&self, slot: &Slot, ctx, tx) -> Result<Slot, ServiceError>;
async fn update_slot(&self, slot: &Slot, ctx, tx) -> Result<(), ServiceError>;
async fn delete_slot(&self, id: &Uuid, ctx, tx) -> Result<(), ServiceError>;
async fn get_slots_for_week(&self, year, week, shiftplan_id: Uuid, ctx, tx)
    -> Result<Arc<[Slot]>, ServiceError>;

// ShiftplanViewService
async fn get_shiftplan_week(&self, shiftplan_id, year, week, ctx, tx)
    -> Result<ShiftplanWeek, ServiceError>;
async fn get_shiftplan_week_for_sales_person(&self, shiftplan_id, year, week,
    sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>;

// ShiftplanEditService (Auszug)
async fn modify_slot(&self, slot: &Slot, change_year, change_week, ctx, tx)
    -> Result<Slot, ServiceError>;
async fn modify_slot_single_week(&self, slot: &Slot, change_year, change_week,
    ctx, tx) -> Result<Slot, ServiceError>;
async fn remove_slot(&self, slot: Uuid, change_year, change_week, ctx, tx)
    -> Result<(), ServiceError>;
async fn book_slot_with_conflict_check(&self, booking: &Booking, ctx, tx)
    -> Result<BookingCreateResult, ServiceError>;
async fn copy_week_with_conflict_check(&self, from_cw, from_year, to_cw,
    to_year, ctx, tx) -> Result<CopyWeekResult, ServiceError>;
async fn delete_booking(&self, booking_id, ctx, tx) -> Result<(), ServiceError>;
```

### Auth-Gates

| Methode | Privilege |
| --- | --- |
| `SlotService::get_*` | `shiftplanner` **oder** `sales` (`slot.rs:83-88, 110-115, 138-143, 167-172`) |
| `SlotService::create_slot` / `update_slot` / `delete_slot` | `shiftplanner` (`slot.rs:211, 269, 292`) |
| `ShiftplanService::create` / `update` / `delete` | `shiftplanner` (`shiftplan_catalog.rs:66-67, 100, 138`) **[Zu prüfen]** exakte Zeilen |
| `ShiftplanViewService::get_shiftplan_week` | siehe intern gebündelte Slot+Booking+Sales-Reads; effektiv `shiftplanner ∨ sales` |
| `ShiftplanViewService::get_shiftplan_*_for_sales_person` | HR **oder** `verify_user_is_sales_person(sales_person_id)` (D-Phase3-12) |
| `ShiftplanEditService::modify_slot` / `modify_slot_single_week` / `remove_slot` | `shiftplan.edit` (`shiftplan_edit.rs:66, 163, 223`) + Wochen-Sperre-Gate |
| `ShiftplanEditService::book_slot_with_conflict_check` | Shiftplanner ∨ Self (D-24-04, `shiftplan_edit.rs:573-589`) |
| `ShiftplanEditService::copy_week_with_conflict_check` | `shiftplan.edit` (Bulk-Operation) |
| `ShiftplanEditService::delete_booking` | delegiert an `BookingService::delete` (Shiftplanner ∨ Self) + Week-Lock |
| `ShiftplanReportService::extract_*` | **[Zu prüfen]** — Auth-Gate liegt intern; SpecialDay/Toggle-Reads laufen unter dem übergebenen Kontext |

### TX-Verhalten

Alle Methoden folgen dem Standard-Pattern `use_transaction(tx).await?` →
Business-Logik → `commit(tx).await?`. Speziell:

- **`modify_slot`** (`shiftplan_edit.rs:56-151`): Öffnet EINE TX,
  update+create des Slots + Booking-Re-Point + Commit. Bei Fehler
  irgendwo → Rollback der gesamten Kette (kritisch, siehe Kapitel 7).
- **`modify_slot_single_week`** (D-35-04): 3-Segment-Split komplett in
  einer TX.
- **`remove_slot`** (`shiftplan_edit.rs:153-208`): Setzt `valid_to` auf
  Sonntag KW-1 (bzw. löscht Slot, wenn Range komplett verschwindet) und
  soft-deleted alle Bookings ab `change_week` — in einer TX.
- **`copy_week_with_conflict_check`** (`shiftplan_edit.rs:788+`): Iteriert
  Bookings der Quell-Woche, ruft pro Booking `book_slot_with_conflict_check`
  intern — Warnings werden akkumuliert, TX umspannt alles.

### Wichtiger Fat-Backend-Punkt

`ShiftplanEditService` ist das Beispiel für einen Business-Logic-
Service, der 12 andere Services orchestriert und alle Cross-Aggregate-
Regeln bündelt. Das FE muss **nichts** davon reproduzieren — es ruft
`POST /shiftplan-edit/booking` und bekommt die vollständige Warning-Liste
zurück (Fat Backend / Thin Client, siehe
[Memory-Feedback](../../../CLAUDE.md)).

## 5. REST-Endpoints

Mounts (`rest/src/lib.rs:638-672`):

| Prefix | Modul |
| --- | --- |
| `/slot` | `rest::slot` |
| `/shiftplan-catalog` | `rest::shiftplan_catalog` |
| `/shiftplan-edit` | `rest::shiftplan_edit` |
| `/shiftplan-info` | `rest::shiftplan` (View-Endpunkte) |
| `/shiftplan` | PDF-Export (siehe `F11-export.md`) |

### `/slot`

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Fehler |
| --- | --- | --- | --- | --- | --- |
| GET | `/` | Alle Slots | — | `Vec<SlotTO>` | 401 |
| GET | `/{id}` | Ein Slot | — | `SlotTO` | 404 |
| GET | `/week/{year}/{month}/{shiftplan_id}` | Slots einer Woche (Pfad-Name `month` ist historisch, tatsächlich `week`) | — | `Vec<SlotTO>` | 401 |
| POST | `/` | Neuen Slot anlegen | `SlotTO` | `SlotTO` | 403, 422 (`OverlappingTimeRange`, `TimeOrderWrong`) |
| PUT | `/{id}` | Slot updaten | `SlotTO` | `SlotTO` | 403, 409 (`EntityConflicts`), 422 (`ModificationNotAllowed`) |

### `/shiftplan-catalog`

| Methode | Pfad | Beschreibung | DTO |
| --- | --- | --- | --- |
| GET | `/` | Alle Schichtpläne | `Vec<ShiftplanTO>` |
| GET | `/{id}` | Ein Schichtplan | `ShiftplanTO` |
| POST | `/` | Anlegen | `ShiftplanTO` |
| PUT | `/{id}` | Update | `ShiftplanTO` |
| DELETE | `/{id}` | Soft-Delete | — |

### `/shiftplan-info`

| Methode | Pfad | Beschreibung | DTO Out |
| --- | --- | --- | --- |
| GET | `/{shiftplan_id}/{year}/{week}` | Wochen-Sicht eines Plans | `ShiftplanWeekTO` |
| GET | `/day/{year}/{week}/{day_of_week}` | Tages-Aggregat über alle Pläne | `ShiftplanDayAggregateTO` |
| GET | `/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}` | Wochen-Sicht mit `unavailable`-Markern für 1 Person | `ShiftplanWeekTO` |
| GET | `/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}` | Tages-Aggregat mit Markern | `ShiftplanDayAggregateTO` |

Die per-sales-person-Varianten setzen `ShiftplanDayTO.unavailable` auf
`AbsencePeriod` / `ManualUnavailable` / `Both` (D-Phase3-10; siehe
[`F05-absence-system.md`](./F05-absence-system.md)).

### `/shiftplan-edit`

| Methode | Pfad | Beschreibung | DTO In | Fehler |
| --- | --- | --- | --- | --- |
| PUT | `/slot/{year}/{week}` | Slot ab KW modifizieren (Slot-Split) | `SlotTO` | 403, 409, 423 (Week locked) |
| PUT | `/slot/{year}/{week}/single-week` | Slot für genau 1 KW ändern (D-35) | `SlotTO` | 403, 409, 423 |
| DELETE | `/slot/{slot_id}/{year}/{week}` | Slot ab KW schließen | — | 403, 423 |
| PUT | `/vacation` | Legacy: Urlaub eintragen (extra_hours + unavailable) | `VacationPayloadTO` | 403 |
| POST | `/booking` | Booking konflikt-aware anlegen | `BookingTO` | 403, 409 (Paid-Limit), 422, 423 |
| POST | `/copy-week` | Woche kopieren mit Konflikt-Prüfung | `CopyWeekRequest` | 403, 423 |

DTOs siehe `rest-types/src/lib.rs`: `SlotTO` (`:308`), `ShiftplanTO`
(`:15`), `ShiftplanWeekTO` (`:1103`), `ShiftplanDayTO` (`:1092`),
`ShiftplanSlotTO` (`:1070`), `ShiftplanBookingTO` (`:1063`).
`BookingCreateResultTO` / `CopyWeekResultTO` (Phase-3-Warning-Aggregate).

**Wichtig zur Toggle-Semantik von `ShiftplanSlotTO`:** Feld `slot.to`
bleibt roh (bidirektional-DTO-Regel P07); die tages-effektive Endzeit
liegt in `effective_to`
(`service/src/shiftplan.rs:52-60`) — nur der ShortDay-Cutoff des D-51-07
Stichtag-Gates verschiebt sie (Phase 51).

## 6. Frontend-Integration

- **Pages:**
  `shifty-dioxus/src/page/shiftplan.rs` (2194 Zeilen — die zentrale Seite
  für den gesamten Schichtplan-Workflow: Katalog-Auswahl,
  Wochen-Navigation, Slot-Rendering, Booking-Aktionen, Week-Message,
  Week-Status, PDF-Export-Button).
- **Components:**
  - `component/shiftplan_tab_bar.rs` — Katalog-Auswahl.
  - `component/slot_edit.rs` — Slot-Editor-Modal (öffnet für create,
    modify_slot, modify_slot_single_week und remove_slot).
- **Services / Coroutines:**
  `service/slot_edit.rs::SlotEditAction` — bündelt Slot-Änderungen.
- **Loader:** `loader::load_shift_plan`, `loader::load_shiftplan_catalog`,
  `loader::load_day_aggregate`, `loader::register_user_to_slot_with_conflict_check`,
  `loader::remove_user_from_slot` — thin HTTP-Wrapper um die Backend-Endpunkte.
- **Proxy:** `shifty-dioxus/Dioxus.toml` mappt (verifiziert per grep):
  - `/slot` → `http://localhost:3000/slot`
  - `/shiftplan-edit` → `…/shiftplan-edit`
  - `/shiftplan-info` → `…/shiftplan-info`
  - `/shiftplan-catalog` → `…/shiftplan-catalog`
  - `/shiftplan` → `…/shiftplan` (PDF)
  - `/sales-person-shiftplan` → `…/sales-person-shiftplan`

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektionen
„Atomarität + Re-Point-Tests" und „Backend-Roundtrip e2e prüfen".

- **Slot-Split ohne Booking-Re-Point in derselben TX** —
  `modify_slot` (`shiftplan_edit.rs:56-151`) muss die Booking-Migration
  in derselben Transaktion wie die Slot-Duplizierung ausführen. Wird das
  gebrochen, sind Bookings entweder doppelt (am alten und neuen Slot,
  Report zählt 2×) oder verwaist (am gelöschten alten Slot, Report zählt
  gar nicht). Regression-Guard: siehe
  `service_impl/src/test/shiftplan_edit.rs`.
- **`modify_slot` verschluckt `max_paid_employees`** — Historischer Bug
  aus Phase 23: die Update-Kaskade übernahm die neuen `max_paid_employees`
  nicht mit. Fix ist heute in `shiftplan_edit.rs:117-120` sichtbar
  (Zeile `new_slot.max_paid_employees = slot.max_paid_employees`). Siehe
  auch [MemPalace-Feedback „Backend-Roundtrip
  e2e"](../domain/edge-cases.md#backend-roundtrip-e2e-pruefen). Regression:
  create-Pfad ≠ edit-Pfad → immer beide manuell durchklicken beim Test.
- **Wochentag-Rollout** — Ein neuer Slot mit `day_of_week = Sonntag`
  greift ab Montag der `valid_from`-Woche; wird die ISO-Woche knapp
  gewählt (z.B. `valid_from = Sonntag`), verliert man den ersten Tag,
  weil die Slot-Semantik immer eine ganze KW braucht. Praktisch
  bevorzugt `create_slot` einen Montag-Termin als `valid_from`.
  **[Zu prüfen]** ob Backend das validiert oder erst der Report darauf
  reagiert.
- **`update_slot` verbietet Zeitfenster-Änderungen** —
  `service_impl/src/slot.rs:319-329` blockt `from`/`to`/`valid_from`/
  `day_of_week`-Mutationen mit `ModificationNotAllowed`. Der übliche Weg
  ist stattdessen `shiftplan_edit::modify_slot` (mit Slot-Split), das
  über die REST-Route `PUT /shiftplan-edit/slot/{year}/{week}` läuft.
  Frontend nutzt entsprechend nie direkt `PUT /slot/{id}` für
  strukturelle Änderungen.
- **Wochen-Sperre-Gate** (Phase 40) blockt ALLE Schreib-Pfade in
  `ShiftplanEditService` (auch Bookings). `shiftplan.edit` ist Bypass;
  `shiftplanner` alleine reicht nicht. Ohne diesen Bypass würde ein
  Shiftplanner in gesperrter Woche stumm blockiert (CR-01, 2026-07-02
  gefixt).
- **Default-Slot-Set** — Die 63 Migration-Slots aus
  `20240619085745_default-slots.sql` sind produktiv **NICHT zwingend**
  vorhanden — Nix-/CI-Datenbanken bekommen sie; dev-DBs, die einmal
  `sqlx database reset` mitbekommen haben, ebenso. Für neue Kunden ist
  das Set der fachlich sinnvolle Ausgangspunkt (Laden 9-19:30 Uhr).
- **`shiftplan_id NULL` auf Legacy-Slots** — Migration
  `20260330000000` fügt `shiftplan_id` als nullable Spalte hinzu und
  backfillt sofort auf `main`. Neu-Anlage per
  `SlotService::create_slot` verlangt `shiftplan_id.is_some()`
  (`slot.rs:220-224`) — es sollten faktisch keine NULL-Rows mehr
  entstehen.
- **PDF-Button-Sichtbarkeit** — `page/shiftplan.rs::should_show_pdf_button`
  (Zeilen 95-97) verlangt einen gewählten Schichtplan + Status `Planned`
  oder `Locked` (siehe [`F11-export.md`](./F11-export.md) für den
  Export-Flow).

## 8. Tests

- **Unit (Slot):** `service_impl/src/test/slot.rs` (1186 Zeilen). Deckt
  ab: `test_get_slots{,_sales_role,_no_permission}` (Zeilen 156-211),
  `test_get_slot{,_sales_role,_not_found,_no_permission}` (212-263),
  `test_create_slot{,_no_permission,_non_zero_id,_non_zero_version,
  _intersects,_time_order,_date_order}` (264-580),
  `test_delete_slot` (580+). `test_overlapping_slots` hat den Overlap-
  Algorithmus inline (`service_impl/src/slot.rs:55-60`) plus die
  Regression-Suite. `clip_to`-Fachlogik: `service/src/slot.rs:176-245`.
- **Unit (View / Edit):**
  - `service_impl/src/test/shiftplan.rs` (1659 Zeilen) — Wochen-/Tages-
    Aggregate inklusive `unavailable`-Marker.
  - `service_impl/src/test/shiftplan_edit.rs` (1997 Zeilen) — Slot-Split,
    Booking-Re-Point, Copy-Week, Konflikt-Warnings, Paid-Limit-
    Enforcement.
  - `service_impl/src/test/shiftplan_edit_lock.rs` (565 Zeilen) —
    Wochen-Sperre-Regression-Suite (Phase 40).
  - `service_impl/src/test/shiftplan_catalog.rs` (290 Zeilen) — CRUD +
    Auth.
  - `service_impl/src/test/shiftplan_report.rs` (612 Zeilen) — Roh-Row-
    Aggregation + ShortDay-Gate (Phase 51 Chain D).
- **Integration:** `shifty_bin/src/integration_test/booking_absence_conflict.rs`
  fährt `book_slot_with_conflict_check` und
  `get_shiftplan_week_for_sales_person` end-to-end gegen In-Mem-SQLite
  durch.
- **Bekannte Lücken:**
  - `min_resources`-Untersetzung wird backend-seitig aktuell nicht hart
    validiert (nur FE-Hinweis). **[Zu prüfen]**
  - Explizite Tests, dass `modify_slot` beim Fehlschlag ROLLBACK
    ausführt, sind nur implizit durch die Transaktions-Boundary
    abgedeckt.

## 9. Historie & Kontext

- **Slots seit Milestone 0** (2024-05-02). Das Modell wurde iterativ
  erweitert: `min_resources` (Aug 2024), `shiftplan_id` +
  Multi-Plan-Katalog (März 2026), `max_paid_employees` (Mai 2026).
- **Phase 23 (v1.2-ish)** — Fix: `modify_slot` behielt neue
  `max_paid_employees` nicht. Trigger: Frontend-Edit sah keinen Effekt,
  weil der Edit-Pfad ≠ Create-Pfad war. Seither MemPalace-Feedback
  „Backend-Roundtrip e2e prüfen".
- **Phase 35 (D-35)** — `modify_slot_single_week` als 3-Segment-Split
  eingeführt (einmalige Ausnahme für genau eine KW), damit
  Shiftplanner nicht mehr manuell die Slot-Version + Roll-back-Slot
  anlegen müssen.
- **Phase 40 (D-40)** — Wochen-Sperre / Week-Lock. Alle Schreib-Pfade
  in `ShiftplanEditService` gate'n gegen `WeekStatusService`;
  `shiftplan.edit` ist Bypass. CR-01 (2026-07-02): auch der reine
  Booking-Pfad muss gate'n, `shiftplanner` alleine ist kein Bypass.
- **Phase 3 / Phase-3-Konflikt-Warnings** — Erweiterung der
  Booking-Pfade um `BookingCreateResult` mit `warnings: Arc<[Warning]>`
  (Cross-Source: AbsencePeriod, Manual-Unavailable, ohne De-Dup pro Tag).
  `ShiftplanViewService::get_*_for_sales_person`-Varianten setzen
  `unavailable`-Marker (D-Phase3-10/-12).
- **Phase 51 Chain D** — `ShiftplanReportService` liest Roh-Zeilen
  (`ShiftplanReportRawRow`) statt SQL-Aggregat; `Slot::clip_to`-Fachlogik
  + `shortday_gate` aggregieren in Rust (D-51-08). Voraussetzung für
  den D-51-07 Stichtag-Toggle. Alte `Shiftplan{Report,QuickOverview}Entity`
  entfielen (`dao/src/shiftplan_report.rs:9-12`).
- **Toggle-Kontext:** `paid_limit_hard_enforcement`
  (`shiftplan_edit.rs:618-660`) und `shortday_active_from`
  (`shiftplan_report.rs:83-95`) sind die aktiven Rollout-Toggles im
  Cluster.
- **Verweise auf Planning-Artefakte:** siehe `.planning/phases/23-*`,
  `.planning/phases/35-*`, `.planning/phases/40-*`, `.planning/phases/51-*`
  für die vollen Kontext-Reads.

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
