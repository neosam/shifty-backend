---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-05-02T04:12:41Z"
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 9
  completed_plans: 5
  percent: 55
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
Plan: 2 of 4 (Plan 01 complete; Plan 02 next)

- **Current milestone**: Range-Based Absence Management
- **Current phase**: 02 — Reporting Integration & Snapshot Versioning
- **Current plan**: 02-01 ✅ Wave-0 Test-Scaffolding (committed in d8dad0aa, f85f4a3f, 0eeff84c, 726e919c)
- **Status**: phase-2-wave-0-complete (next: 02-02 or 02-03 Wave 1)
- **Last action**: 02-01-execute-complete (2026-05-02T04:12:41Z)
- **Progress**: Phase 0/4 complete; Plans 5/9 complete (55%) — `[##--]`

## Performance Metrics

| Metric | Value |
|---|---|
| Phases complete | 0 / 4 |
| Plans complete | 5 / 9 (Phase 1 vollstaendig + Phase 2 Plan 01 Wave 0) |
| Requirements mapped | 19 / 19 |
| Requirements complete | 6 / 19 (Phase-1-vollstaendig + Phase-2-Wave-0 partial: SNAP-01/SNAP-02/REP-02 scaffolded) |
| Open discuss-phase decisions | 9 (see ROADMAP.md "Discuss-Phase Carry-Overs") |

### Phase-Plan Execution Log

| Phase | Plan | Duration | Tasks | Files | Date |
|-------|------|----------|-------|-------|------|
| 02 | 01 | ~12min | 3 (+ 1 Phase-1-Rule-3-Fix) | 8 (5 neu Wave-0 + 3 Phase-1-Fix) | 2026-05-02 |

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

- [ ] `/gsd:execute-plan 02-02` (oder 02-03) — Wave-1-Plans (parallelisierbar).
- [ ] Production-Data-Profile lauffähig vorbereiten (für Phase 4, kann optional schon parallel laufen — read-only).

### Wave-2-Forcing-State (aus Plan 02-01)

Wave 0 hat ein **build-time Forcing** etabliert, das Wave 2 zwingt, Snapshot-Bump und UnpaidLeave-Variante atomar zu liefern:

- **Pin-Map-Test ROT** (`test_snapshot_schema_version_pinned`): erwartet `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`, ist aktuell 2. Wird GRUEN sobald Wave 2 das Bump macht.
- **Compiler-Exhaustive-Match-Test GRUEN** (`test_billing_period_value_type_surface_locked`): listet alle 11 aktuellen Varianten. Wird COMPILE-ERROR sobald Wave 2 die `BillingPeriodValueType::UnpaidLeave`-Variante hinzufuegt -- in der Test-Datei wartet ein auskommentierter Arm zur Aktivierung.

### Blockers

Keine harten Blocker. Wave 1 (Plan 02-02 fuer FeatureFlagService + Plan 02-03 fuer derive_hours_for_range) kann sofort starten.

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

**Next command**: `/gsd:execute-plan 02-02` oder `/gsd:execute-plan 02-03` (Wave-1, parallelisierbar).

---

*State initialized: 2026-05-01 after roadmap creation*
*Last updated: 2026-05-02 (Phase 02 Plan 01 — Wave-0 Test-Scaffolding complete)*
