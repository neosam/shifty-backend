## Why

The multi-shiftplan feature introduced plan-scoped slot queries (`get_slots_for_week` now requires a `shiftplan_id`). Several services that previously worked with the single global plan now pass `Uuid::nil()` as a placeholder, which matches no real plan and produces empty/incorrect results:

- **BlockService**: Generates iCal blocks from the weekly shiftplan view. Currently passes `Uuid::nil()`, so no blocks are generated.
- **BookingInformationService**: Calculates expected hours and volunteer summaries using slot data for a given week. Currently passes `Uuid::nil()`, so all calculations return zero slots.

Additionally, `SlotService::create_slot` accepts `shiftplan_id` as `Option<Uuid>` but does not validate that it is set, allowing slots without a plan association.

## What Changes

- **New DAO method**: `SlotDao::get_slots_for_week_all_plans` — returns all slots for a week that belong to non-planning shiftplans (`is_planning = false`), without requiring a specific plan ID.
- **New Service method**: `SlotService::get_slots_for_week_all_plans` — wraps the new DAO method with permission checks.
- **BlockService**: Uses `get_slots_for_week_all_plans` instead of `get_shiftplan_week` with `Uuid::nil()`, or receives a `shiftplan_id` parameter.
- **BookingInformationService**: Uses `get_slots_for_week_all_plans` instead of `get_slots_for_week` with `Uuid::nil()`.
- **Slot creation validation**: `SlotService::create_slot` validates that `shiftplan_id` is `Some(...)`.
- Remove all `Uuid::nil()` placeholder usages for shiftplan_id.

## Capabilities

### New Capabilities
- `cross-plan-slot-query`: Ability to query slots across all non-planning shift plans for a given week, used by services that need aggregate views.

### Modified Capabilities

## Impact

- **DAO layer**: New method on `SlotDao` trait and SQLite implementation.
- **Service layer**: New method on `SlotService`, updated `BlockService` and `BookingInformationService`.
- **Validation**: `SlotService::create_slot` gains a validation check.
- **No REST API changes**: The new method is internal; existing REST endpoints are unaffected.
- **No migration needed**: Uses existing schema, just a new query pattern.
