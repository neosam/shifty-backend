## Why

The billing period template context currently only exposes `sales_person_id` (a UUID) and a `values` array for each sales person. Templates cannot access employee names, payment type, or dynamic/variable status without hardcoding UUIDs. Additionally, several `BillingPeriodValueType` variants (vacation, sick leave, holidays, extra work) are defined in the enum but never populated with data from the reporting service. This forces template authors to write verbose, fragile templates that cannot dynamically filter employees or show a full hours breakdown.

## What Changes

- **Enrich sales person data in template context**: Add `name`, `is_paid`, and `is_dynamic` fields to each sales person in the template rendering context. `is_dynamic` is `true` if any `EmployeeWorkDetails` entry for that person has `is_dynamic = true`.
- **Populate missing value types**: Fill `VacationHours`, `SickLeave`, `Holiday`, `ExtraWork`, `VacationDays`, and `VacationEntitlement` from the existing reporting service data.
- **Add `values_map` dict format**: Provide an additional `values_map` dictionary (keyed by value type string) alongside the existing `values` array. Existing templates using the array format continue to work unchanged.
- **Add `EmployeeWorkDetailsService` dependency**: The `BillingPeriodReportService` needs access to `EmployeeWorkDetailsService` to resolve `is_dynamic` and `expected_hours` per sales person.

## Capabilities

### New Capabilities
- `enriched-template-context`: Enriched billing period template context with employee metadata and complete value types

### Modified Capabilities
- `template-engine-selection`: The template context data available to both engines is expanded with new fields (additive, non-breaking)

## Impact

- **Code**: `service_impl/src/billing_period_report.rs` (context building, new dependency), `service_impl/src/test/billing_period_report.rs` (new tests)
- **APIs**: No REST API changes — the enrichment is internal to template rendering
- **Dependencies**: New `EmployeeWorkDetailsService` dependency in `BillingPeriodReportServiceImpl`
- **Backward compatibility**: Fully backward compatible — existing templates continue to work. New fields and `values_map` are purely additive.
