## ADDED Requirements

### Requirement: Dynamic employees SHALL have correct vacation day calculations
The system SHALL calculate vacation days for dynamic employees using their actual contract weekly hours (not the balance-adjusted zero value). The formula `vacation_hours / (contract_weekly_hours / workdays_per_week)` SHALL use the real contract hours for all employees regardless of dynamic status.

#### Scenario: Dynamic employee takes a full week of vacation
- **WHEN** a dynamic employee with a 40h/5-day contract takes 40 hours of vacation in a week
- **THEN** the report SHALL show 5 vacation days for that week

#### Scenario: Dynamic employee takes partial vacation in a worked week
- **WHEN** a dynamic employee with a 40h/5-day contract takes 8 hours of vacation and works 32 hours in a week
- **THEN** the report SHALL show 1 vacation day for that week

#### Scenario: Dynamic employee balance remains zero
- **WHEN** a dynamic employee's report is generated
- **THEN** the balance (Stundenkonto) SHALL still be forced to 0 as before

### Requirement: Non-dynamic employee calculations SHALL remain unchanged
The fix SHALL not alter vacation day or balance calculations for non-dynamic (fixed-salary) employees.

#### Scenario: Non-dynamic employee vacation days unchanged
- **WHEN** a non-dynamic employee with a 40h/5-day contract takes 16 hours of vacation
- **THEN** the report SHALL show 2 vacation days (unchanged behavior)
