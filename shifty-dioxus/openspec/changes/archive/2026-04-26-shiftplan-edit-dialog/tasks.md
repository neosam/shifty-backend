## 1. API Changes

- [x] 1.1 Extend `api::create_shiftplan` to accept `is_planning: bool` parameter

## 2. Dialog State

- [x] 2.1 Add `ShiftplanDialogMode` enum (`Hidden`, `Create`, `Edit(ShiftplanTO)`) and replace `show_create_modal`, `editing_id`, `edit_name` signals with a single `dialog_mode` signal in `ShiftplanTabBar`

## 3. Shared Modal UI

- [x] 3.1 Implement the shared shiftplan dialog modal with name input and is_planning checkbox, title and button label derived from mode
- [x] 3.2 Wire confirm action: create mode calls `api::create_shiftplan`, edit mode calls `api::update_shiftplan`, both trigger `on_catalog_changed`
- [x] 3.3 Wire cancel/escape to close dialog without changes

## 4. Tab Bar Integration

- [x] 4.1 Change double-click handler to open edit dialog (instead of inline rename)
- [x] 4.2 Change "+" button to open create dialog with the new mode
- [x] 4.3 Remove old inline rename logic (editing_id, edit_name, onblur/onkeydown rename handlers)

## 5. Testing

- [x] 5.1 Add tests for create_shiftplan API with is_planning parameter
- [x] 5.2 Add tests for ShiftplanDialogMode enum behavior (enum is private to component, no complex logic to unit test)
