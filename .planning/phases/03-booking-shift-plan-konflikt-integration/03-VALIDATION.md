---
phase: 3
slug: booking-shift-plan-konflikt-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-02
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Quelle: `03-RESEARCH.md` § Validation Architecture (Z. 981–1031).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (nativ) + `mockall` 0.13 für Trait-Mocks + `tokio::test` async |
| **Config file** | `Cargo.toml` per Crate (kein extra Test-Config) |
| **Quick run command** | `cargo test -p service_impl test::shiftplan_edit` (oder `test::absence`, `test::shiftplan`) |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30s (per crate) / ~90s (workspace, Phase 1+2 hatte 381 passing tests) |

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
| 03-XX-01 | XX | 1 | BOOK-01 | T-3-AbsServ-Read | AbsenceService::create gibt Warning bei Booking-Konflikt | unit | `cargo test -p service_impl test::absence::test_create_warning_for_booking_in_range` | ❌ W0 | ⬜ pending |
| 03-XX-02 | XX | 1 | BOOK-01 | T-3-AbsServ-Read | AbsenceService::create gibt Warning bei manueller Unavailable im Range | unit | `cargo test -p service_impl test::absence::test_create_warning_for_manual_unavailable_in_range` | ❌ W0 | ⬜ pending |
| 03-XX-03 | XX | 1 | BOOK-01 | — | AbsenceService::update gibt Warnings für ALLE Tage in NEUER Range | unit | `cargo test -p service_impl test::absence::test_update_returns_warnings_for_full_new_range` | ❌ W0 | ⬜ pending |
| 03-XX-04 | XX | 1 | BOOK-02 | T-3-CrossSrc | book_slot_with_conflict_check gibt Warning bei AbsencePeriod-Tag | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_absence_day` | ❌ W0 | ⬜ pending |
| 03-XX-05 | XX | 1 | BOOK-02 | T-3-CrossSrc | book_slot_with_conflict_check gibt Warning bei sales_person_unavailable | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_manual_unavailable` | ❌ W0 | ⬜ pending |
| 03-XX-06 | XX | 1 | BOOK-02 | T-3-CrossSrc | copy_week_with_conflict_check aggregiert Warnings über alle inneren Calls | unit | `cargo test -p service_impl test::shiftplan_edit::test_copy_week_aggregates_warnings` | ❌ W0 | ⬜ pending |
| 03-XX-07 | XX | 2 | BOOK-02 | T-3-CrossSrc | Cross-Source: ein Tag mit beiden Quellen → ZWEI Warnings | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_double_source_two_warnings` | ❌ W0 | ⬜ pending |
| 03-XX-08 | XX | 2 | SC4 | T-3-SoftDel | Pitfall-1: soft-deleted AbsencePeriod triggert KEINE Warning | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_softdeleted_absence_no_warning` | ❌ W0 | ⬜ pending |
| 03-XX-09 | XX | 1 | BOOK-02 (Regression) | — | Klassisches `BookingService::create` und `copy_week` bleiben unverändert (alle alten Tests grün) | unit | `cargo test -p service_impl test::booking` | ✅ existing | ⬜ pending |
| 03-XX-10 | XX | 1 | PLAN-01 | T-3-PerSP | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::AbsencePeriod | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_marker_absence_only` | ❌ W0 | ⬜ pending |
| 03-XX-11 | XX | 1 | PLAN-01 | T-3-PerSP | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::Both | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_marker_both_sources` | ❌ W0 | ⬜ pending |
| 03-XX-12 | XX | 1 | PLAN-01 | T-3-PermHRSelf | Permission HR ∨ self auf den per-sales-person-Methoden | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_forbidden_other_user` | ❌ W0 | ⬜ pending |
| 03-XX-13 | XX | 1 | ALL | T-3-PermHRSelf | _forbidden-Test pro neue public Service-Methode (5 neue Methoden) | unit | `cargo test -p service_impl test_*_forbidden` | ❌ W0 | ⬜ pending |
| 03-XX-14 | XX | 2 | SC4 | T-3-SoftDel | soft-deleted AbsencePeriod erzeugt KEINEN ShiftplanDay-Marker | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_shiftplan_marker_softdeleted_absence_none` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

> Plan-Phase füllt `Plan` (XX → 01/02/...) und `Task ID`-Suffix nach finalem Plan-Schnitt.

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/shiftplan_edit.rs` (oder Modul-Verzeichnis) — Reverse-Warning-Tests für `book_slot_with_conflict_check` + `copy_week_with_conflict_check` + Pitfall-1-Test scaffolden
- [ ] `service_impl/src/test/absence.rs` — Forward-Warning-Tests scaffolden (`test_create_warning_for_*`, `test_update_returns_warnings_for_full_new_range`)
- [ ] `service_impl/src/test/shiftplan.rs` — per-sales-person + UnavailabilityMarker::Both-Tests scaffolden (`test_per_sales_person_*`)
- [ ] `shifty_bin/src/integration_test/booking_absence_conflict.rs` — NEUE Datei analog `absence_period.rs` aus Phase 1 (Cross-Source + Pitfall-1)
- [ ] `shifty_bin/src/integration_test.rs` — `mod booking_absence_conflict;` ergänzen
- [ ] **Regression-Lock:** Plan-File markiert `service_impl/src/test/booking.rs` als "DO NOT MODIFY in Phase 3" — alle bestehenden BookingService-Tests müssen grün bleiben.

**Existing test infrastructure deckt:** mockall-Patterns, TestSetup für Integration, `_forbidden`-Helper (`crate::test::error_test::test_forbidden`).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OpenAPI-Snapshot zeigt neue Wrapper-DTOs (`BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO`, `WarningTO`, `UnavailabilityMarkerTO`) und neue Endpunkte (`POST /shiftplan-edit/booking`, `POST /shiftplan-edit/copy-week`, `GET /shiftplan/.../sales-person/...`) | BOOK-01, BOOK-02, PLAN-01 | OpenAPI-Diff ist visuell zu prüfen; utoipa erzeugt Schema deterministisch, aber Frontend-Migration darauf basiert | `cargo run` → `curl http://localhost:3000/openapi.json | jq '.paths | keys'` und `.components.schemas | keys` |
| `cargo run` Boot-Smoke: kein DI-Cycle-Panic beim Start | ALL | Service-Tier-Konvention soll DI-Konstruktion deterministisch halten; ein DI-Bug zeigt sich erst beim Boot | `cargo run` mit timeout 10s; expected: keine Panic, Server hört auf `:3000` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter (after Wave 0 commit)

**Approval:** pending
