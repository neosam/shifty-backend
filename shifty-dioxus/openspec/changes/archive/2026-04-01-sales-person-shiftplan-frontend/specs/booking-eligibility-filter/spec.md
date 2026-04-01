## ADDED Requirements

### Requirement: Booking dropdown shows only eligible sales persons

The shiftplan booking view SHALL use the `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}` endpoint to load the list of bookable sales persons instead of showing all sales persons.

#### Scenario: Shiftplan with restricted assignments
- **WHEN** shiftplanner opens the booking dropdown for a shiftplan that has sales persons assigned to it
- **THEN** only the eligible sales persons (assigned + those with no assignments at all) are shown in the dropdown

#### Scenario: Shiftplan with no restrictions
- **WHEN** shiftplanner opens the booking dropdown for a shiftplan where no sales person has a specific assignment to it
- **THEN** all active sales persons are shown (permissive model fallback)

### Requirement: Forbidden booking error handling

When the backend rejects a booking with 403 Forbidden (due to ineligibility), the frontend SHALL display a meaningful error message to the user.

#### Scenario: Booking rejected as forbidden
- **WHEN** a booking creation request returns HTTP 403
- **THEN** the system displays a user-friendly error message indicating the sales person is not eligible for this shiftplan

### Requirement: API functions for sales person shiftplan endpoints

The frontend SHALL provide API functions for the three new backend endpoints.

#### Scenario: Get shiftplan assignments for a sales person
- **WHEN** `get_shiftplan_assignments(sales_person_id)` is called
- **THEN** it performs GET to `/sales-person-shiftplan/{id}/shiftplans` and returns `Vec<Uuid>`

#### Scenario: Set shiftplan assignments for a sales person
- **WHEN** `set_shiftplan_assignments(sales_person_id, shiftplan_ids)` is called
- **THEN** it performs PUT to `/sales-person-shiftplan/{id}/shiftplans` with the UUID array as body

#### Scenario: Get bookable sales persons for a shiftplan
- **WHEN** `get_bookable_sales_persons(shiftplan_id)` is called
- **THEN** it performs GET to `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}` and returns `Vec<SalesPersonTO>`
