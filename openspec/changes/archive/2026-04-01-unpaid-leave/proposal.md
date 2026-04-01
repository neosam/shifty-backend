## Why

The system currently tracks vacation, sick leave, holidays, extra work, and unavailability as extra hour categories. There is no way to record **unpaid leave** -- an absence where the employee is not present (like vacation) but which must not count against their vacation entitlement. This is needed for HR to correctly handle cases where employees take time off without pay.

## What Changes

- Add a new `UnpaidLeave` variant to the extra hours category enum across all layers (DAO, Service, REST types).
- Classify `UnpaidLeave` as `AbsenceHours` (reduces expected hours, keeping balance neutral) and `Unavailable` (employee not available for scheduling).
- Track `unpaid_leave_hours` separately in reporting structs (`GroupedReportHours`, `ShortEmployeeReport`, `EmployeeReport`).
- Include unpaid leave hours in the `absence_days` calculation.
- No day-conversion (`unpaid_leave_days`) is needed -- only hours are tracked for billing.

## Capabilities

### New Capabilities
- `unpaid-leave`: Introduces the UnpaidLeave extra hours category with absence-based balance behavior and dedicated reporting fields.

### Modified Capabilities
<!-- No existing spec-level requirements change. The extra hours system gains a new enum variant but existing behavior is untouched. -->

## Impact

- **DAO layer**: `ExtraHoursCategoryEntity` enum, SQLite serialization/deserialization.
- **Service layer**: `ExtraHoursCategory` enum, `ReportType` and `Availability` mappings.
- **Reporting**: `GroupedReportHours`, `ShortEmployeeReport`, `EmployeeReport` structs; weekly aggregation logic in `service_impl/src/reporting.rs`.
- **REST types**: `ExtraHoursCategoryTO` enum, OpenAPI schema.
- **Tests**: New unit/integration tests for the UnpaidLeave category and its reporting behavior.
- **No database migration needed** -- the category is stored as a TEXT column and the new variant is just a new string value.
