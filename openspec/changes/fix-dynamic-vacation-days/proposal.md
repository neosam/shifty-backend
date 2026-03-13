## Why

When an employee is marked as "dynamic" (paid per actual hours worked rather than a fixed salary), the billing period report always shows 0 vacation days. This is because the mechanism that forces the balance to 0 for dynamic employees also overwrites the contract weekly hours used for the vacation-days-per-hour calculation. Vacation days should be calculable for dynamic employees the same way as for fixed-salary employees.

## What Changes

- Fix the `contract_weekly_hours` field in `GroupedReportHours` so that it reflects the actual contract hours for dynamic employees, not the artificially zeroed value used for balance calculation.
- The balance-zeroing logic for dynamic employees remains unchanged — only the vacation day (and similar day-based) calculations are affected.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none — this is a bug fix in existing calculation logic, not a requirement change)

## Impact

- **Code**: `service_impl/src/reporting.rs` — the `hours_per_week` function where `contract_weekly_hours` is assigned from `expected_hours` (which is 0 for dynamic employees). Needs to use `dynamic_working_hours_for_week` instead.
- **Behavior**: Dynamic employees will correctly show vacation days, sick leave days, holiday days, and absence days in billing period reports.
- **APIs**: No REST API changes. The existing `BillingPeriodValueType::VacationDays` value will now return correct non-zero values for dynamic employees.
- **Risk**: Low — only affects the day-calculation path. Balance calculation is untouched.
