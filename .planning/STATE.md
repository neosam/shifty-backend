---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-05-02T04:36:01.750Z"
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 9
  completed_plans: 6
  percent: 67
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

Phase: 02 (reporting-integration-snapshot-versioning) — EXECUTING
Plan: 3 of 4 (Plan 01 + 02 complete; Plan 03 next)

- **Current milestone**: Range-Based Absence Management
- **Current phase**: 02 — Reporting Integration & Snapshot Versioning
- **Current plan**: 02-02 ✅ AbsenceService::derive_hours_for_range (committed in 8fafb6ef, 3e371b06, ae7d0642)
- **Status**: phase-2-wave-1-complete (next: 02-03 oder 02-04)
- **Last action**: 02-02-execute-complete (2026-05-02T04:33:20Z)
- **Progress**: Phase 0/4 complete; Plans 6/9 complete (67%) — `[##--]`

## Performance Metrics

| Metric | Value |
|---|---|
| Phases complete | 0 / 4 |
| Plans complete | 6 / 9 (Phase 1 vollstaendig + Phase 2 Plans 01+02) |
| Requirements mapped | 19 / 19 |
| Requirements complete | 7 / 19 (Phase-1-vollstaendig + Phase-2-Wave-0 partial: SNAP-01/SNAP-02/REP-02 scaffolded; Phase-2-Wave-1: REP-01) |
| Open discuss-phase decisions | 9 (see ROADMAP.md "Discuss-Phase Carry-Overs") |

### Phase-Plan Execution Log

| Phase | Plan | Duration | Tasks | Files | Date |
|-------|------|----------|-------|-------|------|
| 02 | 01 | ~12min | 3 (+ 1 Phase-1-Rule-3-Fix) | 8 (5 neu Wave-0 + 3 Phase-1-Fix) | 2026-05-02 |
| 02 | 02 | ~11min | 3 | 5 (1 service, 1 service_impl, 2 test, 1 main.rs) | 2026-05-02 |

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
- Direction: `BookingService → AbsenceService` (nie umgekehrt; vermeidet Circular Dep).
- `BookingCreateResult { booking, warnings }`-Wrapper für nicht-blockierende Warnings.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit wie der Reporting-Switch (per `CLAUDE.md`).

### Open Todos

- [x] `/gsd:execute-plan 02-02` — Wave 1 derive_hours_for_range (abgeschlossen 2026-05-02).
- [ ] `/gsd:execute-plan 02-03` — Feature-Flag-Infrastruktur (kann sofort starten).
- [ ] `/gsd:execute-plan 02-04` — Wave 2 atomarer Snapshot-Bump-Commit (depends auf 02-03).
- [ ] Production-Data-Profile lauffähig vorbereiten (für Phase 4, kann optional schon parallel laufen — read-only).

### Wave-2-Forcing-State (aus Plan 02-01)

Wave 0 hat ein **build-time Forcing** etabliert, das Wave 2 zwingt, Snapshot-Bump und UnpaidLeave-Variante atomar zu liefern:

- **Pin-Map-Test ROT** (`test_snapshot_schema_version_pinned`): erwartet `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`, ist aktuell 2. Wird GRUEN sobald Wave 2 das Bump macht.
- **Compiler-Exhaustive-Match-Test GRUEN** (`test_billing_period_value_type_surface_locked`): listet alle 11 aktuellen Varianten. Wird COMPILE-ERROR sobald Wave 2 die `BillingPeriodValueType::UnpaidLeave`-Variante hinzufuegt -- in der Test-Datei wartet ein auskommentierter Arm zur Aktivierung.

### Blockers

Keine harten Blocker. Plan 02-03 (Feature-Flag-Infra) kann sofort starten; Plan 02-04 wartet auf 02-03 (Compile-Dependency auf FeatureFlagService).

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

**Next command**: `/gsd:execute-plan 02-03` (FeatureFlagService-Infra; Wave 2 in 02-04 wartet auf 02-03).

---

*State initialized: 2026-05-01 after roadmap creation*
*Last updated: 2026-05-02 (Phase 02 Plan 02 — Wave-1 derive_hours_for_range complete)*
