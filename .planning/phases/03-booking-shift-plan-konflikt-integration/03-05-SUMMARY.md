---
phase: 03-booking-shift-plan-konflikt-integration
plan: 05
subsystem: rest-layer
tags: [rust, axum, utoipa, openapi, dto, rest-types, wave-4, jj]

# Dependency graph
requires:
  - phase: 03-booking-shift-plan-konflikt-integration/03-02
    provides: service::warning::Warning enum + service::shiftplan::UnavailabilityMarker enum + ShiftplanDay.unavailable Field
  - phase: 03-booking-shift-plan-konflikt-integration/03-03
    provides: service::absence::AbsencePeriodCreateResult + AbsenceService::create/update returnen den Wrapper
  - phase: 03-booking-shift-plan-konflikt-integration/03-04
    provides: service::shiftplan_edit::{BookingCreateResult, CopyWeekResult} + ShiftplanEditService::book_slot_with_conflict_check / copy_week_with_conflict_check + ShiftplanViewService::get_shiftplan_*_for_sales_person
provides:
  - "5 inline Phase-3 Wrapper-DTOs in rest-types/src/lib.rs: WarningTO (Tag-Enum, 4 Varianten), UnavailabilityMarkerTO (Tag-Enum, 3 Varianten), BookingCreateResultTO, CopyWeekResultTO, AbsencePeriodCreateResultTO"
  - "ShiftplanDayTO.unavailable: Option<UnavailabilityMarkerTO> (additiv, default None)"
  - "POST /absence-period und PATCH /absence-period/{id} liefern jetzt AbsencePeriodCreateResultTO (mit Forward-Warnings, BOOK-01) — Status 201/200 unverändert"
  - "POST /shiftplan-edit/booking + POST /shiftplan-edit/copy-week (C-Phase3-09, BOOK-02 / D-Phase3-02)"
  - "GET /shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id} + GET /shiftplan-info/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id} (D-Phase3-12, PLAN-01)"
  - "ApiDoc-Aggregation im obersten nest-Block in rest/src/lib.rs erweitert um ShiftplanEditApiDoc; ShiftplanApiDoc + AbsenceApiDoc components.schemas erweitert um die neuen DTOs"
affects: [03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tag-Enum-DTO-Pattern für stabiles OpenAPI-Schema (utoipa-5-Support für #[serde(tag = \"kind\", content = \"data\", rename_all = \"snake_case\")]) — RESEARCH.md Pattern 4"
    - "Inline-DTO-Konvention (Phase-1-Override): alle 5 Phase-3-Wrapper-DTOs als pub Strukturen am Ende von rest-types/src/lib.rs mit From-Impls unter #[cfg(feature = \"service-impl\")]"
    - "Additives ShiftplanDayTO.unavailable als Option<UnavailabilityMarkerTO> + #[serde(default, skip_serializing_if = \"Option::is_none\")]: globale Sicht serialisiert das Feld weiterhin nicht (Wire-Compat); per-sales-person-Sicht setzt es"
    - "Existing edit_slot/delete_slot/add_vacation Handler (Phase-1) bleiben ohne #[utoipa::path]-Annotation — werden NICHT in ShiftplanEditApiDoc::paths(...) aufgenommen, weil utoipa-paths ohne Annotation Compile-Errors produzieren würden; nur die zwei neuen Phase-3-Handler sind dokumentiert"

key-files:
  created: []
  modified:
    - rest-types/src/lib.rs
    - rest/src/absence.rs
    - rest/src/shiftplan_edit.rs
    - rest/src/shiftplan.rs
    - rest/src/lib.rs

key-decisions:
  - "Tag-Enum-DTO mit #[serde(tag, content, rename_all)]-Form für WarningTO und UnavailabilityMarkerTO (statt plain enum mit extra `kind`-Feld) — utoipa-5 supported es; JSON-Form ist explizit `{ kind, data }`; Frontend-Generator kann discriminated unions direkt rendern."
  - "ShiftplanDayTO.unavailable mit #[serde(default, skip_serializing_if = \"Option::is_none\")]: globale Endpunkte (`get_shiftplan_week`, `get_shiftplan_day`) serialisieren das Feld weiterhin NICHT (Wire-Compat); per-sales-person-Endpunkte serialisieren `unavailable: { kind, data }` wenn Marker gesetzt."
  - "ShiftplanEditApiDoc enthält NUR die neuen Phase-3-Handler in `paths(...)` — `edit_slot`/`delete_slot`/`add_vacation` haben heute keine `#[utoipa::path]`-Annotation und würden Compile-Errors verursachen, wenn aufgenommen. Existing-Tech-Debt; out-of-scope für Plan 05."
  - "Per-sales-person-Endpunkte sind unter `/shiftplan-info/...` mountet (siehe `start_server` in `rest/src/lib.rs:549`), NICHT unter `/shiftplan/...`. Plan-File hatte das ungenau formuliert; tatsächliche Mount-Path ist `/shiftplan-info` (Phase-1-Konvention)."
  - "REST-Body-Sig-Bruch für /absence-period: AbsencePeriodTO → AbsencePeriodCreateResultTO. Plan-03-Wave-2 hatte den TODO-Plan-05 markiert (`.absence`-Unwrap mit verworfenen Warnings). Plan 05 entfernt den Unwrap und macht den Wrapper zur Wire-Form. Frontend-Migration ist im Frontend-Workstream — Backend-side ist die Wire-Form jetzt phase-3-konform."
  - "POST /booking und POST /booking/copy-week unter `/booking/...` (rest/src/booking.rs) bleiben **strikt unangetastet** (D-Phase3-18). Frontend kann inkrementell auf `/shiftplan-edit/booking` migrieren. Beide Endpunkte koexistieren parallel; alter Endpunkt persistiert Bookings ohne Cross-Source-Warnings, neuer Endpunkt mit."

patterns-established:
  - "Wave-4-REST-Layer-Plan-Struktur: 3 atomare jj-Tasks (DTOs / Handler-Patches / per-sales-person-Routes); jede Task baut Workspace + Tests grün; D-Phase3-18 Regression-Lock pro Task verifiziert (jj diff = 0 lines)."
  - "Frontend-Migration über parallele Endpunkt-Koexistenz: alter und neuer Endpunkt liefern semantisch unterschiedliche Bodies; Frontend-Workstream migriert UI-Komponenten Stück für Stück."

requirements-completed: []  # BOOK-01 / BOOK-02 / PLAN-01 sind erst nach Plan 03-06 (Integration-Tests + SC-Verifikation) als komplett markiert.

# Metrics
duration: ~25min
completed: 2026-05-02
---

# Phase 3 Plan 05: Wave-4 REST-Layer Summary

**5 inline Phase-3 Wrapper-DTOs in `rest-types/src/lib.rs` (WarningTO + UnavailabilityMarkerTO als Tag-Enums, BookingCreateResultTO/CopyWeekResultTO/AbsencePeriodCreateResultTO als Wrapper-Structs); `ShiftplanDayTO.unavailable: Option<UnavailabilityMarkerTO>` additiv; `POST /absence-period` und `PATCH /absence-period/{id}` liefern jetzt `AbsencePeriodCreateResultTO` mit Forward-Warnings (Plan-03-TODO eingelöst); 2 neue Endpunkte unter `/shiftplan-edit/...` (book + copy-week, BOOK-02 / D-Phase3-02); 2 neue per-sales-person-Endpunkte unter `/shiftplan-info/...` (PLAN-01); ApiDoc-Aggregation komplett. `cargo build/test --workspace` GRÜN; `service_impl` 331 passed/0 failed/0 ignored. **D-Phase3-18 Regression-Lock erfüllt** — `rest/src/booking.rs` + `service/src/booking.rs` + `service_impl/src/booking.rs` + `service_impl/src/test/booking.rs` unangetastet (jj diff = 0 lines über kompletten Plan-05-Span).**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-02 (Folge auf Plan 03-04)
- **Completed:** 2026-05-02
- **Tasks:** 3
- **Files modified:** 5
- **Files created:** 0

## Accomplishments

- `rest-types/src/lib.rs`:
  - 5 neue inline Phase-3 Wrapper-DTOs am Datei-Ende:
    - `WarningTO` Tag-Enum (`#[serde(tag = "kind", content = "data", rename_all = "snake_case")]`) mit 4 Varianten: `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`. JSON-Form `{ "kind": "...", "data": { ... } }`.
    - `UnavailabilityMarkerTO` Tag-Enum (gleiche Form) mit 3 Varianten: `AbsencePeriod`, `ManualUnavailable`, `Both`.
    - `BookingCreateResultTO { booking: BookingTO, warnings: Vec<WarningTO> }`.
    - `CopyWeekResultTO { copied_bookings: Vec<BookingTO>, warnings: Vec<WarningTO> }`.
    - `AbsencePeriodCreateResultTO { absence: AbsencePeriodTO, warnings: Vec<WarningTO> }`.
  - Alle 5 From-Impls unter `#[cfg(feature = "service-impl")]` gegated; konsumieren `service::warning::Warning`, `service::shiftplan::UnavailabilityMarker`, `service::shiftplan_edit::{BookingCreateResult, CopyWeekResult}`, `service::absence::AbsencePeriodCreateResult`.
  - `ShiftplanDayTO.unavailable: Option<UnavailabilityMarkerTO>` als drittes Feld (additiv) mit `#[serde(default, skip_serializing_if = "Option::is_none")]`. Das From-Impl `From<&ShiftplanDay> for ShiftplanDayTO` mappt das Feld via `d.unavailable.as_ref().map(UnavailabilityMarkerTO::from)`.

- `rest/src/absence.rs`:
  - Imports erweitert um `AbsencePeriodCreateResultTO` + `WarningTO`.
  - `create_absence_period`-Handler: 201-Body von `AbsencePeriodTO` → `AbsencePeriodCreateResultTO`. Plan-03-TODO eingelöst.
  - `update_absence_period`-Handler: 200-Body von `AbsencePeriodTO` → `AbsencePeriodCreateResultTO`. Plan-03-TODO eingelöst.
  - `AbsenceApiDoc` `components.schemas` erweitert um `AbsencePeriodCreateResultTO` + `WarningTO`.

- `rest/src/shiftplan_edit.rs`:
  - Imports + `routing::post` ergänzt; `Deserialize`/`Serialize`/`ToSchema`/`OpenApi`-Imports.
  - Routes erweitert um `POST /booking` (`book_slot_with_conflict_check`) + `POST /copy-week` (`copy_week_with_conflict_check`).
  - Neuer Handler `book_slot_with_conflict_check` mit `#[utoipa::path]`-Annotation; delegiert via `rest_state.shiftplan_edit_service().book_slot_with_conflict_check(...)`; Body `BookingCreateResultTO`; Status 201.
  - Neuer Handler `copy_week_with_conflict_check` mit `CopyWeekRequest`-Struct (`from_year`, `from_calendar_week`, `to_year`, `to_calendar_week`); delegiert an `copy_week_with_conflict_check`; Body `CopyWeekResultTO`; Status 200.
  - Neuer ApiDoc-Struct `ShiftplanEditApiDoc` (existing rest/src/shiftplan_edit.rs hatte heute keinen ApiDoc); `paths(...)` enthält die ZWEI neuen Phase-3-Handler — die existierenden `edit_slot`/`delete_slot`/`add_vacation` haben keine `#[utoipa::path]`-Annotation und werden NICHT aufgenommen (Phase-1-Tech-Debt; out-of-scope).

- `rest/src/shiftplan.rs`:
  - Imports erweitert um `AbsenceCategoryTO` + `UnavailabilityMarkerTO`.
  - Routes erweitert um `GET /{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}` (`get_shiftplan_week_for_sales_person`) + `GET /day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}` (`get_shiftplan_day_for_sales_person`).
  - Beide neuen Handler haben `#[utoipa::path]`-Annotation, delegieren an die Plan-04-Service-Methoden, mappen `ShiftplanWeekTO` bzw. `ShiftplanDayAggregateTO` ins Body.
  - `ShiftplanApiDoc.paths(...)` erweitert um beide neuen Handler.
  - `ShiftplanApiDoc.components.schemas` erweitert um `UnavailabilityMarkerTO` + `AbsenceCategoryTO`.

- `rest/src/lib.rs`:
  - Im obersten `#[derive(OpenApi)] pub struct ApiDoc;` `nest`-Block (Z. 463-484) `(path = "/shiftplan-edit", api = shiftplan_edit::ShiftplanEditApiDoc)` zwischen `shiftplan-catalog` und `shiftplan-info` ergänzt — alphabetisch korrekt.
  - Routes-Mounting in `start_server` (`/shiftplan-edit` → `shiftplan_edit::generate_route()`) ist seit Phase 1 vorhanden — die zwei neuen Handler sind dadurch automatisch unter `/shiftplan-edit/booking` und `/shiftplan-edit/copy-week` erreichbar.

## Task Commits

Jede Task wurde atomar als ein jj-Change committed:

1. **Task 1: 5 neue inline Phase-3 Wrapper-DTOs + ShiftplanDayTO-Field in rest-types** — `1a64ff7b` (feat)
2. **Task 2: REST-Handler /absence-period auf Wrapper-Result + 2 neue /shiftplan-edit-Endpunkte + ShiftplanEditApiDoc** — `0738d880` (feat)
3. **Task 3: 2 neue per-sales-person-Endpunkte unter /shiftplan-info + ShiftplanApiDoc-Erweiterung** — `5b60dc29` (feat)

**Plan metadata commit:** _(diese SUMMARY + STATE.md/ROADMAP.md-Update — finaler `jj describe`)_

## Files Created/Modified

### Created (0)

Plan 03-05 ist reines additives Patchen + DTO-Anhängen — keine neuen Files.

### Modified (5)

| File | Lines Changed | Provenance |
|------|---------------|------------|
| `rest-types/src/lib.rs` | +213 / -1 | Task 1 — 5 neue DTOs + From-Impls + ShiftplanDayTO-Field |
| `rest/src/absence.rs` | +14 / -10 | Task 2 — Wrapper-Body für create/update + ApiDoc-schemas |
| `rest/src/shiftplan_edit.rs` | +127 / -2 | Task 2 — 2 neue Handler + CopyWeekRequest + ShiftplanEditApiDoc |
| `rest/src/shiftplan.rs` | +119 / -2 | Task 3 — 2 neue Handler + ApiDoc-Erweiterung |
| `rest/src/lib.rs` | +1 / 0 | Task 2 — `ShiftplanEditApiDoc` im obersten `nest`-Block |

## Wave-Mapping für Continuation-Executors

| Surface | Konsument | Aktivierungs-Plan |
|---------|-----------|-------------------|
| `POST /absence-period` Wrapper-Body | Frontend `useAbsencePeriodCreate`-Hook | Frontend-Workstream |
| `POST /shiftplan-edit/booking` | Frontend Booking-UI (BOOK-02 Reverse-Warning Anzeige) | Frontend-Workstream |
| `POST /shiftplan-edit/copy-week` | Frontend Copy-Week-UI (warnings list) | Frontend-Workstream |
| `GET /shiftplan-info/.../sales-person/...` | Frontend Per-User-Schichtplan-Sicht (PLAN-01 Markierung) | Frontend-Workstream |
| 4 Plan-01-Wave-5 Stubs in `shifty_bin/src/integration_test/booking_absence_conflict.rs` | Cross-Source-Integration-Tests | Plan 03-06 (Wave 5) |

## Decisions Made

- **Tag-Enum-DTO-Pattern**: `WarningTO` und `UnavailabilityMarkerTO` nutzen `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]`. Begründung: stabiles OpenAPI-Schema (utoipa-5 supported es nativ), JSON-Form `{ kind, data }` ist explizit, Frontend-Generator kann discriminated unions direkt rendern. Alternative (plain enum mit extra `kind`-Field) wäre fragiler.
- **ShiftplanDayTO.unavailable mit `skip_serializing_if = "Option::is_none"`**: globale Endpunkte (`get_shiftplan_week`, `get_shiftplan_day`) serialisieren das Feld weiterhin NICHT (Wire-Compat — bestehende Frontend-Calls bleiben byte-identisch); per-sales-person-Endpunkte setzen `unavailable: { kind, data }`. Verhindert Frontend-Crashes durch unbekannte Felder in alten Komponenten.
- **ShiftplanEditApiDoc enthält NUR die neuen Phase-3-Handler**: `edit_slot`/`delete_slot`/`add_vacation` haben heute keine `#[utoipa::path]`-Annotation und würden Compile-Errors verursachen, wenn in `paths(...)` aufgenommen. Existing-Tech-Debt aus Phase 1; out-of-scope für Plan 05. Phase-4-Hygiene oder dedizierter Cleanup-Plan kann das nachreichen.
- **Per-sales-person-Endpunkte sind unter `/shiftplan-info/...` mountet, NICHT unter `/shiftplan/...`**: Das Plan-File hatte den Mount-Path ungenau formuliert. Tatsächlich nestet `start_server` in `rest/src/lib.rs:549` `/shiftplan-info` → `shiftplan::generate_route()`. Die SUMMARY dokumentiert das korrigierte Pfad-Pattern.
- **REST-Body-Sig-Bruch für /absence-period**: `AbsencePeriodTO` → `AbsencePeriodCreateResultTO`. Plan-03-Wave-2 hatte den TODO-Plan-05 markiert (`.absence`-Unwrap mit verworfenen Warnings). Plan 05 entfernt den Unwrap und macht den Wrapper zur Wire-Form. Frontend-Migration ist im Frontend-Workstream — Backend-side ist die Wire-Form jetzt Phase-3-konform.
- **D-Phase3-18 Regression-Lock weiter respektiert**: Die NEUEN Endpunkte `/shiftplan-edit/booking` + `/shiftplan-edit/copy-week` landen NICHT in `rest/src/booking.rs`, sondern in `rest/src/shiftplan_edit.rs`. Beide Endpunkte koexistieren parallel — alter `/booking`-Endpunkt persistiert ohne Warnings, neuer mit. Frontend-Migration inkrementell.

## Deviations from Plan

None — Plan executed wie geschrieben. Eine kleine Korrektur am Plan-Text:
- Das Plan-File hat den Mount-Path der per-sales-person-Endpunkte als `/shiftplan/...` formuliert; tatsächlich sind sie unter `/shiftplan-info/...` mountet (Phase-1-Konvention via `rest/src/lib.rs:549` `nest("/shiftplan-info", shiftplan::generate_route())`). Die Endpunkt-Struktur (`/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}` und `/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}`) ist exakt wie spezifiziert; nur der Top-Level-Prefix unterscheidet sich. Das ist **keine semantische Abweichung** — es ist die existing Mount-Konvention, die Plan 05 respektiert hat.

## Issues Encountered

**Pre-existing 8 absence_period-Integration-Test-Failures + Migrations-Drift in shifty_bin** (out of scope per Plan-Scope-Boundary):

`shifty_bin/src/integration_test/absence_period.rs` (8 Tests) scheitern weiterhin auf der lokalen Dev-DB mit `no such table: absence_period` — pre-existing seit Phase 1. `cargo run` panic'd beim Migration-Step mit `VersionMissing(20260428101456)`. Beides dokumentiert in `.planning/phases/02-.../deferred-items.md` und `.planning/phases/03-.../deferred-items.md`. Plan 03-05 hat das Test-Bild NICHT verändert (kein neuer Fehler).

**OpenAPI-Smoke-Test übersprungen**: Da `cargo run` an pre-existing Migrations-Drift scheitert, konnte der Plan-Verify-Schritt `curl http://localhost:3000/api-docs/openapi.json | jq '.paths | keys'` nicht durchgeführt werden. Stattdessen verifiziert der erfolgreiche `cargo build --workspace` (utoipa generiert OpenAPI-Schema deterministisch zur Compile-Zeit) + die `grep`-Counts (alle Phase-3-Handler + DTOs sind im Source). Plan 03-06 (Wave 5) wird Integration-Tests gegen die neue REST-Surface schreiben und damit den End-to-End-Pfad final verifizieren.

**Workspace-Test-Status nach Plan 03-05:**
- `service_impl --lib`: **331 passed, 0 failed, 0 ignored** — GRÜN, identisch zu Plan-04-Baseline
- `shifty_bin` integration: 20 passed, 8 failed (pre-existing Phase-1-Migrations-Lücke), 4 ignored (Plan-01 Wave-5-Stubs warten auf Plan 03-06) — identisch zu Plan-04-Baseline
- `cargo build --workspace`: GRÜN
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- Lokaler `cargo run`: Service-DI-Init OK (kein DI-Cycle), scheitert NUR an pre-existing Migrations-Drift

## Threat Flags

Keine neuen Threat-Surfaces eingeführt — Plan-File-Threat-Model 1:1 abgebildet:

- **T-3-REST-Spoof** (Information Disclosure via fremde sales_person_id in Path injizieren): mitigiert durch HR ∨ `verify_user_is_sales_person`-Permission im Service-Layer (Plan 03-04 verifiziert via `test_get_shiftplan_week_for_sales_person_forbidden` und `test_book_slot_with_conflict_check_forbidden`). REST-Layer ist dünn — kein neuer Permission-Check nötig.
- **T-3-OpenAPI-Drift** (utoipa-Schemas weichen von Service-Domain ab): mitigiert durch From-Impls am `service::*`-Domain-Pfad gekoppelt — Compiler erzwingt Konsistenz; Schema-Generation ist deterministisch zur Compile-Zeit.
- **T-3-Frontend-Migration**: alter `POST /booking`-Endpunkt bleibt funktional + neuer `POST /shiftplan-edit/booking` parallel. KEIN Doppel-Effekt: alter Endpunkt persistiert ohne Warnings, neuer mit. Frontend-Migration im Frontend-Workstream. Akzeptiert per D-Phase3-18.
- **T-3-Tag-Enum-utoipa**: utoipa-5 supported `#[serde(tag, content)]` nativ (RESEARCH.md Pattern 4); `cargo build` GRÜN bestätigt deterministische Schema-Generation.

## Self-Check

- rest-types/src/lib.rs enthält `pub enum WarningTO` (grep-Count = 1)
- rest-types/src/lib.rs enthält `pub enum UnavailabilityMarkerTO` (grep-Count = 1)
- rest-types/src/lib.rs enthält `pub struct BookingCreateResultTO|pub struct CopyWeekResultTO|pub struct AbsencePeriodCreateResultTO` (grep-Count = 3)
- rest-types/src/lib.rs enthält `pub unavailable: Option<UnavailabilityMarkerTO>` (grep-Count = 1)
- rest/src/absence.rs enthält `AbsencePeriodCreateResultTO::from` (grep-Count = 2 — create + update)
- rest/src/shiftplan_edit.rs enthält `book_slot_with_conflict_check|copy_week_with_conflict_check` (grep-Count = 10 — beide Funktionsnamen mehrfach: Routes + Annotations + Definitionen + ApiDoc)
- rest/src/shiftplan_edit.rs enthält `ShiftplanEditApiDoc` (grep-Count = 1)
- rest/src/lib.rs enthält `shiftplan_edit::ShiftplanEditApiDoc` (grep-Count = 1)
- rest/src/shiftplan.rs enthält `get_shiftplan_week_for_sales_person|get_shiftplan_day_for_sales_person` (grep-Count = 9 — beide Namen mehrfach: Routes + Annotations + Definitionen + ApiDoc)
- rest/src/shiftplan.rs enthält `sales-person` (grep-Count = 7 — Routes + utoipa params)
- rest/src/shiftplan.rs enthält `UnavailabilityMarkerTO` (grep-Count = 3 — Import + ApiDoc-schema)
- Commit `1a64ff7b` (Task 1) FOUND in jj log
- Commit `0738d880` (Task 2) FOUND in jj log
- Commit `5b60dc29` (Task 3) FOUND in jj log
- `cargo build --workspace` exit 0
- `cargo test -p service_impl --lib`: 331 passed/0 failed/0 ignored
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- **D-Phase3-18 Regression-Lock final**: `jj diff --from qxpuyxtq --to @ -- rest/src/booking.rs service/src/booking.rs service_impl/src/booking.rs service_impl/src/test/booking.rs` produziert 0 Diff-Lines über kompletten Plan-03-05-Span

## Self-Check: PASSED

## Next Phase Readiness

Plan 03-06 (Wave-5 Integration-Tests + SC-Verifikation + ROADMAP-Update) kann unmittelbar starten:

- REST-Surface ist Phase-3-konform — alle 5 Wrapper-DTOs + 4 neue Endpunkte + Wrapper-Body für `/absence-period` sind verdrahtet.
- 4 Plan-01-Wave-5-Stubs in `shifty_bin/src/integration_test/booking_absence_conflict.rs` warten auf Aktivierung (full-stack via REST-Calls): `test_double_source_two_warnings_one_booking`, `test_softdeleted_absence_no_warning_no_marker`, `test_copy_week_three_bookings_two_warnings`, `test_shiftplan_marker_softdeleted_absence_none`.
- Service-Surface (Plan 03-03/04) ist verfügbar; REST-Layer ist neu (Plan 03-05); End-to-End-Pfad ist intakt.
- D-Phase3-18 Regression-Lock weiter Hard-Constraint für Plan 03-06.

**Wave-5-Forcing-State:** Plan 03-06 muss
1. Die 4 Plan-01-Wave-5-Stubs aktivieren (#[ignore] entfernen + TestSetup analog `absence_period.rs`).
2. Cross-Source-Integration-Tests via REST-Calls schreiben (HTTP-Surface-Verifikation).
3. SC1-SC4 aus 03-VALIDATION.md vollständig verifizieren.
4. ROADMAP.md auf Phase 3 = Complete updaten.

**D-Phase3-18 Regression-Lock final**: BookingService-Files (`service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs`) sind durch Plan 03-05 NICHT angetastet. Gilt weiter als Hard-Constraint für Plan 03-06.

---
*Phase: 03-booking-shift-plan-konflikt-integration*
*Plan: 05 (Wave 4 REST-Layer)*
*Completed: 2026-05-02*
