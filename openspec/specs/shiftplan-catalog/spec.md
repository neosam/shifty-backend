# shiftplan-catalog Specification

## Purpose
TBD - created by archiving change multi-shiftplan. Update Purpose after archive.
## Requirements
### Requirement: Shiftplan entity exists
The system SHALL store shift plans as entities with the following attributes: id (UUID), name (text), is_planning (boolean), deleted (optional timestamp), and version (UUID). Each shift plan represents a distinct scheduling context (e.g., main store, baking, cleaning).

#### Scenario: Default plan exists after migration
- **WHEN** the system is migrated from a single-plan installation
- **THEN** a shift plan named "main" with `is_planning = false` SHALL exist
- **AND** all existing slots SHALL be associated with this plan

### Requirement: Create shift plan
The system SHALL allow creating a new shift plan with a name and `is_planning` flag. The system SHALL assign a UUID and version automatically.

#### Scenario: Create a planning-only calendar
- **WHEN** a user with appropriate permissions creates a shift plan with name "Baking" and `is_planning = true`
- **THEN** the system SHALL create the shift plan and return it with a generated UUID

#### Scenario: Create a standard shift plan
- **WHEN** a user creates a shift plan with name "Cleaning" and `is_planning = false`
- **THEN** the system SHALL create the shift plan and return it
- **AND** bookings in this plan SHALL count toward hour calculations

### Requirement: List shift plans
The system SHALL provide a list of all non-deleted shift plans.

#### Scenario: List all active plans
- **WHEN** a user requests the list of shift plans
- **THEN** the system SHALL return all shift plans where `deleted` is NULL
- **AND** each entry SHALL include id, name, is_planning, and version

### Requirement: Get single shift plan
The system SHALL allow retrieving a single shift plan by its UUID.

#### Scenario: Get existing plan
- **WHEN** a user requests a shift plan by a valid UUID
- **THEN** the system SHALL return the shift plan details

#### Scenario: Get non-existing plan
- **WHEN** a user requests a shift plan by a UUID that does not exist or is deleted
- **THEN** the system SHALL return a not-found error

### Requirement: Update shift plan
The system SHALL allow updating a shift plan's name and `is_planning` flag. Updates SHALL use optimistic locking via the version field.

#### Scenario: Rename a plan
- **WHEN** a user updates a shift plan's name from "main" to "Store Schedule" with the correct version
- **THEN** the system SHALL update the name and assign a new version

#### Scenario: Version conflict
- **WHEN** a user updates a shift plan with an outdated version
- **THEN** the system SHALL reject the update with a conflict error

### Requirement: Delete shift plan
The system SHALL support soft-deleting a shift plan. Deleting a plan SHALL NOT delete its slots or bookings.

#### Scenario: Soft delete a plan
- **WHEN** a user deletes a shift plan
- **THEN** the plan's `deleted` field SHALL be set to the current timestamp
- **AND** the plan SHALL no longer appear in list results

### Requirement: Slots are scoped to a shift plan
Every slot SHALL be associated with exactly one shift plan via a `shiftplan_id` reference. The slot queries for weekly views SHALL be filtered by `shiftplan_id`.

#### Scenario: Query slots for a specific plan and week
- **WHEN** a user requests the shift plan view for a specific plan and week
- **THEN** the system SHALL return only slots belonging to that plan

#### Scenario: Create a slot for a plan
- **WHEN** a new slot is created
- **THEN** it SHALL be associated with a specific shift plan

### Requirement: Planning calendars excluded from reports
Shift plans with `is_planning = true` SHALL be excluded from all hour calculation and report queries. This ensures planning-only calendars (e.g., baking schedule) do not affect employee balance hours or billing periods.

#### Scenario: Hours from planning calendar not counted
- **WHEN** an employee has bookings in a plan with `is_planning = true`
- **AND** the system calculates the employee's worked hours for a period
- **THEN** bookings from the planning calendar SHALL NOT be included in the total

#### Scenario: Hours from standard calendar counted
- **WHEN** an employee has bookings in a plan with `is_planning = false`
- **AND** the system calculates the employee's worked hours
- **THEN** those bookings SHALL be included in the total

### Requirement: REST API for shift plan management
The system SHALL expose REST endpoints for shift plan CRUD operations with OpenAPI documentation.

#### Scenario: CRUD endpoints available
- **WHEN** the API is deployed
- **THEN** the following endpoints SHALL be available:
  - `GET /shiftplan` -- list all plans
  - `POST /shiftplan` -- create a plan
  - `GET /shiftplan/{id}` -- get a plan
  - `PUT /shiftplan/{id}` -- update a plan
  - `DELETE /shiftplan/{id}` -- delete a plan
  - `GET /shiftplan/{id}/{year}/{week}` -- get weekly view for a plan

### Requirement: Bookings view includes shiftplan context
The `bookings_view` database view SHALL be updated to include the shift plan name, enabling direct visibility of which plan a booking belongs to.

#### Scenario: View shows plan name
- **WHEN** querying the bookings_view
- **THEN** each row SHALL include the shift plan name from the associated slot's plan

