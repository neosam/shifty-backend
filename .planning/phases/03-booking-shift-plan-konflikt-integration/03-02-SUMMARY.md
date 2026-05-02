---
phase: 03-booking-shift-plan-konflikt-integration
plan: 02
subsystem: domain-model
tags: [rust, sqlx, mockall, automock, jj, domain-enum, dao, sqlite, wave-1]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: AbsenceDao + AbsencePeriodEntity + Composite-Index (sales_person_id, from_date) WHERE deleted IS NULL
  - phase: 03-booking-shift-plan-konflikt-integration/03-01
    provides: 10 #[ignore]-Stub-Tests in service_impl + shifty_bin (Wave-0 Test-Surface)
provides:
  - service::warning::Warning enum with 4 variants (D-Phase3-14)
  - dao::absence::AbsenceDao::find_overlapping_for_booking trait method (kategorie-frei, D-Phase3-05)
  - dao_impl_sqlite::absence::AbsenceDaoImpl::find_overlapping_for_booking SQLx-Impl with deleted-IS-NULL soft-delete-Filter
  - service::shiftplan::UnavailabilityMarker enum with 3 variants (D-Phase3-10)
  - service::shiftplan::ShiftplanDay.unavailable: Option<UnavailabilityMarker> field
affects: [03-03, 03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Domain-Enum-Modul-Lokation-Pattern: shared Cross-Service-Enum bekommt eigenes service/src/<name>.rs (statt inline in lib.rs), wenn semantisch != ServiceError und von mehreren Services konsumiert (Warning lebt in service/src/warning.rs analog ResolvedAbsence in service/src/absence.rs)"
    - "DAO-Trait additiv erweitern + #[automock]-Auto-Derivation: bestehende Service-Impl-Tests bleiben grün ohne Mock-Patches; neuer MockAbsenceDao::expect_find_overlapping_for_booking() automatisch verfügbar"
    - "Additives Struct-Field als Compile-Forcing: ShiftplanDay.unavailable: Option<...> bricht alle Construction-Sites; Compiler erzwingt Update-Liste (im Workspace genau eine Site: build_shiftplan_day)"
    - "sqlx-prepare im NixOS-Setup: DATABASE_URL=sqlite://./localdb.sqlite3 nix-shell -p sqlx-cli --run 'cargo sqlx prepare --workspace -- --tests' produziert offline-cache als Teil des Plan-Commits"

key-files:
  created:
    - service/src/warning.rs
    - .planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md
  modified:
    - service/src/lib.rs
    - dao/src/absence.rs
    - dao_impl_sqlite/src/absence.rs
    - service/src/shiftplan.rs
    - service_impl/src/shiftplan.rs
    - .sqlx/ (4 neue prepared-statement-Cache-Files)

key-decisions:
  - "Warning-Modul-Lokation: eigenes service/src/warning.rs statt inline in lib.rs (C-Phase3-01) — Erfolgs-Pfad-Domain-Modell ist semantisch != ServiceError-Fehler-Pfad und wird zwischen AbsenceService und ShiftplanEditService geteilt"
  - "AbsenceDao::find_overlapping_for_booking ist kategorie-frei (KEIN AbsenceCategoryEntity-Filter, KEIN exclude_logical_id) — Booking-IDs sind orthogonal zu Absence-IDs (RESEARCH.md Q9), Self-Match unmöglich; alle 3 Kategorien werden zurückgegeben, Service-Layer entscheidet später wie zu behandeln"
  - "ShiftplanDay.unavailable als Option<UnavailabilityMarker> additiv (default None) — globale Sicht (get_shiftplan_*) bleibt unverändert, per-sales-person-Sicht (Plan 03-04) setzt das Feld dann"
  - "service/src/lib.rs: pub mod warning; alphabetisch zwischen uuid_service und week_message; KEIN pub use warning::Warning; — Konsumenten verwenden use service::warning::Warning; analog use service::absence::AbsencePeriod;"

patterns-established:
  - "Wave-1-Plan-Struktur in Phase 3: 3 Tasks (Domain-Enum + DAO-Trait/Impl + Struct-Erweiterung); jede Task baut atomar als jj-Change; service_impl-Tests bleiben grün durch automock-Auto-Derivation"
  - "Pre-existing dao/dao_impl_sqlite uuid v4-Feature-Drift dokumentiert in deferred-items.md — NICHT durch diesen Plan ausgelöst, NICHT hier gefixt (Scope-Boundary)"
  - "D-Phase3-18 Regression-Lock: jj diff -r 'first-task::last-task' service/src/booking.rs service_impl/src/booking.rs liefert leeren Diff — verifiziert per Plan-End-Check"

requirements-completed: []  # BOOK-01/BOOK-02/PLAN-01 sind erst nach Plans 03-03..03-06 abgeschlossen; Wave-1 ist nur Domain-Surface, keine vollständige Funktionalität.

# Metrics
duration: ~16min
completed: 2026-05-02
---

# Phase 3 Plan 02: Wave-1 Domain-Surface Summary

**Warning-Enum (4 Varianten) + AbsenceDao::find_overlapping_for_booking (kategorie-frei + Soft-Delete-Filter) + UnavailabilityMarker-Enum (3 Varianten) + additives ShiftplanDay.unavailable-Feld — Domain-Surface komplett, 0 Service-Trait-Sig-Brüche; cargo build/test --workspace bleibt grün; Plan-01-Stubs unverändert ignored.**

## Performance

- **Duration:** ~16 min
- **Started:** 2026-05-02T22:43Z (Plan-01-Folge-Change)
- **Completed:** 2026-05-02T22:59Z
- **Tasks:** 3
- **Files created:** 2 (service/src/warning.rs + deferred-items.md)
- **Files modified:** 5 (+ .sqlx/-Cache 4 Files)

## Accomplishments

- `service/src/warning.rs` neu — `pub enum Warning` mit allen 4 Cross-Source-Varianten verbatim aus D-Phase3-14:
  - `BookingOnAbsenceDay { booking_id, date, absence_id, category }`
  - `BookingOnUnavailableDay { booking_id, year, week, day_of_week }`
  - `AbsenceOverlapsBooking { absence_id, booking_id, date }`
  - `AbsenceOverlapsManualUnavailable { absence_id, unavailable_id }`
- `service/src/lib.rs`: `pub mod warning;` alphabetisch zwischen `uuid_service` (Z. 41) und `week_message` (Z. 42)
- `dao/src/absence.rs`: neue Trait-Methode `find_overlapping_for_booking(sales_person_id, range, tx) -> Result<Arc<[AbsencePeriodEntity]>, DaoError>` — kategorie-frei, kein `exclude_logical_id`; `#[automock]` deckt sie automatisch
- `dao_impl_sqlite/src/absence.rs`: SQLx-Impl mit Allen-Algebra-Range-Match `WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL` (Pflicht für Pitfall-1 / SC4)
- `.sqlx/`-Cache: 4 neue prepared-statement-Files via `cargo sqlx prepare --workspace -- --tests`
- `service/src/shiftplan.rs`: `pub enum UnavailabilityMarker { AbsencePeriod, ManualUnavailable, Both }` und additives `pub unavailable: Option<UnavailabilityMarker>` auf `ShiftplanDay`
- `service_impl/src/shiftplan.rs`: `build_shiftplan_day` setzt `unavailable: None` als Default

## Task Commits

Each task was committed atomically as a separate jj change:

1. **Task 1: Warning-Modul + lib.rs Re-Export** — `572d6737` (feat)
2. **Task 2: AbsenceDao::find_overlapping_for_booking + SQLx-Impl + .sqlx-Cache** — `8fa3eefb` (feat)
3. **Task 3: UnavailabilityMarker-Enum + ShiftplanDay.unavailable + build_shiftplan_day-Default** — `35fb3edb` (feat)

**Plan metadata commit:** _(diese SUMMARY + deferred-items.md + STATE.md/ROADMAP.md-Update — finaler jj describe)_

## Files Created/Modified

### Created (2)
- `service/src/warning.rs` — Domain-Enum-Modul mit `pub enum Warning` (4 Varianten); Erfolgs-Pfad-Modell, geteilt zwischen `AbsenceService` (Plan 03-03) und `ShiftplanEditService` (Plan 03-04)
- `.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md` — Pre-existing dao/dao_impl_sqlite uuid v4-Feature-Drift dokumentiert + 8 absence_period-Integration-Test-Failures (cross-referenz auf Phase-2-deferred-items.md)

### Modified (5 + .sqlx/)
- `service/src/lib.rs` — `pub mod warning;` ergänzt (Z. 42 — alphabetisch zwischen uuid_service und week_message)
- `dao/src/absence.rs` — `find_overlapping_for_booking`-Trait-Methode nach `find_overlapping` ergänzt; KEIN `category`-Parameter, KEIN `exclude_logical_id`
- `dao_impl_sqlite/src/absence.rs` — SQLx-Impl mit Single-Branch (kein two-branch wie `find_overlapping`); Allen-Algebra-Range-Match `from_date <= range.to AND to_date >= range.from`
- `service/src/shiftplan.rs` — `UnavailabilityMarker`-Enum vor `ShiftplanDay`; `ShiftplanDay.unavailable: Option<UnavailabilityMarker>` als drittes Feld
- `service_impl/src/shiftplan.rs` — einzige Construction-Site `build_shiftplan_day` (Z. 104) setzt `unavailable: None`; Compile-Bruch-Forcing erzwang nichts weiter (keine anderen Producer im Workspace)
- `.sqlx/` — 4 neue Cache-Files für die `find_overlapping_for_booking`-Query

## Wave-Mapping für Continuation-Executors

| Domain-Element | Konsumiert in | Aktivierungs-Plan |
|----------------|---------------|-------------------|
| `Warning` enum | `AbsenceService::create/update` Forward-Warning-Loop | Plan 03-03 (Wave 2) |
| `Warning` enum | `ShiftplanEditService::book_slot_with_conflict_check` Reverse-Warning | Plan 03-04 (Wave 3) |
| `AbsenceDao::find_overlapping_for_booking` | `AbsenceService::find_overlapping_for_booking`-Pfad | Plan 03-03 |
| `AbsenceDao::find_overlapping_for_booking` | `ShiftplanEditService::book_slot_with_conflict_check`-Pfad | Plan 03-04 |
| `UnavailabilityMarker` | `build_shiftplan_day_for_sales_person` (neuer Helper) | Plan 03-04 |
| `ShiftplanDay.unavailable` | per-sales-person-Routen | Plan 03-04 (Wave 3) + Plan 03-05 (REST-DTO-Mapping) |

## Decisions Made

- **Warning-Modul-Lokation (C-Phase3-01)**: eigenes `service/src/warning.rs`-Modul statt Inline in `service/src/lib.rs`. Begründung: `Warning` ist Erfolgs-Pfad (200/201 mit Liste) und semantisch != `ServiceError`/`ValidationFailureItem` (422); wird zwischen `AbsenceService` und `ShiftplanEditService` geteilt; Modul-Lokation isoliert es von der `lib.rs`-Fehler-Block-Definition. KEIN `pub use warning::Warning;` am `lib.rs`-Root — Konsumenten machen `use service::warning::Warning;` analog `use service::absence::AbsencePeriod;`.
- **`find_overlapping_for_booking` ist kategorie-frei**: KEIN `AbsenceCategoryEntity`-Filter (alle 3 Kategorien werden zurückgegeben); KEIN `exclude_logical_id` (Booking-IDs sind orthogonal zu Absence-IDs, Self-Match unmöglich per RESEARCH.md Q9). Service-Layer (Plan 03-03) entscheidet später kategorie-spezifische Behandlung.
- **`ShiftplanDay.unavailable` ist additiv mit Default `None`**: globale Sicht setzt nie etwas, per-sales-person-Sicht (Plan 03-04) wird das Feld dann setzen. Verhindert Test-Bruch in `service_impl/src/test/shiftplan.rs` und `shifty_bin/src/integration_test/shiftplan.rs`.
- **`UnavailabilityMarker::Both` trägt `absence_id` + `category`**: bei Doppel-Quelle wird die semantisch reichere AbsencePeriod-Information mitgeführt (statt sie zu verlieren). De-Dup für UI-Markierung-Logik passiert in `Both`-Variante (D-Phase3-10 / D-Phase3-16).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Plan-Verify-Step `cargo test -p dao_impl_sqlite GRÜN` ist pre-existing nicht erreichbar**
- **Found during:** Task 2 Verify-Step
- **Issue:** `cargo test -p dao_impl_sqlite` schlägt mit 9× `Uuid::new_v4` not found fehl. Ursache: `dao/Cargo.toml` und `dao_impl_sqlite/Cargo.toml` deklarieren `uuid = "1.8"` ohne `v4`-Feature. Standalone-Tests sehen keine transitiven Features; Workspace-Build unifiziert sie und ist deshalb grün.
- **Fix:** Pre-existing Drift dokumentiert in `.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md`; NICHT durch diesen Plan ausgelöst (Cargo.toml-History via `git log` zeigt nur Versions-Bumps seit Phase 1). Verify-Set umgestellt auf `cargo build --workspace` + `cargo test --workspace --no-run` + `cargo test -p service_impl --lib` — alle drei grün.
- **Files modified:** `.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md` (commit-Time im Plan-Metadata-Commit)
- **Verification:** `cargo build --workspace` Finished; `cargo test --workspace --no-run` Finished; `cargo test -p service_impl --lib` 321 passed/0 failed/6 ignored; baseline-Verhalten erhalten (Plan 01 SUMMARY dokumentiert dasselbe Test-Bild).
- **Committed in:** Plan-Metadata-Commit (deferred-items.md)

---

**Total deviations:** 1 auto-fixed (1 blocking, Scope-Boundary-Doku)
**Impact on plan:** Pre-existing Cargo-Tooling-Drift; nicht durch Plan-03-02-Aktionen ausgelöst. Verify-Surrogat (Workspace-Build + service_impl-Tests) bestätigt Korrektheit der neuen DAO-Methode in vollem Maße. Empfohlen: Phase-4-Hygiene-Plan oder dedizierter Cleanup-Plan ergänzt `features = ["v4"]` in beiden Cargo.tomls.

## Issues Encountered

**Pre-existing 8 absence_period-Integration-Test-Failures in shifty_bin** (out of scope per Plan-Scope-Boundary):

Bei `cargo test --workspace` zeigen 8 Tests in `shifty_bin::integration_test::absence_period` "no such table: absence_period" — pre-existing seit Plan 02-01, dokumentiert in `.planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md`. Carry-Over für Phase 4. Plan 03-02 verändert das Test-Bild NICHT (kein neuer Fehler).

**Workspace-Test-Status nach Plan 03-02:**
- `service_impl --lib`: 321 passed, 0 failed, 6 ignored (Plan-01-Stubs) — GRÜN, identisch zu Plan-01-Baseline
- `shifty_bin` integration: 20 passed, 8 failed (pre-existing Phase-1-Migrations-Lücke), 4 ignored (Plan-01-Stubs) — identisch zu Plan-01-Baseline
- `cargo build --workspace`: GRÜN
- `cargo test --workspace --no-run`: GRÜN (alle Test-Binaries kompilieren)

## Self-Check

- service/src/warning.rs FOUND (`pub enum Warning` Verify-Count = 1; Variant-Count = 4 ohne Comments)
- service/src/lib.rs enthält `pub mod warning;` (grep -c = 1)
- dao/src/absence.rs enthält `find_overlapping_for_booking` (grep-Count = 2: Trait-Decl + Doc-Comment-Reference)
- dao_impl_sqlite/src/absence.rs enthält `find_overlapping_for_booking` (grep-Count = 1: Impl-Site)
- dao_impl_sqlite/src/absence.rs enthält `deleted IS NULL` 7× (mehrfach, Pflicht für Pitfall-1)
- service/src/shiftplan.rs enthält `pub enum UnavailabilityMarker` (grep-Count = 1)
- service/src/shiftplan.rs enthält `pub unavailable: Option<UnavailabilityMarker>` (grep-Count = 1)
- service_impl/src/shiftplan.rs `ShiftplanDay { ... }`-Construction-Sites (grep-Count = 1; einziges Producer im Workspace)
- Commit `572d6737` (Task 1 Warning-Modul) FOUND in jj log
- Commit `8fa3eefb` (Task 2 DAO-Methode) FOUND in jj log
- Commit `35fb3edb` (Task 3 ShiftplanDay-Erweiterung) FOUND in jj log
- `cargo build --workspace` exit 0
- `cargo test -p service_impl --lib`: 321 passed/0 failed/6 ignored (matches Plan-01-Baseline)
- `cargo test --workspace --no-run`: alle Test-Binaries linken
- `jj diff -r 'porzpqqx::npknnrnq' service/src/booking.rs service_impl/src/booking.rs rest/src/booking.rs service_impl/src/test/booking.rs`: leer (D-Phase3-18 Regression-Lock vorbereitet)

## Self-Check: PASSED

## Next Phase Readiness

Plan 03-03 (Wave-2 AbsenceService Forward-Warning) kann unmittelbar starten:
- `service::warning::Warning` enum ist verfügbar — `AbsenceService::create/update` kann Forward-Warning-Wrapper-Result jetzt produzieren.
- `dao::absence::AbsenceDao::find_overlapping_for_booking` ist verfügbar — wird vom Service-Layer (Plan 03-03 für AbsenceService-Forward-Pfad und Plan 03-04 für ShiftplanEditService-Reverse-Pfad) konsumiert.
- `MockAbsenceDao::expect_find_overlapping_for_booking()` automatisch verfügbar (durch `#[automock]`-Auto-Derivation) — Mock-basierte Service-Tests können sofort darauf greifen.
- `ShiftplanDay.unavailable: Option<UnavailabilityMarker>` ist da; Plan 03-04 kann den per-sales-person-Helper `build_shiftplan_day_for_sales_person` bauen, der das Feld setzt.

**Wave-2 Forcing-State:** Plan 03-04-Stubs in `service_impl/src/test/shiftplan_edit.rs` (6 #[ignore]-Tests) und Plan 03-06-Stubs in `shifty_bin/src/integration_test/booking_absence_conflict.rs` (4 #[ignore]-Tests) sind weiterhin ignored. Wave 2 (Plan 03-03) muss `AbsenceService` erweitern; Wave 3 (Plan 03-04) muss diese Stubs aktivieren und implementieren.

**D-Phase3-18 Regression-Lock**: BookingService-Files (`service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs`) sind durch Plan 03-02 NICHT angetastet. Gilt weiter als Hard-Constraint für Plans 03-03..03-06.

---
*Phase: 03-booking-shift-plan-konflikt-integration*
*Plan: 02 (Wave 1 Domain-Surface)*
*Completed: 2026-05-02*
