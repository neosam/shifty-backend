## MODIFIED Requirements

### Requirement: API functions for sales person shiftplan endpoints

The frontend SHALL provide API functions for the sales person shiftplan backend endpoints.

#### Scenario: Get shiftplan assignments for a sales person
- **WHEN** `get_shiftplan_assignments(sales_person_id)` is called
- **THEN** it performs GET to `/sales-person-shiftplan/{id}/shiftplans` and returns `Vec<ShiftplanAssignment>` containing both `shiftplan_id` and `permission_level` for each assignment

#### Scenario: Set shiftplan assignments for a sales person
- **WHEN** `set_shiftplan_assignments(sales_person_id, assignments)` is called
- **THEN** it performs PUT to `/sales-person-shiftplan/{id}/shiftplans` with `Vec<ShiftplanAssignmentTO>` body containing `shiftplan_id` and `permission_level` for each assignment

#### Scenario: Get bookable sales persons for a shiftplan
- **WHEN** `get_bookable_sales_persons(shiftplan_id)` is called
- **THEN** it performs GET to `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}` and returns `Vec<SalesPersonTO>` (unchanged)
