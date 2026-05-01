# extra-hours-update Specification

## Purpose
TBD - created by archiving change extra-hours-update. Update Purpose after archive.

## Requirements

### Requirement: Stable logical identity for extra hours entries

Every extra hours entry SHALL have a stable logical identity that survives across edits. The system SHALL store this as a `logical_id` column on `extra_hours`. The first row written for a logical entry SHALL have `logical_id` equal to its physical row `id`. Subsequent rows that represent edits of the same logical entry SHALL share that `logical_id`. The REST DTO `id` field SHALL carry the `logical_id`; the physical row `id` SHALL NOT be exposed externally.

#### Scenario: First create assigns matching logical_id and id

- **WHEN** a new extra hours entry is created via the service
- **THEN** the persisted row has `id == logical_id`, both freshly minted UUIDs
- **AND** the response DTO's `id` field equals that `logical_id`

#### Scenario: Existing rows backfilled by migration

- **WHEN** the migration that introduces `logical_id` runs against an existing database
- **THEN** every existing row has `logical_id` set equal to its existing `id`
- **AND** subsequent GET requests using a previously-known id continue to return the same entry

### Requirement: Active row uniqueness per logical_id

For any given `logical_id`, the system SHALL guarantee that at most one row has `deleted IS NULL` at any time. This SHALL be enforced at the database level by a partial unique index on `(logical_id) WHERE deleted IS NULL`.

#### Scenario: Read by logical_id returns the unique active row

- **WHEN** the service looks up an entry by its external id (the `logical_id`)
- **THEN** it returns the single row matching `logical_id = ? AND deleted IS NULL`

#### Scenario: Database rejects two active rows with the same logical_id

- **WHEN** an attempt is made to insert a second active row sharing an existing active row's `logical_id`
- **THEN** the database rejects the insert via the partial unique index

### Requirement: Update preserves history via soft-delete-and-insert

The service SHALL implement update by soft-deleting the current active row (setting its `deleted` to the current timestamp) and inserting a new row that carries the same `logical_id`, a freshly-minted physical `id`, a freshly-minted `version`, and `created` set to the current timestamp. Both writes SHALL occur in a single transaction; if either write fails, neither SHALL be committed.

#### Scenario: Successful update produces tombstone plus new active row

- **WHEN** an authorized caller updates an extra hours entry with valid changes
- **THEN** the previously active row is now a tombstone (`deleted` set to the current timestamp, all other fields unchanged)
- **AND** a new row exists with the same `logical_id`, a new physical `id`, a new `version`, `created` set to the current timestamp, and `deleted IS NULL`
- **AND** the response DTO returns `id` equal to the unchanged `logical_id` and `$version` equal to the new `version`

#### Scenario: Insert failure aborts the soft-delete

- **WHEN** the soft-delete of the current row succeeds but the insert of the replacement row fails
- **THEN** the transaction is rolled back
- **AND** the previously active row remains active (no tombstone), and no replacement row exists

### Requirement: Optimistic locking on update

The service SHALL require the caller to supply the `version` of the row they last read. The service SHALL compare this against the active row's current `version` and SHALL reject the update with a conflict error when they differ. On match, the service SHALL proceed and rotate `version` for the new row.

#### Scenario: Update with current version succeeds

- **WHEN** the request body's `$version` equals the active row's `version`
- **THEN** the update proceeds, the new row is written with a new `version`, and the response carries that new `version`

#### Scenario: Update with stale version is rejected

- **WHEN** the request body's `$version` does not equal the active row's `version`
- **THEN** the service returns a conflict error
- **AND** the REST layer maps that error to `409 Conflict`
- **AND** no soft-delete and no insert have occurred

### Requirement: Editable and immutable fields on update

On update, the service SHALL accept changes to `amount`, `category`, `description`, `date_time`, and `custom_extra_hours_id`. The service SHALL reject the update if the request changes `sales_person_id`, `logical_id`, the physical `id`, `created`, or `version` from values supplied by the server.

#### Scenario: Editable fields are persisted to the new row

- **WHEN** an update changes `amount`, `category`, `description`, `date_time`, or `custom_extra_hours_id`
- **THEN** the new row carries the supplied values
- **AND** the tombstone retains the previous values

#### Scenario: Update changing sales_person_id is rejected

- **WHEN** the request body's `sales_person_id` differs from the active row's `sales_person_id`
- **THEN** the service returns an immutable-field error and performs no writes

### Requirement: Permission model for update

The service SHALL allow the update if the caller has `HR_PRIVILEGE` OR the caller is authenticated as the `sales_person_id` of the active row. Otherwise the service SHALL reject the update with a forbidden error.

#### Scenario: HR can update any user's entry

- **WHEN** a caller with `HR_PRIVILEGE` updates an entry belonging to any sales person
- **THEN** the update proceeds

#### Scenario: A user can update their own entry

- **WHEN** a caller authenticated as sales person `S` updates an entry whose active row has `sales_person_id == S`
- **THEN** the update proceeds

#### Scenario: A user cannot update someone else's entry

- **WHEN** a caller without `HR_PRIVILEGE`, authenticated as sales person `S`, attempts to update an entry whose active row has `sales_person_id != S`
- **THEN** the service returns a forbidden error and performs no writes

### Requirement: Update on missing or already-deleted entries

The service SHALL return a not-found error when no active row exists for the supplied `logical_id` (either the id was never created, or the entry has already been deleted).

#### Scenario: Update of a never-created id

- **WHEN** the request targets a `logical_id` that does not exist in `extra_hours`
- **THEN** the service returns a not-found error and performs no writes

#### Scenario: Update of a soft-deleted entry

- **WHEN** the request targets a `logical_id` whose every row is soft-deleted (no row with `deleted IS NULL`)
- **THEN** the service returns a not-found error and performs no writes

### Requirement: REST contract for PUT /extra-hours/{id}

The system SHALL expose `PUT /extra-hours/{id}` where `{id}` is the `logical_id` of the entry. The endpoint SHALL accept an `ExtraHoursTO` body and return the updated entry as `ExtraHoursTO` on success. The endpoint SHALL be annotated with `#[utoipa::path]` and SHALL document `200 OK`, `400 Bad Request`, `403 Forbidden`, `404 Not Found`, and `409 Conflict` responses.

#### Scenario: Successful update responds with new state

- **WHEN** a valid `PUT /extra-hours/{id}` request is processed
- **THEN** the response status is `200 OK`
- **AND** the response body is the updated `ExtraHoursTO` with `id` equal to the request path's `id` and `$version` equal to the new server-assigned version

#### Scenario: Stale version returns 409

- **WHEN** a `PUT /extra-hours/{id}` request carries a `$version` that does not match the server's current `version`
- **THEN** the response status is `409 Conflict`

#### Scenario: Unknown id returns 404

- **WHEN** a `PUT /extra-hours/{id}` request targets a `logical_id` with no active row
- **THEN** the response status is `404 Not Found`

### Requirement: Snapshot drift is permitted on update

The service SHALL NOT block updates to entries that fall inside an already-snapshotted billing period. Snapshot drift between persisted snapshots and the post-update live computation SHALL remain detectable by existing snapshot validators.

#### Scenario: Update inside a billed period proceeds

- **WHEN** an authorized caller updates an entry whose `date_time` falls inside a previously-snapshotted billing period
- **THEN** the update proceeds successfully
- **AND** persisted snapshots for that period are not modified
