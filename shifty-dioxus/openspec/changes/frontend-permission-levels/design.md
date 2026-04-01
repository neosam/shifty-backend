## Context

The backend has added a `permission_level` field (`available` | `planner_only`) to sales person shiftplan assignments. The two REST endpoints (`GET` and `PUT` on `/{id}/shiftplans`) now use `ShiftplanAssignmentTO { shiftplan_id, permission_level }` instead of plain UUIDs. The frontend currently sends and receives `Vec<Uuid>`, which will break against the updated backend.

The `get_bookable_sales_persons` endpoint response format is unchanged — the backend now filters by caller role server-side, so no frontend changes are needed there.

## Goals / Non-Goals

**Goals:**
- Adapt the frontend API layer, state, and UI to the new assignment format
- Allow admins to set `available` or `planner_only` per shiftplan assignment
- Default new assignments to `available` (matching backend default)
- Add i18n support for permission level labels

**Non-Goals:**
- Changing the booking UI or booking flow (backend handles permission filtering)
- Adding role-based visibility in the frontend (backend enforces this)
- Modifying the `get_bookable_sales_persons` API function (response unchanged)

## Decisions

### 1. Introduce a `ShiftplanAssignment` struct in frontend state

Rather than using tuples or raw JSON, introduce a small struct:
```rust
struct ShiftplanAssignment {
    pub shiftplan_id: Uuid,
    pub permission_level: String,
}
```

This lives in the state/service layer and maps directly to the backend DTO. Using a String for `permission_level` (rather than an enum) keeps it flexible if additional levels are added later and matches the backend's approach.

**Alternative considered**: Using an enum — rejected because the backend uses a String and adding a new level would require frontend changes even if the frontend doesn't need to handle it specially.

### 2. Permission level selector as inline dropdown

When a shiftplan checkbox is checked, show a `<select>` dropdown next to it with options "Available" and "Planner Only". When unchecked, hide the dropdown. This keeps the UI compact and consistent with the existing checkbox pattern.

**Alternative considered**: Separate column or table layout — rejected as overengineered for two options.

### 3. Keep `get_bookable_sales_persons` unchanged

The backend now handles role-based filtering server-side. The frontend function signature and usage remain the same.

## Risks / Trade-offs

- **[String-based permission level]** → If the backend adds new levels, the frontend dropdown will need updating. Mitigation: the backend constrains values via CHECK constraint, and new levels would require coordinated changes anyway.
- **[No frontend validation of permission values]** → The frontend sends whatever the dropdown provides. Mitigation: The backend validates via SQL CHECK constraint.
