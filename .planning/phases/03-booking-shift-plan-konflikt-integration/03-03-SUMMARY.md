---
phase: 03-booking-shift-plan-konflikt-integration
plan: 03
subsystem: service-layer
tags: [rust, mockall, gen_service_impl, di, service-tier, business-logic, wave-2, jj]

# Dependency graph
requires:
  - phase: 03-booking-shift-plan-konflikt-integration/03-02
    provides: service::warning::Warning enum + AbsenceDao::find_overlapping_for_booking trait/Impl + AbsencePeriod::from(&AbsencePeriodEntity)
  - phase: 01-absence-domain-foundation
    provides: AbsenceService trait + Impl (Phase-1-Sig: create/update returnten AbsencePeriod)
provides:
  - "service::absence::AbsencePeriodCreateResult { absence: AbsencePeriod, warnings: Arc<[Warning]> }"
  - "AbsenceService::create / ::update neue Sig: Result<AbsencePeriodCreateResult, ServiceError> (BOOK-01 Forward-Warning-Surface)"
  - "AbsenceService::find_overlapping_for_booking — kategorie-frei, Permission HR ∨ self (D-Phase3-12); Service-Surface für BOOK-02 Reverse-Pfad in Plan 03-04"
  - "AbsenceServiceImpl: gen_service_impl! erweitert um BookingService + SalesPersonUnavailableService + SlotService; Forward-Warning-Loop in create + update (D-Phase3-04 / D-Phase3-15 / D-Phase3-16)"
  - "DI in shifty_bin/src/main.rs verdrahtet AbsenceServiceImpl mit den 3 neuen Basic-Service-Deps; Konstruktionsreihenfolge bleibt Tier-konform"
  - "REST-Handler create_absence_period + update_absence_period unwrappen .absence aus Wrapper-Result (Plan-05-Migration markiert TODO)"
affects: [03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wrapper-Result-Sig-Bruch via Compiler-Forcing: AbsenceService-Trait + Impl + Mock + REST-Handler + Integration-Tests + Internal-Helper-Calls werden simultan vom Compiler erzwungen — keine OnceLock/Forward-Decl nötig."
    - "Forward-Warning-Loop als private impl<Deps> AbsenceServiceImpl<Deps>::compute_forward_warnings — DRY zwischen create und update; läuft NACH dem DAO-Persist und VOR dem commit; Authentication::Full im internen Loop (outer Permission ist HR ∨ self bereits verifiziert)."
    - "Service-Tier-Konvention im Code-Layout: Business-Logic-Service (AbsenceService) konsumiert 3 Basic Services (BookingService + SalesPersonUnavailableService + SlotService); Direction Business-Logic ↑ Basic ↓; kein Cycle (D-Phase3-18 BookingService-Files unangetastet)."
    - "Test-Mock-Default-Strategy: Default-Returns von booking_service.get_for_week / sales_person_unavailable_service.get_all_for_sales_person / slot_service.get_slot returnieren neutrale Werte (leere Arc / default_slot_monday()) — Bestand-Tests OHNE Override panicken nicht; Tests die explizite Forward-Warnings prüfen wollen, überschreiben die Mocks lokal."

key-files:
  created: []
  modified:
    - service/src/absence.rs
    - service_impl/src/absence.rs
    - service_impl/src/test/absence.rs
    - rest/src/absence.rs
    - shifty_bin/src/main.rs
    - shifty_bin/src/integration_test/absence_period.rs

key-decisions:
  - "Wrapper als AbsencePeriodCreateResult { absence, warnings: Arc<[Warning]> } — Arc<[T]> ist projektkonsistent (siehe AbsenceService-Trait); ergibt clone-billige Forward-Propagation in Plan 05's REST-DTO."
  - "find_overlapping_for_booking: Permission HR ∨ verify_user_is_sales_person — gleiche Read-Regel wie find_by_sales_person; mitigiert T-3-AbsServ-Read (kein Leak fremder absence-IDs an Non-HR-Konsumenten)."
  - "Forward-Warning-Loop nutzt active.logical_id als stable absence_id im Update-Pfad — D-07-konform; UI referenziert über den Lebenszyklus den logischen Eintrag, nicht die rotierte physische Row."
  - "SlotService als Dep AUFGENOMMEN (nicht 2 sondern 3 neue Deps): Booking trägt nur slot_id + calendar_week + year; für die Date-Auflösung pro Booking-Tag braucht der Loop slot.day_of_week. Pattern analog zu shiftplan_edit.rs:311 (existing call-site)."
  - "REST-Handler droppt Warnings im Body und markiert TODO Plan-05; vermeidet OpenAPI-Bruch in Plan 03 (AbsencePeriodTO ist als 200/201 body schema in der ApiDoc geleakt; ein kompletter Wrapper-DTO würde frontend force-update verlangen)."
  - "Forward-Warning-Loop ignoriert booking.deleted (Pitfall-1: keine soft-deleted Bookings in Warnings) und manual_unavailable.deleted (Pitfall-1 / SC4)."

patterns-established:
  - "Phase-3-Sig-Bruch-Anatomie (Wrapper-Result): 1 Trait-Patch + 1 Impl-Patch + Mock-Tests-Patch (.unwrap().absence) + REST-Handler-Patch (drop warnings) + Integration-Test-Helper-Patch (.unwrap().absence) — 5 Edit-Sites, alle Compiler-erzwungen."
  - "Wave-2-Plan-Struktur (3 Tasks): 1) Trait-Erweiterung, 2) Impl + Tests, 3) REST + DI — jede Task atomar als jj-Change; Build kann zwischen Task 1 und Task 3 partiell scheitern (REST-Crate), das ist OK."

requirements-completed: []  # BOOK-01 wird erst durch Plan 06 (Integration-Tests + Verifikation) als komplett markiert; BOOK-02 hängt zusätzlich an Plan 04 + 06.

# Metrics
duration: ~25min
completed: 2026-05-02
---

# Phase 3 Plan 03: Wave-2 AbsenceService Forward-Warning Summary

**`AbsenceService::create`/`::update` produzieren ab jetzt einen `AbsencePeriodCreateResult { absence, warnings: Arc<[Warning]> }`-Wrapper; pro Booking-Tag in der NEUEN Range eine `AbsenceOverlapsBooking`-Warning, pro überlappendem ManualUnavailable eine `AbsenceOverlapsManualUnavailable`-Warning (D-Phase3-04 / D-Phase3-15 / D-Phase3-16). Neue Trait-Methode `find_overlapping_for_booking` ist Service-Surface für BOOK-02 (Plan 03-04). DI in `shifty_bin/src/main.rs` verdrahtet `AbsenceServiceImpl` mit 3 neuen Basic-Service-Deps — Service-Tier-Konvention respektiert. BookingService 4 Files unangetastet (D-Phase3-18 Regression-Lock erfüllt).**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-02 (Folge auf Plan 03-02)
- **Completed:** 2026-05-02
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- `service/src/absence.rs`:
  - Neuer `pub struct AbsencePeriodCreateResult { absence, warnings }` (Wrapper-Result für BOOK-01).
  - Trait-Sig-Brüche: `create` und `update` returnen jetzt `Result<AbsencePeriodCreateResult, ServiceError>`.
  - Neue Trait-Methode `find_overlapping_for_booking(sales_person_id, range, ctx, tx) -> Result<Arc<[AbsencePeriod]>, ServiceError>` — kategorie-frei, Permission HR ∨ self (D-Phase3-12).

- `service_impl/src/absence.rs`:
  - `gen_service_impl!`-Block erweitert um `BookingService` + `SalesPersonUnavailableService` + `SlotService`.
  - `create`/`update` rufen den neuen privaten Helper `compute_forward_warnings` NACH dem DAO-Persist + VOR dem `commit`.
  - `compute_forward_warnings` iteriert pro betroffene Kalenderwoche genau einmal `BookingService::get_for_week`; für jedes Booking eine `SlotService::get_slot`-Auflösung, dann `Date::from_iso_week_date` + Range-Check; `Booking.deleted` wird gefiltert (Pitfall-1).
  - ManualUnavailables: ein einziger `get_all_for_sales_person`-Call, clientseitiger Soft-Delete- und Range-Filter.
  - Warnings tragen `absence_id = entity.id` (Create-Pfad — D-07: id == logical_id) bzw. `active.logical_id` (Update-Pfad — D-07: stable über Updates).
  - `find_overlapping_for_booking`-Impl: Permission HR ∨ verify_user_is_sales_person, dann DAO-Call + Domain-Mapping.
  - `range_contains`-Helper-Funktion ergänzt (`DateRange` selbst hat kein `contains`).

- `service_impl/src/test/absence.rs`:
  - `AbsenceDependencies`-Struct um 3 neue Mock-Felder erweitert (`MockBookingService`, `MockSalesPersonUnavailableService`, `MockSlotService`).
  - Lokaler `default_slot_monday()`-Fixture-Helper analog zu `service_impl/src/test/shiftplan.rs:32-45`.
  - `build_dependencies` setzt Default-Mocks: leere Bookings, leere ManualUnavailables, Default-Slot — kein `unimplemented!()` (vermeidet Test-Falle für Bestand-Tests, die slot_service nie überschreiben).
  - Zwei Bestand-Test-Stellen patches: `service.create(...).await.expect("...").absence` und `service.update(...).await.expect("...").absence` (Compiler-erzwungen).

- `rest/src/absence.rs`:
  - `create_absence_period` + `update_absence_period` rufen den Wrapper-Pfad und mappen `result.absence` ins `AbsencePeriodTO` — Body-Form bleibt Phase-1; Warnings werden vorerst dropped (TODO Plan-05).

- `shifty_bin/src/main.rs`:
  - `AbsenceServiceDependencies`-typed-Block ergänzt um `BookingService` + `SalesPersonUnavailableService` + `SlotService`-Type-Bindings.
  - `Arc::new(AbsenceServiceImpl { ... })`-Konstruktion ergänzt um die 3 neuen Felder; Konstruktionsreihenfolge respektiert Service-Tier-Konvention (alle drei Basic-Services sind VOR `absence_service` gebaut).

- `shifty_bin/src/integration_test/absence_period.rs`:
  - Zwei Helper-Stellen patches `.absence`-Unwrap (Compiler-erzwungen). Keine semantischen Test-Änderungen.

## Task Commits

Jede Task wurde atomar als ein jj-Change committed:

1. **Task 1: AbsenceService Trait — Wrapper + Sig-Brüche + neue Methode** — `d744efe4` (feat)
2. **Task 2: AbsenceServiceImpl Forward-Warning-Loop + find_overlapping_for_booking + Mock-Tests** — `106ea712` (feat)
3. **Task 3: REST-Handler + main.rs DI + Integration-Test-Helper-Patches** — `dfd66a56` (feat)

**Plan metadata commit:** _(diese SUMMARY + STATE.md/ROADMAP.md-Update — finaler `jj describe`)_

## Files Created/Modified

### Created (0)

Keine — Plan 03-03 ist reines additives Patchen bestehender Files.

### Modified (6)

| File | Lines Changed | Provenance |
|------|---------------|------------|
| `service/src/absence.rs` | +28 / 0 | Task 1 — Wrapper + Sig-Brüche + Trait-Methode |
| `service_impl/src/absence.rs` | +136 / -8 | Task 2 — Imports, gen_service_impl, Forward-Warning-Loop, find_overlapping_for_booking, range_contains-Helper |
| `service_impl/src/test/absence.rs` | +50 / -8 | Task 2 — neue Mocks, default_slot_monday, build_dependencies-Defaults, 2× .absence-Unwrap |
| `rest/src/absence.rs` | +6 / -4 | Task 3 — Wrapper-Pfad mit .absence-Unwrap (TODO Plan-05) |
| `shifty_bin/src/main.rs` | +14 / 0 | Task 3 — typed Deps + DI-Konstruktion (3 neue Felder) |
| `shifty_bin/src/integration_test/absence_period.rs` | +6 / -2 | Task 3 — Helper .absence-Unwrap (2 Stellen) |

## Wave-Mapping für Continuation-Executors

| Service-Surface | Konsument | Aktivierungs-Plan |
|-----------------|-----------|-------------------|
| `AbsenceService::find_overlapping_for_booking` | `ShiftplanEditService::book_slot_with_conflict_check` (Reverse-Warning, BOOK-02) | Plan 03-04 (Wave 3) |
| `AbsencePeriodCreateResult.warnings` (Forward) | REST-Wrapper-DTO `AbsencePeriodCreateResultTO` | Plan 03-05 (Wave 4) |
| `AbsenceService::create/update` Forward-Warnings (BOOK-01) | Cross-Source-Integration-Test `test_double_source_two_warnings_one_booking` | Plan 03-06 (Wave 5) |

## Decisions Made

- **Wrapper-Result `AbsencePeriodCreateResult`** statt Tupel `(AbsencePeriod, Vec<Warning>)` — Domain-Modell für die UI ist explizit; named-fields lesen sich am Call-Site klarer als positional-Args; konsistent mit dem in PROJECT.md geplanten `BookingCreateResult` für Plan 03-04.
- **`Arc<[Warning]>` statt `Vec<Warning>`** — clone-billig (Plan 05 wird mehrfach durch Layers reichen); konsistent mit AbsenceService-Trait, der bereits `Arc<[AbsencePeriod]>` returniert.
- **Forward-Warning-Loop läuft NACH dem DAO-Persist** — semantisch sauber: Self-Conflicts sind validiert, die Absence ist persistiert, Warnings sind reine Read-Operation; bei Permission-Fail oder Validation-Fail wird der Loop nie erreicht.
- **`Authentication::Full` im internen Loop** — outer Permission ist HR ∨ self bereits oben verifiziert (T-3-AbsServ-Read mitigiert); Inner-BookingService/SalesPersonUnavailableService/SlotService kennen diesen Caller-Kontext nicht und würden sonst eigene Permission-Checks durchführen, die das gleiche Result haben.
- **3 neue Deps statt 2** — `SlotService` ist nötig, weil `Booking` selbst keine `day_of_week` trägt (nur `slot_id` + `calendar_week` + `year`); für die Date-Auflösung pro Booking-Tag muss `slot.day_of_week` gelookupt werden. Plan-File hat das als Alternativ-Pfad markiert ("Wenn `BookingService::get_for_week` schon die `Booking`-Struct ... anreicherter Form liefert ..., nutze das. Sonst füge `SlotService` als Dep ergänzend ... hinzu"); `Booking` trägt diese Felder NICHT — daher SlotService.
- **Default-Mocks im Test-Setup statt `unimplemented!()`** — Bestand-Tests (z.B. `test_create_self_overlap_same_category_returns_validation`) durchlaufen die `find_overlapping`-Validation, returnen Err vor dem Forward-Warning-Loop und panicken nicht; Tests die explizite Forward-Warnings asserten wollen, überschreiben die Mocks gezielt. Anti-Pattern: `unimplemented!()`-Default würde JEDEN Bestand-Test, der versehentlich den Loop durchläuft, panic'n — Test-Falle.
- **REST-Body bleibt `AbsencePeriodTO` (kein OpenAPI-Bruch in Plan 03)** — Plan 05 wird `AbsencePeriodCreateResultTO` ergänzen; Plan 03 setzt nur den Service-Layer; Frontend muss noch nicht auf den neuen Wrapper migriert werden.

## Deviations from Plan

None — Plan executed wie geschrieben. Eine Stilanpassung (Helper `range_contains` als private freie Funktion statt inline `range.from() <= d && d <= range.to()`) wurde im Forward-Warning-Loop ergänzt; das ist minimal und keine semantische Abweichung.

## Issues Encountered

**Pre-existing 8 absence_period-Integration-Test-Failures in shifty_bin** (out of scope per Plan-Scope-Boundary):

`shifty_bin/src/integration_test/absence_period.rs` (8 Tests) scheitern weiterhin auf der lokalen Dev-DB mit `no such table: absence_period` — pre-existing seit Phase 1, dokumentiert in `.planning/phases/02-.../deferred-items.md` und `.planning/phases/03-.../deferred-items.md`. Plan 03-03 verändert das Test-Bild NICHT (kein neuer Fehler).

**Workspace-Test-Status nach Plan 03-03:**
- `service_impl --lib` (gefiltert auf `test::absence`): 28 passed, 0 failed, 0 ignored — GRÜN
- `service_impl --lib` (gesamt): 321 passed, 0 failed, 6 ignored (Plan-01-Stubs) — identisch zu Plan-02-Baseline
- `shifty_bin` integration: 20 passed, 8 failed (pre-existing Phase-1-Migrations-Lücke), 4 ignored (Plan-01-Stubs) — identisch zu Plan-02-Baseline
- `cargo build --workspace`: GRÜN
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- `cargo run` (mit timeout 15s): scheitert beim Migration-Step mit `VersionMissing(20260428101456)` — pre-existing Migrations-Drift aus Phase 1 (deferred-items.md). DI-Konstruktion + Service-Initialisierung sind aber erfolgreich — der Bin würde mit valider DB starten.

## Threat Flags

Keine neuen Threat-Surfaces eingeführt — `find_overlapping_for_booking` exposed Absence-IDs nur an HR ∨ self-authorisierte Konsumenten (T-3-AbsServ-Read mitigiert; Plan-Threat-Model 1:1 abgebildet). BookingService bleibt unangetastet (T-3-Cycle-Risk mitigiert). Soft-Delete-Filter für ManualUnavailable und Booking sind im Loop explizit (T-3-Soft-Del-Drop mitigiert).

## Self-Check

- service/src/absence.rs enthält `pub struct AbsencePeriodCreateResult` (grep-Count = 1)
- service/src/absence.rs enthält `find_overlapping_for_booking` (grep-Count = 1)
- service/src/absence.rs enthält `Result<AbsencePeriodCreateResult` (grep-Count = 2 — create + update)
- service_impl/src/absence.rs enthält `AbsenceOverlapsBooking` (grep-Count = 2: Doc-Comment + Code)
- service_impl/src/absence.rs enthält `AbsenceOverlapsManualUnavailable` (grep-Count = 2: Doc-Comment + Code)
- service_impl/src/absence.rs enthält `find_overlapping_for_booking` (grep-Count = 2: Trait-Method-Impl + DAO-Call)
- service_impl/src/test/absence.rs enthält `booking_service: MockBookingService` (grep-Count = 1)
- service_impl/src/test/absence.rs enthält `fn default_slot_monday` (grep-Count = 1)
- service_impl/src/test/absence.rs enthält 0× `unimplemented!()` (grep-Count = 0)
- rest/src/absence.rs enthält `result.absence` (grep-Count = 2)
- shifty_bin/src/main.rs enthält `booking_service: booking_service.clone()` (grep-Count = 5 — 1 in absence_service + 4 in shiftplan_edit/shiftplan_view/booking_information/block — passt)
- Commit `d744efe4` (Task 1 Trait) FOUND in jj log
- Commit `106ea712` (Task 2 Impl + Tests) FOUND in jj log
- Commit `dfd66a56` (Task 3 REST + DI) FOUND in jj log
- `cargo build --workspace` exit 0
- `cargo test -p service_impl test::absence`: 28 passed/0 failed/0 ignored
- **D-Phase3-18 Regression-Lock**: `jj diff --from porzpqqx --to @ -- service/src/booking.rs service_impl/src/booking.rs rest/src/booking.rs service_impl/src/test/booking.rs` produziert 0 Zeilen Diff über den gesamten Phase-3-Plan-02..03-Span

## Self-Check: PASSED

(re-verified after STATE/ROADMAP edits: SUMMARY.md FOUND, all 3 task-commits FOUND in jj log, cargo build --workspace exit 0, D-Phase3-18 Regression-Lock 0 lines diff over Phase-3-Plan-02..03)

## Next Phase Readiness

Plan 03-04 (Wave-3 ShiftplanEditService Reverse-Warning + ShiftplanViewService per-sales-person + DI-Wiring) kann unmittelbar starten:

- `AbsenceService::find_overlapping_for_booking` ist verfügbar — `ShiftplanEditService::book_slot_with_conflict_check` (Plan 03-04) kann via dieselbe DAO-Surface plus Permission-Layer Reverse-Warnings produzieren.
- `service::warning::Warning` enum trägt alle 4 Phase-3-Varianten — Plan 03-04 nutzt `BookingOnAbsenceDay` und `BookingOnUnavailableDay`.
- `ShiftplanDay.unavailable: Option<UnavailabilityMarker>` ist da; Plan 03-04 baut den per-sales-person-Helper, der das Feld setzt.
- Plan-01-Wave-3-Stubs in `service_impl/src/test/shiftplan_edit.rs` (6 #[ignore]-Tests) warten weiter auf Aktivierung in Plan 03-04.
- AbsenceServiceImpl-DI ist sauber Tier-konform; Plan 03-04 wird ShiftplanEditServiceImpl analog mit der `AbsenceService`-Dep ergänzen (Business-Logic ↑ Business-Logic — erlaubt per Service-Tier-Konvention solange kein Cycle).

**Wave-3-Forcing-State:** Plan 03-04 muss
1. `AbsenceService` als Dep in `ShiftplanEditServiceImpl::book_slot_with_conflict_check` aufnehmen (DI-Konstruktion in `main.rs` analog ergänzen).
2. Reverse-Warning-Loop produzieren: pro Booking-Tag, der von einer aktiven `AbsencePeriod` überlappt wird, eine `Warning::BookingOnAbsenceDay`; pro Booking-Tag, der durch `sales_person_unavailable` als nicht verfügbar markiert ist, eine `Warning::BookingOnUnavailableDay`.
3. 6 Plan-01-Stubs aktivieren (#[ignore] entfernen + Mock-DI-Setup analog `service_impl/src/test/booking.rs:113-192`).

**D-Phase3-18 Regression-Lock**: BookingService-Files (`service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs`) sind durch Plan 03-03 NICHT angetastet. Gilt weiter als Hard-Constraint für Plans 03-04..03-06.

---
*Phase: 03-booking-shift-plan-konflikt-integration*
*Plan: 03 (Wave 2 AbsenceService Forward-Warning)*
*Completed: 2026-05-02*
