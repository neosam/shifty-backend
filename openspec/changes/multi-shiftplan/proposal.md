## Why

The system currently supports only a single, global shift plan. All slots and bookings exist in one shared namespace. In practice, the organization needs separate shift plans for different purposes -- e.g., a main store schedule, a baking schedule, and a cleaning schedule. Each plan has its own slot structure and serves a different planning need.

Some plans (like baking or cleaning) are purely organizational and should not count toward employee hour calculations or billing periods. Without multi-plan support, these use cases cannot be modeled.

## What Changes

- **New `shiftplan` table**: Stores shift plan metadata (id, name, `is_planning` flag, soft delete, versioning).
- **Slot association**: Each slot gets a `shiftplan_id` foreign key linking it to a specific plan.
- **Migration of existing data**: All existing slots are assigned to a default plan named "main" with `is_planning = false`.
- **Rename `ShiftplanService` to `ShiftplanViewService`**: Frees the `ShiftplanService` name for the new CRUD service managing shift plan entities.
- **New `ShiftplanService`**: CRUD operations for shift plan entities (create, get, list, update, delete).
- **Shiftplan-scoped queries**: `ShiftplanViewService.get_shiftplan_week` and `SlotService.get_slots_for_week` require a `shiftplan_id` parameter.
- **BREAKING**: `GET /shiftplan/{year}/{week}` replaced by `GET /shiftplan/{shiftplan_id}/{year}/{week}`.
- **Report filtering**: `ShiftplanReportService` queries automatically exclude plans where `is_planning = true`, so planning-only calendars never affect hour calculations or billing.
- **`bookings_view` update**: The database view is updated to include shiftplan context.

## Capabilities

### New Capabilities
- `shiftplan-catalog`: CRUD management of shift plan entities (create, list, get, update, delete) with `is_planning` flag to distinguish operational plans from planning-only calendars.

### Modified Capabilities

## Impact

- **Database**: New `shiftplan` table, `slot.shiftplan_id` column, updated `bookings_view`.
- **DAO layer**: New `ShiftplanDao`; `SlotDao` queries gain `shiftplan_id` filter; `ShiftplanReportDao` queries gain `is_planning` filter.
- **Service layer**: New `ShiftplanService` (CRUD); rename existing `ShiftplanService` to `ShiftplanViewService`; `SlotService` and `ShiftplanViewService` methods gain `shiftplan_id` parameter.
- **REST layer**: New CRUD endpoints for shift plans; existing shiftplan week endpoint changes path to include `shiftplan_id`.
- **rest-types**: New `ShiftplanTO`; `SlotTO` gains `shiftplan_id`.
- **Frontend**: Must be updated simultaneously (co-deployed) due to breaking API change.
- **All crates**: Rename references from `ShiftplanService` to `ShiftplanViewService`.
