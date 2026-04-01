## ADDED Requirements

### Requirement: Permission level on shiftplan assignments
Each sales person to shiftplan assignment SHALL have a `permission_level` with values `available` or `planner_only`. The default for new and existing assignments SHALL be `available`.

#### Scenario: New assignment defaults to available
- **WHEN** a shiftplan assignment is created without specifying a permission level
- **THEN** the permission level SHALL be `available`

#### Scenario: Assignment with planner_only level
- **WHEN** a shiftplanner sets a sales person's shiftplan assignment with permission level `planner_only`
- **THEN** the assignment SHALL be stored with permission level `planner_only`

#### Scenario: Invalid permission level rejected
- **WHEN** an assignment is created with a permission level other than `available` or `planner_only`
- **THEN** the system SHALL reject the request with a validation error

### Requirement: Role-aware eligibility for booking creation
The eligibility check for booking a sales person into a shiftplan SHALL consider the caller's role.

#### Scenario: No assignments - always eligible
- **WHEN** a sales person has no shiftplan assignments
- **THEN** they SHALL be eligible for all shiftplans regardless of who is booking

#### Scenario: Available assignment - self-service eligible
- **WHEN** a sales person has an `available` assignment for a shiftplan
- **THEN** both the sales person themselves and a shiftplanner SHALL be able to create a booking

#### Scenario: Planner-only assignment - shiftplanner can book
- **WHEN** a sales person has a `planner_only` assignment for a shiftplan
- **AND** a shiftplanner creates the booking
- **THEN** the booking SHALL be created successfully

#### Scenario: Planner-only assignment - self-service denied
- **WHEN** a sales person has a `planner_only` assignment for a shiftplan
- **AND** the sales person tries to book themselves
- **THEN** the system SHALL deny the booking with a Forbidden error

#### Scenario: Not assigned - excluded for everyone
- **WHEN** a sales person has assignments to other shiftplans but not to a specific shiftplan
- **AND** anyone tries to book them into that shiftplan
- **THEN** the system SHALL deny the booking with a Forbidden error

### Requirement: Role-aware bookable sales persons list
The list of bookable sales persons for a shiftplan SHALL differ based on the caller's role.

#### Scenario: Shiftplanner sees all eligible persons
- **WHEN** a shiftplanner requests the bookable sales persons for a shiftplan
- **THEN** the list SHALL include persons with `available` assignments, persons with `planner_only` assignments, and persons with no assignments (permissive default)
- **AND** the list SHALL NOT include persons who have assignments to other plans but not this one
- **AND** the list SHALL NOT include inactive persons

#### Scenario: Non-shiftplanner sees only self-service eligible persons
- **WHEN** a non-shiftplanner requests the bookable sales persons for a shiftplan
- **THEN** the list SHALL include persons with `available` assignments and persons with no assignments
- **AND** the list SHALL NOT include persons with `planner_only` assignments for this plan
- **AND** the list SHALL NOT include inactive persons

### Requirement: Booking visibility for planner-only assignments
A sales person SHALL be able to see bookings made for them in `planner_only` shiftplans but SHALL NOT be able to modify them.

#### Scenario: Person sees planner-only booking
- **WHEN** a shiftplanner has booked a sales person into a `planner_only` shiftplan
- **AND** the sales person views their bookings
- **THEN** the booking SHALL be visible

#### Scenario: Person cannot delete planner-only booking
- **WHEN** a sales person tries to delete a booking in a shiftplan where they have a `planner_only` assignment
- **AND** the person is not a shiftplanner
- **THEN** the system SHALL deny the deletion with a Forbidden error

#### Scenario: Shiftplanner can delete planner-only booking
- **WHEN** a shiftplanner tries to delete a booking in a shiftplan where the person has a `planner_only` assignment
- **THEN** the deletion SHALL succeed

### Requirement: API accepts permission levels on assignment updates
The REST endpoint for setting shiftplan assignments SHALL accept permission levels alongside shiftplan IDs.

#### Scenario: Set assignments with permission levels
- **WHEN** a shiftplanner sends a PUT request with a list of shiftplan assignments including permission levels
- **THEN** the assignments SHALL be stored with the specified permission levels

#### Scenario: GET assignments returns permission levels
- **WHEN** a shiftplanner retrieves shiftplan assignments for a sales person
- **THEN** the response SHALL include the permission level for each assignment
