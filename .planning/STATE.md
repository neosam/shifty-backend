---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: ready_to_plan
last_updated: "2026-05-02T20:12:53.614Z"
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 15
  completed_plans: 8
  percent: 53
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

Phase: 3
Plan: Not started

- **Current milestone**: Range-Based Absence Management
- **Current phase**: 02 — Reporting Integration & Snapshot Versioning ✅
- **Current plan**: 02-04 ✅ Atomic Wave-2 Snapshot-Bump + Reporting-Switch (atomar in jj-Change `39be1b73`)
- **Status**: phase-2-complete (next: Phase-3 oder Phase-4 — siehe ROADMAP.md)
- **Last action**: 02-04-execute-complete (2026-05-02T05:14:00Z)
- **Progress**: Phase 1/4 complete; Plans 8/9 complete (89%) — `[####]`

## Performance Metrics

| Metric | Value |
|---|---|
| Phases complete | 1 / 4 |
| Plans complete | 8 / 9 (Phase 1 vollstaendig + Phase 2 vollstaendig 01..04) |
| Requirements mapped | 19 / 19 |
| Requirements complete | 14 / 19 (Phase-1-vollstaendig + Phase-2: SNAP-01/SNAP-02/REP-01/REP-02/REP-03/REP-04) |
| Open discuss-phase decisions | 9 (see ROADMAP.md "Discuss-Phase Carry-Overs") |

### Phase-Plan Execution Log

| Phase | Plan | Duration | Tasks | Files | Date |
|-------|------|----------|-------|-------|------|
| 02 | 01 | ~12min | 3 (+ 1 Phase-1-Rule-3-Fix) | 8 (5 neu Wave-0 + 3 Phase-1-Fix) | 2026-05-02 |
| 02 | 02 | ~11min | 3 | 5 (1 service, 1 service_impl, 2 test, 1 main.rs) | 2026-05-02 |
| 02 | 03 | ~13min | 3 | 13 (6 neu + 7 patches) | 2026-05-02 |
| 02 | 04 | ~35min | 3 (+ 1 Rule-1-Auto-Fix; atomar) | 11 (alle in jj-Change 39be1b73) | 2026-05-02 |

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

### Open Todos

- [x] `/gsd:execute-plan 02-02` — Wave 1 derive_hours_for_range (abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 02-03` — Feature-Flag-Infrastruktur (abgeschlossen 2026-05-02).
- [x] `/gsd:execute-plan 02-04` — Wave 2 atomarer Snapshot-Bump-Commit (abgeschlossen 2026-05-02, jj-Change `39be1b73`).
- [ ] Production-Data-Profile lauffähig vorbereiten (für Phase 4, kann optional schon parallel laufen — read-only).
- [ ] Phase-1-Hygiene: Lokale `localdb.sqlite3`-Drift fixen (siehe `.planning/phases/02-.../deferred-items.md`).

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

**Next command**: Phase-2-Verification — `/gsd:verify-phase 02` oder weiter mit Phase 3 / Phase 4 Plan-Erstellung (`/gsd:plan-phase 03` oder `/gsd:plan-phase 04`).

---

*State initialized: 2026-05-01 after roadmap creation*
*Last updated: 2026-05-02 (Phase 02 Plan 04 — Atomic Wave-2 Snapshot-Bump + Reporting-Switch complete)*
