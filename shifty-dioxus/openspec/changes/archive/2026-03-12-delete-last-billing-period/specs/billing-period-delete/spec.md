## ADDED Requirements

### Requirement: Delete button on last billing period card
The system SHALL display a delete button on the last (first in list) billing period card on the employees page. The button SHALL only be visible to users with the "hr" privilege.

#### Scenario: HR user sees delete button on last billing period
- **WHEN** a user with "hr" privilege views the employees page with at least one billing period
- **THEN** a delete button is shown on the first billing period card only

#### Scenario: Non-HR user does not see delete button
- **WHEN** a user without "hr" privilege views the employees page
- **THEN** no delete button is shown on any billing period card

#### Scenario: Delete button does not navigate to detail page
- **WHEN** an HR user clicks the delete button on a billing period card
- **THEN** the click does not navigate to the billing period detail page

### Requirement: Confirmation dialog before deletion
The system SHALL show a confirmation dialog when the delete button is clicked. The dialog SHALL display the billing period date range and offer Cancel and Delete actions.

#### Scenario: Confirmation dialog appears on delete click
- **WHEN** an HR user clicks the delete button on the last billing period
- **THEN** a modal dialog appears asking for confirmation with the period's date range displayed

#### Scenario: User cancels deletion
- **WHEN** the confirmation dialog is shown and the user clicks Cancel
- **THEN** the dialog closes and no deletion occurs

### Requirement: Successful deletion removes billing period from list
The system SHALL call `DELETE /billing-period/{id}` when the user confirms deletion. On success (HTTP 204), the system SHALL close the dialog and remove the billing period from the displayed list.

#### Scenario: Successful deletion
- **WHEN** the user confirms deletion and the API returns 204
- **THEN** the dialog closes and the billing period disappears from the list

### Requirement: Error display in confirmation dialog
The system SHALL display API error messages inside the confirmation dialog without closing it.

#### Scenario: API returns 409 conflict
- **WHEN** the user confirms deletion and the API returns 409 (not the latest billing period)
- **THEN** the dialog remains open and displays an error message

#### Scenario: API returns 403 forbidden
- **WHEN** the user confirms deletion and the API returns 403
- **THEN** the dialog remains open and displays an error message

### Requirement: API function for billing period deletion
The system SHALL provide a `delete_billing_period` function in `api.rs` that sends a DELETE request to `/billing-period/{id}`.

#### Scenario: API function sends correct request
- **WHEN** `delete_billing_period` is called with a billing period ID
- **THEN** a DELETE request is sent to `/billing-period/{id}` with authentication headers

### Requirement: Internationalized text for delete UI
The system SHALL provide translated text for the delete button, confirmation message, and error messages in English, German, and Czech.

#### Scenario: Delete UI text in German
- **WHEN** the locale is set to German
- **THEN** all delete-related UI text (button, confirmation, errors) is displayed in German

#### Scenario: Delete UI text in English
- **WHEN** the locale is set to English
- **THEN** all delete-related UI text is displayed in English

#### Scenario: Delete UI text in Czech
- **WHEN** the locale is set to Czech
- **THEN** all delete-related UI text is displayed in Czech
