---
phase: 08-absence-crud-page-foundation
plan: 01
subsystem: api
tags: [shifty-backend, vacation-balance, dto-foundation, service-trait, bl-tier, rest-types, automock]

# Dependency graph
requires:
  - phase: 01-absence-domain-foundation
    provides: AbsencePeriod-Domain + AbsenceService-Trait als Vorbild für Trait-Form
  - phase: 06-rest-types-unification
    provides: rest-types als single source of truth mit `service-impl`-Feature-Gating
provides:
  - service::vacation_balance::VacationBalanceService Trait (BL-Tier, automock)
  - service::vacation_balance::VacationBalance Domain-Struct (7 Felder)
  - rest_types::VacationBalanceTO Wire-DTO (ToSchema, kein $version)
  - From-Impls TO↔Domain (feature-gated `service-impl`)
affects: [08-02 service-impl + REST + DI, 08-03 OpenAPI snapshot, 08-04 frontend api/state, 08-05 frontend page-components]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cross-Crate Feature-Gating via #[cfg(feature = \"service-impl\")] (etabliert in v1.2, hier wiederverwendet für Phase-8-DTO)"
    - "Read-only Aggregat ohne $version-Field (kein Optimistic-Lock — Aggregat wird stets neu berechnet)"

key-files:
  created:
    - service/src/vacation_balance.rs
    - .planning/phases/08-absence-crud-page-foundation/08-01-SUMMARY.md
  modified:
    - service/src/lib.rs
    - rest-types/src/lib.rs

key-decisions:
  - "VacationBalanceService landet als BL-Tier (D-04): Service kombiniert in Plan 08-02 Cross-Entity-Daten (EmployeeWorkDetailsService + CarryoverService + AbsenceService); Trait selbst trägt keine Dep-Constraints."
  - "VacationBalanceTO ohne $version-Field: read-only Aggregat, daher kein Optimistic-Lock-Konflikt möglich. Abweichung vom AbsencePeriodTO-Pattern bewusst."
  - "Beide From-Impls (TO→Domain und Domain→TO) feature-gated, weil Backend-Roundtrip-Tests in Plan 08-02 die Reverse-Richtung brauchen — gleiches Pattern wie AbsencePeriodTO."
  - "year als u32 (nicht i32): Kalenderjahr ist immer positiv, u32 spart einen Validation-Path."
  - "carryover_days als i32 (nicht u32): konsistent mit CarryoverService.get_carryover().vacation, das negative Überträge prinzipiell zulässt; Plan 08-02 wird konkret prüfen, ob das Backend Negative produziert."

patterns-established:
  - "Wave-1-Foundation-Plan ohne Test-Code: Trait + Domain-Struct + DTO als Interface-Foundation, Tests landen in Wave 2 (Plan 08-02), wo der MockVacationBalanceService konsumiert wird."
  - "automock-Annotation auf BL-Tier-Trait-Surface: identische Signatur zu AbsenceService — `#[automock(type Context=(); type Transaction=dao::MockTransaction;)]`."

requirements-completed: []  # PLAN-FRONTMATTER-DISKREPANZ: Plan-Frontmatter listet FUI-A-04, aber FUI-A-04 ("AbsencePeriodCreateResultTO.warnings[] als nicht-blockierende Hinweisliste") ist ein Frontend-Render-Requirement, das erst Plan 08-05 (WarningList-Component) erfüllt. Dieser Plan baut VacationBalanceService Backend-Foundation (D-03). Keine Requirement-Closure in dieser Wave.

# Metrics
duration: ~12min
completed: 2026-05-08
---

# Phase 08 Plan 01: Absence-CRUD-Page Foundation — Vacation-Balance Trait + DTO Summary

**Service-Trait + Domain-Struct (`VacationBalanceService` / `VacationBalance`) und Wire-DTO (`VacationBalanceTO`) als Wave-1-Interface-Foundation; Plan 08-02 baut Service-Impl + REST + DI gegen genau diese Symbole.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-08T05:55:00Z (approximately, after STATE.md exec-start at 05:52:27Z)
- **Completed:** 2026-05-08T08:35:00Z (UTC, mit cargo-Wartezeiten — wall-clock; pure Edit-Zeit < 5 Min)
- **Tasks:** 2 / 2
- **Files modified:** 3 (1 NEW + 2 modified)

## Accomplishments

- `VacationBalanceService`-Trait in `service/src/vacation_balance.rs` mit `get(sales_person_id, year, ...)` (HR ∨ self) + `get_team(year, ...)` (HR-only) und `automock`-generiertem `MockVacationBalanceService`.
- `VacationBalance`-Domain-Struct mit 7 Feldern entsprechend UI-SPEC `VacationEntitlementCard` (sales_person_id, year, entitled_days, carryover_days, used_days, planned_days, remaining_days), jedes Feld dokumentiert mit Quelle der Berechnung (Plan 08-02 implementiert die Aggregation).
- `VacationBalanceTO` in `rest-types/src/lib.rs` mit `Clone + Debug + Serialize + Deserialize + ToSchema + PartialEq` und beiden bidirektionalen From-Impls hinter `#[cfg(feature = "service-impl")]` (verhindert WASM-Build-Pull-In im Frontend).
- WASM-Compat-Pfad verifiziert: `cargo check -p rest-types --no-default-features` grün → das `service`-Crate wird im Frontend-Build NICHT gezogen.

## Task Commits

Each task was committed atomically with `jj split` (kein git):

1. **Task 1: VacationBalance-Domain-Struct + Trait in service/src/vacation_balance.rs** — `50d94cfb` (feat)
2. **Task 2: VacationBalanceTO + From-Impls (gefeatured) in rest-types/src/lib.rs** — `12400ac8` (feat)

**Plan metadata commit:** wird vom User manuell angelegt (per `commit_docs: false` und jj-only-VCS).

## Files Created/Modified

- `service/src/vacation_balance.rs` (NEW, 110 Zeilen) — VacationBalanceService Trait + VacationBalance Domain-Struct + automock-Setup; Doc-Header + Per-Field-Doc-Kommentare verweisen auf D-04 / UI-SPEC / Plan 08-02-Berechnungsquelle.
- `service/src/lib.rs` (modified, +1 Zeile) — `pub mod vacation_balance;` zwischen `pub mod uuid_service;` und `pub mod warning;` (alphabetisch).
- `rest-types/src/lib.rs` (modified, +52 Zeilen) — `VacationBalanceTO` Struct + Bidirektionale From-Impls (`From<&service::vacation_balance::VacationBalance> for VacationBalanceTO` und `From<&VacationBalanceTO> for service::vacation_balance::VacationBalance`), eingefügt direkt vor dem Phase-4-Cutover-DTO-Block (Z. 1878), beide From-Impls hinter `#[cfg(feature = "service-impl")]` gegated.

## Verification Results

| Layer | Command | Result |
| ----- | ------- | ------ |
| service-crate compile | `nix develop --command cargo check -p service` | OK (44.03s) |
| rest-types mit service-impl | `nix develop --command cargo check -p rest-types --features service-impl` | OK (29.42s) |
| rest-types ohne service-impl (WASM-Compat) | `nix develop --command cargo check -p rest-types --no-default-features` | OK (2.17s) |
| Workspace-Sanity | `nix develop --command cargo check --workspace` | OK (15.83s) |
| Test-Regression | `nix develop --command cargo test -p service -p rest-types --features service-impl` | 8 passed; 0 failed (existing service::absence + extra_hours + billing_period tests) |

Acceptance-Criteria pro Task vollständig getroffen (alle grep-Probes, Exit-Codes 0). Layer-3 WASM-Build (`cargo build --target wasm32-unknown-unknown --manifest-path shifty-dioxus/Cargo.toml`) ist Plan-08-05-Scope (Wave-5-Build-Gate) und nicht Teil dieses Plans — das `--no-default-features`-Check verifiziert aber den WASM-Compat-Pfad indirekt: rest-types kompiliert ohne `service`-Crate-Pull-In.

## Decisions Made

- **Plan-Frontmatter-Diskrepanz erkannt, nicht umgesetzt:** PLAN.md Frontmatter listet `requirements: [FUI-A-04]`, aber FUI-A-04 ("`AbsencePeriodCreateResultTO.warnings[]` als nicht-blockierende Hinweisliste") ist ein Frontend-Render-Requirement, das erst Plan 08-05 (`WarningList`-Component-Render) tatsächlich erfüllt. Dieser Plan 08-01 ist Backend-Foundation für den D-03-Resturlaubs-Endpoint (`VacationBalanceService` + `VacationBalanceTO`). Ich markiere FUI-A-04 NICHT als complete in REQUIREMENTS.md — das wäre eine falsche Phase-Closure-Behauptung. Korrekturpfad: Plan 08-05 schließt FUI-A-04. Optional kann der User das `requirements`-Feld im PLAN.md retroaktiv leeren (ist aber jj-only-VCS und liegt außerhalb dieses Executor-Scopes).
- **carryover_days `i32` (nicht `u32`):** Service-Layer `Carryover.vacation` ist im Backend bereits typed; ich habe das CONTEXT.md-Pattern (`i32` in 08-PATTERNS.md "Apply to VacationBalanceTO") übernommen. Plan 08-02 verifiziert beim Service-Impl, ob das Backend tatsächlich Negative produziert (z. B. zu viel genommen → negative Carry).
- **Doc-Kommentare statt Schema-Description:** Per-Field-Doc-Kommentare in der Domain-Struct verweisen auf Quelle der Berechnung (z. B. "Quelle: `EmployeeWorkDetailsService::vacation_days_for_year`"). Plan 08-02-Implementer braucht keine zusätzliche Recherche, was wo herkommt.
- **Position des DTO-Blocks:** VacationBalanceTO landet vor dem Phase-4-Cutover-Block (Z. 1878) statt direkt nach AbsencePeriodCreateResultTO — chronologisch sortiert nach Phase (8 < 4-cutover-extension), bleibt aber im Absence-Domain-Cluster.

## Deviations from Plan

None - plan executed exactly as written. Beide Tasks folgten dem PLAN.md `<action>`-Block 1:1 ohne Auto-Fixes oder Rule-1/2/3-Anwendungen.

## Issues Encountered

- **jj-Split statt jj-Commit:** Working-Copy enthielt bereits die orchestrator-induzierte STATE.md-Modifikation. Lösung: `jj split <task-files> --message "..."` extrahiert genau die Task-1- bzw. Task-2-Files in einen separaten Change und lässt die STATE.md-Änderung im verbleibenden Working-Copy-Change. Saubere atomare Commits ohne Cross-Concerns.

## User Setup Required

None — keine externen Services, keine Env-Var-Änderungen, keine Migrationen.

## Next Phase Readiness

- Plan 08-02 (Service-Impl + ≥6 Unit-Tests + REST + DI) kann gegen die hier definierten Symbole bauen, ohne Trait-Form oder DTO-Form revisitieren zu müssen.
- `MockVacationBalanceService` ist via `automock` verfügbar — Plan 08-02-Tests setzen Mock-Expectations gegen genau dieses Symbol.
- `VacationBalanceTO::from(&domain)` und `(&to).into()` sind beide testbar in Plan 08-02 (Backend-Roundtrip).
- Alle Layer-1-/Layer-2-/Layer-3-Verifikationen aus PLAN.md `<verification>` grün.

## Self-Check: PASSED

Verifizierte Artefakte:
- ✓ FOUND: service/src/vacation_balance.rs (110 Zeilen, trait + struct + automock vorhanden per grep)
- ✓ FOUND: pub mod vacation_balance Eintrag in service/src/lib.rs (Z. 43)
- ✓ FOUND: pub struct VacationBalanceTO in rest-types/src/lib.rs (Z. 1888)
- ✓ FOUND: impl From<&service::vacation_balance::VacationBalance> for VacationBalanceTO (1 Match)
- ✓ FOUND: impl From<&VacationBalanceTO> for service::vacation_balance::VacationBalance (1 Match)
- ✓ FOUND: Commit 50d94cfb (Task 1) in jj log
- ✓ FOUND: Commit 12400ac8 (Task 2) in jj log
- ✓ ALL: cargo check -p service / -p rest-types --features service-impl / -p rest-types --no-default-features / --workspace alle Exit 0
- ✓ ALL: cargo test -p service -p rest-types --features service-impl: 8 passed, 0 failed

---

*Phase: 08-absence-crud-page-foundation*
*Plan: 01*
*Completed: 2026-05-08*
