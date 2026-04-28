## ADDED Requirements

### Requirement: Employee metadata in template context
Each sales person entry in the billing period template context SHALL include `name` (string), `is_paid` (boolean), and `is_dynamic` (boolean) fields.

#### Scenario: Name is available in template
- **WHEN** a billing period report is rendered
- **THEN** each sales person entry in the template context SHALL include the `name` field from the corresponding `SalesPerson` entity

#### Scenario: is_paid is available in template
- **WHEN** a billing period report is rendered
- **THEN** each sales person entry in the template context SHALL include the `is_paid` field from the corresponding `SalesPerson` entity

#### Scenario: is_dynamic is true when any work details entry is dynamic
- **WHEN** a billing period report is rendered for a sales person who has at least one `EmployeeWorkDetails` entry with `is_dynamic = true`
- **THEN** the `is_dynamic` field in the template context SHALL be `true`

#### Scenario: is_dynamic is false when no work details are dynamic
- **WHEN** a billing period report is rendered for a sales person whose `EmployeeWorkDetails` entries all have `is_dynamic = false`
- **THEN** the `is_dynamic` field in the template context SHALL be `false`

#### Scenario: is_dynamic is false when no work details exist
- **WHEN** a billing period report is rendered for a sales person who has no `EmployeeWorkDetails` entries
- **THEN** the `is_dynamic` field in the template context SHALL be `false`

### Requirement: Complete value types populated
The billing period report SHALL populate all defined `BillingPeriodValueType` variants from reporting data, including `ExtraWork`, `VacationHours`, `SickLeave`, `Holiday`, `VacationDays`, and `VacationEntitlement`.

#### Scenario: Vacation hours populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `VacationHours` value type SHALL be populated with `vacation_hours` from the reporting service (delta, ytd_from, ytd_to, full_year)

#### Scenario: Sick leave hours populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `SickLeave` value type SHALL be populated with `sick_leave_hours` from the reporting service

#### Scenario: Holiday hours populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `Holiday` value type SHALL be populated with `holiday_hours` from the reporting service

#### Scenario: Extra work hours populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `ExtraWork` value type SHALL be populated with `extra_work_hours` from the reporting service

#### Scenario: Vacation days populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `VacationDays` value type SHALL be populated with `vacation_days` from the reporting service

#### Scenario: Vacation entitlement populated
- **WHEN** a billing period report is built for a sales person
- **THEN** the `VacationEntitlement` value type SHALL be populated with `vacation_entitlement` from the reporting service

### Requirement: Dictionary-based values access
Each sales person entry in the template context SHALL include a `values_map` field that provides dictionary-based access to values keyed by value type string.

#### Scenario: values_map contains all value types
- **WHEN** a billing period report is rendered
- **THEN** the `values_map` field SHALL contain an entry for each value type (e.g., `values_map.overall`, `values_map.balance`, `values_map.vacation_hours`)

#### Scenario: values_map entries have all four metrics
- **WHEN** a value type entry exists in `values_map`
- **THEN** it SHALL contain `delta`, `ytd_from`, `ytd_to`, and `full_year` fields

#### Scenario: values array still available
- **WHEN** a billing period report is rendered
- **THEN** the existing `values` array format SHALL still be present alongside `values_map`

#### Scenario: values array and values_map contain same data
- **WHEN** a billing period report is rendered
- **THEN** the data in `values_map` SHALL be consistent with the data in the `values` array

### Requirement: Backward compatibility
Existing templates using the `values` array format and `sales_person_id` lookups SHALL continue to produce identical output after this change.

#### Scenario: Existing Tera template produces same output
- **WHEN** an existing Tera template that uses `values` array iteration is rendered
- **THEN** the output SHALL be identical to the output before this change

#### Scenario: Existing MiniJinja template produces same output
- **WHEN** an existing MiniJinja template that uses `values` array iteration is rendered
- **THEN** the output SHALL be identical to the output before this change
