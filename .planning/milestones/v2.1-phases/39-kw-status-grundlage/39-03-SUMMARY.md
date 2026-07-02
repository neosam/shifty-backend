---
phase: 39-kw-status-grundlage
plan: 03
subsystem: rest
tags: [rest, rest-types, dto, openapi, utoipa, di-wiring, basic-tier, week_status]

# Dependency graph
requires:
  - phase: 39-02-service
    provides: "WeekStatusService trait + WeekStatus domain enum (Unset/InPlanning/Planned/Locked)"
  - phase: 39-01-database
    provides: "WeekStatusDaoImpl (constructor new(pool))"
provides:
  - "WeekStatusTO + WeekStatusKindTO (rest-types, ToSchema) + From-impls <-> service::week_status::WeekStatus"
  - "GET/PUT /week-status/by-year-and-week/{year}/{week} REST endpoints (utoipa-annotated, in ApiDoc)"
  - "WeekStatusService DI-wired in main.rs basic-tier + RestStateDef assoc type/accessor"
affects: [39-04-frontend, 39-05-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Slim wire DTO (no id/version) — FE always sends year+week+status, service holds version internally"
    - "GET+PUT on same path (no id endpoint, D-39-06); write gate stays in service, not duplicated in handler"
    - "Basic-tier service DI next to week_message, before business-logic layer (D-39-12)"

key-files:
  created:
    - rest/src/week_status.rs
  modified:
    - rest-types/src/lib.rs
    - rest/src/lib.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "WeekStatusKindTO uses snake_case serde; variant named Unset never None (D-39-03), carries 4th wire variant (D-39-04)"
  - "Permission gate NOT duplicated in handler — set_week_status maps Forbidden->403 via error_handler (T-39-01)"
  - "RestStateDef WeekStatusService assoc type bounds only Context (no Self::Transaction assoc type exists in the trait)"

requirements-completed: [WST-01]

# Metrics
duration: 6min
completed: 2026-07-02
status: complete
---

# Phase 39 Plan 03: KW-Status REST + DTO + DI-Wiring Summary

**Transport layer for the KW status: slim `WeekStatusTO`/`WeekStatusKindTO` in rest-types, GET/PUT REST handlers on `/week-status/by-year-and-week/{year}/{week}` with utoipa annotations + ApiDoc registration, and basic-tier DI wiring of `WeekStatusService` in main.rs next to week_message.**

## Performance
- **Duration:** ~6 min
- **Tasks:** 3
- **Files:** 1 created, 3 modified

## Accomplishments
- `rest-types/src/lib.rs`: `WeekStatusKindTO { Unset, InPlanning, Planned, Locked }` (snake_case, ToSchema, PartialEq) + slim `WeekStatusTO { year, calendar_week, status }` (ToSchema, no id/version). Bidirectional `From`-impls to/from `service::week_status::WeekStatus` under `#[cfg(feature = "service-impl")]`, full 4-variant mapping incl. `Unset ↔ Unset`.
- `rest/src/week_status.rs`: `generate_route` with GET+PUT on the same `/by-year-and-week/{year}/{week}` path. `get_week_status_by_year_and_week` (open, returns `WeekStatusTO` with `status=Unset` when no row) and `upsert_week_status` (`Json(WeekStatusTO)` → `set_week_status`, returns resulting status). Both carry `#[instrument]` + `#[utoipa::path]` (200 + 403 on PUT). `WeekStatusApiDoc` registers both paths + both schemas.
- `rest/src/lib.rs`: `mod week_status`, `WeekStatusService` assoc type + `week_status_service()` accessor in `RestStateDef`, `(path = "/week-status", api = WeekStatusApiDoc)` in central ApiDoc, `.nest("/week-status", ...)` in router.
- `shifty_bin/src/main.rs`: `WeekStatusDao` alias, `WeekStatusServiceDependencies` (DAO + Permission + Clock + Uuid + Transaction only), `WeekStatusService` alias, `RestStateImpl` field, `RestStateDef` assoc type + accessor, and construction — all placed directly next to `week_message` in the basic-service layer (D-39-12), no domain-service dependency.

## Task Commits
1. **Task 1: WeekStatusTO + WeekStatusKindTO in rest-types** — `38d98a2` (feat)
2. **Task 2: REST handlers + router + ApiDoc registration** — `52da956` (feat)
3. **Task 3: DI-wiring in main.rs (basic-tier)** — `2c6c140` (feat)

## Decisions Made
- **RestStateDef assoc-type bound:** the plan spec'd `WeekStatusService<Context = Context, Transaction = Self::Transaction>`, but `RestStateDef` exposes no `Transaction` associated type (E0220). Reduced to `<Context = Context>` to match the established week_message pattern; the transaction type stays generic and handlers pass `None`. See Deviations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Dropped `Transaction = Self::Transaction` bound on the RestStateDef assoc type**
- **Found during:** Task 2 (`cargo build -p rest`)
- **Issue:** The plan's suggested bound `service::week_status::WeekStatusService<Context = Context, Transaction = Self::Transaction>` failed to compile — `RestStateDef` has no `Transaction` associated type (`error[E0220]: associated type Transaction not found for Self`). No other service in the trait pins a Transaction.
- **Fix:** Bounded the assoc type on `<Context = Context>` only, mirroring `WeekMessageService` and every other service in `RestStateDef`. Handlers pass `None` for tx, so the concrete transaction type is resolved at the impl site in main.rs.
- **Files modified:** rest/src/lib.rs
- **Commit:** 52da956

## Threat Mitigations Applied
- **T-39-01 (Elevation of Privilege, PUT):** handler delegates to `set_week_status`; the shiftplanner gate lives in the service and maps `Forbidden` → HTTP 403 via `error_handler`. No separate ungated write path.
- **T-39-02 (Input validation):** `WeekStatusKindTO` is a closed serde enum → unknown wire values rejected as deserialization errors.
- **T-39-03 (Info disclosure, GET):** accepted by design — status is not sensitive, all roles read.

## Scope Guard
No lock ENFORCEMENT / HTTP 423 built — that is Phase 40. This plan only exposes the status CRUD over HTTP.

## Gate Results
- `cargo build -p rest-types --features service-impl` — pass
- `cargo build -p rest` — pass
- `cargo build` (binary + full wiring) — pass
- `cargo clippy --workspace -- -D warnings` — pass
- `cargo test --workspace` — pass (no regressions; service_impl 541, rest 64, others green)
- No new `query!` added (DAO from Wave 1 reused) → no `cargo sqlx prepare` needed.

## ApiDoc + DI Verification
- `WeekStatusApiDoc` registered in the central `ApiDoc` paths list (`/week-status`); router nests `/week-status`; both handlers carry `#[utoipa::path]`.
- `week_status_service` wired in the basic-service layer of `main.rs` directly beside `week_message_service`, before the business-logic layer; depends on no domain service (D-39-12).
- `RestStateImpl` field + accessor + `RestStateDef` assoc type consistent.

## Next Phase Readiness
- Wave 4 (FE foundation) can import `WeekStatusTO`/`WeekStatusKindTO` from rest-types and call GET/PUT `/week-status/by-year-and-week/{year}/{week}`.
- No blockers.

## Self-Check: PASSED
- `rest/src/week_status.rs` present on disk.
- Commits 38d98a2, 52da956, 2c6c140 exist in git history.

---
*Phase: 39-kw-status-grundlage*
*Completed: 2026-07-02*
