## Context

The backend has a new `sales_person_shiftplan` table and corresponding REST endpoints that implement a permissive assignment model: a sales person with no shiftplan assignments is eligible for all plans, while one with any assignments is only eligible for the assigned plans. The booking service enforces this at creation time (returns 403 if ineligible).

The frontend currently shows all active sales persons in the booking dropdown regardless of shiftplan. The SalesPersonDetails page manages basic info, settings, and user account linking. Shiftplan catalog data is already loaded in the shiftplan page via `load_shiftplan_catalog()`.

## Goals / Non-Goals

**Goals:**
- Allow admins to assign shiftplans to sales persons via checkbox list on the SalesPersonDetails page
- Save assignments atomically with the rest of the SalesPerson form via the existing Save button
- Filter the booking dropdown to only show eligible sales persons per shiftplan
- Provide meaningful error feedback when a booking is rejected as 403
- Support assignment management for both new and existing sales persons

**Non-Goals:**
- Shiftplanner-facing assignment UI (only admin for now)
- Bulk assignment management across multiple sales persons
- Backend changes (all endpoints already exist)
- Filtering soft-deleted shiftplans (backend responsibility)

## Decisions

### 1. Shiftplan assignment as a section in SalesPersonDetails, not a separate page

Assignments are a property of a sales person, not a standalone entity. Adding a new section below "Settings" keeps the workflow simple: open person → configure everything → save.

Alternative: Separate management page or modal. Rejected because it fragments the workflow and the data set is small (typically < 10 shiftplans).

### 2. Checkbox list for shiftplan selection

Each shiftplan from the catalog is shown as a checkbox. Checked = assigned. All unchecked = eligible everywhere (permissive model).

Alternative: Multi-select dropdown. Rejected because checkboxes make the current state immediately visible and match the Settings section pattern (is_paid, inactive checkboxes).

### 3. Local state until Save, then persist

Shiftplan assignments are tracked in `SelectedSalesPerson` state alongside `user_id`. On Save, the service first saves the sales person (POST or PUT), then calls PUT on `/sales-person-shiftplan/{id}/shiftplans` with the selected IDs. For new sales persons, the POST response provides the ID needed for the assignment call.

Alternative: Immediate save on checkbox click (like role assignments on user details). Rejected per user requirement — mixing immediate and deferred saves is confusing.

### 4. Use `get_bookable_sales_persons` endpoint for booking filter

Instead of loading all sales persons and filtering client-side, use the dedicated `GET /sales-person-shiftplan/by-shiftplan/{shiftplan_id}` endpoint. This keeps the eligibility logic server-authoritative and avoids duplicating the permissive model in the frontend.

Alternative: Client-side filtering by loading assignments for each person. Rejected — more API calls and duplicated logic.

### 5. Extend existing UserManagementAction enum

Add new variants (`UpdateShiftplanAssignments`, `LoadShiftplanAssignments`) to the existing coroutine-based service rather than creating a separate service. The assignment lifecycle is tied to the sales person lifecycle.

## Risks / Trade-offs

- **[Save ordering for new sales persons]** The sales person must exist before assignments can be saved. → Save sales person first, use returned ID, then save assignments. If assignment save fails, the person exists but without assignments (permissive model means they're eligible everywhere — safe default).

- **[Stale catalog data]** Shiftplan catalog is loaded once when the page opens. If someone creates a new shiftplan concurrently, it won't appear. → Acceptable for admin-only feature with low concurrency. Catalog is reloaded on page navigation.

- **[No route-level auth guard]** The SalesPersonDetails route is accessible to anyone who knows the URL, even without admin privilege. → Existing pattern in the codebase. Backend enforces permissions regardless. Out of scope for this change.
