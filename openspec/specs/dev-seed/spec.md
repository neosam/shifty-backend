# dev-seed Specification

## Purpose
TBD - created by archiving change dev-seed-endpoint. Update Purpose after archive.
## Requirements
### Requirement: Seed test data endpoint
The system SHALL provide a `POST /dev/seed` endpoint that creates a fixed set of test data for local development. The endpoint SHALL only be compiled when the `mock_auth` feature flag is enabled. The endpoint SHALL be documented in OpenAPI/Swagger.

The endpoint SHALL create the following data:

**Sales Persons:**

| Name           | Active | is_paid | Background Color |
|----------------|--------|---------|------------------|
| Anna Müller    | yes    | true    | #FF6B6B (red)    |
| Max Schmidt    | yes    | true    | #4ECDC4 (teal)   |
| Lisa Weber     | no     | true    | #45B7D1 (blue)   |
| Tom Bauer      | yes    | false   | #96CEB4 (green)  |
| Sarah Fischer  | yes    | true    | #FFEAA7 (yellow) |

**Employee Work Details (contracts):**

| Sales Person   | Hours/Week | Workdays | Vacation Days |
|----------------|------------|----------|---------------|
| Anna Müller    | 40h        | Mo-Fr    | 30            |
| Max Schmidt    | 20h        | Mo-Mi    | 15            |
| Lisa Weber     | 30h        | Mo-Do    | 24            |
| Tom Bauer      | 10h        | Sa-So    | 0             |
| Sarah Fischer  | 35h        | Mo-Fr    | 28            |

**Extra Hours:**

| Sales Person   | Category   | Amount | Day (relative to current week) |
|----------------|------------|--------|-------------------------------|
| Anna Müller    | Vacation   | 8h     | Monday of current week        |
| Max Schmidt    | SickLeave  | 8h     | Tuesday of current week       |
| Sarah Fischer  | ExtraWork  | 2h     | Wednesday of current week     |

**Bookings (current calendar week, on existing default slots):**

| Sales Person   | Booked Days                        | Note                    |
|----------------|------------------------------------|-------------------------|
| Anna Müller    | Tue, Wed, Thu, Fri                 | Mon = vacation          |
| Max Schmidt    | Mon                                | Tue = sick              |
| Tom Bauer      | Sat                                | Weekend volunteer       |
| Sarah Fischer  | Mon, Tue, Wed, Thu, Fri            | Full week               |

**Special Days:**

| Day              | Calendar Week | Day of Week | Type     | Time  |
|------------------|---------------|-------------|----------|-------|
| Karfreitag       | KW 14         | Friday      | Holiday  | —     |
| Ostermontag      | KW 14         | Monday      | Holiday  | —     |
| Heiligabend      | KW 52         | Wednesday   | ShortDay | 12:00 |

All data SHALL be created using `Authentication::Full` to bypass permission checks.

#### Scenario: Seed on empty database
- **WHEN** `POST /dev/seed` is called on an empty database
- **THEN** the system creates 5 sales persons, their work contracts, extra hours, bookings for the current week, and special days
- **THEN** the response status is 200 with a confirmation message

#### Scenario: Seed on database with existing data
- **WHEN** `POST /dev/seed` is called on a database that already contains data
- **THEN** the system creates additional data without deleting existing records
- **THEN** no errors occur due to ID conflicts (UUIDs are generated fresh)

#### Scenario: Endpoint not available in production
- **WHEN** the application is compiled without the `mock_auth` feature flag
- **THEN** the `/dev/seed` endpoint does not exist and returns 404

### Requirement: Clear all data endpoint
The system SHALL provide a `POST /dev/clear` endpoint that deletes all data from the database. The endpoint SHALL only be compiled when the `mock_auth` feature flag is enabled. The endpoint SHALL be documented in OpenAPI/Swagger.

The clear operation SHALL delete data from all relevant tables in an order that respects foreign key constraints.

#### Scenario: Clear all data
- **WHEN** `POST /dev/clear` is called
- **THEN** all records are deleted from booking, sales_person_user, sales_person_unavailable, extra_hours, employee_work_details, sales_person, special_day, and slot tables
- **THEN** the response status is 200 with a confirmation message

#### Scenario: Clear on empty database
- **WHEN** `POST /dev/clear` is called on an already empty database
- **THEN** the operation succeeds without errors
- **THEN** the response status is 200

#### Scenario: Endpoint not available in production
- **WHEN** the application is compiled without the `mock_auth` feature flag
- **THEN** the `/dev/clear` endpoint does not exist and returns 404

### Requirement: Seeded sales persons have diverse states
The seeded sales persons SHALL match the exact data specified in the "Seed test data endpoint" requirement tables. Specifically:
- Anna Müller: active, paid, #FF6B6B
- Max Schmidt: active, paid, #4ECDC4
- Lisa Weber: inactive, paid, #45B7D1
- Tom Bauer: active, unpaid (volunteer), #96CEB4
- Sarah Fischer: active, paid, #FFEAA7

#### Scenario: Sales person variety
- **WHEN** `POST /dev/seed` is called
- **THEN** exactly 5 sales persons are created with the names, states, and colors specified above

### Requirement: Seeded work contracts match sales person profiles
Each seeded sales person SHALL have an employee work details record matching the contract table:
- Anna Müller: 40h/week, Mo-Fr, 5 workdays, 30 vacation days
- Max Schmidt: 20h/week, Mo-Mi, 3 workdays, 15 vacation days
- Lisa Weber: 30h/week, Mo-Do, 4 workdays, 24 vacation days
- Tom Bauer: 10h/week, Sa-So, 2 workdays, 0 vacation days
- Sarah Fischer: 35h/week, Mo-Fr, 5 workdays, 28 vacation days

#### Scenario: Work contract creation
- **WHEN** `POST /dev/seed` is called
- **THEN** each created sales person has exactly one employee work details record
- **THEN** work contracts match the hours, workdays, and vacation days specified above

### Requirement: Seeded extra hours match defined scenarios
Extra hours SHALL be created as specified:
- Anna Müller: 8h Vacation on Monday of the current week
- Max Schmidt: 8h SickLeave on Tuesday of the current week
- Sarah Fischer: 2h ExtraWork on Wednesday of the current week

#### Scenario: Extra hours creation
- **WHEN** `POST /dev/seed` is called
- **THEN** exactly 3 extra hours entries are created matching the persons, categories, amounts, and dates above

### Requirement: Seeded bookings use current calendar week
Bookings SHALL be created for the current calendar week on existing default slots:
- Anna Müller: Tue, Wed, Thu, Fri (Monday skipped due to vacation)
- Max Schmidt: Mon only (Tuesday skipped due to sick leave)
- Tom Bauer: Sat (weekend volunteer)
- Sarah Fischer: Mon, Tue, Wed, Thu, Fri (full week)
- Lisa Weber: no bookings (inactive)

#### Scenario: Bookings for current week
- **WHEN** `POST /dev/seed` is called
- **THEN** bookings are created for active sales persons on slots in the current calendar week
- **THEN** booking days match the schedule above, respecting absences

### Requirement: Seeded special days
Special days SHALL be created as specified:
- Karfreitag: KW 14, Friday, Holiday
- Ostermontag: KW 14, Monday, Holiday
- Heiligabend: KW 52, Wednesday, ShortDay at 12:00

#### Scenario: Special days creation
- **WHEN** `POST /dev/seed` is called
- **THEN** exactly 3 special days are created matching the dates, types, and times above

