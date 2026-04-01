## ADDED Requirements

### Requirement: Admin can view shiftplan assignments for a sales person

The SalesPersonDetails page SHALL display a "Shiftplan Assignments" section showing all available shiftplans as checkboxes. Each checkbox SHALL indicate whether the sales person is assigned to that shiftplan. The section SHALL appear for both new and existing sales persons.

#### Scenario: Existing sales person with assignments
- **WHEN** admin opens SalesPersonDetails for a sales person with shiftplan assignments
- **THEN** the system loads the shiftplan catalog and the person's current assignments, and displays checkboxes with assigned plans checked

#### Scenario: Existing sales person with no assignments
- **WHEN** admin opens SalesPersonDetails for a sales person with no assignments
- **THEN** all checkboxes are unchecked and an info message explains that the person is eligible for all shiftplans

#### Scenario: New sales person
- **WHEN** admin creates a new sales person
- **THEN** the shiftplan assignment section is visible with all checkboxes unchecked

### Requirement: Admin can modify shiftplan assignments

The admin SHALL be able to check and uncheck shiftplan checkboxes to modify assignments. Changes SHALL be held in local state until the Save button is clicked.

#### Scenario: Toggle assignment on
- **WHEN** admin checks a shiftplan checkbox
- **THEN** the checkbox state updates locally but no API call is made

#### Scenario: Toggle assignment off
- **WHEN** admin unchecks a shiftplan checkbox
- **THEN** the checkbox state updates locally but no API call is made

### Requirement: Assignments are saved with the Save button

When the admin clicks the Save button, the system SHALL save both the sales person data and the shiftplan assignments. For new sales persons, the system SHALL first create the sales person, then save assignments using the new ID.

#### Scenario: Save existing sales person with changed assignments
- **WHEN** admin modifies shiftplan assignments and clicks Save
- **THEN** the system saves the sales person data via PUT and then saves assignments via PUT to `/sales-person-shiftplan/{id}/shiftplans`

#### Scenario: Save new sales person with assignments
- **WHEN** admin creates a new sales person with shiftplan assignments selected and clicks Save
- **THEN** the system creates the sales person via POST, obtains the new ID, and then saves assignments via PUT to `/sales-person-shiftplan/{id}/shiftplans`

#### Scenario: Save with all assignments unchecked
- **WHEN** admin unchecks all shiftplan assignments and clicks Save
- **THEN** the system saves an empty array to `/sales-person-shiftplan/{id}/shiftplans`, making the person eligible for all plans

### Requirement: Permissive model info message

The UI SHALL display an informational message explaining the permissive model: when no shiftplans are assigned, the person is eligible for all shiftplans.

#### Scenario: Info message displayed
- **WHEN** the shiftplan assignment section is rendered
- **THEN** an info text is shown explaining that no selection means eligible everywhere

### Requirement: i18n support for assignment UI

All user-visible text in the shiftplan assignment section SHALL be translatable in English, German, and Czech.

#### Scenario: German locale
- **WHEN** admin views the page with German locale
- **THEN** all labels and messages in the shiftplan assignment section are displayed in German

#### Scenario: English locale
- **WHEN** admin views the page with English locale
- **THEN** all labels and messages in the shiftplan assignment section are displayed in English
