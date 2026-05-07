## Why

Shiftplan properties (currently only name, but `is_planning` already exists in the data model) cannot be edited through a proper dialog. The current UI uses inline rename on double-click and a minimal create modal with only a name field. As more properties need to be configurable (starting with `is_planning`), a unified edit/create dialog is needed.

## What Changes

- Replace the inline rename (double-click) with an edit dialog modal that shows all shiftplan properties
- Extend the create modal to use the same dialog, supporting name and `is_planning` fields
- One shared modal component serves both create and edit modes
- The dialog is only accessible in structure mode (change_structure_mode)
- Double-click on a tab opens the edit dialog instead of inline rename

## Capabilities

### New Capabilities
- `shiftplan-edit-dialog`: Unified modal dialog for creating and editing shiftplan properties (name, is_planning)

### Modified Capabilities

## Impact

- `src/component/shiftplan_tab_bar.rs`: Replace inline rename and simple create modal with unified edit/create dialog
- `src/api.rs`: `create_shiftplan` may need to accept `is_planning` parameter
- No backend changes required — `ShiftplanTO.is_planning` and `update_shiftplan` API already exist
