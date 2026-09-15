# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-07-22
**Phase:** 55-manual-rebooking-hr-alert
**Areas discussed:** Rebooking Approve/Reject Concurrency Control, F3 Manual Rebooking Modal Design, Sub-plan Breakdown Approach, State Conditional Update Implementation

---

## Rebooking Approve/Reject Concurrency Control

| Option     | Description                        | Selected |
| ---------- | ---------------------------------- | -------- |
| State-only check | `WHERE state='pending'` only |          |
| Version-based optimistic locking | `WHERE state='pending' AND version=?` | ✓        |

**User's choice:** Version-based optimistic locking
**Notes:** Discussed the pros and cons of simple state checking vs. version checking for concurrent operations. Chose version-based approach as it provides stronger guarantees. Aligns with system's existing version tracking mechanisms and best practices.

## F3 Manual Rebooking Modal Design

| Option     | Description                        | Selected |
| ---------- | ---------------------------------- | -------- |
| Detailed IST/DANN view | Modal shows full calculation details |          |
| Minimalist approach | Amount, direction, week selection only | ✓        |

**User's choice:** Minimalist approach
**Notes:** Chose to follow fat backend principle where calculations are done server-side. All values shown by FE are pre-computed in the backend.

## Sub-plan Breakdown Approach

| Option     | Description                        | Selected |
| ---------- | ---------------------------------- | -------- |
| 8-Plan approach (Phase 54 pattern) | Split into 8 separate plans like Phase 54 |          |
| 4–6 Plan approach | More focused breakdown with less granularity | ✓        |

**User's choice:** 4–6 Plan approach  
**Notes:** Followed the precedent of Phase 54 but adjusted for the scope of this phase, focusing on key areas: Business Logic Service, REST Endpoint, Property Tests, Frontend Integration.

## State Conditional Update Implementation

| Option     | Description                        | Selected |
| ---------- | ---------------------------------- | -------- |
| State-only update | Use only `WHERE state = 'pending'` pattern |          |
| Optimistic locking with version check | Use `WHERE state='pending' AND version=?` | ✓        |

**User's choice:** Optimistic locking with version check
**Notes:** Decision made based on project research (Pitfall 12) and existing conventions. Version checking provides better concurrency guarantees than simple state checks alone.

---

## the agent's Discretion

- Error handling for 409 conflicts during approve/reject operations
- Default values for manual rebooking modal fields (week selection, direction choices)
- Specific implementation details in service methods that weren't fully specified in requirements  

## Deferred Ideas

- Undo functionality after approve/reject (REB-UNDO-01) - explicitly deferred to v2.7+ as anti-feature
- CLI Backfill support (REB-AUTO-04) - remains in Phase 56 as HR-gated REST only