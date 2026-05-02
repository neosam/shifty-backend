---
phase: 3
slug: booking-shift-plan-konflikt-integration
status: completed
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-02
last_updated: 2026-05-02
---

# Phase 3 — Validation Strategy

> Per-phase validation contract — abgeschlossen mit Plan 03-06.
> Quelle: `03-RESEARCH.md` § Validation Architecture (Z. 981–1031).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (nativ) + `mockall` 0.13 für Trait-Mocks + `tokio::test` async |
| **Config file** | `Cargo.toml` per Crate (kein extra Test-Config) |
| **Quick run command** | `cargo test -p service_impl test::shiftplan_edit` (oder `test::absence`, `test::shiftplan`) |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30s (per crate) / ~90s (workspace; Phase-3-Endstand: 336 service_impl + 32 shifty_bin = 368 passing tests) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl test::<modul>` (relevantes Modul) — < 30 s
- **After every plan wave:** Run `cargo test --workspace` — ~ 90 s
- **Before `/gsd-verify-work`:** Full suite green + `cargo build --workspace` + `cargo run` boot OK
- **Max feedback latency:** 30 s

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-06-01 | 06 | 5 | BOOK-01 | T-3-AbsServ-Read | AbsenceService::create gibt Warning bei Booking-Konflikt | unit | `cargo test -p service_impl test::absence::test_create_warning_for_booking_in_range` | ✅ | ✅ green |
| 03-06-02 | 06 | 5 | BOOK-01 | T-3-AbsServ-Read | AbsenceService::create gibt Warning bei manueller Unavailable im Range | unit | `cargo test -p service_impl test::absence::test_create_warning_for_manual_unavailable_in_range` | ✅ | ✅ green |
| 03-06-03 | 06 | 5 | BOOK-01 | — | AbsenceService::update gibt Warnings für ALLE Tage in NEUER Range | unit | `cargo test -p service_impl test::absence::test_update_returns_warnings_for_full_new_range` | ✅ | ✅ green |
| 03-04-04 | 04 | 3 | BOOK-02 | T-3-CrossSrc | book_slot_with_conflict_check gibt Warning bei AbsencePeriod-Tag | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_absence_day` | ✅ | ✅ green |
| 03-04-05 | 04 | 3 | BOOK-02 | T-3-CrossSrc | book_slot_with_conflict_check gibt Warning bei sales_person_unavailable | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_manual_unavailable` | ✅ | ✅ green |
| 03-04-06 | 04 | 3 | BOOK-02 | T-3-CrossSrc | copy_week_with_conflict_check aggregiert Warnings über alle inneren Calls | unit | `cargo test -p service_impl test::shiftplan_edit::test_copy_week_aggregates_warnings` | ✅ | ✅ green |
| 03-06-07 | 06 | 5 | BOOK-02 | T-3-CrossSrc | Cross-Source: ein Tag mit beiden Quellen → ZWEI Warnings | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_double_source_two_warnings_one_booking` | ✅ | ✅ green |
| 03-06-08 | 06 | 5 | SC4 | T-3-SoftDel | Pitfall-1: soft-deleted AbsencePeriod triggert KEINE Warning | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_softdeleted_absence_no_warning_no_marker` | ✅ | ✅ green |
| 03-XX-09 | — | — | BOOK-02 (Regression) | — | Klassisches `BookingService::create` und `copy_week` bleiben unverändert (D-Phase3-18 Regression-Lock) | unit | `cargo test -p service_impl test::booking` | ✅ existing | ✅ green |
| 03-04-10 | 04 | 3 | PLAN-01 | T-3-PerSP | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::AbsencePeriod | unit | `cargo test -p service_impl test::shiftplan::test_get_shiftplan_week_for_sales_person_marker_absence_only` | ✅ | ✅ green |
| 03-04-11 | 04 | 3 | PLAN-01 | T-3-PerSP | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::Both | unit | `cargo test -p service_impl test::shiftplan::test_get_shiftplan_week_for_sales_person_marker_both` | ✅ | ✅ green |
| 03-06-11b | 06 | 5 | PLAN-01 | T-3-PerSP | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::ManualUnavailable | unit | `cargo test -p service_impl test::shiftplan::test_get_shiftplan_week_for_sales_person_marker_manual_only` | ✅ | ✅ green |
| 03-04-11c | 04 | 3 | SC4 | T-3-SoftDel | get_shiftplan_week_for_sales_person + soft-deleted absence → kein Marker | unit | `cargo test -p service_impl test::shiftplan::test_get_shiftplan_week_for_sales_person_softdeleted_absence_no_marker` | ✅ | ✅ green |
| 03-04-12 | 04 | 3 | PLAN-01 | T-3-PermHRSelf | Permission HR ∨ self auf den per-sales-person-Methoden | unit | `cargo test -p service_impl test::shiftplan::test_get_shiftplan_week_for_sales_person_forbidden` | ✅ | ✅ green |
| 03-06-13 | 06 | 5 | ALL | T-3-PermHRSelf | _forbidden-Test pro neue public Service-Methode (5 neue Methoden) | unit | `cargo test -p service_impl test_*_forbidden` | ✅ | ✅ green |
| 03-06-13b | 06 | 5 | BOOK-01 | T-3-PermHRSelf | find_overlapping_for_booking forbidden | unit | `cargo test -p service_impl test::absence::test_find_overlapping_for_booking_forbidden` | ✅ | ✅ green |
| 03-06-14 | 06 | 5 | SC4 | T-3-SoftDel | soft-deleted AbsencePeriod erzeugt KEINEN ShiftplanDay-Marker | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_shiftplan_marker_softdeleted_absence_none` | ✅ | ✅ green |
| 03-06-15 | 06 | 5 | BOOK-02 | T-3-CrossSrc | copy_week_three_bookings_two_warnings full-stack | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_copy_week_three_bookings_two_warnings` | ✅ | ✅ green |

*Status legend: pending (not used here — all rows green) · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `service_impl/src/test/shiftplan_edit.rs` — Reverse-Warning-Tests aktiviert (Plan 03-04 Wave 3, alle 6 grün)
- [x] `service_impl/src/test/absence.rs` — Forward-Warning-Tests + find_overlapping_for_booking_forbidden (Plan 03-06)
- [x] `service_impl/src/test/shiftplan.rs` — per-sales-person + UnavailabilityMarker-Marker-Suite (Plan 03-04 + Plan 03-06 marker_manual_only)
- [x] `shifty_bin/src/integration_test/booking_absence_conflict.rs` — 4 Cross-Source-Tests aktiv (Plan 03-06)
- [x] `shifty_bin/src/integration_test.rs` — `mod booking_absence_conflict;` (Plan 03-01)
- [x] **Regression-Lock D-Phase3-18:** `service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs` unangetastet — `jj diff` über kompletten Phase-3-Span = 0 Lines.

**Existing test infrastructure deckt:** mockall-Patterns, TestSetup für Integration, `_forbidden`-Helper (`crate::test::error_test::test_forbidden`).

---

## Phase-3 Success Criteria — Final Verification

| SC | Anforderung | Verifiziert via |
|----|-------------|-----------------|
| **SC1** | Forward-Warning beim Anlegen einer überlappenden Absence (Wrapper mit Booking-IDs + Daten); Persistenz unverändert | `test_create_warning_for_booking_in_range` (service_impl/test/absence.rs) + `test_create_warning_for_manual_unavailable_in_range` + `test_update_returns_warnings_for_full_new_range` + Integration `test_double_source_two_warnings_one_booking` |
| **SC2** | Reverse-Warning beim Anlegen eines Bookings auf absence-day OR sales_person_unavailable (BookingService::create unverändert grün — D-Phase3-18) | `test_book_slot_warning_on_absence_day` + `test_book_slot_warning_on_manual_unavailable` + `test_copy_week_aggregates_warnings` (service_impl/test/shiftplan_edit.rs) + Integration `test_double_source_two_warnings_one_booking` + `test_copy_week_three_bookings_two_warnings` |
| **SC3** | Shift-Plan-Markierung für Mitarbeiter über Zeitraum (AbsencePeriod ∨ sales_person_unavailable; manuelle Einträge bleiben für Einzeltage möglich) | `test_get_shiftplan_week_for_sales_person_marker_absence_only` + `test_marker_manual_only` + `test_marker_both` (D-Phase3-10) + `test_marker_softdeleted_absence_no_marker` + `test_get_shiftplan_week_for_sales_person_forbidden` (5 Tests in service_impl/test/shiftplan.rs) |
| **SC4** | Soft-deleted AbsencePeriod triggert keine Warning + keine Markierung (Pitfall-1) | Service-Layer: `test_book_slot_no_warning_when_softdeleted_absence` + `test_get_shiftplan_week_for_sales_person_softdeleted_absence_no_marker`; Integration: `test_softdeleted_absence_no_warning_no_marker` + `test_shiftplan_marker_softdeleted_absence_none` (DAO-Layer-Filter `WHERE deleted IS NULL` aus Plan 03-02) |

**Alle 4 Success-Criteria erfüllt.**

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OpenAPI-Snapshot zeigt neue Wrapper-DTOs (`BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `WarningTO`, `UnavailabilityMarkerTO`) und neue Endpunkte (`POST /shiftplan-edit/booking`, `POST /shiftplan-edit/copy-week`, `GET /shiftplan-info/.../sales-person/...`) | BOOK-01, BOOK-02, PLAN-01 | OpenAPI-Diff ist visuell zu prüfen; utoipa erzeugt Schema deterministisch zur Compile-Zeit (verifiziert via `cargo build`) | `cargo run` → `curl http://localhost:3000/openapi.json | jq '.paths \| keys'`. **Note:** Boot-Smoke ist auf der lokalen Dev-DB durch pre-existing Migrations-Drift `VersionMissing(20260428101456)` blockiert (siehe `deferred-items.md`); In-Memory-SQLite (32 Integration-Tests) bestätigt Service-Init + REST-Surface vollständig. |
| `cargo run` Boot-Smoke: kein DI-Cycle-Panic beim Start | ALL | Service-Tier-Konvention soll DI-Konstruktion deterministisch halten | In-Memory-Substitut: `cargo test -p shifty_bin --tests` zeigt 32 passed (vorher 24, +8 absence_period dank Migration-Recovery + 4 booking_absence_conflict). Service-DI mit allen Phase-3-Deps erfolgreich. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter
- [x] Plan-Referenzen in Task-IDs gefüllt (03-04 / 03-06 statt 03-XX)
- [x] D-Phase3-18 Regression-Lock final = 0 lines diff über kompletten Phase-3-Span

**Approval:** approved (Plan 06 abgeschlossen, alle 4 SCs erfüllt, alle Tests grün, D-Phase3-18 Regression-Lock final verifiziert)
