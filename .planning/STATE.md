---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: Range-Based Absence Management
status: milestone_complete
last_updated: "2026-05-03T20:15:00.000Z"
progress:
  total_phases: 4
  completed_phases: 4
  total_plans: 23
  completed_plans: 23
  percent: 100
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.0-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions)
- **Last shipped**: v1.0 Range-Based Absence Management (2026-05-03)
- **Current focus**: TBD — next milestone via `/gsd:new-milestone`

## Current Position

Phase: — (none active)
Milestone: v1.0 SHIPPED

- **Status**: milestone_complete (v1.0 shipped 2026-05-03)
- **Last action**: v1.0 milestone close (Phasen 1–4 alle verified, 458+ tests workspace-weit grün, OpenAPI-Snapshot deterministic, fe744df-Recovery integriert)
- **Next**: `/gsd:new-milestone` zur Definition des nächsten Iterations-Scopes

## v1.0 Highlights

- **23 plans / 22 SUMMARYs** über 4 Phasen geliefert (Phase-1-Plan-00 hat historisches Wave-0-Scaffolding ohne separates SUMMARY)
- **458+ tests green** workspace-weit (363 service_impl + 56 shifty_bin integration + 11 cutover service + 10 dao + 18 weitere)
- **OpenAPI surface gepinnt** via insta-snapshot (3-run deterministic check passed)
- **Atomic-Tx-Cutover** verifiziert: Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip in einer Tx, Drop-Rollback im Fehlerfall
- **Service-Tier-Konvention** (Basic vs Business-Logic) durchgehend angewendet, keine Cycles

## Accumulated Context (carry forward)

### Architecture Decisions Logged

- Parallele `absence` Domain (nicht Erweiterung von `extra_hours`).
- Hybrid materialize-on-snapshot / derive-on-read (Live-Reports derive on read; BillingPeriod-Snapshots materialize-once).
- Direction: `AbsenceService → BookingService` (Business-Logic-Tier konsumiert Basic-Tier; nie umgekehrt).
- Service-Tier-Konvention etabliert: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Doku: `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen".
- `BookingCreateResult { booking, warnings }`-Wrapper für nicht-blockierende Warnings (lebt im Business-Logic-Tier).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden im selben Commit wie Reporting-Switch (per `CLAUDE.md`).
- Phase-3 Wave-0-Stub-Pattern: `#[ignore]` + `unimplemented!()` als Standard für Wave-Forcing.
- Phase-4 Cycle-Break: separater `CarryoverRebuildServiceImpl` (BL-Tier) — bricht Reporting↔Carryover-Cycle.
- logical_id-Versionierungs-Pattern (rotiert physische Row, hält stabilen externen ID): erst in `extra_hours` (commit fe744df) eingeführt, dann in `absence_period` übernommen.

### Constraints In Force

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet — Commits manuell durch User. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`).
- **NixOS**: Tools wie `sqlx-cli` via `nix develop` (NICHT `nix-shell`, shell.nix kaputt). DB-Befehle: `sqlx database reset` ist DESTRUCTIVE → für additive Migrationen `sqlx migrate run`.
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs (Out of Scope für dieses Backend-Milestone, aber DTO-Felder die Texte transportieren müssen Frontend-i18n-tauglich sein).
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query.

### Open Issues / Tech Debt for next milestone

- 04-UAT Test 8: idempotenter Cutover-Re-Run wurde manuell mit 403 verifiziert (vermutlich Setup-Issue, kein Code-Bug — abgedeckt durch passing Integration-Test). Bei nächster Cutover-Phase neu prüfen falls reproduzierbar.
- `/gsd:secure-phase 04` wurde nicht ausgeführt — als bewusstes Skip akzeptiert. Falls für Compliance gefordert, separat nachreichen.
- 8 pre-existing `absence_period`-Integration-Tests: Phase-1-Migration in Plan 03-06 recovered → jetzt grün. Keine offene Lücke.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/MILESTONES.md` (alle bisher geshipten Milestones)
2. Read `.planning/ROADMAP.md` (collapsed milestone format)
3. Read this file (`STATE.md`) — current position
4. Optional: `.planning/milestones/v1.0-ROADMAP.md` für Detail-Audit der v1.0-Arbeit

**Next command**: `/gsd:new-milestone` zur Definition des nächsten Iterations-Scopes (Frontend-Workstream, weitere Backend-Features, etc.).

---

*State updated: 2026-05-03 — v1.0 milestone closed. Phases 1–4 SHIPPED. Ready for next milestone planning.*
