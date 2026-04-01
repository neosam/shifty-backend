## 1. State and API Layer

- [x] 1.1 Add `ShiftplanAssignment` struct (with `shiftplan_id: Uuid` and `permission_level: String`) to the state module
- [x] 1.2 Update `api::get_shiftplan_assignments` to return `Vec<ShiftplanAssignment>` (deserialize `ShiftplanAssignmentTO` from backend)
- [x] 1.3 Update `api::set_shiftplan_assignments` to accept and send `Vec<ShiftplanAssignment>` as JSON body
- [x] 1.4 Update `SelectedSalesPerson.shiftplan_assignments` from `Vec<Uuid>` to `Vec<ShiftplanAssignment>`

## 2. Service Layer

- [x] 2.1 Update `load_shiftplan_assignments` in `user_management.rs` to handle `Vec<ShiftplanAssignment>`
- [x] 2.2 Update `save` logic to send `Vec<ShiftplanAssignment>` when calling `set_shiftplan_assignments`
- [x] 2.3 Update `UpdateShiftplanAssignments` action to carry `Vec<ShiftplanAssignment>`

## 3. UI

- [x] 3.1 Update shiftplan assignment checkbox logic in `sales_person_details.rs` to work with `ShiftplanAssignment` (default new assignments to `available`)
- [x] 3.2 Add permission level dropdown (`<select>`) next to each checked shiftplan checkbox with options "Available" and "Planner Only"
- [x] 3.3 Wire dropdown change events to update permission level in local state

## 4. i18n

- [x] 4.1 Add i18n keys for permission level labels: `Available`, `PlannerOnly` (and optionally `PermissionLevel` as section label)
- [x] 4.2 Add English translations
- [x] 4.3 Add German translations
- [x] 4.4 Add Czech translations

## 5. Verification

- [x] 5.1 Verify the app compiles without errors (`cargo check`)
- [x] 5.2 Run existing tests (`cargo test`) and fix any failures from type changes
