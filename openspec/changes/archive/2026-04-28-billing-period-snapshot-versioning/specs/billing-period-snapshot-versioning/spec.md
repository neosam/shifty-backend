## ADDED Requirements

### Requirement: Persisted Snapshot Schema Version Column

The `billing_period` table SHALL include a `snapshot_schema_version` column of type `INTEGER NOT NULL` with a column default of `1`. Every persisted billing period row SHALL carry a value in this column.

#### Scenario: Newly created billing period carries the current schema version

- **WHEN** the billing-period-report service persists a new billing period
- **THEN** the resulting `billing_period` row's `snapshot_schema_version` equals the value of the `CURRENT_SNAPSHOT_SCHEMA_VERSION` constant at the time of write

#### Scenario: Pre-existing billing periods are backfilled to version 1

- **WHEN** the schema migration that introduces the column is applied to a database that already contains `billing_period` rows
- **THEN** every pre-existing row's `snapshot_schema_version` is `1`

### Requirement: Single Source of Truth for the Current Schema Version

The system SHALL expose exactly one constant `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32` in the service implementation crate. All snapshot-writing code paths SHALL source the version they persist from this constant and SHALL NOT compute, derive, or override it on a per-call basis.

#### Scenario: Writer reads the constant verbatim

- **WHEN** any code path constructs a billing period for persistence
- **THEN** the value placed into the `snapshot_schema_version` field is the value of `CURRENT_SNAPSHOT_SCHEMA_VERSION` and not a parameter, configuration value, or derived expression

### Requirement: Snapshot Schema Version is Immutable Once Persisted

Once a `billing_period` row has been persisted, its `snapshot_schema_version` value SHALL NOT change for the lifetime of the row. Operations that mutate other fields of a billing period (including soft-delete) SHALL leave the version untouched.

#### Scenario: Soft-delete preserves the recorded version

- **WHEN** a billing period is soft-deleted via the existing delete operation
- **THEN** the row's `snapshot_schema_version` is identical to the value persisted at creation time

### Requirement: Read-Only Exposure in Transport Object

The `BillingPeriodTO` transport object SHALL include a `snapshot_schema_version` field. The field SHALL be present in the OpenAPI schema as a non-optional integer. The REST layer SHALL NOT accept a client-supplied value for this field on creation; it is read-only metadata.

#### Scenario: Get response exposes the persisted version

- **WHEN** a client issues `GET /billing_period/{id}` for an existing billing period
- **THEN** the response body contains a `snapshot_schema_version` field equal to the value persisted on that row

#### Scenario: Create endpoint ignores any client-supplied version

- **WHEN** a client issues `POST /billing_period` (regardless of whether the request body attempts to specify a version)
- **THEN** the resulting persisted row's `snapshot_schema_version` is `CURRENT_SNAPSHOT_SCHEMA_VERSION`, independent of any value present in the request body

### Requirement: Project Documentation Enforces the Bump Rule

The project SHALL maintain explicit, discoverable instructions in two locations directing future authors to increment `CURRENT_SNAPSHOT_SCHEMA_VERSION` whenever they add, remove, rename, or change the computation of any persisted `value_type` on `billing_period_sales_person`.

#### Scenario: CLAUDE.md contains the bump rule

- **WHEN** a reader inspects `shifty-backend/CLAUDE.md`
- **THEN** the file contains a section dedicated to billing period snapshot schema versioning that names `CURRENT_SNAPSHOT_SCHEMA_VERSION`, lists the conditions under which it must be incremented, and states the reason

#### Scenario: openspec config surfaces the bump rule in both context and rules

- **WHEN** a reader inspects `shifty-backend/openspec/config.yaml`
- **THEN** the `context:` block contains a sentence that names `CURRENT_SNAPSHOT_SCHEMA_VERSION` and states the bump condition
- **AND** the `rules.tasks` list contains an entry that reminds task authors to include a bump subtask when the change touches persisted `value_type`s or their computation
