# Phase 3: Booking & Shift-Plan Konflikt-Integration - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Bookings und Shift-Plan-Anzeige werden konflikt-aware bezüglich `AbsencePeriod` (Phase 1). Drei Wirkungs-Pfade:

1. **Forward-Warning (BOOK-01):** Beim Anlegen einer `AbsencePeriod`, die ein bestehendes Booking überlappt, gibt `AbsenceService::create/update` einen Wrapper mit `Vec<Warning>` zurück (mit konkreten Booking-IDs und Daten). Die Absence wird trotzdem persistiert; kein Auto-Löschen, kein Block.
2. **Reverse-Warning (BOOK-02):** Beim Anlegen eines Bookings auf einem Tag, der entweder durch `AbsencePeriod` oder durch `sales_person_unavailable` als nicht verfügbar markiert ist, gibt `BookingService::create` eine Warnung zurück. Booking wird trotzdem angelegt.
3. **Shift-Plan-Markierung (PLAN-01):** Eine neue per-sales-person-Variante des `ShiftplanViewService` markiert pro Tag, ob er durch `AbsencePeriod` oder `sales_person_unavailable` (oder beides) abgedeckt ist. Die bisherige Doppel-Eintragung (ExtraHours + sales_person_unavailable) entfällt für die zeitraum-basierten Kategorien.

**Architektur-Direction (aus STATE.md, Phase-1-D-08, Phase-3-Discuss):**
- `BookingService → AbsenceService` (nie umgekehrt). `AbsenceService` darf keine Booking-Lookups machen.
- `BookingService` darf direkt `AbsenceService` UND `SalesPersonUnavailableService` als Dependencies haben — die Quellen-Aggregation lebt im Booking-Service, nicht in einer eigenen Abstraktions-Schicht.
- Soft-deleted `AbsencePeriod`s triggern keine Warnung, keine Markierung (Pitfall 6 / SC4).

**In Scope (Phase 3):**
- Neue DAO-Methode `AbsenceDao::find_overlapping_for_booking(sales_person_id, range, tx) -> Arc<[AbsencePeriodEntity]>` (cross-kategorie, single SQL-Query mit IN-Clause oder ohne Category-Filter, nutzt bestehenden composite index `(sales_person_id, from_date) WHERE deleted IS NULL`).
- Neue Service-Methode `AbsenceService::find_overlapping_for_booking(sales_person_id, range, ctx, tx)` (HR ∨ self).
- `enum Warning { BookingOnAbsenceDay { booking_id, date, absence_id, category }, BookingOnUnavailableDay { booking_id, year, week, day_of_week }, AbsenceOverlapsBooking { absence_id, booking_id, date }, AbsenceOverlapsManualUnavailable { absence_id, unavailable_id } }` in `service/src/lib.rs` (oder eigenem Modul, Plan-Phase entscheidet).
- `BookingService::create` Signatur-Bruch auf `Result<BookingCreateResult, ServiceError>` mit `struct BookingCreateResult { booking: Booking, warnings: Arc<[Warning]> }`.
- `BookingService::copy_week` Signatur-Bruch auf `Result<CopyWeekResult, ServiceError>` mit `struct CopyWeekResult { copied_bookings: Arc<[Booking]>, warnings: Arc<[Warning]> }` — aggregiert Warnings über alle inneren `create`-Calls.
- `AbsenceService::create` Signatur-Bruch auf `Result<AbsencePeriodCreateResult, ServiceError>` mit `struct AbsencePeriodCreateResult { absence: AbsencePeriod, warnings: Arc<[Warning]> }`.
- `AbsenceService::update` Signatur-Bruch auf `Result<AbsencePeriodCreateResult, _>` symmetrisch zu `create` — liefert volle Warning-Liste über die NEUE Range (kein Diff-Modus).
- `BookingService` bekommt `AbsenceService` + `SalesPersonUnavailableService` als neue Dependencies; `BookingService::create` führt zusätzlich zur bisherigen Validation:
  - Date-Konversion `time::Date::from_iso_week_date(year, calendar_week, slot.day_of_week)` (Slot wird ohnehin geladen, Pattern existiert in `shiftplan.rs:138`).
  - `absence_service.find_overlapping_for_booking(sales_person_id, single_day_range, ...)` — produziert `Warning::BookingOnAbsenceDay` für jeden Treffer (eine Warning pro betroffenem Booking-Tag).
  - `sales_person_unavailable_service.get_by_week_for_sales_person(sales_person_id, year, week, ...)` — produziert `Warning::BookingOnUnavailableDay` wenn `(day_of_week)` matcht.
- `AbsenceService::create` (und `update`) führt zusätzlich:
  - Booking-Lookup via `BookingService::get_for_week` (oder neuer Methode für Range — Plan-Phase darf wählen, falls Performance es nahelegt) — produziert `Warning::AbsenceOverlapsBooking` pro betroffenem Booking-Tag.
  - `sales_person_unavailable_service.get_all_for_sales_person` (oder Range-Variante) — produziert `Warning::AbsenceOverlapsManualUnavailable` pro überlappendem manuellen Eintrag. **Kein Auto-Cleanup** — User entscheidet selbst, ob er die alten Unavailables löscht.
- `AbsenceService` bekommt `BookingService` + `SalesPersonUnavailableService` als neue Dependencies (Direction `Absence → Booking` ist erlaubt für reine Lese-Zwecke der Konflikt-Detektion; nur das **schreibende** Coupling `Booking → Absence` ist verboten — das bedeutet: AbsenceService liest Bookings, aber kein Booking-Service-Schreibpfad kommt zurück nach AbsenceService).
- Neue Methode `ShiftplanViewService::get_shiftplan_week_for_sales_person(shiftplan_id, year, week, sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>` und symmetrisch `get_shiftplan_day_for_sales_person`.
- `ShiftplanDay` bekommt neues Feld `unavailable: Option<UnavailabilityMarker>` mit `enum UnavailabilityMarker { AbsencePeriod { absence_id, category }, ManualUnavailable, Both { absence_id, category } }`. Slots bleiben sichtbar — Frontend rendert per-Tag-Badge zusätzlich.
- Permission für die per-sales-person-Methode: HR ∨ `verify_user_is_sales_person(sales_person_id)` (Phase-1-D-09-Pattern).
- Neuer REST-Endpoint `GET /shiftplan/{shiftplan_id}/year/{year}/week/{week}/sales-person/{sales_person_id}` und entsprechend für day-Aggregat. ApiDoc + utoipa-Annotation Pflicht.
- REST-Wrapper-DTOs: `BookingCreateResultTO { booking: BookingTO, warnings: Vec<WarningTO> }`, `AbsencePeriodCreateResultTO { absence: AbsencePeriodTO, warnings: Vec<WarningTO> }`, `CopyWeekResultTO { copied_bookings: Vec<BookingTO>, warnings: Vec<WarningTO> }` — alle inline in `rest-types/src/lib.rs` (Repo-Konvention aus Phase 1 Override). `WarningTO` als Tag-Enum, utoipa-konform.
- Status: REST-Endpoints geben **immer** 200/201 (auch mit Warnings); Status-Code unterscheidet nicht zwischen "clean" und "mit Warnings".
- Pflicht-Tests:
  - `_forbidden`-Tests pro neue public service method (Permission-Pattern aus Phase 1 D-11).
  - Pitfall-6-Test: soft-deleted `AbsencePeriod` triggert KEINE Warnung und KEINE Markierung (SC4) — sowohl im Service-Test (Mock mit gelöschter Row) als auch im Integration-Test (`shifty_bin/src/integration_test/`).
  - Symmetrische Forward + Reverse Warning-Tests mit konkreten Fixture-IDs.
  - `copy_week`-Aggregations-Test: 3 Quell-Bookings, davon 2 auf Absence-Tagen → CopyWeekResult enthält 2 Warnings + 3 kopierte Bookings.
  - Cross-Source-Test: ein Booking-Tag, der durch BEIDE (AbsencePeriod + sales_person_unavailable) markiert ist → ZWEI Warnings (eine pro Quelle), Booking wird trotzdem angelegt.
  - ShiftplanDay-Marker-Test: `UnavailabilityMarker::Both` wird nur dann gesetzt, wenn an dem Tag wirklich beide Quellen treffen.

**Strikt nicht in Scope (Phase 3):**
- `SalesPersonUnavailableService::create` bleibt unverändert — keine Warning beim Anlegen einer manuellen Unavailable auf einem Absence-Tag (deferred, Folgephase). Das `Warning::ManualUnavailableOnAbsenceDay`-Pendant wird in Phase 3 NICHT als Enum-Variante eingeführt.
- Migration aus `ExtraHours` (Phase 4 / MIG-01).
- Validierungs-Gate / Cutover (Phase 4 / MIG-02..04).
- Atomares Feature-Flag-Flippen (Phase 4).
- Reporting-Kategorie-Trigger-Differenzierung (z.B. UnpaidLeave nur als Hinweis statt Warning) — Plan-Phase darf das aufgreifen, aber Default ist: alle 3 Kategorien (Vacation/Sick/UnpaidLeave) triggern gleichermaßen Warnings.
- Multi-Wochen-Range-Variante `get_shiftplan_for_sales_person(range)` — Frontend loopt mehrere Wochen wenn nötig; Backend bleibt per-Woche.
- Frontend (Dioxus) — separater Workstream. Backward-Compat des alten Frontends ist nicht Gegenstand dieser Phase (Wrapper-DTO-Bruch ist erlaubt).
- REST-Deprecation alter Endpunkte (Phase 4 / MIG-05).

</domain>

<decisions>
## Implementation Decisions

### Warning API-Migration (Area A)

- **D-Phase3-01:** **Signatur-Bruch in beiden Schreib-Services.** `BookingService::create -> Result<BookingCreateResult, ServiceError>` und `AbsenceService::create -> Result<AbsencePeriodCreateResult, ServiceError>` mit Wrapper-Struct `{ entity, warnings: Arc<[Warning]> }`. Begründung: Compiler erzwingt, dass alle Aufrufer (Tests, REST-Handler, copy_week, integration-tests) die Warnings sehen — keine zwei parallelen Pfade, keine stille Datenverluste. Phase 1 D-08 hatte explizit "API-Bruch oder additiver Wrapper?" auf Phase 3 verschoben; Antwort: Bruch.
- **D-Phase3-02:** **`copy_week` aggregiert Warnings.** `BookingService::copy_week -> Result<CopyWeekResult, ServiceError>` mit `struct CopyWeekResult { copied_bookings: Arc<[Booking]>, warnings: Arc<[Warning]> }`. Alle Warnings aus der inneren `create`-Schleife werden gesammelt; keine wird abgebrochen, keine swallowed. Konsistent mit BookingCreateResult.
- **D-Phase3-03:** **REST-Wrapper-DTO im Body, immer 200/201.** `POST /booking` und `POST /absence-period` (und `PATCH /absence-period/{id}`, und `POST /booking/copy-week`) geben Wrapper-DTOs zurück: `BookingCreateResultTO { booking: BookingTO, warnings: Vec<WarningTO> }`, analog für die anderen. Status-Code unterscheidet nicht zwischen "clean" und "mit Warnings" — Frontend prüft `warnings.is_empty()`. utoipa-Schema dokumentiert den Bruch direkt; OpenAPI-Snapshot zeigt die Veränderung.
- **D-Phase3-04:** **`AbsenceService::update` symmetrisch zu `create`.** `update -> Result<AbsencePeriodCreateResult, _>` (NICHT `Result<AbsencePeriod, _>`). Warnings werden für ALLE Tage in der NEUEN Range berechnet (kein Diff-Modus). Begründung: einfache Mental-Model ("wenn die Range sich ändert, kommen die Warnings raus, die du hättest, wenn du die Absence gerade neu angelegt hättest"); Diff-Logik wäre fehlerträchtig und bringt User keinen Vorteil ("ich sehe nur den Diff" wäre eine Frontend-Aufgabe, kein Backend-Feature).

### Cross-Source-Unavailability Lookup (Area B)

- **D-Phase3-05:** **Neue DAO-Methode mit IN-Clause.** `AbsenceDao::find_overlapping_for_booking(sales_person_id: Uuid, range: DateRange, tx: Self::Transaction) -> Result<Arc<[AbsencePeriodEntity]>, DaoError>` als single SQL-Query mit `WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL` (KEIN Category-Filter — alle 3 Kategorien werden zurückgegeben; Service-Layer entscheidet später, wie zu behandeln). Nutzt den bestehenden composite index `(sales_person_id, from_date) WHERE deleted IS NULL` aus Phase 1 D-04. Single roundtrip auch bei copy_week-Loops; Performance-skalierbar.
- **D-Phase3-06:** **`BookingService` kombiniert beide Quellen direkt.** `BookingService` bekommt `AbsenceService` + `SalesPersonUnavailableService` als neue DI-Dependencies; ruft beide separat im `create`-Pfad und konstruiert `Warning`-Varianten pro Quelle. Architektur-Direction `BookingService → AbsenceService` (Research/STATE.md) bleibt gewahrt; `AbsenceService` lernt nichts über `sales_person_unavailable`. KEIN neuer `UnavailabilityService` als Aggregator (vermeidet Indirektion für nur 2 Quellen).
- **D-Phase3-07:** **Date-Konversion inline per `time::Date::from_iso_week_date`.** Im `BookingService::create`-Body, direkt nach dem bestehenden Slot-Lookup (Zeile 260-263 in `service_impl/src/booking.rs`): `let booking_date = time::Date::from_iso_week_date(booking.year as i32, booking.calendar_week as u8, slot.day_of_week.into())?;` (oder analog mit der vorhandenen `DayOfWeek -> time::Weekday`-Konversion). Pattern existiert bereits in `service_impl/src/shiftplan.rs:138`. Keine neue Service-Surface, kein extra Roundtrip.
- **D-Phase3-08:** **`AbsenceService::create/update` lädt Bookings per `BookingService`.** Direction `AbsenceService → BookingService` ist NUR für Lese-Lookups erlaubt (Konflikt-Detektion); kein Schreibpfad zurück. Plan-Phase entscheidet, ob `BookingService::get_for_week` mehrfach für mehrere Kalenderwochen einer Range gerufen wird oder eine neue Range-Methode `BookingService::get_for_range(sales_person_id, range)` eingeführt wird (Vorgabe: erst die einfache Loop-Variante, optimieren falls Tests Performance-Druck zeigen).

### Shift-Plan-Markierungs-Surface (Area C)

- **D-Phase3-09:** **`ShiftplanViewService` bekommt per-sales-person-Variante.** Neue Methoden `get_shiftplan_week_for_sales_person(shiftplan_id, year, week, sales_person_id, ctx, tx) -> Result<ShiftplanWeek, ServiceError>` und `get_shiftplan_day_for_sales_person(year, week, day_of_week, sales_person_id, ctx, tx) -> Result<ShiftplanDayAggregate, ServiceError>`. Bestehende globale `get_shiftplan_week` / `get_shiftplan_day` bleiben unverändert (additiv).
- **D-Phase3-10:** **`ShiftplanDay` bekommt `unavailable: Option<UnavailabilityMarker>`.** `enum UnavailabilityMarker { AbsencePeriod { absence_id: Uuid, category: AbsenceCategory }, ManualUnavailable, Both { absence_id: Uuid, category: AbsenceCategory } }`. Pro Tag genau eine Variante — bei Doppel-Quelle wird `Both` gesetzt (mit der absence_id und category aus der AbsencePeriod, da die mehr semantischen Inhalt trägt als der bloße `sales_person_unavailable`-Eintrag). Slots bleiben sichtbar — Frontend rendert die Markierung als Per-Tag-Badge ZUSÄTZLICH zu den Slots.
- **D-Phase3-11:** **`build_shiftplan_day` bekommt eine optionale Parameter-Variante** (oder einen neuen Parallel-Helper `build_shiftplan_day_for_sales_person`), die zusätzlich `absence_periods: &[AbsencePeriod]` (für die Woche) und `manual_unavailables: &[SalesPersonUnavailable]` (für die Woche) entgegennimmt und das `unavailable`-Feld pro Tag ableitet. Plan-Phase entscheidet zwischen "Optional-Parameter" und "Parallel-Helper" — Vorgabe: Parallel-Helper, weil er die Globalsicht nicht touchiert (kein Risiko, Bestand zu brechen).
- **D-Phase3-12:** **Permission HR ∨ `verify_user_is_sales_person(sales_person_id)`.** Gleiche Regel wie AbsenceService Phase 1 D-09. Neuer REST-Endpoint `GET /shiftplan/{shiftplan_id}/year/{year}/week/{week}/sales-person/{sales_person_id}` und analog für day-Aggregate. ApiDoc + utoipa-Annotation Pflicht.
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
- **D-Phase3-17:** **`SalesPersonUnavailableService::create` bleibt unverändert (deferred).** Anlegen einer manuellen Unavailable auf einem Tag, der bereits durch eine `AbsencePeriod` abgedeckt ist, läuft still durch — keine Warning, kein Block. Phase 3 erweitert NUR Booking + Absence-Schreibpfade. Eine Warning-Variante `ManualUnavailableOnAbsenceDay` wird in Phase 3 NICHT eingeführt — die Folgephase, die `SalesPersonUnavailableService::create` symmetrisiert, fügt sie dann hinzu (Vermeidung von dead enum-Arms). UI-Markierung-Logik (`UnavailabilityMarker::Both`) sieht den Doppel-Zustand sowieso.

### Claude's Discretion

- **C-Phase3-01:** **Modul-Lokation für `Warning` und `BookingCreateResult` / `AbsencePeriodCreateResult` / `CopyWeekResult`.** Plan-Phase wählt zwischen einem neuen `service/src/warning.rs` (eigenes Modul, sauber) und Inline in den jeweiligen Service-Modulen (`service/src/booking.rs`, `service/src/absence.rs`). Vorgabe: eigenes `warning.rs`-Modul für `Warning`-Enum (geteilt zwischen Booking und Absence); Wrapper-Structs leben jeweils im Service-Modul, das sie produziert.
- **C-Phase3-02:** **Range-Lookup-API für AbsenceService::create.** Plan-Phase darf zwischen Mehrfach-`BookingService::get_for_week`-Calls (Loop über Kalenderwochen einer Range) und einer neuen `BookingService::get_for_range(sales_person_id, range)`-Methode wählen. Vorgabe: erst die Loop-Variante; nur optimieren wenn Tests einen messbaren Performance-Druck zeigen (premature-optimization-Vermeidung).
- **C-Phase3-03:** **`build_shiftplan_day_for_sales_person`-Layout.** Plan-Phase darf zwischen "neuer Helper neben `build_shiftplan_day`" und "Optional-Parameter am bestehenden Helper" wählen. Vorgabe: neuer Helper, weil er die globale Sicht nicht anfasst (Test-Stabilität für Phase 1/2-Tests).
- **C-Phase3-04:** **`Warning`-zu-`WarningTO`-Conversion.** Plan-Phase entscheidet zwischen `From<&Warning> for WarningTO` (Rust-Standard, einfach) und einer Tag-und-Daten-Map-Repräsentation für utoipa (`{ "kind": "BookingOnAbsenceDay", "data": { ... } }` mit `#[serde(tag = "kind", content = "data")]`). Vorgabe: Tag-und-Daten mit `#[serde(tag, content)]` — generiert sauberes OpenAPI-Schema für Frontend-Generatoren.
- **C-Phase3-05:** **Test-Fixture für Doppel-Quelle.** Plan-Phase darf entscheiden, ob ein dedizierter Cross-Source-Integration-Test in `shifty_bin/src/integration_test/` lebt oder ob er als Sub-Test eines bestehenden Booking/Absence-Integration-Tests folgt. Vorgabe: eigene Datei `shifty_bin/src/integration_test/booking_absence_conflict.rs` analog `absence_period.rs` aus Phase 1.
- **C-Phase3-06:** **Performance-Caching für `copy_week`.** N Bookings in `copy_week` = N `find_overlapping_for_booking`-DAO-Calls + N `sales_person_unavailable.get_by_week`-Calls. Plan-Phase darf vor der Schleife eine Pre-fetch-Optimierung einbauen (eine Range-Query, dann clientside-Filter), wenn Performance-Tests das nahelegen. Vorgabe: erst messen, dann optimieren — Default-Implementation = N Calls.
- **C-Phase3-07:** **Kategorie-Trigger-Differenzierung.** Default: alle 3 AbsenceCategorien (Vacation/Sick/UnpaidLeave) triggern Warnings gleichermaßen. Plan-Phase darf eine Differenzierung einführen (z.B. UnpaidLeave als nur "informativ"), wenn UX-Argumente vorliegen. Vorgabe: keine Differenzierung, alle gleich.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level Spezifikationen

- `.planning/ROADMAP.md` § Phase 3 — Goal, Depends-on Phase 1, Success-Criteria 1-4. Insbesondere SC4 (Pitfall-6: soft-deleted AbsencePeriods triggern keine Warnung).
- `.planning/STATE.md` — Architektur-Decisions: `BookingCreateResult { booking, warnings }`-Wrapper-Pattern aus Research; Direction `BookingService → AbsenceService` (nie umgekehrt für Schreibpfade).
- `shifty-backend/CLAUDE.md` — Layered Architecture; OpenAPI-Pflicht für REST; Service-Method-Signature-Pattern (`Option<Transaction>`).
- `shifty-backend/CLAUDE.local.md` — VCS via `jj` (alle Commits manuell durch User; GSD-Auto-Commit ist deaktiviert via `commit_docs: false`-Konvention im STATE/local-instructions); NixOS-Hinweise.
- `~/.claude/CLAUDE.md` — Tests sind Pflicht für jede Änderung.

### Vorphase-Outputs (Pflicht-Lektüre)

- `.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — D-08 (Phase-3-Wrapper-Decision), D-09 (Permission-Pattern HR ∨ self), D-12 (find_overlapping kategorie-scoped), D-15 (exclude_logical_id beim Update), D-16/17 (DateRange-API mit `iter_days`/`day_count` schon verfügbar).
- `.planning/phases/01-absence-domain-foundation/01-VERIFICATION.md` — Phase-1-Verification-Report; bestätigt dass `AbsenceService` + `AbsenceDao` end-to-end existieren.
- `.planning/phases/01-absence-domain-foundation/01-PATTERNS.md` — Pattern-Mapping aus Phase 1 (Code-Templates wiederverwendbar).
- `.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md` — D-Phase2-08-A (ReportingService-Switch hinter `absence_range_source_active` flag); für Phase 3 NICHT direkt relevant (Phase 3 ist flag-unabhängig — der `find_overlapping_for_booking`-Pfad nutzt KEIN `derive_hours_for_range`).

### Code-Templates für Phase 3

#### BookingService-Erweiterung

- `service/src/booking.rs:93` — `BookingService::create`-Signatur (heutige Form `Result<Booking, ServiceError>`); wird zu `Result<BookingCreateResult, ServiceError>`.
- `service/src/booking.rs:99-107` — `BookingService::copy_week`-Signatur (heutige Form `Result<(), ServiceError>`); wird zu `Result<CopyWeekResult, ServiceError>`.
- `service_impl/src/booking.rs:181-303` — aktuelle `create`-Implementation (Permission, Validation, Slot-Eligibility, DAO-Create); HIER neue Schritte einfügen (Date-Konversion + AbsenceService-Lookup + SalesPersonUnavailableService-Lookup + Warning-Construction).
- `service_impl/src/booking.rs:260-263` — bestehender Slot-Lookup; perfekter Andock-Punkt für die `time::Date::from_iso_week_date`-Konversion.
- `service_impl/src/booking.rs:305-363` — aktuelle `copy_week`-Implementation; Warning-Aggregation in der inneren Schleife einfügen.
- `service_impl/src/shiftplan.rs:138` — `time::Date::from_iso_week_date`-Pattern als Vorlage.

#### AbsenceService-Erweiterung

- `service/src/absence.rs:154` — `create`-Signatur (heutige Form `Result<AbsencePeriod, ServiceError>`); wird zu `Result<AbsencePeriodCreateResult, ServiceError>`.
- `service/src/absence.rs:161` — `update`-Signatur (heutige Form `Result<AbsencePeriod, ServiceError>`); wird symmetrisch zu `Result<AbsencePeriodCreateResult, ServiceError>`.
- `service_impl/src/absence.rs:137` — aktuelle `create`-Implementation; HIER neue Schritte: BookingService::get_for_week-Loop für die Range, SalesPersonUnavailableService-Lookup, Warning-Konstruktion.
- `service_impl/src/absence.rs:204` — aktuelle `update`-Implementation; analoge Erweiterung wie create (auf NEUE Range, kein Diff-Modus).
- `service_impl/src/absence.rs:175,248` — bestehende `find_overlapping`-Calls (kategorie-scoped, für Self-Overlap-Check); NEU dazu kommt der cross-kategorie `find_overlapping_for_booking`-Call vom BookingService aus.

#### DAO-Erweiterung

- `dao/src/absence.rs:78` — `find_overlapping`-Trait-Methode (kategorie-scoped); NEUE Methode `find_overlapping_for_booking(sales_person_id, range, tx)` daneben.
- `dao_impl_sqlite/src/absence.rs` (Phase 1) — SQLx-Pattern; SQL-Template für IN-Clause oder Category-frei: `SELECT ... FROM absence_period WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL`.

#### ShiftplanViewService-Erweiterung

- `service/src/shiftplan.rs` — Trait `ShiftplanViewService` mit `get_shiftplan_week` / `get_shiftplan_day` (heute global). NEUE Methoden `get_shiftplan_week_for_sales_person` / `get_shiftplan_day_for_sales_person`.
- `service/src/shiftplan.rs:14-31` — `ShiftplanDay` und `ShiftplanBooking`-Strukturen; `ShiftplanDay` bekommt `unavailable: Option<UnavailabilityMarker>`-Feld.
- `service_impl/src/shiftplan.rs:24-108` — `build_shiftplan_day`-Helper; neuer Parallel-Helper `build_shiftplan_day_for_sales_person` mit zusätzlichen `absence_periods` + `manual_unavailables` Parametern.
- `service_impl/src/shiftplan.rs:110-120` — `gen_service_impl!` für `ShiftplanViewServiceImpl`; neue DI-Dependencies: `AbsenceService` + `SalesPersonUnavailableService` (für die per-sales-person-Methoden).
- `service_impl/src/shiftplan.rs:127-203` — `get_shiftplan_week`-Implementation; Vorlage für die per-sales-person-Variante.

#### REST-Layer

- `rest/src/booking.rs` — REST-Handler für POST /booking; auf Wrapper-DTO umstellen + `error_handler`-Wrapper-Pattern bleibt.
- `rest/src/absence.rs` — REST-Handler für POST/PATCH /absence-period; analog umstellen.
- `rest/src/shiftplan.rs` — neue Endpoints `GET /shiftplan/{id}/year/{y}/week/{w}/sales-person/{sp}` und day-Aggregat-Variante.
- `rest-types/src/lib.rs` — neue inline DTOs `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `WarningTO`, `UnavailabilityMarkerTO` (Repo-Konvention: alle DTOs inline, siehe Phase 1 Override).
- `rest/src/lib.rs:120` — `error_handler`-Wrapper bleibt unverändert (mappt nur ServiceError; Warnings sind Erfolgs-Pfad).
- `rest/src/lib.rs` — ApiDoc-Erweiterung für neue Wrapper-DTOs + neue Endpoints (utoipa-Schema).

#### SalesPersonUnavailableService (read-only Konsumenten)

- `service/src/sales_person_unavailable.rs:67-74` — `get_by_week_for_sales_person(sales_person_id, year, calendar_week, ctx, tx)` — direkt nutzbar von `BookingService::create` für die Reverse-Warning.
- `service/src/sales_person_unavailable.rs:61-66` — `get_all_for_sales_person(sales_person_id, ctx, tx)` — nutzbar für `AbsenceService::create` (filtert clientside auf Range; oder Plan-Phase führt eine Range-Methode ein).
- `service_impl/src/sales_person_unavailable.rs` — bestehende Implementation; Phase 3 touchiert diese NICHT (deferred per D-Phase3-17).

#### Permission/Validation

- `service_impl/src/permission.rs` — `HR_PRIVILEGE`, `SHIFTPLANNER_PRIVILEGE`, `SALES_PRIVILEGE`-Konstanten; Permission-Konvention für die neuen per-sales-person-Methoden.
- `service/src/sales_person.rs:124-129` — `verify_user_is_sales_person`-Trait-Methode (für Permission-Gate auf `get_shiftplan_*_for_sales_person`).
- `service/src/lib.rs:121-128` — `ServiceError`-Surface; KEINE neuen Varianten erwartet (Warnings sind Erfolgs-Pfad).

#### DI-Verdrahtung

- `shifty_bin/src/main.rs` — `BookingServiceDependencies`-Block bekommt `AbsenceService` + `SalesPersonUnavailableService`. `AbsenceServiceDependencies`-Block bekommt `BookingService` + `SalesPersonUnavailableService`. `ShiftplanViewServiceDependencies` bekommt `AbsenceService` + `SalesPersonUnavailableService`. Plan-Phase prüft auf Initialisierungs-Reihenfolge (potenzielle DI-Reihenfolge-Probleme bei Cycle-Lookup; sollte aber nicht passieren da alle als `Arc` injiziert werden).

#### Testing

- `service_impl/src/test/booking.rs` (falls existiert; sonst Plan-Phase erstellt sie) — Service-Tests für `BookingService::create` mit Mock-`AbsenceService` und Mock-`SalesPersonUnavailableService`.
- `service_impl/src/test/absence/` (Phase 1) — Vorlage für Service-Tests-Struktur; neue Tests für Forward-Warning + Doppel-Quelle.
- `shifty_bin/src/integration_test/absence_period.rs` (Phase 1) — Vorlage für Integration-Test-Setup; neue Datei `booking_absence_conflict.rs` für Cross-Source-Tests.
- `service_impl/src/test/extra_hours.rs` — `_forbidden`-Test-Pattern (Mock-Setup mit `expect_check_permission` + `expect_verify_user_is_sales_person`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`AbsenceService` aus Phase 1** (`service/src/absence.rs`) — Trait + DI vorhanden; bekommt zwei neue Methoden (`find_overlapping_for_booking`) und Signatur-Bruch auf `*CreateResult`.
- **`AbsenceDao::find_overlapping`** (`dao/src/absence.rs:78`) — kategorie-scoped, bestehend; NEU `find_overlapping_for_booking` daneben mit cross-kategorie-Lookup.
- **`SalesPersonUnavailableService::get_by_week_for_sales_person`** (`service/src/sales_person_unavailable.rs:67-74`) — direkt nutzbar für die Reverse-Warning beim Booking-Create. Keine API-Änderung nötig.
- **`time::Date::from_iso_week_date`** — Standard-time-API, bereits in `service_impl/src/shiftplan.rs:138` benutzt; `(year, week, weekday) -> time::Date`. Konvertiert einen Booking-Tag zu einem konkreten Datum für den AbsencePeriod-Lookup.
- **`shifty_utils::DateRange`** (Phase 1, `shifty-utils/src/date_range.rs`) — `iter_days`, `day_count`, `overlaps`, `contains` schon verfügbar; nutzbar für die Range-basierten Lookups in Phase 3.
- **`gen_service_impl!`-Macro** — DI-Pattern direkt für die erweiterten ServiceDeps wiederverwendbar; neue Dependencies werden mechanisch hinzugefügt.
- **`tokio::join!` für Permission-Checks** (`service_impl/src/extra_hours.rs:236-245`-Pattern) — direkt für `HR ∨ verify_user_is_sales_person` in den per-sales-person-Methoden.
- **`build_shiftplan_day`-Helper** (`service_impl/src/shiftplan.rs:24-108`) — als Vorlage für `build_shiftplan_day_for_sales_person`; gleiche Slot-Filter-Logik (Holiday, ShortDay-Cutoff), neue Parameter für absence + manual_unavailable.
- **REST `error_handler`-Wrapper** (`rest/src/lib.rs:120`) — bleibt unverändert; Warnings sind im Erfolgs-Body, nicht im ServiceError.
- **`#[automock]` auf Service-Traits** — Mock-`AbsenceService` und Mock-`SalesPersonUnavailableService` sind für `BookingService`-Tests sofort verfügbar (Pattern aus Phase 1/2 etabliert).

### Established Patterns

- **Layered Architecture:** REST → Service-Trait → DAO-Trait → SQLx. Phase 3 erweitert auf 3 Schichten konsistent.
- **Soft-Delete-Konvention:** `WHERE deleted IS NULL` in jedem Read-SQL — gilt auch für `find_overlapping_for_booking` (Pitfall-6-Test verifiziert).
- **Wrapper-DTO-Inline-Konvention:** alle DTOs in `rest-types/src/lib.rs` inline (Phase 1 Override aus 01-VALIDATION.md). `BookingCreateResultTO`, `AbsencePeriodCreateResultTO` etc. folgen dem.
- **Permission-Pattern:** `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(sales_person_id))` + `.or()`. Direkt für die per-sales-person-Methoden.
- **`gen_service_impl!`-DI-Erweiterung:** mechanisches Hinzufügen neuer Service-Dependencies; Plan-Phase prüft DI-Reihenfolge in `shifty_bin/src/main.rs`.
- **Architektur-Direction (STATE.md):** `BookingService → AbsenceService` für Konflikt-Detektion; `AbsenceService → BookingService` NUR für Lese-Lookups (kein zyklischer Schreibpfad).

### Integration Points

- **`service/src/booking.rs:93,99-107`** — `BookingService::create` und `copy_week` Signatur-Brüche. Alle Call-Sites müssen mit (Compiler-Druck — gewünscht).
- **`service/src/absence.rs:154,161`** — `AbsenceService::create` und `update` Signatur-Brüche. Alle Call-Sites müssen mit.
- **`service/src/shiftplan.rs:14-69`** — `ShiftplanViewService` Trait + `ShiftplanDay`-Struct. Neue Methoden + neues Feld `unavailable`.
- **`dao/src/absence.rs:78`** — DAO-Trait neue Methode `find_overlapping_for_booking`.
- **`dao_impl_sqlite/src/absence.rs`** (Phase 1) — neue SQL-Query (kein Migration nötig — bestehender Index `(sales_person_id, from_date) WHERE deleted IS NULL` aus Phase 1 D-04 reicht).
- **`shifty_bin/src/main.rs`** — DI-Erweiterung: `BookingServiceDependencies` bekommt `AbsenceService` + `SalesPersonUnavailableService`; `AbsenceServiceDependencies` bekommt `BookingService` + `SalesPersonUnavailableService`; `ShiftplanViewServiceDependencies` bekommt beide.
- **`rest/src/booking.rs` + `rest/src/absence.rs` + `rest/src/shiftplan.rs`** — REST-Handler-Migration auf Wrapper-DTOs; neuer per-sales-person-Endpoint; ApiDoc-Erweiterung.
- **`rest-types/src/lib.rs`** — inline DTOs: `WarningTO` (Tag-Enum), `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `UnavailabilityMarkerTO`.

### Risiken / Pitfalls für Phase 3

- **Pitfall 6 (Soft-Delete + Warnings, SC4):** soft-deleted `AbsencePeriod`s dürfen KEINE Warnung und KEINE Markierung triggern. DAO-Konvention `WHERE deleted IS NULL` ist Pflicht für `find_overlapping_for_booking`. Test: ein soft-deleted AbsencePeriod im Range → BookingCreateResult.warnings ist leer; ShiftplanDay.unavailable ist None.
- **Architektur-Direction (Cycle-Vermeidung):** `BookingService` und `AbsenceService` referenzieren sich gegenseitig (BookingService → AbsenceService für Reverse, AbsenceService → BookingService für Forward). Beide sind Lese-Pfade, keine Schreib-Cycles. DI funktioniert, weil `Arc<dyn Trait>`-Injection die Compile-Time-Cycle-Detection vermeidet. Plan-Phase prüft die `gen_service_impl!`-Reihenfolge.
- **Date-Konversion-Pitfall:** `time::Date::from_iso_week_date` kann fehlschlagen (ungültige Week-Nummer). Plan-Phase entscheidet: Mappung auf `ServiceError::InternalError` (sollte nicht vorkommen, da Validation-Pre-Check) oder eigene Variante. Default: InternalError mit Kontext-Logging.
- **Doppel-Quelle De-Dup:** Bei der Anzeige (UnavailabilityMarker) muss klar sein, welche Quelle priorisiert wird, wenn beide vorliegen. Entscheidung: `Both` als eigene Variante, trägt absence-id (mehr semantischer Inhalt). Manuelle Unavailable hat keine semantischen Felder außer der ID.
- **copy_week-Performance:** Bei 50 Bookings = 50 `find_overlapping_for_booking`-Calls + 50 `get_by_week_for_sales_person`-Calls. Default OK, aber Plan-Phase darf eine Pre-fetch-Optimierung einbauen wenn Tests Druck zeigen (C-Phase3-06).
- **Forward-Warning-Performance bei langen Absences:** Eine 60-Tage-Absence (z.B. Sabbatical) ⇒ Loop über 60 Tage × N Bookings/Tag. Plan-Phase darf entscheiden, ob `BookingService::get_for_range(sales_person_id, range)` als neue Methode eingeführt wird (C-Phase3-02). Default: per-Woche-Loop.
- **Frontend-Bruch:** Wrapper-DTO-Bruch im REST trifft das alte Dioxus-Frontend (BookingTO im Body wird zu BookingCreateResultTO). Frontend-Workstream ist separat — Phase 3 dokumentiert den Bruch im OpenAPI-Snapshot, Frontend muss nachziehen. Aus Backend-Sicht ist der Bruch erwünscht (sauberes API).

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
  pub struct BookingCreateResult {
      pub booking: Booking,
      pub warnings: Arc<[Warning]>,
  }

  pub struct AbsencePeriodCreateResult {
      pub absence: AbsencePeriod,
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
- **Test-Fixture-Idee für Cross-Source-Test (Plan-Phase darf finalisieren):**
  - 1 Sales-Person `sp1`, 1 Slot Mo (`day_of_week = Monday`), Year 2026 / KW 18.
  - 1 AbsencePeriod `Vacation, 2026-04-27 .. 2026-04-30` (Mo-Do).
  - 1 SalesPersonUnavailable `(2026, 18, Monday)`.
  - Booking-Create-Versuch für `sp1` auf Slot Mo, KW 18, 2026 → BookingCreateResult mit 2 Warnings: `BookingOnAbsenceDay { date: 2026-04-27, absence_id, category: Vacation }` + `BookingOnUnavailableDay { ... }`. Booking trotzdem persistiert.
  - ShiftplanDay-Aufruf für `sp1`, Mo KW 18 → `unavailable: Some(UnavailabilityMarker::Both { absence_id, category: Vacation })`.
- **Pitfall-6-Test (verbatim für Plan-Phase):**
  ```rust
  // GIVEN: AbsencePeriod for sp1 on 2026-04-27 .. 2026-04-30, dann soft-deleted
  // WHEN: BookingService::create für sp1 auf Slot Mo KW 18 2026
  // THEN: BookingCreateResult.warnings ist EMPTY (KEIN BookingOnAbsenceDay)
  // UND: ShiftplanDay.unavailable ist None (KEIN AbsencePeriod-Marker)
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
- **Performance-Caching für `copy_week` (Pre-fetch-Range-Query statt N Calls)** — erst messen, dann optimieren. Plan-Phase entscheidet basierend auf Test-Performance.
- **Eigene Range-Methode `BookingService::get_for_range(sales_person_id, range)`** — Plan-Phase darf einführen falls Performance bei langen Absences es nahelegt; Default: Loop über Kalenderwochen mit `get_for_week`.
- **Frontend-Backward-Compat** — Wrapper-DTO-Bruch im REST trifft das alte Dioxus-Frontend; Frontend-Workstream ist separat. Phase 3 dokumentiert den Bruch im OpenAPI-Snapshot.
- **REST-Deprecation alter Endpunkte** — Phase 4 / MIG-05 (im Zusammenhang mit ExtraHours-Migration).
- **Phase-4-Cutover-Gate (MIG-02/MIG-03)** — Phase 4. Phase 3 nutzt keinen Reporting-Pfad; ist flag-unabhängig.
- **Carryover-Refresh** — Phase 4 (MIG-04).
- **Phase-1-Hygiene-Drift** (lokale `localdb.sqlite3`-Migration aus Phase-2-deferred-items.md) — Phase 4 wird das nachreichen; Phase 3 ist davon nicht betroffen, weil Phase-3-Tests in Mock + In-Memory-SQLite laufen.

</deferred>

---

*Phase: 3-Booking-Shift-Plan-Konflikt-Integration*
*Context gathered: 2026-05-02*
