## Context

The backend has added `UnpaidLeave` as a new `ExtraHoursCategory` variant. It follows the same pattern as existing absence categories (Vacation, SickLeave, Holiday): it marks the employee as unavailable, counts toward absence days, and reduces expected hours. The `rest-types` crate (shared between backend and frontend) already includes the new `ExtraHoursCategoryTO::UnpaidLeave` variant and `unpaid_leave_hours` fields on `WorkingHoursReportTO`, `EmployeeReportTO`, and `ShortEmployeeReportTO`.

The frontend currently does not handle this variant, which would cause match exhaustiveness errors once the updated `rest-types` dependency is pulled in.

## Goals / Non-Goals

**Goals:**
- Full parity with existing absence categories (Vacation, SickLeave, Holiday) for the new UnpaidLeave category
- Users can create, view, and delete unpaid leave entries
- Unpaid leave hours are displayed in all report views where other absence hours appear
- All three locales (En, De, Cs) have translations for unpaid leave labels

**Non-Goals:**
- No special business logic beyond what the backend already implements (absence days calculation, expected hours reduction)
- No separate "unpaid leave days" calculation (unlike vacation which has vacation_days) — only hours are tracked
- No changes to the weekly overview absence calculation (unpaid leave is already included in `absence_hours` from the backend)

## Decisions

### Follow the existing category pattern exactly

Every existing absence category (Vacation, SickLeave, Holiday) has:
1. A `WorkingHoursCategory` enum variant
2. An `is_*()` helper method
3. An `identifier()` string and `from_identifier()` parser
4. A `to_i18n_key()` mapping
5. A `Display` implementation
6. Bidirectional `From` conversions with `ExtraHoursCategoryTO` and `ExtraHoursReportCategoryTO`
7. Fields on `WorkingHours` and `Employee` structs
8. A section in the employee_view component listing entries
9. An option in the add_extra_hours_form dropdown
10. i18n keys in all three locales

UnpaidLeave will follow this same pattern with no deviations. This keeps the code consistent and predictable.

**Alternative considered:** Using the existing `Custom` category mechanism. Rejected because unpaid leave is a built-in backend category with dedicated fields, not a user-defined custom category.

### Place dropdown option with absence types

In the extra hours form dropdown, "Unpaid Leave" will be placed after "Unavailable" and before the custom separator. This groups it with other absence-related options.

### No description text for unpaid leave section

Unlike "Unavailable" and "Extra Work" which have explanatory description paragraphs, unpaid leave needs no additional explanation — the label is self-descriptive.

## Risks / Trade-offs

- **[rest-types version sync]** The frontend must use the updated `rest-types` that includes `UnpaidLeave`. If the backend changes are not merged yet, the frontend changes will not compile. **Mitigation:** The backend diff shows the changes are ready; coordinate deployment.
- **[Czech translation accuracy]** "Neplacene volno" is the standard Czech term for unpaid leave but should be verified by a native speaker. **Mitigation:** Translation can be easily updated later.
