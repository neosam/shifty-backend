---
phase: 28-urlaubsanspruch-korrektur-offset
plan: 02
subsystem: backend-read-path
tags: [service, business-logic, rest, axum, utoipa, dto, di, api-hiding, mockall]
requires:
  - "28-01: Basic VacationEntitlementOffsetService + DAO + migration"
provides:
  - "service::vacation_balance::VacationBalance += offset_days, computed_entitled_days (HR-only breakdown)"
  - "rest_types::VacationBalanceTO += offset_days, computed_entitled_days (Option, WASM-safe)"
  - "rest_types::VacationEntitlementOffsetTO (plain DTO, ToSchema)"
  - "rest/src/vacation_entitlement_offset.rs (HR-gated POST upsert + DELETE + VacationEntitlementOffsetApiDoc)"
  - "RestStateDef::VacationEntitlementOffsetService assoc type + accessor"
  - "VacationBalanceService now consumes the Basic offset service (BL->Basic, no cycle)"
affects:
  - "Frontend (Phase 28 later plans): VacationBalanceTO carries effective entitled_days + HR-only offset breakdown"
tech-stack:
  added: []
  patterns:
    - "Offset added AFTER round() — integer-day correction, not into the f32 sum (D-28-02)"
    - "API-level field hiding via is_hr = hr.is_ok() captured before hr.or(sp)? (D-28-03)"
    - "Authentication::Full internal offset read (effective value for both roles), exposure gated separately"
    - "rest-types From impls feature-gated behind service-impl; new fields Option => WASM round-trips (Pitfall 3)"
    - "DI: Basic offset service constructed between carryover and vacation_balance (BL consumes Basic)"
    - "mockall FIFO: offset tests REPLACE the default Ok(None) mock instead of adding a 2nd expectation"
key-files:
  created:
    - rest/src/vacation_entitlement_offset.rs
  modified:
    - service/src/vacation_balance.rs
    - service_impl/src/vacation_balance.rs
    - rest-types/src/lib.rs
    - rest/src/lib.rs
    - shifty_bin/src/main.rs
    - service_impl/src/test/vacation_balance.rs
decisions:
  - "D-28-02: entitled_effective = round(base) + offset_days; offset added after .round(), flows into remaining_days"
  - "D-28-03: entitled_days ALWAYS effective; offset_days/computed_entitled_days Some only when is_hr, else None"
  - "D-28-06b: dedicated HR-gated REST CRUD; HR_PRIVILEGE enforced inside the Basic offset service"
  - "Internal offset read uses Authentication::Full by design (per-plan NOTE) — kept, not turned into a permission error"
metrics:
  duration: ~40m
  completed: 2026-06-29
status: complete
---

# Phase 28 Plan 02: Wire Vacation-Entitlement-Offset into the Read Path Summary

The signed per-(person,year) offset from Plan 28-01 now flows into the vacation-balance read path: `VacationBalanceService` reads the Basic offset service, adds the offset after `.round()`, exposes the breakdown only to HR callers (server-side API-level hiding), and a dedicated HR-gated REST CRUD endpoint lets HR set/clear the offset — all green under build/wasm-compat/test/clippy.

## What was built

**Task 1 — Offset into vacation_balance + API-hiding + DI (commit rtmwslss 7a8b6e3c, `feat`)**
- `service/src/vacation_balance.rs`: `VacationBalance` gains `offset_days: Option<i32>` and `computed_entitled_days: Option<f32>` (doc-commented HR-only breakdown; effective value always in `entitled_days`).
- `service_impl/src/vacation_balance.rs`: added `VacationEntitlementOffsetService` dep to the `gen_service_impl!` block (BL consumes Basic, no cycle). `compute_balance` gained an `is_hr: bool` param; the pre-offset local was renamed to `computed_entitled_days`, the offset is read once via `Authentication::Full` (by design — effective value correct for both roles), `entitled_effective = computed_entitled_days + offset_days as f32` (added AFTER round, D-28-02), and `remaining_days` uses the effective value. Return struct: `entitled_days = entitled_effective` (always), breakdown fields `Some(..)` only when `is_hr`. `get` captures `is_hr = hr.is_ok()` BEFORE `hr.or(sp)?`; `get_team` passes `is_hr = true`.
- `rest-types/src/lib.rs`: `VacationBalanceTO` gained the two `Option` fields (`#[serde(default)]`) and both feature-gated `From` impls map them — WASM round-trips (Pitfall 3).
- `shifty_bin/src/main.rs`: `VacationEntitlementOffsetDao` type alias + DAO construction (~:715), `VacationEntitlementOffsetServiceDependencies` + impl + type alias, Basic offset service constructed between `carryover_service` (~:864) and `vacation_balance_service` (~:873), wired into the `VacationBalanceServiceImpl` initializer and `VacationBalanceServiceDependencies`, plus `RestStateImpl` field + `RestStateDef` assoc type + accessor.

**Task 2 — HR-gated REST CRUD endpoint (commit mtrvrltu 37eea3cb, `feat`)**
- `rest-types/src/lib.rs`: `VacationEntitlementOffsetTO { sales_person_id, year, offset_days }` (plain primitives, `Serialize/Deserialize/ToSchema/Clone/Debug/PartialEq`, no `service-impl` gate).
- `rest/src/vacation_entitlement_offset.rs`: `generate_route()` with `POST /` (upsert → 200) and `DELETE /{sales_person_id}/{year}` (→ 204), each with `#[utoipa::path]` + `#[instrument(skip(rest_state))]` + `error_handler`; HR enforcement happens inside the Basic service. `VacationEntitlementOffsetApiDoc` lists both paths + `components(schemas(VacationEntitlementOffsetTO))`.
- `rest/src/lib.rs`: `mod vacation_entitlement_offset;`, ApiDoc registered in the nested-OpenApi list, route mounted at `/vacation-entitlement-offset`.

**Task 3 — Backend tests (commit umxzmuvm 276d4f45, `test`)**
- `service_impl/src/test/vacation_balance.rs`: wired `MockVacationEntitlementOffsetService` into the test deps (default `Ok(None)`; offset tests REPLACE the mock to avoid mockall FIFO matching). Added `offset_calc` (round(17)+1=18 / −2=15, flows into remaining), `offset_delta` (base 17→20 → effective 21, delta survives), `offset_api_hiding` (HR → `Some` breakdown, self-only → `None`, `entitled_days` always effective).

## must_haves coverage

- D-28-02: offset added after `.round()`, `remaining_days` uses `entitled_effective` — done (`offset_calc`).
- D-28-03: breakdown `Some` only when `is_hr`, `entitled_days` always effective — done (`offset_api_hiding`); delta survives recomputation — done (`offset_delta`).
- D-28-06b: HR-gated REST CRUD (`#[utoipa::path]` + `ToSchema` + ApiDoc registered + route mounted), writes via Basic service's HR gate — done.
- D-28-09: offset read/write + balance breakdown year-scoped (one offset per person+year) — done.

## Deviations from Plan

None — plan executed as written. The internal offset read intentionally uses `Authentication::Full` (per the plan's inline NOTE) and was kept as-is.

## Gate results

- `cargo build --workspace` (SQLX_OFFLINE=true): success.
- `cargo build --target wasm32-unknown-unknown -p rest-types`: PASSES from the frontend (`shifty-dioxus`) workspace where `uuid`'s `js` feature is resolved (rest-types compiles wasm-clean, no `service` pulled — Pitfall 3 satisfied). NOTE: the same command run standalone from the backend workspace root fails with a pre-existing `uuid`-needs-`js`/`getrandom` randomness error — an environmental gap independent of this plan's `Option`-field additions (the real Pitfall-3 wasm gate runs in the frontend workspace per 28-RESEARCH §"Phase gate", line 410).
- `cargo test -p service_impl vacation_balance`: 18 passed, 0 failed (15 existing + 3 new: offset_calc, offset_delta, offset_api_hiding).
- `cargo test --workspace`: all green — service_impl 504 passed, shifty_bin 61, rest 5 + openapi_surface 3 (OpenAPI registration intact), rest_types 5, service 13, dao 11, dao_impl_sqlite 3, shifty_utils 11; 0 failed across the workspace.
- `cargo clippy --workspace -- -D warnings`: clean, exit 0.

## API-hiding behavior + DI order (confirmation)

- API-hiding: `entitled_days` is the effective value (`round(base)+offset`) for BOTH roles; `offset_days` + `computed_entitled_days` are `Some` ONLY when `is_hr` (captured as `hr.is_ok()` before `hr.or(sp)?`), else `None`. Verified by `offset_api_hiding`.
- DI order: Basic `vacation_entitlement_offset_service` constructed AFTER `carryover_service` and BEFORE `vacation_balance_service`; Business-Logic consumes Basic — no forward-reference, no cycle.

## Known Stubs

None. Read path + REST CRUD fully wired and tested.

## Self-Check: PASSED

- Files exist: `rest/src/vacation_entitlement_offset.rs` created; `service/src/vacation_balance.rs`, `service_impl/src/vacation_balance.rs`, `rest-types/src/lib.rs`, `rest/src/lib.rs`, `shifty_bin/src/main.rs`, `service_impl/src/test/vacation_balance.rs` modified.
- Commits exist in jj log: rtmwslss 7a8b6e3c, mtrvrltu 37eea3cb, umxzmuvm 276d4f45.
