### Requirement: Delete latest billing period by ID
The system SHALL allow authorized users to soft-delete a billing period by its ID, provided it is the most recent (latest by start date) non-deleted billing period. The operation SHALL require HR_PRIVILEGE permission.

#### Scenario: Successful deletion of the latest billing period
- **WHEN** a user with HR_PRIVILEGE sends DELETE /billing-periods/{id} where {id} is the latest billing period
- **THEN** the system soft-deletes the billing period (sets deleted timestamp and deleted_by) and returns 204 No Content

#### Scenario: Successful cascade deletion of sales person entries
- **WHEN** a billing period is soft-deleted
- **THEN** all associated billing_period_sales_person records for that billing period SHALL also be soft-deleted

#### Scenario: Deletion rejected for non-latest billing period
- **WHEN** a user with HR_PRIVILEGE sends DELETE /billing-periods/{id} where {id} is NOT the latest billing period
- **THEN** the system SHALL return 409 Conflict with error message indicating the billing period is not the latest

#### Scenario: Deletion rejected for non-existent billing period
- **WHEN** a user sends DELETE /billing-periods/{id} where {id} does not exist or is already deleted
- **THEN** the system SHALL return 404 Not Found

#### Scenario: Deletion rejected without HR privilege
- **WHEN** a user without HR_PRIVILEGE sends DELETE /billing-periods/{id}
- **THEN** the system SHALL return 403 Forbidden

### Requirement: Next billing period creation ignores soft-deleted periods
The system SHALL ensure that when creating a new billing period after a soft-delete, all previously soft-deleted billing periods are ignored when determining the start date of the new period.

#### Scenario: New billing period created after soft-delete uses correct start date
- **WHEN** the latest billing period has been soft-deleted and a new billing period is created
- **THEN** the new billing period's start date SHALL be determined by the latest non-deleted billing period's end date, ignoring the soft-deleted period
