## ADDED Requirements

### Requirement: UnpaidLeave variant in WorkingHoursCategory
The `WorkingHoursCategory` enum SHALL include an `UnpaidLeave` variant. It MUST support bidirectional conversion with `ExtraHoursCategoryTO::UnpaidLeave` and `ExtraHoursReportCategoryTO::UnpaidLeave`. It MUST have the identifier `"unpaid_leave"`, map to a dedicated i18n key, and be displayable.

#### Scenario: Identifier round-trip
- **WHEN** `WorkingHoursCategory::UnpaidLeave.identifier()` is called
- **THEN** it returns `"unpaid_leave"`

#### Scenario: From identifier parsing
- **WHEN** `WorkingHoursCategory::from_identifier("unpaid_leave")` is called
- **THEN** it returns `WorkingHoursCategory::UnpaidLeave`

#### Scenario: Conversion from ExtraHoursCategoryTO
- **WHEN** an `ExtraHoursCategoryTO::UnpaidLeave` is converted to `WorkingHoursCategory`
- **THEN** the result is `WorkingHoursCategory::UnpaidLeave`

#### Scenario: Conversion to ExtraHoursCategoryTO
- **WHEN** a `WorkingHoursCategory::UnpaidLeave` is converted to `ExtraHoursCategoryTO`
- **THEN** the result is `ExtraHoursCategoryTO::UnpaidLeave`

#### Scenario: Conversion from ExtraHoursReportCategoryTO
- **WHEN** an `ExtraHoursReportCategoryTO::UnpaidLeave` is converted to `WorkingHoursCategory`
- **THEN** the result is `WorkingHoursCategory::UnpaidLeave`

### Requirement: is_unpaid_leave helper method
The `WorkingHoursCategory` enum MUST provide an `is_unpaid_leave()` method that returns `true` only for the `UnpaidLeave` variant.

#### Scenario: Positive match
- **WHEN** `is_unpaid_leave()` is called on `WorkingHoursCategory::UnpaidLeave`
- **THEN** it returns `true`

#### Scenario: Negative match
- **WHEN** `is_unpaid_leave()` is called on any other variant (e.g., `Vacation`, `SickLeave`)
- **THEN** it returns `false`

### Requirement: Unpaid leave hours tracked in WorkingHours
The `WorkingHours` struct SHALL include an `unpaid_leave_hours: f32` field. It MUST be populated from `WorkingHoursReportTO.unpaid_leave_hours`.

#### Scenario: Mapping from WorkingHoursReportTO
- **WHEN** a `WorkingHoursReportTO` with `unpaid_leave_hours: 16.0` is converted to `WorkingHours`
- **THEN** the resulting `WorkingHours.unpaid_leave_hours` equals `16.0`

### Requirement: Unpaid leave hours tracked in Employee
The `Employee` struct SHALL include an `unpaid_leave_hours: f32` field. It MUST be populated from `EmployeeReportTO.unpaid_leave_hours` and default to `0.0` when constructed from `ShortEmployeeReportTO`.

#### Scenario: Mapping from EmployeeReportTO
- **WHEN** an `EmployeeReportTO` with `unpaid_leave_hours: 24.0` is converted to `Employee`
- **THEN** the resulting `Employee.unpaid_leave_hours` equals `24.0`

#### Scenario: Default from ShortEmployeeReportTO
- **WHEN** a `ShortEmployeeReportTO` is converted to `Employee`
- **THEN** the resulting `Employee.unpaid_leave_hours` equals `0.0`

### Requirement: Unpaid leave selectable in extra hours form
The add extra hours form dropdown SHALL include an "Unpaid Leave" option with value `"unpaid_leave"`. It MUST be placed among the absence category options.

#### Scenario: User selects unpaid leave
- **WHEN** a user opens the extra hours form and selects "Unpaid Leave" from the category dropdown
- **THEN** the form category is set to `WorkingHoursCategory::UnpaidLeave`

### Requirement: Unpaid leave entries displayed in employee view
The employee extra hours list view SHALL include a dedicated section for unpaid leave entries, similar to existing sections for vacation, holidays, and sick leave. Each entry MUST show date, hours, and description.

#### Scenario: Viewing unpaid leave entries
- **WHEN** an employee has unpaid leave extra hours entries
- **THEN** the employee detail view displays them under an "Unpaid Leave" heading with date, amount, and description for each entry

### Requirement: Unpaid leave hours shown in reports
The working hours summary and employee report views SHALL display `unpaid_leave_hours` as a labeled line item, similar to how vacation_hours, sick_leave_hours, and holiday_hours are displayed.

#### Scenario: Weekly working hours summary
- **WHEN** a user views the weekly working hours breakdown for an employee with unpaid leave hours
- **THEN** an "Unpaid Leave" line shows the hours value

#### Scenario: Employee overall report
- **WHEN** a user views the employee report with unpaid leave hours
- **THEN** an "Unpaid Leave" line shows the total hours value

### Requirement: i18n translations for unpaid leave
All unpaid leave labels SHALL have translations in English, German, and Czech locales.

#### Scenario: English label
- **WHEN** the locale is English
- **THEN** the unpaid leave category label is "Unpaid Leave"

#### Scenario: German label
- **WHEN** the locale is German
- **THEN** the unpaid leave category label is "Unbezahlter Urlaub"

#### Scenario: Czech label
- **WHEN** the locale is Czech
- **THEN** the unpaid leave category label is "Neplacene volno"
