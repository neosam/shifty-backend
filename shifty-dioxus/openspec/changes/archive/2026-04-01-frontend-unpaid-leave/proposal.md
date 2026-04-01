## Why

The backend has introduced `UnpaidLeave` as a new `ExtraHoursCategory`. It behaves like other absence types (Vacation, SickLeave, Holiday) — it marks the employee as unavailable, counts toward absence days, and reduces expected hours. The frontend currently has no awareness of this category, so users cannot create, view, or report on unpaid leave entries.

## What Changes

- Add `UnpaidLeave` variant to the frontend's `WorkingHoursCategory` enum with all associated mappings (identifiers, i18n keys, Display, conversions to/from REST types)
- Add `unpaid_leave_hours: f32` field to `WorkingHours`, `Employee`, and their `From` implementations
- Add "Unpaid Leave" as a selectable category in the extra hours form dropdown (grouped with other absence types)
- Display unpaid leave entries in the employee detail view (extra hours list section)
- Display `unpaid_leave_hours` in the working hours summary and employee report views
- Add i18n translations for unpaid leave labels in all three locales (En, De, Cs)

## Capabilities

### New Capabilities
- `unpaid-leave-category`: Frontend support for the UnpaidLeave extra hours category — enum variant, REST type mappings, form selection, display in reports, and i18n translations

### Modified Capabilities

## Impact

- `src/state/employee.rs`: WorkingHoursCategory enum, WorkingHours struct, Employee struct, and all From impls
- `src/component/add_extra_hours_form.rs`: Category dropdown options
- `src/component/employee_view.rs`: Extra hours list display and working hours summary display
- `src/i18n/mod.rs`, `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs`: New translation keys
- `src/service/employee.rs`: Default initialization values
- Dependency: requires updated `rest-types` crate with `UnpaidLeave` variant and `unpaid_leave_hours` fields
