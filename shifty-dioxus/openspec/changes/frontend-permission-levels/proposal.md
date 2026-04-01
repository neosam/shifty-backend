## Why

The backend now supports permission levels (`available` / `planner_only`) on sales person shiftplan assignments. The frontend still sends and receives plain UUID arrays, which will break once the backend change is deployed. The UI also lacks any way to set or display the permission level for an assignment.

## What Changes

- **BREAKING**: Update API functions `get_shiftplan_assignments` and `set_shiftplan_assignments` to use the new `ShiftplanAssignmentTO` format (`{ shiftplan_id, permission_level }`) instead of `Vec<Uuid>`
- Update the `SelectedSalesPerson` state to carry permission levels alongside shiftplan IDs
- Add a permission level selector (dropdown) next to each shiftplan assignment checkbox in the SalesPersonDetails page
- Add i18n keys for the permission level labels

## Capabilities

### New Capabilities

### Modified Capabilities
- `sales-person-shiftplan-assignment`: Assignments now carry a permission level field that must be displayed and editable
- `booking-eligibility-filter`: API function signatures change from `Vec<Uuid>` to the new assignment DTO format

## Impact

- **API layer** (`src/api.rs`): `get_shiftplan_assignments` return type and `set_shiftplan_assignments` request body change
- **State** (`src/service/user_management.rs`): `SelectedSalesPerson.shiftplan_assignments` type changes from `Vec<Uuid>` to a struct carrying permission levels
- **UI** (`src/page/sales_person_details.rs`): Checkbox list gets an additional dropdown per active assignment
- **i18n** (`src/i18n/`): New keys for "Available", "Planner Only", and section labels
