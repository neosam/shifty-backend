# Capability: Sales Person Shiftplan Assignment

## Purpose
Allow administrators to view and manage which shiftplans a sales person is assigned to, using a checkbox-based UI on the SalesPersonDetails page.

## Requirements

### Requirement: Admin can view shiftplan assignments for a sales person

The SalesPersonDetails page SHALL display a "Shiftplan Assignments" section showing all available shiftplans as checkboxes. Each checkbox SHALL indicate whether the sales person is assigned to that shiftplan. When a shiftplan is assigned, the UI SHALL display the current permission level (`available` or `planner_only`) next to the checkbox. The section SHALL appear for both new and existing sales persons.

#### Scenario: Existing sales person with assignments
- **WHEN** admin opens SalesPersonDetails for a sales person with shiftplan assignments
- **THEN** the system loads the shiftplan catalog and the person's current assignments (including permission levels), and displays checkboxes with assigned plans checked and their permission level shown in a dropdown

#### Scenario: Existing sales person with planner_only assignment
- **WHEN** admin opens SalesPersonDetails for a sales person who has a `planner_only` assignment
- **THEN** the corresponding checkbox is checked and the dropdown shows "Planner Only"

#### Scenario: Existing sales person with no assignments
- **WHEN** admin opens SalesPersonDetails for a sales person with no assignments
- **THEN** all checkboxes are unchecked, no permission level dropdowns are shown, and an info message explains that the person is eligible for all shiftplans

#### Scenario: New sales person
- **WHEN** admin creates a new sales person
- **THEN** the shiftplan assignment section is visible with all checkboxes unchecked

### Requirement: Admin can modify shiftplan assignments

The admin SHALL be able to check and uncheck shiftplan checkboxes to modify assignments. When checking a shiftplan, the assignment SHALL default to `available` permission level. The admin SHALL be able to change the permission level via a dropdown. Changes SHALL be held in local state until the Save button is clicked.

#### Scenario: Toggle assignment on
- **WHEN** admin checks a shiftplan checkbox
- **THEN** the checkbox state updates locally with permission level `available`, and a permission level dropdown appears

#### Scenario: Toggle assignment off
- **WHEN** admin unchecks a shiftplan checkbox
- **THEN** the checkbox state updates locally and the permission level dropdown disappears

#### Scenario: Change permission level
- **WHEN** admin changes the permission level dropdown for an assigned shiftplan
- **THEN** the permission level updates locally but no API call is made

### Requirement: Assignments are saved with the Save button

When the admin clicks the Save button, the system SHALL save both the sales person data and the shiftplan assignments including permission levels. For new sales persons, the system SHALL first create the sales person, then save assignments using the new ID.

#### Scenario: Save existing sales person with changed assignments
- **WHEN** admin modifies shiftplan assignments and clicks Save
- **THEN** the system saves the sales person data via PUT and then saves assignments via PUT to `/sales-person-shiftplan/{id}/shiftplans` with `Vec<ShiftplanAssignmentTO>` body

#### Scenario: Save new sales person with assignments
- **WHEN** admin creates a new sales person with shiftplan assignments selected and clicks Save
- **THEN** the system creates the sales person via POST, obtains the new ID, and then saves assignments via PUT to `/sales-person-shiftplan/{id}/shiftplans` with `Vec<ShiftplanAssignmentTO>` body

#### Scenario: Save with all assignments unchecked
- **WHEN** admin unchecks all shiftplan assignments and clicks Save
- **THEN** the system saves an empty array to `/sales-person-shiftplan/{id}/shiftplans`, making the person eligible for all plans

### Requirement: Permissive model info message

The UI SHALL display an informational message explaining the permissive model: when no shiftplans are assigned, the person is eligible for all shiftplans.

#### Scenario: Info message displayed
- **WHEN** the shiftplan assignment section is rendered
- **THEN** an info text is shown explaining that no selection means eligible everywhere

### Requirement: i18n support for assignment UI

All user-visible text in the shiftplan assignment section SHALL be translatable in English, German, and Czech. This includes permission level labels.

#### Scenario: German locale
- **WHEN** admin views the page with German locale
- **THEN** all labels and messages in the shiftplan assignment section are displayed in German

#### Scenario: English locale
- **WHEN** admin views the page with English locale
- **THEN** all labels and messages in the shiftplan assignment section are displayed in English

#### Scenario: Permission level labels in German
- **WHEN** admin views the page with German locale
- **THEN** permission levels are displayed as "Verfuegbar" and "Nur Planer"

#### Scenario: Permission level labels in English
- **WHEN** admin views the page with English locale
- **THEN** permission levels are displayed as "Available" and "Planner Only"

### Requirement: Permission level dropdown for assigned shiftplans

For each assigned shiftplan, the UI SHALL display a dropdown select element with two options: "Available" and "Planner Only". The dropdown SHALL only be visible when the shiftplan checkbox is checked. The dropdown SHALL default to "Available" for newly checked assignments.

#### Scenario: Dropdown appears on check
- **WHEN** admin checks a previously unchecked shiftplan
- **THEN** a dropdown appears next to the checkbox with "Available" selected

#### Scenario: Dropdown disappears on uncheck
- **WHEN** admin unchecks a previously checked shiftplan
- **THEN** the dropdown next to the checkbox disappears
