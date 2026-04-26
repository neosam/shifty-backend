## ADDED Requirements

### Requirement: Create shiftplan via dialog
The system SHALL display a modal dialog when the user clicks the "+" button in structure mode. The dialog SHALL contain a name text field and an "is planning" checkbox. On submit, the system SHALL create a new shiftplan with the provided values.

#### Scenario: Create with default values
- **WHEN** user clicks "+" in structure mode
- **THEN** the dialog SHALL open with an empty name field and "is planning" unchecked

#### Scenario: Create with is_planning enabled
- **WHEN** user fills in a name, checks "is planning", and confirms
- **THEN** the system SHALL create a shiftplan with `is_planning: true`

#### Scenario: Cancel create
- **WHEN** user clicks "Abbrechen" or presses Escape in the create dialog
- **THEN** the dialog SHALL close without creating a shiftplan

### Requirement: Edit shiftplan via dialog
The system SHALL display a modal dialog when the user double-clicks a shiftplan tab in structure mode. The dialog SHALL pre-populate the name and "is planning" fields with the current values.

#### Scenario: Open edit dialog
- **WHEN** user double-clicks a shiftplan tab in structure mode
- **THEN** the dialog SHALL open with the shiftplan's current name and `is_planning` value

#### Scenario: Save edited properties
- **WHEN** user modifies name or is_planning and confirms
- **THEN** the system SHALL update the shiftplan via the API and refresh the catalog

#### Scenario: Cancel edit
- **WHEN** user clicks "Abbrechen" or presses Escape in the edit dialog
- **THEN** the dialog SHALL close without saving changes

### Requirement: Dialog only available in structure mode
The system SHALL NOT open the create or edit dialog when structure mode is inactive.

#### Scenario: Double-click outside structure mode
- **WHEN** user double-clicks a shiftplan tab while structure mode is inactive
- **THEN** no dialog SHALL open

#### Scenario: Plus button hidden outside structure mode
- **WHEN** structure mode is inactive
- **THEN** the "+" button SHALL NOT be visible

### Requirement: Shared modal for create and edit
The system SHALL use a single modal component for both create and edit operations, with the title and button label reflecting the current mode.

#### Scenario: Create mode title
- **WHEN** the dialog is in create mode
- **THEN** the title SHALL be "Neuen Shiftplan erstellen" and the confirm button SHALL say "Erstellen"

#### Scenario: Edit mode title
- **WHEN** the dialog is in edit mode
- **THEN** the title SHALL be "Shiftplan bearbeiten" and the confirm button SHALL say "Speichern"
