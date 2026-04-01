## Context

The extra hours system uses an enum (`ExtraHoursCategory`) to classify different types of time entries. Each variant maps to a `ReportType` (how it affects the balance calculation) and an `Availability` (whether the employee is available). The enum is mirrored across three layers: DAO entity, service model, and REST transport object.

Reporting aggregates extra hours per category into dedicated fields (e.g., `vacation_hours`, `sick_leave_hours`) used by the frontend for display and by billing for calculations.

Adding a new built-in category requires touching each layer but follows a well-established pattern -- the same pattern used for `Vacation`, `SickLeave`, `Holiday`, and `Unavailable`.

## Goals / Non-Goals

**Goals:**
- Add `UnpaidLeave` as a first-class extra hours category.
- Classify it as `AbsenceHours` + `Unavailable` so it reduces expected hours and marks the employee as unavailable.
- Track `unpaid_leave_hours` separately in all reporting structs.
- Include unpaid leave in `absence_days` calculation.

**Non-Goals:**
- No `unpaid_leave_days` field -- only hours are needed for billing.
- No database migration -- the category column is TEXT and accepts any string.
- No changes to the custom extra hours system.
- No frontend changes (those will be handled separately in shifty-dioxus).

## Decisions

### 1. New enum variant rather than custom extra hours

**Decision**: Add `UnpaidLeave` as a built-in enum variant in `ExtraHoursCategory` / `ExtraHoursCategoryEntity` / `ExtraHoursCategoryTO`.

**Rationale**: The custom extra hours system currently only supports `WorkingHours` or `None` report types and always sets availability to `Available`. Unpaid leave needs `AbsenceHours` + `Unavailable`, which would require extending the custom system. Since unpaid leave is a universally needed HR concept (not company-specific), a built-in variant is more appropriate and simpler.

**Alternative considered**: Extending `CustomExtraHours` with configurable `ReportType` and `Availability` fields. Rejected because it adds schema migration complexity and unpaid leave is a standard concept.

### 2. No day conversion

**Decision**: Only track `unpaid_leave_hours`, no `unpaid_leave_days` field.

**Rationale**: The business only needs hours for billing. Adding a days field would add unnecessary complexity. If days are needed later, it's a straightforward addition.

### 3. Include in absence_days

**Decision**: `absence_days()` calculation will sum unpaid leave hours alongside vacation, sick leave, and holiday hours.

**Rationale**: The employee is genuinely absent. `absence_days` represents total days the employee was not present, regardless of pay status.

### 4. Database serialization

**Decision**: Serialize as the string `"UnpaidLeave"` in the `category` TEXT column, matching the pattern of other variants.

**Rationale**: Consistent with existing serialization (`"Vacation"`, `"SickLeave"`, etc.). No migration needed since the column accepts any text value.

## Risks / Trade-offs

- **[Backward compatibility]** Older application versions that read the database will encounter an unknown `"UnpaidLeave"` category string. → This is acceptable since the application is deployed as a single unit (backend + frontend together). The DAO `TryFrom` implementation will need to handle the new variant.
- **[Frontend not included]** The frontend will need a corresponding update to display and create unpaid leave entries. → This is tracked separately and is expected.
