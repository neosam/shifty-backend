# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) - Gray Areas

## Domain Boundary

Phase 55 implements two user-facing rebooking triggers that build upon the data model foundation established in Phase 54:

**F3 (Manuelle Umbuchung — REB-MANUAL-01..03):**
HR can initiate a manual rebooking of voluntary vs paid hours in the employee yearly report. This involves a new `RebookingReconciliationService` that orchestrates exactly one transaction with:
1. Two `ExtraHours` rows (`-N VolunteerWork`, `+N ExtraWork` — or vice versa);
2. A `rebooking_batch` (`kind=Manual`, `state=Approved`) + a `rebooking_batch_entry` with `extra_hours_out_id` / `extra_hours_in_id` FKs;
3. Marker (Phase-54-source-Spalte) on both ExtraHours-Rows to maintain rebooking neutrality in read aggregates.

**F5 (HR-Alert + Vorschlags-Modal — HR-ALERT-01..04):**
The employee overview shows a persistent warning banner for affected employees. Clicking the banner opens an IST/DANN suggestion modal; HR approve → same double-entry semantics as F3, but `kind=HrSuggestion, state=Approved`. HR reject → `state=Rejected` persists (audit trail).

## Already Decided Implementation Details

### D-55-01 (D-F5-01) — Alert Threshold: `balance <= -0.5h`
The predicate for the HR-alert warning banner is: `cap_planned_hours_to_expected = true` AND `balance <= -0.5h` AND `voluntary_ist > 0`. This uses a tolerance to avoid fake alerts from float-noise.

### D-55-02 (D-F5-02) — Alert Payload Shape: DTO-additive
The `ShortEmployeeReportTO` gets two additional fields: `has_pending_rebooking: bool` and `pending_rebooking_id: Option<Uuid>`. This follows precedent from VAA-04.

### D-55-03 (D-F5-03) — proposed_rebooking_hours = `min(|balance|, voluntary_ist)`
Backend-calculated value that balances the deficit without overbooking.

### D-55-04 (D-55-UNDO-01) — No Undo after Approve/Reject
This is explicitly deferred to v2.7+ as an anti-feature.

### D-55-05 (D-55-F3-01) — F3 Week Selection: HR chooses the week
HR can select any week (including historical weeks) for rebooking.

### D-55-06 (D-55-F3-02) — F3 Button Placement: TopBar / Action-Menu
The F3 trigger is in the TopBar or Action-Menu, not directly in the voluntary stats row.

### D-55-07 (D-55-F5-04) — Alert Removal Conditions
The alert disappears after either approve or reject - this aligns with the audit requirement that alerts should disappear when no longer relevant.

## Gray Areas Requiring Discussion

1. **Sub-plan breakdown** - Should there be separate sub-plans for backend BL, REST APIs and DTOs, property tests, and frontend components?

2. **F3 Modal design** - Should the manual rebooking modal be more detailed or minimalistic? 
   - Full IST/DANN details vs. minimal inputs (just amount + direction + week)
   - Consider backend vs frontend design patterns (Fat Backend, Thin Client principle)

3. **State conditional update approach** - Is `WHERE state='pending'` sufficient for concurrency control or should optimistic locking with version be used?
   - D-55-04 shows "state-conditional UPDATE (`WHERE state='pending'`, affected-rows==1)" but the planner notes that "Optimistic-Locking" might be preferred

4. **Integration approach** - How exactly will the new modal components integrate with existing employee overview and detail pages?
   - Banner integration in HR path
   - TopBar trigger placement

5. **Backend component naming pattern** - How should new service trait names follow existing conventions?
   - Will there be a `RebookingReconciliationService` trait that follows existing patterns?

6. **User experience for manual booking** 
   - What should the default values be in the manual modal (week selection, direction etc.)
   - Should there be any validation pre-submission?

7. **Error handling and UI feedback** 
   - How should HTTP 409 errors for slot collisions be handled in the frontend?
   - Are there existing patterns for handling such errors in the UI?

This analysis considers the existing project context (fat backend, thin client principle, layered architecture with defined service tiers) and prior phase decisions.