## Why

The backend change `weekly-planned-hours-cap` (in `shifty-backend`) introduces two new concepts that the frontend currently cannot expose:

1. A per-`EmployeeWorkDetails` opt-in flag `cap_planned_hours_to_expected` that caps weekly shiftplan hours at `expected_hours`.
2. A new `VolunteerWork` extra-hours category whose hours are recorded but balance-neutral, plus a new `volunteer_hours` aggregate exposed on every report transport object.

Without UI work, planners cannot toggle the cap on a contract record, cannot manually book volunteer hours, and cannot see auto-attributed volunteer hours that the backend now computes silently. The feature ships invisible until the frontend follows.

## What Changes

- Add a `cap_planned_hours_to_expected: bool` field to the frontend `EmployeeWorkDetails` state model and propagate it through `EmployeeWorkDetailsTO` conversions in both directions.
- Add a labelled checkbox to the `EmployeeWorkDetailsForm` component, accompanied by a short helper text that explains the cap semantics; the checkbox is editable in `New`/`Edit` modes and disabled in `ReadOnly` mode, mirroring existing fields.
- Add a `VolunteerWork` variant to the frontend `WorkingHoursCategory` enum, including its identifier (`"volunteer_work"`), helper (`is_volunteer_work`), `Display`, i18n key mapping, and all three TO conversion impls (`ExtraHoursCategoryTO`, `ExtraHoursReportCategoryTO`).
- Add a `volunteer_hours: f32` field to the `WorkingHours` and `Employee` state structs and populate it from the corresponding `WorkingHoursReportTO` and `EmployeeReportTO` fields (default to `0.0` when constructing from `ShortEmployeeReportTO`).
- Add a `"Volunteer Work"` option to the `AddExtraHoursForm` dropdown, positioned near `"Extra Work"` since both represent actual presence.
- Display `volunteer_hours` as a dedicated line item in the per-week and per-period sections of `EmployeeView`, always shown, consistent with how other category aggregates are rendered.
- Map the new `"VOLUNTEER"` `value_type` to a localised label in `BillingPeriodDetails` so that snapshot rows for volunteer hours render with a human-readable header instead of a raw key.
- Add i18n keys and translations (En / De / Cs) for the new category label, the cap-flag form label, and a short helper text that describes the cap behaviour.

## Capabilities

### New Capabilities

- `weekly-planned-hours-cap-ui`: Frontend ability to toggle the `cap_planned_hours_to_expected` flag on an `EmployeeWorkDetails` record via the work-details form, including state plumbing through the `EmployeeWorkDetailsTO` conversions and a localised helper text that describes the cap behaviour.
- `volunteer-work-category-ui`: Frontend support for the `VolunteerWork` extra-hours category — enum variant with identifier and i18n mapping, all TO conversions, dropdown option in the add-extra-hours form, dedicated display line in the employee view, mapping of the persisted `"VOLUNTEER"` billing-period `value_type` to a localised label, and translations in all three supported locales.

### Modified Capabilities

*(none — both capabilities are new; existing frontend specs are not altered)*

## Impact

- **State layer**: `src/state/employee_work_details.rs` (new field, conversions); `src/state/employee.rs` (new enum variant, helpers, conversions, new struct fields, report mappings).
- **Components**: `src/component/employee_work_details_form.rs` (cap checkbox + helper text); `src/component/add_extra_hours_form.rs` (dropdown option); `src/component/employee_view.rs` (new line items in two report sections).
- **Pages**: `src/page/billing_period_details.rs` (new `value_type` label mapping).
- **i18n**: `src/i18n/mod.rs` (new keys); `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs` (new translations).
- **Backend coupling**: read-only consumer of `rest-types` fields already shipped by `weekly-planned-hours-cap`. No backend changes are part of this proposal.
- **Risks**: `WorkingHoursCategory::from_identifier` panics on unknown identifiers; the new variant must be added there to avoid runtime crashes when a server-side identifier is parsed. Compiler exhaustiveness checks cover every other touch point.
