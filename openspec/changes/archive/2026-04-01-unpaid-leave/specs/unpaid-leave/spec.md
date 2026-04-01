## ADDED Requirements

### Requirement: UnpaidLeave extra hours category exists
The system SHALL provide an `UnpaidLeave` variant in the extra hours category enum at all layers (DAO entity, service model, REST transport object).

#### Scenario: Creating an unpaid leave entry
- **WHEN** a user creates an extra hours entry with category `UnpaidLeave`
- **THEN** the system SHALL persist the entry with category `"UnpaidLeave"` in the database

#### Scenario: Reading an unpaid leave entry from database
- **WHEN** the database contains an extra hours row with category `"UnpaidLeave"`
- **THEN** the system SHALL deserialize it as `ExtraHoursCategoryEntity::UnpaidLeave`

#### Scenario: REST API round-trip
- **WHEN** a client sends a POST request with category `UnpaidLeave`
- **THEN** the system SHALL accept it and return the entry with category `UnpaidLeave` in the response

### Requirement: UnpaidLeave is classified as absence hours
The `UnpaidLeave` category SHALL have report type `AbsenceHours`. This means unpaid leave hours reduce the expected hours for the period, keeping the balance neutral.

#### Scenario: Balance calculation with unpaid leave
- **WHEN** an employee has 40 expected hours in a week and 8 hours of unpaid leave
- **THEN** the effective expected hours SHALL be 32 (40 - 8) and the balance SHALL not be negatively affected by the unpaid leave

### Requirement: UnpaidLeave marks employee as unavailable
The `UnpaidLeave` category SHALL have availability `Unavailable`. The employee is not available for scheduling during unpaid leave.

#### Scenario: Availability during unpaid leave
- **WHEN** an employee has an unpaid leave entry for a given time period
- **THEN** the system SHALL report the employee as unavailable for that period

### Requirement: Unpaid leave hours tracked separately in reports
The reporting structs (`GroupedReportHours`, `ShortEmployeeReport`, `EmployeeReport`) SHALL include a dedicated `unpaid_leave_hours` field that aggregates only `UnpaidLeave` extra hours entries.

#### Scenario: Weekly report includes unpaid leave hours
- **WHEN** an employee has 8 hours of unpaid leave and 8 hours of vacation in a week
- **THEN** the grouped report SHALL show `unpaid_leave_hours: 8.0` and `vacation_hours: 8.0` separately

#### Scenario: Employee report aggregates unpaid leave hours
- **WHEN** an employee has unpaid leave entries across multiple weeks
- **THEN** the `EmployeeReport` SHALL sum all unpaid leave hours in the `unpaid_leave_hours` field

### Requirement: Unpaid leave does not affect vacation entitlement
The `UnpaidLeave` category SHALL NOT be counted toward vacation days consumption. There SHALL be no `unpaid_leave_days` field or vacation-day conversion.

#### Scenario: Vacation days unaffected by unpaid leave
- **WHEN** an employee has 24 hours of vacation and 16 hours of unpaid leave
- **THEN** the vacation days calculation SHALL only consider the 24 vacation hours, not the unpaid leave hours

### Requirement: Unpaid leave included in absence days
The `absence_days()` calculation SHALL include `unpaid_leave_hours` in its sum alongside vacation, sick leave, and holiday hours.

#### Scenario: Absence days includes unpaid leave
- **WHEN** an employee has 8 hours vacation, 8 hours sick leave, and 8 hours unpaid leave with 8 hours_per_day
- **THEN** `absence_days()` SHALL return 3.0 (24 total absence hours / 8 hours_per_day)
