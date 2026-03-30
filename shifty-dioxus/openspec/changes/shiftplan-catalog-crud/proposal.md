## Why

The backend supports full CRUD operations for shiftplans (`POST/PUT/DELETE /shiftplan-catalog`), but the frontend only supports reading and selecting shiftplans via the tab bar. Users need to be able to create, rename, and delete shiftplans directly from the frontend without manual API calls.

## What Changes

- **Create shiftplan**: A `+` button in the tab bar (visible only in planning mode) opens a modal to enter the name for a new shiftplan
- **Rename shiftplan**: Double-clicking a tab in planning mode turns it into an inline text input for renaming
- **Delete shiftplan**: An `✕` button on each tab in planning mode opens a confirmation dialog before deleting
- **New API functions**: `create_shiftplan`, `update_shiftplan`, `delete_shiftplan` in `src/api.rs`
- **Tab bar enhancement**: `ShiftplanTabBar` component gains planning mode awareness with create/rename/delete UI

## Capabilities

### New Capabilities
- `shiftplan-crud`: Create, rename, and delete shiftplans from the tab bar in the shift plan view

### Modified Capabilities

## Impact

- `src/api.rs`: New API functions for POST, PUT, DELETE on `/shiftplan-catalog`
- `src/component/shiftplan_tab_bar.rs`: Extended with planning mode UI (✕ buttons, + button, inline edit)
- `src/page/shiftplan.rs`: Pass `change_structure_mode` to tab bar, handle catalog refresh after mutations
- `src/loader.rs`: New loader functions for create/update/delete operations
