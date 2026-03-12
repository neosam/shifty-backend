## Context

Billing periods can currently only be mass-deleted via `clear_all_billing_periods()`. When a billing period is created with incorrect data (e.g., wrong end date), the only recovery is to delete all periods and recreate them. The system needs a targeted delete for the most recent billing period only, following existing soft-delete patterns.

The existing codebase already has:
- Soft delete infrastructure on `billing_period` and `billing_period_sales_person` tables
- `clear_all` methods on both DAOs that set `deleted`/`deleted_at` timestamps
- Permission checking via `HR_PRIVILEGE`
- `all_ordered_desc()` DAO method that returns periods sorted by start date descending

## Goals / Non-Goals

**Goals:**
- Allow soft-deleting the most recent billing period by ID
- Enforce that only the latest period can be deleted (business rule)
- Cascade soft delete to associated sales person entries
- Follow existing patterns for DAO methods, service methods, REST endpoints, and error handling

**Non-Goals:**
- Deleting arbitrary (non-latest) billing periods
- Hard delete of billing periods
- Undo/restore of soft-deleted billing periods
- Recalculation of carryover hours after deletion

## Decisions

### 1. New `ServiceError::NotLatestBillingPeriod(Uuid)` variant

**Rationale**: Existing error variants don't capture this business rule. `EntityConflicts` is for version conflicts, not business logic. A dedicated variant makes the error semantically clear and maps cleanly to HTTP 409.

**Alternative**: Reuse `EntityConflicts` — rejected because the semantics are wrong (this isn't a version conflict).

### 2. Explicit ID parameter with server-side validation

**Rationale**: The client must pass the ID of the billing period to delete. The server validates it is the latest. This prevents accidental deletion of the wrong period due to stale client state — the client must know which period it's deleting.

**Alternative**: Endpoint without ID that auto-deletes the latest — rejected because it's error-prone if the client's view is stale.

### 3. DAO-level `delete_by_id` and `delete_by_billing_period_id` methods

**Rationale**: Following the existing pattern where `clear_all` is a concrete DAO method, the new delete methods will also be concrete implementations. The `delete_by_billing_period_id` on `BillingPeriodSalesPersonDao` handles the cascade.

**Alternative**: Reuse `update` method to set deleted fields — rejected because a dedicated method is clearer and follows the `clear_all` precedent.

### 4. Service method ordering: find_by_id → all_ordered_desc → cascade delete

**Rationale**: Check existence first (404), then check if latest (409), then delete. This gives the most specific error for each failure mode.

## Risks / Trade-offs

- **[Stale client state]** → Client must pass the correct ID; server validates it's the latest. Stale clients get 409 and must refresh.
- **[No cascade enforcement at DB level]** → Cascade is handled in application code, not via foreign key constraints. This is consistent with the existing `clear_all` pattern. → Mitigation: Service method deletes sales person entries first, then the billing period.
- **[No undo]** → Soft-deleted periods cannot be restored via API. → Acceptable for the use case (mistake correction). Database records are preserved and could be manually restored if needed.
