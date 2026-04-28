## Context

The multi-shiftplan feature introduced the ability to have multiple shift plans, each with its own set of slots. Currently, any sales person can be booked into any slot in any plan — there is no mechanism to restrict eligibility. Administrators need to control which employees belong to which plans as the number of plans grows.

The existing permission system (RBAC with roles/privileges) controls who can perform actions, but not which domain objects they can act upon. This change introduces a domain-level restriction on the relationship between sales persons and shift plans.

## Goals / Non-Goals

**Goals:**
- Allow administrators to assign sales persons to specific shift plans via an N:M relationship
- Enforce eligibility at booking creation time (hard validation in the service layer)
- Provide REST endpoints for managing assignments and querying bookable sales persons
- Use a permissive model: unassigned sales persons can be booked everywhere

**Non-Goals:**
- Visibility restrictions (filtering which plans a user can see) — future enhancement on the same data model
- Retroactive enforcement of existing bookings
- Frontend implementation (backend-only change)
- New privilege types — reuse `shiftplanner` for assignment management

## Decisions

### 1. Permissive eligibility model

**Decision**: A sales person with zero assignments is eligible for all plans. Once at least one assignment exists, they are restricted to only assigned plans.

**Rationale**: This avoids a breaking migration that would need to assign all existing sales persons to all existing plans. Existing workflows remain untouched. Restriction is opt-in per sales person.

**Alternative considered**: Restrictive model (must be explicitly assigned). Rejected because it requires a data migration and could break existing setups if the migration is incomplete.

### 2. Assignment management from the sales person side

**Decision**: The PUT endpoint to set assignments lives under `/api/sales-person/{id}/shiftplans`, accepting a list of shiftplan IDs. This replaces the full set of assignments atomically.

**Rationale**: The primary UI workflow is editing a sales person and selecting their plans via checkboxes. A full-replace PUT is simpler than individual add/remove operations and matches the UI interaction model.

**Alternative considered**: Endpoints on the shiftplan side (`/api/shiftplan-catalog/{id}/sales-persons`). Rejected because the UI entry point is the employee management screen, not the plan management screen.

### 3. Simple join table without additional columns

**Decision**: The `sales_person_shiftplan` table contains only the two foreign keys as a composite primary key, plus audit columns (`update_timestamp`, `update_process`).

**Rationale**: Current requirement is only booking eligibility. Future visibility controls can be added via an additional column or a separate table. YAGNI — don't add columns we don't need yet.

**Alternative considered**: Adding `can_view` / `can_book` boolean columns now. Rejected to keep the initial implementation simple.

### 4. Eligibility check in booking service

**Decision**: The booking service `create()` method will load the full slot (not just `exists()`) to get `shiftplan_id`, then check eligibility via the new DAO.

**Rationale**: The slot must be loaded to determine which plan it belongs to. The eligibility check is a simple query: does the sales person have any assignments? If yes, is this plan among them?

### 5. Separate bookable sales persons endpoint

**Decision**: A new `GET /api/sales-person/by-shiftplan/{shiftplan_id}` endpoint returns all sales persons eligible to be booked in a given plan, applying the permissive logic.

**Rationale**: The frontend needs this for the booking UI — showing only eligible employees when creating a booking in a specific plan. This is distinct from the management endpoint which returns only explicit assignments.

## Risks / Trade-offs

- **[Performance]** The eligibility check adds one extra query per booking creation (check assignments). → Acceptable for booking creation frequency. Can be optimized with caching if needed later.
- **[Consistency]** Existing bookings may violate new assignments. → Accepted by design. No retroactive enforcement. Administrators are informed this is forward-looking only.
- **[Permissive model edge case]** Removing all assignments from a sales person makes them eligible everywhere again. → This is intentional behavior, but could surprise administrators. Document clearly in API/UI.
