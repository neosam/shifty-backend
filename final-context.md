# Phase 55: Manual Rebooking + HR-Alert-Modal (F3 + F5) - Context

**Gathered:** 2026-07-22  
**Status:** Ready for planning  

<domain>
## Phase Boundary

Phase 55 implements the two user-facing rebooking triggers that build upon the data model foundation established in Phase 54:

**F3 (Manuelle Umbuchung — REB-MANUAL-01..03):**
HR can initiate a manual rebooking of voluntary vs paid hours in the employee yearly report. This involves a new `RebookingReconciliationService` that orchestrates exactly one transaction with:
1. Two `ExtraHours` rows (`-N VolunteerWork`, `+N ExtraWork` — or vice versa);
2. A `rebooking_batch` (`kind=Manual`, `state=Approved`) + a `rebooking_batch_entry` with `extra_hours_out_id` / `extra_hours_in_id` FKs;
3. Marker (Phase-54-source-Spalte) on both ExtraHours-Rows to maintain rebooking neutrality in read aggregates.

**F5 (HR-Alert + Vorschlags-Modal — HR-ALERT-01..04):**
The employee overview shows a persistent warning banner for affected employees. Clicking the banner opens an IST/DANN suggestion modal; HR approve → same double-entry semantics as F3, but `kind=HrSuggestion, state=Approved`. HR reject → `state=Rejected` persists (audit trail).

</domain>

<decisions>
## Implementation Decisions

### Rebooking Approve/Reject Concurrency Control
- **D-55-08:** Use version-based optimistic locking for approve/reject operations
  - **Approach:** `WHERE state='pending' AND version=?` in database updates
  - **Rationale:** Provides stronger concurrency guarantees than simple state checking alone
  - **Justification:** Aligns with best practices and system's existing version tracking mechanisms

### F3 Manual Rebooking Modal Design
- **D-55-09:** Minimalist approach for manual rebooking modal
  - **Content:** Amount, direction, week selection only  
  - **Rationale:** Fat backend principle where all calculations (proposed hours) are done on server-side
  - **Consistency:** All values shown by FE are pre-computed in the backend

### Sub-plan Breakdown Approach
- **D-55-10:** Follow Phase 54 pattern of 4–6 distinct plans
  - **Backend BL Service**
  - **REST Endpoints** 
  - **Property Tests for Rebooking Neutrality**
  - **Frontend Components**  
  - **Integration Points**
  - **Docs-Sync** (can be combined with FE integration)

### State Conditional Update Implementation
- **D-55-11:** Implement optimistic locking with version check
  - **Database Pattern:** `WHERE state='pending' AND version=?` with affected rows check 
  - **Concurrency Handling:** Returns HTTP 409 on failed version checks
  - **Rationale:** Ensures atomicity for concurrent approval/rejection operations

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope Documentation
- `CLAUDE.md` - System architecture, layered patterns, service tier conventions, error handling approaches
- `.planning/ROADMAP.md` - Phase goals, requirements mapping, and implementation strategy for v2.6 milestone
- `.planning/REQUIREMENTS.md` - Complete requirements specification for F1-F5 features (REB-MANUAL-01..03, HR-ALERT-01..04)

### Data Model and Service Documentation
- `docs/features/F14-rebooking.md` - Technical specification of rebooking data model, batch structures
- `docs/architecture/02-service-tiers.md` - Service tier classification rules (Basic vs Business Logic services)  
- `migrations/sqlite/20260707000000_create-rebooking-batch.sql` - Database schema definition for rebooking tables

### Architecture and Conventions
- `.planning/research/PITFALLS.md` - Critical issues to avoid (Pitfall 12 on state conditional updates, Pitfall 6 on race conditions)
- `shifty-bin/src/main.rs` - Service wiring for DI in the application 
- `service/src/rebooking_batch.rs` - Existing service trait definitions
- `service_impl/src/rebooking_batch.rs` - Basic service implementation patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `service/src/rebooking_batch.rs` - Existing trait definitions that follow Basic Service pattern
- `service_impl/src/rebooking_batch.rs` - Basic service implementation patterns with transaction management
- `service/src/voluntary_stats.rs` - Business logic service demonstrating HR-only content handling 
- REST layers (`rest/src/`) - Pattern for API endpoints using `#[utoipa::path]` annotations

### Established Patterns
- Fat Backend, Thin Client - All business logic calculations happen in backend services; frontend displays DTOs
- Layered Architecture - Services follow defined service tier rules to prevent cyclic dependencies
- Service Method Pattern - Every service method accepts `Option<Transaction>` and handles transactions properly
- Error Handling - Consistent mapping of `ServiceError` to appropriate HTTP status codes
- Authentication Context - All service methods receive `Authentication<Context>` 

### Integration Points
- `shifty_bin/src/main.rs` - Service DI wiring needs updated for new `RebookingReconciliationServiceImpl`
- REST layer in `rest/src/` - New endpoints for manual rebooking and suggestion management needed
- Frontend routing in `shifty-dioxus/Dioxus.toml` - Need proxy routes for new `/rebooking/*` paths

</code_context>

<specifics>
## Specific Ideas

- "Follow existing design principles" - maintain fat backend, thin client pattern throughout
- "Use consistent state handling" - similar to other services in the codebase that handle pending states  
- "Ensure database constraints are respected" - leverage existing UNIQUE constraint for concurrency control
- "Minimal UI footprint" - avoid overloading the employee details page with too much information

</specifics>

<deferred>
## Deferred Ideas

### Reviewed Todos (not folded)
- None - discussion stayed within phase scope

### Future Considerations 
- Undo functionality after approve/reject (REB-UNDO-01) - explicitly deferred to v2.7+ as anti-feature
- CLI Backfill support (REB-AUTO-04) - remains in Phase 56 as HR-gated REST only

</deferred>

---

*Phase: 55-manual-rebooking-hr-alert*
*Context gathered: 2026-07-22*