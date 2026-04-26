# shiftplan-refresh Specification

## Purpose

Defines the automatic refresh behavior of the shiftplan week view in response to slot mutations (create, edit, delete) so users always see the current state without manual navigation.

## Requirements

### Requirement: Shiftplan view refreshes after slot creation
The system SHALL automatically reload and display updated shiftplan data after a new slot is successfully created.

#### Scenario: New slot appears immediately
- **WHEN** user creates a new slot via the slot edit dialog
- **THEN** the week view SHALL display the new slot without requiring manual navigation

### Requirement: Shiftplan view refreshes after slot edit
The system SHALL automatically reload and display updated shiftplan data after an existing slot is successfully edited.

#### Scenario: Edited slot updates immediately
- **WHEN** user modifies an existing slot's properties (time, resources) and saves
- **THEN** the week view SHALL display the updated slot data without requiring manual navigation

### Requirement: Shiftplan view refreshes after slot deletion
The system SHALL automatically reload and display updated shiftplan data after a slot is successfully deleted.

#### Scenario: Deleted slot disappears immediately
- **WHEN** user deletes an existing slot
- **THEN** the week view SHALL remove the deleted slot without requiring manual navigation

### Requirement: No refresh on cancel
The system SHALL NOT trigger a shiftplan refresh when the user cancels the slot edit dialog.

#### Scenario: Cancel does not reload
- **WHEN** user opens the slot edit dialog and clicks cancel
- **THEN** the week view SHALL NOT reload its data
