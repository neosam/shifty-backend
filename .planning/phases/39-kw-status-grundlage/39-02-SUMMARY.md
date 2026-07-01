---
phase: 39-kw-status-grundlage
plan: 02
subsystem: service
tags: [service, basic-tier, tdd, week_status, iso-week, permission-gate, mockall]

# Dependency graph
requires:
  - phase: 39-01-database
    provides: "WeekStatusDao trait + MockWeekStatusDao + WeekStatusKind{InPlanning,Planned,Locked} + WeekStatusEntity"
provides:
  - "WeekStatus domain enum (Unset/InPlanning/Planned/Locked) + From<WeekStatusKind>"
  - "WeekStatusService trait (get_week_status/set_week_status) + MockWeekStatusService"
  - "WeekStatusServiceImpl (Basic-tier) with permission gate, upsert/soft-delete, free transitions"
affects: [39-03-rest, 39-04-frontend, 39-05-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Basic-tier service via gen_service_impl! (DAO + Permission + Clock + Uuid + Transaction only, no domain-service dep)"
    - "Domain enum with a service-only Unset variant mapped to row-absence (never persisted)"
    - "Pure iso_week proof tests via time::to_iso_week_date() as ground-truth (D-39-11)"

key-files:
  created:
    - service/src/week_status.rs
    - service_impl/src/week_status.rs
    - service_impl/src/test/week_status.rs
  modified:
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs

key-decisions:
  - "Unset carried as 4th domain variant, mapped to row absence; never reaches DAO (D-39-04/03)"
  - "check_permission(SHIFTPLANNER_PRIVILEGE) is the first statement in set_week_status, before use_transaction (D-39-01, T-39-01)"
  - "find + create/update/delete share one transaction (no TOCTOU, T-39-04)"
  - "No transition validation: all transitions free incl. Locked->InPlanning / Locked->Unset (D-39-02)"

requirements-completed: [WST-01]

# Metrics
duration: 7min
completed: 2026-07-01
status: complete
---

# Phase 39 Plan 02: KW-Status Business-Logik (Basic-Tier Service) Summary

**Basic-tier `WeekStatusService` (TDD) exposing the `WeekStatus` domain enum with a shiftplanner-only permission gate, ISO-week-correct upsert / soft-delete (Unset == row absence), and free status transitions.**

## Performance
- **Duration:** ~7 min
- **Tasks:** 2 (RED then GREEN)
- **Files:** 3 created, 3 modified

## Accomplishments
- `service/src/week_status.rs`: `WeekStatus { Unset, InPlanning, Planned, Locked }` domain enum + `From<dao::week_status::WeekStatusKind>` (Unset never originates from the DAO). `WeekStatusService` trait with `#[automock]` + `#[async_trait]`, assoc `Context`/`Transaction`, and `get_week_status` (all roles) / `set_week_status` (shiftplanner only).
- `service_impl/src/week_status.rs`: `WeekStatusServiceImpl` via `gen_service_impl!` — Basic-tier deps only (`WeekStatusDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`; no domain service, D-39-12). `set_week_status` gates permission first, then find+write in one transaction; `Unset` soft-deletes the active row (or no-op when absent), non-Unset upserts via create/update with fresh uuid/version and clock-`created`. `get_week_status` maps row→`WeekStatus`, absence→`Unset`, no gate.
- `service_impl/src/test/week_status.rs`: full mockall suite — the 5 mandatory KW-53 iso_week cases (pure `time` proof), permission-denied (no DAO write), soft-delete/no-op, create-when-absent, update-when-present, free transitions (incl. Locked→InPlanning and Locked→Unset), and get-mapping.

## Task Commits
1. **Task 1 (RED): trait/enum + failing test suite** — `3950c31` (test)
2. **Task 2 (GREEN): service logic + permission gate** — `8da6e1a` (feat)

## The 5 mandatory KW-53 / boundary cases (D-39-11) — all green
- 2021-01-01 → (2020, 53)
- 2020-12-28 → (2020, 53)
- 2025-12-29 → (2026, 1)
- 2025-12-28 → (2025, 52)
- 2026-03-15 → (2026, 11)

## Decisions Made
None beyond the plan — followed D-39-01/02/03/04/11/12 and threat mitigations T-39-01/03/04 as specified.

## Deviations from Plan
None - plan executed exactly as written.

## TDD Gate Compliance
Both gates present as separate commits: `test(39-02)` RED (`3950c31`, 8 service tests fail on `todo!()`) then `feat(39-02)` GREEN (`8da6e1a`, all 13 tests pass). The 5 `iso_week` tests pass in BOTH phases by design — they are a pure `time::to_iso_week_date()` proof (not service-under-test), pinning the ISO ground truth per the plan; this is intentional, not an unexpected RED-phase pass.

## Scope Guard
No lock ENFORCEMENT built (no `assert_week_not_locked` / HTTP 423) — that is Phase 40. This plan only builds the status CRUD service. `set_week_status` performs no transition validation (D-39-02).

## Gate Results
- `cargo build --workspace` — pass
- `cargo clippy --workspace -- -D warnings` — pass
- `cargo test -p service_impl week_status` — pass (13 tests: 5 iso_week + 8 service)
- `cargo test --workspace` — pass (no regressions; 541-test service_impl suite green)
- No new `query!` added → no `cargo sqlx prepare` needed.

## Next Phase Readiness
- Wave 3 (39-03 REST/DI) can consume `WeekStatusService` trait + `WeekStatus` enum for rest-types `From` impls and handlers.
- No blockers.

## Self-Check: PASSED
- All 3 created source files present on disk.
- Both task commits (3950c31, 8da6e1a) exist in git history.

---
*Phase: 39-kw-status-grundlage*
*Completed: 2026-07-01*
