# Phase 3: Booking & Shift-Plan Konflikt-Integration - Context

**Gathered:** 2026-05-02
**Updated:** 2026-05-02 (Re-Discuss: Service-Tier-Konvention etabliert; siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services")
**Status:** Ready for planning

<domain>
## Phase Boundary

Bookings und Shift-Plan-Anzeige werden konflikt-aware bezüglich `AbsencePeriod` (Phase 1). Drei Wirkungs-Pfade:

1. **Forward-Warning (BOOK-01):** Beim Anlegen oder Update einer `AbsencePeriod`, die ein bestehendes Booking überlappt, gibt `AbsenceService::create/update` einen Wrapper mit `Vec<Warning>` zurück (mit konkreten Booking-IDs und Daten). Die Absence wird trotzdem persistiert; kein Auto-Löschen, kein Block. `AbsenceService` ist Business-Logic-Tier (siehe CLAUDE.md) — er darf `BookingService` (Basic-Tier) und `SalesPersonUnavailableService` (Basic-Tier) einseitig konsumieren.
2. **Reverse-Warning (BOOK-02):** Beim Anlegen eines Bookings auf einem Tag, der entweder durch `AbsencePeriod` oder durch `sales_person_unavailable` als nicht verfügbar markiert ist, gibt ein **neuer Schreib-Pfad oben** (Business-Logic-Tier; natürlicher Andock-Punkt ist `ShiftplanEditService`, der `BookingService` heute schon als Dep hält) eine Warnung zurück. Booking wird trotzdem angelegt. **`BookingService` selbst bleibt strikt basic — keine neuen Deps, keine Signatur-Brüche, keine Warning-Produktion.** `POST /booking` bleibt unverändert; das Frontend wird auf den neuen konflikt-aware-Endpunkt umgezogen.
3. **Shift-Plan-Markierung (PLAN-01):** Eine neue per-sales-person-Variante des `ShiftplanViewService` markiert pro Tag, ob er durch `AbsencePeriod` oder `sales_person_unavailable` (oder beides) abgedeckt ist. Die bisherige Doppel-Eintragung (ExtraHours + sales_person_unavailable) entfällt für die zeitraum-basierten Kategorien.

**Architektur-Direction (aus STATE.md, Phase-1-D-08, Phase-3-Discuss + Re-Discuss 2026-05-02):**
- `BookingService` ist **Basic-Tier** (Entity-Manager): konsumiert nur DAOs + Permission + Transaction. Keine Domain-Service-Deps, weder schreibend noch lesend.
- `AbsenceService` ist **Business-Logic-Tier**: darf `BookingService` und `SalesPersonUnavailableService` einseitig konsumieren (für Forward-Warning-Read).
- `ShiftplanEditService` ist **Business-Logic-Tier**: darf alle drei Basic-Services konsumieren (`BookingService`, `AbsenceService` ist Mid-Tier, `SalesPersonUnavailableService`); produziert die Reverse-Warning + Wrapper-Results für konflikt-aware-Schreiben.
- `ShiftplanViewService` ist **Business-Logic-Tier**: konsumiert `BookingService`, `AbsenceService`, `SalesPersonUnavailableService` — produziert die per-sales-person-Sicht mit `UnavailabilityMarker`.
- Soft-deleted `AbsencePeriod`s triggern keine Warnung, keine Markierung (Pitfall 6 / SC4).
- Kein Cycle in der Service-Hierarchie: `BookingService → BookingDao` (Tree-blatt); `AbsenceService → BookingService + SalesPersonUnavailableService` (einseitig); `ShiftplanEditService → BookingService + AbsenceService + SalesPersonUnavailableService` (einseitig); `ShiftplanViewService` analog.

**In Scope (Phase 3):**
- Neue DAO-Methode `AbsenceDao::find_overlapping_for_booking(sales_person_id, range, tx) -> Arc<[AbsencePeriodEntity]>` (cross-kategorie, single SQL-Query mit IN-Clause oder ohne Category-Filter, nutzt bestehenden composite index `(sales_person_id, from_date) WHERE deleted IS NULL`).
- Neue Service-Methode `AbsenceService::find_overlapping_for_booking(sales_person_id, range, ctx, tx)` (HR ∨ self).
- `enum Warning { BookingOnAbsenceDay { booking_id, date, absence_id, category }, BookingOnUnavailableDay { booking_id, year, week, day_of_week }, AbsenceOverlapsBooking { absence_id, booking_id, date }, AbsenceOverlapsManualUnavailable { absence_id, unavailable_id } }` in `service/src/lib.rs` (oder eigenem Modul, Plan-Phase entscheidet).
- `AbsenceService::create` Signatur-Bruch auf `Result<AbsencePeriodCreateResult, ServiceError>` mit `struct AbsencePeriodCreateResult { absence: AbsencePeriod, warnings: Arc<[Warning]> }`.
- `AbsenceService::update` Signatur-Bruch auf `Result<AbsencePeriodCreateResult, _>` symmetrisch zu `create` — liefert volle Warning-Liste über die NEUE Range (kein Diff-Modus).
- `AbsenceService` bekommt `BookingService` + `SalesPersonUnavailableService` als neue Dependencies (Business-Logic-Tier konsumiert Basic-Tier; Direction `AbsenceService → BookingService` + `AbsenceService → SalesPersonUnavailableService`):
  - Booking-Lookup via `BookingService::get_for_week` (oder neuer Methode für Range — Plan-Phase darf wählen, falls Performance es nahelegt) — produziert `Warning::AbsenceOverlapsBooking` pro betroffenem Booking-Tag.
  - `sales_person_unavailable_service.get_all_for_sales_person` (oder Range-Variante) — produziert `Warning::AbsenceOverlapsManualUnavailable` pro überlappendem manuellen Eintrag. **Kein Auto-Cleanup** — User entscheidet selbst, ob er die alten Unavailables löscht.
- `BookingService::create` und `BookingService::copy_week` bleiben **unverändert** — keine Signatur-Brüche, keine neuen Deps, keine Warning-Produktion. Begründung: Service-Tier-Konvention (siehe CLAUDE.md § "Service-Tier-Konventionen") — Basic Services managen nur ihr Fach-Objekt.
- **Neuer konflikt-aware-Schreib-Pfad im Business-Logic-Tier** (Plan-Phase wählt Lokation und Naming; Vorgabe: Erweiterung des bestehenden `ShiftplanEditService`, der `BookingService` als Dep schon hält):
  - Neue Methode `ShiftplanEditService::book_slot_with_conflict_check(...) -> Result<BookingCreateResult, ServiceError>` mit `struct BookingCreateResult { booking: Booking, warnings: Arc<[Warning]> }`.
  - Neue Methode `ShiftplanEditService::copy_week_with_conflict_check(...) -> Result<CopyWeekResult, ServiceError>` mit `struct CopyWeekResult { copied_bookings: Arc<[Booking]>, warnings: Arc<[Warning]> }` — aggregiert Warnings über alle inneren `BookingService::create`-Calls.
  - Date-Konversion `time::Date::from_iso_week_date(year, calendar_week, slot.day_of_week)` lebt **hier**, vor dem internen `BookingService::create`-Aufruf.
  - Lookup-Sequenz: Slot lesen → Date konstruieren → `AbsenceService::find_overlapping_for_booking(sales_person_id, single_day_range, ...)` → `Warning::BookingOnAbsenceDay` pro Treffer; → `SalesPersonUnavailableService::get_by_week_for_sales_person(sales_person_id, year, week, ...)` → `Warning::BookingOnUnavailableDay` wenn `(day_of_week)` matcht; → `BookingService::create` für die eigentliche Persistierung; → Warnings mit dem persistierten Booking zum `BookingCreateResult` zusammenfügen.
  - Dependencies: `BookingService` (basic, schon vorhanden), `AbsenceService` (business-logic, neu), `SalesPersonUnavailableService` (basic, neu), `SlotService`/`SalesPersonService` (basic, je nach bestehender Verdrahtung), `PermissionService`, `TransactionDao`.
- Neue Methode `ShiftplanViewService::get_shiftplan_week_for_sales_person(shiftplan_id, year, week, sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>` und symmetrisch `get_shiftplan_day_for_sales_person`.
- `ShiftplanDay` bekommt neues Feld `unavailable: Option<UnavailabilityMarker>` mit `enum UnavailabilityMarker { AbsencePeriod { absence_id, category }, ManualUnavailable, Both { absence_id, category } }`. Slots bleiben sichtbar — Frontend rendert per-Tag-Badge zusätzlich.
- Permission für die per-sales-person-Methode: HR ∨ `verify_user_is_sales_person(sales_person_id)` (Phase-1-D-09-Pattern). Gleiches Pattern für die neuen `ShiftplanEditService`-Methoden.
- **REST-Endpunkte:**
  - `POST /booking` und `POST /booking/copy-week` bleiben **unverändert** (BookingTO im Body, keine Wrapper). Frontend wird parallel auf die neuen Endpunkte umgezogen.
  - **Neue Endpunkte** für konflikt-aware-Booking, im REST-Bereich des `ShiftplanEditService` (oder neue Route-Gruppe — Plan-Phase wählt). Beispiel-Naming: `POST /shiftplan-edit/booking`, `POST /shiftplan-edit/copy-week`. Beide geben Wrapper-DTOs (`BookingCreateResultTO`, `CopyWeekResultTO`) zurück. ApiDoc + utoipa-Annotation Pflicht.
  - `POST /absence-period` und `PATCH /absence-period/{id}` geben Wrapper-DTO (`AbsencePeriodCreateResultTO`) zurück — Signatur-Bruch akzeptiert (AbsenceService ist Business-Logic-Tier; Wrapper ist sein eigenes Result).
  - Neuer REST-Endpunkt `GET /shiftplan/{shiftplan_id}/year/{year}/week/{week}/sales-person/{sales_person_id}` und entsprechend für day-Aggregat (per-sales-person-View). ApiDoc + utoipa-Annotation Pflicht.
- REST-Wrapper-DTOs: `BookingCreateResultTO { booking: BookingTO, warnings: Vec<WarningTO> }`, `AbsencePeriodCreateResultTO { absence: AbsencePeriodTO, warnings: Vec<WarningTO> }`, `CopyWeekResultTO { copied_bookings: Vec<BookingTO>, warnings: Vec<WarningTO> }` — alle inline in `rest-types/src/lib.rs` (Repo-Konvention aus Phase 1 Override). `WarningTO` als Tag-Enum, utoipa-konform.
- Status: REST-Endpunkte mit Wrapper-Result geben **immer** 200/201 (auch mit Warnings); Status-Code unterscheidet nicht zwischen "clean" und "mit Warnings".
- Pflicht-Tests:
  - `_forbidden`-Tests pro neue public service method (Permission-Pattern aus Phase 1 D-11).
  - Pitfall-6-Test: soft-deleted `AbsencePeriod` triggert KEINE Warnung und KEINE Markierung (SC4) — sowohl im Service-Test (Mock mit gelöschter Row) als auch im Integration-Test (`shifty_bin/src/integration_test/`).
  - Symmetrische Forward + Reverse Warning-Tests mit konkreten Fixture-IDs.
  - `copy_week_with_conflict_check`-Aggregations-Test: 3 Quell-Bookings, davon 2 auf Absence-Tagen → CopyWeekResult enthält 2 Warnings + 3 kopierte Bookings.
  - Cross-Source-Test: ein Booking-Tag, der durch BEIDE (AbsencePeriod + sales_person_unavailable) markiert ist → ZWEI Warnings (eine pro Quelle), Booking wird trotzdem angelegt.
  - ShiftplanDay-Marker-Test: `UnavailabilityMarker::Both` wird nur dann gesetzt, wenn an dem Tag wirklich beide Quellen treffen.
  - **Regression-Test (NEU durch Re-Discuss):** Bestehende Tests von `BookingService::create` + `BookingService::copy_week` bleiben grün — der Basic-Service wird nicht angefasst. Sicherstellt, dass die Konflikt-Detektion wirklich ausschließlich im neuen Schreib-Pfad oben lebt.

**Strikt nicht in Scope (Phase 3):**
- `SalesPersonUnavailableService::create` bleibt unverändert — keine Warning beim Anlegen einer manuellen Unavailable auf einem Absence-Tag (deferred, Folgephase). Das `Warning::ManualUnavailableOnAbsenceDay`-Pendant wird in Phase 3 NICHT als Enum-Variante eingeführt.
- Migration aus `ExtraHours` (Phase 4 / MIG-01).
- Validierungs-Gate / Cutover (Phase 4 / MIG-02..04).
- Atomares Feature-Flag-Flippen (Phase 4).
- Reporting-Kategorie-Trigger-Differenzierung (z.B. UnpaidLeave nur als Hinweis statt Warning) — Plan-Phase darf das aufgreifen, aber Default ist: alle 3 Kategorien (Vacation/Sick/UnpaidLeave) triggern gleichermaßen Warnings.
- Multi-Wochen-Range-Variante `get_shiftplan_for_sales_person(range)` — Frontend loopt mehrere Wochen wenn nötig; Backend bleibt per-Woche.
- Frontend (Dioxus) — separater Workstream. Frontend muss vom alten `POST /booking`-Endpunkt auf den neuen konflikt-aware-Endpunkt umgezogen werden — diese Umstellung ist NICHT Gegenstand dieser Phase, lebt im Frontend-Workstream.
- **Deprecation oder Entfernung des alten `POST /booking`-Endpunkts** — Phase 3 fügt den neuen Endpunkt nur HINZU; der alte bleibt vollständig funktional als Basic-Service-Surface. Eine spätere Phase entscheidet (oder entscheidet bewusst nicht), ihn zu deprecieren.
- REST-Deprecation alter Endpunkte (Phase 4 / MIG-05).

</domain>

<decisions>
## Implementation Decisions

### Warning API-Migration (Area A)

- **D-Phase3-01:** **Wrapper-Result lebt nur im Business-Logic-Tier.** `BookingService::create` bleibt unverändert (`Result<Booking, ServiceError>`); BookingService ist Basic-Tier und produziert keine Cross-Entity-Warnings. Stattdessen: neue Methode `ShiftplanEditService::book_slot_with_conflict_check(...) -> Result<BookingCreateResult, ServiceError>` mit `struct BookingCreateResult { booking: Booking, warnings: Arc<[Warning]> }`. `AbsenceService::create -> Result<AbsencePeriodCreateResult, ServiceError>` mit `struct AbsencePeriodCreateResult { absence: AbsencePeriod, warnings: Arc<[Warning]> }` — AbsenceService IST Business-Logic-Tier, also darf der Wrapper dort leben. **Begründung:** Service-Tier-Konvention (siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen") trennt Basic-Services (Entity-Manager, nur DAO/Permission/Tx) von Business-Logic-Services (Cross-Entity-Aggregate). Reverse-Warnings sind Cross-Entity-Logik und gehören damit ins Business-Logic-Tier — nicht in den Basic-`BookingService`. Phase 1 D-08 hatte "API-Bruch oder additiver Wrapper?" auf Phase 3 verschoben; die Re-Discuss-Antwort 2026-05-02 lautet: **Wrapper-Bruch nur im Business-Logic-Tier; Basic-Service bleibt unangetastet.**
- **D-Phase3-02:** **`copy_week_with_conflict_check` aggregiert Warnings im Business-Logic-Tier.** `BookingService::copy_week` bleibt unverändert (`Result<(), ServiceError>` heutige Form). Neue Methode `ShiftplanEditService::copy_week_with_conflict_check(...) -> Result<CopyWeekResult, ServiceError>` mit `struct CopyWeekResult { copied_bookings: Arc<[Booking]>, warnings: Arc<[Warning]> }`. Alle Warnings aus der inneren Schleife (pro Booking ein `book_slot_with_conflict_check`-Call ODER eine optimierte Pre-fetch-Variante — Plan-Phase entscheidet) werden gesammelt; keine wird abgebrochen, keine swallowed. Konsistent mit `BookingCreateResult`.
- **D-Phase3-03:** **REST-Wrapper-DTOs am neuen Schreib-Pfad oben; alter Endpunkt unverändert.** `POST /booking` und `POST /booking/copy-week` bleiben unverändert (BookingTO im Body, keine Warnings). **Neue Endpunkte** im REST-Bereich des `ShiftplanEditService` (oder neue Route-Gruppe — Plan-Phase wählt Naming, Vorgabe: `POST /shiftplan-edit/booking` und `POST /shiftplan-edit/copy-week`) geben Wrapper-DTOs (`BookingCreateResultTO`, `CopyWeekResultTO`) zurück. `POST /absence-period` und `PATCH /absence-period/{id}` geben Wrapper-DTO (`AbsencePeriodCreateResultTO`) zurück — Signatur-Bruch dort ist OK, weil AbsenceService Business-Logic ist. Status-Code unterscheidet nicht zwischen "clean" und "mit Warnings" — Frontend prüft `warnings.is_empty()`. utoipa-Schema dokumentiert die neuen Endpunkte direkt; OpenAPI-Snapshot zeigt die Erweiterung. **Frontend-Migration vom alten zum neuen Booking-Endpunkt liegt im Frontend-Workstream** (nicht Phase 3).
- **D-Phase3-04:** **`AbsenceService::update` symmetrisch zu `create`.** `update -> Result<AbsencePeriodCreateResult, _>` (NICHT `Result<AbsencePeriod, _>`). Warnings werden für ALLE Tage in der NEUEN Range berechnet (kein Diff-Modus). Begründung: einfache Mental-Model ("wenn die Range sich ändert, kommen die Warnings raus, die du hättest, wenn du die Absence gerade neu angelegt hättest"); Diff-Logik wäre fehlerträchtig und bringt User keinen Vorteil ("ich sehe nur den Diff" wäre eine Frontend-Aufgabe, kein Backend-Feature).

### Cross-Source-Unavailability Lookup (Area B)

- **D-Phase3-05:** **Neue DAO-Methode mit IN-Clause.** `AbsenceDao::find_overlapping_for_booking(sales_person_id: Uuid, range: DateRange, tx: Self::Transaction) -> Result<Arc<[AbsencePeriodEntity]>, DaoError>` als single SQL-Query mit `WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL` (KEIN Category-Filter — alle 3 Kategorien werden zurückgegeben; Service-Layer entscheidet später, wie zu behandeln). Nutzt den bestehenden composite index `(sales_person_id, from_date) WHERE deleted IS NULL` aus Phase 1 D-04. Single roundtrip auch bei copy_week-Loops; Performance-skalierbar.
- **D-Phase3-06:** **`ShiftplanEditService` aggregiert beide Quellen direkt.** Der `ShiftplanEditService` (Business-Logic-Tier) bekommt `AbsenceService` (Business-Logic) + `SalesPersonUnavailableService` (Basic) als neue DI-Dependencies (zusätzlich zu seinem bestehenden `BookingService`-Dep). Im neuen `book_slot_with_conflict_check`-Pfad ruft er beide separat und konstruiert `Warning`-Varianten pro Quelle. **`BookingService` bleibt strikt basic — keine neuen Deps.** KEIN neuer `UnavailabilityService` als Aggregator (vermeidet Indirektion für nur 2 Quellen). Service-Tier-Direction: Business-Logic (`ShiftplanEditService`) → Basic (`BookingService`, `SalesPersonUnavailableService`) + Mid-Tier (`AbsenceService`); kein Cycle.
- **D-Phase3-07:** **Date-Konversion inline im Business-Logic-Tier.** Im Body der neuen `book_slot_with_conflict_check`-Methode (im `ShiftplanEditService`-Impl), direkt nach dem `BookingService::create`-internen Slot-Lookup oder als Pre-Step davor: `let booking_date = time::Date::from_iso_week_date(booking.year as i32, booking.calendar_week as u8, slot.day_of_week.into())?;` (oder analog mit der vorhandenen `DayOfWeek -> time::Weekday`-Konversion). Pattern existiert bereits in `service_impl/src/shiftplan.rs:138`. **`BookingService::create` selbst bekommt KEINE neuen Schritte.**
- **D-Phase3-08:** **`AbsenceService::create/update` lädt Bookings per `BookingService`.** Direction `AbsenceService → BookingService` ist erlaubt: AbsenceService ist Business-Logic-Tier, BookingService ist Basic-Tier; Business-Logic darf Basic einseitig konsumieren. Plan-Phase entscheidet, ob `BookingService::get_for_week` mehrfach für mehrere Kalenderwochen einer Range gerufen wird oder eine neue Range-Methode `BookingService::get_for_range(sales_person_id, range)` eingeführt wird (Vorgabe: erst die einfache Loop-Variante, optimieren falls Tests Performance-Druck zeigen). **Kein Cycle-Druck mehr** — `BookingService` weiß nichts von `AbsenceService`.

### Shift-Plan-Markierungs-Surface (Area C)

- **D-Phase3-09:** **`ShiftplanViewService` bekommt per-sales-person-Variante.** Neue Methoden `get_shiftplan_week_for_sales_person(shiftplan_id, year, week, sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>` und `get_shiftplan_day_for_sales_person(year, week, day_of_week, sales_person_id, ctx, tx) -> Result<ShiftplanDayAggregate, ServiceError>`. Bestehende globale `get_shiftplan_week` / `get_shiftplan_day` bleiben unverändert (additiv). `ShiftplanViewService` ist Business-Logic-Tier — er darf `AbsenceService` + `SalesPersonUnavailableService` als Deps halten.
- **D-Phase3-10:** **`ShiftplanDay` bekommt `unavailable: Option<UnavailabilityMarker>`.** `enum UnavailabilityMarker { AbsencePeriod { absence_id: Uuid, category: AbsenceCategory }, ManualUnavailable, Both { absence_id: Uuid, category: AbsenceCategory } }`. Pro Tag genau eine Variante — bei Doppel-Quelle wird `Both` gesetzt (mit der absence_id und category aus der AbsencePeriod, da die mehr semantischen Inhalt trägt als der bloße `sales_person_unavailable`-Eintrag). Slots bleiben sichtbar — Frontend rendert die Markierung als Per-Tag-Badge ZUSÄTZLICH zu den Slots.
- **D-Phase3-11:** **`build_shiftplan_day` bekommt eine optionale Parameter-Variante** (oder einen neuen Parallel-Helper `build_shiftplan_day_for_sales_person`), die zusätzlich `absence_periods: &[AbsencePeriod]` (für die Woche) und `manual_unavailables: &[SalesPersonUnavailable]` (für die Woche) entgegennimmt und das `unavailable`-Feld pro Tag ableitet. Plan-Phase entscheidet zwischen "Optional-Parameter" und "Parallel-Helper" — Vorgabe: Parallel-Helper, weil er die Globalsicht nicht touchiert (kein Risiko, Bestand zu brechen).
- **D-Phase3-12:** **Permission HR ∨ `verify_user_is_sales_person(sales_person_id)`.** Gleiche Regel wie AbsenceService Phase 1 D-09. Neuer REST-Endpoint `GET /shiftplan/{shiftplan_id}/year/{year}/week/{week}/sales-person/{sales_person_id}` und analog für day-Aggregate. Gleiche Permission auch für die neuen `ShiftplanEditService`-Methoden (`book_slot_with_conflict_check`, `copy_week_with_conflict_check`). ApiDoc + utoipa-Annotation Pflicht.
- **D-Phase3-13:** **Per-Woche-Scope.** Methoden-Signatur folgt dem bestehenden `get_shiftplan_week`-Pattern (`year + calendar_week`). Multi-Wochen-Sichten = Frontend-Loop. Backend bleibt schmal.

### Warning-Datenmodell + Doppel-Markierung (Area D)

- **D-Phase3-14:** **Warning als Enum mit klaren Varianten.**
  ```rust
  pub enum Warning {
      BookingOnAbsenceDay { booking_id: Uuid, date: time::Date, absence_id: Uuid, category: AbsenceCategory },
      BookingOnUnavailableDay { booking_id: Uuid, year: u32, week: u8, day_of_week: DayOfWeek },
      AbsenceOverlapsBooking { absence_id: Uuid, booking_id: Uuid, date: time::Date },
      AbsenceOverlapsManualUnavailable { absence_id: Uuid, unavailable_id: Uuid },
  }
  ```
  Compiler-Druck für alle Konsumenten (REST-Mapping zu `WarningTO`, Test-Asserts, copy_week-Aggregation). Jede Variante trägt nur die relevanten Felder. Modul-Lokation: Plan-Phase entscheidet (`service/src/warning.rs` neu vs. inline in `service/src/lib.rs` analog `ServiceError`).
- **D-Phase3-15:** **Granularität: eine Warning pro betroffenem Booking-Tag.** Bei einer 14-Tage-Absence über 3 Bookings: maximal 3 Warnings (eine pro konfliktiertem Booking) — KEINE Warning für absence-tage ohne Booking. Symmetrisch im Reverse-Pfad (1 Booking → max 1 oder 2 Warnings je nach Quellen). Konsistent für Frontend-Listen-Rendering.
- **D-Phase3-16:** **Doppel-Quelle: Warning + kein Auto-Cleanup.** Bei `AbsenceService::create/update`, das einen bestehenden `sales_person_unavailable`-Eintrag im Range überdeckt: `Warning::AbsenceOverlapsManualUnavailable { absence_id, unavailable_id }` wird ins Result gepackt. Die `sales_person_unavailable`-Einträge werden NICHT automatisch soft-deleted (irreversibel; bricht Architektur-Direction; bricht Re-Run-Idempotenz von Phase 4). User entscheidet selbst, ob er sie manuell entfernt. SC3 ("kein Konflikt zwischen den Quellen") wird nicht durch Cleanup, sondern durch die `UnavailabilityMarker::Both`-De-Dup in der Anzeige (Area C) erfüllt.
- **D-Phase3-17:** **`SalesPersonUnavailableService::create` bleibt unverändert (deferred).** Anlegen einer manuellen Unavailable auf einem Tag, der bereits durch eine `AbsencePeriod` abgedeckt ist, läuft still durch — keine Warning, kein Block. Phase 3 erweitert NUR Absence-Schreibpfade + den neuen konflikt-aware-Booking-Pfad oben. Eine Warning-Variante `ManualUnavailableOnAbsenceDay` wird in Phase 3 NICHT eingeführt — die Folgephase, die `SalesPersonUnavailableService::create` symmetrisiert, fügt sie dann hinzu (Vermeidung von dead enum-Arms). UI-Markierung-Logik (`UnavailabilityMarker::Both`) sieht den Doppel-Zustand sowieso.

### Service-Tier-Korollar (Re-Discuss 2026-05-02)

- **D-Phase3-18:** **`BookingService` bleibt strikt Basic-Tier.** Keine neuen Service-Deps, keine Signatur-Brüche, keine Warning-Produktion. Konsumiert ausschließlich `BookingDao`, `PermissionService`, `TransactionDao` (+ ggf. `SlotService`/`SalesPersonService` falls heute schon vorhanden, weil das primäre CRUD-Validation ist). **Begründung:** Service-Tier-Konvention in `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services". Konsequenz: Cross-Entity-Logik (Reverse-Warning) lebt im Business-Logic-Tier (`ShiftplanEditService`); jeder Konsument, der die Warnings sehen will, ruft den neuen Endpunkt — Plan-Phase darf REST-Routen so schneiden, wie Frontend es braucht.

### Claude's Discretion

- **C-Phase3-01:** **Modul-Lokation für `Warning` und `BookingCreateResult` / `AbsencePeriodCreateResult` / `CopyWeekResult`.** Plan-Phase wählt zwischen einem neuen `service/src/warning.rs` (eigenes Modul, sauber) und Inline in den jeweiligen Service-Modulen (`service/src/absence.rs`, `service/src/shiftplan_edit.rs`). Vorgabe: eigenes `warning.rs`-Modul für `Warning`-Enum (geteilt zwischen `AbsenceService` und `ShiftplanEditService`); Wrapper-Structs leben jeweils im Service-Modul, das sie produziert (`AbsencePeriodCreateResult` in `service/src/absence.rs`; `BookingCreateResult` + `CopyWeekResult` in `service/src/shiftplan_edit.rs`).
- **C-Phase3-02:** **Range-Lookup-API für AbsenceService::create.** Plan-Phase darf zwischen Mehrfach-`BookingService::get_for_week`-Calls (Loop über Kalenderwochen einer Range) und einer neuen `BookingService::get_for_range(sales_person_id, range)`-Methode wählen. Vorgabe: erst die Loop-Variante; nur optimieren wenn Tests einen messbaren Performance-Druck zeigen (premature-optimization-Vermeidung). **Hinweis:** Eine neue Methode auf `BookingService` ist erlaubt — sie ist immer noch CRUD-/Read-Surface des Booking-Aggregats und bricht die Service-Tier-Konvention nicht (Basic Services dürfen alle Read-/Write-Methoden auf ihrem eigenen Aggregat haben).
- **C-Phase3-03:** **`build_shiftplan_day_for_sales_person`-Layout.** Plan-Phase darf zwischen "neuer Helper neben `build_shiftplan_day`" und "Optional-Parameter am bestehenden Helper" wählen. Vorgabe: neuer Helper, weil er die globale Sicht nicht anfasst (Test-Stabilität für Phase 1/2-Tests).
- **C-Phase3-04:** **`Warning`-zu-`WarningTO`-Conversion.** Plan-Phase entscheidet zwischen `From<&Warning> for WarningTO` (Rust-Standard, einfach) und einer Tag-und-Daten-Map-Repräsentation für utoipa (`{ "kind": "BookingOnAbsenceDay", "data": { ... } }` mit `#[serde(tag = "kind", content = "data")]`). Vorgabe: Tag-und-Daten mit `#[serde(tag, content)]` — generiert sauberes OpenAPI-Schema für Frontend-Generatoren.
- **C-Phase3-05:** **Test-Fixture für Doppel-Quelle.** Plan-Phase darf entscheiden, ob ein dedizierter Cross-Source-Integration-Test in `shifty_bin/src/integration_test/` lebt oder ob er als Sub-Test eines bestehenden Booking/Absence-Integration-Tests folgt. Vorgabe: eigene Datei `shifty_bin/src/integration_test/booking_absence_conflict.rs` analog `absence_period.rs` aus Phase 1.
- **C-Phase3-06:** **Performance-Caching für `copy_week_with_conflict_check`.** N Bookings im konflikt-aware-copy-week = N `find_overlapping_for_booking`-DAO-Calls + N `sales_person_unavailable.get_by_week`-Calls. Plan-Phase darf vor der Schleife eine Pre-fetch-Optimierung einbauen (eine Range-Query, dann clientside-Filter), wenn Performance-Tests das nahelegen. Vorgabe: erst messen, dann optimieren — Default-Implementation = N Calls.
- **C-Phase3-07:** **Kategorie-Trigger-Differenzierung.** Default: alle 3 AbsenceCategorien (Vacation/Sick/UnpaidLeave) triggern Warnings gleichermaßen. Plan-Phase darf eine Differenzierung einführen (z.B. UnpaidLeave als nur "informativ"), wenn UX-Argumente vorliegen. Vorgabe: keine Differenzierung, alle gleich.
- **C-Phase3-08:** **Naming/Lokation des konflikt-aware-Schreib-Pfads.** Plan-Phase wählt zwischen "Erweiterung des bestehenden `ShiftplanEditService`" und "neuer dedizierter Service" (z.B. `BookingConflictService`). Vorgabe: `ShiftplanEditService` erweitern, weil er heute schon `BookingService` als Dep hält und semantisch zur "Schreib-Operationen auf Shift-Plan-Ebene"-Schicht gehört. Methoden-Naming `book_slot_with_conflict_check` / `copy_week_with_conflict_check` sind Vorschläge — Plan-Phase darf prägnantere Namen wählen (z.B. `book_slot_with_warnings` / `copy_week_with_warnings`).
- **C-Phase3-09:** **REST-Routen-Schnitt für die neuen Endpunkte.** Plan-Phase wählt zwischen "neue Route-Gruppe `/shiftplan-edit/booking`" und "Erweiterung der bestehenden `/shiftplan/...`-Routen". Vorgabe: neue Route-Gruppe parallel zu den alten `/booking`-Routen; alter Endpunkt bleibt unangetastet, neuer ist klar erkennbar. Frontend-Migration ist im Frontend-Workstream — Backend dokumentiert beide Endpunkte im OpenAPI.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level Spezifikationen

- `.planning/ROADMAP.md` § Phase 3 — Goal, Depends-on Phase 1, Success-Criteria 1-4. Insbesondere SC4 (Pitfall-6: soft-deleted AbsencePeriods triggern keine Warnung).
- `.planning/STATE.md` — Architektur-Decisions: Service-Tier-Konvention etabliert 2026-05-02 (Re-Discuss); Direction `AbsenceService → BookingService` (Business-Logic ↑ konsumiert Basic ↓).
- `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" — **MUST READ.** Definiert die Service-Tier-Trennung; Basic = nur DAO/Permission/Tx, Business-Logic = darf Domain-Services konsumieren. Begründet, warum `BookingService` strikt basic bleibt.
- `shifty-backend/CLAUDE.md` § "Implementation Patterns" — Layered Architecture; OpenAPI-Pflicht für REST; Service-Method-Signature-Pattern (`Option<Transaction>`).
- `shifty-backend/CLAUDE.local.md` — VCS via `jj` (alle Commits manuell durch User; GSD-Auto-Commit ist deaktiviert via `commit_docs: false`-Konvention im STATE/local-instructions); NixOS-Hinweise.
- `~/.claude/CLAUDE.md` — Tests sind Pflicht für jede Änderung.

### Vorphase-Outputs (Pflicht-Lektüre)

- `.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — D-08 (Phase-3-Wrapper-Decision), D-09 (Permission-Pattern HR ∨ self), D-12 (find_overlapping kategorie-scoped), D-15 (exclude_logical_id beim Update), D-16/17 (DateRange-API mit `iter_days`/`day_count` schon verfügbar).
- `.planning/phases/01-absence-domain-foundation/01-VERIFICATION.md` — Phase-1-Verification-Report; bestätigt dass `AbsenceService` + `AbsenceDao` end-to-end existieren.
- `.planning/phases/01-absence-domain-foundation/01-PATTERNS.md` — Pattern-Mapping aus Phase 1 (Code-Templates wiederverwendbar).
- `.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md` — D-Phase2-08-A (ReportingService-Switch hinter `absence_range_source_active` flag); für Phase 3 NICHT direkt relevant (Phase 3 ist flag-unabhängig — der `find_overlapping_for_booking`-Pfad nutzt KEIN `derive_hours_for_range`).
- `.planning/phases/03-booking-shift-plan-konflikt-integration/03-RESEARCH.md` — Tech-Recherche (Patterns, Pitfalls, Code-Anchors). **Achtung:** Sektion `## #1 Cycle-Vermeidung Booking ↔ Absence in gen_service_impl!-DI` ist durch die Re-Discuss obsolet (kein Cycle mehr); die übrigen Sektionen (Patterns 1-5, Pitfalls 1-7, Code-Examples Operation 1-5, Validation Architecture, Q-Open-1..5) sind weiterhin gültig — Plan-Phase liest sie als Referenz.

### Code-Templates für Phase 3

#### BookingService (Basic — bleibt unverändert)

- `service/src/booking.rs:63` — `BookingService`-Trait. **Phase 3 fügt KEINE neuen Methoden hinzu** (außer ggf. einer optionalen `get_for_range`-Methode pro C-Phase3-02 — das ist additiv und bricht die Service-Tier-Konvention nicht).
- `service/src/booking.rs:93` — `BookingService::create`-Signatur (heutige Form `Result<Booking, ServiceError>`). **Bleibt unverändert in Phase 3.**
- `service/src/booking.rs:99-107` — `BookingService::copy_week`-Signatur (heutige Form `Result<(), ServiceError>`). **Bleibt unverändert in Phase 3.**
- `service_impl/src/booking.rs:181-303` — aktuelle `create`-Implementation. **NICHT anfassen** — Konflikt-Logik lebt im neuen Schreib-Pfad oben.
- `service_impl/src/booking.rs:305-363` — aktuelle `copy_week`-Implementation. **NICHT anfassen.**

#### ShiftplanEditService-Erweiterung (Business-Logic — neuer Schreib-Pfad)

- `service/src/shiftplan_edit.rs` — Trait `ShiftplanEditService` (existing). **Neue Methoden** `book_slot_with_conflict_check` und `copy_week_with_conflict_check` (oder Plan-Phase-Naming) hinzufügen.
- `service_impl/src/shiftplan_edit.rs:26` — `gen_service_impl!`-Block. **Neue Deps** hinzufügen: `AbsenceService`, `SalesPersonUnavailableService` (zusätzlich zum bestehenden `BookingService`-Dep).
- `service_impl/src/shiftplan.rs:138` — `time::Date::from_iso_week_date`-Pattern als Vorlage für die Date-Konversion im neuen Schreib-Pfad.
- Date-Konversion + Lookup-Sequenz (siehe Domain-Block) lebt im neuen Methoden-Body.

#### AbsenceService-Erweiterung (Business-Logic — Forward-Warning)

- `service/src/absence.rs:154` — `create`-Signatur (heutige Form `Result<AbsencePeriod, ServiceError>`); wird zu `Result<AbsencePeriodCreateResult, ServiceError>`.
- `service/src/absence.rs:161` — `update`-Signatur (heutige Form `Result<AbsencePeriod, ServiceError>`); wird symmetrisch zu `Result<AbsencePeriodCreateResult, ServiceError>`.
- `service_impl/src/absence.rs:137` — aktuelle `create`-Implementation; HIER neue Schritte: BookingService::get_for_week-Loop für die Range, SalesPersonUnavailableService-Lookup, Warning-Konstruktion.
- `service_impl/src/absence.rs:204` — aktuelle `update`-Implementation; analoge Erweiterung wie create (auf NEUE Range, kein Diff-Modus).
- `service_impl/src/absence.rs:175,248` — bestehende `find_overlapping`-Calls (kategorie-scoped, für Self-Overlap-Check); NEU dazu kommt der cross-kategorie `find_overlapping_for_booking`-Call.

#### DAO-Erweiterung

- `dao/src/absence.rs:78` — `find_overlapping`-Trait-Methode (kategorie-scoped); NEUE Methode `find_overlapping_for_booking(sales_person_id, range, tx)` daneben.
- `dao_impl_sqlite/src/absence.rs` (Phase 1) — SQLx-Pattern; SQL-Template für IN-Clause oder Category-frei: `SELECT ... FROM absence_period WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL`.

#### ShiftplanViewService-Erweiterung (Business-Logic — Read-Markierung)

- `service/src/shiftplan.rs` — Trait `ShiftplanViewService` mit `get_shiftplan_week` / `get_shiftplan_day` (heute global). NEUE Methoden `get_shiftplan_week_for_sales_person` / `get_shiftplan_day_for_sales_person`.
- `service/src/shiftplan.rs:14-31` — `ShiftplanDay` und `ShiftplanBooking`-Strukturen; `ShiftplanDay` bekommt `unavailable: Option<UnavailabilityMarker>`-Feld.
- `service_impl/src/shiftplan.rs:24-108` — `build_shiftplan_day`-Helper; neuer Parallel-Helper `build_shiftplan_day_for_sales_person` mit zusätzlichen `absence_periods` + `manual_unavailables` Parametern.
- `service_impl/src/shiftplan.rs:110-120` — `gen_service_impl!` für `ShiftplanViewServiceImpl`; neue DI-Dependencies: `AbsenceService` + `SalesPersonUnavailableService` (für die per-sales-person-Methoden).
- `service_impl/src/shiftplan.rs:127-203` — `get_shiftplan_week`-Implementation; Vorlage für die per-sales-person-Variante.

#### REST-Layer

- `rest/src/booking.rs` — REST-Handler für `POST /booking`. **Bleibt unverändert in Phase 3.** Frontend bleibt funktional auf dem alten Endpunkt; Migration auf den neuen Endpunkt im Frontend-Workstream.
- `rest/src/absence.rs` — REST-Handler für POST/PATCH /absence-period; auf Wrapper-DTO umstellen (AbsenceService ist Business-Logic, Wrapper ist sein eigenes Result).
- `rest/src/shiftplan_edit.rs` (oder neue Datei `rest/src/shiftplan_edit_booking.rs` — Plan-Phase wählt) — **Neue Endpunkte** `POST /shiftplan-edit/booking` und `POST /shiftplan-edit/copy-week` (Naming Plan-Phase). Wrapper-DTO im Body.
- `rest/src/shiftplan.rs` — neue Endpoints `GET /shiftplan/{id}/year/{y}/week/{w}/sales-person/{sp}` und day-Aggregat-Variante (per-sales-person-View, Read-Pfad).
- `rest-types/src/lib.rs` — neue inline DTOs `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `WarningTO`, `UnavailabilityMarkerTO` (Repo-Konvention: alle DTOs inline, siehe Phase 1 Override).
- `rest/src/lib.rs:120` — `error_handler`-Wrapper bleibt unverändert (mappt nur ServiceError; Warnings sind Erfolgs-Pfad).
- `rest/src/lib.rs` — ApiDoc-Erweiterung für neue Wrapper-DTOs + neue Endpunkte (utoipa-Schema).

#### SalesPersonUnavailableService (read-only Konsumenten)

- `service/src/sales_person_unavailable.rs:67-74` — `get_by_week_for_sales_person(sales_person_id, year, calendar_week, ctx, tx)` — direkt nutzbar von `ShiftplanEditService::book_slot_with_conflict_check` für die Reverse-Warning.
- `service/src/sales_person_unavailable.rs:61-66` — `get_all_for_sales_person(sales_person_id, ctx, tx)` — nutzbar für `AbsenceService::create` (filtert clientside auf Range; oder Plan-Phase führt eine Range-Methode ein).
- `service_impl/src/sales_person_unavailable.rs` — bestehende Implementation; Phase 3 touchiert diese NICHT (deferred per D-Phase3-17).

#### Permission/Validation

- `service_impl/src/permission.rs` — `HR_PRIVILEGE`, `SHIFTPLANNER_PRIVILEGE`, `SALES_PRIVILEGE`-Konstanten; Permission-Konvention für die neuen per-sales-person-Methoden + die neuen `ShiftplanEditService`-Methoden.
- `service/src/sales_person.rs:124-129` — `verify_user_is_sales_person`-Trait-Methode (für Permission-Gate auf `get_shiftplan_*_for_sales_person` und auf die neuen `book_slot_with_conflict_check`-Methoden).
- `service/src/lib.rs:121-128` — `ServiceError`-Surface; KEINE neuen Varianten erwartet (Warnings sind Erfolgs-Pfad).
- `service/src/lib.rs:108` — `ServiceError::TimeComponentRangeError(#[from] time::error::ComponentRange)` — bestehender Fehler-Pfad für `time::Date::from_iso_week_date` (siehe RESEARCH.md #2).

#### DI-Verdrahtung

- `shifty_bin/src/main.rs` — Konstruktionsreihenfolge gemäß Service-Tier-Konvention: erst alle Basic-Services (`BookingService`, `SalesPersonUnavailableService`, `SlotService`, `SalesPersonService`, ...), dann die Business-Logic-Schicht (`AbsenceService` mit `BookingService`+`SalesPersonUnavailableService`-Deps; `ShiftplanEditService` mit `BookingService`+`AbsenceService`+`SalesPersonUnavailableService`-Deps; `ShiftplanViewService` mit denselben drei). **Kein Cycle, keine `OnceLock`-Tricks.** Plan-Phase verifiziert die Reihenfolge in der Datei.

#### Testing

- `service_impl/src/test/booking.rs` (existing) — Service-Tests für `BookingService::create`. **Bleiben grün ohne Änderung** — `BookingService` wird in Phase 3 nicht angefasst.
- `service_impl/src/test/shiftplan_edit/` (oder analog — Plan-Phase wählt) — Service-Tests für die neuen `book_slot_with_conflict_check` / `copy_week_with_conflict_check`-Methoden mit Mock-`AbsenceService`, Mock-`SalesPersonUnavailableService`, Mock-`BookingService`.
- `service_impl/src/test/absence/` (Phase 1) — Vorlage für Service-Tests-Struktur; neue Tests für Forward-Warning + Doppel-Quelle in `AbsenceService::create/update`.
- `shifty_bin/src/integration_test/absence_period.rs` (Phase 1) — Vorlage für Integration-Test-Setup; neue Datei `booking_absence_conflict.rs` für Cross-Source-Tests gegen den NEUEN konflikt-aware-Endpunkt.
- `service_impl/src/test/extra_hours.rs` — `_forbidden`-Test-Pattern (Mock-Setup mit `expect_check_permission` + `expect_verify_user_is_sales_person`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`AbsenceService` aus Phase 1** (`service/src/absence.rs`) — Trait + DI vorhanden; bekommt zwei neue Methoden (`find_overlapping_for_booking`) und Signatur-Bruch auf `*CreateResult`.
- **`AbsenceDao::find_overlapping`** (`dao/src/absence.rs:78`) — kategorie-scoped, bestehend; NEU `find_overlapping_for_booking` daneben mit cross-kategorie-Lookup.
- **`SalesPersonUnavailableService::get_by_week_for_sales_person`** (`service/src/sales_person_unavailable.rs:67-74`) — direkt nutzbar für die Reverse-Warning beim NEUEN Booking-Schreib-Pfad. Keine API-Änderung nötig.
- **`time::Date::from_iso_week_date`** — Standard-time-API, bereits in `service_impl/src/shiftplan.rs:138` benutzt; `(year, week, weekday) -> time::Date`. Konvertiert einen Booking-Tag zu einem konkreten Datum für den AbsencePeriod-Lookup.
- **`shifty_utils::DateRange`** (Phase 1, `shifty-utils/src/date_range.rs`) — `iter_days`, `day_count`, `overlaps`, `contains` schon verfügbar; nutzbar für die Range-basierten Lookups in Phase 3.
- **`gen_service_impl!`-Macro** — DI-Pattern direkt für die erweiterten ServiceDeps wiederverwendbar; neue Dependencies werden mechanisch hinzugefügt.
- **`tokio::join!` für Permission-Checks** (`service_impl/src/extra_hours.rs:236-245`-Pattern) — direkt für `HR ∨ verify_user_is_sales_person` in den per-sales-person-Methoden + im neuen Schreib-Pfad oben.
- **`ShiftplanEditService` (existing)** (`service/src/shiftplan_edit.rs` + `service_impl/src/shiftplan_edit.rs`) — hält heute schon `BookingService` als Dep. **Natürlicher Andock-Punkt** für die neuen `book_slot_with_conflict_check` / `copy_week_with_conflict_check`-Methoden.
- **`build_shiftplan_day`-Helper** (`service_impl/src/shiftplan.rs:24-108`) — als Vorlage für `build_shiftplan_day_for_sales_person`; gleiche Slot-Filter-Logik (Holiday, ShortDay-Cutoff), neue Parameter für absence + manual_unavailable.
- **REST `error_handler`-Wrapper** (`rest/src/lib.rs:120`) — bleibt unverändert; Warnings sind im Erfolgs-Body, nicht im ServiceError.
- **`#[automock]` auf Service-Traits** — Mock-`AbsenceService`, Mock-`SalesPersonUnavailableService`, Mock-`BookingService` sind für die Service-Tests sofort verfügbar (Pattern aus Phase 1/2 etabliert).

### Established Patterns

- **Layered Architecture:** REST → Service-Trait → DAO-Trait → SQLx. Phase 3 erweitert auf 3 Schichten konsistent.
- **Service-Tier-Konvention (NEU 2026-05-02):** Basic Services (Entity-Manager) konsumieren nur DAOs/Permission/Tx; Business-Logic Services kombinieren Aggregate. Cross-Entity-Logik lebt im Business-Logic-Tier. Siehe `shifty-backend/CLAUDE.md`.
- **Soft-Delete-Konvention:** `WHERE deleted IS NULL` in jedem Read-SQL — gilt auch für `find_overlapping_for_booking` (Pitfall-6-Test verifiziert).
- **Wrapper-DTO-Inline-Konvention:** alle DTOs in `rest-types/src/lib.rs` inline (Phase 1 Override aus 01-VALIDATION.md). `BookingCreateResultTO`, `AbsencePeriodCreateResultTO` etc. folgen dem.
- **Permission-Pattern:** `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(sales_person_id))` + `.or()`. Direkt für die per-sales-person-Methoden und für den neuen Schreib-Pfad oben.
- **`gen_service_impl!`-DI-Erweiterung:** mechanisches Hinzufügen neuer Service-Dependencies; Plan-Phase prüft DI-Reihenfolge in `shifty_bin/src/main.rs`: erst Basic, dann Business-Logic.
- **Architektur-Direction (STATE.md + CLAUDE.md):** `AbsenceService → BookingService` (Business-Logic ↑ konsumiert Basic ↓); `ShiftplanEditService → BookingService + AbsenceService + SalesPersonUnavailableService`; `ShiftplanViewService` analog. **Kein zyklischer Service-Cycle.**

### Integration Points

- **`service/src/shiftplan_edit.rs` + `service_impl/src/shiftplan_edit.rs`** — Trait + Impl bekommt neue Methoden + zwei neue Deps. **Hauptzentrum von Phase 3** für den Reverse-Warning-Pfad.
- **`service/src/absence.rs:154,161`** — `AbsenceService::create` und `update` Signatur-Brüche. Alle Call-Sites müssen mit (Compiler-Druck — gewünscht).
- **`service/src/shiftplan.rs:14-69`** — `ShiftplanViewService` Trait + `ShiftplanDay`-Struct. Neue Methoden + neues Feld `unavailable`.
- **`dao/src/absence.rs:78`** — DAO-Trait neue Methode `find_overlapping_for_booking`.
- **`dao_impl_sqlite/src/absence.rs`** (Phase 1) — neue SQL-Query (kein Migration nötig — bestehender Index `(sales_person_id, from_date) WHERE deleted IS NULL` aus Phase 1 D-04 reicht).
- **`shifty_bin/src/main.rs`** — DI-Erweiterung in der Reihenfolge: Basic → Business-Logic. `AbsenceService` bekommt `BookingService` + `SalesPersonUnavailableService` als Deps; `ShiftplanEditService` bekommt `BookingService` + `AbsenceService` + `SalesPersonUnavailableService`; `ShiftplanViewService` analog. **`BookingService` bekommt KEINE neuen Service-Deps.**
- **`rest/src/absence.rs` + `rest/src/shiftplan.rs` + neuer Handler im REST-Bereich des `ShiftplanEditService`** — REST-Handler-Migration auf Wrapper-DTOs (nur AbsencePeriod + neue Endpunkte); neuer per-sales-person-Endpoint; ApiDoc-Erweiterung. **`rest/src/booking.rs` bleibt unverändert.**
- **`rest-types/src/lib.rs`** — inline DTOs: `WarningTO` (Tag-Enum), `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `UnavailabilityMarkerTO`.

### Risiken / Pitfalls für Phase 3

- **Pitfall 6 (Soft-Delete + Warnings, SC4):** soft-deleted `AbsencePeriod`s dürfen KEINE Warnung und KEINE Markierung triggern. DAO-Konvention `WHERE deleted IS NULL` ist Pflicht für `find_overlapping_for_booking`. Test: ein soft-deleted AbsencePeriod im Range → BookingCreateResult.warnings ist leer; ShiftplanDay.unavailable ist None.
- **Service-Tier-Drift:** Versuchung, "schnell mal" eine neue Service-Dep auf `BookingService` zu hängen. Plan-Phase muss aktiv prüfen, dass `BookingServiceDeps` in Phase 3 KEINE neuen Domain-Service-Deps bekommt — Forward-Warning ist im AbsenceService, Reverse-Warning ist im neuen Schreib-Pfad oben. Verstoß bricht die in CLAUDE.md etablierte Konvention.
- **Date-Konversion-Pitfall:** `time::Date::from_iso_week_date` kann fehlschlagen (ungültige Week-Nummer). Lebt im neuen Schreib-Pfad oben. Default: `?`-Operator → `ServiceError::TimeComponentRangeError` (existing); REST-Layer mapped auf 500 (Catch-All), das ist akzeptabel als Defense-in-Depth da Validation oben den häufigsten Fall fängt.
- **Doppel-Quelle De-Dup:** Bei der Anzeige (UnavailabilityMarker) muss klar sein, welche Quelle priorisiert wird, wenn beide vorliegen. Entscheidung: `Both` als eigene Variante, trägt absence-id (mehr semantischer Inhalt). Manuelle Unavailable hat keine semantischen Felder außer der ID.
- **`copy_week_with_conflict_check`-Performance:** Bei 50 Bookings = 50 `find_overlapping_for_booking`-Calls + 50 `get_by_week_for_sales_person`-Calls. Default OK, aber Plan-Phase darf eine Pre-fetch-Optimierung einbauen wenn Tests Druck zeigen (C-Phase3-06).
- **Forward-Warning-Performance bei langen Absences:** Eine 60-Tage-Absence (z.B. Sabbatical) ⇒ Loop über 60 Tage × N Bookings/Tag. Plan-Phase darf entscheiden, ob `BookingService::get_for_range(sales_person_id, range)` als neue Methode eingeführt wird (C-Phase3-02). Default: per-Woche-Loop. **Hinweis:** Eine neue Methode auf `BookingService` ist erlaubt — sie ist Read-Surface auf dem eigenen Aggregat und bricht die Service-Tier-Konvention nicht.
- **Frontend-Migration:** Wrapper-DTO-Bruch betrifft NUR `POST /absence-period` und `PATCH /absence-period/{id}`. `POST /booking` bleibt unverändert — die konflikt-aware-Variante ist ein NEUER Endpunkt. Frontend-Workstream zieht nach. Aus Backend-Sicht ist die Trennung sauber (alter Endpunkt = pure CRUD; neuer Endpunkt = mit Warnings).
- **REST-Handler-Doppelung:** Der neue konflikt-aware-Endpunkt ruft intern `BookingService::create` über den `ShiftplanEditService`. Plan-Phase muss sicherstellen, dass beide Endpunkte (`POST /booking` und `POST /shiftplan-edit/booking`) konsistente Permission-Gates haben — die alte `BookingService::create`-Permission läuft sowieso, der neue Endpunkt fügt das HR ∨ self-Pattern hinzu (analog `ShiftplanViewService`).

</code_context>

<specifics>
## Specific Ideas

- **`Warning`-Enum als Datenmodell-Beispiel:**
  ```rust
  // service/src/warning.rs (oder inline in service/src/lib.rs)
  pub enum Warning {
      BookingOnAbsenceDay {
          booking_id: Uuid,
          date: time::Date,
          absence_id: Uuid,
          category: AbsenceCategory,
      },
      BookingOnUnavailableDay {
          booking_id: Uuid,
          year: u32,
          week: u8,
          day_of_week: DayOfWeek,
      },
      AbsenceOverlapsBooking {
          absence_id: Uuid,
          booking_id: Uuid,
          date: time::Date,
      },
      AbsenceOverlapsManualUnavailable {
          absence_id: Uuid,
          unavailable_id: Uuid,
      },
  }
  ```
- **Wrapper-Structs:**
  ```rust
  // service/src/absence.rs (Business-Logic Service)
  pub struct AbsencePeriodCreateResult {
      pub absence: AbsencePeriod,
      pub warnings: Arc<[Warning]>,
  }

  // service/src/shiftplan_edit.rs (Business-Logic Service, NEU für Phase 3)
  pub struct BookingCreateResult {
      pub booking: Booking,
      pub warnings: Arc<[Warning]>,
  }

  pub struct CopyWeekResult {
      pub copied_bookings: Arc<[Booking]>,
      pub warnings: Arc<[Warning]>,
  }
  ```
- **`UnavailabilityMarker` für ShiftplanDay:**
  ```rust
  pub enum UnavailabilityMarker {
      AbsencePeriod { absence_id: Uuid, category: AbsenceCategory },
      ManualUnavailable,
      Both { absence_id: Uuid, category: AbsenceCategory },
  }
  ```
- **REST-DTO mit utoipa-Tag-Enum (Vorgabe für `WarningTO`):**
  ```rust
  #[derive(Serialize, Deserialize, ToSchema)]
  #[serde(tag = "kind", content = "data", rename_all = "snake_case")]
  pub enum WarningTO {
      BookingOnAbsenceDay {
          booking_id: Uuid,
          date: time::Date,
          absence_id: Uuid,
          category: AbsenceCategoryTO,
      },
      BookingOnUnavailableDay { booking_id: Uuid, year: u32, week: u8, day_of_week: DayOfWeekTO },
      AbsenceOverlapsBooking { absence_id: Uuid, booking_id: Uuid, date: time::Date },
      AbsenceOverlapsManualUnavailable { absence_id: Uuid, unavailable_id: Uuid },
  }
  ```
- **Skizze `ShiftplanEditService::book_slot_with_conflict_check` (Plan-Phase darf finalisieren):**
  ```rust
  async fn book_slot_with_conflict_check(
      &self,
      booking: Booking,
      ctx: Authentication<Self::Context>,
      tx: Option<Self::Transaction>,
  ) -> Result<BookingCreateResult, ServiceError> {
      let tx = self.transaction_dao.use_transaction(tx).await?;
      // Permission: HR ∨ self
      tokio::try_join!(
          self.permission_service.check_permission(HR_PRIVILEGE, ctx.clone()),
          self.sales_person_service.verify_user_is_sales_person(booking.sales_person_id, ctx.clone())
      )?; // pseudo — actual pattern: .or()-chain
      // Slot lookup (delegate to BookingService or repeat)
      let slot = self.slot_service.get_slot(booking.slot_id, ctx.clone(), Some(tx.clone())).await?;
      let booking_date = time::Date::from_iso_week_date(booking.year as i32, booking.calendar_week as u8, slot.day_of_week.into())?;
      let single_day_range = DateRange::new(booking_date, booking_date)?;
      let mut warnings: Vec<Warning> = Vec::new();
      // Forward-from-Absence-Side
      let absences = self.absence_service.find_overlapping_for_booking(booking.sales_person_id, single_day_range, ctx.clone(), Some(tx.clone())).await?;
      for absence in absences.iter() {
          warnings.push(Warning::BookingOnAbsenceDay {
              booking_id: booking.id, // not yet assigned — pull from create-result instead
              date: booking_date,
              absence_id: absence.id,
              category: absence.category,
          });
      }
      // Manual-Unavailable-Side
      let unavailables = self.sales_person_unavailable_service.get_by_week_for_sales_person(booking.sales_person_id, booking.year, booking.calendar_week, ctx.clone(), Some(tx.clone())).await?;
      if unavailables.iter().any(|u| u.day_of_week == slot.day_of_week) {
          warnings.push(Warning::BookingOnUnavailableDay {
              booking_id: booking.id,
              year: booking.year,
              week: booking.calendar_week,
              day_of_week: slot.day_of_week,
          });
      }
      // Persist via Basic-Service
      let persisted = self.booking_service.create(booking, ctx, Some(tx.clone())).await?;
      // Patch warning booking_ids to the persisted ID
      let warnings: Arc<[Warning]> = warnings.into_iter().map(|w| patch_booking_id(w, persisted.id)).collect::<Vec<_>>().into();
      self.transaction_dao.commit(tx).await?;
      Ok(BookingCreateResult { booking: persisted, warnings })
  }
  ```
  *Hinweis:* der booking_id-Patch ist Feinarbeit — Plan-Phase darf entscheiden, ob die Warning-Konstruktion VOR oder NACH dem `create` läuft. Vorgabe: NACH (so steht die echte ID drin).
- **Test-Fixture-Idee für Cross-Source-Test (Plan-Phase darf finalisieren):**
  - 1 Sales-Person `sp1`, 1 Slot Mo (`day_of_week = Monday`), Year 2026 / KW 18.
  - 1 AbsencePeriod `Vacation, 2026-04-27 .. 2026-04-30` (Mo-Do).
  - 1 SalesPersonUnavailable `(2026, 18, Monday)`.
  - `book_slot_with_conflict_check`-Aufruf für `sp1` auf Slot Mo, KW 18, 2026 → BookingCreateResult mit 2 Warnings: `BookingOnAbsenceDay { date: 2026-04-27, absence_id, category: Vacation }` + `BookingOnUnavailableDay { ... }`. Booking trotzdem persistiert.
  - ShiftplanDay-Aufruf für `sp1`, Mo KW 18 → `unavailable: Some(UnavailabilityMarker::Both { absence_id, category: Vacation })`.
- **Pitfall-6-Test (verbatim für Plan-Phase):**
  ```rust
  // GIVEN: AbsencePeriod for sp1 on 2026-04-27 .. 2026-04-30, dann soft-deleted
  // WHEN: ShiftplanEditService::book_slot_with_conflict_check für sp1 auf Slot Mo KW 18 2026
  // THEN: BookingCreateResult.warnings ist EMPTY (KEIN BookingOnAbsenceDay)
  // UND: ShiftplanDay.unavailable ist None (KEIN AbsencePeriod-Marker)
  // UND: Klassisches POST /booking funktioniert weiter unverändert (Regression-Schutz)
  ```
- **DAO-SQL-Template für `find_overlapping_for_booking`:**
  ```sql
  SELECT id, logical_id, sales_person_id, category, from_date, to_date,
         description, created, deleted, version
  FROM absence_period
  WHERE sales_person_id = ?
    AND from_date <= ?  -- range.to
    AND to_date >= ?    -- range.from
    AND deleted IS NULL
  ORDER BY from_date
  ```
  Single roundtrip; nutzt composite index aus Phase 1 D-04.

</specifics>

<deferred>
## Deferred Ideas

- **`SalesPersonUnavailableService::create` symmetrisieren** (Warning bei Anlegen einer manuellen Unavailable auf einem AbsencePeriod-Tag) — Folgephase, nicht Phase 3. Warning-Variante `ManualUnavailableOnAbsenceDay` wird in Phase 3 NICHT eingeführt; die Folgephase fügt sie zusammen mit dem neuen Wrapper-Type hinzu.
- **Auto-Cleanup von überlappenden `sales_person_unavailable`-Einträgen beim Anlegen einer AbsencePeriod** — bewusst NICHT in Phase 3 (irreversibel; bricht Architektur-Direction; bricht Phase-4-Re-Run-Idempotenz). User entscheidet selbst.
- **Multi-Wochen-Range-Variante `get_shiftplan_for_sales_person(range)`** — Backend bleibt per-Woche; Frontend loopt. Future-Phase falls UI das fordert.
- **Kategorie-Trigger-Differenzierung** (z.B. UnpaidLeave nur als "informativ" statt Warning) — Default für Phase 3: alle 3 Kategorien gleich. Plan-Phase darf bei UX-Argument eingreifen.
- **Performance-Caching für `copy_week_with_conflict_check` (Pre-fetch-Range-Query statt N Calls)** — erst messen, dann optimieren. Plan-Phase entscheidet basierend auf Test-Performance.
- **Eigene Range-Methode `BookingService::get_for_range(sales_person_id, range)`** — Plan-Phase darf einführen falls Performance bei langen Absences es nahelegt; Default: Loop über Kalenderwochen mit `get_for_week`. **Hinweis:** Read-Methode auf dem eigenen Aggregat ist Service-Tier-konform.
- **Frontend-Migration vom alten `POST /booking` zum neuen konflikt-aware-Endpunkt** — Frontend-Workstream, nicht Phase 3. Phase 3 dokumentiert beide Endpunkte im OpenAPI-Snapshot.
- **REST-Deprecation des alten `POST /booking`-Endpunkts** — bewusst deferred. Phase 3 fügt nur hinzu. Eine spätere Phase entscheidet, ob der alte Endpunkt deprecated wird oder dauerhaft als Basic-Service-Surface bleibt.
- **REST-Deprecation alter Endpunkte im Reporting-Bereich** — Phase 4 / MIG-05 (im Zusammenhang mit ExtraHours-Migration).
- **Phase-4-Cutover-Gate (MIG-02/MIG-03)** — Phase 4. Phase 3 nutzt keinen Reporting-Pfad; ist flag-unabhängig.
- **Carryover-Refresh** — Phase 4 (MIG-04).
- **Phase-1-Hygiene-Drift** (lokale `localdb.sqlite3`-Migration aus Phase-2-deferred-items.md) — Phase 4 wird das nachreichen; Phase 3 ist davon nicht betroffen, weil Phase-3-Tests in Mock + In-Memory-SQLite laufen.

</deferred>

---

*Phase: 3-Booking-Shift-Plan-Konflikt-Integration*
*Context gathered: 2026-05-02*
*Re-Discuss applied: 2026-05-02 — Service-Tier-Konvention etabliert; BookingService bleibt strikt basic; Reverse-Warning + Wrapper-Result wandert in den Business-Logic-Tier (`ShiftplanEditService`); kein Cycle mehr in der Service-Hierarchie.*
