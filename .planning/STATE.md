---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Slot Capacity & Constraints
status: phase_in_progress
last_updated: "2026-05-04T06:02:17Z"
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 6
  completed_plans: 2
  percent: 33
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.0-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions)
- **Last shipped**: v1.0 Range-Based Absence Management (2026-05-03)
- **Current focus**: v1.1 Phase 5 Slot Paid Capacity Warning — planning complete, ready for execute

## Current Position

Phase: 05 (slot-paid-capacity-warning) — EXECUTING
Plan: 3 of 6 (05-01 + 05-03 done; Wave 2 remainder = 05-04 next, then Wave 3)
Milestone: v1.1 Slot Capacity & Constraints (in progress)

- **Status**: phase_in_progress (Phase 5 / Milestone v1.1) — Wave 2 partially complete (2/6 plans, 33%)
- **Last action** (2026-05-04): Plan 05-03 executed. `service::slot::Slot` now carries `max_paid_employees: Option<u8>`; both `From` impls round-trip the field; Plan 05-01's 2 Rule-3 shims replaced with real flow. 5 owned test files migrated (slot/booking/block/absence/shiftplan_edit). 3 new service tests verify create-with-limit, update-changes-limit, update-clears-limit. Permission Pflicht-Test covered transitively via existing `SHIFTPLANNER_PRIVILEGE` gate. 3 new Rule-3 forward-compat shims placed at OUT-OF-SCOPE compile-blocker sites: `service_impl/src/test/shiftplan.rs` (Plan 05-04 owns), `rest-types/src/lib.rs::From<&SlotTO>` (Plan 05-05 owns), `shifty_bin/.../booking_absence_conflict.rs` (integration). 451+ tests green workspace-wide (3 new + 448 baseline).
- **Next**: Plan 05-04 — Shiftplan-View read aggregation: `ShiftplanSlot.current_paid_count: u8` computed inline in `build_shiftplan_day` + 4 read tests + extension of `test/shiftplan.rs` (the Rule-3 mechanical shim is already in place; Plan 04 still owns the read-aggregation logic and new tests).

## v1.0 Highlights

- **23 plans / 22 SUMMARYs** über 4 Phasen geliefert (Phase-1-Plan-00 hat historisches Wave-0-Scaffolding ohne separates SUMMARY)
- **458+ tests green** workspace-weit (363 service_impl + 56 shifty_bin integration + 11 cutover service + 10 dao + 18 weitere)
- **OpenAPI surface gepinnt** via insta-snapshot (3-run deterministic check passed)
- **Atomic-Tx-Cutover** verifiziert: Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip in einer Tx, Drop-Rollback im Fehlerfall
- **Service-Tier-Konvention** (Basic vs Business-Logic) durchgehend angewendet, keine Cycles

## Accumulated Context (carry forward)

### Architecture Decisions Logged

- **Phase-5-Plan-01 Foundation:** Nullable Slot-Capacity-Spalte landet ohne Backfill (D-15) — migration kopiert `min_resources`-Pattern, strippt aber `DEFAULT` und `NOT NULL`. Read-Site-Cast `row.col.map(|n| n as u8)` ist die Standard-Konvention für nullable INTEGER → `Option<u8>` (analog zu `min_resources as u8` für nicht-nullable). `update_slot` UPDATE wird für den neuen Knob in-place erweitert (kein Temporal-Replay-Concern für nicht-zeitliche Slot-Konfig); `min_resources`-Gap explizit out-of-scope.
- **Phase-5-Plan-01 Forward-Compat-Shim-Pattern (Rule 3):** Wenn ein DAO-Feld eine Phase vor seinem Service-Layer-Mirror landet, hardcode `None` in `From<&Service::Slot> for SlotEntity` und im zentralen Test-Fixture, mit Inline-Kommentar auf den Folge-Plan. Plan 05-03 muss beide Stellen ersetzen.
- **Phase-5-Plan-03 Service-Tier-Wiring:** `service::Slot` bekommt das neue Feld; `create_slot`/`update_slot` brauchen KEINE direkte Code-Änderung, weil sie bereits `..slot.clone()`-Spread verwenden — das Feld fließt transparent durch. Field bewusst NICHT in `ModificationNotAllowed`-Liste (D-11: in-place mutable). `SHIFTPLANNER_PRIVILEGE`-Gate deckt Permission-Pflicht-Test transitiv ab.
- **Phase-5-Plan-03 Sequential-Wave-2-Friction:** Plan 03 + Plan 04 waren als parallele Wave-2-Siblings geplant (disjoint files). Bei sequenzieller Ausführung blockiert das Test-Target-Compile, weil `cargo test --lib slot::` nur Test-Filter ist, kein Compile-Filter. Lösung: Rule-3-Shim in 3 OUT-OF-SCOPE-Sites (`test/shiftplan.rs`, `rest-types/src/lib.rs`, `shifty_bin/.../booking_absence_conflict.rs`) — minimal mechanisches `max_paid_employees: None` mit Folge-Plan-Kommentar.
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
