# cross-plan-slot-query Specification

## Purpose
TBD - created by archiving change multi-shiftplan-aggregation. Update Purpose after archive.
## Requirements
### Requirement: Query slots across all non-planning plans
The system SHALL provide a method to retrieve all slots for a given week that belong to shift plans where `is_planning = false`. Slots belonging to planning-only calendars SHALL be excluded.

#### Scenario: Get all operational slots for a week
- **WHEN** a service requests all non-planning slots for year 2024, week 3
- **THEN** the system SHALL return slots from all shift plans where `is_planning = false`
- **AND** slots from shift plans where `is_planning = true` SHALL NOT be included

#### Scenario: Slots without a plan are included
- **WHEN** a slot has `shiftplan_id = NULL` (legacy data)
- **THEN** the slot SHALL be included in the result (backward compatibility)

### Requirement: BlockService uses cross-plan slot query
The BlockService SHALL use the cross-plan slot query to generate blocks from all operational shift plans, not from a single plan.

#### Scenario: Blocks generated from multiple plans
- **WHEN** an employee has bookings in the "main" plan and the "cleaning" plan (both non-planning)
- **THEN** the block generation SHALL include slots from both plans

#### Scenario: Blocks exclude planning calendars
- **WHEN** slots exist in a planning-only calendar
- **THEN** the block generation SHALL NOT include those slots

### Requirement: BookingInformationService uses cross-plan slot query
The BookingInformationService SHALL use the cross-plan slot query for calculating expected hours and volunteer summaries.

#### Scenario: Expected hours calculated from all operational plans
- **WHEN** the system calculates expected hours for a week
- **THEN** it SHALL include slots from all non-planning shift plans

### Requirement: Slot creation requires shiftplan_id
The `SlotService::create_slot` method SHALL validate that `shiftplan_id` is set. Creating a slot without a plan association SHALL return a validation error.

#### Scenario: Create slot without shiftplan_id
- **WHEN** a user creates a slot with `shiftplan_id = None`
- **THEN** the system SHALL return a validation error

#### Scenario: Create slot with shiftplan_id
- **WHEN** a user creates a slot with a valid `shiftplan_id`
- **THEN** the slot SHALL be created successfully

### Requirement: No Uuid::nil() placeholders remain
All usages of `Uuid::nil()` as a placeholder for `shiftplan_id` in production code SHALL be removed and replaced with proper implementations.

#### Scenario: No nil UUID in slot queries
- **WHEN** any service queries slots
- **THEN** it SHALL use either a specific `shiftplan_id` or the cross-plan query method
- **AND** `Uuid::nil()` SHALL NOT be used as a shiftplan_id placeholder

