## ADDED Requirements

### Requirement: Sales person shiftplan assignment storage
The system SHALL maintain an N:M relationship between sales persons and shift plans in a `sales_person_shiftplan` join table with composite primary key `(sales_person_id, shiftplan_id)`.

#### Scenario: Empty assignment state
- **WHEN** a sales person has no entries in the assignment table
- **THEN** the sales person is considered eligible for all shift plans (permissive model)

#### Scenario: Explicit assignments exist
- **WHEN** a sales person has one or more entries in the assignment table
- **THEN** the sales person is eligible only for the shift plans listed in those entries

### Requirement: Set assignments for a sales person
The system SHALL provide a `PUT /api/sales-person/{id}/shiftplans` endpoint that accepts a JSON array of shiftplan UUIDs and atomically replaces all assignments for that sales person. The endpoint SHALL require the `shiftplanner` privilege.

#### Scenario: Set assignments successfully
- **WHEN** an authenticated user with `shiftplanner` privilege sends `PUT /api/sales-person/{sp-id}/shiftplans` with body `["plan-id-1", "plan-id-2"]`
- **THEN** the system deletes all existing assignments for `sp-id` and creates entries for `plan-id-1` and `plan-id-2`
- **THEN** the system returns HTTP 200

#### Scenario: Clear all assignments
- **WHEN** an authenticated user with `shiftplanner` privilege sends `PUT /api/sales-person/{sp-id}/shiftplans` with body `[]`
- **THEN** the system deletes all existing assignments for `sp-id`
- **THEN** the sales person becomes eligible for all shift plans again (permissive model)

#### Scenario: Insufficient privilege
- **WHEN** a user without `shiftplanner` privilege sends `PUT /api/sales-person/{sp-id}/shiftplans`
- **THEN** the system returns HTTP 403 Forbidden

#### Scenario: Sales person does not exist
- **WHEN** a user sends `PUT /api/sales-person/{unknown-id}/shiftplans`
- **THEN** the system returns HTTP 404 Not Found

### Requirement: Get assignments for a sales person
The system SHALL provide a `GET /api/sales-person/{id}/shiftplans` endpoint that returns the list of shiftplan UUIDs explicitly assigned to that sales person. The endpoint SHALL require the `shiftplanner` privilege.

#### Scenario: Sales person has assignments
- **WHEN** an authenticated user with `shiftplanner` privilege requests `GET /api/sales-person/{sp-id}/shiftplans`
- **AND** the sales person has assignments to `plan-id-1` and `plan-id-2`
- **THEN** the system returns HTTP 200 with body `["plan-id-1", "plan-id-2"]`

#### Scenario: Sales person has no assignments
- **WHEN** an authenticated user with `shiftplanner` privilege requests `GET /api/sales-person/{sp-id}/shiftplans`
- **AND** the sales person has no assignments
- **THEN** the system returns HTTP 200 with body `[]`

### Requirement: Get bookable sales persons for a shiftplan
The system SHALL provide a `GET /api/sales-person/by-shiftplan/{shiftplan_id}` endpoint that returns all sales persons eligible to be booked in the given shift plan. Eligibility follows the permissive model.

#### Scenario: No sales persons have any assignments
- **WHEN** no sales person has any assignment entries
- **THEN** the endpoint returns all active (non-deleted, non-inactive) sales persons

#### Scenario: Mixed assignment state
- **WHEN** sales person A has no assignments, sales person B is assigned to plan X, and sales person C is assigned to plan Y
- **AND** the request is for plan X
- **THEN** the endpoint returns sales person A (no assignments = eligible everywhere) and sales person B (explicitly assigned)
- **THEN** sales person C is NOT returned (has assignments but not to plan X)

### Requirement: Booking eligibility enforcement
The system SHALL reject booking creation when the sales person is not eligible for the slot's shift plan. Eligibility follows the permissive model. Existing bookings SHALL NOT be affected.

#### Scenario: Sales person with no assignments books any plan
- **WHEN** a sales person with no assignments creates a booking for a slot in any shift plan
- **THEN** the booking is created successfully

#### Scenario: Sales person assigned to the plan books that plan
- **WHEN** a sales person assigned to plan X creates a booking for a slot in plan X
- **THEN** the booking is created successfully

#### Scenario: Sales person assigned to other plans books a non-assigned plan
- **WHEN** a sales person assigned to plan X (but not plan Y) creates a booking for a slot in plan Y
- **THEN** the system rejects the booking with an appropriate error

#### Scenario: Existing booking remains after assignment change
- **WHEN** a sales person has an existing booking in plan Y
- **AND** an administrator removes the sales person's assignment to plan Y
- **THEN** the existing booking remains unchanged
