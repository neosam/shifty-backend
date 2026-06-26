---
phase: 24-paid-capacity-enforcement-config
plan: 01
subsystem: api
tags: [rust, axum, service-error, http-409, sqlite-migration, toggle, paid-capacity]

# Dependency graph
requires:
  - phase: 23-frontend-slot-paid-capacity-ui
    provides: max_paid_employees slot field + soft warning infrastructure that this plan extends to hard enforcement
provides:
  - ServiceError::PaidLimitExceeded { current: u8, max: u8 } variant (non-403 error contract)
  - HTTP 409 Conflict mapping for PaidLimitExceeded in rest/src/lib.rs error_handler
  - OpenAPI 409 response documented on book_slot_with_conflict_check endpoint
  - migration seeding paid_limit_hard_enforcement toggle with enabled=0 (soft default)
affects:
  - 24-02 (backend enforcement — consumes ServiceError::PaidLimitExceeded + toggle)
  - 24-05 (frontend inline block handler — matches HTTP 409 status)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ServiceError struct-variant with named fields: #[error(\"...\")]\nPaidLimitExceeded { current: u8, max: u8 }"
    - "error_handler match arm using err @ ServiceError::Variant { .. } pattern for 409"
    - "Toggle seed migration: INSERT INTO toggle without CREATE TABLE (table pre-exists)"

key-files:
  created:
    - migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql
  modified:
    - service/src/lib.rs
    - rest/src/lib.rs
    - rest/src/shiftplan_edit.rs

key-decisions:
  - "PaidLimitExceeded maps to HTTP 409 (not 403) — 403 is silently dropped by the frontend (D-13); 409 aligns with EntityConflicts precedent and is frontend-surfaceable"
  - "Toggle stored in existing toggle table (not feature_flag) per D-24-01a — feature_flag is reserved for SaaS/marketing gating"
  - "Default enabled=0 (soft) ensures no regression on existing installs (D-24-01)"
  - "Migration only does INSERT — toggle table already exists from 20260105000000_app-toggles.sql"

patterns-established:
  - "Non-403 error contract: any new BookingError that the frontend must surface must use a status other than 403"
  - "Toggle seed pattern: INSERT INTO toggle (name, enabled, description, update_process) VALUES (..., 0, ...)"

requirements-completed: [D-24-01, D-24-01a, D-24-07, D-24-05]

# Metrics
duration: 20min
completed: 2026-06-27
---

# Phase 24 Plan 01: Error Contract + Toggle Seed Summary

**ServiceError::PaidLimitExceeded { current, max } added with HTTP 409 mapping and paid_limit_hard_enforcement toggle seeded as soft-default (enabled=0)**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-06-27T00:00:00Z
- **Completed:** 2026-06-27T00:20:00Z
- **Tasks:** 3
- **Files modified:** 4 (3 modified + 1 created)

## Accomplishments
- Added `ServiceError::PaidLimitExceeded { current: u8, max: u8 }` to `service/src/lib.rs` before `InternalError`, using thiserror struct-variant form
- Added HTTP 409 mapping arm in `rest/src/lib.rs` `error_handler` for `PaidLimitExceeded` — specifically NOT 403 (which the frontend silently ignores per D-13)
- Updated `#[utoipa::path]` on `book_slot_with_conflict_check` to document the new 409 response (OpenAPI gate)
- Created `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` seeding `paid_limit_hard_enforcement` with `enabled=0` (soft default, no regression) in the existing toggle table

## Task Commits

No commits made — per vcs_jj_only instruction, all changes left in working copy for manual jj commit by user.

## Files Created/Modified
- `service/src/lib.rs` - Added `PaidLimitExceeded { current: u8, max: u8 }` variant before `InternalError`
- `rest/src/lib.rs` - Added `err @ ServiceError::PaidLimitExceeded { .. }` -> 409 arm in `error_handler`
- `rest/src/shiftplan_edit.rs` - Added `(status = 409, description = "Paid employee limit exceeded — booking blocked")` to `#[utoipa::path]` responses
- `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` - INSERT seeding `paid_limit_hard_enforcement` toggle with `enabled=0`

## Decisions Made
- HTTP 409 chosen over other non-403 statuses because it semantically fits "booking conflicts with capacity limit" and aligns with the existing `EntityConflicts`->409 precedent in `error_handler`
- Toggle stored in `toggle` table (not `feature_flag`) per D-24-01a; `toggle_admin` privilege already exists from prior migration

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None — all three backend gates (cargo build, cargo test, cargo clippy -D warnings) passed on first attempt.

## Known Stubs

None — this plan only adds error infrastructure (enum variant + HTTP mapping + migration seed). No UI stubs or placeholder data flows.

## Threat Flags

None — the error message exposes only small integer counts (current/max), no PII, no IDs. The toggle seed enforces the safe default (enabled=0 = soft). Both align with T-24-01 and T-24-02 mitigations in the plan's threat model.

## User Setup Required

None — the migration is additive (INSERT only, table pre-exists). The toggle will be applied automatically on next `sqlx migrate run`. No environment variables or dashboard steps required.

## Next Phase Readiness
- Error contract is complete: `ServiceError::PaidLimitExceeded` + HTTP 409 ready for 24-02 (backend enforcement) to emit and 24-05 (frontend) to match
- Toggle seed ready: `paid_limit_hard_enforcement` will be in the database after `sqlx migrate run`
- No blockers for 24-02 or subsequent plans

## Self-Check

- [x] `service/src/lib.rs` contains `PaidLimitExceeded { current: u8, max: u8 }` — verified by grep returning 1
- [x] `rest/src/lib.rs` contains `PaidLimitExceeded` match arm with status 409 — verified by grep
- [x] `rest/src/shiftplan_edit.rs` #[utoipa::path] responses contain `status = 409` — verified by grep
- [x] Migration file exists and contains `paid_limit_hard_enforcement` without `CREATE TABLE` — verified by bash test
- [x] `cargo build --workspace` — PASSED (2m 02s)
- [x] `cargo test --workspace` — PASSED (471 unit + 61 integration = 532+ tests, 0 failed)
- [x] `cargo clippy --workspace -- -D warnings` — PASSED (no warnings)

## Self-Check: PASSED

---
*Phase: 24-paid-capacity-enforcement-config*
*Completed: 2026-06-27*
