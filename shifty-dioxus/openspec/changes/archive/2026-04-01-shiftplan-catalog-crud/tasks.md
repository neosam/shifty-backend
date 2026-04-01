## 1. API functions

- [x] 1.1 Add `create_shiftplan(config, name)` to `src/api.rs` (POST /shiftplan-catalog)
- [x] 1.2 Add `update_shiftplan(config, shiftplan)` to `src/api.rs` (PUT /shiftplan-catalog/{id})
- [x] 1.3 Add `delete_shiftplan(config, id)` to `src/api.rs` (DELETE /shiftplan-catalog/{id})

## 2. Tab bar planning mode

- [x] 2.1 Add `planning_mode: bool` prop to `ShiftplanTabBar`
- [x] 2.2 Show `✕` button on each tab when `planning_mode` is true
- [x] 2.3 Show `+` button at end of tab bar when `planning_mode` is true

## 3. Create shiftplan modal

- [x] 3.1 Add create modal state (visible, name input) and render modal with text input and submit/cancel buttons
- [x] 3.2 On submit: call `create_shiftplan` API, emit event to refresh catalog and select new shiftplan

## 4. Delete shiftplan with confirmation

- [x] 4.1 Add delete confirmation state and render confirmation dialog when `✕` is clicked
- [x] 4.2 On confirm: call `delete_shiftplan` API, emit event to refresh catalog

## 5. Inline rename

- [x] 5.1 Add editing state per tab (editing_id, edit_name) and switch to input on double-click in planning mode
- [x] 5.2 On Enter/blur: call `update_shiftplan` API, emit event to refresh catalog. On Escape: revert.

## 6. Page integration

- [x] 6.1 Pass `change_structure_mode` as `planning_mode` to `ShiftplanTabBar` in `src/page/shiftplan.rs`
- [x] 6.2 Handle catalog refresh callbacks: restart `shiftplan_catalog` resource, update `selected_shiftplan_id` accordingly

## 7. Tests

- [x] 7.1 Write tests for new API function signatures and ShiftplanTO serialization for create/update
