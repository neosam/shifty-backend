## Why

The current shiftplan API only supports querying a single shiftplan for an entire calendar week (`GET /shiftplan/{shiftplan_id}/{year}/{week}`). There is no way to get a consolidated view of all shiftplans for a specific day. Users need a daily aggregate view that shows all shiftplans together for a given day — useful for daily operations where seeing who is working across all plans at a glance matters more than the weekly view of a single plan.

## What Changes

- **New REST endpoint**: `GET /shiftplan-day/{year}/{week}/{day_of_week}` returns all shiftplans for a specific day, grouped by shiftplan.
- **New service method**: `ShiftplanViewService::get_shiftplan_day` aggregates data across all shiftplans for one day.
- **Refactored day-building logic**: The existing per-day logic inside `get_shiftplan_week` (slot filtering, special day handling, booking assignment, self_added calculation) is extracted into a shared helper function `build_shiftplan_day`. Both `get_shiftplan_week` and the new `get_shiftplan_day` reuse it.
- **New DTOs**: `ShiftplanDayAggregateTO` and `PlanDayViewTO` for the REST response.

## Capabilities

### New Capabilities
- `shiftplan-day-aggregate`: Ability to query a consolidated day view across all shiftplans, with slots grouped by plan, including special day handling (holidays, short days).

### Modified Capabilities

## Impact

- **Service layer**: New method on `ShiftplanViewService` trait. Refactored `get_shiftplan_week` implementation to use extracted helper.
- **REST layer**: New endpoint and route registration in `rest/src/shiftplan.rs` and `rest/src/lib.rs`.
- **rest-types**: New `ShiftplanDayAggregateTO` and `PlanDayViewTO` structs with `ToSchema` for OpenAPI.
- **Dependencies**: `ShiftplanViewService` gains a dependency on `ShiftplanService` (catalog) to enumerate all plans.
- **No database changes**: Uses existing queries; no new migrations needed.
