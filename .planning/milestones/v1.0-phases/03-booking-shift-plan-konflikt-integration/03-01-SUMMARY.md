---
phase: 03-booking-shift-plan-konflikt-integration
plan: 01
subsystem: testing
tags: [rust, cargo, mockall, tokio-test, scaffolding, ignore-stubs, wave-0, jj]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: AbsencePeriod-Surface (DAO/Service/REST/Permission) — Cross-Source-Stubs zielen auf diese Domain
  - phase: 02-reporting-integration-and-snapshot-versioning
    provides: stabile Reporting-Inputs — Stubs vermeiden Berührung der Phase-2-Wave-2-Surface
provides:
  - 6 #[ignore]-markierte Reverse-Warning + Permission-Stub-Tests in service_impl/src/test/shiftplan_edit.rs
  - 4 #[ignore]-markierte Cross-Source + Pitfall-1 Integration-Stub-Tests in shifty_bin/src/integration_test/booking_absence_conflict.rs
  - Test-Surface-Lock für Wave 1+2+3+5 (Tests müssen aktiviert werden — sichtbar in cargo test --list)
affects: [03-02, 03-03, 03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wave-0-Test-Scaffolding via #[ignore] + unimplemented!() — sichtbar in cargo test --list, panic'd bei versehentlichem #[ignore]-Entfernen ohne Implementation"
    - "Cross-Crate-Stub-Wiring: service_impl + shifty_bin parallel"

key-files:
  created:
    - service_impl/src/test/shiftplan_edit.rs
    - shifty_bin/src/integration_test/booking_absence_conflict.rs
  modified:
    - service_impl/src/test/mod.rs
    - shifty_bin/src/integration_test.rs
    - .planning/STATE.md  (chore commit für Plan-Start-Marker)

key-decisions:
  - "Stub-Pattern: #[ignore] + unimplemented!() — sichert sowohl Test-Listing als auch Panic-bei-Fehlaktivierung"
  - "Stubs ohne TestSetup-Imports — Wave 5 fügt In-Memory-SQLite-Setup analog absence_period.rs hinzu"
  - "KEINE Berührung von service_impl/src/test/booking.rs — D-Phase3-18 Regression-Lock"

patterns-established:
  - "Phase-3 Wave-Mapping: shiftplan_edit.rs-Stubs gehören Wave 3, booking_absence_conflict.rs-Stubs gehören Wave 5"
  - "Doc-Comment im Test-Modul listet pro Stub die Validation-ID (BOOK-02, SC4, D-Phase3-XX) — Continuation-Hints für Wave-Executors"

requirements-completed: []  # Plan-File listet BOOK-01/02/PLAN-01 als requirements, aber Plan 03-01 ist reines Test-Scaffolding — Requirements werden erst durch Plan 03-02..03-06 erfuellt.

# Metrics
duration: ~14min
completed: 2026-05-02
---

# Phase 3 Plan 01: Wave-0 Test-Scaffolding Summary

**10 #[ignore]-Stub-Tests (6 service_impl Reverse-Warning + Permission, 4 shifty_bin Cross-Source + Pitfall-1) wired in Test-Modul-Surface; cargo build --workspace bleibt grün; Wave 1+2+3+5 sind nun forciert, alle benannten Tests zu aktivieren.**

## Performance

- **Duration:** ~14 min
- **Started:** 2026-05-02 (initial STATE.md mark in working copy)
- **Completed:** 2026-05-02
- **Tasks:** 2
- **Files created:** 2
- **Files modified:** 3 (incl. STATE.md prep)

## Accomplishments

- `service_impl/src/test/shiftplan_edit.rs` neu — 6 Reverse-Warning + Permission-Stub-Tests mit exakten Namen aus `03-VALIDATION.md` (BOOK-02 / SC4 / D-Phase3-12 / D-Phase3-15)
- `shifty_bin/src/integration_test/booking_absence_conflict.rs` neu — 4 Cross-Source + Pitfall-1 + ShiftplanDay-Marker-Stub-Tests
- Modul-Wiring `pub mod shiftplan_edit;` in `service_impl/src/test/mod.rs` und `mod booking_absence_conflict;` in `shifty_bin/src/integration_test.rs` (beide unter `#[cfg(test)]`)
- `cargo build --workspace` GRÜN nach beiden Tasks
- `cargo test -p service_impl --tests`: 321 passed, 0 failed, **6 ignored** (genau die neuen Stubs)
- Cross-Crate-Test-List zeigt alle 10 neuen Test-Namen — `cargo test --workspace -- --list` sichtbar

## Task Commits

Each task was committed atomically as a separate jj change:

0. **Pre-task: STATE.md plan-start marker** — `60776314` (chore)
1. **Task 1: Service-Impl-Test-Modul `shiftplan_edit` mit Reverse-Warning-Stubs** — `fd777925` (test)
2. **Task 2: Integration-Test-Datei `booking_absence_conflict.rs` mit Cross-Source-Stubs** — `a27d19af` (test)

**Plan metadata commit:** _(diese SUMMARY + STATE.md/ROADMAP.md-Update — siehe finaler jj describe)_

## Files Created/Modified

### Created (2)
- `service_impl/src/test/shiftplan_edit.rs` — 6 #[ignore]-Stub-Tests:
  - `test_book_slot_warning_on_absence_day` (Wave-3, BOOK-02 / D-Phase3-14)
  - `test_book_slot_warning_on_manual_unavailable` (Wave-3, BOOK-02)
  - `test_book_slot_no_warning_when_softdeleted_absence` (Wave-3, SC4 / Pitfall-1)
  - `test_copy_week_aggregates_warnings` (Wave-3, D-Phase3-02 / D-Phase3-15)
  - `test_book_slot_with_conflict_check_forbidden` (Wave-3, D-Phase3-12)
  - `test_copy_week_with_conflict_check_forbidden` (Wave-3, D-Phase3-12)
- `shifty_bin/src/integration_test/booking_absence_conflict.rs` — 4 #[ignore]-Stub-Tests:
  - `test_double_source_two_warnings_one_booking` (Wave-5, BOOK-02 Cross-Source)
  - `test_softdeleted_absence_no_warning_no_marker` (Wave-5, SC4 / Pitfall-1)
  - `test_copy_week_three_bookings_two_warnings` (Wave-5, BOOK-02 / D-Phase3-02)
  - `test_shiftplan_marker_softdeleted_absence_none` (Wave-5, PLAN-01 + SC4 Read-Pfad)

### Modified (3)
- `service_impl/src/test/mod.rs` — `pub mod shiftplan_edit;` zwischen `shiftplan` und `shiftplan_catalog` eingefügt
- `shifty_bin/src/integration_test.rs` — `mod booking_absence_conflict;` (mit `#[cfg(test)]`) unmittelbar vor `mod dev_seed;` eingefügt
- `.planning/STATE.md` — Phase-Position-Marker auf "Phase: 03 EXECUTING / Plan: 1 of 6" (vor Plan-Start vom Orchestrator vorbereitet)

## Wave-Mapping für Continuation-Executors

| Test-Name | Plan / Wave | Erwartete Aktivierungs-Schritte |
|-----------|-------------|---------------------------------|
| test_book_slot_warning_on_absence_day | Plan 03-04 (Wave 3) | #[ignore] entfernen + Mock-DI-Setup analog `service_impl/src/test/booking.rs:113-192` |
| test_book_slot_warning_on_manual_unavailable | Plan 03-04 (Wave 3) | dito |
| test_book_slot_no_warning_when_softdeleted_absence | Plan 03-04 (Wave 3) | Mock liefert Empty-Vec |
| test_copy_week_aggregates_warnings | Plan 03-04 (Wave 3) | 3 source bookings → 2 warnings (D-Phase3-15: keine De-Dup) |
| test_book_slot_with_conflict_check_forbidden | Plan 03-04 (Wave 3) | beide Permission-Probes Forbidden |
| test_copy_week_with_conflict_check_forbidden | Plan 03-04 (Wave 3) | dito |
| test_double_source_two_warnings_one_booking | Plan 03-06 (Wave 5) | TestSetup analog `absence_period.rs:1-100`, full-stack |
| test_softdeleted_absence_no_warning_no_marker | Plan 03-06 (Wave 5) | full-stack soft-delete + assert empty warnings |
| test_copy_week_three_bookings_two_warnings | Plan 03-06 (Wave 5) | full-stack copy_week_with_conflict_check |
| test_shiftplan_marker_softdeleted_absence_none | Plan 03-06 (Wave 5) | get_shiftplan_week_for_sales_person Read-Pfad |

## Decisions Made

- **Stub-Pattern**: `#[ignore]` + `unimplemented!()` statt leerer Bodies. Begründung: Test-Liste zeigt die Tests (Sichtbarkeits-Forcing für Wave-Executors), aber `cargo test` führt sie nicht aus. Falls jemand `#[ignore]` versehentlich ohne Implementation entfernt, panic'd der Test sofort statt "0 assertions passed" silent-pass.
- **KEIN TestSetup-Import in Wave 0**: Stub-Bodies sind reine `unimplemented!()`-Macros — vermeidet Phantom-Imports, die später wieder entfernt/umstrukturiert werden müssten. Wave 5 baut TestSetup analog `absence_period.rs` von Grund auf.
- **Doc-Comment-Provenance**: Jedes Stub-Modul listet pro Test die Validation-ID (BOOK-02, SC4, D-Phase3-XX) — Continuation-Executors müssen 03-VALIDATION.md nicht erneut parsen, um die Bedeutung jedes Tests zu finden.

## Deviations from Plan

None — Plan executed exactly as written. Beide Tasks haben das spezifizierte Verify-Output produziert (6 service_impl Stubs in test list, 4 shifty_bin Stubs, mod-Wiring vorhanden).

## Issues Encountered

**Pre-existing 8 absence_period integration-test failures** (out of scope per Plan-Scope-Boundary):

Während der `cargo test --workspace`-Verifikation zeigten 8 Tests in `shifty_bin::integration_test::absence_period` "no such table: absence_period". Diese sind **pre-existing** und in `.planning/phases/02-.../deferred-items.md` als Phase-1-Migrations-Lücke auf der lokalen Dev-DB für Phase 4 dokumentiert (siehe STATE.md L101-103). Sie wurden NICHT durch diesen Plan ausgelöst — `cargo test -p service_impl --tests` zeigt 321 passed / 0 failed / 6 ignored. Plan-Scope-Boundary: Out-of-Scope-Failures werden nicht angefasst.

**Workspace-Test-Status nach Plan 03-01:**
- `service_impl`: 321 passed, 0 failed, 6 ignored (Plan-Stubs) — GRÜN
- `shifty_bin` integration: 20 passed, 8 failed (pre-existing Phase-4-Carry-Over), 4 ignored (Plan-Stubs)
- Build: `cargo build --workspace` GRÜN

## Self-Check

- service_impl/src/test/shiftplan_edit.rs FOUND
- shifty_bin/src/integration_test/booking_absence_conflict.rs FOUND
- service_impl/src/test/mod.rs contains `pub mod shiftplan_edit;` (verified via Edit)
- shifty_bin/src/integration_test.rs contains `mod booking_absence_conflict;` (grep -c = 1)
- Commit `fd777925` (Task 1) FOUND in jj log
- Commit `a27d19af` (Task 2) FOUND in jj log
- Commit `60776314` (Pre-task STATE.md mark) FOUND in jj log
- cargo build --workspace exits 0
- cargo test -p service_impl: 321 passed / 6 ignored (matches expected)
- cargo test -p shifty_bin --list: contains all 4 new test names

## Self-Check: PASSED

## Next Phase Readiness

Plan 03-02 (Wave-1 Domain-Surface) kann unmittelbar starten:
- Wave-0-Test-Surface ist gesichtet und sichtbar
- Build green, kein DI-Cycle
- D-Phase3-18 Regression-Lock auf `service_impl/src/test/booking.rs` UNANGETASTET — bestätigt durch unverändertes 321-passed-Resultat

**Forcing-Wirkung für Wave 3 + Wave 5:** Jeder Continuation-Executor (Plan 03-04 / 03-06) wird die Stub-Tests sehen und MUSS sie aktivieren — sonst bleibt die Validation-Coverage unvollständig, was beim `/gsd:verify-phase 03` als gap auffallen würde.

**Empfehlung:** `nyquist_compliant: true` in `03-VALIDATION.md` Frontmatter setzen, sobald STATE.md/ROADMAP.md committed sind (Wave-0-Requirements aus 03-VALIDATION.md "Wave 0 Requirements" sind anteilig erfüllt — vollständig erst mit Plan 03-02, das absence.rs/shiftplan.rs-Stubs erweitert).

---
*Phase: 03-booking-shift-plan-konflikt-integration*
*Plan: 01 (Wave 0 Test-Scaffolding)*
*Completed: 2026-05-02*
