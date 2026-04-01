## Context

The sales person to shiftplan assignment system (introduced in `sales-person-shiftplan-assignment`) uses a permissive model: no assignments means eligible everywhere, having assignments restricts to listed plans. Currently the eligibility check is binary -- either a person can be booked into a plan or not, regardless of who performs the booking.

Shift planners need the flexibility to book people into plans that those people cannot self-assign to. This requires a permission level on the assignment that distinguishes between full self-service access and planner-only access.

## Goals / Non-Goals

**Goals:**
- Add a `permission_level` field to shiftplan assignments with values `available` and `planner_only`
- Make eligibility checks context-aware (who is performing the booking matters)
- Ensure `get_bookable_sales_persons` returns the correct list based on the caller's role
- Prevent self-service booking deletion for `planner_only` assignments
- Maintain backward compatibility: existing assignments default to `available`

**Non-Goals:**
- Frontend changes (handled separately in shifty-dioxus)
- Changing the permissive default (no assignments = eligible everywhere stays)
- Adding an explicit "excluded" state (exclusion remains implicit via not being listed)

## Decisions

### 1. Permission level as TEXT column with CHECK constraint

Add `permission_level TEXT NOT NULL DEFAULT 'available' CHECK(permission_level IN ('available', 'planner_only'))` to the `sales_person_shiftplan` table.

**Why TEXT over INTEGER:** Matches existing patterns in the codebase (e.g., process columns are TEXT). More readable in queries and debugging. SQLite has no enum type, and a CHECK constraint enforces valid values.

**Alternative considered:** Boolean `planner_only` column. Rejected because a TEXT enum is more extensible if a third level is ever needed, and is more self-documenting.

### 2. Extend `is_eligible` with authentication context

Change `is_eligible` signature to accept `Authentication<Context>` so it can distinguish between shiftplanner and self-service calls.

Logic:
- No assignments for person → always eligible (unchanged)
- Has assignment for this plan with `available` → eligible for everyone
- Has assignment for this plan with `planner_only` → eligible only if caller has `SHIFTPLANNER_PRIVILEGE`
- No assignment for this plan (but has other assignments) → not eligible for anyone

**Why on `is_eligible` and not on the booking service:** The eligibility concept belongs in the shiftplan assignment service. The booking service already delegates to `is_eligible` and should continue to do so, just passing context through.

### 3. Extend `get_bookable_sales_persons` to be role-aware

The method already receives `Authentication<Context>` but currently ignores it (parameter named `_context`). It will use the context to determine which permission levels to include:

- Shiftplanner caller → return persons with `available` OR `planner_only` assignments (plus unassigned persons)
- Non-shiftplanner caller → return only persons with `available` assignments (plus unassigned persons)

### 4. Extend DAO to store and retrieve permission level

The DAO needs to:
- Store `permission_level` when setting assignments
- Return permission level info when checking assignments (new method or modified `is_assigned`)
- New method `get_permission_level(sales_person_id, shiftplan_id)` returning `Option<String>` (None if not assigned)

The `set_for_sales_person` method needs to accept permission levels alongside shiftplan IDs.

### 5. Booking deletion permission check

When a non-shiftplanner tries to delete a booking, check if the booking's shiftplan has a `planner_only` assignment for that person. If so, deny deletion.

### 6. REST API changes

- `PUT /{id}/shiftplans` currently accepts `Vec<Uuid>`. Change to accept a list of objects with `shiftplan_id` and `permission_level`. **BREAKING** change for this endpoint.
- `GET /{id}/shiftplans` should return permission levels alongside shiftplan IDs
- `GET /by-shiftplan/{shiftplan_id}` behavior changes based on caller role (no API shape change needed)

## Risks / Trade-offs

- **[Breaking API change]** The PUT endpoint changes from `Vec<Uuid>` to structured objects. → Mitigation: Frontend and backend are deployed together; coordinate the change.
- **[Migration]** Existing rows need a default permission level. → Mitigation: `DEFAULT 'available'` in migration ensures backward compatibility.
- **[Performance]** `get_bookable_sales_persons` now needs to check permission levels per person. → Mitigation: Already iterating all persons; adding permission level check is marginal overhead.

## Migration Plan

1. Add migration: `ALTER TABLE sales_person_shiftplan ADD COLUMN permission_level TEXT NOT NULL DEFAULT 'available' CHECK(permission_level IN ('available', 'planner_only'))`
2. Existing rows automatically get `available` via DEFAULT
3. No data migration needed
4. Rollback: Drop the column (SQLite requires table rebuild, but in practice rollback would mean reverting the migration)
