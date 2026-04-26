## Why

The backend now supports multiple shift plans (shiftplan catalog) as first-class entities with CRUD operations. The existing API endpoints for the week view and slots now require a `shiftplan_id`. The frontend must reflect these changes so users can switch between different shift plans and see the correct data.

## What Changes

- **New API functions**: CRUD operations for the shiftplan catalog (`GET/POST/PUT/DELETE /shiftplan`)
- **Changed API signatures**: `get_shiftplan_week` and `get_slots_for_week` receive an additional `shiftplan_id` parameter
- **New REST type**: `ShiftplanTO` with `id`, `name`, `is_planning`, `deleted`, `version`
- **Extended REST type**: `SlotTO` gets an optional `shiftplan_id` field
- **Tab Bar UI**: Above the shift plan view for selecting the active shift plan
- **State management**: Track available shift plans and the currently selected one
- **Slot assignment**: New slots automatically receive the `shiftplan_id` of the active tab

## Capabilities

### New Capabilities
- `shiftplan-catalog`: Loading, displaying, and selecting shift plans via a tab bar in the shift plan view

### Modified Capabilities

## Impact

- `src/api.rs`: New API functions for shiftplan catalog, signature changes on existing functions
- `rest-types/src/lib.rs`: New `ShiftplanTO` type, `SlotTO` extended with `shiftplan_id`
- `src/state/`: New state for shiftplan selection
- `src/component/`: New tab bar component
- `src/page/`: Shiftplan page uses selected shiftplan
- `src/service/`: Loaders/services pass `shiftplan_id` to API calls
- `src/state/slot_edit.rs`: Slot editor sets `shiftplan_id` from active shiftplan
