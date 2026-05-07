# Shiftplan CRUD

## Purpose
Allow users to create, rename, and delete shiftplans through the tab bar UI when planning mode is active, with corresponding API functions for shiftplan mutations.

## Requirements

### Requirement: Create shiftplan via modal
The system SHALL display a `+` button at the end of the tab bar when planning mode is active. Clicking it SHALL open a modal dialog with a name input field and a submit button.

#### Scenario: Open create modal
- **WHEN** planning mode is active and the user clicks the `+` button
- **THEN** a modal dialog opens with a text input for the shiftplan name and a submit button

#### Scenario: Submit new shiftplan
- **WHEN** the user enters a name and submits the modal
- **THEN** a `POST /shiftplan-catalog` request is sent, the catalog reloads, and the new shiftplan is selected

#### Scenario: Cancel creation
- **WHEN** the user cancels the modal
- **THEN** the modal closes and no shiftplan is created

#### Scenario: Planning mode inactive
- **WHEN** planning mode is not active
- **THEN** the `+` button is not visible

### Requirement: Delete shiftplan with confirmation
The system SHALL display an `✕` button on each tab when planning mode is active. Clicking it SHALL show a confirmation dialog before deleting.

#### Scenario: Confirm delete
- **WHEN** the user clicks `✕` on a tab and confirms the deletion
- **THEN** a `DELETE /shiftplan-catalog/{id}` request is sent, the catalog reloads, and the first remaining shiftplan is selected

#### Scenario: Cancel delete
- **WHEN** the user clicks `✕` on a tab and cancels
- **THEN** the shiftplan is not deleted

#### Scenario: Delete currently selected shiftplan
- **WHEN** the deleted shiftplan was the currently selected one
- **THEN** the first remaining shiftplan is auto-selected after reload

#### Scenario: Planning mode inactive
- **WHEN** planning mode is not active
- **THEN** no `✕` buttons are visible

### Requirement: Rename shiftplan via inline edit
The system SHALL allow renaming a shiftplan by double-clicking its tab name when planning mode is active. The tab name SHALL turn into a text input field.

#### Scenario: Start inline rename
- **WHEN** planning mode is active and the user double-clicks a tab name
- **THEN** the tab name turns into an editable text input pre-filled with the current name

#### Scenario: Confirm rename
- **WHEN** the user presses Enter or the input loses focus
- **THEN** a `PUT /shiftplan-catalog/{id}` request is sent with the new name and the catalog reloads

#### Scenario: Cancel rename
- **WHEN** the user presses Escape
- **THEN** the input reverts to the original name without sending a request

#### Scenario: Planning mode inactive
- **WHEN** planning mode is not active
- **THEN** double-clicking a tab does nothing special (normal selection behavior)

### Requirement: API functions for shiftplan mutations
The system SHALL provide API functions to create, update, and delete shiftplans.

#### Scenario: Create shiftplan
- **WHEN** `create_shiftplan(config, shiftplan)` is called
- **THEN** `POST /shiftplan-catalog` is sent and the created `ShiftplanTO` is returned

#### Scenario: Update shiftplan
- **WHEN** `update_shiftplan(config, shiftplan)` is called
- **THEN** `PUT /shiftplan-catalog/{id}` is sent and the updated `ShiftplanTO` is returned

#### Scenario: Delete shiftplan
- **WHEN** `delete_shiftplan(config, id)` is called
- **THEN** `DELETE /shiftplan-catalog/{id}` is sent
