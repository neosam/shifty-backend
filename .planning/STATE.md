---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Frontend rest-types Konsolidierung
status: ready_to_complete
last_updated: "2026-05-07T19:10:00.000Z"
last_activity: 2026-05-07 -- Phase 7 verified, milestone closure-ready
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 6
  completed_plans: 6
  percent: 100
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format — v1.0 + v1.1 archived; v1.2 inline expanded)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.1-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped**: v1.1 Slot Capacity & Constraints (2026-05-04)
- **Current milestone**: v1.2 Frontend rest-types Konsolidierung (closure-ready 2026-05-07, awaiting `/gsd-complete-milestone v1.2`)
- **Current focus**: Milestone-Closure — Phase 6 (5/5, 8/8 V-Truths) und Phase 7 (1/1, 4/4 Success Criteria) abgeschlossen. Nächster Schritt: `/gsd-complete-milestone v1.2` zur Archivierung; danach `/gsd-new-milestone v1.3` für Frontend-Abwesenheiten-Maske (Seed-Trigger erfüllt).

## Current Position

Phase: 7
Plan: 1/1 complete
Status: Milestone closure-ready
Last activity: 2026-05-07 — Phase 7 verified (User-UAT auf Integrationsumgebung + lokale cargo check/test Re-Verifikation)

- **Backend baseline**: `cargo check --workspace` GREEN (32 s); `cargo test --workspace --no-run` GREEN; 461 Tests grün (v1.1-Baseline). Diese Baseline wird in Phase 7 als Regressions-Schwelle gegen RC-01 verwendet.
- **Frontend baseline**: `shifty-dioxus/` baut gegen seinen eigenen Fork `shifty-dioxus/rest-types/` v1.0.5-dev (1468 Zeilen). Backend-`rest-types` v1.13.0-dev hat 2041 Zeilen — 17 fehlende TO-Structs/Enums, 4 fehlende Felder, fehlende Match-Arme. Phase 6 schließt diese Lücke.
- **Backlog (deferred to v1.3+)**: Frontend Capacity-Editor & `current_paid_count`-Anzeige (FUI-01, FUI-02), `VolunteerWork`/`UnpaidLeave`-UI (FUI-03), `cap_planned_hours_to_expected`-Settings-UI (FUI-04), Min-Paid-Capacity / Skill-Matching (SC-01, SC-02), 04-UAT Test 8 Re-Check, `/gsd:secure-phase 04`, zwei offene Review-Todos.

## v1.2 Phase Plan

| Phase | Goal | Requirements | Status |
|-------|------|--------------|--------|
| 6 — rest-types Unification & Frontend Compile-Through | Backend-`rest-types` als single source of truth verdrahten; Frontend-Fork löschen; alle fehlenden TOs adressieren bis WASM-Compile grün | RT-01, RT-02, RT-03, FC-01, FC-02 | ✓ Complete (5/5 plans, 8/8 V-Truths verified, 2026-05-07) |
| 7 — Runtime Smoke & Regression Safety | `dx serve` startet ohne Panics; manueller Login + Shiftplan-Navigation; Backend-Workspace ohne Regression | FC-03, RC-01 | ✓ Complete (1/1 plan, 4/4 Success Criteria verified, 2026-05-07) |

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

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet — Commits manuell durch User. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`). KEINE `git commit`/`git add` aus Agents heraus.
- **NixOS**: Tools wie `sqlx-cli` via `nix develop` (NICHT `nix-shell`, shell.nix kaputt). DB-Befehle: `sqlx database reset` ist DESTRUCTIVE → für additive Migrationen `sqlx migrate run`. Für WASM-Builds in `shifty-dioxus/` ggf. `nix develop` für `wasm32-unknown-unknown`-Toolchain + `dx`/Tailwind.
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert. (Für v1.2 nicht relevant — keine Reporting-Änderung.)
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. v1.2 fügt KEIN neues sichtbares UI hinzu (no-op-Match-Arme sind ausdrücklich ok), daher keine i18n-Pflicht. Falls ein Plan ausnahmsweise Text rendert, en/de/cs alle drei.
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query. (Backend-seitig in v1.2 nur `rest-types`-Cargo.toml-Anpassungen erwartet — keine Service/DAO-Edits.)
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Plan-File `<objective>` darf CONTEXT.md-Tier-Hints overriden (Phase-5-D-12-Präzedenz).
- **rest-types-Cross-Crate-Konstruktion**: Backend-`rest-types/Cargo.toml` hat ein `service-impl`-Feature, das auf das `service`-Crate zeigt. Frontend MUSS dieses Feature OFF lassen (`default-features = false`) — sonst zieht es das `service`-Crate in den WASM-Build und reißt die Toolchain auseinander. v1.2-Phase-6 muss verifizieren, dass das `default-features = false`-Setting tatsächlich greift.

### Open Issues / Tech Debt for v1.3+ (post-v1.2)

- **Frontend User-facing Closure** — UI-Anzeige `current_paid_count`/`max_paid_employees`, Capacity-Editor in Slot-Settings, sichtbare `VolunteerWork`/`UnpaidLeave`-Rendering, `cap_planned_hours_to_expected`-Settings-UI. v1.2 macht den Compile-Pfad frei; v1.3 baut die UI darauf.
- **Min-Paid-Capacity / Skill-Matching** — weitere Slot-Constraints als künftige Backend-Features gemerkt (SC-01, SC-02).
- **04-UAT Test 8** (idempotenter Cutover-Re-Run): bei nächster Cutover-Phase neu prüfen.
- **`/gsd:secure-phase 04`** — als bewusstes Skip akzeptiert; Compliance separat klären falls gefordert.
- **Zwei offene Review-Todos** (`list_user_invitations` silent-empty, OIDC `silentRenewIframe`) — eigener Todo-Lifecycle.
- **Drift-Detection-Skript** als langfristige Versicherung — sobald Phase 6 die strukturelle Lösung (eine Crate) etabliert hat, ist ein CI-Diff-Skript überflüssig. Falls die Konsolidierung aus irgendeinem Grund nicht kompletter Erfolg ist, wäre ein `compare-rest-types.sh` der Fallback (CONCERNS §1 "Fix Approach Option 1").

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/MILESTONES.md` (alle bisher geshipten Milestones — v1.0 + v1.1)
2. Read `.planning/ROADMAP.md` (collapsed v1.0/v1.1, expanded v1.2 mit Phasen 6 + 7)
3. Read this file (`STATE.md`) — current position
4. Read `.planning/REQUIREMENTS.md` — v1.2-Requirements (RT/FC/RC) und Traceability
5. Read `.planning/codebase/frontend/CONCERNS.md` §1 — die konkrete Drift-Inventur, die Phase 6 abarbeiten muss
6. Optional: `.planning/codebase/frontend/INTEGRATIONS.md` — `api.rs`-Endpoint-Map und OIDC-Notes für Phase-7-UAT-Setup

**Next command**: `/gsd-complete-milestone v1.2` — archiviert v1.2 (ROADMAP-Block in `milestones/v1.2-ROADMAP.md`, REQUIREMENTS in `milestones/v1.2-REQUIREMENTS.md`, MILESTONES.md-Eintrag, ROADMAP.md-Reorganisation, PROJECT.md Evolution-Review). Danach `/gsd-new-milestone v1.3` für Frontend Abwesenheiten-Maske — der Seed `seeds/abwesenheiten-frontend-milestone.md` matcht beim Start automatisch.

---

*State updated: 2026-05-07 — v1.2 milestone closure-ready. Beide Phasen 6 und 7 abgeschlossen, alle 7 v1.2-Requirements (RT-01..03, FC-01..03, RC-01) abgehakt. Bereit für /gsd-complete-milestone v1.2 → /gsd-new-milestone v1.3.*
