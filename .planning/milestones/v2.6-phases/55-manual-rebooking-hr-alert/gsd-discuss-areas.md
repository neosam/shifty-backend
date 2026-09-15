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

**Current context:** The existing pattern shows the backend services in Phase 54 were broken into 8 plans covering different aspects (DAO, business logic, REST endpoints, frontend integration). We need to determine a similar breakdown for this phase.

### 2. F3 Modal Design Approach
What level of detail should be included in the manual rebooking modal?
- **Detailed approach**: Show IST/DANN table with calculations
- **Minimalist approach**: Just amount, direction, and week selection  
- Consider the fat backend principle - all backend-calculated values like "proposed_hours" are computed on the backend

### 3. State Conditional Update Approach
For concurrency control during approve/reject operations:
- Should we rely only on `WHERE state='pending'` (simpler but potentially less robust)
- Or should we implement optimistic locking with version checking (`WHERE state='pending' AND version=?`)

**Current context:** Planner documentation mentions that the task text says "state-conditional UPDATE (`WHERE state='pending'`, affected-rows==1)" and recommends `state='pending'` might be sufficient, but there's a note asking for clarification.

### 4. Integration with Existing Components
How should new modals and banners integrate:
- **Employee overview page**: Banner integration in HR path  
- **Employee details page**: TopBar trigger placement 
- **Error handling**: How slot collision errors (HTTP 409) should be handled in the UI

### 5. Service Trait Naming Convention
What naming pattern should follow existing conventions for the new `RebookingReconciliationService` trait:
- Based on established patterns in service/src/ and service_impl/src/
- Must fit into the layered architecture where Basic services don't depend on other Business logic services

### 6. Default Values and UX Considerations  
For the manual booking modal:
- What should be the default week selection?
- How to handle validation pre-submission? 
- Should there be any user guidance or hints in the UI?

### 7. Error Handling and UI Feedback
How should different types of errors be handled:
- HTTP 409 slot collisions from UNIQUE constraints
- Other potential service errors during approve/reject operations
- Integration with existing error handling patterns in the application

This analysis considers the existing project context of fat backend, thin client principle, layered architecture with defined service tiers, and established naming conventions documented in CLAUDE.md.