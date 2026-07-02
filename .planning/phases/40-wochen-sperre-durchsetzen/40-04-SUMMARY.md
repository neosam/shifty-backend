---
phase: 40-wochen-sperre-durchsetzen
plan: 04
subsystem: rest
tags: [rust, rest-layer, week-lock, enforcement, security, openapi, utoipa, bypass-closure]

# Dependency graph
requires:
  - phase: 40-wochen-sperre-durchsetzen
    provides: "ShiftplanEditService::delete_booking (get->assert_week_not_locked->delete) + real lock enforcement + ServiceError::WeekLocked -> HTTP 423 arm"
provides:
  - "DELETE /booking/{id} routed through the lock-gated ShiftplanEditService::delete_booking (WST-04 bypass closed)"
  - "HTTP 423 documented in the OpenAPI responses of the annotated write endpoints (book_slot, copy_week) (D-40-01)"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "REST handler routing to Business-Logic tier (ShiftplanEditService) instead of Basic tier (BookingService) to inherit the lock gate"
    - "No-utoipa-annotation convention preserved for shiftplan-edit sibling handlers (delete_booking, edit_slot, delete_slot)"

key-files:
  created: []
  modified:
    - rest/src/booking.rs
    - rest/src/shiftplan_edit.rs

key-decisions:
  - "delete_booking handler now calls shiftplan_edit_service().delete_booking(...) instead of booking_service().delete(...); route and RestStateDef unchanged, success response (200 + empty body) unchanged. In the Locked case error_handler now returns 423 via the 40-01 arm."
  - "Added `use service::shiftplan_edit::ShiftplanEditService;` to rest/src/booking.rs (Rule 3 blocking fix) so delete_booking is in scope — mirrors the import in rest/src/shiftplan_edit.rs."
  - "423 documented only on the already-annotated handlers (book_slot_with_conflict_check, copy_week_with_conflict_check). delete_booking follows the documented no-utoipa-annotation codebase convention (rest/src/shiftplan_edit.rs:72,:237); no new OpenApi/ApiDoc infrastructure was forced into booking.rs."

requirements-completed: [WST-04, WST-03]

coverage:
  - id: D1
    description: "DELETE /booking/{id} routes through ShiftplanEditService::delete_booking (Basic-tier BookingService::delete bypass closed, WST-04)"
    requirement: "WST-04"
    verification:
      - kind: build
        detail: "cargo build -p rest green (handler compiles against delete_booking trait method)"
      - kind: test
        detail: "cargo test --workspace green — no regressions in booking-delete tests; delete path now inherits the 40-03 lock-enforcement matrix"
  - id: D2
    description: "HTTP 423 documented in OpenAPI on the annotated write endpoints (D-40-01)"
    requirement: "WST-03"
    verification:
      - kind: build
        detail: "book_slot_with_conflict_check and copy_week_with_conflict_check responses list (status = 423, ...); cargo build -p rest green"

metrics:
  tasks-completed: 2
  files-modified: 2
  duration: ~7m
  completed: 2026-07-02

status: complete
---

# Phase 40 Plan 04: DELETE /booking Re-Routing + OpenAPI 423 Summary

Re-routed `DELETE /booking/{id}` from the un-gated Basic-tier `BookingService::delete` to the lock-gated Business-Logic `ShiftplanEditService::delete_booking`, closing the last non-shiftplanner write bypass (WST-04), and documented the HTTP 423 lock response in the OpenAPI spec of the annotated write endpoints.

## What Was Built

**Task 1 — DELETE /booking/{id} re-routing (WST-04):**
- `rest/src/booking.rs` `delete_booking` handler now calls `rest_state.shiftplan_edit_service().delete_booking(booking_id, context.into(), None)` instead of `booking_service().delete(...)`.
- Route and `RestStateDef` are unchanged (`shiftplan_edit_service()` accessor already existed at `rest/src/lib.rs:436`). Success response (200 + empty body) unchanged; in a Locked week `error_handler` now returns 423 via the 40-01 `ServiceError::WeekLocked` arm.
- Delete semantics (Shiftplanner ∨ Self permission, conflict behavior) preserved — `delete_booking` internally delegates to `booking_service.delete` and only adds the lock gate.

**Task 2 — OpenAPI 423 documentation (D-40-01):**
- Added `(status = 423, description = "Week is locked — changes are not possible")` to the `responses(...)` blocks of `book_slot_with_conflict_check` (real 423 path: self-booker in a locked week) and `copy_week_with_conflict_check` (consistency; bypass-only in practice) in `rest/src/shiftplan_edit.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing trait import for delete_booking**
- **Found during:** Task 1
- **Issue:** `cargo build -p rest` failed with E0599 (`no method named delete_booking`) because the `ShiftplanEditService` trait was not in scope in `rest/src/booking.rs`.
- **Fix:** Added `use service::shiftplan_edit::ShiftplanEditService;` — mirrors the existing import in `rest/src/shiftplan_edit.rs:13`.
- **Files modified:** rest/src/booking.rs
- **Commit:** e1c9e6f

## Convention Note (delete_booking + utoipa)

`delete_booking` (rest/src/booking.rs) intentionally carries NO `#[utoipa::path]` annotation, consistent with its sibling shiftplan-edit handlers `edit_slot`/`edit_slot_single_week`/`delete_slot`, whose absent annotations are documented as a deliberate codebase convention at `rest/src/shiftplan_edit.rs:72` and `:237`. `booking.rs` also has no local `ApiDoc`/`OpenApi` frame. No new utoipa infrastructure was forced in; the 423 documentation lives on the already-annotated handlers only. No existing annotation was removed.

## Gate Results

- `cargo build -p rest` — green
- `cargo clippy --workspace -- -D warnings` — green
- `cargo test --workspace` — green (no regressions)

## Known Stubs

None.

## Threat Flags

None — no new security surface introduced (bypass closed, no new endpoints/schema).

## Commits

- e1c9e6f: feat(40-04): route DELETE /booking/{id} through ShiftplanEditService::delete_booking
- 12db006: docs(40-04): document HTTP 423 response in OpenAPI for write endpoints
