---
phase: 36-special-days-bugfixes
plan: "01"
subsystem: backend
tags: [bugfix, special-days, service, dao, tdd, sdf-01]
status: complete

dependency_graph:
  requires: []
  provides: [special-day-same-date-replace]
  affects: [service_impl/special_days.rs, dao_impl_sqlite/special_day.rs]

tech_stack:
  added: []
  patterns:
    - atomic same-date UPDATE replacing duplicate INSERT guard
    - TDD RED/GREEN/REFACTOR cycle (service-level mock tests)

key_files:
  created: []
  modified:
    - service_impl/src/test/special_days.rs
    - service_impl/src/special_days.rs
    - dao_impl_sqlite/src/special_day.rs
    - .sqlx/query-6cc953f85b1cdd138b7f134e56c79423b71e5f27abede1ada2b4ee87f698af3b.json

decisions:
  - "D-01: atomic in-place UPDATE (single SQL statement) rather than delete-then-create; rollback-safe"
  - "D-02: POST /special-days contract unchanged; no new PUT endpoint; shiftplan dropdown handler untouched"
  - "D-04: central service fix covers both entry surfaces (Schichtplan dropdown + Settings Card-3)"
  - "D-09: two directional switch tests (Holiday→ShortDay, ShortDay→Holiday) plus converted duplicate test"

metrics:
  duration: "~25 min"
  completed: "2026-07-01"
  tasks_completed: 3
  tasks_total: 3

requirements: [SDF-01]
---

# Phase 36 Plan 01: SDF-01 Special-Day Same-Date Atomic Replacement Summary

**One-liner:** Replaced the `ValidationError(Duplicate)` guard in the special-day `create` service path with an atomic in-place UPDATE so switching a day's type (Holiday↔ShortDay) succeeds without error on both UI surfaces.

## What Was Built

### Task 1 (RED) — Failing tests proving the bug

Added three tests to `service_impl/src/test/special_days.rs`:

- **`test_create_replaces_same_date_entry`** (converted from `test_create_rejects_duplicate`): verifies that a second POST for the same (year, calendar_week, day_of_week) calls `dao.update` once and returns `Ok`, not `ValidationError(Duplicate)`.
- **`test_create_switches_holiday_to_shortday`**: existing Holiday at (2026, W1, Monday) → create ShortDay with `time_of_day`; asserts `dao.update` called with the existing id and `day_type=ShortDay`; `dao.create` called 0 times.
- **`test_create_switches_shortday_to_holiday`**: reverse direction; asserts `dao.update` called with existing id, `day_type=Holiday`, `time_of_day=None`; `dao.create` called 0 times.

All three were RED (`ValidationError([Duplicate])`) against the unmodified service.

### Task 2 (GREEN) — DAO and service implementation

**`dao_impl_sqlite/src/special_day.rs` — `update` extended:**

The existing UPDATE statement only set `deleted`, `update_version`, `update_process`. It now also sets `day_type` and `time_of_day` (reusing the identical Holiday/ShortDay string serialization and `[hour]:[minute]:[second]` time formatting from `create`). The statement remains a single atomic SQL UPDATE — no intermediate state, rollback-safe (D-01).

```sql
UPDATE special_day
SET deleted = ?, update_version = ?, update_process = ?, day_type = ?, time_of_day = ?
WHERE id = ?
```

The `delete` soft-delete path is unaffected: it passes the entity's existing `day_type`/`time_of_day` back unchanged.

**`service_impl/src/special_days.rs` — `create` replacement branch:**

The old duplicate guard (returned `ValidationFailureItem::Duplicate`) is replaced by a replacement branch:

1. Call `find_by_week` (unchanged).
2. Look for an active entry (`deleted.is_none()`) matching `day_of_week`.
3. If found: clone the existing entity, overwrite `day_type` and `time_of_day` with the already-normalized new values, assign a fresh version UUID (`"special-day-service::replace version"`), call `dao.update(&updated, "special-days-service::replace")`, return `SpecialDay::from(&updated)`.
4. If not found: fall through to the existing `dao.create` path (fresh id + version).

Pre-existing validation (ShortDay-needs-time, calendar_week bounds, Holiday time_of_day normalization, nil id/version checks) runs unchanged before the replacement branch.

**`.sqlx/` offline cache:** Regenerated via `cargo sqlx prepare --workspace` inside `nix develop`. Old entry deleted, new entry written for the extended UPDATE query. CI uses `SQLX_OFFLINE=true`; this prevents a clean-build / CI failure.

## SDF-01 Reproduction (D-03)

**Pre-fix behavior:**

A second POST to `/special-days` for an existing (year, calendar_week, day_of_week) — e.g., switching Monday/W1/2026 from `Holiday` to `ShortDay` — hit the duplicate guard and returned:

```
HTTP 422 Unprocessable Entity
{ "ValidationError": ["Duplicate"] }
```

The mapping is at `rest/src/lib.rs:187–189`: `ServiceError::ValidationError(_)` → status 422. The Schichtplan dropdown displayed an error and the type was never persisted.

**Post-fix behavior:**

The same POST replaces the existing row in place via a single atomic UPDATE. The response is HTTP 201 with the updated `SpecialDay` (existing id, new `day_type`, new `time_of_day`). No error surfaces in the shift-plan dropdown. Both entry surfaces (Schichtplan per-day dropdown and Settings Card-3) are fixed by this single central change (D-04), because both call the same `create` service path.

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED: `test(36-01)` | 519a9e2 | 3 tests failing as expected |
| GREEN: `feat(36-01)` | 3e521ec | All 14 special_days tests passing |
| REFACTOR | — | Not needed; code is already clean |

## Backend Gates

| Gate | Result |
|------|--------|
| `cargo test -p service_impl special_days` | 14/14 passed |
| `cargo test --workspace` | All passed (528 unit + 64 integration) |
| `cargo build` | Clean |
| `cargo clippy --workspace -- -D warnings` | Clean (no warnings) |
| `.sqlx/` cache | Regenerated; fresh `cargo sqlx prepare` yields no further diff |

## Invariants Confirmed

- **No new i18n text** added (pure backend logic change).
- **Snapshot schema version stays 12** — no persisted `BillingPeriodValueType` path touched.
- **No new REST endpoints** — POST `/special-days` contract unchanged (D-02).
- **Exactly one active row per date after a switch** — the replacement path calls `dao.update` once; `dao.create` is never called on the replace path.
- **Permission gate unchanged** — `SHIFTPLANNER_PRIVILEGE` check at the top of `create` is unaffected.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

No new external attack surface introduced. The fix is an internal-only service-path change behind the existing `SHIFTPLANNER_PRIVILEGE` gate. The REST contract is unchanged.

## Self-Check

Files exist:
- dao_impl_sqlite/src/special_day.rs — modified
- service_impl/src/special_days.rs — modified
- service_impl/src/test/special_days.rs — modified
- .sqlx/query-6cc953f85b1cdd138b7f134e56c79423b71e5f27abede1ada2b4ee87f698af3b.json — new

Commits exist:
- 519a9e2 — test(36-01): RED tests
- 3e521ec — feat(36-01): GREEN implementation + .sqlx

## Self-Check: PASSED
