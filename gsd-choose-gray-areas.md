# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) - Gray Areas for Discussion

## Domain Boundary

Phase 55 implements the two user-facing rebooking triggers that build upon the data model foundation established in Phase 54:

**F3 (Manuelle Umbuchung — REB-MANUAL-01..03):**
HR can initiate a manual rebooking of voluntary vs paid hours in the employee yearly report. This involves a new `RebookingReconciliationService` that orchestrates exactly one transaction with:
1. Two `ExtraHours` rows (`-N VolunteerWork`, `+N ExtraWork` — or vice versa);
2. A `rebooking_batch` (`kind=Manual`, `state=Approved`) + a `rebooking_batch_entry` with `extra_hours_out_id` / `extra_hours_in_id` FKs;
3. Marker (Phase-54-source-Spalte) on both ExtraHours-Rows to maintain rebooking neutrality in read aggregates.

**F5 (HR-Alert + Vorschlags-Modal — HR-ALERT-01..04):**
The employee overview shows a persistent warning banner for affected employees. Clicking the banner opens an IST/DANN suggestion modal; HR approve → same double-entry semantics as F3, but `kind=HrSuggestion, state=Approved`. HR reject → `state=Rejected` persists (audit trail).

## Already Decided Implementation Details

1. **Alert threshold**: `balance <= -0.5h` (with tolerance for float noise)
2. **Alert payload shape**: DTO-additive approach with `has_pending_rebooking: bool` and `pending_rebooking_id: Option<Uuid>`
3. **Suggested rebooking hours computation**: `min(|balance|, voluntary_ist)` 
4. **No undo feature after approve/reject** (deferred to v2.7+)
5. **Manual Rebooking week selection**: HR can choose the week freely for rebooking
6. **F3 button placement**: TopBar / Action-Menu in employee details page
7. **Alert removal conditions**: Alert disappears after either approve or reject

## Gray Areas for Discussion

### 1. Sub-plan Breakdown
Should there be separate sub-plans for:
- Backend business logic service (`RebookingReconciliationService`)
- REST endpoints and DTOs 
- Property tests for rebooking neutrality
- Frontend components, loaders, and i18n

The previous phase (54) was broken into 8 plans covering different aspects. We need to determine a similar breakdown approach.

### 2. F3 Modal Design Approach  
What level of detail should be included in the manual rebooking modal?
- Detailed IST/DANN table with backend calculations vs. minimalist input fields
- This respects the fat backend, thin client principle where most computations are done on the server-side

### 3. State Conditional Update Approach
For concurrency control during approve/reject operations:
- Should we rely on `WHERE state='pending'` only (simpler)
- Or implement optimistic locking with version checking (`WHERE state='pending' AND version=?`)
- The existing documentation already notes there's ambiguity about whether optimistic locking is required

### 4. Integration with Existing Components
How should new UI components integrate:
- Employee overview page: Banner integration in HR path  
- Employee details page: TopBar trigger placement
- Error handling: What happens when slot collisions occur (HTTP 409)?

### 5. Service Trait Naming Convention
How should the new `RebookingReconciliationService` name fit existing conventions from CLAUDE.md?
- Must follow the pattern of service tier classification where Basic services don't depend on Business logic services 
- Pattern established in service/src/rebooking_batch.rs

### 6. Default Values and UX Considerations
What should be default behaviors:
- What should be the default week selection for manual booking?
- How to handle pre-validation of inputs?
- User guidance or hints needed in UI?

### 7. Error Handling and UI Feedback
How should different error types be handled:
- HTTP 409 slot collisions from UNIQUE constraints
- Other potential service errors
- Integration with existing error handling patterns

These decisions will directly impact how downstream agents (researcher, planner) proceed with implementation. The context already establishes certain constraints around fat backend/thin client design principles and layered architecture that should be maintained.