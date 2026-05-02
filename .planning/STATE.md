---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: ready_to_plan
last_updated: "2026-05-03T00:10:00.000Z"
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 15
  completed_plans: 15
  percent: 75
# completed_phases counts phases with /gsd:verify-phase passed (Phase 1 + Phase 2);
# Phase 3 has all 6 plans done (all SCs verified per Test-IDs in 03-VALIDATION.md),
# awaiting `/gsd:verify-phase 03` to flip Phase-3 status to "Complete".
---

# Project State: Shifty Backend — Range-Based Absence Management

## Project Reference

- **Project document**: `.planning/PROJECT.md`
- **Requirements**: `.planning/REQUIREMENTS.md`
- **Roadmap**: `.planning/ROADMAP.md`
- **Research**: `.planning/research/SUMMARY.md`
- **Codebase context**: `.planning/codebase/`
- **Core value**: Mitarbeiter und HR sehen jederzeit eine korrekte Stunden- und Abwesenheits-Bilanz, und Änderungen an Stammdaten ziehen sich automatisch in alle abhängigen Berechnungen durch — ohne manuelle Nacharbeit.
- **Current focus**: Replace per-date hour-amount absence accounting with range-based absences whose per-day hour effects are derived from the contract valid on that day.

## Current Position

Phase: 4
Plan: Not started

- **Current milestone**: Range-Based Absence Management
- **Current phase**: 03 — Booking & Shift-Plan Konflikt-Integration (all 6 plans complete; awaiting `/gsd:verify-phase 03`)
- **Current plan**: 03-06 ✅ Wave-5 Final-Tests + Phase-Closure (4 Forward-Warning-Tests in test/absence.rs, 1 marker_manual_only-Test in test/shiftplan.rs, 4 Cross-Source-Integration-Tests aktiviert, Phase-1-Migration recovered, 03-VALIDATION.md completed, ROADMAP Phase-3 [x]; jj-Changes `e02964a6`+`7aeed067`+`d9ca9ba9`+`02d0642f`+`3e9c74f7`)
- **Status**: phase-3-all-plans-done (next: `/gsd:verify-phase 03` to flip Phase-3 status to verified-complete; then `/gsd:plan-phase 04` for Migration & Cutover)
- **Last action**: 03-06-execute-complete (2026-05-02)
- **Progress**: Phases 1+2 complete; Phase 3 all-plans-done; Plans 15/15 done (100%)

## Performance Metrics

| Metric | Value |
|---|---|
| Phases complete | 2 / 4 (P1 + P2 verified; P3 all-plans-done, awaiting /gsd:verify-phase 03) |
| Plans complete | 15 / 15 (Phase 1 vollstaendig + Phase 2 vollstaendig 01..04 + Phase 3 vollstaendig 01..06) |
| Requirements mapped | 19 / 19 |
| Requirements complete | 17 / 19 (Phase-1-vollstaendig + Phase-2: SNAP-01/SNAP-02/REP-01..04 + Phase-3: BOOK-01/BOOK-02/PLAN-01) |
| Open discuss-phase decisions | 4 (Phase-4-blocking) |

### Phase-Plan Execution Log

| Phase | Plan | Duration | Tasks | Files | Date |
|-------|------|----------|-------|-------|------|
| 02 | 01 | ~12min | 3 (+ 1 Phase-1-Rule-3-Fix) | 8 (5 neu Wave-0 + 3 Phase-1-Fix) | 2026-05-02 |
| 02 | 02 | ~11min | 3 | 5 (1 service, 1 service_impl, 2 test, 1 main.rs) | 2026-05-02 |
| 02 | 03 | ~13min | 3 | 13 (6 neu + 7 patches) | 2026-05-02 |
| 02 | 04 | ~35min | 3 (+ 1 Rule-1-Auto-Fix; atomar) | 11 (alle in jj-Change 39be1b73) | 2026-05-02 |
| 03 | 01 | ~14min | 2 | 5 (2 neu test-stubs + 2 mod-patches + 1 STATE-prep) | 2026-05-02 |
| 03 | 02 | ~16min | 3 | 7 (1 neu warning.rs + 1 neu deferred-items.md + 5 patches + .sqlx-Cache 4 Files) | 2026-05-02 |
| 03 | 03 | ~25min | 3 | 6 (alle patches: service/absence + service_impl/absence + test/absence + rest/absence + main.rs + integration_test/absence_period) | 2026-05-02 |
| 03 | 04 | ~30min | 3 | 7 (alle patches: service/shiftplan_edit + service_impl/shiftplan_edit + test/shiftplan_edit + service/shiftplan + service_impl/shiftplan + test/shiftplan + main.rs) | 2026-05-02 |
| 03 | 05 | ~25min | 3 | 5 (alle patches: rest-types/lib.rs + rest/absence + rest/shiftplan_edit + rest/shiftplan + rest/lib.rs) | 2026-05-02 |
| 03 | 06 | ~25min | 4 (Task-1 Plan-04-Carryover; +Rule-3 Migration-Recovery) | 7 (1 neu Migration + 4 patches Tests/VALIDATION/ROADMAP + 1 SUMMARY + 1 STATE.md) | 2026-05-02 |

## Accumulated Context

### Key Decisions Logged

Aus `PROJECT.md` (alle als "Pending" — werden bei Phase-Transition validiert):

- Abwesenheiten als Zeitraum (`from_date`, `to_date`) statt einzelner Datums-Einträge mit Stunden.
- Nur Ganztage, keine Halbtage / Stundenebene in v1.
- Vertragsänderungen wirken nur prospektiv (neue Periode neu berechnet, alte unverändert).
- Booking-Konflikte = Warnung (nicht-blockierend), kein Auto-Löschen.
- Feiertage orthogonal: 0 Urlaubsstunden am Feiertag, separate Feiertags-Gutschrift bleibt unverändert.
- Migration via Heuristik aus wochenweisen Bestands-`ExtraHours`.
- Cutover via Feature-Flag mit Stunden-Konto-Identitäts-Gate (Null-Drift-Garantie).
- Backend-First, Frontend folgt in separatem Workstream.

### Architecture Decisions Logged

Aus `research/ARCHITECTURE.md`:

- Parallele `absence` Domain (nicht Erweiterung von `extra_hours`).
- Hybrid materialize-on-snapshot / derive-on-read (Live-Reports derive on read; BillingPeriod-Snapshots materialize-once).
- Direction: `AbsenceService → BookingService` (Business-Logic-Tier konsumiert Basic-Tier; nie umgekehrt — verhindert Circular Dep). Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services".
- Service-Tier-Konvention etabliert (2026-05-02): Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Konflikt-aware Schreib-Pfade (z.B. Booking-mit-Warning) leben im Business-Logic-Tier (z.B. `ShiftplanEditService`), nicht in Basic Services. Doku: `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen".
- `BookingCreateResult { booking, warnings }`-Wrapper für nicht-blockierende Warnings (lebt im Business-Logic-Tier, nicht im `BookingService` selbst).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit wie der Reporting-Switch (per `CLAUDE.md`).
- Phase-3 Wave-0-Stub-Pattern (2026-05-02): `#[ignore]` + `unimplemented!()` als Standard für Wave-Forcing. Test-Liste sichtbar in `cargo test --list`, Body panic'd bei versehentlichem Aktivieren ohne Implementation. Pattern in `service_impl/src/test/shiftplan_edit.rs` und `shifty_bin/src/integration_test/booking_absence_conflict.rs` etabliert.
- Phase-3 Wave-1 Domain-Surface (2026-05-02): `service::warning::Warning` lebt in eigenem `service/src/warning.rs`-Modul (C-Phase3-01); `pub mod warning;` alphabetisch in `lib.rs`; KEIN `pub use` am Root — Konsumenten via `use service::warning::Warning;`. `AbsenceDao::find_overlapping_for_booking` ist kategorie-frei (alle 3 AbsenceCategoryEntity-Werte) und ohne `exclude_logical_id` (Booking-IDs orthogonal zu Absence-IDs). `ShiftplanDay.unavailable: Option<UnavailabilityMarker>` additiv mit Default `None` — globale Sicht setzt nie etwas.
- Phase-3 Wave-2 AbsenceService Forward-Warning (2026-05-02): `AbsenceService::create`/`::update` returnen `AbsencePeriodCreateResult { absence, warnings: Arc<[Warning]> }` (Wrapper-Sig-Bruch). Forward-Warning-Loop in `compute_forward_warnings` (private Helper im `impl<Deps> AbsenceServiceImpl<Deps>`) läuft NACH dem DAO-Persist + VOR `commit`; `Authentication::Full` im internen Loop (outer Permission ist HR ∨ self bereits verifiziert). 3 neue Basic-Service-Deps am AbsenceServiceImpl: `BookingService` + `SalesPersonUnavailableService` + `SlotService` (SlotService nötig, weil Booking nur slot_id + calendar_week + year trägt; day_of_week kommt aus Slot). Warnings tragen `absence_id = entity.id` (Create) bzw. `active.logical_id` (Update — D-07-stable). Soft-Delete-Filter für Booking + ManualUnavailable explizit (Pitfall-1 / SC4).
- Phase-3 Wave-3 Reverse-Warning + per-sales-person-Sicht (2026-05-02): `ShiftplanEditService::book_slot_with_conflict_check` produziert `BookingCreateResult { booking, warnings: Arc<[Warning]> }`; `::copy_week_with_conflict_check` aggregiert Warnings ohne De-Dup (D-Phase3-15). Wrapper-Structs leben in `service/src/shiftplan_edit.rs` (NICHT in `service/src/booking.rs` — D-Phase3-18 Regression-Lock). Permission HR ∨ self für `book_slot`, `shiftplan.edit` für `copy_week` (Bulk-Op-Pattern). `ShiftplanViewService::get_shiftplan_*_for_sales_person` setzt `unavailable: Option<UnavailabilityMarker>` per Tag mit 4-Wege-Match (None/AbsencePeriod/ManualUnavailable/Both, D-Phase3-10). Neuer Parallel-Helper `build_shiftplan_day_for_sales_person` (C-Phase3-03) lässt globale Sicht unangetastet. DI-Konstruktionsreihenfolge unverändert Tier-konform (`absence_service` → `shiftplan_edit_service` → `shiftplan_view_service`).
- Phase-3 Wave-4 REST-Layer (2026-05-02): 5 inline Phase-3 Wrapper-DTOs in `rest-types/src/lib.rs` (`WarningTO` + `UnavailabilityMarkerTO` als Tag-Enums mit `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]` — utoipa-5-Support, RESEARCH.md Pattern 4; `BookingCreateResultTO` + `CopyWeekResultTO` + `AbsencePeriodCreateResultTO` als Wrapper-Structs). `ShiftplanDayTO.unavailable: Option<UnavailabilityMarkerTO>` additiv mit `#[serde(default, skip_serializing_if = "Option::is_none")]` — globale Endpunkte serialisieren das Feld weiterhin nicht (Wire-Compat); per-sales-person-Endpunkte setzen es. `POST /absence-period` und `PATCH /absence-period/{id}` liefern jetzt `AbsencePeriodCreateResultTO` (Plan-03-TODO eingelöst). Neue Endpunkte `POST /shiftplan-edit/booking` + `POST /shiftplan-edit/copy-week` (BOOK-02 / D-Phase3-02) und `GET /shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}` + `GET /shiftplan-info/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}` (PLAN-01). ShiftplanEditApiDoc neu (existing edit_slot/delete_slot/add_vacation bleiben außerhalb der `paths(...)` — Phase-1-Tech-Debt; out-of-scope). `POST /booking` und `POST /booking/copy-week` strikt unangetastet (D-Phase3-18 Regression-Lock).

### Open Todos

- [x] `/gsd:execute-plan 02-02` — Wave 1 derive_hours_for_range (abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 02-03` — Feature-Flag-Infrastruktur (abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 02-04` — Wave 2 atomarer Snapshot-Bump-Commit (abgeschlossen 2026-05-02, jj-Change `39be1b73`).
- [x] `/gsd:execute-plan 03-01` — Wave-0 Test-Scaffolding (10 #[ignore]-Stubs; jj-Changes `60776314`+`fd777925`+`a27d19af`, abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 03-02` — Wave-1 Domain-Surface (Warning-Enum + AbsenceDao::find_overlapping_for_booking + UnavailabilityMarker + ShiftplanDay-Field; jj-Changes `572d6737`+`8fa3eefb`+`35fb3edb`, abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 03-03` — Wave-2 AbsenceService Forward-Warning (Sig-Brüche AbsenceService::create/update zu AbsencePeriodCreateResult + Forward-Warning-Loop + find_overlapping_for_booking + neue DI-Deps; jj-Changes `d744efe4`+`106ea712`+`dfd66a56`, abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 03-04` — Wave-3 ShiftplanEditService Reverse-Warning + ShiftplanViewService per-sales-person + DI-Wiring (6 Plan-01-Wave-3-Stubs aktiviert + 4 neue per-sales-person-Tests; jj-Changes `448c265e`+`92358f05`+`00ce6d87`, abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 03-05` — Wave-4 REST-Layer (5 Wrapper-DTOs + Wrapper-Body für /absence-period + 2 neue /shiftplan-edit-Endpunkte + 2 neue per-sales-person-Endpunkte unter /shiftplan-info + ApiDoc; jj-Changes `1a64ff7b`+`0738d880`+`5b60dc29`, abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 03-06` — Wave-5 Final-Tests + Phase-Closure (4 Forward-Warning-Tests in test/absence.rs, 1 marker_manual_only-Test in test/shiftplan.rs, 4 Cross-Source-Integration-Tests aktiviert, Phase-1-Migration recovered als Bonus, 03-VALIDATION.md completed, ROADMAP Phase-3 [x]; jj-Changes `e02964a6`+`7aeed067`+`d9ca9ba9`+`02d0642f`+`3e9c74f7`, abgeschlossen 2026-05-02).
- [ ] `/gsd:verify-phase 03` — Phase-3-Closure-Verification (alle 4 SCs sind in 03-VALIDATION.md durch konkrete Test-IDs erfüllt; Verify-Phase ist nur noch Formal-Akt).
- [ ] Production-Data-Profile lauffähig vorbereiten (für Phase 4, kann optional schon parallel laufen — read-only).
- [ ] Phase-1-Hygiene: Lokale `localdb.sqlite3`-Drift fixen (siehe `.planning/phases/02-.../deferred-items.md`).
- [ ] Phase-Hygiene: `dao/Cargo.toml` und `dao_impl_sqlite/Cargo.toml` `features = ["v4"]` ergänzen (siehe `.planning/phases/03-.../deferred-items.md`); aktuell pre-existing Drift, der `cargo test -p dao*` standalone bricht.

### Wave-2-Forcing-State (aus Plan 02-01)

Wave 0 hat ein **build-time Forcing** etabliert, das Wave 2 zwingt, Snapshot-Bump und UnpaidLeave-Variante atomar zu liefern:

- **Pin-Map-Test ROT** (`test_snapshot_schema_version_pinned`): erwartet `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`, ist aktuell 2. Wird GRUEN sobald Wave 2 das Bump macht.
- **Compiler-Exhaustive-Match-Test GRUEN** (`test_billing_period_value_type_surface_locked`): listet alle 11 aktuellen Varianten. Wird COMPILE-ERROR sobald Wave 2 die `BillingPeriodValueType::UnpaidLeave`-Variante hinzufuegt -- in der Test-Datei wartet ein auskommentierter Arm zur Aktivierung.

### Blockers

Keine harten Blocker. Phase 2 ist abgeschlossen — atomarer Wave-2-Commit `39be1b73` enthaelt Snapshot-Bump 2→3, UnpaidLeave-Variante, Reporting-Switch und alle 4 neuen Tests. Plus Rule-1-Auto-Fix in FeatureFlagService::is_enabled (Authentication::Full-Bypass) macht 7 vorher fehlschlagende Reporting-Integration-Tests gruen.

**Carry-Over fuer Phase 4:** Die in `deferred-items.md` dokumentierten 8 fehlschlagenden `absence_period`-Integration-Tests (pre-existing Phase-1-Migrations-Luecke) sollten von Phase 4 nachgereicht werden, damit `cargo run` auf der lokalen Dev-DB sauber bootet.

### Constraints In Force

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet — Commits manuell durch User. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`).
- **NixOS**: Tools wie `sqlx-cli` ggf. via `nix-shell` aufrufen (siehe `CLAUDE.local.md`).
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert (per `CLAUDE.md`). Fällig in Phase 2.
- **Multi-Sprache**: Alle benutzersichtbaren Texte in en/de/cs (Out of Scope für dieses Backend-Milestone, aber DTO-Felder, die Texte transportieren, müssen Frontend-i18n-tauglich sein).
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/PROJECT.md` (Active requirements AB-01..AB-08, Key Decisions, Out of Scope).
2. Read `.planning/REQUIREMENTS.md` (19 v1 REQs, traceability table mit Phase-Mapping).
3. Read `.planning/ROADMAP.md` (4 Phasen, Build-Order-Rationale, Discuss-Carry-Overs).
4. Read this file (`STATE.md`) — current position.
5. Optional: `research/SUMMARY.md` für TL;DR der Architektur- und Risiko-Entscheidungen.

**Next command**: `/gsd:verify-phase 03` — Phase-3-Closure-Verification (alle 6 Plans sind durch; alle 4 SCs sind in 03-VALIDATION.md durch konkrete Test-IDs erfüllt; D-Phase3-18 Regression-Lock final 0-Diff verifiziert; cargo test --workspace grün mit 336 service_impl + 32 shifty_bin = 368 passing tests).

---

*State initialized: 2026-05-01 after roadmap creation*
*Last updated: 2026-05-02 (Phase 03 Plan 06 — Wave-5 Final-Tests + Phase-Closure complete; all 6 Phase-3 plans done; awaiting `/gsd:verify-phase 03`)*
