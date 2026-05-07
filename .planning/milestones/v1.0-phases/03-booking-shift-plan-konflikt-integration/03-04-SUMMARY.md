---
phase: 03-booking-shift-plan-konflikt-integration
plan: 04
subsystem: service-layer
tags: [rust, mockall, gen_service_impl, di, service-tier, business-logic, wave-3, jj]

# Dependency graph
requires:
  - phase: 03-booking-shift-plan-konflikt-integration/03-02
    provides: Warning enum (4 Varianten) + AbsenceDao::find_overlapping_for_booking + UnavailabilityMarker enum + ShiftplanDay.unavailable Field
  - phase: 03-booking-shift-plan-konflikt-integration/03-03
    provides: AbsenceService::find_overlapping_for_booking (Service-Surface kategorie-frei, Permission HR ∨ self)
provides:
  - "service::shiftplan_edit::BookingCreateResult { booking, warnings: Arc<[Warning]> } + CopyWeekResult { copied_bookings, warnings }"
  - "ShiftplanEditService::book_slot_with_conflict_check (BOOK-02 Reverse-Warning, Permission HR ∨ self)"
  - "ShiftplanEditService::copy_week_with_conflict_check (D-Phase3-02, Permission shiftplan.edit, KEINE De-Dup)"
  - "ShiftplanViewService::get_shiftplan_week_for_sales_person + get_shiftplan_day_for_sales_person (PLAN-01, Permission HR ∨ self)"
  - "service_impl::shiftplan::build_shiftplan_day_for_sales_person (Parallel-Helper C-Phase3-03, 4-Wege-De-Dup mit Both-Variante)"
  - "ShiftplanEditServiceImpl: gen_service_impl! erweitert um AbsenceService-Dep"
  - "ShiftplanViewServiceImpl: gen_service_impl! erweitert um AbsenceService + SalesPersonUnavailableService-Deps"
  - "DI-Wiring in shifty_bin/src/main.rs: 3 neue clone-Pässe (1 für Edit, 2 für View); typed Bindings ergänzt"
  - "10 neue passende Service-Impl-Tests (6 ShiftplanEdit + 4 ShiftplanView per-sales-person)"
affects: [03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wrapper-Result-Pattern auf Business-Logic-Layer: BookingCreateResult/CopyWeekResult leben in service/src/shiftplan_edit.rs (NICHT in BookingService — D-Phase3-18-Lock); konsistent mit AbsencePeriodCreateResult aus Plan 03-03."
    - "Parallel-Helper-Pattern (C-Phase3-03): build_shiftplan_day_for_sales_person delegiert an build_shiftplan_day für Slot/Booking/Holiday-Logik und ergänzt das unavailable-Feld via 4-Wege-Match (None/AbsencePeriod/ManualUnavailable/Both); globale Sicht bleibt unangetastet."
    - "Test-Mock-Default-Strategy: build_dependencies setzt absence_service.find_overlapping_for_booking → Empty + sales_person_unavailable_service.get_by_week_for_sales_person → Empty + booking_service.create → persisted_booking; Tests, die explizite Warnings asserten, überschreiben mit checkpoint() + neuer expect."
    - "Permission HR ∨ self via tokio::join! mit hr.or(sp)? — Pattern verbatim aus extra_hours.rs:117-123 + AbsenceService::find_by_sales_person; bewährt in Phase 1."
    - "Counter-basierte Mock-Returns für copy_week-Aggregations-Test: Arc<Mutex<u32>>-Counter im Mock-Closure, Match auf Counter-Wert für Per-Aufruf-Differenzierung — alternative zu sequentieller mockall-Erwartungs-Komposition."

key-files:
  created: []
  modified:
    - service/src/shiftplan_edit.rs
    - service_impl/src/shiftplan_edit.rs
    - service_impl/src/test/shiftplan_edit.rs
    - service/src/shiftplan.rs
    - service_impl/src/shiftplan.rs
    - service_impl/src/test/shiftplan.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "copy_week_with_conflict_check Permission ist shiftplan.edit (HR/SHIFTPLANNER) — bulk-Op-Permission, NICHT HR ∨ self pro Source-Booking. Begründung: copy_week ist eine Schichtplan-Ebene-Operation, analog zu modify_slot/remove_slot in der gleichen Datei. Eine Per-Source-Booking-Permission würde pro Aufruf 7+ Permission-Probes auslösen und das Bulk-Pattern brechen."
  - "Reverse-Warning-Loop bricht nach EINER BookingOnUnavailableDay-Warning per day_of_week-Match — ein einzelner SP hat per Schema höchstens einen aktiven sales_person_unavailable-Eintrag pro (sp, year, week, dow); break verhindert duplikate Warnings, falls die DAO doch zwei liefert (Defensiv-Code)."
  - "Forbidden-Test für copy_week_with_conflict_check baut auf permission-only Forbidden — KEIN HR ∨ self-Test, weil die Methode shiftplan.edit-Permission nutzt; Test stellt sicher, dass der Permission-Pfad korrekt ist (nicht das HR-or-self-Pattern)."
  - "Tests aktivieren die 6 Plan-01-#[ignore]-Stubs mit echten Mock-DI-Setups statt sie nur durchzuziehen — passt zu Wave-3-Forcing-Anforderung in Plan 03-01-SUMMARY: 'Wave 3 muss diese Stubs aktivieren'. Stub-Test-Namen identisch zur Plan-01-Liste; #[ignore]-Attribute entfernt."
  - "ShiftplanViewService-Tests erfordern Default-Mock get_all_user_assignments → leere HashMap, weil HR-grant in Tests implizit auch SHIFTPLANNER-Privileg grantet (Mock-expect_check_permission filtert nicht nach role) und der Body dann get_all_user_assignments aufruft. Pattern: Default-Mocks im build_dependencies, Tests überschreiben gezielt."
  - "DI-Konstruktionsreihenfolge unverändert Tier-konform: absence_service (Z. 752) → shiftplan_edit_service (Z. 831) → shiftplan_view_service (Z. 856). Keine Block-Reorder nötig."

patterns-established:
  - "Wave-3-Plan-Struktur in Phase 3: 3 Tasks (Trait+Impl ShiftplanEditService / Trait+Impl ShiftplanViewService / DI in main.rs); jede Task atomar als jj-Change; alle 3 Tasks fügen TESTS gleichzeitig hinzu (Task 1: 6 Tests aktiviert, Task 2: 4 neue Tests)."
  - "D-Phase3-18 Regression-Lock-Verifikation: jj diff für 4 BookingService-Files == 0 Lines bei jedem Task-Commit + final über kompletten Plan-03-04-Span (von qzkyxvrl bis @-)."

requirements-completed: []  # BOOK-02 (Reverse-Warning) + PLAN-01 (per-sales-person-Marker) sind nach Plan 03-04 funktional komplett, werden aber erst nach Plan 03-06 (Integration-Tests + SC-Verifikation) als requirements-completed markiert.

# Metrics
duration: ~30min
completed: 2026-05-02
---

# Phase 3 Plan 04: Wave-3 ShiftplanEditService Reverse-Warning + ShiftplanViewService per-sales-person Summary

**`ShiftplanEditService::book_slot_with_conflict_check` + `::copy_week_with_conflict_check` produzieren ab jetzt einen `BookingCreateResult`/`CopyWeekResult`-Wrapper mit Cross-Source-Warnings; pro überlappende AbsencePeriod eine `Warning::BookingOnAbsenceDay`, pro überlappenden ManualUnavailable eine `Warning::BookingOnUnavailableDay` (D-Phase3-15: KEINE De-Dup). `ShiftplanViewService::get_shiftplan_*_for_sales_person` setzt `unavailable: Option<UnavailabilityMarker>` per Tag mit 4-Wege-Match (None/AbsencePeriod/ManualUnavailable/Both, D-Phase3-10). Der neue Parallel-Helper `build_shiftplan_day_for_sales_person` (C-Phase3-03) lässt die globale Sicht unangetastet. DI in `shifty_bin/src/main.rs` verdrahtet `ShiftplanEditServiceImpl` mit `absence_service.clone()` und `ShiftplanViewServiceImpl` mit `absence_service + sales_person_unavailable_service.clone()` — Service-Tier-Konvention respektiert. BookingService 4 Files unangetastet (D-Phase3-18 Regression-Lock erfüllt). 10 neue Tests grün (6 ShiftplanEdit + 4 ShiftplanView).**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-05-02 (Folge auf Plan 03-03)
- **Completed:** 2026-05-02
- **Tasks:** 3
- **Files modified:** 7
- **Files created:** 0

## Accomplishments

- `service/src/shiftplan_edit.rs`:
  - `pub struct BookingCreateResult { booking, warnings: Arc<[Warning]> }`
  - `pub struct CopyWeekResult { copied_bookings: Arc<[Booking]>, warnings: Arc<[Warning]> }`
  - 2 neue Trait-Methoden `book_slot_with_conflict_check` + `copy_week_with_conflict_check`
- `service_impl/src/shiftplan_edit.rs`:
  - `gen_service_impl!`-Block erweitert um `AbsenceService` als 12. Feld
  - `book_slot_with_conflict_check`: Permission HR ∨ self via `tokio::join!`; Slot-Lookup; `Date::from_iso_week_date` für Booking-Tag; AbsenceService::find_overlapping_for_booking + SalesPersonUnavailableService::get_by_week_for_sales_person; Persist via BookingService::create; Warnings mit echter persistierter ID; ManualUnavailable-Loop bricht nach erstem day_of_week-Match (Defensiv-Code).
  - `copy_week_with_conflict_check`: Permission `shiftplan.edit`; Source-Bookings via BookingService::get_for_week; Per-Source-Loop ruft intern `book_slot_with_conflict_check` und aggregiert Warnings ohne De-Dup.
- `service_impl/src/test/shiftplan_edit.rs`:
  - 6 #[ignore]-Stub-Tests aus Plan 03-01 AKTIVIERT mit echten Mock-DI-Setups:
    - `test_book_slot_warning_on_absence_day` (BOOK-02 / D-Phase3-14 BookingOnAbsenceDay) — GREEN
    - `test_book_slot_warning_on_manual_unavailable` (BOOK-02 / D-Phase3-14 BookingOnUnavailableDay) — GREEN
    - `test_book_slot_no_warning_when_softdeleted_absence` (SC4 / Pitfall-1, ManualUnavailable mit `deleted.is_some()` ignoriert) — GREEN
    - `test_copy_week_aggregates_warnings` (D-Phase3-02 / D-Phase3-15: 3 source bookings, 2 warnings, KEINE De-Dup) — GREEN
    - `test_book_slot_with_conflict_check_forbidden` (D-Phase3-12 HR ∨ self → beide Forbidden) — GREEN
    - `test_copy_week_with_conflict_check_forbidden` (shiftplan.edit Forbidden) — GREEN
- `service/src/shiftplan.rs`:
  - 2 neue Trait-Methoden `get_shiftplan_week_for_sales_person` + `get_shiftplan_day_for_sales_person` (PLAN-01).
- `service_impl/src/shiftplan.rs`:
  - `gen_service_impl!`-Block erweitert um `AbsenceService` + `SalesPersonUnavailableService` als 8.+9. Feld
  - Neuer Parallel-Helper `build_shiftplan_day_for_sales_person` (C-Phase3-03): delegiert für Slots/Bookings/Holiday-Filter an `build_shiftplan_day`, dann 4-Wege-De-Dup-Match (None/AbsencePeriod/ManualUnavailable/Both) mit Soft-Delete-Filter (Pitfall-1 / SC4) und Per-Sales-Person-Filter
  - 2 neue Impl-Methoden: Permission HR ∨ self; Pre-Fetch via `find_by_sales_person` + `get_by_week_for_sales_person` (Authentication::Full-Bypass intern); Pro-Tag-Loop via Parallel-Helper.
- `service_impl/src/test/shiftplan.rs`:
  - `ShiftplanViewServiceDependencies` erweitert um `MockAbsenceService` + `MockSalesPersonUnavailableService`-Felder + Trait-Bindings + `build_service`-Wiring + Default-Mocks (`expect_find_by_sales_person` → Empty, `expect_get_by_week_for_sales_person` → Empty, `expect_verify_user_is_sales_person` → Forbidden default für HR-Or-Self-Pattern, `expect_get_all_user_assignments` → leere HashMap)
  - 4 neue Tests:
    - `test_get_shiftplan_week_for_sales_person_marker_absence_only` — UnavailabilityMarker::AbsencePeriod{absence_id, category} auf Mo
    - `test_get_shiftplan_week_for_sales_person_marker_both` — UnavailabilityMarker::Both{absence_id, category} bei Doppel-Quelle (D-Phase3-10)
    - `test_get_shiftplan_week_for_sales_person_softdeleted_absence_no_marker` — Pitfall-1 / SC4: deleted.is_some() filtert raus
    - `test_get_shiftplan_week_for_sales_person_forbidden` — D-Phase3-12 HR ∨ self → beide Forbidden
- `shifty_bin/src/main.rs`:
  - `ShiftplanEditServiceDependencies` erweitert um `type AbsenceService = AbsenceService;`
  - `ShiftplanViewServiceDependencies` erweitert um `type AbsenceService = AbsenceService; type SalesPersonUnavailableService = SalesPersonUnavailableService;`
  - 1 neuer clone-Pass in `Arc::new(ShiftplanEditServiceImpl { ... })`: `absence_service: absence_service.clone()`
  - 2 neue clone-Pässe in `Arc::new(ShiftplanViewServiceImpl { ... })`: `absence_service.clone()` + `sales_person_unavailable_service.clone()`
  - Konstruktionsreihenfolge unverändert Tier-konform: `absence_service` (Z. 752) → `shiftplan_edit_service` (Z. 831) → `shiftplan_view_service` (Z. 856)

## Task Commits

Jede Task wurde atomar als ein jj-Change committed:

1. **Task 1: ShiftplanEditService Reverse-Warning + 6 Stub-Tests aktiviert** — `448c265e` (feat)
2. **Task 2: ShiftplanViewService per-sales-person + 4-Wege-Marker + 4 Tests** — `92358f05` (feat)
3. **Task 3: DI-Wiring in main.rs (3 clone-Pässe + 3 typed Bindings)** — `00ce6d87` (feat)

**Plan metadata commit:** _(diese SUMMARY + STATE.md/ROADMAP.md-Update — finaler `jj describe`)_

## Files Created/Modified

### Created (0)

Plan 03-04 ist reines additives Patchen + Test-Aktivierung — keine neuen Files.

### Modified (7)

| File | Lines Changed | Provenance |
|------|---------------|------------|
| `service/src/shiftplan_edit.rs` | +63 / 0 | Task 1 — BookingCreateResult + CopyWeekResult + 2 Trait-Methoden |
| `service_impl/src/shiftplan_edit.rs` | +169 / -3 | Task 1 — gen_service_impl-Erweiterung + 2 Impl-Methoden + Imports |
| `service_impl/src/test/shiftplan_edit.rs` | +475 / -52 | Task 1 — 6 #[ignore]-Stubs durch volle Mock-DI-Setups ersetzt |
| `service/src/shiftplan.rs` | +33 / 0 | Task 2 — 2 neue Trait-Methoden |
| `service_impl/src/shiftplan.rs` | +263 / -2 | Task 2 — gen_service_impl-Erweiterung + Parallel-Helper + 2 Impl-Methoden |
| `service_impl/src/test/shiftplan.rs` | +218 / -4 | Task 2 — Mock-DI-Setup-Erweiterung + 4 neue Tests |
| `shifty_bin/src/main.rs` | +12 / 0 | Task 3 — 3 typed Bindings + 3 clone-Pässe |

## Wave-Mapping für Continuation-Executors

| Service-Surface | Konsument | Aktivierungs-Plan |
|-----------------|-----------|-------------------|
| `BookingCreateResult` (Reverse) | REST-Wrapper-DTO `BookingCreateResultTO` + Endpunkt `POST /booking/with-conflict-check` | Plan 03-05 (Wave 4) |
| `CopyWeekResult` | REST-Wrapper-DTO + Endpunkt `POST /booking/copy-week-with-conflict-check` | Plan 03-05 (Wave 4) |
| `get_shiftplan_*_for_sales_person` | REST-Endpunkte `GET /shiftplan/.../sales-person/{id}` | Plan 03-05 (Wave 4) |
| Reverse-Warning-Cross-Source-Verifikation | 4 Plan-01-Wave-5-Stubs in `shifty_bin/src/integration_test/booking_absence_conflict.rs` | Plan 03-06 (Wave 5) |

## Decisions Made

- **`copy_week_with_conflict_check` Permission ist `shiftplan.edit`, nicht HR ∨ self**: Bulk-Operation-Pattern. HR ∨ self pro Source-Booking würde 7+ Probes pro Aufruf auslösen und ist semantisch falsch (copy_week ist Schichtplan-Edit, nicht Per-User-Mitarbeiter-Pflege). Konsistent mit existierendem `modify_slot`/`remove_slot`-Pattern in derselben Datei.
- **Reverse-Warning-Loop bricht nach erstem ManualUnavailable-Match**: Schema garantiert höchstens einen aktiven `sales_person_unavailable`-Eintrag pro `(sp, year, week, dow)`-Kombination. Defensiv-Code mit `break` verhindert duplikate Warnings, falls die DAO doch zwei liefert. Doppelte Defensive: `mu.deleted.is_none()` + `mu.day_of_week == slot.day_of_week`.
- **Counter-Mock-Pattern für `test_copy_week_aggregates_warnings`**: 3 Bookings durchlaufen, Counter macht den 1. + 2. Aufruf zu Konflikt-Treffern und den 3. zu Free-Day. Alternative wäre sequentielle mockall-Erwartungen via `times(1)` — Counter-Pattern ist lesbarer für Multi-Call-Differenzierung.
- **Keine Block-Reordering in main.rs nötig**: Existing Konstruktionsreihenfolge ist bereits Tier-konform (`absence_service` Z. 752 vor `shiftplan_edit_service` Z. 831 vor `shiftplan_view_service` Z. 856). Plan-Vorhersage stimmte.
- **Default-Mock für `get_all_user_assignments` in ShiftplanView-Tests**: Tests, die HR via `expect_check_permission().returning(|_, _| Ok(()))` granten, granten implizit auch SHIFTPLANNER (kein Privilege-Filter im Mock); der Body ruft dann `get_all_user_assignments`. Default `Ok(HashMap::new())` verhindert Mock-Panik. Alternative wäre per-test-Override — Default ist DRY.
- **Default-Mock für `verify_user_is_sales_person` → Forbidden in ShiftplanView-Tests**: HR-Or-Self läuft via `tokio::join!` — beide Probes WERDEN aufgerufen, auch wenn HR Ok ist. Default Forbidden + HR-Override-Tests funktionieren via `.or()`-Kurzschluss; forbidden-Tests setzen beide explizit.

## Deviations from Plan

None — Plan executed wie geschrieben. Eine kleine Adaption am Test-Setup für ShiftplanViewService: `default verify_user_is_sales_person` und `default get_all_user_assignments` mussten zu `build_dependencies` ergänzt werden, weil das Plan-File diese Defaults nicht spezifizierte und die Mock-Probes via `tokio::join!` parallel laufen. Diese Defaults sind reine Test-Infrastruktur, keine semantische Abweichung.

## Issues Encountered

**Pre-existing 8 absence_period-Integration-Test-Failures in shifty_bin** (out of scope per Plan-Scope-Boundary):

`shifty_bin/src/integration_test/absence_period.rs` (8 Tests) scheitern weiterhin auf der lokalen Dev-DB mit `no such table: absence_period` — pre-existing seit Phase 1, dokumentiert in `.planning/phases/02-.../deferred-items.md` und `.planning/phases/03-.../deferred-items.md`. Plan 03-04 verändert das Test-Bild NICHT (kein neuer Fehler).

**Workspace-Test-Status nach Plan 03-04:**
- `service_impl --lib` (gefiltert auf `test::shiftplan_edit`): 6 passed, 0 failed, 0 ignored — GRÜN (alle Plan-01-Stubs aktiviert)
- `service_impl --lib` (gefiltert auf `test::shiftplan`): 13 passed, 0 failed, 0 ignored — GRÜN (existing 9 + 4 neue)
- `service_impl --lib` (gesamt): **331 passed**, 0 failed, **0 ignored** (Plan-01-Stubs aktiviert!) — GRÜN, +10 vs. Plan-03-Baseline
- `shifty_bin` integration: 20 passed, 8 failed (pre-existing Phase-1-Migrations-Lücke), 4 ignored (Plan-01 Wave-5-Stubs warten auf Plan 03-06) — identisch zu Plan-03-Baseline
- `cargo build --workspace`: GRÜN
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- `timeout 12 cargo run`: scheitert beim Migration-Step mit `VersionMissing(20260428101456)` — pre-existing Migrations-Drift aus Phase 1 (deferred-items.md). DI-Konstruktion + Service-Initialisierung sind aber erfolgreich — kein DI-Cycle-Panic, kein OnceLock-Init-Error.

## Threat Flags

Keine neuen Threat-Surfaces eingeführt. Threat-Model des Plan-Files 1:1 abgebildet:

- **T-3-CrossSrc** (Information Disclosure via fremde absence_id leaken): mitigiert durch HR ∨ self-Permission auf `book_slot_with_conflict_check`. Verifiziert in `test_book_slot_with_conflict_check_forbidden`.
- **T-3-PerSP** (per-sales-person-Sicht zeigt fremde Verfügbarkeiten): mitigiert durch HR ∨ self-Permission auf `get_shiftplan_*_for_sales_person`. Verifiziert in `test_get_shiftplan_week_for_sales_person_forbidden`.
- **T-3-CycleDI** (versehentlicher AbsenceService-Dep auf BookingService): mitigiert durch D-Phase3-18 Regression-Lock-Verifikation `jj diff service/src/booking.rs service_impl/src/booking.rs rest/src/booking.rs service_impl/src/test/booking.rs` über kompletten Plan-03-04-Span = 0 Lines.
- **T-3-Boot-Cycle** (DI-Konstruktion mit zyklischer Reihenfolge → Panic at boot): mitigiert durch Boot-Smoke-Test (`timeout 12 cargo run`) — Boot scheitert NICHT an DI-Cycle, sondern an pre-existing Migrations-Drift (anderer Layer).

## Self-Check

- service/src/shiftplan_edit.rs enthält `pub struct BookingCreateResult` (grep-Count = 1)
- service/src/shiftplan_edit.rs enthält `pub struct CopyWeekResult` (grep-Count = 1)
- service/src/shiftplan_edit.rs enthält `book_slot_with_conflict_check` (grep-Count = 1 — Trait-Decl)
- service/src/shiftplan_edit.rs enthält `copy_week_with_conflict_check` (grep-Count = 1 — Trait-Decl)
- service_impl/src/shiftplan_edit.rs enthält `book_slot_with_conflict_check` (grep-Count = 1 — Impl)
- service_impl/src/shiftplan_edit.rs enthält `copy_week_with_conflict_check` (grep-Count = 1 — Impl, der ruft auch `book_slot_with_conflict_check` ein zweites mal aber Recursion via self.)
- service_impl/src/shiftplan.rs enthält `build_shiftplan_day_for_sales_person` (grep-Count = 4 — Helper-Decl + 3 Aufrufe)
- service_impl/src/shiftplan.rs enthält `UnavailabilityMarker::Both` (grep-Count = 1 — im Helper-Match)
- service/src/shiftplan.rs enthält `get_shiftplan_week_for_sales_person` + `get_shiftplan_day_for_sales_person` (grep-Count = 2)
- service_impl/src/shiftplan.rs enthält `get_shiftplan_week_for_sales_person` + `get_shiftplan_day_for_sales_person` (grep-Count = 2 — Impl-Sigs)
- shifty_bin/src/main.rs enthält `absence_service: absence_service.clone()` (grep-Count = 2 — 1 in shiftplan_edit + 1 in shiftplan_view)
- Konstruktionsreihenfolge: `let absence_service` (Z. 752) < `let shiftplan_edit_service` (Z. 831) < `let shiftplan_view_service` (Z. 856) — Tier-konform
- Commit `448c265e` (Task 1) FOUND in jj log
- Commit `92358f05` (Task 2) FOUND in jj log
- Commit `00ce6d87` (Task 3) FOUND in jj log
- `cargo build --workspace` exit 0
- `cargo test -p service_impl --lib`: 331 passed/0 failed/0 ignored
- `cargo test -p service_impl --lib test::shiftplan_edit`: 6 passed/0 failed/0 ignored
- `cargo test -p service_impl --lib test::shiftplan`: 13 passed/0 failed/0 ignored
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- `timeout 12 cargo run`: Service-DI-Init OK, scheitert an Migrations-Drift (pre-existing)
- **D-Phase3-18 Regression-Lock final**: `jj diff --from qzkyxvrl --to @ -- service/src/booking.rs service_impl/src/booking.rs rest/src/booking.rs service_impl/src/test/booking.rs` produziert 0 Diff-Lines über kompletten Plan-03-04-Span

## Self-Check: PASSED

## Next Phase Readiness

Plan 03-05 (Wave-4 REST-Layer) kann unmittelbar starten:

- `BookingCreateResult` + `CopyWeekResult` sind verfügbar — Plan 03-05 ergänzt `BookingCreateResultTO` + `CopyWeekResultTO`-Wrapper-DTOs in `rest-types/src/lib.rs`.
- `AbsencePeriodCreateResult` (aus Plan 03-03) wartet bereits auf seinen `AbsencePeriodCreateResultTO`-DTO + REST-Migration der `create_absence_period`/`update_absence_period`-Endpunkte (TODO Plan-05 in Plan 03-03 markiert).
- `get_shiftplan_*_for_sales_person` Service-Surfaces sind verfügbar — Plan 03-05 ergänzt entsprechende REST-Endpunkte.
- `Warning` enum + `UnavailabilityMarker` enum sind seit Plan 03-02 verfügbar — Plan 03-05 mappt sie in DTOs.
- 6 Plan-01-Wave-3-Stubs sind aktiviert und alle GRÜN — Wave 5 (Plan 03-06) muss nur noch die 4 Wave-5-Integration-Stubs in `shifty_bin/src/integration_test/booking_absence_conflict.rs` aktivieren.

**Wave-4-Forcing-State:** Plan 03-05 muss
1. 5 Wrapper-DTOs in `rest-types/src/lib.rs` ergänzen (`BookingCreateResultTO`, `CopyWeekResultTO`, `WarningTO`, `UnavailabilityMarkerTO`, `AbsencePeriodCreateResultTO`).
2. 4 neue Endpunkte in `rest/src/`: `POST /booking/with-conflict-check`, `POST /booking/copy-week-with-conflict-check`, `GET /shiftplan/.../sales-person/{id}/week/...`, `GET /shiftplan/.../sales-person/{id}/day/...`.
3. ApiDoc + Router-Wiring + utoipa-Annotations.
4. D-Phase3-18 Regression-Lock weiter respektieren — die NEUEN Endpunkte landen NICHT in `rest/src/booking.rs`, sondern in einer eigenen Datei (z.B. `rest/src/booking_with_conflict.rs`) oder in `rest/src/shiftplan_edit.rs` (existing).

**D-Phase3-18 Regression-Lock**: BookingService-Files (`service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs`) sind durch Plan 03-04 NICHT angetastet. Gilt weiter als Hard-Constraint für Plans 03-05..03-06.

---
*Phase: 03-booking-shift-plan-konflikt-integration*
*Plan: 04 (Wave 3 ShiftplanEditService Reverse-Warning + ShiftplanViewService per-sales-person + DI-Wiring)*
*Completed: 2026-05-02*
