---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: ready_to_plan
last_updated: "2026-05-01T22:11:25.726Z"
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 9
  completed_plans: 4
  percent: 44
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

Phase: 2
Plan: Not started

- **Current milestone**: Range-Based Absence Management
- **Current phase**: 1 — Absence Domain Foundation
- **Current plan**: none yet (awaiting `/gsd:discuss-phase` then `/gsd:plan-phase 1`)
- **Status**: pending
- **Last action**: roadmap_created (2026-05-01)
- **Progress**: Phase 0/4 complete (0%) — `[----]`

## Performance Metrics

| Metric | Value |
|---|---|
| Phases complete | 0 / 4 |
| Plans complete | 0 / 0 (no plans drafted yet) |
| Requirements mapped | 19 / 19 |
| Requirements complete | 0 / 19 |
| Open discuss-phase decisions | 9 (see ROADMAP.md "Discuss-Phase Carry-Overs") |

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

- [ ] `/gsd:discuss-phase` für Phase 1 durchlaufen (Domain-Naming, Kategorie-Scope, lazy-on-read formalisieren).
- [ ] `/gsd:plan-phase 1` aufrufen, sobald Phase-1-Discuss-Punkte geklärt sind.
- [ ] Production-Data-Profile lauffähig vorbereiten (für Phase 4, kann optional schon parallel laufen — read-only).

### Blockers

Keine harten Blocker für Phase 1. Plan-Phase 1 wartet nur auf `/gsd:discuss-phase`-Resolution für die zwei phase-1-blockierenden Decisions (Domain-Naming, Kategorie-Scope).

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

**Next command**: `/gsd:discuss-phase` (zur Resolution der Phase-1-blockierenden Discuss-Carry-Overs), dann `/gsd:plan-phase 1`.

---

*State initialized: 2026-05-01 after roadmap creation*
*Last updated: 2026-05-01*
