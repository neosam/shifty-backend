---
phase: 08-absence-crud-page-foundation
plan: 02
subsystem: api
tags: [shifty-backend, vacation-balance, service-impl, rest, di-wiring, bl-tier, utoipa]

# Dependency graph
requires:
  - phase: 08-absence-crud-page-foundation
    plan: 01
    provides: VacationBalanceService Trait + VacationBalance Domain-Struct + VacationBalanceTO DTO
  - phase: 01-absence-domain-foundation
    provides: AbsencePeriod-Domain + AbsenceService Trait
  - phase: 04-cutover-extension
    provides: CarryoverService.get_carryover Trait-Method
provides:
  - service_impl::vacation_balance::VacationBalanceServiceImpl (BL-Tier, gen_service_impl!)
  - service_impl::vacation_balance::VacationBalanceServiceDeps (Type-Family)
  - REST GET /vacation-balance/{sales_person_id}/{year} (HR ∨ self)
  - REST GET /vacation-balance/team/{year} (HR-only)
  - VacationBalanceApiDoc (utoipa)
  - DI-Wiring in shifty_bin/src/main.rs (Deps + Konstruktor + RestStateImpl-Field + Getter)
affects:
  - 08-03 OpenAPI snapshot (insta-Refresh, weil ApiDoc neue Endpoints sieht)
  - 08-04 Frontend api/state (kann den Resturlaub konsumieren)
  - 08-05 Frontend page-components (VacationEntitlementCard + VacationPerPersonList)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "BL-Tier-Service-Impl via gen_service_impl! mit Cross-Service-Deps (analog AbsenceServiceImpl)"
    - "Permission HR ∨ self via tokio::join!(check_permission(HR), verify_user_is_sales_person).or() — wiederverwendet aus absence.rs:110-119"
    - "Helper-Methode compute_balance ohne Permission-Check für interne Aggregation in get_team (Authentication::Full für innere Service-Calls)"
    - "Tag-Berechnung beschneidet AbsencePeriod-Range auf Kalenderjahr-Grenzen via days_in_year_for_period-Helper"
    - "Aktive Vacation-Periodeen splitten auf clock.date_now() — Vergangenheits-Anteil zu used, Zukunfts-Anteil zu planned"

key-files:
  created:
    - service_impl/src/vacation_balance.rs
    - service_impl/src/test/vacation_balance.rs
    - rest/src/vacation_balance.rs
    - .planning/phases/08-absence-crud-page-foundation/08-02-SUMMARY.md
  modified:
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - rest/src/lib.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "Tag-Berechnung in dieser Iteration ohne Special-Day-/Holiday-Subtraktion (08-RESEARCH.md A5-Note): used/planned-Days zählen Kalendertage minus Year-Schnitt, ohne Wochenenden oder Feiertage zu filtern. Frontend erhält das Aggregat 1:1; Refinement (Tag-Äquivalent via EmployeeWorkDetails.has_day_of_week) ist Out-of-Scope für Plan 02."
  - "Aktive Vacation-Periodeen (today ∈ [from, to]) splitten auf today als Stichtag — Vergangenheits-Tage in used, Zukunfts-Tage in planned. So gibt es keine 'Verschwinden'-Diskontinuität, wenn eine Periode heute beginnt."
  - "compute_balance als private Helper-Methode für Code-Sharing zwischen get und get_team. Helper macht KEINE Permission-Checks und nutzt Authentication::Full für innere Service-Calls — der Outer-Permission-Gate des Aufrufers reicht."
  - "Carryover-Year-Semantik: get_carryover(sp_id, year) wird mit dem angefragten year direkt aufgerufen (kein year-1-Shift). Konvention im Repo (vgl. service_impl/src/carryover.rs)."
  - "Konstruktor-Position in main.rs: direkt nach carryover_service (Z. ~843), vor reporting_service. Diese Position erfüllt Pitfall 3 (DI-Reihenfolge: VacationBalance benötigt absence_service Z. 798, working_hours_service Z. 788, carryover_service Z. 843)."

patterns-established:
  - "Erste BL-Tier-Service-Impl in Phase 8, die nur Domain-Services (kein DAO) konsumiert — Vorbild für künftige Read-Aggregate."
  - "REST-Routen-Reihenfolge: konkrete Paths VOR Wildcard-Paths. /team/{year} muss VOR /{sales_person_id}/{year} stehen, sonst routet Axum 'team' als Uuid."

requirements-completed: []
# PLAN-FRONTMATTER-DISKREPANZ (analog Plan 01): Frontmatter listet
# requirements: [FUI-A-04], aber FUI-A-04 (warnings[] als nicht-blockierende
# Hinweisliste) ist ein Frontend-Render-Requirement, das erst Plan 08-05
# (WarningList-Component) erfüllt. Plan 02 baut den BL-Tier-Service +
# REST + DI für D-03/D-04 (Backend-Foundation für Resturlaub). Keine
# Requirement-Closure in dieser Wave.

# Metrics
duration: ~25min
completed: 2026-05-08
---

# Phase 08 Plan 02: Absence-CRUD-Page Foundation — VacationBalance Service-Impl + REST + DI Summary

**Wave-2-Backend-Implementation: VacationBalanceServiceImpl als BL-Tier-Service mit gen_service_impl!, REST-Endpoints `/vacation-balance/{sp}/{year}` (HR ∨ self) und `/vacation-balance/team/{year}` (HR-only), und vollständiges DI-Wiring in shifty_bin/src/main.rs. Damit ist die Backend-Foundation für die D-03-Resturlaubs-Anzeige in Wave 3 fertig.**

## Performance

- **Duration:** ~25 min (mit cargo build/test Wartezeiten)
- **Started:** 2026-05-08 nach Plan 01 Cleanup
- **Completed:** 2026-05-08T08:52Z
- **Tasks:** 3 / 3
- **Files modified:** 8 (3 NEW + 4 modified + 1 SUMMARY)

## Accomplishments

- **VacationBalanceServiceImpl** als BL-Tier-Service in `service_impl/src/vacation_balance.rs` mit `gen_service_impl!`-Macro (D-04). 7 Cross-Service-Deps: AbsenceService, EmployeeWorkDetailsService, CarryoverService, SalesPersonService, PermissionService, ClockService, TransactionDao.
- **`get(sales_person_id, year)`** mit Permission HR ∨ self via `tokio::join!(...).or()` und korrekter Tag-Aggregation: `entitled_days = Σ vacation_days_for_year` über alle aktiven Verträge, `carryover_days = Carryover.vacation`, `used_days/planned_days` aus AbsencePeriod-Filter (category=Vacation, deleted=None) gesplittet auf `clock.date_now()`, `remaining_days = entitled + carryover - (used + planned)`.
- **`get_team(year)`** HR-only; iteriert über `sales_person_service.get_all_paid()` und ruft pro Person die Helper-Methode `compute_balance` auf.
- **7 Unit-Tests** in `service_impl/src/test/vacation_balance.rs`, alle grün:
  1. `get_returns_entitlement_minus_used_minus_planned` — Self-Path Happy mit 25 entitled + 5 carryover − 5 used − 10 planned = 15 remaining
  2. `get_with_hr_succeeds` — HR-Path, verify_user_is_sales_person fails aber `or()` filtert
  3. `get_other_sales_person_without_hr_is_forbidden` — T-8-AUTH-01 + T-8-IDOR-01
  4. `get_team_without_hr_is_forbidden` — T-8-AUTH-02
  5. `get_team_aggregates_per_paid_sales_person` — HR-Path mit 2 Personen, je korrekt aggregiert
  6. `get_with_no_active_contract_returns_zero_entitlement` — Edge-Case
  7. `get_year_without_carryover_returns_zero_carryover` — None-Path
- **REST-Endpoints** `/vacation-balance/{sales_person_id}/{year}` (200/403/404) und `/vacation-balance/team/{year}` (200/403) in `rest/src/vacation_balance.rs` mit `#[utoipa::path(...)]`-Annotation auf jedem Handler. Routen-Reihenfolge `/team/{year}` VOR `/{sales_person_id}/{year}`.
- **`VacationBalanceApiDoc`** registriert `VacationBalanceTO` als Schema-Component und beide Handler im OpenAPI-Surface.
- **`rest/src/lib.rs`-Wiring**: `mod vacation_balance;`, `RestStateDef::VacationBalanceService`-Type-Assoc + Getter, ApiDoc nest-Eintrag (`/vacation-balance` → `VacationBalanceApiDoc`), Router nest (`.nest("/vacation-balance", vacation_balance::generate_route())`).
- **`shifty_bin/src/main.rs`-DI**: `VacationBalanceServiceDependencies` struct + impl, `type VacationBalanceService = ...`, Konstruktor direkt nach `carryover_service` (Z. ~843, Pitfall 3 erfüllt), `RestStateImpl`-Field + Getter im `impl RestStateDef for RestStateImpl`-Block, Self-Init am Ende von `new()`.

## Task Commits

Atomar mit `jj commit -m "..."`:

1. **Task 1: VacationBalanceServiceImpl (BL-tier) + 7 unit tests** — `590a97fb` (feat)
2. **Task 2: /vacation-balance REST endpoints with utoipa** — `d31ecd5a` (feat)
3. **Task 3: wire VacationBalanceService into DI** — `8ba7a99a` (feat)

**Plan metadata commit:** wird vom User manuell angelegt (`commit_docs: false`, jj-only-VCS).

## Files Created/Modified

- `service_impl/src/vacation_balance.rs` (NEW, 218 Zeilen) — BL-Tier-Service-Impl: `gen_service_impl!`-Block, `days_in_year_for_period`-Helper, `impl VacationBalanceService for ...` mit `get`/`get_team`, plus private `compute_balance`-Helper.
- `service_impl/src/test/vacation_balance.rs` (NEW, 386 Zeilen) — 7 mockall-basierte Tests + `VacationBalanceDependencies`-Struct + `build_dependencies()`-Helper + Test-Daten-Constructor (`vacation_period`, `full_year_contract`, `paid_sales_person`).
- `service_impl/src/lib.rs` (modified, +1 Zeile) — `pub mod vacation_balance;` zwischen `uuid_service` und `week_message`.
- `service_impl/src/test/mod.rs` (modified, +2 Zeilen) — `#[cfg(test)] pub mod vacation_balance;`.
- `rest/src/vacation_balance.rs` (NEW, 121 Zeilen) — Router (`/team/{year}` vor `/{sales_person_id}/{year}`), zwei Handler mit `#[utoipa::path]` und `#[instrument]`, ApiDoc-Block.
- `rest/src/lib.rs` (modified, +12 Zeilen) — `mod vacation_balance;`, RestStateDef-Trait-Erweiterung (Type-Assoc + Getter), ApiDoc nest-Eintrag, Router-nest.
- `shifty_bin/src/main.rs` (modified, +30 Zeilen) — `VacationBalanceServiceDependencies` Struct + impl, Type-Alias, `RestStateImpl`-Field, RestStateDef-impl-Erweiterung (Type-Assoc + Getter), Konstruktor + Self-Init.

## Verification Results

| Layer | Command | Result |
| ----- | ------- | ------ |
| service_impl compile | `nix develop --command cargo check -p service_impl` | OK (33.88s) |
| rest compile | `nix develop --command cargo check -p rest` | OK (32.68s) |
| shifty_bin build | `nix develop --command cargo build --bin shifty_bin` | OK (43.68s) |
| service_impl tests | `nix develop --command cargo test -p service_impl vacation_balance` | 7 passed; 0 failed |
| Workspace tests | `nix develop --command cargo test --workspace` | 388 service_impl + 56 service_impl integ + 11 service + 10 dao + 8 dao_impl_sqlite Doc-tests, 0 failed |

Acceptance-Criteria pro Task vollständig getroffen:
- Task 1: gen_service_impl=1, VacationBalanceServiceImpl=3, Cross-Service-Deps=11 (≥5), use_transaction=2 (≥1), join!=2 (≥1), HR_PRIVILEGE=3 (≥2), pub mod-Einträge=1+1, #[tokio::test]=7 (≥6).
- Task 2: utoipa::path=3 (≥2), tags="VacationBalance"=2, Handler=2, ApiDoc=1, generate_route=1, vacation_balance_service-Aufrufe=2 (≥2), mod=1, type=1, fn=1, ApiDoc-Refs=1, generate_route-Refs=1, .nest=1.
- Task 3: VacationBalanceServiceDependencies=4 (≥3), VacationBalanceServiceDeps=1, type VacationBalanceService=2, vacation_balance_service=5 (≥4), fn vacation_balance_service=1.

## Decisions Made

- **Special-Day-Subtraktion verschoben** (siehe key-decisions). Die Tag-Anzahl pro Vacation-Periode ist `(to - from).whole_days() + 1`, beschnitten auf das Jahr. Wochenenden, Feiertage, Vertragsstunden-Anteile werden in dieser Iteration NICHT berücksichtigt. Das ist konsistent mit 08-RESEARCH.md "A5-Note" und macht das Aggregat zur reinen Kalendertage-Sicht; für die UI-Spec der `VacationEntitlementCard` reicht das.
- **Aktive Periodeen splitten auf today**: Wenn `today ∈ [from, to]`, wird die Periode in einen used- (from..today) und planned-Anteil (today+1..to) gesplittet, damit es keine Diskontinuität gibt, wenn eine Periode heute beginnt oder gestern endete. Das gleiche Aggregat ist heute und morgen aussagekräftig.
- **compute_balance als private Helper**: Code-Sharing zwischen `get` und `get_team` ohne erneute Permission-Checks. Der Outer-Aufrufer (`get` HR ∨ self, `get_team` HR-only) verifiziert Permission einmalig; der Helper benutzt `Authentication::Full` für innere Service-Calls (analog `compute_forward_warnings` in `absence.rs`).
- **Carryover-Year direkt durchgereicht**: Plan 02 nutzt `get_carryover(sp_id, year)` mit dem angefragten Jahr ohne Shift. Wenn die Repo-Konvention an anderer Stelle `year-1` ist (Snapshot-Vorjahr), würde Plan 03/04 das im Frontend bemerken — das ist hier nicht der Fall (vgl. Carryover-Tests im Repo).
- **Konstruktor-Position direkt nach `carryover_service`**: erfüllt Pitfall 3 (alle Cross-Deps im Scope) und sortiert sich logisch in den Backend-Service-Stack ein, ohne Reporting/BookingInformation/etc. zu beeinflussen.

## Deviations from Plan

None — Plan exakt wie geschrieben ausgeführt. Keine Auto-Fixes oder Rule-1/2/3-Anwendungen nötig; alle Verifikations-Layer beim ersten Versuch grün (außer einem fehlenden Test-Import des `VacationBalanceService`-Traits, der sofort hinzugefügt wurde — kein Plan-Pfad-Issue).

## Issues Encountered

- **Test-Compile: fehlender Trait-Import**: Initial fehlte `use service::vacation_balance::VacationBalanceService;` in den Tests, sodass `svc.get(...)` als Method-not-found gemeldet wurde. Der Compiler hat den exakten Help-Hint gegeben, Fix war eine Zeile. Kein Plan-Issue.
- **Bin-Target-Name**: `cargo build --bin shifty` schlägt fehl; das Target heißt `shifty_bin`. Habe entsprechend gewechselt — Plan-Acceptance-Probe (Z. 705) sagt zwar "shifty", aber Cargo.toml ist authoritativ. Kein Code-Issue.

## User Setup Required

None — keine externen Services, keine Env-Var-Änderungen, keine Migrationen.

## Next Phase Readiness

- **Plan 08-03 (OpenAPI insta-Snapshot Refresh)** kann den Snapshot mit `cargo insta accept` aktualisieren — die ApiDoc enthält jetzt `VacationBalanceApiDoc` mit zwei neuen Endpoints und einem neuen Schema (`VacationBalanceTO`).
- **Plan 08-04 (Frontend api/state)** kann gegen `/vacation-balance/{sp}/{year}` und `/vacation-balance/team/{year}` API-Funktionen schreiben; `VacationBalanceTO` ist via `rest-types` (default-features=false) verfügbar.
- **Plan 08-05 (Frontend page-components)**: `VacationEntitlementCard` und `VacationPerPersonList` haben jetzt einen autoritativen Backend-Endpoint.
- Manueller Smoke (außerhalb dieses Plans, optional in Plan 06): `cargo run` startet, `curl http://localhost:3000/vacation-balance/00000000-.../2026` antwortet 401/403 — Route ist aktiv.

## Self-Check: PASSED

Verifizierte Artefakte:
- ✓ FOUND: service_impl/src/vacation_balance.rs (218 Zeilen)
- ✓ FOUND: service_impl/src/test/vacation_balance.rs (386 Zeilen, 7 #[tokio::test])
- ✓ FOUND: rest/src/vacation_balance.rs (121 Zeilen)
- ✓ FOUND: pub mod vacation_balance in service_impl/src/lib.rs
- ✓ FOUND: pub mod vacation_balance in service_impl/src/test/mod.rs
- ✓ FOUND: mod vacation_balance + RestStateDef + ApiDoc + .nest in rest/src/lib.rs
- ✓ FOUND: VacationBalanceServiceDependencies + Konstruktor + RestStateImpl-Field + Getter in shifty_bin/src/main.rs
- ✓ FOUND: jj commit 590a97fb (Task 1)
- ✓ FOUND: jj commit d31ecd5a (Task 2)
- ✓ FOUND: jj commit 8ba7a99a (Task 3)
- ✓ ALL: cargo check -p service_impl / cargo check -p rest / cargo build --bin shifty_bin / cargo test --workspace alle Exit 0
- ✓ ALL: 7 vacation_balance-Tests grün; 388 service_impl-Tests gesamt grün; keine Regression

---

*Phase: 08-absence-crud-page-foundation*
*Plan: 02*
*Completed: 2026-05-08*
