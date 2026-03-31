## Why

The backend now supports assigning sales persons to specific shiftplans (permissive model: no assignments = eligible everywhere, has assignments = only eligible for assigned plans). The frontend needs UI to manage these assignments and must filter the sales person list when creating bookings, so shiftplanners only see eligible persons. Without this, the backend will reject bookings with 403 Forbidden but the user has no way to manage assignments or understand why a booking fails.

## What Changes

- Add a shiftplan assignment section to the SalesPersonDetails page (checkbox list of all shiftplans from catalog)
- Assignments are saved together with the existing Save button (not immediately on click)
- Works for both new and existing sales persons
- Add API functions to communicate with the new `/sales-person-shiftplan` endpoints
- Filter the sales person dropdown in the shiftplan booking view to only show eligible persons using `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}`
- Handle 403 Forbidden errors from booking attempts with a meaningful user-facing message
- Add i18n keys for all new UI text (En, De, Cs)

## Capabilities

### New Capabilities
- `sales-person-shiftplan-assignment`: UI for managing which shiftplans a sales person is allowed to be booked into, integrated into the SalesPersonDetails page
- `booking-eligibility-filter`: Filter the sales person list in the booking view to only show persons eligible for the current shiftplan

### Modified Capabilities

## Impact

- **Pages**: `sales_person_details.rs` (new section), `shiftplan.rs` (filtered person list)
- **API**: New functions in `api.rs` for `/sales-person-shiftplan` endpoints
- **Services**: `user_management.rs` extended with shiftplan assignment state and actions
- **State**: `user_management.rs` state extended with shiftplan assignment data
- **i18n**: New keys in `mod.rs`, `en.rs`, `de.rs`, `cs.rs`
- **Dioxus.toml**: May need proxy entry for `/sales-person-shiftplan` if not covered by existing proxy config
