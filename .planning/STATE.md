---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Frontend Abwesenheiten + UI-Closure-Restanten
status: executing
last_updated: "2026-05-08T05:12:50.496Z"
last_activity: 2026-05-08 -- Phase 8 planning complete
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 6
  completed_plans: 0
  percent: 0
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format — v1.0, v1.1, v1.2 archived)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.2-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped**: v1.2 Frontend rest-types Konsolidierung (2026-05-07)
- **Current milestone**: v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (gestartet 2026-05-07)
- **Current focus**: v1.3 — Frontend-Abwesenheiten-Maske gegen `/absence-period` als Hauptthema; UI-Closure FUI-01..04 sekundär

## Current Position

Phase: 8 — Absence-CRUD-Page Foundation (context gathered)
Plan: —
Status: Ready to execute
Last activity: 2026-05-08 -- Phase 8 planning complete

## Shipped Milestones

### v1.2 — Frontend rest-types Konsolidierung (2026-05-07)

- **2 Phases (6, 7), 6 Plans**, 466 tests green workspace-weit
- Backend-`rest-types` als single source of truth verdrahtet; Frontend-Fork gelöscht; WASM-Build grün
- Phase 7 als Subsumption-Closure-Phase abgeschlossen (User-UAT auf Integrationsumgebung + Phase-6-V-Truth-Reuse)
- 8/8 V-Truths (P6) + 4/4 Success Criteria (P7) verified

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

**v1.2 (Phasen 6–7 — Frontend rest-types Konsolidierung):**

- **Cross-Workspace-Path-Dep mit `default-features = false`** (Plan 06-01): `shifty-dioxus/Cargo.toml` referenziert die Backend-`rest-types`-Crate via `path = "../rest-types"` mit explizitem `default-features = false`, um den `service-impl`-Feature-Pull-In zu vermeiden, der den WASM-Build durch das `service`-Crate sprengen würde.
- **Wave-0 Backend-Prep vor Cargo-Swap** (Plan 06-00): Pre-Migration der Invitation-DTO-Familie mit konsistentem Derive-Set (`Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema`) macht den Wave-1-Cargo-Swap mechanisch sauber. Backend-Derive-Erweiterung statt Frontend-Hack ist die korrekte Lösung für `assert_eq!`-Tests.
- **State-Editor-Mirror für nicht-editierte Felder** (Plan 06-04): `SlotEditItem` muss `max_paid_employees` als Field-Mirror tragen, weil sonst der Edit-Roundtrip (`SlotTO -> SlotEditItem -> SlotTO`) den Backend-Wert auf `None` setzt. Field-Mirror mit Default ist Pflicht für Datenintegrität, auch wenn das Feld in der aktuellen Phase nicht editiert wird.
- **Subsumption-Verification-Pattern** (Phase 7): Reine UAT-/Smoke-Phasen ohne eigenen Code-Change können in einem einzigen Plan-Summary mit Verweis auf die vorhergehende Phase abgeschlossen werden. Voraussetzungen: (1) automatische Test-Kriterien sind in der Vorgänger-Phase grün dokumentiert; (2) manuelle UAT-Kriterien sind vom User auf einer realen Umgebung verifiziert; (3) beide Belege werden in der Closure-Phase explizit referenziert.
- **No-op-Match-Arm-Pattern** (Plan 06-04): Für Phasen, deren Scope explizit "keine User-facing Features" ist, sind exhaustive Match-Arme via `WarningTO::PaidEmployeeLimitExceeded => rsx! { "" }` ausdrücklich erlaubt. UI-Closure folgt im nächsten Milestone.

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
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. v1.3 wird Frontend-Abwesenheiten-Maske mit signifikantem i18n-Volumen einführen — gleichzeitig in allen drei Locales pflegen (kein Locale::En-statt-Locale::De-Bug).
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Plan-File `<objective>` darf CONTEXT.md-Tier-Hints overriden (Phase-5-D-12-Präzedenz).
- **rest-types-Cross-Crate-Konstruktion** (etabliert in v1.2): Backend-`rest-types/Cargo.toml` hat ein `service-impl`-Feature, das auf das `service`-Crate zeigt. Frontend MUSS dieses Feature OFF lassen (`default-features = false`) — sonst zieht es das `service`-Crate in den WASM-Build und reißt die Toolchain auseinander.

### Open Issues / Tech Debt for v1.3+ (live backlog)

- **Frontend Abwesenheiten-Maske** (FUI-A-01..09) — neue Top-Level-Maske gegen `/absence-period` REST-API (HR-Sicht + Employee-Self-Service); siehe `notes/abwesenheiten-frontend-context.md` und `seeds/abwesenheiten-frontend-milestone.md`. **Hauptthema für v1.3.**
- **Frontend User-facing Closure** (FUI-01..04) — sichtbares `current_paid_count`/`max_paid_employees`-Rendering, Capacity-Editor in Slot-Settings, sichtbare `VolunteerWork`/`UnpaidLeave`-Rendering, `cap_planned_hours_to_expected`-Settings-UI. v1.2 hat den Compile-Pfad freigemacht; v1.3 baut die UI darauf.
- **Min-Paid-Capacity / Skill-Matching** (SC-01, SC-02) — weitere Slot-Constraints als künftige Backend-Features gemerkt.
- **04-UAT Test 8** (idempotenter Cutover-Re-Run): bei nächster Cutover-Phase neu prüfen.
- **`/gsd:secure-phase 04`** — als bewusstes Skip akzeptiert; Compliance separat klären falls gefordert.
- **Zwei offene Review-Todos** (`list_user_invitations` silent-empty, OIDC `silentRenewIframe`) — eigener Todo-Lifecycle.

### Phase-Verzeichnis-Cleanup (optional)

`.planning/phases/01-04` (v1.0), `.planning/phases/05` (v1.1), `.planning/phases/06-07` (v1.2) liegen alle noch im aktiven `phases/`-Verzeichnis. `gsd-sdk milestone.complete` hat sie nicht automatisch in `milestones/v1.X-phases/` verschoben (`archived.phases: false`). Bei Bedarf manuell via `/gsd-cleanup` oder `mkdir milestones/v1.X-phases && mv phases/...` archivieren.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/MILESTONES.md` (geshipte Milestones — v1.0, v1.1, v1.2)
2. Read `.planning/ROADMAP.md` (v1.3-Phasen aktiv; v1.0–v1.2 collapsed)
3. Read `.planning/REQUIREMENTS.md` (v1.3-Scope, REQ-IDs, Coverage)
4. Read this file (`STATE.md`) — current position
5. Read `.planning/notes/abwesenheiten-frontend-context.md` — v1.3 Briefing
6. Read `.planning/seeds/abwesenheiten-frontend-milestone.md` — Sub-Phasen-Skizze
7. Read `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md` — Backend-Integrations-Brief
8. Read `shifty-dioxus/shifty-design/project/absences.jsx` — Mockup (729 Zeilen JSX)

**Next command**: `/gsd-discuss-phase 8` — gather context für die erste v1.3-Phase (Absence-CRUD-Page Foundation), oder `/gsd-plan-phase 8` für direkten Plan-Einstieg.

---

*State updated: 2026-05-07 — v1.3 gestartet (Frontend Abwesenheiten + UI-Closure-Restanten). Phasen 8+ definiert in ROADMAP.md.*
