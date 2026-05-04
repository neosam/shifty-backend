---
gsd_state_version: 1.0
milestone: none
milestone_name: "(planning next)"
status: milestone_complete
last_shipped: v1.1
last_updated: "2026-05-04T09:00:00Z"
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format — v1.0 + v1.1 archived)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.1-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions)
- **Last shipped**: v1.1 Slot Capacity & Constraints (2026-05-04)
- **Current focus**: Planning next milestone via `/gsd:new-milestone`

## Current Position

Milestone: v1.1 SHIPPED 2026-05-04. No active phase. No active plan.

- **Status**: milestone_complete. Workspace `cargo build --workspace` GREEN; `cargo test --workspace` 461 tests pass; `cargo run` boots cleanly to `127.0.0.1:3000`.
- **Last action** (2026-05-04): v1.1 milestone closed. Archive at `milestones/v1.1-ROADMAP.md`. ROADMAP.md kollabiert auf `<details>`-Block. MILESTONES.md mit v1.1-Eintrag erweitert.
- **Next**: `/gsd:new-milestone` zur Scope-Definition. Backlog-Optionen (aus v1.1-Deferred):
  - Frontend-Workstream (shifty-dioxus) — UI für Capacity-Anzeige + Editor
  - Min-Paid-Capacity / Skill-Matching — weitere Slot-Constraints
  - Andere Backend-Themen (Performance, Reporting-UX, Permissions)

## Shipped Milestones

### v1.1 — Slot Capacity & Constraints (2026-05-04)

- **1 Phase, 6 Plans**, 461 tests green (+6 über v1.0-Baseline 455)
- Slots: `max_paid_employees: Option<u8>` mit nicht-blockierender Warning-Emission
- 16/16 D-decisions verified (status: passed, gaps_remaining = [])
- Legacy `POST /booking` + `BookingService::create` unverändert (D-Phase3-18 Regression-Lock gehalten)

### v1.0 — Range-Based Absence Management (2026-05-03)

- **23 plans / 22 SUMMARYs** über 4 Phasen geliefert
- 458+ tests green workspace-weit
- OpenAPI surface gepinnt via insta-snapshot (3-run deterministic check passed)
- Atomic-Tx-Cutover verifiziert (Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip)
- Service-Tier-Konvention (Basic vs Business-Logic) durchgehend angewendet

## Accumulated Context (carry forward)

### Architecture Decisions Logged

**v1.1 (Phase 5 — Slot Paid Capacity Warning):**

- **Warning-Emission-Heart-Pattern** (Plan 05-06): Soft-Warning-Emission im Business-Logic-Tier-Service; insert die Limit-Check-Logik zwischen die existierende Cross-Source-Warning-Emission und das finale `transaction_dao.commit(tx)`. Persistierte Entity in-hand, warnings-Akkumulator in-hand. Kein Rollback (D-07). Helper als private Methode auf einem zweiten `impl<Deps>`-Block; Helper-Signatur: `tx: Deps::Transaction` by-value. Inner cross-service-calls verwenden `Authentication::Full`. D-12-Korrektur: Helper lebt auf `ShiftplanEditServiceImpl` (Business-Logic-Tier), NICHT auf `BookingService` (Basic-Tier per CLAUDE.md + v1.0 D-Phase3-18 Regression-Lock).
- **Wire-Tier-Mirror-Pattern** (Plan 05-05): Additive Service-Tier-Field/Variant landet wire-tier in `rest-types/src/lib.rs` durch 3 Mechanismen: (1) Struct-Feld auf `*TO` + beide `From`-Impls — Backward-Compat via `#[serde(default)]`; (2) Enum-Variant am Ende mit `#[serde(rename_all = "snake_case")]`-Auto-Tag + matching `From`-Arm (rustc enforced Exhaustivität); (3) Cascade-DTOs erben automatisch via `Vec<*TO>`-Embedding.
- **Wave-Coupling-Pattern** (Plan 05-02): Wenn ein additiver Variant zu einem Domain-Enum ein exhaustive downstream `match` ohne Wildcard bricht, schedule Producer-Plan + Consumer-Plan in der GLEICHEN Wave; Standalone-Akzeptanz reduziert sich auf `cargo build -p {producer-crate}`.
- **Read-Aggregation-Pattern** (Plan 05-04): `current_paid_count: u8` wird inline in `build_shiftplan_day` aus bereits resolvten `slot_bookings` per `.filter(|sb| sb.sales_person.is_paid.unwrap_or(false)).count().min(u8::MAX as usize) as u8` abgeleitet. Als `u8` (nicht `Option<u8>`).
- **Forward-Compat-Shim-Pattern (Rule 3)** (Plan 05-01, 05-03): Wenn DAO-Feld eine Phase vor seinem Service-Layer-Mirror landet, hardcode `None` in `From<&Service::Slot> for SlotEntity` und im zentralen Test-Fixture mit Inline-Kommentar auf Folge-Plan.
- **Sequential-Wave-Friction-Mitigation** (Plan 05-03): Wenn parallel-geplante Wave-Plans sequenziell ausgeführt werden, Rule-3-Shims in OUT-OF-SCOPE-Sites mit Folge-Plan-Kommentar einsetzen statt Wave-Reorder.
- **D-12-Override-Präzedenz**: Wenn CONTEXT.md einen Tier-Hint liefert, der gegen CLAUDE.md Service-Tier-Konvention verstößt, **das Plan-File `<objective>` overrid**et CONTEXT.md explizit. Service-Tier-Konvention ist die durchsetzungsstärkere Regel.

**v1.0 (Phasen 1–4):**

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
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs (Out of Scope für Backend-Milestones, aber DTO-Felder die Texte transportieren müssen Frontend-i18n-tauglich sein — Phase 5 D-08 hält das durch strukturierte Variants ohne Strings).
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. CLAUDE.md ist die durchsetzungsstärkere Regel — Plan-File `<objective>` darf CONTEXT.md-Tier-Hints overriden, wenn diese gegen die Konvention verstoßen würden (Phase-5-D-12-Präzedenz).

### Open Issues / Tech Debt for next milestone

- **Frontend-Workstream (shifty-dioxus)** — UI-Anzeige `current_paid_count` / `max_paid_employees` per Slot + Capacity-Editor in Slot-Settings stehen aus.
- **Min-Paid-Capacity / Skill-Matching** — weitere Slot-Constraints als künftige Backend-Features gemerkt.
- **04-UAT Test 8** (idempotenter Cutover-Re-Run): manuell mit 403 verifiziert (vermutlich Setup-Issue, kein Code-Bug — abgedeckt durch passing Integration-Test). Bei nächster Cutover-Phase neu prüfen falls reproduzierbar.
- **`/gsd:secure-phase 04`** wurde nicht ausgeführt — als bewusstes Skip akzeptiert. Falls für Compliance gefordert, separat nachreichen.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/MILESTONES.md` (alle bisher geshipten Milestones — v1.0 + v1.1)
2. Read `.planning/ROADMAP.md` (collapsed milestone format)
3. Read this file (`STATE.md`) — current position
4. Optional: `.planning/milestones/v1.1-ROADMAP.md` für Detail-Audit der v1.1-Arbeit (Phase 5 D-decisions, Patterns, Wave-Topologie)

**Next command**: `/gsd:new-milestone` zur Scope-Definition des nächsten Iterations-Themas (Frontend-Workstream, weitere Backend-Features, etc.).

---

*State updated: 2026-05-04 — v1.1 milestone closed. Phase 5 SHIPPED. Ready for next milestone planning.*
